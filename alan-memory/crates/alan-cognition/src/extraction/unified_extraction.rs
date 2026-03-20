use std::sync::Arc;

use serde::Deserialize;
use tracing::{debug, warn};

use crate::context::CognitiveContext;
use crate::error::Result;
use crate::extraction::output::{
    EventParticipant, ExtractedEntity, ExtractedEvent, ExtractedFact, ExtractedMemory,
    ExtractedRelation, ExtractedTemporalRef,
};
use crate::extraction::prompt::unified::unified_extraction_prompt;
use crate::traits::llm::{self, LlmBackend};
use crate::traits::ner::NerBackend;

/// Single unified extraction result from one LLM call.
#[derive(Debug, Clone, Deserialize)]
pub struct UnifiedExtractionResult {
    #[serde(default)]
    pub entities: Vec<LlmUnifiedEntity>,
    #[serde(default)]
    pub facts: Vec<LlmUnifiedFact>,
    #[serde(default)]
    pub events: Vec<LlmUnifiedEvent>,
    #[serde(default)]
    pub memories: Vec<LlmUnifiedMemory>,
    #[serde(default)]
    pub relations: Vec<LlmUnifiedRelation>,
    #[serde(default)]
    pub temporal_refs: Vec<LlmUnifiedTemporalRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmUnifiedEntity {
    pub label: String,
    pub entity_type: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub description: Option<String>,
    pub confidence: Option<f64>,
    pub source_fragment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmUnifiedFact {
    pub label: String,
    pub subject: String,
    pub predicate: String,
    pub object: Option<String>,
    pub object_value: Option<String>,
    #[serde(default)]
    pub fact_type: Option<String>,
    pub certainty: Option<f64>,
    pub source_fragment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmUnifiedEvent {
    pub label: String,
    pub predicate: String,
    pub status: Option<String>,
    pub is_ongoing: Option<bool>,
    #[serde(default)]
    pub participants: Vec<String>,
    pub temporal_ref: Option<String>,
    pub causal_hint: Option<String>,
    pub source_fragment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmUnifiedMemory {
    pub label: String,
    pub predicate: String,
    pub memory_type: Option<String>,
    pub significance: Option<String>,
    #[serde(default)]
    pub emotions: Vec<String>,
    pub intensity: Option<f64>,
    pub reflection: Option<String>,
    #[serde(default)]
    pub related_events: Vec<String>,
    #[serde(default)]
    pub related_entities: Vec<String>,
    pub source_fragment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmUnifiedRelation {
    pub source_ref: String,
    pub target_ref: String,
    pub relation_type: String,
    pub label: String,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmUnifiedTemporalRef {
    pub expression: String,
    pub temporal_type: String,
    pub anchor_ref: String,
    pub relation_to_anchor: String,
}

/// Unified extraction step: single LLM call extracts all layers at once.
pub struct UnifiedExtractionStep {
    pub llm: Arc<dyn LlmBackend>,
    pub ner: Option<Arc<dyn NerBackend>>,
}

impl UnifiedExtractionStep {
    pub fn new(llm: Arc<dyn LlmBackend>, ner: Option<Arc<dyn NerBackend>>) -> Self {
        Self { llm, ner }
    }

    pub async fn execute(&self, ctx: &mut CognitiveContext) -> Result<()> {
        let text = ctx.full_text();
        if text.trim().is_empty() {
            return Ok(());
        }

        let conversation_id = ctx.conversation.as_ref().map(|c| c.id.as_str());

        // ── Step 1: Unified LLM extraction ─────────────────────────────
        let (system, user) = unified_extraction_prompt(&text, conversation_id, None);

        let result: UnifiedExtractionResult = match llm::complete_structured(
            self.llm.as_ref(),
            &user,
            Some(&system),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                warn!("Unified LLM extraction failed: {e}");
                ctx.record_error(e);
                return Ok(());
            }
        };

        debug!(
            entities = result.entities.len(),
            facts = result.facts.len(),
            events = result.events.len(),
            memories = result.memories.len(),
            relations = result.relations.len(),
            temporal_refs = result.temporal_refs.len(),
            "Unified extraction complete"
        );

        // ── Step 2: Optional NER cross-check ───────────────────────────
        let mut ner_labels: std::collections::HashSet<String> = std::collections::HashSet::new();

        if let Some(ref ner) = self.ner {
            match ner.extract_entities(&text).await {
                Ok(spans) => {
                    for span in &spans {
                        ner_labels.insert(span.text.to_lowercase());
                    }

                    // Add NER-only entities that the LLM missed.
                    let llm_labels: std::collections::HashSet<String> = result
                        .entities
                        .iter()
                        .map(|e| e.label.to_lowercase())
                        .collect();

                    for span in spans {
                        if !llm_labels.contains(&span.text.to_lowercase()) {
                            ctx.extracted_entities.push(ExtractedEntity {
                                label: span.text.clone(),
                                entity_type: Some(span.entity_type.clone()),
                                confidence: span.confidence * 0.8, // NER-only gets slight penalty
                                source_fragment: span.text,
                                aliases: Vec::new(),
                                source_message_id: None,
                                extraction_method: "ner".to_string(),
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!("NER cross-check failed, continuing: {e}");
                    ctx.record_error(e);
                }
            }
        }

        // ── Step 3: Convert to extraction output types ─────────────────

        // Entities
        for e in result.entities {
            let confidence = e.confidence.unwrap_or(0.5);
            // Boost confidence if NER confirms this entity.
            let boosted = if ner_labels.contains(&e.label.to_lowercase()) {
                (confidence + 0.1).min(1.0)
            } else {
                confidence
            };

            ctx.extracted_entities.push(ExtractedEntity {
                label: e.label,
                entity_type: e.entity_type,
                confidence: boosted,
                source_fragment: e.source_fragment.unwrap_or_default(),
                aliases: e.aliases,
                source_message_id: None,
                extraction_method: "unified_llm".to_string(),
            });
        }

        // Facts
        for f in result.facts {
            ctx.extracted_facts.push(ExtractedFact {
                label: f.label,
                predicate: f.predicate,
                subject_label: f.subject,
                object_label: f.object,
                fact_type: f.fact_type.unwrap_or_else(|| "encyclopedic".to_string()),
                certainty: f.certainty.unwrap_or(0.5),
                source_fragment: f.source_fragment.unwrap_or_default(),
                source_message_id: None,
            });
        }

        // Events
        for ev in result.events {
            let status = ev.status.unwrap_or_else(|| "occurred".to_string());
            let is_ongoing = ev.is_ongoing.unwrap_or(status == "ongoing");

            let participants: Vec<EventParticipant> = ev
                .participants
                .into_iter()
                .map(|label| EventParticipant {
                    label,
                    role: "participant".to_string(),
                })
                .collect();

            ctx.extracted_events.push(ExtractedEvent {
                label: ev.label,
                predicate: ev.predicate,
                status,
                is_ongoing,
                temporal_ref: ev.temporal_ref,
                participants,
                causal_hint: ev.causal_hint,
                source_fragment: ev.source_fragment.unwrap_or_default(),
                source_message_id: None,
            });
        }

        // Memories
        for m in result.memories {
            ctx.extracted_memories.push(ExtractedMemory {
                label: m.label,
                predicate: m.predicate,
                memory_type: m.memory_type,
                significance: m.significance,
                emotions: m.emotions,
                intensity: m.intensity.unwrap_or(0.5),
                reflection: m.reflection,
                connected_events: m.related_events,
                connected_entities: m.related_entities,
                source_fragment: m.source_fragment.unwrap_or_default(),
                source_message_id: None,
            });
        }

        // Relations
        for r in result.relations {
            ctx.extracted_relations.push(ExtractedRelation {
                from_label: r.source_ref,
                to_label: r.target_ref,
                edge_type: r.relation_type,
                properties: serde_json::json!({"label": r.label}),
                confidence: r.confidence.unwrap_or(0.5),
                source_message_id: None,
            });
        }

        // Temporal refs
        for t in result.temporal_refs {
            ctx.extracted_temporal_refs.push(ExtractedTemporalRef {
                expression: t.expression,
                temporal_type: t.temporal_type,
                anchor_entity_label: t.anchor_ref,
                relation_type: t.relation_to_anchor,
                source_message_id: None,
            });
        }

        Ok(())
    }
}
