use async_trait::async_trait;
use alan_core::Embedding;

use crate::error::Result;

#[async_trait]
pub trait EmbedderBackend: Send + Sync {
    /// Embed a batch of texts, returning one embedding per input.
    async fn embed(&self, texts: &[String]) -> Result<Vec<Embedding>>;

    /// Convenience: embed a single text.
    async fn embed_single(&self, text: &str) -> Result<Embedding> {
        let results = self.embed(&[text.to_string()]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| crate::error::CognitionError::EmbedderError("No embedding returned".into()))
    }
}
