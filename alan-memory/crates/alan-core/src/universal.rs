use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::NodeId;
use crate::layer::Layer;

/// Universal columns present on every node table in the ontology.
/// Both entity nodes and relation (edge) nodes carry these fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalColumns {
    /// Unique node identifier.
    pub id: NodeId,
    /// Raw label as extracted from source text.
    pub label: String,
    /// Canonicalized / disambiguated label.
    pub label_resolved: Option<String>,
    /// When the system first learned this knowledge.
    pub learned_at: Option<DateTime<Utc>>,
    /// When this knowledge expires (for forgetting — never delete, just expire).
    pub expire_at: Option<DateTime<Utc>>,
    /// Record creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp.
    pub updated_at: DateTime<Utc>,
    /// Ontological layer this node belongs to.
    pub layer: Layer,
    /// Free-text situational context.
    pub context: Option<String>,
}

impl UniversalColumns {
    /// Create new universal columns with sensible defaults.
    pub fn new(label: impl Into<String>, layer: Layer) -> Self {
        let now = Utc::now();
        Self {
            id: NodeId::new(),
            label: label.into(),
            label_resolved: None,
            learned_at: Some(now),
            expire_at: None,
            created_at: now,
            updated_at: now,
            layer,
            context: None,
        }
    }

    /// Create with a specific ID (e.g., loaded from DB).
    pub fn with_id(id: NodeId, label: impl Into<String>, layer: Layer) -> Self {
        let now = Utc::now();
        Self {
            id,
            label: label.into(),
            label_resolved: None,
            learned_at: Some(now),
            expire_at: None,
            created_at: now,
            updated_at: now,
            layer,
            context: None,
        }
    }

    /// Set the resolved label.
    pub fn set_resolved(&mut self, resolved: impl Into<String>) {
        self.label_resolved = Some(resolved.into());
        self.updated_at = Utc::now();
    }

    /// Set expiration (forgetting — never delete).
    pub fn set_expiration(&mut self, expire_at: DateTime<Utc>) {
        self.expire_at = Some(expire_at);
        self.updated_at = Utc::now();
    }

    /// Check if this node has expired.
    pub fn is_expired(&self) -> bool {
        self.expire_at
            .map(|exp| exp <= Utc::now())
            .unwrap_or(false)
    }

    /// Get the display label (resolved if available, otherwise raw).
    pub fn display_label(&self) -> &str {
        self.label_resolved.as_deref().unwrap_or(&self.label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_columns_creation() {
        let cols = UniversalColumns::new("Berlin", Layer::Entity);
        assert_eq!(cols.label, "Berlin");
        assert_eq!(cols.layer, Layer::Entity);
        assert!(cols.label_resolved.is_none());
        assert!(cols.expire_at.is_none());
        assert!(!cols.is_expired());
    }

    #[test]
    fn test_display_label() {
        let mut cols = UniversalColumns::new("Berlin", Layer::Entity);
        assert_eq!(cols.display_label(), "Berlin");

        cols.set_resolved("Berlin, Germany");
        assert_eq!(cols.display_label(), "Berlin, Germany");
    }

    #[test]
    fn test_serialization() {
        let cols = UniversalColumns::new("Test", Layer::Fact);
        let json = serde_json::to_string(&cols).unwrap();
        let deserialized: UniversalColumns = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.label, "Test");
        assert_eq!(deserialized.layer, Layer::Fact);
    }
}
