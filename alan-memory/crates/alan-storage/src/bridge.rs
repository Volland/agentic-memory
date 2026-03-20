use std::ops::Range;

use async_trait::async_trait;

use alan_core::conversation::{Conversation, Message};
use alan_core::embedding::Embedding;
use alan_core::entity::{AnyContentNode, NodeType};
use alan_core::graph::traits::{ConversationStore, NodeStore, RelationStore, VectorStore};
use alan_core::graph::wiring::BipartiteEdge;
use alan_core::id::NodeId;
use alan_core::relation::{AnyRelationNode, EdgeNodeType};

use crate::duckdb::DuckDbConversationStore;
use crate::ladybugdb::{InMemoryNodeStore, InMemoryRelationStore, InMemoryVectorStore};

/// Unified storage facade that delegates to the appropriate backend:
/// - Conversations/Messages -> DuckDbConversationStore
/// - Content nodes -> InMemoryNodeStore (LadybugDB placeholder)
/// - Relations/edges -> InMemoryRelationStore (LadybugDB placeholder)
/// - Vector search -> InMemoryVectorStore
pub struct DualStore {
    pub conversations: DuckDbConversationStore,
    pub nodes: InMemoryNodeStore,
    pub relations: InMemoryRelationStore,
    pub vectors: InMemoryVectorStore,
}

impl DualStore {
    pub fn new(
        conversations: DuckDbConversationStore,
        nodes: InMemoryNodeStore,
        relations: InMemoryRelationStore,
        vectors: InMemoryVectorStore,
    ) -> Self {
        Self {
            conversations,
            nodes,
            relations,
            vectors,
        }
    }
}

#[async_trait]
impl ConversationStore for DualStore {
    async fn create_conversation(&self, conv: &Conversation) -> alan_core::Result<()> {
        self.conversations.create_conversation(conv).await
    }

    async fn append_message(&self, msg: &Message) -> alan_core::Result<()> {
        self.conversations.append_message(msg).await
    }

    async fn get_conversation(&self, id: &NodeId) -> alan_core::Result<Option<Conversation>> {
        self.conversations.get_conversation(id).await
    }

    async fn get_messages(
        &self,
        conversation_id: &NodeId,
        range: Option<Range<usize>>,
    ) -> alan_core::Result<Vec<Message>> {
        self.conversations.get_messages(conversation_id, range).await
    }

    async fn list_conversations(&self, limit: usize) -> alan_core::Result<Vec<Conversation>> {
        self.conversations.list_conversations(limit).await
    }
}

#[async_trait]
impl NodeStore for DualStore {
    async fn create_node(&self, node: &AnyContentNode) -> alan_core::Result<NodeId> {
        self.nodes.create_node(node).await
    }

    async fn get_node(&self, id: &NodeId) -> alan_core::Result<Option<AnyContentNode>> {
        self.nodes.get_node(id).await
    }

    async fn update_node(&self, node: &AnyContentNode) -> alan_core::Result<()> {
        self.nodes.update_node(node).await
    }

    async fn delete_node(&self, id: &NodeId) -> alan_core::Result<()> {
        self.nodes.delete_node(id).await
    }

    async fn find_by_label(
        &self,
        label: &str,
        node_type: Option<NodeType>,
    ) -> alan_core::Result<Vec<AnyContentNode>> {
        self.nodes.find_by_label(label, node_type).await
    }

    async fn list_by_type(&self, node_type: NodeType) -> alan_core::Result<Vec<AnyContentNode>> {
        self.nodes.list_by_type(node_type).await
    }
}

#[async_trait]
impl RelationStore for DualStore {
    async fn create_edge(
        &self,
        edge: &BipartiteEdge,
        relation: &AnyRelationNode,
    ) -> alan_core::Result<()> {
        self.relations.create_edge(edge, relation).await
    }

    async fn get_edges_from(
        &self,
        node_id: &NodeId,
        edge_type: Option<EdgeNodeType>,
    ) -> alan_core::Result<Vec<(BipartiteEdge, AnyRelationNode)>> {
        self.relations.get_edges_from(node_id, edge_type).await
    }

    async fn get_edges_to(
        &self,
        node_id: &NodeId,
        edge_type: Option<EdgeNodeType>,
    ) -> alan_core::Result<Vec<(BipartiteEdge, AnyRelationNode)>> {
        self.relations.get_edges_to(node_id, edge_type).await
    }

    async fn get_relation(&self, id: &NodeId) -> alan_core::Result<Option<AnyRelationNode>> {
        self.relations.get_relation(id).await
    }

    async fn update_relation(&self, relation: &AnyRelationNode) -> alan_core::Result<()> {
        self.relations.update_relation(relation).await
    }
}

#[async_trait]
impl VectorStore for DualStore {
    async fn search(
        &self,
        node_type: NodeType,
        query_embedding: &Embedding,
        top_k: usize,
    ) -> alan_core::Result<Vec<(AnyContentNode, f32)>> {
        self.vectors.search(node_type, query_embedding, top_k).await
    }

    async fn search_messages(
        &self,
        query_embedding: &Embedding,
        top_k: usize,
    ) -> alan_core::Result<Vec<(Message, f32)>> {
        self.vectors.search_messages(query_embedding, top_k).await
    }
}
