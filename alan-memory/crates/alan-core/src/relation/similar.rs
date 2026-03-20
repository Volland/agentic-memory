use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::{EdgeNodeType, RelationNode};

/// Similarity edge between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Similar {
    pub universal: UniversalColumns,
    pub similarity: f64,
    pub sim_context: Option<String>,
    pub sim_method: Option<String>,
}

impl Similar {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Relation),
            similarity: 0.0,
            sim_context: None,
            sim_method: None,
        }
    }

    pub fn with_similarity(mut self, similarity: f64) -> Self {
        self.similarity = similarity;
        self
    }

    pub fn with_sim_context(mut self, sim_context: impl Into<String>) -> Self {
        self.sim_context = Some(sim_context.into());
        self
    }

    pub fn with_sim_method(mut self, sim_method: impl Into<String>) -> Self {
        self.sim_method = Some(sim_method.into());
        self
    }
}

impl RelationNode for Similar {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn edge_type(&self) -> EdgeNodeType {
        EdgeNodeType::Similar
    }
}
