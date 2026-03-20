use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;

use crate::context::CognitiveContext;
use crate::error::Result;
use crate::traits::cognitive_process::{CognitiveProcess, ProcessKind, ProcessResult};
use crate::traits::embedder::EmbedderBackend;
use crate::traits::llm::LlmBackend;

use super::consolidation_decision::ConsolidationDecisionStep;
use super::embedding_step::EmbeddingComputeStep;
use super::relation_wiring::RelationWiringStep;
use super::similarity_detection::SimilarityDetectionStep;

/// Unified consolidation process that uses a single LLM call for all
/// consolidation decisions (CREATE / MERGE / SUPERSEDE / REINFORCE / SKIP),
/// followed by embedding, relation wiring, and similarity detection.
///
/// Pipeline:
///   1. Consolidation Decision (1 LLM call) — decides action per item
///   2. Embedding Computation — batch embed all new/updated nodes
///   3. Relation Wiring — Source/Contains edges + Supersedes edges
///   4. Similarity Detection — cosine similarity edges
///
/// Total LLM calls: 1 (down from N pairwise comparisons)
pub struct UnifiedConsolidationProcess {
    llm: Arc<dyn LlmBackend>,
    embedder: Arc<dyn EmbedderBackend>,
}

impl UnifiedConsolidationProcess {
    pub fn new(llm: Arc<dyn LlmBackend>, embedder: Arc<dyn EmbedderBackend>) -> Self {
        Self { llm, embedder }
    }
}

#[async_trait]
impl CognitiveProcess for UnifiedConsolidationProcess {
    fn name(&self) -> &str {
        "unified_consolidation"
    }

    fn kind(&self) -> ProcessKind {
        ProcessKind::Reactive
    }

    async fn execute(&self, ctx: &mut CognitiveContext) -> Result<ProcessResult> {
        let start = Instant::now();
        let mut warnings: Vec<String> = Vec::new();

        // 1. Consolidation decisions — CREATE/MERGE/SUPERSEDE/REINFORCE/SKIP
        let decision_step = ConsolidationDecisionStep::new(
            Arc::clone(&self.llm),
            Arc::clone(&self.embedder),
        );
        if let Err(e) = decision_step.execute(ctx).await {
            warnings.push(format!("Consolidation decision error: {e}"));
        }

        // 2. Compute embeddings for any resolved node missing one.
        let embed_step = EmbeddingComputeStep::new(Arc::clone(&self.embedder));
        if let Err(e) = embed_step.execute(ctx).await {
            warnings.push(format!("Embedding computation error: {e}"));
        }

        // 3. Relation wiring — Source/Contains edges.
        let wiring_step = RelationWiringStep::new();
        if let Err(e) = wiring_step.execute(ctx).await {
            warnings.push(format!("Relation wiring error: {e}"));
        }

        // 4. Similarity detection.
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
