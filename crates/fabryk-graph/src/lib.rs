//! Knowledge graph infrastructure for Fabryk.
//!
//! This crate provides graph storage, traversal algorithms, and
//! persistence using petgraph and optional rkyv caching.
//!
//! # Features
//!
//! - `graph-rkyv-cache`: Enable rkyv-based graph persistence with
//!   content-hash cache validation
//! - `test-utils`: Export mock types for testing in downstream crates
//!
//! # Key Abstractions
//!
//! - `GraphExtractor` trait: Domain implementations provide this to
//!   extract nodes and edges from content files
//! - `Relationship` enum: Common relationship types with `Custom(String)`
//!   for domain-specific relationships
//! - `NodeType` enum: Distinguishes domain vs user-query nodes
//! - `GraphData`: Core graph structure with runtime mutation support

pub mod algorithms;
pub mod builder;
pub mod extractor;
pub mod persistence;
pub mod query;
pub mod stats;
pub mod types;
pub mod validation;

// Re-exports — algorithms
pub use algorithms::{
    calculate_centrality, find_bridges, get_related, neighborhood, prerequisites_sorted,
    shortest_path, CentralityScore, NeighborhoodResult, PathResult, PrerequisitesResult,
};

// Re-exports — builder
pub use builder::{BuildError, BuildStats, ErrorHandling, GraphBuilder, ManualEdge};

// Re-exports — extractor
pub use extractor::GraphExtractor;

// Re-exports — persistence
pub use persistence::{
    is_cache_fresh, load_graph, load_graph_from_str, save_graph, GraphMetadata, SerializableGraph,
};

// Re-exports — query
pub use query::{
    CategoryCount, EdgeInfo, GraphInfoResponse, NeighborInfo, NeighborhoodResponse, NodeSummary,
    PathResponse, PathStep, PrerequisiteInfo, PrerequisitesResponse, RelatedConceptsResponse,
    RelatedGroup, RelationshipCount,
};

// Re-exports — stats
pub use stats::{compute_stats, quick_summary, top_nodes_by_degree, DegreeDirection, GraphStats};

// Re-exports — types
pub use types::{Edge, EdgeOrigin, GraphData, Node, NodeType, Relationship};

// Re-exports — validation
pub use validation::{is_valid, validate_graph, ValidationIssue, ValidationResult};

#[cfg(any(test, feature = "test-utils"))]
pub use extractor::mock::{MockEdgeData, MockExtractor, MockNodeData};

#[cfg(feature = "graph-rkyv-cache")]
pub use persistence::rkyv_cache;
