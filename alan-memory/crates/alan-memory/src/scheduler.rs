use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A simple periodic scheduler placeholder.
///
/// In a full implementation this would spawn tokio tasks that periodically
/// run forgetting, temporal linking, and other background processes.
/// For now it only tracks running state.
pub struct PeriodicScheduler {
    running: Arc<AtomicBool>,
}

impl PeriodicScheduler {
    /// Create a new scheduler (not running).
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the scheduler.
    ///
    /// Placeholder: in a real implementation this would spawn tokio tasks
    /// for periodic forgetting, temporal linking, etc.
    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        // TODO: spawn periodic tokio tasks here.
    }

    /// Stop the scheduler.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Check whether the scheduler is currently running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Default for PeriodicScheduler {
    fn default() -> Self {
        Self::new()
    }
}
