use std::sync::Arc;

use serde::Deserialize;
use tracing::{debug, warn};

use alan_core::entity::{AnyContentNode, Entity};

use crate::context::CognitiveContext;
use crate::error::{CognitionError, Result};
use crate::traits::{EmbedderBackend, LlmBackend};
use crate::traits::llm as llm_util;

use super::prompt::resolution;

/// LLM response for entity resolution.
#[derive(Debug, Deserialize)]
struct ResolutionResponse {
    same_entity: bool,
    confidence: f64,
    #[allow(dead_code)]
    reasoning: String,
}

/// Step that deduplicates extracted entities by detecting when two labels
/// refer to the same real-world concept.
pub struct EntityResolutionStep {
    pub llm: Arc<dyn LlmBackend>,
    pub embedder: Arc<dyn EmbedderBackend>,
    /// Minimum cosine similarity to consider two entities as potential duplicates.
    pub similarity_threshold: f64,
}

impl EntityResolutionStep {
    pub fn new(llm: Arc<dyn LlmBackend>, embedder: Arc<dyn EmbedderBackend>) -> Self {
        Self {
            llm,
            embedder,
            similarity_threshold: 0.75,
        }
    }

    pub async fn execute(&self, ctx: &mut CognitiveContext) -> Result<()> {
        if ctx.extracted_entities.is_empty() {
            return Ok(());
        }

        debug!(
            count = ctx.extracted_entities.len(),
            "Starting entity resolution"
        );

        // Compute embeddings for all extracted entity labels.
        let labels: Vec<String> = ctx
            .extracted_entities
            .iter()
            .map(|e| e.label.clone())
            .collect();
        let embeddings = self.embedder.embed(&labels).await?;

        // Track which indices have been merged into another entity.
        let count = ctx.extracted_entities.len();
        let mut merged_into: Vec<Option<usize>> = vec![None; count];

        // Compare all pairs using embedding similarity, then confirm with LLM.
        for i in 0..count {
            if merged_into[i].is_some() {
                continue;
            }
            for j in (i + 1)..count {
                if merged_into[j].is_some() {
                    continue;
                }

                // Quick similarity gate.
                let sim = embeddings[i]
                    .cosine_similarity(&embeddings[j])
                    .unwrap_or(0.0);

                if (sim as f64) < self.similarity_threshold {
                    continue;
                }

                // Confirm with LLM.
                let prompt = resolution::entity_resolution_prompt(
                    &ctx.extracted_entities[i].label,
                    &ctx.extracted_entities[j].label,
                );
                let system = resolution::entity_resolution_system();

                match llm_util::complete_structured::<ResolutionResponse>(
                    self.llm.as_ref(),
                    &prompt,
                    Some(system),
                )
                .await
                {
                    Ok(resp) if resp.same_entity && resp.confidence > 0.6 => {
                        debug!(
                            a = %ctx.extracted_entities[i].label,
                            b = %ctx.extracted_entities[j].label,
                            confidence = resp.confidence,
                            "Merging duplicate entities"
                        );
                        merged_into[j] = Some(i);
                    }
                    Ok(_) => {}
                    Err(e) => {
                        warn!(error = %e, "Entity resolution LLM call failed, skipping pair");
                        ctx.record_error(CognitionError::ProcessFailed {
                            process: "entity_resolution".into(),
                            message: e.to_string(),
                        });
                    }
                }
            }
        }

        // Build resolved entity nodes, skipping merged duplicates.
        for (i, extracted) in ctx.extracted_entities.iter().enumerate() {
            if merged_into[i].is_some() {
                continue;
            }

            let mut entity = Entity::new(&extracted.label);
            if let Some(ref etype) = extracted.entity_type {
                entity = entity.with_context(etype.clone());
            }
            entity = entity.with_embedding(embeddings[i].clone());

            // Collect labels of any entities merged into this one.
            let aliases: Vec<&str> = merged_into
                .iter()
                .enumerate()
                .filter(|(_, target)| **target == Some(i))
                .map(|(j, _)| ctx.extracted_entities[j].label.as_str())
                .collect();

            if !aliases.is_empty() {
                let resolved_label =
                    format!("{} (aka {})", extracted.label, aliases.join(", "));
                entity = entity.with_resolved(resolved_label);
            }

            ctx.resolved_nodes.push(AnyContentNode::Entity(entity));
        }

        debug!(
            resolved = ctx.resolved_nodes.len(),
            "Entity resolution complete"
        );
        Ok(())
    }
}
