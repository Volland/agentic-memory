use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::NodeId;

/// Conversation record (layer -1, DuckDB-stored).
/// Represents a complete conversation session between user and assistant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: NodeId,
    pub title: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    /// Participant type: "user", "assistant", "pair", "group".
    pub participant: Option<String>,
    /// LLM model identifier used in this conversation.
    pub model: Option<String>,
    /// Summary of the conversation.
    pub summary: Option<String>,
    /// Tags for categorization.
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Conversation {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: NodeId::new(),
            title: None,
            started_at: now,
            ended_at: None,
            participant: None,
            model: None,
            summary: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn end_conversation(&mut self) {
        self.ended_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Self::new()
    }
}
