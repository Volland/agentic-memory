use serde::{Deserialize, Serialize};

/// Default embedding dimension (matches the ontology schema FLOAT[518]).
pub const DEFAULT_EMBEDDING_DIM: usize = 518;

/// A vector embedding for semantic search.
/// Uses `Vec<f32>` rather than a fixed array to accommodate different embedding models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Embedding(Vec<f32>);

impl Embedding {
    /// Create a new embedding from a vector of floats.
    pub fn new(values: Vec<f32>) -> Self {
        Self(values)
    }

    /// Create a zero-filled embedding of the given dimension.
    pub fn zeros(dim: usize) -> Self {
        Self(vec![0.0; dim])
    }

    /// Create a zero-filled embedding with the default dimension.
    pub fn default_zeros() -> Self {
        Self::zeros(DEFAULT_EMBEDDING_DIM)
    }

    /// Get the dimension of this embedding.
    pub fn dim(&self) -> usize {
        self.0.len()
    }

    /// Get the raw float values.
    pub fn values(&self) -> &[f32] {
        &self.0
    }

    /// Consume and return the inner vector.
    pub fn into_values(self) -> Vec<f32> {
        self.0
    }

    /// Validate that this embedding matches the expected dimension.
    pub fn validate_dim(&self, expected: usize) -> Result<(), EmbeddingError> {
        if self.0.len() != expected {
            Err(EmbeddingError::DimensionMismatch {
                expected,
                actual: self.0.len(),
            })
        } else {
            Ok(())
        }
    }

    /// Compute cosine similarity with another embedding.
    pub fn cosine_similarity(&self, other: &Self) -> Result<f32, EmbeddingError> {
        if self.0.len() != other.0.len() {
            return Err(EmbeddingError::DimensionMismatch {
                expected: self.0.len(),
                actual: other.0.len(),
            });
        }

        let dot: f32 = self.0.iter().zip(other.0.iter()).map(|(a, b)| a * b).sum();
        let norm_a: f32 = self.0.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.0.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return Ok(0.0);
        }

        Ok(dot / (norm_a * norm_b))
    }
}

impl From<Vec<f32>> for Embedding {
    fn from(values: Vec<f32>) -> Self {
        Self(values)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("Embedding dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let e = Embedding::new(vec![1.0, 2.0, 3.0]);
        let sim = e.cosine_similarity(&e).unwrap();
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = Embedding::new(vec![1.0, 0.0]);
        let b = Embedding::new(vec![0.0, 1.0]);
        let sim = a.cosine_similarity(&b).unwrap();
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn test_dimension_mismatch() {
        let a = Embedding::new(vec![1.0, 2.0]);
        let b = Embedding::new(vec![1.0, 2.0, 3.0]);
        assert!(a.cosine_similarity(&b).is_err());
    }

    #[test]
    fn test_validate_dim() {
        let e = Embedding::default_zeros();
        assert!(e.validate_dim(DEFAULT_EMBEDDING_DIM).is_ok());
        assert!(e.validate_dim(512).is_err());
    }
}
