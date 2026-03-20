use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::{EdgeNodeType, RelationNode};

/// Temporal ordering edge: source happened before target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Before {
    pub universal: UniversalColumns,
    pub gap_duration: Option<String>,
    pub confidence: f64,
}

impl Before {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Relation),
            gap_duration: None,
            confidence: 1.0,
        }
    }

    pub fn with_gap_duration(mut self, gap_duration: impl Into<String>) -> Self {
        self.gap_duration = Some(gap_duration.into());
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
}

impl RelationNode for Before {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn edge_type(&self) -> EdgeNodeType {
        EdgeNodeType::Before
    }
}
