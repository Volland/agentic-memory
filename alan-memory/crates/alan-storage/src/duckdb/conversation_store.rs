use std::collections::HashMap;
use std::ops::Range;
use std::sync::Mutex;

use async_trait::async_trait;

use alan_core::conversation::{Conversation, Message};
use alan_core::graph::traits::ConversationStore;
use alan_core::id::NodeId;

use super::connection::DuckDbConnection;

/// In-memory implementation of ConversationStore backed by HashMaps.
/// Uses a DuckDbConnection placeholder for future real-DB migration.
pub struct DuckDbConversationStore {
    /// Placeholder connection (unused until real bindings land).
    #[allow(dead_code)]
    connection: DuckDbConnection,
    conversations: Mutex<HashMap<String, Conversation>>,
    messages: Mutex<HashMap<String, Vec<Message>>>,
}

impl DuckDbConversationStore {
    /// Create a new in-memory conversation store.
    pub fn new(connection: DuckDbConnection) -> Self {
        Self {
            connection,
            conversations: Mutex::new(HashMap::new()),
            messages: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ConversationStore for DuckDbConversationStore {
    async fn create_conversation(&self, conv: &Conversation) -> alan_core::Result<()> {
        let mut store = self
            .conversations
            .lock()
            .map_err(|e| alan_core::AlanError::Validation(format!("Lock poisoned: {e}")))?;
        let key = conv.id.as_str().to_string();
        if store.contains_key(&key) {
            return Err(alan_core::AlanError::Validation(format!(
                "Conversation already exists: {key}"
            )));
        }
        store.insert(key.clone(), conv.clone());
        // Ensure a message vec exists for this conversation.
        let mut msgs = self
            .messages
            .lock()
            .map_err(|e| alan_core::AlanError::Validation(format!("Lock poisoned: {e}")))?;
        msgs.entry(key).or_insert_with(Vec::new);
        Ok(())
    }

    async fn append_message(&self, msg: &Message) -> alan_core::Result<()> {
        let conv_key = msg.conversation_id.as_str().to_string();
        // Verify conversation exists.
        {
            let convs = self
                .conversations
                .lock()
                .map_err(|e| alan_core::AlanError::Validation(format!("Lock poisoned: {e}")))?;
            if !convs.contains_key(&conv_key) {
                return Err(alan_core::AlanError::NotFound(format!(
                    "Conversation not found: {conv_key}"
                )));
            }
        }
        let mut store = self
            .messages
            .lock()
            .map_err(|e| alan_core::AlanError::Validation(format!("Lock poisoned: {e}")))?;
        store
            .entry(conv_key)
            .or_insert_with(Vec::new)
            .push(msg.clone());
        Ok(())
    }

    async fn get_conversation(&self, id: &NodeId) -> alan_core::Result<Option<Conversation>> {
        let store = self
            .conversations
            .lock()
            .map_err(|e| alan_core::AlanError::Validation(format!("Lock poisoned: {e}")))?;
        Ok(store.get(id.as_str()).cloned())
    }

    async fn get_messages(
        &self,
        conversation_id: &NodeId,
        range: Option<Range<usize>>,
    ) -> alan_core::Result<Vec<Message>> {
        let store = self
            .messages
            .lock()
            .map_err(|e| alan_core::AlanError::Validation(format!("Lock poisoned: {e}")))?;
        let key = conversation_id.as_str();
        match store.get(key) {
            Some(msgs) => {
                let mut sorted = msgs.clone();
                sorted.sort_by_key(|m| m.message_index);
                match range {
                    Some(r) => {
                        let end = r.end.min(sorted.len());
                        let start = r.start.min(end);
                        Ok(sorted[start..end].to_vec())
                    }
                    None => Ok(sorted),
                }
            }
            None => Ok(Vec::new()),
        }
    }

    async fn list_conversations(&self, limit: usize) -> alan_core::Result<Vec<Conversation>> {
        let store = self
            .conversations
            .lock()
            .map_err(|e| alan_core::AlanError::Validation(format!("Lock poisoned: {e}")))?;
        let mut convs: Vec<Conversation> = store.values().cloned().collect();
        // Sort by started_at descending (most recent first).
        convs.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        convs.truncate(limit);
        Ok(convs)
    }
}
