use tracing::{debug, warn};

use alan_core::entity::NodeType;
use alan_core::graph::wiring::BipartiteEdge;
use alan_core::relation::{AnyRelationNode, Contains, Source};
use alan_core::relation::contains::ContainmentType;
use alan_core::relation::source::ExtractionMethod;

use crate::context::CognitiveContext;
use crate::error::{CognitionError, Result};

/// Step that creates Source and Contains edges connecting extracted knowledge
/// back to the source messages.
pub struct RelationWiringStep;

impl RelationWiringStep {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, ctx: &mut CognitiveContext) -> Result<()> {
        if ctx.resolved_nodes.is_empty() {
            return Ok(());
        }

        debug!(
            nodes = ctx.resolved_nodes.len(),
            messages = ctx.messages.len(),
            "Starting relation wiring"
        );

        // Collect first message ID as default source if individual message IDs
        // are not tracked on the resolved nodes.
        let default_msg_id = ctx.messages.first().map(|m| m.id.clone());

        let mut new_edges: Vec<(BipartiteEdge, AnyRelationNode)> = Vec::new();
        let mut deferred_errors: Vec<CognitionError> = Vec::new();

        for node in &ctx.resolved_nodes {
            let node_id = node.universal().id.clone();
            let node_type = node.node_type();

            // Source edges are only valid from Entity/Fact/Event/Memory → Message.
            if !matches!(
                node_type,
                NodeType::Entity | NodeType::Fact | NodeType::Event | NodeType::Memory
            ) {
                continue;
            }

            if let Some(ref msg_id) = default_msg_id {
                // Create a Source edge: knowledge_node --[Source]--> message
                let source_rel = Source::new(format!("source:{}", node.universal().label))
                    .with_extraction_method(ExtractionMethod::Llm)
                    .with_confidence(1.0);
                let source_id = source_rel.universal.id.clone();

                match BipartiteEdge::new(
                    node_id.clone(),
                    node_type,
                    source_id,
                    alan_core::relation::EdgeNodeType::Source,
                    msg_id.clone(),
                    NodeType::Message,
                ) {
                    Ok(edge) => {
                        new_edges.push((edge, AnyRelationNode::Source(source_rel)));
                    }
                    Err(e) => {
                        warn!(error = %e, "Invalid Source wiring, skipping");
                        deferred_errors.push(CognitionError::Core(e));
                    }
                }

                // Create a Contains edge: message --[Contains]--> knowledge_node
                let contains_rel = Contains::new(format!("contains:{}", node.universal().label))
                    .with_containment_type(ContainmentType::Composition);
                let contains_id = contains_rel.universal.id.clone();

                match BipartiteEdge::new(
                    msg_id.clone(),
                    NodeType::Message,
                    contains_id,
                    alan_core::relation::EdgeNodeType::Contains,
                    node_id.clone(),
                    node_type,
                ) {
                    Ok(edge) => {
                        new_edges.push((edge, AnyRelationNode::Contains(contains_rel)));
                    }
                    Err(e) => {
                        warn!(error = %e, "Invalid Contains wiring, skipping");
                        deferred_errors.push(CognitionError::Core(e));
                    }
                }
            }
        }

        debug!(edges = new_edges.len(), "Relation wiring produced edges");
        ctx.resolved_edges.extend(new_edges);
        for err in deferred_errors {
            ctx.record_error(err);
        }

        Ok(())
    }
}
