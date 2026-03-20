use std::sync::Arc;

use serde::Deserialize;
use tracing::{debug, warn};

use alan_core::entity::{AnyContentNode, Fact};

use crate::context::CognitiveContext;
use crate::error::{CognitionError, Result};
use crate::traits::LlmBackend;
use crate::traits::llm as llm_util;

use super::prompt::contradiction;

/// LLM response for contradiction detection.
#[derive(Debug, Deserialize)]
struct ContradictionResponse {
    relationship: String,
    confidence: f64,
    #[allow(dead_code)]
    reasoning: String,
    suggested_certainty_adjustment: f64,
}

/// Step that detects contradictions between new and existing facts and adjusts
/// certainty accordingly.
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
            "Starting fact merging"
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

        // Clone extracted facts so we don't hold an immutable borrow on ctx.
        let extracted_facts = ctx.extracted_facts.clone();

        for extracted in &extracted_facts {
            let mut certainty = extracted.certainty;

            // Compare with existing facts for contradictions / confirmations.
            for (existing_idx, existing_label) in &existing_facts {
                let prompt = contradiction::contradiction_detection_prompt(
                    &extracted.label,
                    existing_label,
                );
                let system = contradiction::contradiction_detection_system();

                match llm_util::complete_structured::<ContradictionResponse>(
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
                            "contradicts" => {
                                debug!(
                                    new = %extracted.label,
                                    existing = %existing_label,
                                    "Fact contradiction detected — reducing certainty"
                                );
                                // Weaken existing fact.
                                if let AnyContentNode::Fact(ref mut f) =
                                    ctx.resolved_nodes[*existing_idx]
                                {
                                    f.certainty = (f.certainty
                                        - resp.suggested_certainty_adjustment.abs())
                                    .clamp(0.0, 1.0);
                                }
                                // Weaken new fact slightly as well — both are uncertain.
                                certainty = (certainty * 0.8).clamp(0.0, 1.0);
                            }
                            _ => {
                                // "unrelated" — no adjustment.
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        warn!(error = %e, "Contradiction detection LLM call failed");
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

        debug!("Fact merging complete");
        Ok(())
    }
}
