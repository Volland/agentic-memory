use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::{EdgeNodeType, RelationNode};

/// Validity window end edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidTo {
    pub universal: UniversalColumns,
    pub precision: String,
    pub confidence: f64,
    pub termination: Option<String>,
}

impl ValidTo {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Relation),
            precision: "exact".to_string(),
            confidence: 1.0,
            termination: None,
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

    pub fn with_termination(mut self, termination: impl Into<String>) -> Self {
        self.termination = Some(termination.into());
        self
    }
}

impl RelationNode for ValidTo {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn edge_type(&self) -> EdgeNodeType {
        EdgeNodeType::ValidTo
    }
}
