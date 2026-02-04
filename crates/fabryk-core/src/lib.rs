//! Core types, traits, errors, and utilities for the Fabryk knowledge fabric.
//!
//! This crate provides the foundational building blocks used by all other
//! Fabryk crates. It has no internal Fabryk dependencies.
//!
//! # Features
//!
//! - Error types and `Result` alias
//! - File and path utilities
//! - `ConfigProvider` trait for domain configuration
//! - `AppState` for application state management

#![doc = include_str!("../README.md")]

pub mod error;

pub use error::{Error, Result};

// Modules to be added during extraction:
// pub mod util;
// pub mod traits;
// pub mod state;
