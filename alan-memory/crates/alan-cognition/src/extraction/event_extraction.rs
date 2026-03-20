use std::sync::Arc;

use serde::Deserialize;
use tracing::warn;

use crate::context::CognitiveContext;
use crate::error::Result;
use crate::extraction::output::{EventParticipant, ExtractedEvent};
use crate::extraction::prompt::event::event_extraction_prompt;
use crate::traits::classifier::ClassifierBackend;
use crate::traits::llm::{self, LlmBackend};

/// Extraction step that identifies events from conversation text.
pub struct EventExtractionStep {
    pub llm: Arc<dyn LlmBackend>,
    pub classifier: Option<Arc<dyn ClassifierBackend>>,
}

#[derive(Debug, Deserialize)]
struct LlmEventParticipant {
    label: String,
    role: String,
}

#[derive(Debug, Deserialize)]
struct LlmEvent {
    label: String,
    predicate: String,
    status: Option<String>,
    is_ongoing: Option<bool>,
    temporal_ref: Option<String>,
    #[serde(default)]
    participants: Vec<LlmEventParticipant>,
    causal_hint: Option<String>,
    source_fragment: Option<String>,
}

impl EventExtractionStep {
    pub fn new(llm: Arc<dyn LlmBackend>, classifier: Option<Arc<dyn ClassifierBackend>>) -> Self {
        Self { llm, classifier }
    }

    pub async fn execute(&self, ctx: &mut CognitiveContext) -> Result<()> {
        let text = ctx.full_text();
        if text.trim().is_empty() {
            return Ok(());
        }

        let entity_labels: Vec<String> = ctx
            .extracted_entities
            .iter()
            .map(|e| e.label.clone())
            .collect();
        let fact_labels: Vec<String> = ctx
            .extracted_facts
            .iter()
            .map(|f| f.label.clone())
            .collect();

        let (system, user) = event_extraction_prompt(&text, &entity_labels, &fact_labels);

        let llm_events: Vec<LlmEvent> = match llm::complete_structured(
            self.llm.as_ref(),
            &user,
            Some(&system),
        )
        .await
        {
            Ok(events) => events,
            Err(e) => {
                warn!("LLM event extraction failed: {e}");
                ctx.record_error(e);
                Vec::new()
            }
        };

        let status_labels: Vec<String> = vec![
            "completed".into(),
            "ongoing".into(),
            "planned".into(),
            "cancelled".into(),
        ];

        let mut extracted: Vec<ExtractedEvent> = Vec::new();

        for ev in llm_events {
            let mut status = ev.status.unwrap_or_else(|| "completed".to_string());

            // Optionally refine status using classifier.
            if let Some(ref classifier) = self.classifier {
                if let Ok(classifications) = classifier
                    .classify(&ev.source_fragment.as_deref().unwrap_or(&ev.predicate), &status_labels)
                    .await
                {
                    if let Some((best_label, _confidence)) = classifications.first() {
                        status = best_label.clone();
                    }
                }
            }

            let is_ongoing = ev.is_ongoing.unwrap_or(status == "ongoing");

            let participants: Vec<EventParticipant> = ev
                .participants
                .into_iter()
                .map(|p| EventParticipant {
                    label: p.label,
                    role: p.role,
                })
                .collect();

            extracted.push(ExtractedEvent {
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

        ctx.extracted_events.extend(extracted);
        Ok(())
    }
}
