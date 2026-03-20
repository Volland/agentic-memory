use std::sync::Arc;

use serde::Deserialize;
use tracing::{debug, warn};

use alan_core::entity::NodeType;
use alan_core::graph::wiring::BipartiteEdge;
use alan_core::relation::{
    AnyRelationNode, BecauseOf, Causes, EdgeNodeType, LeadsTo, Prevents,
};
use alan_core::NodeId;

use crate::context::CognitiveContext;
use crate::error::{CognitionError, Result};
use crate::traits::LlmBackend;
use crate::traits::llm as llm_util;

use super::prompt::causality;

/// LLM response for causality detection.
#[derive(Debug, Deserialize)]
struct CausalityItem {
    from_index: usize,
    to_index: usize,
    relation: String,
    probability: f64,
    strength: f64,
    #[serde(default)]
    mechanism: Option<String>,
}

/// Snapshot of a node needed for wiring.
struct NodeSnapshot {
    id: NodeId,
    node_type: NodeType,
    label: String,
}

/// Step that detects and wires causal relationships (LeadsTo, Causes, Prevents,
/// BecauseOf) between events using LLM reasoning.
pub struct CausalityStep {
    pub llm: Arc<dyn LlmBackend>,
}

impl CausalityStep {
    pub fn new(llm: Arc<dyn LlmBackend>) -> Self {
        Self { llm }
    }

    pub async fn execute(&self, ctx: &mut CognitiveContext) -> Result<()> {
        // Collect event/memory/fact node snapshots (owned, no borrows).
        let event_nodes: Vec<NodeSnapshot> = ctx
            .resolved_nodes
            .iter()
            .filter(|n| {
                matches!(
                    n.node_type(),
                    NodeType::Event | NodeType::Memory | NodeType::Fact
                )
            })
            .map(|n| NodeSnapshot {
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
            "Detecting causal relationships between events"
        );

        let prompt_events: Vec<(usize, &str)> = event_nodes
            .iter()
            .enumerate()
            .map(|(idx, snap)| (idx, snap.label.as_str()))
            .collect();

        let prompt = causality::causality_detection_prompt(&prompt_events);
        let system = causality::causality_detection_system();

        let items = match llm_util::complete_structured::<Vec<CausalityItem>>(
            self.llm.as_ref(),
            &prompt,
            Some(system),
        )
        .await
        {
            Ok(items) => items,
            Err(e) => {
                warn!(error = %e, "Causality detection LLM call failed");
                ctx.record_error(CognitionError::ProcessFailed {
                    process: "causality".into(),
                    message: e.to_string(),
                });
                return Ok(());
            }
        };

        debug!(count = items.len(), "Causal relationships detected");

        let mut new_edges: Vec<(BipartiteEdge, AnyRelationNode)> = Vec::new();
        let mut deferred_errors: Vec<CognitionError> = Vec::new();

        for item in items {
            if item.from_index >= event_nodes.len() || item.to_index >= event_nodes.len() {
                continue;
            }

            let from = &event_nodes[item.from_index];
            let to = &event_nodes[item.to_index];

            let label = format!("{}:{}:{}", item.relation, from.label, to.label);

            let (edge_type, rel_node) = match item.relation.as_str() {
                "leads_to" => {
                    let mut rel = LeadsTo::new(&label)
                        .with_probability(item.probability)
                        .with_strength(item.strength);
                    if let Some(ref mech) = item.mechanism {
                        rel = rel.with_mechanism(mech.clone());
                    }
                    (EdgeNodeType::LeadsTo, AnyRelationNode::LeadsTo(rel))
                }
                "causes" => {
                    let mut rel = Causes::new(&label)
                        .with_probability(item.probability)
                        .with_strength(item.strength);
                    if let Some(ref mech) = item.mechanism {
                        rel = rel.with_mechanism(mech.clone());
                    }
                    (EdgeNodeType::Causes, AnyRelationNode::Causes(rel))
                }
                "prevents" => {
                    let mut rel = Prevents::new(&label)
                        .with_probability(item.probability)
                        .with_strength(item.strength);
                    if let Some(ref mech) = item.mechanism {
                        rel = rel.with_mechanism(mech.clone());
                    }
                    (EdgeNodeType::Prevents, AnyRelationNode::Prevents(rel))
                }
                "because_of" => {
                    let mut rel = BecauseOf::new(&label)
                        .with_probability(item.probability)
                        .with_strength(item.strength);
                    if let Some(ref mech) = item.mechanism {
                        rel = rel.with_explanation(mech.clone());
                    }
                    (EdgeNodeType::BecauseOf, AnyRelationNode::BecauseOf(rel))
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
                    warn!(error = %e, "Invalid causal wiring");
                    deferred_errors.push(CognitionError::Core(e));
                }
            }
        }

        debug!(edges = new_edges.len(), "Causality wiring produced edges");
        ctx.resolved_edges.extend(new_edges);
        for err in deferred_errors {
            ctx.record_error(err);
        }

        Ok(())
    }
}
