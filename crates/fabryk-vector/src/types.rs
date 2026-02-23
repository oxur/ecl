//! Common types for the vector search module.
//!
//! These types are used across all vector backends and embedding providers,
//! and are always available regardless of feature flags.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Configuration
// ============================================================================

/// Vector search configuration.
///
/// Controls backend selection, embedding model, storage paths, and behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConfig {
    /// Backend type: "lancedb" or "simple".
    #[serde(default = "default_backend")]
    pub backend: String,

    /// Embedding provider: "fastembed" or "mock".
    #[serde(default = "default_provider")]
    pub provider: String,

    /// Embedding model name (e.g., "bge-small-en-v1.5").
    #[serde(default = "default_model")]
    pub model: String,

    /// Embedding dimension (auto-detected if 0).
    #[serde(default)]
    pub dimension: usize,

    /// Path to the vector database directory.
    pub db_path: Option<String>,

    /// Path to content for indexing.
    pub content_path: Option<String>,

    /// Path to cache directory for embedding models.
    pub cache_path: Option<String>,

    /// Whether vector search is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Default search result limit.
    #[serde(default = "default_limit")]
    pub default_limit: usize,

    /// Default similarity threshold (0.0 to 1.0).
    #[serde(default = "default_threshold")]
    pub similarity_threshold: f32,

    /// Batch size for embedding operations.
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_backend() -> String {
    "lancedb".to_string()
}

fn default_provider() -> String {
    "fastembed".to_string()
}

fn default_model() -> String {
    "bge-small-en-v1.5".to_string()
}

fn default_true() -> bool {
    true
}

fn default_limit() -> usize {
    10
}

fn default_threshold() -> f32 {
    0.0
}

fn default_batch_size() -> usize {
    64
}

impl Default for VectorConfig {
    fn default() -> Self {
        Self {
            backend: default_backend(),
            provider: default_provider(),
            model: default_model(),
            dimension: 0,
            db_path: None,
            content_path: None,
            cache_path: None,
            enabled: default_true(),
            default_limit: default_limit(),
            similarity_threshold: default_threshold(),
            batch_size: default_batch_size(),
        }
    }
}

// ============================================================================
// Documents
// ============================================================================

/// A document prepared for vector embedding.
///
/// Domain-agnostic representation: domains compose the `text` field with
/// whatever content should be embedded (title, description, body, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    /// Unique document identifier.
    pub id: String,

    /// Text to be embedded (pre-composed by the domain extractor).
    pub text: String,

    /// Optional category for filtering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Arbitrary metadata key-value pairs.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

impl VectorDocument {
    /// Create a new vector document.
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            category: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the category.
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Add a metadata key-value pair.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// A document with its computed embedding vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedDocument {
    /// The original document.
    pub document: VectorDocument,

    /// The embedding vector.
    pub embedding: Vec<f32>,
}

impl EmbeddedDocument {
    /// Create a new embedded document.
    pub fn new(document: VectorDocument, embedding: Vec<f32>) -> Self {
        Self {
            document,
            embedding,
        }
    }

    /// The embedding dimension.
    pub fn dimension(&self) -> usize {
        self.embedding.len()
    }
}

// ============================================================================
// Search types
// ============================================================================

/// Parameters for a vector search request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VectorSearchParams {
    /// Search query string (will be embedded).
    pub query: String,

    /// Maximum results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,

    /// Minimum similarity score (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity_threshold: Option<f32>,

    /// Filter by category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Metadata filters as key-value pairs.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata_filters: HashMap<String, String>,
}

impl VectorSearchParams {
    /// Create search params with a query string.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            ..Default::default()
        }
    }

    /// Set the result limit.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the similarity threshold.
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.similarity_threshold = Some(threshold);
        self
    }

    /// Set a category filter.
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Add a metadata filter.
    pub fn with_filter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata_filters.insert(key.into(), value.into());
        self
    }
}

/// A single vector search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    /// Document identifier.
    pub id: String,

    /// Similarity score (0.0 to 1.0, higher is more similar).
    pub score: f32,

    /// Raw distance from the query vector.
    pub distance: f32,

    /// Metadata snapshot from the indexed document.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

