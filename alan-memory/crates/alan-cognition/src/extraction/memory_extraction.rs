use std::sync::Arc;

use serde::Deserialize;
use tracing::warn;

use crate::context::CognitiveContext;
use crate::error::Result;
use crate::extraction::output::ExtractedMemory;
use crate::extraction::prompt::memory::memory_extraction_prompt;
use crate::traits::llm::{self, LlmBackend};

/// Extraction step that identifies personally significant memories.
pub struct MemoryExtractionStep {
    pub llm: Arc<dyn LlmBackend>,
}

#[derive(Debug, Deserialize)]
struct LlmMemory {
    label: String,
    predicate: String,
    memory_type: Option<String>,
    significance: Option<String>,
    emotions: Option<Vec<String>>,
    intensity: Option<f64>,
    reflection: Option<String>,
    #[serde(default)]
    connected_events: Vec<String>,
    #[serde(default)]
    connected_entities: Vec<String>,
    source_fragment: Option<String>,
}

impl MemoryExtractionStep {
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
        let fact_labels: Vec<String> = ctx
            .extracted_facts
            .iter()
            .map(|f| f.label.clone())
            .collect();
        let event_labels: Vec<String> = ctx
            .extracted_events
            .iter()
            .map(|e| e.label.clone())
            .collect();

        let (system, user) =
            memory_extraction_prompt(&text, &entity_labels, &fact_labels, &event_labels);

        let llm_memories: Vec<LlmMemory> = match llm::complete_structured(
            self.llm.as_ref(),
            &user,
            Some(&system),
        )
        .await
        {
            Ok(memories) => memories,
            Err(e) => {
                warn!("LLM memory extraction failed: {e}");
                ctx.record_error(e);
                Vec::new()
            }
        };

        let extracted: Vec<ExtractedMemory> = llm_memories
            .into_iter()
            .map(|m| ExtractedMemory {
                label: m.label,
                predicate: m.predicate,
                memory_type: m.memory_type,
                significance: m.significance,
                emotions: m.emotions.unwrap_or_default(),
                intensity: m.intensity.unwrap_or(0.5),
                reflection: m.reflection,
                connected_events: m.connected_events,
                connected_entities: m.connected_entities,
                source_fragment: m.source_fragment.unwrap_or_default(),
                source_message_id: None,
            })
            .collect();

        ctx.extracted_memories.extend(extracted);
        Ok(())
    }
}
