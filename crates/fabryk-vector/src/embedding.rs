//! Embedding provider trait and mock implementation.
//!
//! This module defines the `EmbeddingProvider` trait that abstracts over
//! different embedding generation backends (fastembed, OpenAI, etc.).
//!
//! # Providers
//!
//! - `MockEmbeddingProvider`: Deterministic fixed-dimension vectors for testing
//! - `FastEmbedProvider`: Local embedding via fastembed (requires `vector-fastembed` feature)

use async_trait::async_trait;
use fabryk_core::Result;

/// Trait for generating text embeddings.
///
/// Implementations wrap specific embedding libraries (fastembed, OpenAI, etc.)
/// and provide a uniform async interface. The trait requires `Send + Sync` to
/// allow safe sharing across async tasks.
///
/// # Thread Safety
///
/// Implementations should handle internal synchronization (e.g., `Arc<Mutex<>>`)
/// for thread-unsafe underlying libraries.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate an embedding for a single text.
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for a batch of texts.
    ///
    /// Default implementation calls `embed` for each text sequentially.
    /// Backends that support native batching should override this.
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    /// The embedding dimension.
    fn dimension(&self) -> usize;

    /// The provider name for diagnostics.
    fn name(&self) -> &str;
}

/// A mock embedding provider for testing.
///
/// Generates deterministic vectors based on the input text hash.
/// Each component is derived from the text bytes, producing consistent
/// embeddings for the same input.
pub struct MockEmbeddingProvider {
    dimension: usize,
}

impl MockEmbeddingProvider {
    /// Create a new mock provider with the given dimension.
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }

    /// Generate a deterministic embedding from text.
    fn deterministic_embedding(&self, text: &str) -> Vec<f32> {
        let mut embedding = vec![0.0f32; self.dimension];
        let bytes = text.as_bytes();

        for (i, val) in embedding.iter_mut().enumerate() {
            // Use byte values to create deterministic but varied components
            let byte_idx = i % bytes.len().max(1);
            let byte_val = if bytes.is_empty() {
                0u8
            } else {
                bytes[byte_idx]
            };
            *val = ((byte_val as f32 + i as f32) % 256.0) / 256.0;
        }

        // Normalize to unit vector
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }

        embedding
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        Ok(self.deterministic_embedding(text))
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|t| self.deterministic_embedding(t))
            .collect())
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn name(&self) -> &str {
        "mock"
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_provider_creation() {
        let provider = MockEmbeddingProvider::new(384);
        assert_eq!(provider.dimension(), 384);
        assert_eq!(provider.name(), "mock");
    }

    #[tokio::test]
    async fn test_mock_embed_single() {
        let provider = MockEmbeddingProvider::new(8);
        let embedding = provider.embed("hello world").await.unwrap();

        assert_eq!(embedding.len(), 8);

        // Verify unit-normalized
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }

    #[tokio::test]
    async fn test_mock_embed_deterministic() {
        let provider = MockEmbeddingProvider::new(16);
        let e1 = provider.embed("same text").await.unwrap();
        let e2 = provider.embed("same text").await.unwrap();

        assert_eq!(e1, e2);
    }

    #[tokio::test]
    async fn test_mock_embed_different_texts() {
        let provider = MockEmbeddingProvider::new(16);
        let e1 = provider.embed("text one").await.unwrap();
        let e2 = provider.embed("text two").await.unwrap();

        assert_ne!(e1, e2);
    }

    #[tokio::test]
    async fn test_mock_embed_batch() {
        let provider = MockEmbeddingProvider::new(8);
        let texts = vec!["hello", "world", "test"];
        let embeddings = provider.embed_batch(&texts).await.unwrap();

        assert_eq!(embeddings.len(), 3);
        for emb in &embeddings {
            assert_eq!(emb.len(), 8);
        }
    }

    #[tokio::test]
    async fn test_mock_embed_empty_text() {
        let provider = MockEmbeddingProvider::new(4);
        let embedding = provider.embed("").await.unwrap();

        assert_eq!(embedding.len(), 4);
        // Empty text produces zero vector (all 0s mapped from byte 0)
        // After normalization, it should remain zeros since norm is 0
    }

    #[tokio::test]
    async fn test_mock_embed_batch_empty() {
        let provider = MockEmbeddingProvider::new(4);
        let texts: Vec<&str> = vec![];
        let embeddings = provider.embed_batch(&texts).await.unwrap();

        assert!(embeddings.is_empty());
    }

    #[test]
    fn test_trait_object_safety() {
        // Verify EmbeddingProvider can be used as a trait object
        fn _assert_object_safe(_: &dyn EmbeddingProvider) {}
    }
}
