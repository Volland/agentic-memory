use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::{EdgeNodeType, RelationNode};

/// Supersede edge indicating that a newer node replaces an older one.
///
/// Used in the no-delete model: instead of removing outdated facts, events,
/// or memories, we create a Supersedes edge from the new node to the old node
/// and mark the old node with an expiration timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supersedes {
    pub universal: UniversalColumns,
    /// Why the old information is outdated or replaced.
    pub reason: String,
    /// Confidence that this is a genuine supersede relationship.
    pub confidence: f64,
}

impl Supersedes {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Relation),
            reason: String::new(),
            confidence: 1.0,
        }
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
}

impl RelationNode for Supersedes {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn edge_type(&self) -> EdgeNodeType {
        EdgeNodeType::Supersedes
    }
}
