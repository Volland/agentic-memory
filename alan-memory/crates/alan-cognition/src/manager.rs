use std::sync::Arc;

use tracing::{debug, info};

use crate::consolidation::ConsolidationProcess;
use crate::context::CognitiveContext;
use crate::error::{CognitionError, Result};
use crate::forgetting::ForgettingProcess;
use crate::temporal::TemporalLinkingProcess;
use crate::traits::cognitive_process::{CognitiveProcess, ProcessKind, ProcessResult};
use crate::traits::embedder::EmbedderBackend;
use crate::traits::llm::LlmBackend;

/// Manages all cognitive processes and provides convenience methods for
/// running them individually or in groups.
pub struct CognitiveProcessManager {
    processes: Vec<Box<dyn CognitiveProcess>>,
}

impl CognitiveProcessManager {
    /// Create an empty manager.
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
        }
    }

    /// Register a cognitive process.
    pub fn register(&mut self, process: Box<dyn CognitiveProcess>) {
        info!(process = process.name(), "Registering cognitive process");
        self.processes.push(process);
    }

    /// Create a manager pre-populated with the default process set:
    /// consolidation, temporal linking, and forgetting.
    pub fn default_with_backends(
        llm: Arc<dyn LlmBackend>,
        embedder: Arc<dyn EmbedderBackend>,
    ) -> Self {
        let mut manager = Self::new();

        manager.register(Box::new(ConsolidationProcess::new(
            Arc::clone(&llm),
            Arc::clone(&embedder),
        )));

        manager.register(Box::new(TemporalLinkingProcess::new(Arc::clone(&llm))));

        manager.register(Box::new(ForgettingProcess::new()));

        manager
    }

    /// Run all reactive processes in registration order.
    /// Typically called after new messages are ingested and extraction is
    /// complete.
    pub async fn process_ingestion(
        &self,
        ctx: &mut CognitiveContext,
    ) -> Result<Vec<ProcessResult>> {
        let mut results = Vec::new();

        for process in &self.processes {
            if !matches!(process.kind(), ProcessKind::Reactive) {
                continue;
            }

            debug!(process = process.name(), "Running reactive process");
            match process.execute(ctx).await {
                Ok(result) => {
                    info!(
                        process = result.process_name.as_str(),
                        items = result.items_produced,
                        duration_ms = result.duration.as_millis() as u64,
                        "Reactive process complete"
                    );
                    results.push(result);
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    ctx.record_error(CognitionError::ProcessFailed {
                        process: process.name().to_string(),
                        message: err_msg,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Run all periodic processes. Typically called on a timer.
    pub async fn run_periodic(
        &self,
        ctx: &mut CognitiveContext,
    ) -> Result<Vec<ProcessResult>> {
        let mut results = Vec::new();

        for process in &self.processes {
            if matches!(process.kind(), ProcessKind::Reactive) {
                continue;
            }

            debug!(process = process.name(), "Running periodic process");
            match process.execute(ctx).await {
                Ok(result) => {
                    info!(
                        process = result.process_name.as_str(),
                        items = result.items_produced,
                        duration_ms = result.duration.as_millis() as u64,
                        "Periodic process complete"
                    );
                    results.push(result);
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    ctx.record_error(CognitionError::ProcessFailed {
                        process: process.name().to_string(),
                        message: err_msg,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Run a specific process by name.
    pub async fn run_process(
        &self,
        name: &str,
        ctx: &mut CognitiveContext,
    ) -> Result<ProcessResult> {
        let process = self
            .processes
            .iter()
            .find(|p| p.name() == name)
            .ok_or_else(|| CognitionError::ProcessFailed {
                process: name.to_string(),
                message: "Process not found".to_string(),
            })?;

        debug!(process = name, "Running specific process");
        let result = process.execute(ctx).await?;

        info!(
            process = result.process_name.as_str(),
            items = result.items_produced,
            duration_ms = result.duration.as_millis() as u64,
            "Process complete"
        );

        Ok(result)
    }

    /// List all registered process names.
    pub fn process_names(&self) -> Vec<&str> {
        self.processes.iter().map(|p| p.name()).collect()
    }
}

impl Default for CognitiveProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
