use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::{EdgeNodeType, RelationNode};

/// Temporal overlap edge: source happened during target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct During {
    pub universal: UniversalColumns,
    pub overlap_type: String,
    pub confidence: f64,
}

impl During {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Relation),
            overlap_type: "full".to_string(),
            confidence: 1.0,
        }
    }

    pub fn with_overlap_type(mut self, overlap_type: impl Into<String>) -> Self {
        self.overlap_type = overlap_type.into();
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
}

impl RelationNode for During {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn edge_type(&self) -> EdgeNodeType {
        EdgeNodeType::During
    }
}
