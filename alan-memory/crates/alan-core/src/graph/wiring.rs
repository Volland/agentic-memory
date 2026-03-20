use serde::{Deserialize, Serialize};

use crate::entity::NodeType;
use crate::error::AlanError;
use crate::id::NodeId;
use crate::relation::EdgeNodeType;

/// A bipartite edge: from_node --[edge_node]--> to_node.
/// In the bipartite graph, entities never connect directly.
/// All relationships flow through a dedicated edge node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BipartiteEdge {
    /// The source content node.
    pub from_node: NodeId,
    /// The source node type (for wiring validation).
    pub from_type: NodeType,
    /// The relation/edge node sitting between source and target.
    pub edge_node: NodeId,
    /// The edge node type.
    pub edge_type: EdgeNodeType,
    /// The target content node.
    pub to_node: NodeId,
    /// The target node type (for wiring validation).
    pub to_type: NodeType,
}

impl BipartiteEdge {
    /// Create a new bipartite edge, validating against the ontology wiring matrix.
    pub fn new(
        from_node: NodeId,
        from_type: NodeType,
        edge_node: NodeId,
        edge_type: EdgeNodeType,
        to_node: NodeId,
        to_type: NodeType,
    ) -> Result<Self, AlanError> {
        validate_wiring(from_type, edge_type, to_type)?;
        Ok(Self {
            from_node,
            from_type,
            edge_node,
            edge_type,
            to_node,
            to_type,
        })
    }

    /// Create without validation (for loading from DB where constraints are already enforced).
    pub fn new_unchecked(
        from_node: NodeId,
        from_type: NodeType,
        edge_node: NodeId,
        edge_type: EdgeNodeType,
        to_node: NodeId,
        to_type: NodeType,
    ) -> Self {
        Self {
            from_node,
            from_type,
            edge_node,
            edge_type,
            to_node,
            to_type,
        }
    }
}

/// Validate that a proposed wiring is ontologically valid.
/// Based on the wiring constraint matrix from the LadybugDB ontology.
pub fn validate_wiring(
    from_type: NodeType,
    edge_type: EdgeNodeType,
    to_type: NodeType,
) -> Result<(), AlanError> {
    use EdgeNodeType::*;
    use NodeType::*;

    let valid = match edge_type {
        Contains => {
            matches!(
                from_type,
                Conversation | Message | Memory | Event | Fact | Time
            ) && matches!(
                to_type,
                Message | Memory | Event | Fact | Entity | Time
            )
        }
        Source => {
            matches!(from_type, Entity | Fact | Event | Memory)
                && matches!(to_type, Message)
        }
        LeadsTo => {
            matches!(from_type, Event | Memory | Fact)
                && matches!(to_type, Event | Memory | Fact | AbstractTime)
        }
        Prevents => {
            matches!(from_type, Event | Fact)
                && matches!(to_type, Event | Fact | Memory)
        }
        Causes => {
            matches!(from_type, Event | Fact)
                && matches!(to_type, Event | Fact | Memory)
        }
        BecauseOf => {
            matches!(from_type, Event | Fact | Memory)
                && matches!(to_type, Event | Fact)
        }
        Similar => {
            matches!(from_type, Entity | Fact | Event | Memory)
                && matches!(to_type, Entity | Fact | Event | Memory)
        }
        HasProperty => {
            matches!(from_type, Entity | Fact | Event | Memory)
                && matches!(to_type, Entity)
        }
        Before => {
            matches!(from_type, Event | Memory | Time | AbstractTime)
                && matches!(to_type, Event | Memory | Time | AbstractTime)
        }
        After => {
            matches!(from_type, Event | Memory | Time | AbstractTime)
                && matches!(to_type, Event | Memory | Time | AbstractTime)
        }
        During => {
            matches!(from_type, Event | Memory)
                && matches!(to_type, Event | Memory | Time)
        }
        ValidFrom => {
            matches!(from_type, Fact | Event | Memory)
                && matches!(to_type, Time | AbstractTime)
        }
        ValidTo => {
            matches!(from_type, Fact | Event | Memory)
                && matches!(to_type, Time | AbstractTime)
        }
    };

    if valid {
        Ok(())
    } else {
        Err(AlanError::InvalidWiring {
            from_type,
            edge_type,
            to_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_wirings() {
        // Memory --[Contains]--> Event (valid)
        assert!(validate_wiring(NodeType::Memory, EdgeNodeType::Contains, NodeType::Event).is_ok());
        // Event --[ValidFrom]--> Time (valid)
        assert!(
            validate_wiring(NodeType::Event, EdgeNodeType::ValidFrom, NodeType::Time).is_ok()
        );
        // Fact --[Source]--> Message (valid)
        assert!(
            validate_wiring(NodeType::Fact, EdgeNodeType::Source, NodeType::Message).is_ok()
        );
        // Event --[Before]--> Event (valid)
        assert!(
            validate_wiring(NodeType::Event, EdgeNodeType::Before, NodeType::Event).is_ok()
        );
        // Event --[Causes]--> Memory (valid)
        assert!(
            validate_wiring(NodeType::Event, EdgeNodeType::Causes, NodeType::Memory).is_ok()
        );
    }

    #[test]
    fn test_invalid_wirings() {
        // Entity --[Causes]--> Entity (invalid: Causes only from Event/Fact)
        assert!(
            validate_wiring(NodeType::Entity, EdgeNodeType::Causes, NodeType::Entity).is_err()
        );
        // Message --[ValidFrom]--> Time (invalid: ValidFrom only from Fact/Event/Memory)
        assert!(
            validate_wiring(NodeType::Message, EdgeNodeType::ValidFrom, NodeType::Time).is_err()
        );
        // Entity --[During]--> Time (invalid: During only from Event/Memory)
        assert!(
            validate_wiring(NodeType::Entity, EdgeNodeType::During, NodeType::Time).is_err()
        );
        // Fact --[Source]--> Entity (invalid: Source target must be Message)
        assert!(
            validate_wiring(NodeType::Fact, EdgeNodeType::Source, NodeType::Entity).is_err()
        );
    }
}
