use std::sync::Arc;

use tracing::debug;

use alan_core::entity::AnyContentNode;
use alan_core::graph::wiring::BipartiteEdge;
use alan_core::relation::{AnyRelationNode, EdgeNodeType, Similar};

use crate::context::CognitiveContext;
use crate::error::Result;
use crate::traits::EmbedderBackend;

/// Step that detects semantically similar nodes among the resolved content
/// nodes and creates Similar edges between them.
pub struct SimilarityDetectionStep {
    pub embedder: Arc<dyn EmbedderBackend>,
    /// Minimum cosine similarity to create a Similar edge.
    pub similarity_threshold: f64,
}

impl SimilarityDetectionStep {
    pub fn new(embedder: Arc<dyn EmbedderBackend>) -> Self {
        Self {
            embedder,
            similarity_threshold: 0.80,
        }
    }

    pub async fn execute(&self, ctx: &mut CognitiveContext) -> Result<()> {
        let nodes = &ctx.resolved_nodes;
        if nodes.len() < 2 {
            return Ok(());
        }

        debug!(
            count = nodes.len(),
            "Starting similarity detection among resolved nodes"
        );

        let mut new_edges: Vec<(BipartiteEdge, AnyRelationNode)> = Vec::new();

        // Compare all pairs of nodes that carry embeddings.
        for i in 0..nodes.len() {
            let emb_a = get_embedding(&nodes[i]);
            if emb_a.is_none() {
                continue;
            }
            let emb_a = emb_a.unwrap();
            let type_a = nodes[i].node_type();

            for j in (i + 1)..nodes.len() {
                let emb_b = get_embedding(&nodes[j]);
                if emb_b.is_none() {
                    continue;
                }
                let emb_b = emb_b.unwrap();
                let type_b = nodes[j].node_type();

                // Validate that Similar edges are allowed between these types.
                if alan_core::graph::wiring::validate_wiring(type_a, EdgeNodeType::Similar, type_b)
                    .is_err()
                {
                    continue;
                }

                let sim = emb_a.cosine_similarity(emb_b).unwrap_or(0.0) as f64;
                if sim < self.similarity_threshold {
                    continue;
                }

                let label = format!(
                    "similar:{}:{}",
                    nodes[i].universal().label,
                    nodes[j].universal().label,
                );
                let similar_rel = Similar::new(&label)
                    .with_similarity(sim)
                    .with_sim_method("cosine_embedding".to_string());
                let edge_node_id = similar_rel.universal.id.clone();

                let edge = BipartiteEdge::new_unchecked(
                    nodes[i].universal().id.clone(),
                    type_a,
                    edge_node_id,
                    EdgeNodeType::Similar,
                    nodes[j].universal().id.clone(),
                    type_b,
                );

                new_edges.push((edge, AnyRelationNode::Similar(similar_rel)));
            }
        }

        debug!(
            edges = new_edges.len(),
            "Similarity detection produced edges"
        );
        ctx.resolved_edges.extend(new_edges);

        Ok(())
    }
}

/// Extract the label embedding from any content node variant that carries one.
fn get_embedding(node: &AnyContentNode) -> Option<&alan_core::Embedding> {
    match node {
        AnyContentNode::Entity(e) => e.label_embedding.as_ref(),
        AnyContentNode::Fact(f) => f.label_embedding.as_ref(),
        AnyContentNode::Event(e) => e.label_embedding.as_ref(),
        AnyContentNode::Memory(m) => m.label_embedding.as_ref(),
        AnyContentNode::Time(_) | AnyContentNode::AbstractTime(_) => None,
    }
}
