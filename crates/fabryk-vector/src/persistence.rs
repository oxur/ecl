//! Persistence and freshness checking for vector indices.
//!
//! Provides content-hash-based staleness detection so vector indices
//! can persist across restarts. When the content hash matches, the
//! existing index is still valid and doesn't need rebuilding.

use fabryk_core::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Metadata stored alongside a vector index for freshness checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    /// Content hash at build time.
    pub content_hash: String,

    /// Number of documents indexed.
    pub document_count: usize,

    /// Embedding dimension.
    pub embedding_dimension: usize,

    /// Build timestamp (ISO 8601).
    pub built_at: String,

    /// Embedding provider name.
    pub provider: String,

    /// Model name used for embeddings.
    pub model: String,
}

/// Check if an existing vector index is fresh.
///
/// Compares the stored content hash with a freshly computed one.
/// Returns `true` if the index exists and the hashes match.
///
/// # Arguments
///
/// * `metadata_path` - Path to the index metadata JSON file
/// * `current_hash` - Freshly computed content hash
pub fn is_index_fresh(metadata_path: &Path, current_hash: &str) -> bool {
    match load_metadata(metadata_path) {
        Ok(metadata) => metadata.content_hash == current_hash,
        Err(_) => false,
    }
}

/// Save index metadata to a JSON file.
pub fn save_metadata(metadata_path: &Path, metadata: &IndexMetadata) -> Result<()> {
    let json = serde_json::to_string_pretty(metadata)?;
    std::fs::write(metadata_path, json)
        .map_err(|e| fabryk_core::Error::io_with_path(e, metadata_path))?;
    Ok(())
}

/// Load index metadata from a JSON file.
pub fn load_metadata(metadata_path: &Path) -> Result<IndexMetadata> {
    let json = std::fs::read_to_string(metadata_path)
        .map_err(|e| fabryk_core::Error::io_with_path(e, metadata_path))?;
    let metadata: IndexMetadata = serde_json::from_str(&json)?;
    Ok(metadata)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_metadata() -> IndexMetadata {
        IndexMetadata {
            content_hash: "abc123def456".to_string(),
            document_count: 42,
            embedding_dimension: 384,
            built_at: "2025-01-15T12:00:00Z".to_string(),
            provider: "fastembed".to_string(),
            model: "bge-small-en-v1.5".to_string(),
        }
    }

    #[test]
    fn test_metadata_serialization() {
        let metadata = sample_metadata();
        let json = serde_json::to_string(&metadata).unwrap();

        assert!(json.contains("abc123def456"));
        assert!(json.contains("42"));
        assert!(json.contains("384"));
        assert!(json.contains("fastembed"));

        let deserialized: IndexMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.content_hash, "abc123def456");
        assert_eq!(deserialized.document_count, 42);
    }

    #[test]
    fn test_save_and_load_metadata() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("index_metadata.json");

        let metadata = sample_metadata();
        save_metadata(&path, &metadata).unwrap();

        let loaded = load_metadata(&path).unwrap();
        assert_eq!(loaded.content_hash, metadata.content_hash);
        assert_eq!(loaded.document_count, metadata.document_count);
        assert_eq!(loaded.embedding_dimension, metadata.embedding_dimension);
        assert_eq!(loaded.provider, metadata.provider);
        assert_eq!(loaded.model, metadata.model);
    }

    #[test]
    fn test_is_index_fresh_matching_hash() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("metadata.json");

        let metadata = sample_metadata();
        save_metadata(&path, &metadata).unwrap();

        assert!(is_index_fresh(&path, "abc123def456"));
    }

    #[test]
    fn test_is_index_fresh_different_hash() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("metadata.json");

        let metadata = sample_metadata();
        save_metadata(&path, &metadata).unwrap();

        assert!(!is_index_fresh(&path, "different_hash"));
    }

    #[test]
    fn test_is_index_fresh_missing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");

        assert!(!is_index_fresh(&path, "any_hash"));
    }

    #[test]
    fn test_load_metadata_invalid_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bad.json");
        std::fs::write(&path, "not valid json").unwrap();

        assert!(load_metadata(&path).is_err());
    }

    #[test]
    fn test_save_metadata_invalid_path() {
        let path = Path::new("/nonexistent/dir/metadata.json");
        let metadata = sample_metadata();
        assert!(save_metadata(path, &metadata).is_err());
    }
}
