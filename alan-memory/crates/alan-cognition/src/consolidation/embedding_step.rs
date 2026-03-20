use std::sync::Arc;

use tracing::debug;

use alan_core::entity::AnyContentNode;

use crate::context::CognitiveContext;
use crate::error::Result;
use crate::traits::EmbedderBackend;

/// Step that computes label embeddings for all resolved content nodes that
/// do not yet have one.
pub struct EmbeddingComputeStep {
    pub embedder: Arc<dyn EmbedderBackend>,
}

impl EmbeddingComputeStep {
    pub fn new(embedder: Arc<dyn EmbedderBackend>) -> Self {
        Self { embedder }
    }

    pub async fn execute(&self, ctx: &mut CognitiveContext) -> Result<()> {
        // Collect indices and labels for nodes missing an embedding.
        let needs_embedding: Vec<(usize, String)> = ctx
            .resolved_nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| {
                let has_embedding = match node {
                    AnyContentNode::Entity(e) => e.label_embedding.is_some(),
                    AnyContentNode::Fact(f) => f.label_embedding.is_some(),
                    AnyContentNode::Event(e) => e.label_embedding.is_some(),
                    AnyContentNode::Memory(m) => m.label_embedding.is_some(),
                    AnyContentNode::Time(_) | AnyContentNode::AbstractTime(_) => true,
                };
                if has_embedding {
                    None
                } else {
                    Some((idx, node.universal().label.clone()))
                }
            })
            .collect();

        if needs_embedding.is_empty() {
            return Ok(());
        }

        debug!(
            count = needs_embedding.len(),
            "Computing embeddings for resolved nodes"
        );

        let labels: Vec<String> = needs_embedding.iter().map(|(_, l)| l.clone()).collect();
        let embeddings = self.embedder.embed(&labels).await?;

        for ((idx, _), embedding) in needs_embedding.into_iter().zip(embeddings.into_iter()) {
            match &mut ctx.resolved_nodes[idx] {
                AnyContentNode::Entity(e) => {
                    e.label_embedding = Some(embedding);
                }
                AnyContentNode::Fact(f) => {
                    f.label_embedding = Some(embedding);
                }
                AnyContentNode::Event(e) => {
                    e.label_embedding = Some(embedding);
                }
                AnyContentNode::Memory(m) => {
                    m.label_embedding = Some(embedding);
                }
                AnyContentNode::Time(_) | AnyContentNode::AbstractTime(_) => {}
            }
        }

        debug!("Embedding computation complete");
        Ok(())
    }
}
