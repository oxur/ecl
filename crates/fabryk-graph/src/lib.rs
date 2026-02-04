//! Knowledge graph infrastructure for Fabryk.
//!
//! This crate provides graph storage, traversal algorithms, and
//! persistence using petgraph and optional rkyv caching.
//!
//! # Features
//!
//! - `graph-rkyv-cache`: Enable rkyv-based graph persistence with
//!   content-hash cache validation
//!
//! # Key Abstractions
//!
//! - `GraphExtractor` trait: Domain implementations provide this to
//!   extract nodes and edges from content files
//! - `Relationship` enum: Common relationship types with `Custom(String)`
//!   for domain-specific relationships

#![doc = include_str!("../README.md")]

// Modules to be added during extraction:
// pub mod types;
// pub mod extractor;
// pub mod builder;
// pub mod algorithms;
// pub mod persistence;
// pub mod query;
// pub mod stats;
// pub mod validation;
