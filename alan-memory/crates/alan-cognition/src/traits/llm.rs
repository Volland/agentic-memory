use async_trait::async_trait;
use serde::de::DeserializeOwned;

use crate::error::Result;

#[async_trait]
pub trait LlmBackend: Send + Sync {
    /// Send a prompt to the LLM and return the raw completion text.
    async fn complete(&self, prompt: &str, system: Option<&str>) -> Result<String>;

    /// Send a prompt and return the completion parsed as a JSON value.
    ///
    /// Implementations should instruct the model to return valid JSON.
    async fn complete_json(&self, prompt: &str, system: Option<&str>) -> Result<serde_json::Value>;
}

/// Helper: call `complete_json` and deserialize the resulting value into `T`.
pub async fn complete_structured<T: DeserializeOwned>(
    llm: &dyn LlmBackend,
    prompt: &str,
    system: Option<&str>,
) -> Result<T> {
    let value = llm.complete_json(prompt, system).await?;
    let parsed: T = serde_json::from_value(value)?;
    Ok(parsed)
}
