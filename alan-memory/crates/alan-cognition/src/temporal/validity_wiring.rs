use tracing::{debug, warn};

use alan_core::entity::{AnyContentNode, NodeType};
use alan_core::graph::wiring::BipartiteEdge;
use alan_core::relation::{AnyRelationNode, EdgeNodeType, ValidFrom, ValidTo};
use alan_core::NodeId;

use crate::context::CognitiveContext;
use crate::error::{CognitionError, Result};

/// Step that wires ValidFrom / ValidTo edges between facts / events / memories
/// and their associated Time / AbstractTime nodes based on extracted temporal
/// references.
pub struct ValidityWiringStep;

impl ValidityWiringStep {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, ctx: &mut CognitiveContext) -> Result<()> {
        if ctx.extracted_temporal_refs.is_empty() {
            return Ok(());
        }

        debug!("Starting validity wiring");

        // Pre-collect wiring instructions to avoid borrowing ctx immutably
        // while also needing to call record_error.
        struct WiringInstruction {
            anchor_id: NodeId,
            anchor_type: NodeType,
            anchor_label: String,
            time_id: NodeId,
            time_type: NodeType,
            time_label: String,
            relation_type: String,
        }

        let mut instructions: Vec<WiringInstruction> = Vec::new();

        for tref in &ctx.extracted_temporal_refs {
            let anchor = ctx.resolved_nodes.iter().find(|n| {
                n.universal().label == tref.anchor_entity_label
                    || n.universal()
                        .label_resolved
                        .as_deref()
                        .map(|r| r.contains(&tref.anchor_entity_label))
                        .unwrap_or(false)
            });

            let time_node = ctx.resolved_nodes.iter().find(|n| {
                matches!(
                    n,
                    AnyContentNode::Time(_) | AnyContentNode::AbstractTime(_)
                ) && n.universal().label == tref.expression
            });

            let (anchor, time_node) = match (anchor, time_node) {
                (Some(a), Some(t)) => (a, t),
                _ => continue,
            };

            let anchor_type = anchor.node_type();
            if !matches!(
                anchor_type,
                NodeType::Fact | NodeType::Event | NodeType::Memory
            ) {
                continue;
            }

            instructions.push(WiringInstruction {
                anchor_id: anchor.universal().id.clone(),
                anchor_type,
                anchor_label: anchor.universal().label.clone(),
                time_id: time_node.universal().id.clone(),
                time_type: time_node.node_type(),
                time_label: time_node.universal().label.clone(),
                relation_type: tref.relation_type.clone(),
            });
        }

        let mut new_edges: Vec<(BipartiteEdge, AnyRelationNode)> = Vec::new();
        let mut deferred_errors: Vec<CognitionError> = Vec::new();

        for instr in instructions {
            match instr.relation_type.as_str() {
                "valid_from" | "at" => {
                    let rel = ValidFrom::new(format!(
                        "valid_from:{}:{}",
                        instr.anchor_label, instr.time_label
                    ));
                    let edge_id = rel.universal.id.clone();

                    match BipartiteEdge::new(
                        instr.anchor_id,
                        instr.anchor_type,
                        edge_id,
                        EdgeNodeType::ValidFrom,
                        instr.time_id,
                        instr.time_type,
                    ) {
                        Ok(edge) => {
                            new_edges.push((edge, AnyRelationNode::ValidFrom(rel)));
                        }
                        Err(e) => {
                            warn!(error = %e, "Invalid ValidFrom wiring");
                            deferred_errors.push(CognitionError::Core(e));
                        }
                    }
                }
                "valid_to" => {
                    let rel = ValidTo::new(format!(
                        "valid_to:{}:{}",
                        instr.anchor_label, instr.time_label
                    ));
                    let edge_id = rel.universal.id.clone();

                    match BipartiteEdge::new(
                        instr.anchor_id,
                        instr.anchor_type,
                        edge_id,
                        EdgeNodeType::ValidTo,
                        instr.time_id,
                        instr.time_type,
                    ) {
                        Ok(edge) => {
                            new_edges.push((edge, AnyRelationNode::ValidTo(rel)));
                        }
                        Err(e) => {
                            warn!(error = %e, "Invalid ValidTo wiring");
                            deferred_errors.push(CognitionError::Core(e));
                        }
                    }
                }
                _ => {
                    // "during", "before", "after" are handled by ordering step.
                }
            }
        }

        debug!(edges = new_edges.len(), "Validity wiring produced edges");
        ctx.resolved_edges.extend(new_edges);
        for err in deferred_errors {
            ctx.record_error(err);
        }

        Ok(())
    }
}