/// Collection of vector search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResults {
    /// Search result items, ordered by score (highest first).
    pub items: Vec<VectorSearchResult>,

    /// Total number of matching documents.
    pub total: usize,

    /// Backend that executed the search.
    pub backend: String,
}

impl VectorSearchResults {
    /// Create empty results.
    pub fn empty(backend: &str) -> Self {
        Self {
            items: Vec::new(),
            total: 0,
            backend: backend.to_string(),
        }
    }
}

// ============================================================================
// Index statistics
// ============================================================================

/// Statistics from a vector index build operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndexStats {
    /// Number of documents indexed.
    pub documents_indexed: usize,

    /// Number of files processed.
    pub files_processed: usize,

    /// Number of files skipped due to errors.
    pub files_skipped: usize,

    /// Embedding dimension used.
    pub embedding_dimension: usize,

    /// Content hash for freshness checking.
    pub content_hash: String,

    /// Build duration in milliseconds.
    pub build_duration_ms: u64,

    /// Errors encountered (if not fail-fast).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<BuildError>,

    /// Whether the result was loaded from cache.
    #[serde(default)]
    pub from_cache: bool,
}

/// An error that occurred during vector index building.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildError {
    /// Path to the problematic file.
    pub file: PathBuf,
    /// Error message.
    pub message: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // VectorConfig tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_vector_config_default() {
        let config = VectorConfig::default();
        assert_eq!(config.backend, "lancedb");
        assert_eq!(config.provider, "fastembed");
        assert_eq!(config.model, "bge-small-en-v1.5");
        assert_eq!(config.dimension, 0);
        assert!(config.db_path.is_none());
        assert!(config.content_path.is_none());
        assert!(config.cache_path.is_none());
        assert!(config.enabled);
        assert_eq!(config.default_limit, 10);
        assert_eq!(config.similarity_threshold, 0.0);
        assert_eq!(config.batch_size, 64);
    }

    #[test]
    fn test_vector_config_serialization() {
        let config = VectorConfig {
            backend: "lancedb".to_string(),
            db_path: Some("/tmp/vectors".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"backend\":\"lancedb\""));
        assert!(json.contains("\"/tmp/vectors\""));
    }

    #[test]
    fn test_vector_config_deserialization_with_defaults() {
        let json = r#"{"backend": "lancedb"}"#;
        let config: VectorConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.backend, "lancedb");
        assert_eq!(config.default_limit, 10);
        assert!(config.enabled);
        assert_eq!(config.batch_size, 64);
    }

    // ------------------------------------------------------------------------
    // VectorDocument tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_vector_document_new() {
        let doc = VectorDocument::new("doc-1", "Hello world");
        assert_eq!(doc.id, "doc-1");
        assert_eq!(doc.text, "Hello world");
        assert!(doc.category.is_none());
        assert!(doc.metadata.is_empty());
    }

    #[test]
    fn test_vector_document_with_category() {
        let doc = VectorDocument::new("doc-1", "text").with_category("harmony");
        assert_eq!(doc.category, Some("harmony".to_string()));
    }

    #[test]
    fn test_vector_document_with_metadata() {
        let doc = VectorDocument::new("doc-1", "text")
            .with_metadata("author", "test")
            .with_metadata("tier", "beginner");

        assert_eq!(doc.metadata.len(), 2);
        assert_eq!(doc.metadata.get("author").unwrap(), "test");
        assert_eq!(doc.metadata.get("tier").unwrap(), "beginner");
    }

    #[test]
    fn test_vector_document_serialization() {
        let doc = VectorDocument::new("doc-1", "text content")
            .with_category("test")
            .with_metadata("key", "value");

        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("doc-1"));
        assert!(json.contains("text content"));
        assert!(json.contains("test"));

        let deserialized: VectorDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "doc-1");
        assert_eq!(deserialized.text, "text content");
        assert_eq!(deserialized.category, Some("test".to_string()));
    }

    #[test]
    fn test_vector_document_serialization_skips_empty() {
        let doc = VectorDocument::new("doc-1", "text");
        let json = serde_json::to_string(&doc).unwrap();

        // category and metadata should be omitted when empty/None
        assert!(!json.contains("category"));
        assert!(!json.contains("metadata"));
    }

    // ------------------------------------------------------------------------
    // EmbeddedDocument tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_embedded_document_new() {
        let doc = VectorDocument::new("doc-1", "text");
        let embedding = vec![0.1, 0.2, 0.3];
        let embedded = EmbeddedDocument::new(doc, embedding);

        assert_eq!(embedded.document.id, "doc-1");
        assert_eq!(embedded.embedding.len(), 3);
        assert_eq!(embedded.dimension(), 3);
    }

    // ------------------------------------------------------------------------
    // VectorSearchParams tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_search_params_default() {
        let params = VectorSearchParams::default();
        assert!(params.query.is_empty());
        assert!(params.limit.is_none());
        assert!(params.similarity_threshold.is_none());
        assert!(params.category.is_none());
        assert!(params.metadata_filters.is_empty());
    }

    #[test]
    fn test_search_params_builder() {
        let params = VectorSearchParams::new("semantic query")
            .with_limit(5)
            .with_threshold(0.5)
            .with_category("harmony")
            .with_filter("tier", "advanced");

        assert_eq!(params.query, "semantic query");
        assert_eq!(params.limit, Some(5));
        assert_eq!(params.similarity_threshold, Some(0.5));
        assert_eq!(params.category, Some("harmony".to_string()));
        assert_eq!(params.metadata_filters.get("tier").unwrap(), "advanced");
    }

    #[test]
    fn test_search_params_serialization() {
        let params = VectorSearchParams::new("test query").with_limit(10);

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("test query"));
        assert!(json.contains("10"));

        // Optional None fields should be skipped
        let minimal = VectorSearchParams::new("q");
        let json = serde_json::to_string(&minimal).unwrap();
        assert!(!json.contains("limit"));
        assert!(!json.contains("similarity_threshold"));
    }

    // ------------------------------------------------------------------------
    // VectorSearchResult tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_search_result_serialization() {
        let result = VectorSearchResult {
            id: "doc-1".to_string(),
            score: 0.85,
            distance: 0.176,
            metadata: HashMap::from([("category".to_string(), "harmony".to_string())]),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("doc-1"));
        assert!(json.contains("0.85"));
    }

    #[test]
    fn test_search_result_empty_metadata_skipped() {
        let result = VectorSearchResult {
            id: "doc-1".to_string(),
            score: 0.5,
            distance: 1.0,
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("metadata"));
    }

    // ------------------------------------------------------------------------
    // VectorSearchResults tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_search_results_empty() {
        let results = VectorSearchResults::empty("lancedb");
        assert!(results.items.is_empty());
        assert_eq!(results.total, 0);
        assert_eq!(results.backend, "lancedb");
    }

    // ------------------------------------------------------------------------
    // VectorIndexStats tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_index_stats_serialization() {
        let stats = VectorIndexStats {
            documents_indexed: 100,
            files_processed: 50,
            files_skipped: 2,
            embedding_dimension: 384,
            content_hash: "abc123".to_string(),
            build_duration_ms: 1500,
            errors: vec![],
            from_cache: false,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("100"));
        assert!(json.contains("384"));
        assert!(json.contains("abc123"));

        // Empty errors should be omitted
        assert!(!json.contains("errors"));
    }

    #[test]
    fn test_index_stats_with_errors() {
        let stats = VectorIndexStats {
            documents_indexed: 10,
            files_processed: 12,
            files_skipped: 2,
            embedding_dimension: 384,
            content_hash: "hash".to_string(),
            build_duration_ms: 500,
            errors: vec![BuildError {
                file: PathBuf::from("/test/bad.md"),
                message: "parse error".to_string(),
            }],
            from_cache: false,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("errors"));
        assert!(json.contains("parse error"));
    }
}
