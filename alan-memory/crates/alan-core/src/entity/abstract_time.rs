use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::{NodeType, OntologyNode};

/// Abstract time node (layer 1).
/// Represents semantic temporal concepts like "future", "past", "soon", "never".
/// These are not calendar dates but meaningful temporal markers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractTime {
    pub universal: UniversalColumns,
    /// Semantic meaning of this abstract time (e.g., "unbounded forward", "recent past").
    pub semantics: String,
}

impl AbstractTime {
    pub fn new(label: impl Into<String>, semantics: impl Into<String>) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Entity),
            semantics: semantics.into(),
        }
    }

    /// Create the canonical "future" abstract time node.
    pub fn future() -> Self {
        Self::new("future", "unbounded forward")
    }

    /// Create the canonical "past" abstract time node.
    pub fn past() -> Self {
        Self::new("past", "unbounded backward")
    }

    /// Create the canonical "now" abstract time node.
    pub fn now() -> Self {
        Self::new("now", "current moment")
    }

    /// Create the canonical "soon" abstract time node.
    pub fn soon() -> Self {
        Self::new("soon", "near future")
    }

    /// Create the canonical "never" abstract time node.
    pub fn never() -> Self {
        Self::new("never", "will not occur")
    }
}

impl OntologyNode for AbstractTime {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn node_type(&self) -> NodeType {
        NodeType::AbstractTime
    }
}
