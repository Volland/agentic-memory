use crate::entity::{AnyContentNode, NodeType};
use crate::error::Result;
use crate::id::NodeId;
use crate::relation::AnyRelationNode;

use super::wiring::BipartiteEdge;

/// Convenience builder for creating bipartite edges.
/// Handles the 3-step wiring pattern: from_node --> edge_node --> to_node.
pub struct EdgeBuilder {
    from_node: NodeId,
    from_type: NodeType,
}

impl EdgeBuilder {
    /// Start building an edge from a content node.
    pub fn from(node: &AnyContentNode) -> Self {
        Self {
            from_node: node.universal().id.clone(),
            from_type: node.node_type(),
        }
    }

    /// Start building an edge from a node ID and type.
    pub fn from_id(id: NodeId, node_type: NodeType) -> Self {
        Self {
            from_node: id,
            from_type: node_type,
        }
    }

    /// Wire through a relation node to a target content node.
    /// Validates the wiring against the ontology constraint matrix.
    pub fn through(
        self,
        relation: &AnyRelationNode,
        to_node: &AnyContentNode,
    ) -> Result<BipartiteEdge> {
        BipartiteEdge::new(
            self.from_node,
            self.from_type,
            relation.universal().id.clone(),
            relation.edge_type(),
            to_node.universal().id.clone(),
            to_node.node_type(),
        )
    }

    /// Wire through a relation node to a target node ID and type.
    pub fn through_to(
        self,
        relation: &AnyRelationNode,
        to_id: NodeId,
        to_type: NodeType,
    ) -> Result<BipartiteEdge> {
        BipartiteEdge::new(
            self.from_node,
            self.from_type,
            relation.universal().id.clone(),
            relation.edge_type(),
            to_id,
            to_type,
        )
    }
}

/// Batch builder for creating multiple edges at once.
pub struct BatchEdgeBuilder {
    edges: Vec<(BipartiteEdge, AnyRelationNode)>,
}

impl BatchEdgeBuilder {
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }

    /// Add a validated edge to the batch.
    pub fn add(
        mut self,
        edge: BipartiteEdge,
        relation: AnyRelationNode,
    ) -> Self {
        self.edges.push((edge, relation));
        self
    }

    /// Get all accumulated edges.
    pub fn build(self) -> Vec<(BipartiteEdge, AnyRelationNode)> {
        self.edges
    }
}

impl Default for BatchEdgeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
