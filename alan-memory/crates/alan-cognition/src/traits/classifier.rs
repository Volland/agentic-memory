use async_trait::async_trait;

use crate::error::Result;

#[async_trait]
pub trait ClassifierBackend: Send + Sync {
    /// Classify text against a set of candidate labels.
    /// Returns (label, confidence) pairs sorted by descending confidence.
    async fn classify(&self, text: &str, labels: &[String]) -> Result<Vec<(String, f64)>>;
}
