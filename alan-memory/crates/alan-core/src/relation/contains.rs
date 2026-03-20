use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::{EdgeNodeType, RelationNode};

/// How one node contains another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainmentType {
    Composition,
    Membership,
    Aggregation,
    PartOf,
}

/// Containment relation between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contains {
    pub universal: UniversalColumns,
    pub containment_type: ContainmentType,
    pub weight: f64,
}

impl Contains {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Relation),
            containment_type: ContainmentType::Composition,
            weight: 1.0,
        }
    }

    pub fn with_containment_type(mut self, containment_type: ContainmentType) -> Self {
        self.containment_type = containment_type;
        self
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }
}

impl RelationNode for Contains {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn edge_type(&self) -> EdgeNodeType {
        EdgeNodeType::Contains
    }
}
