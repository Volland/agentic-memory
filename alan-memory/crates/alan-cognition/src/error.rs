#[derive(Debug, thiserror::Error)]
pub enum CognitionError {
    #[error("LLM backend error: {0}")]
    LlmError(String),
    #[error("NER backend error: {0}")]
    NerError(String),
    #[error("Embedder error: {0}")]
    EmbedderError(String),
    #[error("Classifier error: {0}")]
    ClassifierError(String),
    #[error("Process '{process}' failed: {message}")]
    ProcessFailed { process: String, message: String },
    #[error("Deserialization of model output failed: {0}")]
    OutputParse(#[from] serde_json::Error),
    #[error("Core error: {0}")]
    Core(#[from] alan_core::AlanError),
}

pub type Result<T> = std::result::Result<T, CognitionError>;
