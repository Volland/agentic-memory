#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("DuckDB error: {0}")]
    DuckDb(String),
    #[error("LadybugDB error: {0}")]
    LadybugDb(String),
    #[error("Bridge error: {0}")]
    Bridge(String),
    #[error("Schema migration error: {0}")]
    Migration(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Core error: {0}")]
    Core(#[from] alan_core::AlanError),
}

pub type Result<T> = std::result::Result<T, StorageError>;

impl StorageError {
    /// Convert this StorageError into an AlanError for trait impls.
    pub fn into_alan_error(self) -> alan_core::AlanError {
        match self {
            StorageError::NotFound(msg) => alan_core::AlanError::NotFound(msg),
            StorageError::Core(e) => e,
            other => alan_core::AlanError::Validation(other.to_string()),
        }
    }
}
