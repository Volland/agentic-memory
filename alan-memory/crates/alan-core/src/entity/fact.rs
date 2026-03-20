use serde::{Deserialize, Serialize};

use crate::embedding::Embedding;
use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::{NodeType, OntologyNode};

/// Fact node (layer 2).
/// A relational assertion with a predicate — simultaneously a node and a
/// representation of relationships between entities.
/// Facts form the core of semantic memory.
/// Facts can reference other facts (self-referential) to build narrative chains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub universal: UniversalColumns,
    /// Embedding vector for semantic search.
    pub label_embedding: Option<Embedding>,
    /// The predicate/relationship type (e.g., "is-capital-of", "works-at").
    pub predicate: String,
    /// Confidence level 0.0–1.0.
    pub certainty: f64,
    /// Origin of this fact (e.g., "user", "extraction", "inference").
    pub source: Option<String>,
}

impl Fact {
    pub fn new(label: impl Into<String>, predicate: impl Into<String>) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Fact),
            label_embedding: None,
            predicate: predicate.into(),
            certainty: 1.0,
            source: None,
        }
    }

    pub fn with_certainty(mut self, certainty: f64) -> Self {
        self.certainty = certainty.clamp(0.0, 1.0);
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_embedding(mut self, embedding: Embedding) -> Self {
        self.label_embedding = Some(embedding);
        self
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.universal.context = Some(context.into());
        self
    }

    /// Decay certainty by a factor (for the forgetting process).
    /// Never goes below 0.0.
    pub fn decay_certainty(&mut self, factor: f64) {
        self.certainty = (self.certainty * factor).max(0.0);
        self.universal.updated_at = chrono::Utc::now();
    }
}

impl OntologyNode for Fact {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn node_type(&self) -> NodeType {
        NodeType::Fact
    }
}
