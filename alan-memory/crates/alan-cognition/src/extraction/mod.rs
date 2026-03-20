pub mod output;
pub mod entity_extraction;
pub mod fact_extraction;
pub mod event_extraction;
pub mod memory_extraction;
pub mod unified_extraction;
pub mod unified_process;
pub mod prompt;

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;

use crate::context::CognitiveContext;
use crate::error::Result;
use crate::traits::classifier::ClassifierBackend;
use crate::traits::cognitive_process::{CognitiveProcess, ProcessKind, ProcessResult};
use crate::traits::llm::LlmBackend;
use crate::traits::ner::NerBackend;

use self::entity_extraction::EntityExtractionStep;
use self::event_extraction::EventExtractionStep;
use self::fact_extraction::FactExtractionStep;
use self::memory_extraction::MemoryExtractionStep;

/// Top-level extraction process that chains entity, fact, event, and memory
/// extraction steps in sequence.
pub struct ExtractionProcess {
    llm: Arc<dyn LlmBackend>,
    ner: Option<Arc<dyn NerBackend>>,
    classifier: Option<Arc<dyn ClassifierBackend>>,
}

impl ExtractionProcess {
    pub fn new(
        llm: Arc<dyn LlmBackend>,
        ner: Option<Arc<dyn NerBackend>>,
        classifier: Option<Arc<dyn ClassifierBackend>>,
    ) -> Self {
        Self {
            llm,
            ner,
            classifier,
        }
    }
}

#[async_trait]
impl CognitiveProcess for ExtractionProcess {
    fn name(&self) -> &str {
        "extraction"
    }

    fn kind(&self) -> ProcessKind {
        ProcessKind::Reactive
    }

    async fn execute(&self, ctx: &mut CognitiveContext) -> Result<ProcessResult> {
        let start = Instant::now();
        let mut warnings: Vec<String> = Vec::new();

        // 1. Entity extraction
        let entity_step = EntityExtractionStep::new(Arc::clone(&self.llm), self.ner.clone());
        if let Err(e) = entity_step.execute(ctx).await {
            warnings.push(format!("Entity extraction error: {e}"));
        }

        // 2. Fact extraction (uses extracted entities)
        let fact_step = FactExtractionStep::new(Arc::clone(&self.llm));
        if let Err(e) = fact_step.execute(ctx).await {
            warnings.push(format!("Fact extraction error: {e}"));
        }

        // 3. Event extraction (uses extracted entities + facts)
        let event_step =
            EventExtractionStep::new(Arc::clone(&self.llm), self.classifier.clone());
        if let Err(e) = event_step.execute(ctx).await {
            warnings.push(format!("Event extraction error: {e}"));
        }

        // 4. Memory extraction (uses all prior extractions)
        let memory_step = MemoryExtractionStep::new(Arc::clone(&self.llm));
        if let Err(e) = memory_step.execute(ctx).await {
            warnings.push(format!("Memory extraction error: {e}"));
        }

        let items_produced = ctx.extracted_entities.len()
            + ctx.extracted_facts.len()
            + ctx.extracted_events.len()
            + ctx.extracted_memories.len();

        Ok(ProcessResult {
            process_name: self.name().to_string(),
            items_produced,
            duration: start.elapsed(),
            warnings,
        })
    }
}
