use serde::{Deserialize, Serialize};

use crate::embedding::Embedding;
use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::event::EventStatus;
use super::{NodeType, OntologyNode};

/// Memory node (layer 4).
/// The pinnacle of the ontology — a life-significant experience worth preserving.
/// Inherits from Event (and thus from Fact and Entity).
/// Carries emotional context, significance, and personal reflection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub universal: UniversalColumns,
    /// Embedding vector for semantic search.
    pub label_embedding: Option<Embedding>,
    /// The predicate (e.g., "life_chapter", "milestone").
    pub predicate: String,
    /// Confidence level 0.0–1.0.
    pub certainty: f64,
    /// Origin of this memory.
    pub source: Option<String>,
    /// Current status.
    pub status: EventStatus,
    /// Whether this experience is still ongoing.
    pub is_ongoing: bool,
    /// Why this memory matters (e.g., "Major life transition").
    pub significance: Option<String>,
    /// Emotional context (e.g., ["excitement", "hope", "anxiety"]).
    pub emotions: Vec<String>,
    /// Personal reflection on this experience.
    pub reflection: Option<String>,
}

impl Memory {
    pub fn new(label: impl Into<String>, predicate: impl Into<String>) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Memory),
            label_embedding: None,
            predicate: predicate.into(),
            certainty: 1.0,
            source: None,
            status: EventStatus::Occurred,
            is_ongoing: false,
            significance: None,
            emotions: Vec::new(),
            reflection: None,
        }
    }

    pub fn with_significance(mut self, significance: impl Into<String>) -> Self {
        self.significance = Some(significance.into());
        self
    }

    pub fn with_emotions(mut self, emotions: Vec<String>) -> Self {
        self.emotions = emotions;
        self
    }

    pub fn with_reflection(mut self, reflection: impl Into<String>) -> Self {
        self.reflection = Some(reflection.into());
        self
    }

    pub fn with_status(mut self, status: EventStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_ongoing(mut self, ongoing: bool) -> Self {
        self.is_ongoing = ongoing;
        self
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
}

impl OntologyNode for Memory {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn node_type(&self) -> NodeType {
        NodeType::Memory
    }
}
