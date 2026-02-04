//! Fabryk Core — shared types, traits, errors, and utilities.
//!
//! This crate provides the foundational types used across all Fabryk crates.
//! It has no internal Fabryk dependencies (dependency level 0).
//!
//! # Modules
//!
//! - [`error`]: Error types and Result alias

#![doc = include_str!("../README.md")]

pub mod error;

// Re-export key types at crate root for convenience
pub use error::{Error, Result};

// Modules to be added during extraction:
// pub mod util;
// pub mod traits;
// pub mod state;
