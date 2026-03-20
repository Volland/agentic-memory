use std::sync::Arc;

use serde::Deserialize;
use tracing::{debug, warn};

use alan_core::entity::NodeType;
use alan_core::graph::wiring::BipartiteEdge;
use alan_core::relation::{After, AnyRelationNode, Before, During, EdgeNodeType};
use alan_core::NodeId;

use crate::context::CognitiveContext;
use crate::error::{CognitionError, Result};
use crate::traits::LlmBackend;
use crate::traits::llm as llm_util;

use super::prompt::ordering;

/// LLM response for ordering detection.
#[derive(Debug, Deserialize)]
struct OrderingItem {
    from_index: usize,
    to_index: usize,
    relation: String,
    confidence: f64,
    #[serde(default)]
    gap_description: Option<String>,
}

/// Snapshot of a node needed for wiring (avoids holding borrows into ctx).
struct NodeSnapshot {
    id: NodeId,
    node_type: NodeType,
    label: String,
}

/// Step that detects and wires Before / After / During relationships between
/// events and memories using LLM reasoning.
pub struct OrderingStep {
    pub llm: Arc<dyn LlmBackend>,
}

impl OrderingStep {
    pub fn new(llm: Arc<dyn LlmBackend>) -> Self {
        Self { llm }
    }

    pub async fn execute(&self, ctx: &mut CognitiveContext) -> Result<()> {
        // Collect event/memory node snapshots (owned data, no borrows into ctx).
        let event_nodes: Vec<NodeSnapshot> = ctx
            .resolved_nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| {
                matches!(n.node_type(), NodeType::Event | NodeType::Memory)
            })
            .map(|(_i, n)| NodeSnapshot {
                id: n.universal().id.clone(),
                node_type: n.node_type(),
                label: n.universal().label.clone(),
            })
            .collect();

        if event_nodes.len() < 2 {
            return Ok(());
        }

        debug!(
            events = event_nodes.len(),
            "Detecting temporal ordering between events"
        );

        // Build indexed list for the prompt.
        let prompt_events: Vec<(usize, &str)> = event_nodes
            .iter()
            .enumerate()
            .map(|(prompt_idx, snap)| (prompt_idx, snap.label.as_str()))
            .collect();

        let prompt = ordering::ordering_detection_prompt(&prompt_events);
        let system = ordering::ordering_detection_system();

        let items = match llm_util::complete_structured::<Vec<OrderingItem>>(
            self.llm.as_ref(),
            &prompt,
            Some(system),
        )
        .await
        {
            Ok(items) => items,
            Err(e) => {
                warn!(error = %e, "Ordering detection LLM call failed");
                ctx.record_error(CognitionError::ProcessFailed {
                    process: "ordering".into(),
                    message: e.to_string(),
                });
                return Ok(());
            }
        };

        debug!(count = items.len(), "Ordering relationships detected");

        let mut new_edges: Vec<(BipartiteEdge, AnyRelationNode)> = Vec::new();
        let mut deferred_errors: Vec<CognitionError> = Vec::new();

        for item in items {
            if item.from_index >= event_nodes.len() || item.to_index >= event_nodes.len() {
                continue;
            }

            let from = &event_nodes[item.from_index];
            let to = &event_nodes[item.to_index];

            let (edge_type, rel_node) = match item.relation.as_str() {
                "before" => {
                    let mut rel = Before::new(format!(
                        "before:{}:{}",
                        from.label, to.label,
                    ))
                    .with_confidence(item.confidence);
                    if let Some(ref gap) = item.gap_description {
                        rel = rel.with_gap_duration(gap.clone());
                    }
                    (EdgeNodeType::Before, AnyRelationNode::Before(rel))
                }
                "after" => {
                    let mut rel = After::new(format!(
                        "after:{}:{}",
                        from.label, to.label,
                    ))
                    .with_confidence(item.confidence);
                    if let Some(ref gap) = item.gap_description {
                        rel = rel.with_gap_duration(gap.clone());
                    }
                    (EdgeNodeType::After, AnyRelationNode::After(rel))
                }
                "during" => {
                    let rel = During::new(format!(
                        "during:{}:{}",
                        from.label, to.label,
                    ))
                    .with_confidence(item.confidence);
                    (EdgeNodeType::During, AnyRelationNode::During(rel))
                }
                _ => continue,
            };

            let edge_node_id = rel_node.universal().id.clone();

            match BipartiteEdge::new(
                from.id.clone(),
                from.node_type,
                edge_node_id,
                edge_type,
                to.id.clone(),
                to.node_type,
            ) {
                Ok(edge) => {
                    new_edges.push((edge, rel_node));
                }
                Err(e) => {
                    warn!(error = %e, "Invalid ordering wiring");
                    deferred_errors.push(CognitionError::Core(e));
                }
            }
        }

        debug!(edges = new_edges.len(), "Ordering wiring produced edges");
        ctx.resolved_edges.extend(new_edges);
        for err in deferred_errors {
            ctx.record_error(err);
        }

        Ok(())
    }
}
