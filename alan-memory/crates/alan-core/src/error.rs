use crate::relation::EdgeNodeType;
use crate::entity::NodeType;

/// Core error type for the Alan Memory ontology.
#[derive(Debug, thiserror::Error)]
pub enum AlanError {
    #[error("Invalid wiring: {from_type:?} --[{edge_type:?}]--> {to_type:?} is not allowed by the ontology")]
    InvalidWiring {
        from_type: NodeType,
        edge_type: EdgeNodeType,
        to_type: NodeType,
    },

    #[error("Node not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid layer value: {0}")]
    InvalidLayer(i16),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Embedding error: {0}")]
    Embedding(#[from] crate::embedding::EmbeddingError),
}

/// Convenience Result type alias.
pub type Result<T> = std::result::Result<T, AlanError>;
