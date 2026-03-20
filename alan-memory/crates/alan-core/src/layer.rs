use serde::{Deserialize, Serialize};
use std::fmt;

/// Ontological layer in the memory graph.
/// Controls visibility and determines which abstraction level a node belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Layer {
    /// Raw conversational data (DuckDB-projected): Conversation, Message
    Conversation = -1,
    /// Edge/relation nodes: Contains, Source, Similar, etc.
    Relation = 0,
    /// Base entities and time nodes
    Entity = 1,
    /// Relational assertions with predicates
    Fact = 2,
    /// Temporally-bounded occurrences
    Event = 3,
    /// Life-significant experiences with emotional context
    Memory = 4,
}

impl Layer {
    /// Convert from i16 (database representation).
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            -1 => Some(Self::Conversation),
            0 => Some(Self::Relation),
            1 => Some(Self::Entity),
            2 => Some(Self::Fact),
            3 => Some(Self::Event),
            4 => Some(Self::Memory),
            _ => None,
        }
    }

    /// Convert to i16 for database storage.
    pub fn as_i16(self) -> i16 {
        self as i16
    }

    /// Whether this layer represents content nodes (carry embeddings).
    pub fn is_content_layer(self) -> bool {
        matches!(self, Self::Entity | Self::Fact | Self::Event | Self::Memory)
    }
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conversation => write!(f, "Conversation (-1)"),
            Self::Relation => write!(f, "Relation (0)"),
            Self::Entity => write!(f, "Entity (1)"),
            Self::Fact => write!(f, "Fact (2)"),
            Self::Event => write!(f, "Event (3)"),
            Self::Memory => write!(f, "Memory (4)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_ordering() {
        assert!(Layer::Conversation < Layer::Relation);
        assert!(Layer::Entity < Layer::Fact);
        assert!(Layer::Fact < Layer::Event);
        assert!(Layer::Event < Layer::Memory);
    }

    #[test]
    fn test_layer_roundtrip() {
        for layer in [
            Layer::Conversation,
            Layer::Relation,
            Layer::Entity,
            Layer::Fact,
            Layer::Event,
            Layer::Memory,
        ] {
            assert_eq!(Layer::from_i16(layer.as_i16()), Some(layer));
        }
    }

    #[test]
    fn test_content_layer() {
        assert!(!Layer::Conversation.is_content_layer());
        assert!(!Layer::Relation.is_content_layer());
        assert!(Layer::Entity.is_content_layer());
        assert!(Layer::Fact.is_content_layer());
        assert!(Layer::Event.is_content_layer());
        assert!(Layer::Memory.is_content_layer());
    }
}
