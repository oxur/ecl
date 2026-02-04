//! Full-text search infrastructure for Fabryk.
//!
//! This crate provides search functionality with a Tantivy backend
//! (feature-gated).
//!
//! # Features
//!
//! - `fts-tantivy`: Enable Tantivy-based full-text search
//!
//! # Default Schema
//!
//! Fabryk provides a sensible default schema suitable for knowledge
//! domains. Custom schemas can be added in future versions.

#![doc = include_str!("../README.md")]

// Modules to be added during extraction:
// pub mod backend;
// pub mod schema;
// pub mod document;
// pub mod query;
// #[cfg(feature = "fts-tantivy")]
// pub mod tantivy_search;
// #[cfg(feature = "fts-tantivy")]
// pub mod indexer;
