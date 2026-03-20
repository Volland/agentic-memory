pub mod relevance_scoring;
pub mod expiration;
pub mod certainty_decay;
pub mod consolidation_candidates;

use std::time::{Duration, Instant};

use async_trait::async_trait;

use crate::context::CognitiveContext;
use crate::error::Result;
use crate::traits::cognitive_process::{CognitiveProcess, ProcessKind, ProcessResult};

use self::certainty_decay::CertaintyDecayStep;
use self::consolidation_candidates::ConsolidationCandidateStep;
use self::expiration::ExpirationStep;
use self::relevance_scoring::RelevanceScoringStep;

/// Default periodic interval for the forgetting process (1 hour).
const DEFAULT_INTERVAL_SECS: u64 = 3600;

/// Top-level forgetting process. Purely algorithmic — no LLM calls.
///
/// Pipeline: relevance_scoring -> expiration -> certainty_decay -> consolidation_candidates
pub struct ForgettingProcess {
    interval: Duration,
}

impl ForgettingProcess {
    pub fn new() -> Self {
        Self {
            interval: Duration::from_secs(DEFAULT_INTERVAL_SECS),
        }
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }
}

impl Default for ForgettingProcess {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CognitiveProcess for ForgettingProcess {
    fn name(&self) -> &str {
        "forgetting"
    }

    fn kind(&self) -> ProcessKind {
        ProcessKind::Periodic {
            interval: self.interval,
        }
    }

    async fn execute(&self, ctx: &mut CognitiveContext) -> Result<ProcessResult> {
        let start = Instant::now();
        let mut warnings: Vec<String> = Vec::new();
        let mut items_produced: usize = 0;

        // 1. Score all nodes by relevance.
        let scoring_step = RelevanceScoringStep::new();
        let scores = match scoring_step.execute(ctx).await {
            Ok(s) => s,
            Err(e) => {
                warnings.push(format!("Relevance scoring error: {e}"));
                Vec::new()
            }
        };

        // 2. Expire low-relevance nodes (mark only, never delete).
        let expiration_step = ExpirationStep::new();
        match expiration_step.execute(ctx, &scores).await {
            Ok(count) => items_produced += count,
            Err(e) => warnings.push(format!("Expiration error: {e}")),
        }

        // 3. Decay certainty on unreinforced facts.
        let decay_step = CertaintyDecayStep::new();
        match decay_step.execute(ctx).await {
            Ok(count) => items_produced += count,
            Err(e) => warnings.push(format!("Certainty decay error: {e}")),
        }

        // 4. Identify consolidation candidates (suggestions only).
        let candidate_step = ConsolidationCandidateStep::new();
        match candidate_step.execute(ctx).await {
            Ok(candidates) => items_produced += candidates.len(),
            Err(e) => warnings.push(format!("Consolidation candidates error: {e}")),
        }

        Ok(ProcessResult {
            process_name: self.name().to_string(),
            items_produced,
            duration: start.elapsed(),
            warnings,
        })
    }
}
