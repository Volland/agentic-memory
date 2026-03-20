use alan_core::conversation::conversation::Conversation;
use alan_core::conversation::message::Message;
use alan_core::entity::AnyContentNode;
use alan_core::graph::wiring::BipartiteEdge;
use alan_core::relation::AnyRelationNode;

use crate::error::CognitionError;
use crate::extraction::output::{
    ExtractedEntity, ExtractedEvent, ExtractedFact, ExtractedMemory, ExtractedRelation,
    ExtractedTemporalRef,
};

/// Central context object threaded through every cognitive process.
///
/// Processes read from earlier stages and append their own results so that
/// downstream steps can build on prior extractions.
#[derive(Debug)]
pub struct CognitiveContext {
    // ── Input ──────────────────────────────────────────────────────────
    pub messages: Vec<Message>,
    pub conversation: Option<Conversation>,

    // ── Extraction results ─────────────────────────────────────────────
    pub extracted_entities: Vec<ExtractedEntity>,
    pub extracted_facts: Vec<ExtractedFact>,
    pub extracted_events: Vec<ExtractedEvent>,
    pub extracted_memories: Vec<ExtractedMemory>,

    // ── Temporal & relation extraction ─────────────────────────────────
    pub extracted_temporal_refs: Vec<ExtractedTemporalRef>,
    pub extracted_relations: Vec<ExtractedRelation>,

    // ── Resolved graph objects ─────────────────────────────────────────
    pub resolved_nodes: Vec<AnyContentNode>,
    pub resolved_edges: Vec<(BipartiteEdge, AnyRelationNode)>,

    // ── Metadata ───────────────────────────────────────────────────────
    pub processing_errors: Vec<CognitionError>,
}

impl CognitiveContext {
    /// Create a new context from a batch of messages.
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            messages,
            conversation: None,
            extracted_entities: Vec::new(),
            extracted_facts: Vec::new(),
            extracted_events: Vec::new(),
            extracted_memories: Vec::new(),
            extracted_temporal_refs: Vec::new(),
            extracted_relations: Vec::new(),
            resolved_nodes: Vec::new(),
            resolved_edges: Vec::new(),
            processing_errors: Vec::new(),
        }
    }

    /// Create a context with an associated conversation.
    pub fn with_conversation(messages: Vec<Message>, conversation: Conversation) -> Self {
        let mut ctx = Self::new(messages);
        ctx.conversation = Some(conversation);
        ctx
    }

    /// Concatenate all message contents into a single text block.
    pub fn full_text(&self) -> String {
        self.messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Record a non-fatal processing error.
    pub fn record_error(&mut self, err: CognitionError) {
        self.processing_errors.push(err);
    }
}
