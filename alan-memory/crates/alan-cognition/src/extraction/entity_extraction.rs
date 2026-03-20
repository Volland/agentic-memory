use std::sync::Arc;

use serde::Deserialize;
use tracing::warn;

use crate::context::CognitiveContext;
use crate::error::Result;
use crate::extraction::output::ExtractedEntity;
use crate::extraction::prompt::entity::entity_extraction_prompt;
use crate::traits::llm::{self, LlmBackend};
use crate::traits::ner::NerBackend;

/// Extraction step that identifies entities in conversation text.
pub struct EntityExtractionStep {
    pub llm: Arc<dyn LlmBackend>,
    pub ner: Option<Arc<dyn NerBackend>>,
}

/// LLM output schema for a single entity.
#[derive(Debug, Deserialize)]
struct LlmEntity {
    label: String,
    entity_type: Option<String>,
    confidence: Option<f64>,
    source_fragment: Option<String>,
}

impl EntityExtractionStep {
    pub fn new(llm: Arc<dyn LlmBackend>, ner: Option<Arc<dyn NerBackend>>) -> Self {
        Self { llm, ner }
    }

    pub async fn execute(&self, ctx: &mut CognitiveContext) -> Result<()> {
        let text = ctx.full_text();
        if text.trim().is_empty() {
            return Ok(());
        }

        // ── Step 1: Optional NER pass ──────────────────────────────────
        let mut ner_entities: Vec<ExtractedEntity> = Vec::new();

        if let Some(ref ner) = self.ner {
            match ner.extract_entities(&text).await {
                Ok(spans) => {
                    for span in spans {
                        ner_entities.push(ExtractedEntity {
                            label: span.text.clone(),
                            entity_type: Some(span.entity_type.clone()),
                            confidence: span.confidence,
                            source_fragment: span.text,
                            source_message_id: None,
                            extraction_method: "ner".to_string(),
                        });
                    }
                }
                Err(e) => {
                    warn!("NER extraction failed, continuing with LLM only: {e}");
                    ctx.record_error(e);
                }
            }
        }

        // ── Step 2: LLM extraction ─────────────────────────────────────
        let (system, user) = entity_extraction_prompt(&text);
        let llm_entities: Vec<LlmEntity> = match llm::complete_structured(
            self.llm.as_ref(),
            &user,
            Some(&system),
        )
        .await
        {
            Ok(entities) => entities,
            Err(e) => {
                warn!("LLM entity extraction failed: {e}");
                ctx.record_error(e);
                Vec::new()
            }
        };

        let llm_extracted: Vec<ExtractedEntity> = llm_entities
            .into_iter()
            .map(|e| ExtractedEntity {
                label: e.label,
                entity_type: e.entity_type,
                confidence: e.confidence.unwrap_or(0.5),
                source_fragment: e.source_fragment.unwrap_or_default(),
                source_message_id: None,
                extraction_method: "llm".to_string(),
            })
            .collect();

        // ── Step 3: Merge NER + LLM, deduplicate by label ──────────────
        let mut seen = std::collections::HashSet::new();
        let mut merged = Vec::new();

        // Prefer LLM entities (richer labels), then add unseen NER entities.
        for entity in llm_extracted.into_iter().chain(ner_entities.into_iter()) {
            let key = entity.label.to_lowercase();
            if seen.insert(key) {
                merged.push(entity);
            }
        }

        ctx.extracted_entities.extend(merged);
        Ok(())
    }
}
