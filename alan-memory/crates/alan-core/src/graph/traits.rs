use std::ops::Range;

use async_trait::async_trait;

use crate::conversation::{Conversation, Message};
use crate::embedding::Embedding;
use crate::entity::{AnyContentNode, NodeType};
use crate::error::Result;
use crate::id::NodeId;
use crate::relation::AnyRelationNode;

use super::wiring::BipartiteEdge;
use crate::relation::EdgeNodeType;

/// Storage trait for content nodes (Entity, Time, Fact, Event, Memory, etc.).
#[async_trait]
pub trait NodeStore: Send + Sync {
    /// Create a new content node.
    async fn create_node(&self, node: &AnyContentNode) -> Result<NodeId>;

    /// Get a content node by ID.
    async fn get_node(&self, id: &NodeId) -> Result<Option<AnyContentNode>>;

    /// Update an existing content node.
    async fn update_node(&self, node: &AnyContentNode) -> Result<()>;

    /// Delete a content node (use sparingly — prefer expiration).
    async fn delete_node(&self, id: &NodeId) -> Result<()>;

    /// Find nodes by label (exact or fuzzy match).
    async fn find_by_label(
        &self,
        label: &str,
        node_type: Option<NodeType>,
    ) -> Result<Vec<AnyContentNode>>;

    /// Get all non-expired nodes of a given type.
    async fn list_by_type(&self, node_type: NodeType) -> Result<Vec<AnyContentNode>>;
}

/// Storage trait for relation (edge) nodes and bipartite wiring.
#[async_trait]
pub trait RelationStore: Send + Sync {
    /// Create a bipartite edge with its relation node.
    async fn create_edge(
        &self,
        edge: &BipartiteEdge,
        relation: &AnyRelationNode,
    ) -> Result<()>;

    /// Get all edges originating from a node.
    async fn get_edges_from(
        &self,
        node_id: &NodeId,
        edge_type: Option<EdgeNodeType>,
    ) -> Result<Vec<(BipartiteEdge, AnyRelationNode)>>;

    /// Get all edges targeting a node.
    async fn get_edges_to(
        &self,
        node_id: &NodeId,
        edge_type: Option<EdgeNodeType>,
    ) -> Result<Vec<(BipartiteEdge, AnyRelationNode)>>;

    /// Get a specific relation node by ID.
    async fn get_relation(&self, id: &NodeId) -> Result<Option<AnyRelationNode>>;

    /// Update a relation node.
    async fn update_relation(&self, relation: &AnyRelationNode) -> Result<()>;
}

/// Storage trait for conversations and messages (DuckDB layer).
#[async_trait]
pub trait ConversationStore: Send + Sync {
    /// Create a new conversation.
    async fn create_conversation(&self, conv: &Conversation) -> Result<()>;

    /// Append a message to a conversation.
    async fn append_message(&self, msg: &Message) -> Result<()>;

    /// Get a conversation by ID.
    async fn get_conversation(&self, id: &NodeId) -> Result<Option<Conversation>>;

    /// Get messages for a conversation, optionally within an index range.
    async fn get_messages(
        &self,
        conversation_id: &NodeId,
        range: Option<Range<usize>>,
    ) -> Result<Vec<Message>>;

    /// List recent conversations.
    async fn list_conversations(&self, limit: usize) -> Result<Vec<Conversation>>;
}

/// Storage trait for vector/semantic search.
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Search for similar content nodes by embedding.
    /// Returns nodes with their similarity scores, ordered by descending similarity.
    async fn search(
        &self,
        node_type: NodeType,
        query_embedding: &Embedding,
        top_k: usize,
    ) -> Result<Vec<(AnyContentNode, f32)>>;

    /// Search messages by content embedding.
    async fn search_messages(
        &self,
        query_embedding: &Embedding,
        top_k: usize,
    ) -> Result<Vec<(Message, f32)>>;
}
