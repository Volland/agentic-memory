use serde::{Deserialize, Serialize};

use alan_core::NodeId;

/// Intermediate extraction result for an entity (not yet resolved to an ontology node).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub label: String,
    pub entity_type: Option<String>,
    pub confidence: f64,
    pub source_fragment: String,
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
    pub certainty: f64,
    pub source_fragment: String,
    pub source_message_id: Option<NodeId>,
}

/// Intermediate extraction result for an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEvent {
    pub label: String,
    pub predicate: String,
    pub status: String,
    pub is_ongoing: bool,
    pub temporal_ref: Option<String>,
    pub source_fragment: String,
    pub source_message_id: Option<NodeId>,
}

/// Intermediate extraction result for a memory / significant experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedMemory {
    pub label: String,
    pub predicate: String,
    pub significance: Option<String>,
    pub emotions: Vec<String>,
    pub reflection: Option<String>,
    pub source_fragment: String,
    pub source_message_id: Option<NodeId>,
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
