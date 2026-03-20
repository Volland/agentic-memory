use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// A single named-entity span detected by a NER backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NerSpan {
    pub text: String,
    pub entity_type: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f64,
}

#[async_trait]
pub trait NerBackend: Send + Sync {
    /// Extract named-entity spans from the given text.
    async fn extract_entities(&self, text: &str) -> Result<Vec<NerSpan>>;
}
