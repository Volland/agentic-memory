use chrono::Utc;
use tracing::debug;

use alan_core::entity::{AnyContentNode, NodeType};
use alan_core::NodeId;

use crate::context::CognitiveContext;
use crate::error::Result;

/// Configuration for relevance scoring weights.
pub struct RelevanceScoringConfig {
    /// Weight for recency component (0.0 - 1.0).
    pub recency_weight: f64,
    /// Weight for connection density component (0.0 - 1.0).
    pub density_weight: f64,
    /// Weight for layer importance component (0.0 - 1.0).
    pub layer_weight: f64,
    /// Half-life in hours for recency decay.
    pub recency_half_life_hours: f64,
}

impl Default for RelevanceScoringConfig {
    fn default() -> Self {
        Self {
            recency_weight: 0.4,
            density_weight: 0.3,
            layer_weight: 0.3,
            recency_half_life_hours: 168.0, // 1 week
        }
    }
}

/// Step that scores all resolved nodes by relevance, using a combination
/// of recency, connection density, and layer weight.
pub struct RelevanceScoringStep {
    pub config: RelevanceScoringConfig,
}

impl RelevanceScoringStep {
    pub fn new() -> Self {
        Self {
            config: RelevanceScoringConfig::default(),
        }
    }

    pub fn with_config(config: RelevanceScoringConfig) -> Self {
        Self { config }
    }

    pub async fn execute(&self, ctx: &CognitiveContext) -> Result<Vec<(NodeId, f64)>> {
        let now = Utc::now();
        let mut scores: Vec<(NodeId, f64)> = Vec::new();

        for node in &ctx.resolved_nodes {
            let node_id = node.universal().id.clone();

            // 1. Recency score: exponential decay based on time since learned_at.
            let recency = if let Some(learned_at) = node.universal().learned_at {
                let hours_ago = (now - learned_at).num_hours().max(0) as f64;
                (-hours_ago.ln().max(0.0) / self.config.recency_half_life_hours).exp()
            } else {
                0.5 // Default if no learned_at
            };

            // 2. Connection density: count edges involving this node.
            let density = count_edges_for_node(&node_id, ctx) as f64;
            // Normalize density using a sigmoid-like function.
            let density_score = 1.0 - (1.0 / (1.0 + density));

            // 3. Layer weight: memories > events > facts > entities.
            let layer_score = layer_importance(node);

            let relevance = self.config.recency_weight * recency
                + self.config.density_weight * density_score
                + self.config.layer_weight * layer_score;

            scores.push((node_id, relevance.clamp(0.0, 1.0)));
        }

        debug!(count = scores.len(), "Relevance scores computed");
        Ok(scores)
    }
}

/// Count edges that reference a given node ID (as from_node or to_node).
fn count_edges_for_node(node_id: &NodeId, ctx: &CognitiveContext) -> usize {
    ctx.resolved_edges
        .iter()
        .filter(|(edge, _)| edge.from_node == *node_id || edge.to_node == *node_id)
        .count()
}

/// Return a 0.0 - 1.0 importance weight based on the node's ontological layer.
fn layer_importance(node: &AnyContentNode) -> f64 {
    match node.node_type() {
        NodeType::Memory => 1.0,
        NodeType::Event => 0.75,
        NodeType::Fact => 0.5,
        NodeType::Entity => 0.25,
        NodeType::Time | NodeType::AbstractTime => 0.1,
        NodeType::Conversation | NodeType::Message => 0.05,
    }
}
