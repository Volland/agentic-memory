use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::{EdgeNodeType, RelationNode};

/// Validity window start edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidFrom {
    pub universal: UniversalColumns,
    pub precision: String,
    pub confidence: f64,
}

impl ValidFrom {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Relation),
            precision: "exact".to_string(),
            confidence: 1.0,
        }
    }

    pub fn with_precision(mut self, precision: impl Into<String>) -> Self {
        self.precision = precision.into();
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
}

impl RelationNode for ValidFrom {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn edge_type(&self) -> EdgeNodeType {
        EdgeNodeType::ValidFrom
    }
}
