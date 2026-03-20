use tracing::info;

use alan_cognition::chunk::{chunk_messages, ChunkStrategy};
use alan_cognition::context::CognitiveContext;
use alan_cognition::manager::CognitiveProcessManager;
use alan_cognition::traits::cognitive_process::ProcessResult;

use alan_core::conversation::{Conversation, Message};
use alan_core::embedding::Embedding;
use alan_core::entity::{AnyContentNode, Fact, Memory, NodeType};
use alan_core::graph::traits::{ConversationStore, NodeStore, RelationStore, VectorStore};
use alan_storage::bridge::DualStore;

use crate::config::{ChunkMode, MemoryConfig};
use crate::error::{MemoryError, Result};
use crate::ingest::{document_to_messages, DocumentMetadata, IngestResult};

/// The main API surface for the Alan Memory system.
///
/// Combines storage (DualStore), cognitive processing (CognitiveProcessManager),
/// and configuration into a single cohesive interface.
pub struct MemoryStore {
    storage: DualStore,
    cognition: CognitiveProcessManager,
    config: MemoryConfig,
}

impl MemoryStore {
    /// Create a new MemoryStore from its component parts.
    pub fn new(
        storage: DualStore,
        cognition: CognitiveProcessManager,
        config: MemoryConfig,
    ) -> Self {
        Self {
            storage,
            cognition,
            config,
        }
    }

    /// Access the underlying storage.
    pub fn storage(&self) -> &DualStore {
        &self.storage
    }

    /// Access the configuration.
    pub fn config(&self) -> &MemoryConfig {
        &self.config
    }

    /// Convert internal ChunkMode to alan_cognition ChunkStrategy.
    fn chunk_strategy(&self) -> ChunkStrategy {
        match &self.config.chunk_strategy {
            ChunkMode::PerMessage => ChunkStrategy::PerMessage,
            ChunkMode::SlidingWindow {
                window_size,
                overlap,
            } => ChunkStrategy::SlidingWindow {
                window_size: *window_size,
                overlap: *overlap,
            },
            ChunkMode::ByTurn => ChunkStrategy::ByTurn,
            ChunkMode::TokenBudget { max_tokens } => ChunkStrategy::TokenBudget {
                max_tokens: *max_tokens,
            },
        }
    }

    /// Ingest a conversation and its messages into the memory system.
    ///
    /// This will:
    /// 1. Store the conversation and messages in DuckDB.
    /// 2. Chunk messages according to the configured strategy.
    /// 3. Run extraction and consolidation cognitive processes.
    /// 4. Persist extracted nodes and edges into the graph store.
    /// 5. Trigger temporal linking.
    pub async fn ingest_conversation(
        &self,
        conv: Conversation,
        messages: Vec<Message>,
    ) -> Result<IngestResult> {
        info!(
            conversation_id = conv.id.as_str(),
            message_count = messages.len(),
            "Ingesting conversation"
        );

        // 1. Store conversation and messages.
        self.storage
            .create_conversation(&conv)
            .await
            .map_err(|e| MemoryError::Storage(e.to_string()))?;

        for msg in &messages {
            self.storage
                .append_message(msg)
                .await
                .map_err(|e| MemoryError::Storage(e.to_string()))?;
        }

        // 2. Chunk messages.
        let strategy = self.chunk_strategy();
        let chunks = chunk_messages(&messages, &strategy);

        let mut total_result = IngestResult::default();

        // 3. Process each chunk through cognitive pipeline.
        for chunk in chunks {
            let mut ctx = CognitiveContext::with_conversation(
                chunk.messages,
                conv.clone(),
            );

            // Run reactive processes (extraction, consolidation).
            let _results = self
                .cognition
                .process_ingestion(&mut ctx)
                .await
                .map_err(|e| MemoryError::Cognition(e.to_string()))?;

            // Count extracted items.
            total_result.entities_extracted += ctx.extracted_entities.len();
            total_result.facts_extracted += ctx.extracted_facts.len();
            total_result.events_extracted += ctx.extracted_events.len();
            total_result.memories_extracted += ctx.extracted_memories.len();

            // 4. Persist resolved nodes.
            for node in &ctx.resolved_nodes {
                match self.storage.create_node(node).await {
                    Ok(_) => total_result.nodes_created += 1,
                    Err(e) => {
                        tracing::warn!(error = %e, "Failed to persist node");
                    }
                }
            }

            // 5. Persist resolved edges.
            for (edge, relation) in &ctx.resolved_edges {
                match self.storage.create_edge(edge, relation).await {
                    Ok(_) => total_result.edges_created += 1,
                    Err(e) => {
                        tracing::warn!(error = %e, "Failed to persist edge");
                    }
                }
            }
        }

        info!(
            nodes = total_result.nodes_created,
            edges = total_result.edges_created,
            entities = total_result.entities_extracted,
            facts = total_result.facts_extracted,
            events = total_result.events_extracted,
            memories = total_result.memories_extracted,
            "Ingestion complete"
        );

        Ok(total_result)
    }

