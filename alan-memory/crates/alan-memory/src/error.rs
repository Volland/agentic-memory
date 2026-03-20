#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Cognition error: {0}")]
    Cognition(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Core error: {0}")]
    Core(#[from] alan_core::AlanError),
}

pub type Result<T> = std::result::Result<T, MemoryError>;
