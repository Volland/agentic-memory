pub mod temporal_extraction;
pub mod time_resolution;
pub mod validity_wiring;
pub mod ordering;
pub mod causality;
pub mod prompt;

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;

use crate::context::CognitiveContext;
use crate::error::Result;
use crate::traits::cognitive_process::{CognitiveProcess, ProcessKind, ProcessResult};
use crate::traits::llm::LlmBackend;

use self::causality::CausalityStep;
use self::ordering::OrderingStep;
use self::temporal_extraction::TemporalExtractionStep;
use self::time_resolution::TimeResolutionStep;
use self::validity_wiring::ValidityWiringStep;

/// Default periodic interval for temporal linking (5 minutes).
const DEFAULT_INTERVAL_SECS: u64 = 300;

/// Top-level temporal linking process.
///
/// Pipeline: temporal_extraction -> time_resolution -> validity_wiring -> ordering -> causality
pub struct TemporalLinkingProcess {
    llm: Arc<dyn LlmBackend>,
    interval: Duration,
}

impl TemporalLinkingProcess {
    pub fn new(llm: Arc<dyn LlmBackend>) -> Self {
        Self {
            llm,
            interval: Duration::from_secs(DEFAULT_INTERVAL_SECS),
        }
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }
}

#[async_trait]
impl CognitiveProcess for TemporalLinkingProcess {
    fn name(&self) -> &str {
        "temporal_linking"
    }

    fn kind(&self) -> ProcessKind {
        ProcessKind::Periodic {
            interval: self.interval,
        }
    }

    async fn execute(&self, ctx: &mut CognitiveContext) -> Result<ProcessResult> {
        let start = Instant::now();
        let mut warnings: Vec<String> = Vec::new();

        // 1. Extract temporal expressions from text.
        let extract_step = TemporalExtractionStep::new(Arc::clone(&self.llm));
        if let Err(e) = extract_step.execute(ctx).await {
            warnings.push(format!("Temporal extraction error: {e}"));
        }

        // 2. Resolve temporal expressions to Time / AbstractTime nodes.
        let resolve_step = TimeResolutionStep::new();
        if let Err(e) = resolve_step.execute(ctx).await {
            warnings.push(format!("Time resolution error: {e}"));
        }

        // 3. Wire ValidFrom / ValidTo edges.
        let validity_step = ValidityWiringStep::new();
        if let Err(e) = validity_step.execute(ctx).await {
            warnings.push(format!("Validity wiring error: {e}"));
        }

        // 4. Detect ordering (Before / After / During).
        let ordering_step = OrderingStep::new(Arc::clone(&self.llm));
        if let Err(e) = ordering_step.execute(ctx).await {
            warnings.push(format!("Ordering detection error: {e}"));
        }

        // 5. Detect causality (LeadsTo / Causes / Prevents / BecauseOf).
        let causality_step = CausalityStep::new(Arc::clone(&self.llm));
        if let Err(e) = causality_step.execute(ctx).await {
            warnings.push(format!("Causality detection error: {e}"));
        }

        let items_produced =
            ctx.extracted_temporal_refs.len() + ctx.resolved_edges.len();

        Ok(ProcessResult {
            process_name: self.name().to_string(),
            items_produced,
            duration: start.elapsed(),
            warnings,
        })
    }
}
