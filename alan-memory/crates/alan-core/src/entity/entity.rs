use serde::{Deserialize, Serialize};

use crate::embedding::Embedding;
use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::{NodeType, OntologyNode};

/// Base entity node (layer 1).
/// The most abstract concept — everything in the ontology inherits from Entity.
/// Extensible with domain-specific types (person, place, organization, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub universal: UniversalColumns,
    /// Embedding vector for semantic search.
    pub label_embedding: Option<Embedding>,
}

impl Entity {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Entity),
            label_embedding: None,
        }
    }

    pub fn with_embedding(mut self, embedding: Embedding) -> Self {
        self.label_embedding = Some(embedding);
        self
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.universal.context = Some(context.into());
        self
    }

    pub fn with_resolved(mut self, resolved: impl Into<String>) -> Self {
        self.universal.label_resolved = Some(resolved.into());
        self
    }
}

impl OntologyNode for Entity {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn node_type(&self) -> NodeType {
        NodeType::Entity
    }
}
