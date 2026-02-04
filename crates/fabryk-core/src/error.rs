//! Error types for Fabryk operations.
//!
//! This module provides a common `Error` type and `Result<T>` alias used across
//! all Fabryk crates. Uses `thiserror` for derive macros.
//!
//! **Note**: This is a minimal stub. See milestone 1.2 for the full extraction.

use thiserror::Error;

/// Errors that can occur in Fabryk operations.
///
/// This is a minimal stub — milestone 1.2 adds additional variants,
/// backtrace support, and inspector methods.
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Content not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Invalid data or format.
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl Error {
    /// Create a configuration error.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a not found error.
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create an invalid data error.
    pub fn invalid_data(msg: impl Into<String>) -> Self {
        Self::InvalidData(msg.into())
    }
}

/// Result type alias using Fabryk's Error type.
pub type Result<T> = std::result::Result<T, Error>;
