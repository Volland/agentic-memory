pub mod entity_resolution;
pub mod fact_merging;
pub mod relation_wiring;
pub mod similarity_detection;
pub mod embedding_step;
pub mod consolidation_decision;
pub mod unified_process;
pub mod prompt;

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;

use crate::context::CognitiveContext;
use crate::error::Result;
use crate::traits::cognitive_process::{CognitiveProcess, ProcessKind, ProcessResult};
use crate::traits::embedder::EmbedderBackend;
use crate::traits::llm::LlmBackend;

use self::embedding_step::EmbeddingComputeStep;
use self::entity_resolution::EntityResolutionStep;
use self::fact_merging::FactMergingStep;
use self::relation_wiring::RelationWiringStep;
use self::similarity_detection::SimilarityDetectionStep;

/// Top-level consolidation process that resolves extracted knowledge into
/// deduplicated, wired, and embedded graph nodes.
///
/// Pipeline: entity_resolution -> fact_merging -> embedding -> relation_wiring -> similarity_detection
pub struct ConsolidationProcess {
    llm: Arc<dyn LlmBackend>,
    embedder: Arc<dyn EmbedderBackend>,
}

impl ConsolidationProcess {
    pub fn new(llm: Arc<dyn LlmBackend>, embedder: Arc<dyn EmbedderBackend>) -> Self {
        Self { llm, embedder }
    }
}

#[async_trait]
impl CognitiveProcess for ConsolidationProcess {
    fn name(&self) -> &str {
        "consolidation"
    }

    fn kind(&self) -> ProcessKind {
        ProcessKind::Reactive
    }

    async fn execute(&self, ctx: &mut CognitiveContext) -> Result<ProcessResult> {
        let start = Instant::now();
        let mut warnings: Vec<String> = Vec::new();

        // 1. Entity resolution — deduplicate extracted entities into AnyContentNode::Entity.
        let entity_step =
            EntityResolutionStep::new(Arc::clone(&self.llm), Arc::clone(&self.embedder));
        if let Err(e) = entity_step.execute(ctx).await {
            warnings.push(format!("Entity resolution error: {e}"));
        }

        // 2. Fact merging — detect contradictions, adjust certainty, build Fact nodes.
        let fact_step = FactMergingStep::new(Arc::clone(&self.llm));
        if let Err(e) = fact_step.execute(ctx).await {
            warnings.push(format!("Fact merging error: {e}"));
        }

        // 3. Build Event and Memory nodes from extracted items.
        convert_events_and_memories(ctx);

        // 4. Compute embeddings for any resolved node that is still missing one.
        let embed_step = EmbeddingComputeStep::new(Arc::clone(&self.embedder));
        if let Err(e) = embed_step.execute(ctx).await {
            warnings.push(format!("Embedding computation error: {e}"));
        }

        // 5. Relation wiring — create Source / Contains edges.
        let wiring_step = RelationWiringStep::new();
        if let Err(e) = wiring_step.execute(ctx).await {
            warnings.push(format!("Relation wiring error: {e}"));
        }

        // 6. Similarity detection — create Similar edges.
        let sim_step = SimilarityDetectionStep::new(Arc::clone(&self.embedder));
        if let Err(e) = sim_step.execute(ctx).await {
            warnings.push(format!("Similarity detection error: {e}"));
        }

        let items_produced = ctx.resolved_nodes.len() + ctx.resolved_edges.len();

        Ok(ProcessResult {
            process_name: self.name().to_string(),
            items_produced,
            duration: start.elapsed(),
            warnings,
        })
    }
}

/// Convert extracted events and memories into resolved content nodes.
fn convert_events_and_memories(ctx: &mut CognitiveContext) {
    use alan_core::entity::{AnyContentNode, Event, EventStatus, Memory};

    for extracted in &ctx.extracted_events {
        let status = match extracted.status.as_str() {
            "planned" => EventStatus::Planned,
            "cancelled" => EventStatus::Cancelled,
            "hypothetical" => EventStatus::Hypothetical,
            _ => EventStatus::Occurred,
        };
        let event = Event::new(&extracted.label, &extracted.predicate)
            .with_status(status)
            .with_ongoing(extracted.is_ongoing)
            .with_source("extraction".to_string());
        ctx.resolved_nodes.push(AnyContentNode::Event(event));
    }

    for extracted in &ctx.extracted_memories {
        let mut memory = Memory::new(&extracted.label, &extracted.predicate)
            .with_source("extraction".to_string())
            .with_emotions(extracted.emotions.clone());
        if let Some(ref sig) = extracted.significance {
            memory = memory.with_significance(sig.clone());
        }
        if let Some(ref refl) = extracted.reflection {
            memory = memory.with_reflection(refl.clone());
        }
        ctx.resolved_nodes.push(AnyContentNode::Memory(memory));
    }
}
