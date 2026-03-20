use std::sync::Arc;

use serde::Deserialize;
use tracing::{debug, warn};

use alan_core::entity::{AnyContentNode, Entity, Event, EventStatus, Fact, Memory};

use crate::context::CognitiveContext;
use crate::error::{CognitionError, Result};
use crate::extraction::output::ExtractedRelation;
use crate::traits::embedder::EmbedderBackend;
use crate::traits::llm::{self, LlmBackend};

use super::prompt::consolidation_decision::consolidation_decision_prompt;

/// LLM response for consolidation decisions.
#[derive(Debug, Clone, Deserialize)]
pub struct ConsolidationDecisionResponse {
    pub decisions: Vec<ConsolidationDecision>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsolidationDecision {
    pub extracted_ref: String,
    pub action: String,
    pub existing_node_id: Option<String>,
    pub reason: String,
    pub supersede_reason: Option<String>,
    pub certainty_boost: Option<f64>,
    pub merge_updates: Option<MergeUpdates>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MergeUpdates {
    #[serde(default)]
    pub aliases_to_add: Vec<String>,
    pub description_update: Option<String>,
    pub entity_type_update: Option<String>,
}

/// Step that compares extracted knowledge against existing graph context
/// and decides CREATE / MERGE / SUPERSEDE / REINFORCE / SKIP for each item.
///
/// This is designed to work with the unified extraction output.
/// When no graph context is available (empty graph), all items become CREATE.
pub struct ConsolidationDecisionStep {
    pub llm: Arc<dyn LlmBackend>,
    pub embedder: Arc<dyn EmbedderBackend>,
}

impl ConsolidationDecisionStep {
    pub fn new(llm: Arc<dyn LlmBackend>, embedder: Arc<dyn EmbedderBackend>) -> Self {
        Self { llm, embedder }
    }

    pub async fn execute(&self, ctx: &mut CognitiveContext) -> Result<()> {
        let has_items = !ctx.extracted_entities.is_empty()
            || !ctx.extracted_facts.is_empty()
            || !ctx.extracted_events.is_empty()
            || !ctx.extracted_memories.is_empty();

        if !has_items {
            return Ok(());
        }

        debug!("Starting consolidation decision step");

        // Build a JSON summary of extracted items for the LLM.
        let extracted_json = build_extracted_summary(ctx);

        // Build existing graph context from already-resolved nodes.
        // In a full implementation this would do vector search against the
        // persistent graph; here we compare against in-context resolved nodes.
        let graph_context = build_graph_context(ctx);

        // If there's no existing graph context, skip the LLM call —
        // everything is CREATE by default.
        if graph_context.trim().is_empty() || graph_context == "No existing nodes in graph context." {
            debug!("No graph context — all items will be created");
            create_all_nodes(ctx).await;
            return Ok(());
        }

        // Single LLM call for all consolidation decisions.
        let (system, user) = consolidation_decision_prompt(&extracted_json, &graph_context);

        let response: ConsolidationDecisionResponse = match llm::complete_structured(
            self.llm.as_ref(),
            &user,
            Some(&system),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                warn!("Consolidation decision LLM call failed, creating all: {e}");
                ctx.record_error(CognitionError::ProcessFailed {
                    process: "consolidation_decision".into(),
                    message: e.to_string(),
                });
                create_all_nodes(ctx).await;
                return Ok(());
            }
        };

        debug!(
            decisions = response.decisions.len(),
            "Consolidation decisions received"
        );

        // Apply decisions.
        apply_decisions(ctx, &response.decisions);

        Ok(())
    }
}

/// Build a JSON summary of extracted items for the consolidation prompt.
fn build_extracted_summary(ctx: &CognitiveContext) -> String {
    let mut parts = Vec::new();

    if !ctx.extracted_entities.is_empty() {
        let entities: Vec<serde_json::Value> = ctx
            .extracted_entities
            .iter()
            .enumerate()
            .map(|(i, e)| {
                serde_json::json!({
                    "ref": format!("entity:{i}"),
                    "label": e.label,
                    "entity_type": e.entity_type,
                    "confidence": e.confidence,
                })
            })
            .collect();
        parts.push(format!("\"entities\": {}", serde_json::to_string(&entities).unwrap_or_default()));
    }

    if !ctx.extracted_facts.is_empty() {
        let facts: Vec<serde_json::Value> = ctx
            .extracted_facts
            .iter()
            .enumerate()
            .map(|(i, f)| {
                serde_json::json!({
                    "ref": format!("fact:{i}"),
                    "label": f.label,
                    "subject": f.subject_label,
                    "predicate": f.predicate,
                    "object": f.object_label,
                    "certainty": f.certainty,
                })
            })
            .collect();
        parts.push(format!("\"facts\": {}", serde_json::to_string(&facts).unwrap_or_default()));
    }

    if !ctx.extracted_events.is_empty() {
        let events: Vec<serde_json::Value> = ctx
            .extracted_events
            .iter()
            .enumerate()
            .map(|(i, e)| {
                serde_json::json!({
                    "ref": format!("event:{i}"),
                    "label": e.label,
                    "predicate": e.predicate,
                    "status": e.status,
                })
            })
            .collect();
        parts.push(format!("\"events\": {}", serde_json::to_string(&events).unwrap_or_default()));
    }

    if !ctx.extracted_memories.is_empty() {
        let memories: Vec<serde_json::Value> = ctx
            .extracted_memories
            .iter()
            .enumerate()
            .map(|(i, m)| {
                serde_json::json!({
                    "ref": format!("memory:{i}"),
                    "label": m.label,
                    "predicate": m.predicate,
                    "emotions": m.emotions,
                })
            })
            .collect();
        parts.push(format!("\"memories\": {}", serde_json::to_string(&memories).unwrap_or_default()));
    }

    format!("{{{}}}", parts.join(", "))
}

/// Build graph context from already-resolved nodes in the context.
fn build_graph_context(ctx: &CognitiveContext) -> String {
    if ctx.resolved_nodes.is_empty() {
        return "No existing nodes in graph context.".to_string();
    }

    let mut nodes = Vec::new();
    for node in &ctx.resolved_nodes {
        let u = node.universal();
        let node_type = format!("{:?}", node.node_type());
        nodes.push(format!(
            "<existing_node>\n  <id>{}</id>\n  <type>{}</type>\n  <label>{}</label>\n</existing_node>",
            u.id.as_str(), node_type, u.label
        ));
    }

    nodes.join("\n")
}

/// Default behavior when there's no graph context: create all nodes.
async fn create_all_nodes(ctx: &mut CognitiveContext) {
    // Entities
    for extracted in &ctx.extracted_entities {
        let mut entity = Entity::new(&extracted.label);
        if let Some(ref etype) = extracted.entity_type {
            entity = entity.with_context(etype.clone());
        }
        ctx.resolved_nodes.push(AnyContentNode::Entity(entity));
    }

    // Facts
    for extracted in &ctx.extracted_facts {
        let fact = Fact::new(&extracted.label, &extracted.predicate)
            .with_certainty(extracted.certainty)
            .with_source("extraction".to_string());
        ctx.resolved_nodes.push(AnyContentNode::Fact(fact));
    }

    // Events
    for extracted in &ctx.extracted_events {
        let status = match extracted.status.as_str() {
            "planned" => EventStatus::Planned,
            "cancelled" => EventStatus::Cancelled,
            "hypothetical" => EventStatus::Hypothetical,
            _ => EventStatus::Occurred,
        };
        let event = Event::new(&extracted.label, &extracted.predicate)
            .with_status(status)
            .with_ongoing(extracted.is_ongoing)
            .with_source("extraction".to_string());
        ctx.resolved_nodes.push(AnyContentNode::Event(event));
    }

    // Memories
    for extracted in &ctx.extracted_memories {
        let mut memory = Memory::new(&extracted.label, &extracted.predicate)
            .with_source("extraction".to_string())
            .with_emotions(extracted.emotions.clone());
        if let Some(ref sig) = extracted.significance {
            memory = memory.with_significance(sig.clone());
        }
        if let Some(ref refl) = extracted.reflection {
            memory = memory.with_reflection(refl.clone());
        }
        ctx.resolved_nodes.push(AnyContentNode::Memory(memory));
    }
}

/// Apply consolidation decisions to the context.
fn apply_decisions(ctx: &mut CognitiveContext, decisions: &[ConsolidationDecision]) {
    for decision in decisions {
        match decision.action.to_uppercase().as_str() {
            "CREATE" => {
                apply_create(ctx, &decision.extracted_ref);
            }
            "MERGE" => {
                if let Some(ref existing_id) = decision.existing_node_id {
                    apply_merge(ctx, &decision.extracted_ref, existing_id, &decision.merge_updates);
                }
            }
            "SUPERSEDE" => {
                if let Some(ref existing_id) = decision.existing_node_id {
                    apply_supersede(
                        ctx,
                        &decision.extracted_ref,
                        existing_id,
                        decision.supersede_reason.as_deref().unwrap_or("update"),
                    );
                }
            }
            "REINFORCE" => {
                if let Some(ref existing_id) = decision.existing_node_id {
                    let boost = decision.certainty_boost.unwrap_or(0.1);
                    apply_reinforce(ctx, existing_id, boost);
                }
            }
            "SKIP" => {
                debug!(
                    ref_ = %decision.extracted_ref,
                    reason = %decision.reason,
                    "Skipping extracted item"
                );
            }
            _ => {
                warn!(action = %decision.action, "Unknown consolidation action, treating as CREATE");
                apply_create(ctx, &decision.extracted_ref);
            }
        }
    }
}

fn parse_ref(extracted_ref: &str) -> Option<(&str, usize)> {
    let parts: Vec<&str> = extracted_ref.splitn(2, ':').collect();
    if parts.len() == 2 {
        parts[1].parse::<usize>().ok().map(|idx| (parts[0], idx))
    } else {
        None
    }
}

fn apply_create(ctx: &mut CognitiveContext, extracted_ref: &str) {
    let Some((kind, idx)) = parse_ref(extracted_ref) else {
        return;
    };

    match kind {
        "entity" => {
            if let Some(e) = ctx.extracted_entities.get(idx) {
                let mut entity = Entity::new(&e.label);
                if let Some(ref etype) = e.entity_type {
                    entity = entity.with_context(etype.clone());
                }
                ctx.resolved_nodes.push(AnyContentNode::Entity(entity));
            }
        }
        "fact" => {
            if let Some(f) = ctx.extracted_facts.get(idx) {
                let fact = Fact::new(&f.label, &f.predicate)
                    .with_certainty(f.certainty)
                    .with_source("extraction".to_string());
                ctx.resolved_nodes.push(AnyContentNode::Fact(fact));
            }
        }
        "event" => {
            if let Some(e) = ctx.extracted_events.get(idx) {
                let status = match e.status.as_str() {
                    "planned" => EventStatus::Planned,
                    "cancelled" => EventStatus::Cancelled,
                    "hypothetical" => EventStatus::Hypothetical,
                    _ => EventStatus::Occurred,
                };
                let event = Event::new(&e.label, &e.predicate)
                    .with_status(status)
                    .with_ongoing(e.is_ongoing)
                    .with_source("extraction".to_string());
                ctx.resolved_nodes.push(AnyContentNode::Event(event));
            }
        }
        "memory" => {
            if let Some(m) = ctx.extracted_memories.get(idx) {
                let mut memory = Memory::new(&m.label, &m.predicate)
                    .with_source("extraction".to_string())
                    .with_emotions(m.emotions.clone());
                if let Some(ref sig) = m.significance {
                    memory = memory.with_significance(sig.clone());
                }
                if let Some(ref refl) = m.reflection {
                    memory = memory.with_reflection(refl.clone());
                }
                ctx.resolved_nodes.push(AnyContentNode::Memory(memory));
            }
        }
        _ => {}
    }
}

fn apply_merge(
    ctx: &mut CognitiveContext,
    extracted_ref: &str,
    existing_id: &str,
    merge_updates: &Option<MergeUpdates>,
) {
    // Find the existing node and update it.
    if let Some(node) = ctx
        .resolved_nodes
        .iter_mut()
        .find(|n| n.universal().id.as_str() == existing_id)
    {
        if let Some(updates) = merge_updates {
            if let AnyContentNode::Entity(e) = node {
                if let Some(ref desc) = updates.description_update {
                    e.universal.label_resolved = Some(desc.clone());
                }
                if let Some(ref etype) = updates.entity_type_update {
                    e.universal.context = Some(etype.clone());
                }
            }
        }
        debug!(existing_id = %existing_id, "Merged into existing node");
    } else {
        // Fallback: if existing node not found in context, create as new.
        apply_create(ctx, extracted_ref);
    }
}

fn apply_supersede(
    ctx: &mut CognitiveContext,
    extracted_ref: &str,
    existing_id: &str,
    reason: &str,
) {
    // 1. Create the new node.
    let pre_count = ctx.resolved_nodes.len();
    apply_create(ctx, extracted_ref);

    // 2. If a new node was created, create a Supersedes relation.
    if ctx.resolved_nodes.len() > pre_count {
        let new_node = &ctx.resolved_nodes[ctx.resolved_nodes.len() - 1];
        let new_label = new_node.universal().label.clone();

        // Weaken the existing node's certainty.
        if let Some(node) = ctx
            .resolved_nodes
            .iter_mut()
            .find(|n| n.universal().id.as_str() == existing_id)
        {
            match node {
                AnyContentNode::Fact(f) => {
                    f.certainty = (f.certainty * 0.2).clamp(0.0, 1.0);
                }
                AnyContentNode::Event(e) => {
                    e.certainty = (e.certainty * 0.2).clamp(0.0, 1.0);
                }
                _ => {}
            }
        }

        // Record a Supersedes relation for the wiring step.
        ctx.extracted_relations.push(ExtractedRelation {
            from_label: new_label,
            to_label: existing_id.to_string(),
            edge_type: "Supersedes".to_string(),
            properties: serde_json::json!({
                "reason": reason,
            }),
            confidence: 1.0,
            source_message_id: None,
        });

        debug!(
            existing_id = %existing_id,
            reason = %reason,
            "Supersede: new node replaces existing"
        );
    }
}

fn apply_reinforce(ctx: &mut CognitiveContext, existing_id: &str, boost: f64) {
    if let Some(node) = ctx
        .resolved_nodes
        .iter_mut()
        .find(|n| n.universal().id.as_str() == existing_id)
    {
        match node {
            AnyContentNode::Fact(f) => {
                f.certainty = (f.certainty + boost).clamp(0.0, 1.0);
            }
            AnyContentNode::Event(e) => {
                e.certainty = (e.certainty + boost).clamp(0.0, 1.0);
            }
            _ => {}
        }
        debug!(existing_id = %existing_id, boost = boost, "Reinforced existing node");
    }
}
