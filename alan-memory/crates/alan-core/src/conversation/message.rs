use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::embedding::Embedding;
use crate::id::NodeId;

/// Message record (layer -1, DuckDB-stored).
/// A single message within a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: NodeId,
    /// ID of the parent conversation.
    pub conversation_id: NodeId,
    /// Role: "user", "assistant", "system", "tool".
    pub role: String,
    /// Message content text.
    pub content: String,
    /// Embedding for semantic search over messages.
    pub content_embedding: Option<Embedding>,
    /// Token count for this message.
    pub token_count: Option<i32>,
    /// Ordering within the conversation.
    pub message_index: i32,
    /// For branching conversations.
    pub parent_message_id: Option<NodeId>,
    pub created_at: DateTime<Utc>,
}

impl Message {
    pub fn new(
        conversation_id: NodeId,
        role: impl Into<String>,
        content: impl Into<String>,
        message_index: i32,
    ) -> Self {
        Self {
            id: NodeId::new(),
            conversation_id,
            role: role.into(),
            content: content.into(),
            content_embedding: None,
            token_count: None,
            message_index,
            parent_message_id: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_embedding(mut self, embedding: Embedding) -> Self {
        self.content_embedding = Some(embedding);
        self
    }

    pub fn with_token_count(mut self, count: i32) -> Self {
        self.token_count = Some(count);
        self
    }

    pub fn with_parent(mut self, parent_id: NodeId) -> Self {
        self.parent_message_id = Some(parent_id);
        self
    }
}
