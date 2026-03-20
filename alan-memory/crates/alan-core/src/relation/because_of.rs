use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::{EdgeNodeType, RelationNode};

/// Causal edge indicating one node exists because of another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BecauseOf {
    pub universal: UniversalColumns,
    pub probability: f64,
    pub strength: f64,
    pub explanation: Option<String>,
}

impl BecauseOf {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Relation),
            probability: 1.0,
            strength: 1.0,
            explanation: None,
        }
    }

    pub fn with_probability(mut self, probability: f64) -> Self {
        self.probability = probability;
        self
    }

    pub fn with_strength(mut self, strength: f64) -> Self {
        self.strength = strength;
        self
    }

    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = Some(explanation.into());
        self
    }
}

impl RelationNode for BecauseOf {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn edge_type(&self) -> EdgeNodeType {
        EdgeNodeType::BecauseOf
    }
}
