use std::sync::Arc;

use serde::Deserialize;
use tracing::{debug, warn};

use crate::context::CognitiveContext;
use crate::error::{CognitionError, Result};
use crate::extraction::output::ExtractedTemporalRef;
use crate::traits::LlmBackend;
use crate::traits::llm as llm_util;

use super::prompt::temporal;

/// LLM response for temporal extraction.
#[derive(Debug, Deserialize)]
struct TemporalExtractionItem {
    expression: String,
    temporal_type: String,
    anchor_entity_label: String,
    relation_type: String,
}

/// Step that extracts temporal expressions from message text and resolved node
/// labels, populating `ctx.extracted_temporal_refs`.
pub struct TemporalExtractionStep {
    pub llm: Arc<dyn LlmBackend>,
}

impl TemporalExtractionStep {
    pub fn new(llm: Arc<dyn LlmBackend>) -> Self {
        Self { llm }
    }

    pub async fn execute(&self, ctx: &mut CognitiveContext) -> Result<()> {
        let text = ctx.full_text();
        if text.is_empty() {
            return Ok(());
        }

        // Gather labels of known entities / events / facts for anchoring.
        let entity_labels: Vec<&str> = ctx
            .resolved_nodes
            .iter()
            .map(|n| n.universal().label.as_str())
            .collect();

        debug!("Starting temporal extraction");

        let prompt = temporal::temporal_extraction_prompt(&text, &entity_labels);
        let system = temporal::temporal_extraction_system();

        match llm_util::complete_structured::<Vec<TemporalExtractionItem>>(
            self.llm.as_ref(),
            &prompt,
            Some(system),
        )
        .await
        {
            Ok(items) => {
                debug!(count = items.len(), "Temporal expressions extracted");
                for item in items {
                    let source_msg_id = ctx.messages.first().map(|m| m.id.clone());
                    ctx.extracted_temporal_refs.push(ExtractedTemporalRef {
                        expression: item.expression,
                        temporal_type: item.temporal_type,
                        anchor_entity_label: item.anchor_entity_label,
                        relation_type: item.relation_type,
                        source_message_id: source_msg_id,
                    });
                }
            }
            Err(e) => {
                warn!(error = %e, "Temporal extraction LLM call failed");
                ctx.record_error(CognitionError::ProcessFailed {
                    process: "temporal_extraction".into(),
                    message: e.to_string(),
                });
            }
        }

        Ok(())
    }
}
