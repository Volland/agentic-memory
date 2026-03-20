use serde::{Deserialize, Serialize};

use crate::embedding::Embedding;
use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::{NodeType, OntologyNode};

/// Status of an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    /// The event has occurred.
    Occurred,
    /// The event is planned but hasn't happened yet.
    Planned,
    /// The event was cancelled.
    Cancelled,
    /// The event is hypothetical / might have happened.
    Hypothetical,
}

impl Default for EventStatus {
    fn default() -> Self {
        Self::Occurred
    }
}

/// Event node (layer 3).
/// A specialized fact that represents a temporally-bounded occurrence.
/// Inherits all fact properties plus status and ongoing tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub universal: UniversalColumns,
    /// Embedding vector for semantic search.
    pub label_embedding: Option<Embedding>,
    /// The predicate (e.g., "relocation", "graduation").
    pub predicate: String,
    /// Confidence level 0.0–1.0.
    pub certainty: f64,
    /// Origin of this event.
    pub source: Option<String>,
    /// Current status of the event.
    pub status: EventStatus,
    /// Whether this event is still ongoing.
    pub is_ongoing: bool,
}

impl Event {
    pub fn new(label: impl Into<String>, predicate: impl Into<String>) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Event),
            label_embedding: None,
            predicate: predicate.into(),
            certainty: 1.0,
            source: None,
            status: EventStatus::Occurred,
            is_ongoing: false,
        }
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

impl OntologyNode for Event {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn node_type(&self) -> NodeType {
        NodeType::Event
    }
}
