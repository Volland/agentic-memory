use serde::{Deserialize, Serialize};

use alan_core::NodeId;

/// Intermediate extraction result for an entity (not yet resolved to an ontology node).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub label: String,
    pub entity_type: Option<String>,
    pub confidence: f64,
    pub source_fragment: String,
    /// Other names or forms used in the text for this entity.
    #[serde(default)]
    pub aliases: Vec<String>,
    pub source_message_id: Option<NodeId>,
    pub extraction_method: String,
}

/// Intermediate extraction result for a fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFact {
    pub label: String,
    pub predicate: String,
    pub subject_label: String,
    pub object_label: Option<String>,
    /// "semantic" for general truths, "encyclopedic" for specific data about entities.
    #[serde(default = "default_fact_type")]
    pub fact_type: String,
    pub certainty: f64,
    pub source_fragment: String,
    pub source_message_id: Option<NodeId>,
}

fn default_fact_type() -> String {
    "encyclopedic".to_string()
}

/// Participant in an event with a semantic role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventParticipant {
    pub label: String,
    /// One of: agent, patient, instrument, beneficiary, location.
    pub role: String,
}

/// Intermediate extraction result for an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEvent {
    pub label: String,
    pub predicate: String,
    /// One of: occurred, ongoing, planned, cancelled, hypothetical, negated, recurring.
    pub status: String,
    pub is_ongoing: bool,
    pub temporal_ref: Option<String>,
    /// Entities involved and their roles.
    #[serde(default)]
    pub participants: Vec<EventParticipant>,
    /// Brief causal context if mentioned.
    pub causal_hint: Option<String>,
    pub source_fragment: String,
    pub source_message_id: Option<NodeId>,
}

/// Intermediate extraction result for a memory / significant experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedMemory {
    pub label: String,
    pub predicate: String,
    /// One of: episodic, achievement, relational, transformative, difficult, aspirational.
    #[serde(default)]
    pub memory_type: Option<String>,
    pub significance: Option<String>,
    pub emotions: Vec<String>,
    /// Emotional intensity 0.0-1.0.
    #[serde(default = "default_intensity")]
    pub intensity: f64,
    pub reflection: Option<String>,
    /// Event labels that this memory encompasses.
    #[serde(default)]
    pub connected_events: Vec<String>,
    /// Entity labels involved in this memory.
    #[serde(default)]
    pub connected_entities: Vec<String>,
    pub source_fragment: String,
    pub source_message_id: Option<NodeId>,
}

fn default_intensity() -> f64 {
    0.5
}

/// A temporal reference extracted from text that should be linked to an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTemporalRef {
    pub expression: String,
    pub temporal_type: String,
    pub anchor_entity_label: String,
    pub relation_type: String,
    pub source_message_id: Option<NodeId>,
}

/// A relationship extracted between two entities (by label, not yet resolved).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRelation {
    pub from_label: String,
    pub to_label: String,
    pub edge_type: String,
    pub properties: serde_json::Value,
    pub confidence: f64,
    pub source_message_id: Option<NodeId>,
}

/// Result of the supersede check during consolidation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupersedeResult {
    /// The label of the new node that supersedes.
    pub new_label: String,
    /// The label of the old node being superseded.
    pub old_label: String,
    /// Why the old information is outdated.
    pub reason: String,
    /// Confidence that this is a true supersede relationship.
    pub confidence: f64,
}
