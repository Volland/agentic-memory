use std::sync::Arc;

use serde::Deserialize;
use tracing::warn;

use crate::context::CognitiveContext;
use crate::error::Result;
use crate::extraction::output::ExtractedFact;
use crate::extraction::prompt::fact::fact_extraction_prompt;
use crate::traits::llm::{self, LlmBackend};

/// Extraction step that identifies facts (subject-predicate-object triples).
pub struct FactExtractionStep {
    pub llm: Arc<dyn LlmBackend>,
}

#[derive(Debug, Deserialize)]
struct LlmFact {
    label: String,
    predicate: String,
    subject_label: String,
    object_label: Option<String>,
    #[serde(default)]
    fact_type: Option<String>,
    certainty: Option<f64>,
    source_fragment: Option<String>,
}

impl FactExtractionStep {
    pub fn new(llm: Arc<dyn LlmBackend>) -> Self {
        Self { llm }
    }

    pub async fn execute(&self, ctx: &mut CognitiveContext) -> Result<()> {
        let text = ctx.full_text();
        if text.trim().is_empty() {
            return Ok(());
        }

        let entity_labels: Vec<String> = ctx
            .extracted_entities
            .iter()
            .map(|e| e.label.clone())
            .collect();

        let (system, user) = fact_extraction_prompt(&text, &entity_labels);

        let llm_facts: Vec<LlmFact> = match llm::complete_structured(
            self.llm.as_ref(),
            &user,
            Some(&system),
        )
        .await
        {
            Ok(facts) => facts,
            Err(e) => {
                warn!("LLM fact extraction failed: {e}");
                ctx.record_error(e);
                Vec::new()
            }
        };

        let extracted: Vec<ExtractedFact> = llm_facts
            .into_iter()
            .map(|f| ExtractedFact {
                label: f.label,
                predicate: f.predicate,
                subject_label: f.subject_label,
                object_label: f.object_label,
                fact_type: f.fact_type.unwrap_or_else(|| "encyclopedic".to_string()),
                certainty: f.certainty.unwrap_or(0.5),
                source_fragment: f.source_fragment.unwrap_or_default(),
                source_message_id: None,
            })
            .collect();

        ctx.extracted_facts.extend(extracted);
        Ok(())
    }
}
