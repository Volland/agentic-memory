use std::sync::Mutex;

use async_trait::async_trait;

use alan_core::conversation::Message;
use alan_core::embedding::Embedding;
use alan_core::entity::{AnyContentNode, NodeType};
use alan_core::graph::traits::VectorStore;

/// In-memory vector search implementation.
/// Uses brute-force cosine similarity over all indexed nodes.
pub struct InMemoryVectorStore {
    entries: Mutex<Vec<(AnyContentNode, Embedding)>>,
    messages: Mutex<Vec<(Message, Embedding)>>,
}

impl InMemoryVectorStore {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
            messages: Mutex::new(Vec::new()),
        }
    }

    /// Index a content node with its embedding for later search.
    pub fn index_node(&self, node: AnyContentNode, embedding: Embedding) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.push((node, embedding));
        }
    }

    /// Index a message with its embedding for later search.
    pub fn index_message(&self, message: Message, embedding: Embedding) {
        if let Ok(mut messages) = self.messages.lock() {
            messages.push((message, embedding));
        }
    }
}

impl Default for InMemoryVectorStore {
    fn default() -> Self {
        Self::new()
    }
}

fn lock_err(e: impl std::fmt::Display) -> alan_core::AlanError {
    alan_core::AlanError::Validation(format!("Lock poisoned: {e}"))
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn search(
        &self,
        node_type: NodeType,
        query_embedding: &Embedding,
        top_k: usize,
    ) -> alan_core::Result<Vec<(AnyContentNode, f32)>> {
        let entries = self.entries.lock().map_err(lock_err)?;

        let mut scored: Vec<(AnyContentNode, f32)> = entries
            .iter()
            .filter(|(node, _)| node.node_type() == node_type)
            .filter_map(|(node, emb)| {
                emb.cosine_similarity(query_embedding)
                    .ok()
                    .map(|score| (node.clone(), score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        Ok(scored)
    }

    async fn search_messages(
        &self,
        query_embedding: &Embedding,
        top_k: usize,
    ) -> alan_core::Result<Vec<(Message, f32)>> {
        let messages = self.messages.lock().map_err(lock_err)?;

        let mut scored: Vec<(Message, f32)> = messages
            .iter()
            .filter_map(|(msg, emb)| {
                emb.cosine_similarity(query_embedding)
                    .ok()
                    .map(|score| (msg.clone(), score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        Ok(scored)
    }
}
