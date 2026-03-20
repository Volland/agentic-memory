use std::time::Duration;

use async_trait::async_trait;

use crate::context::CognitiveContext;
use crate::error::Result;

/// Whether a process is triggered reactively or on a periodic schedule.
pub enum ProcessKind {
    /// Runs in response to new input (e.g. extraction after a message arrives).
    Reactive,
    /// Runs on a timer (e.g. periodic consolidation).
    Periodic { interval: Duration },
}

/// Summary returned after a cognitive process completes.
pub struct ProcessResult {
    pub process_name: String,
    pub items_produced: usize,
    pub duration: Duration,
    pub warnings: Vec<String>,
}

#[async_trait]
pub trait CognitiveProcess: Send + Sync {
    /// Human-readable name of this process (e.g. "extraction").
    fn name(&self) -> &str;

    /// Whether this process is reactive or periodic.
    fn kind(&self) -> ProcessKind;

    /// Execute the cognitive process, reading from and writing to the context.
    async fn execute(&self, ctx: &mut CognitiveContext) -> Result<ProcessResult>;
}
