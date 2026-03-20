use std::sync::Arc;

use serde::Deserialize;
use tracing::{debug, warn};

use alan_core::entity::{AnyContentNode, Fact};

use crate::context::CognitiveContext;
use crate::error::{CognitionError, Result};
use crate::extraction::output::SupersedeResult;
use crate::traits::LlmBackend;
use crate::traits::llm as llm_util;

use super::prompt::contradiction;

/// LLM response for the enhanced contradiction/supersede detection.
#[derive(Debug, Deserialize)]
struct ReconciliationResponse {
    relationship: String,
    confidence: f64,
    #[allow(dead_code)]
    reasoning: String,
    #[serde(default)]
    should_supersede: bool,
    supersede_reason: Option<String>,
    suggested_certainty_adjustment: f64,
}

/// Step that detects contradictions and updates between new and existing facts,
/// adjusts certainty, and creates supersede records instead of deleting.
pub struct FactMergingStep {
    pub llm: Arc<dyn LlmBackend>,
}

impl FactMergingStep {
    pub fn new(llm: Arc<dyn LlmBackend>) -> Self {
        Self { llm }
    }

    pub async fn execute(&self, ctx: &mut CognitiveContext) -> Result<()> {
        if ctx.extracted_facts.is_empty() {
            return Ok(());
        }

        debug!(
            count = ctx.extracted_facts.len(),
            "Starting fact merging with supersede model"
        );

        // Collect existing fact labels from already-resolved nodes.
        let existing_facts: Vec<(usize, String)> = ctx
            .resolved_nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| {
                if let AnyContentNode::Fact(f) = node {
                    Some((idx, f.universal.label.clone()))
                } else {
                    None
                }
            })
            .collect();

        let mut new_fact_nodes = Vec::new();
        let mut supersede_results = Vec::new();

        // Clone extracted facts so we don't hold an immutable borrow on ctx.
        let extracted_facts = ctx.extracted_facts.clone();

        for extracted in &extracted_facts {
            let mut certainty = extracted.certainty;

            // Compare with existing facts for contradictions / confirmations / updates.
            for (existing_idx, existing_label) in &existing_facts {
                let prompt = contradiction::contradiction_detection_prompt(
                    &extracted.label,
                    existing_label,
                );
                let system = contradiction::contradiction_detection_system();

                match llm_util::complete_structured::<ReconciliationResponse>(
                    self.llm.as_ref(),
                    &prompt,
                    Some(system),
                )
                .await
                {
                    Ok(resp) if resp.confidence > 0.5 => {
                        match resp.relationship.as_str() {
                            "confirms" => {
                                debug!(
                                    new = %extracted.label,
                                    existing = %existing_label,
                                    "Fact confirmed — reinforcing certainty"
                                );
                                // Reinforce the existing fact.
                                if let AnyContentNode::Fact(ref mut f) =
                                    ctx.resolved_nodes[*existing_idx]
                                {
                                    f.certainty =
                                        (f.certainty + resp.suggested_certainty_adjustment.abs())
                                            .clamp(0.0, 1.0);
                                }
                                // Boost new fact certainty as well.
                                certainty = (certainty
                                    + resp.suggested_certainty_adjustment.abs())
                                .clamp(0.0, 1.0);
                            }
                            "contradicts" | "updates" => {
                                debug!(
                                    new = %extracted.label,
                                    existing = %existing_label,
                                    relationship = %resp.relationship,
                                    "Fact supersede detected"
                                );

                                if resp.should_supersede {
                                    // Record supersede — do NOT delete the old fact.
                                    // The old fact gets weakened certainty and an
                                    // expiration will be set by the forgetting pipeline.
                                    if let AnyContentNode::Fact(ref mut f) =
                                        ctx.resolved_nodes[*existing_idx]
                                    {
                                        // Drastically reduce certainty of superseded fact.
                                        f.certainty = (f.certainty * 0.2).clamp(0.0, 1.0);
                                    }

                                    supersede_results.push(SupersedeResult {
                                        new_label: extracted.label.clone(),
                                        old_label: existing_label.clone(),
                                        reason: resp
                                            .supersede_reason
                                            .unwrap_or_else(|| resp.relationship.clone()),
                                        confidence: resp.confidence,
                                    });

                                    // New fact gets high certainty — it's the latest info.
                                    certainty = certainty.max(0.8);
                                } else {
                                    // Weaken both slightly when contradicting without
                                    // clear supersede.
                                    if let AnyContentNode::Fact(ref mut f) =
                                        ctx.resolved_nodes[*existing_idx]
                                    {
                                        f.certainty = (f.certainty
                                            - resp.suggested_certainty_adjustment.abs())
                                        .clamp(0.0, 1.0);
                                    }
                                    certainty = (certainty * 0.8).clamp(0.0, 1.0);
                                }
                            }
                            _ => {
                                // "unrelated" — no adjustment.
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        warn!(error = %e, "Fact reconciliation LLM call failed");
                        ctx.record_error(CognitionError::ProcessFailed {
                            process: "fact_merging".into(),
                            message: e.to_string(),
                        });
                    }
                }
            }

            // Build the resolved Fact node.
            let fact = Fact::new(&extracted.label, &extracted.predicate)
                .with_certainty(certainty)
                .with_source("extraction".to_string());

            new_fact_nodes.push(AnyContentNode::Fact(fact));
        }

        ctx.resolved_nodes.extend(new_fact_nodes);

        // TODO: Convert supersede_results into Supersedes edges in relation wiring.
        // For now, store them as extracted relations so the wiring step can pick them up.
        for sr in supersede_results {
            ctx.extracted_relations.push(crate::extraction::output::ExtractedRelation {
                from_label: sr.new_label,
                to_label: sr.old_label,
                edge_type: "Supersedes".to_string(),
                properties: serde_json::json!({
                    "reason": sr.reason,
                    "confidence": sr.confidence,
                }),
                confidence: sr.confidence,
                source_message_id: None,
            });
        }

        debug!("Fact merging with supersede model complete");
        Ok(())
    }
}
