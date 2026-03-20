use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::{EdgeNodeType, RelationNode};

/// Causal edge indicating one node causes another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Causes {
    pub universal: UniversalColumns,
    pub probability: f64,
    pub strength: f64,
    pub mechanism: Option<String>,
    pub directness: String,
}

impl Causes {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Relation),
            probability: 1.0,
            strength: 1.0,
            mechanism: None,
            directness: "direct".to_string(),
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

    pub fn with_mechanism(mut self, mechanism: impl Into<String>) -> Self {
        self.mechanism = Some(mechanism.into());
        self
    }

    pub fn with_directness(mut self, directness: impl Into<String>) -> Self {
        self.directness = directness.into();
        self
    }
}

impl RelationNode for Causes {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn edge_type(&self) -> EdgeNodeType {
        EdgeNodeType::Causes
    }
}
