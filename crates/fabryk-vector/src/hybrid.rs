//! Hybrid search combining vector and full-text search results.
//!
//! Implements Reciprocal Rank Fusion (RRF) for merging ranked result lists
//! from different search backends. Generalized from the Taproot implementation.
//!
//! # Algorithm
//!
//! RRF score for document `d`: `score(d) = Σ 1/(k + rank_i(d))`
//!
//! Where `rank_i(d)` is the 1-based rank of `d` in result list `i`, and `k`
//! is a constant (default 60) that controls how much weight is given to
//! lower-ranked items.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::VectorSearchResult;

/// A hybrid search result combining vector and keyword search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResult {
    /// Document identifier.
    pub id: String,

    /// Combined RRF score (higher is better).
    pub score: f32,

    /// Source of the result: "vector", "keyword", or "hybrid".
    pub source: String,

    /// Metadata snapshot.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

/// An FTS result suitable for RRF merging.
///
/// This is a simplified representation of an FTS search result,
/// containing just the fields needed for rank fusion.
#[derive(Debug, Clone)]
pub struct FtsResult {
    /// Document identifier.
    pub id: String,
    /// Relevance score from FTS.
    pub score: f32,
    /// Metadata snapshot.
    pub metadata: HashMap<String, String>,
}

/// Merge vector and FTS results using Reciprocal Rank Fusion.
///
/// # Arguments
///
/// * `vector_results` - Results from vector similarity search
/// * `fts_results` - Results from full-text keyword search
/// * `limit` - Maximum results to return
/// * `k` - RRF constant (default 60, higher gives more weight to lower-ranked items)
///
/// # Algorithm
///
/// For each document appearing in any result list:
/// `rrf_score = Σ 1/(k + rank_i)` where `rank_i` is the 1-based rank
/// in each list where the document appears.
///
/// Documents appearing in both lists will naturally score higher.
pub fn reciprocal_rank_fusion(
    vector_results: &[VectorSearchResult],
    fts_results: &[FtsResult],
    limit: usize,
    k: u32,
) -> Vec<HybridSearchResult> {
    let mut scores: HashMap<String, f32> = HashMap::new();
    let mut metadata: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut sources: HashMap<String, (bool, bool)> = HashMap::new(); // (has_vector, has_fts)

    // Score vector results
    for (rank, result) in vector_results.iter().enumerate() {
        let rrf_score = 1.0 / (k as f32 + (rank + 1) as f32);
        *scores.entry(result.id.clone()).or_insert(0.0) += rrf_score;
        metadata
            .entry(result.id.clone())
            .or_insert_with(|| result.metadata.clone());
        sources.entry(result.id.clone()).or_insert((false, false)).0 = true;
    }

    // Score FTS results
    for (rank, result) in fts_results.iter().enumerate() {
        let rrf_score = 1.0 / (k as f32 + (rank + 1) as f32);
        *scores.entry(result.id.clone()).or_insert(0.0) += rrf_score;
        metadata
            .entry(result.id.clone())
            .or_insert_with(|| result.metadata.clone());
        sources.entry(result.id.clone()).or_insert((false, false)).1 = true;
    }

    // Build results and sort by RRF score
    let mut results: Vec<HybridSearchResult> = scores
        .into_iter()
        .map(|(id, score)| {
            let (has_vector, has_fts) = sources.get(&id).copied().unwrap_or((false, false));
            let source = match (has_vector, has_fts) {
                (true, true) => "hybrid",
                (true, false) => "vector",
                (false, true) => "keyword",
                (false, false) => "unknown",
            }
            .to_string();

            HybridSearchResult {
                id: id.clone(),
                score,
                source,
                metadata: metadata.remove(&id).unwrap_or_default(),
            }
        })
        .collect();

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);
    results
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vector_results(ids: &[&str]) -> Vec<VectorSearchResult> {
        ids.iter()
            .enumerate()
            .map(|(i, id)| VectorSearchResult {
                id: id.to_string(),
                score: 1.0 - (i as f32 * 0.1),
                distance: i as f32 * 0.1,
                metadata: HashMap::new(),
            })
            .collect()
    }

    fn make_fts_results(ids: &[&str]) -> Vec<FtsResult> {
        ids.iter()
            .enumerate()
            .map(|(i, id)| FtsResult {
                id: id.to_string(),
                score: 1.0 - (i as f32 * 0.1),
                metadata: HashMap::new(),
            })
            .collect()
    }

    #[test]
    fn test_rrf_both_sources() {
        let vector = make_vector_results(&["a", "b", "c"]);
        let fts = make_fts_results(&["d", "e", "f"]);

        let results = reciprocal_rank_fusion(&vector, &fts, 10, 60);

        assert_eq!(results.len(), 6);
        // All results should be from single sources
        for r in &results {
            assert!(r.source == "vector" || r.source == "keyword");
        }
    }

    #[test]
    fn test_rrf_vector_only() {
        let vector = make_vector_results(&["a", "b"]);
        let fts: Vec<FtsResult> = vec![];

        let results = reciprocal_rank_fusion(&vector, &fts, 10, 60);

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.source == "vector"));
    }

    #[test]
    fn test_rrf_fts_only() {
        let vector: Vec<VectorSearchResult> = vec![];
        let fts = make_fts_results(&["x", "y"]);

        let results = reciprocal_rank_fusion(&vector, &fts, 10, 60);

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.source == "keyword"));
    }

    #[test]
    fn test_rrf_deduplication() {
        // Same document in both lists
        let vector = make_vector_results(&["shared", "vec-only"]);
        let fts = make_fts_results(&["shared", "fts-only"]);

        let results = reciprocal_rank_fusion(&vector, &fts, 10, 60);

        assert_eq!(results.len(), 3); // shared, vec-only, fts-only

        // "shared" should be marked as hybrid
        let shared = results.iter().find(|r| r.id == "shared").unwrap();
        assert_eq!(shared.source, "hybrid");

        // "shared" should rank highest (it gets RRF scores from both lists)
        assert_eq!(results[0].id, "shared");
    }

    #[test]
    fn test_rrf_respects_limit() {
        let vector = make_vector_results(&["a", "b", "c", "d", "e"]);
        let fts = make_fts_results(&["f", "g", "h", "i", "j"]);

        let results = reciprocal_rank_fusion(&vector, &fts, 3, 60);

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_rrf_ordering() {
        // Items in both sources should rank higher than items in one
        let vector = make_vector_results(&["both-1", "both-2", "vec-only"]);
        let fts = make_fts_results(&["both-1", "both-2", "fts-only"]);

        let results = reciprocal_rank_fusion(&vector, &fts, 10, 60);

        // First two should be the shared ones
        let top_2_ids: Vec<&str> = results.iter().take(2).map(|r| r.id.as_str()).collect();
        assert!(top_2_ids.contains(&"both-1"));
        assert!(top_2_ids.contains(&"both-2"));
    }

    #[test]
    fn test_rrf_empty_inputs() {
        let results = reciprocal_rank_fusion(&[], &[], 10, 60);
        assert!(results.is_empty());
    }

    #[test]
    fn test_rrf_k_parameter_effect() {
        let vector = make_vector_results(&["a"]);
        let fts = make_fts_results(&["a"]);

        // With k=1, rank 1 contributes 1/(1+1) = 0.5 per list = 1.0 total
        let results_k1 = reciprocal_rank_fusion(&vector, &fts, 10, 1);
        // With k=60, rank 1 contributes 1/(60+1) ≈ 0.0164 per list ≈ 0.0328 total
        let results_k60 = reciprocal_rank_fusion(&vector, &fts, 10, 60);

        assert!(results_k1[0].score > results_k60[0].score);
    }

    #[test]
    fn test_rrf_scores_decrease_with_rank() {
        let vector = make_vector_results(&["first", "second", "third"]);
        let fts: Vec<FtsResult> = vec![];

        let results = reciprocal_rank_fusion(&vector, &fts, 10, 60);

        // Scores should decrease with rank
        for i in 0..results.len() - 1 {
            assert!(results[i].score >= results[i + 1].score);
        }
    }

    #[test]
    fn test_rrf_preserves_metadata() {
        let vector = vec![VectorSearchResult {
            id: "doc-1".to_string(),
            score: 0.9,
            distance: 0.1,
            metadata: HashMap::from([("category".to_string(), "harmony".to_string())]),
        }];

        let results = reciprocal_rank_fusion(&vector, &[], 10, 60);

        assert_eq!(results[0].metadata.get("category").unwrap(), "harmony");
    }

    #[test]
    fn test_hybrid_result_serialization() {
        let result = HybridSearchResult {
            id: "doc-1".to_string(),
            score: 0.5,
            source: "hybrid".to_string(),
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("doc-1"));
        assert!(json.contains("hybrid"));
        // Empty metadata should be omitted
        assert!(!json.contains("metadata"));
    }
}