    /// Ingest a document by converting it to pseudo-messages and processing
    /// it through the same pipeline as conversations.
    pub async fn ingest_document(
        &self,
        text: &str,
        metadata: DocumentMetadata,
    ) -> Result<IngestResult> {
        let (conv, messages) = document_to_messages(text, &metadata);
        self.ingest_conversation(conv, messages).await
    }

    /// Search for memories by semantic similarity.
    ///
    /// Returns Memory nodes whose embeddings are closest to the query embedding,
    /// filtered by the configured similarity threshold.
    pub async fn search_memories(
        &self,
        _query: &str,
        query_embedding: &Embedding,
        top_k: usize,
    ) -> Result<Vec<Memory>> {
        let results = self
            .storage
            .search(NodeType::Memory, query_embedding, top_k)
            .await
            .map_err(|e| MemoryError::Storage(e.to_string()))?;

        let threshold = self.config.similarity_threshold as f32;
        let memories: Vec<Memory> = results
            .into_iter()
            .filter(|(_, score)| *score >= threshold)
            .filter_map(|(node, _)| match node {
                AnyContentNode::Memory(m) => Some(m),
                _ => None,
            })
            .collect();

        Ok(memories)
    }

    /// Search for facts by semantic similarity.
    ///
    /// Returns Fact nodes whose embeddings are closest to the query embedding,
    /// filtered by the configured similarity threshold.
    pub async fn search_facts(
        &self,
        _query: &str,
        query_embedding: &Embedding,
        top_k: usize,
    ) -> Result<Vec<Fact>> {
        let results = self
            .storage
            .search(NodeType::Fact, query_embedding, top_k)
            .await
            .map_err(|e| MemoryError::Storage(e.to_string()))?;

        let threshold = self.config.similarity_threshold as f32;
        let facts: Vec<Fact> = results
            .into_iter()
            .filter(|(_, score)| *score >= threshold)
            .filter_map(|(node, _)| match node {
                AnyContentNode::Fact(f) => Some(f),
                _ => None,
            })
            .collect();

        Ok(facts)
    }

    /// Manually trigger the forgetting process.
    pub async fn run_forgetting(&self) -> Result<ProcessResult> {
        let mut ctx = CognitiveContext::new(Vec::new());
        self.cognition
            .run_process("forgetting", &mut ctx)
            .await
            .map_err(|e| MemoryError::Cognition(e.to_string()))
    }

    /// Manually trigger temporal linking.
    pub async fn run_temporal_linking(&self) -> Result<ProcessResult> {
        let mut ctx = CognitiveContext::new(Vec::new());
        self.cognition
            .run_process("temporal_linking", &mut ctx)
            .await
            .map_err(|e| MemoryError::Cognition(e.to_string()))
    }
}
