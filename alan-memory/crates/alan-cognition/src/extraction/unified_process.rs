use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;

use crate::context::CognitiveContext;
use crate::error::Result;
use crate::traits::cognitive_process::{CognitiveProcess, ProcessKind, ProcessResult};
use crate::traits::llm::LlmBackend;
use crate::traits::ner::NerBackend;

use super::unified_extraction::UnifiedExtractionStep;

/// Unified extraction process: single LLM call extracts entities, facts, events,
/// memories, relations, and temporal references all at once.
///
/// This replaces the sequential 4-step ExtractionProcess with a single-shot approach
/// that gives the LLM full context for better classification decisions.
///
/// Total LLM calls: 1 (down from 4)
pub struct UnifiedExtractionProcess {
    llm: Arc<dyn LlmBackend>,
    ner: Option<Arc<dyn NerBackend>>,
}

impl UnifiedExtractionProcess {
    pub fn new(
        llm: Arc<dyn LlmBackend>,
        ner: Option<Arc<dyn NerBackend>>,
    ) -> Self {
        Self { llm, ner }
    }
}

#[async_trait]
impl CognitiveProcess for UnifiedExtractionProcess {
    fn name(&self) -> &str {
        "unified_extraction"
    }

    fn kind(&self) -> ProcessKind {
        ProcessKind::Reactive
    }

    async fn execute(&self, ctx: &mut CognitiveContext) -> Result<ProcessResult> {
        let start = Instant::now();
        let mut warnings: Vec<String> = Vec::new();

        let step = UnifiedExtractionStep::new(Arc::clone(&self.llm), self.ner.clone());
        if let Err(e) = step.execute(ctx).await {
            warnings.push(format!("Unified extraction error: {e}"));
        }

        let items_produced = ctx.extracted_entities.len()
            + ctx.extracted_facts.len()
            + ctx.extracted_events.len()
            + ctx.extracted_memories.len()
            + ctx.extracted_relations.len()
            + ctx.extracted_temporal_refs.len();

        Ok(ProcessResult {
            process_name: self.name().to_string(),
            items_produced,
            duration: start.elapsed(),
            warnings,
        })
    }
}
