use serde::{Deserialize, Serialize};

/// Strategy for chunking messages during ingestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkMode {
    /// Each message is its own chunk.
    PerMessage,
    /// Sliding window with configurable size and overlap.
    SlidingWindow { window_size: usize, overlap: usize },
    /// Group by conversation turn (user + assistant pair).
    ByTurn,
    /// Token-budget based chunking.
    TokenBudget { max_tokens: usize },
}

impl Default for ChunkMode {
    fn default() -> Self {
        Self::ByTurn
    }
}

/// Configuration for the MemoryStore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Path to the DuckDB database file.
    pub duckdb_path: String,
    /// Path to the LadybugDB database file.
    pub ladybugdb_path: String,
    /// Embedding vector dimension.
    pub embedding_dim: usize,
    /// Strategy for chunking messages.
    pub chunk_strategy: ChunkMode,
    /// Interval (in seconds) between automatic forgetting runs.
    pub forgetting_interval_secs: u64,
    /// Interval (in seconds) between automatic temporal linking runs.
    pub temporal_interval_secs: u64,
    /// Minimum cosine similarity threshold for search results.
    pub similarity_threshold: f64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            duckdb_path: "memory.duckdb".to_string(),
            ladybugdb_path: "memory.ladybugdb".to_string(),
            embedding_dim: 518,
            chunk_strategy: ChunkMode::default(),
            forgetting_interval_secs: 3600,
            temporal_interval_secs: 300,
            similarity_threshold: 0.7,
        }
    }
}

/// Builder for constructing a MemoryConfig with custom values.
pub struct MemoryConfigBuilder {
    config: MemoryConfig,
}

impl MemoryConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: MemoryConfig::default(),
        }
    }

    pub fn duckdb_path(mut self, path: impl Into<String>) -> Self {
        self.config.duckdb_path = path.into();
        self
    }

    pub fn ladybugdb_path(mut self, path: impl Into<String>) -> Self {
        self.config.ladybugdb_path = path.into();
        self
    }

    pub fn embedding_dim(mut self, dim: usize) -> Self {
        self.config.embedding_dim = dim;
        self
    }

    pub fn chunk_strategy(mut self, strategy: ChunkMode) -> Self {
        self.config.chunk_strategy = strategy;
        self
    }

    pub fn forgetting_interval_secs(mut self, secs: u64) -> Self {
        self.config.forgetting_interval_secs = secs;
        self
    }

    pub fn temporal_interval_secs(mut self, secs: u64) -> Self {
        self.config.temporal_interval_secs = secs;
        self
    }

    pub fn similarity_threshold(mut self, threshold: f64) -> Self {
        self.config.similarity_threshold = threshold;
        self
    }

    pub fn build(self) -> MemoryConfig {
        self.config
    }
}

impl Default for MemoryConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryConfig {
    /// Start building a MemoryConfig with the builder pattern.
    pub fn builder() -> MemoryConfigBuilder {
        MemoryConfigBuilder::new()
    }
}
