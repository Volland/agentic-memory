use serde::{Deserialize, Serialize};
use std::fmt;
use ulid::Ulid;

/// Unique identifier for all nodes in the ontology graph.
/// Uses ULID for time-ordered, globally unique IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(String);

impl NodeId {
    /// Generate a new unique NodeId.
    pub fn new() -> Self {
        Self(Ulid::new().to_string())
    }

    /// Create a NodeId from an existing string (e.g., loaded from DB).
    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Create a prefixed NodeId (e.g., "e-berlin", "ev-moved").
    pub fn with_prefix(prefix: &str) -> Self {
        Self(format!("{}-{}", prefix, Ulid::new()))
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_generation() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_node_id_from_string() {
        let id = NodeId::from_string("e-berlin");
        assert_eq!(id.as_str(), "e-berlin");
    }

    #[test]
    fn test_node_id_serialization() {
        let id = NodeId::from_string("test-id");
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: NodeId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }
}
