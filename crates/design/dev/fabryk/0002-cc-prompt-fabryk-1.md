---
title: "CC Prompt: Fabryk 1.2 — Error Types & Result"
milestone: "1.2"
phase: 1
author: "Claude (Opus 4.5)"
created: 2026-02-03
prerequisites: ["1.1 Workspace scaffold"]
governing-docs: [0009-audit §4.1, 0013-project-plan]
---

# CC Prompt: Fabryk 1.2 — Error Types & Result

## Context

Phase 1 extracts shared types, traits, errors, and utilities into `fabryk-core`.
Milestone 1.1 created the workspace scaffold. This milestone extracts the error
types — the simplest extraction because `error.rs` is already fully generic
(Classification: G, no changes needed).

## Source File

```
~/lab/music-comp/ai-music-theory/crates/server/src/error.rs
```

**From Audit §1 (File Inventory):** 182 lines. Types: `Error`, `Result<T>`.
External deps: `thiserror`.

**Classification:** Generic (G) — no domain coupling.

## Objective

1. Extract `error.rs` from the music-theory server into `fabryk-core`
2. Define `fabryk_core::Error` and `fabryk_core::Result<T>`
3. Ensure the error type covers all the error variants needed by downstream
   Fabryk crates (file I/O, config, serialisation, etc.)
4. Verify: `fabryk-core` compiles and error types are exported

## Implementation Steps

### Step 1: Read the source file

```bash
cat ~/lab/music-comp/ai-music-theory/crates/server/src/error.rs
```

Understand the current error variants. Expect to see variants for:

- I/O errors (`std::io::Error`)
- Config/parsing errors (TOML, YAML, JSON)
- Search errors (Tantivy-related)
- Graph errors (petgraph-related)
- General string errors

### Step 2: Create `fabryk-core/src/error.rs`

Copy the file, then review each error variant:

- **Keep** variants that are generic infrastructure errors (I/O, serialisation,
  config loading, "not found", "invalid input")
- **Remove** any variants that are music-theory-specific (unlikely given the G
  classification, but verify)
- **Conditionally compile** variants that depend on optional features. For
  example, if there are Tantivy-specific error variants, those belong in
  `fabryk-fts`, not here. If there are petgraph-specific variants, those belong
  in `fabryk-graph`. Keep `fabryk-core` error lean — only errors for crates
  that `fabryk-core` itself depends on.

The error type should follow this pattern:

```rust
//! Error types for the Fabryk ecosystem.
//!
//! Provides a common `Error` type and `Result<T>` alias used across
//! all Fabryk crates. Domain-specific error types can wrap or extend
//! this as needed.

use std::path::PathBuf;

/// Common error type for Fabryk operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error (file operations, network, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// YAML parsing error.
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// JSON parsing error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Configuration error with descriptive message.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Requested resource not found.
    #[error("{resource_type} not found: {id}")]
    NotFound {
        resource_type: String,
        id: String,
    },

    /// Invalid input or argument.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Path-related error.
    #[error("Path error: {path}: {message}")]
    Path {
        path: PathBuf,
        message: String,
    },

    /// Generic error with message (escape hatch).
    #[error("{0}")]
    Other(String),
}

/// Convenience Result type alias for Fabryk operations.
pub type Result<T> = std::result::Result<T, Error>;
```

Also provide constructor helpers if the original file has them:

```rust
impl Error {
    /// Create a configuration error.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a not-found error.
    pub fn not_found(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            id: id.into(),
        }
    }

    /// Create a path error.
    pub fn path(path: impl Into<PathBuf>, msg: impl Into<String>) -> Self {
        Self::Path {
            path: path.into(),
            message: msg.into(),
        }
    }
}
```

**Key decision:** If the original `error.rs` has error variants for Tantivy,
petgraph, rkyv, or other crate-specific errors, do **not** include them here.
Those will be defined in their respective Fabryk crates (`fabryk-fts`,
`fabryk-graph`). The crate-specific error types can contain a
`Core(fabryk_core::Error)` variant or use `#[from]` to wrap the core error.

### Step 3: Update `fabryk-core/src/lib.rs`

```rust
//! Fabryk Core — shared types, traits, errors, and utilities.
//!
//! This crate provides the foundational types used across all Fabryk crates.
//! It has no internal Fabryk dependencies (dependency level 0).

pub mod error;

// Re-export key types at crate root for convenience
pub use error::{Error, Result};
```

### Step 4: Update `fabryk-core/Cargo.toml`

Ensure the dependencies include what the error module needs:

```toml
[dependencies]
thiserror = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
serde_yaml = { workspace = true }
```

### Step 5: Add tests

If the original `error.rs` has tests, bring them along. If not, add basic tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::config("missing field");
        assert_eq!(err.to_string(), "Configuration error: missing field");
    }

    #[test]
    fn test_not_found_display() {
        let err = Error::not_found("Concept", "major-triad");
        assert_eq!(err.to_string(), "Concept not found: major-triad");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_result_alias() {
        let ok: Result<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);

        let err: Result<i32> = Err(Error::config("bad"));
        assert!(err.is_err());
    }
}
```

### Step 6: Verify

```bash
cd ~/lab/music-comp/fabryk
cargo check -p fabryk-core
cargo test -p fabryk-core
cargo clippy -p fabryk-core -- -D warnings
```

## Exit Criteria

- [ ] `fabryk-core/src/error.rs` exists with `Error` enum and `Result<T>` alias
- [ ] Error type covers: I/O, YAML, JSON, Config, NotFound, InvalidInput, Path, Other
- [ ] No domain-specific (music-theory) error variants present
- [ ] No Tantivy/petgraph/rkyv-specific error variants (those go in downstream crates)
- [ ] Constructor helpers provided (`Error::config()`, `Error::not_found()`, etc.)
- [ ] `cargo test -p fabryk-core` passes
- [ ] `cargo clippy -p fabryk-core -- -D warnings` clean

## Commit Message

```
feat(core): extract error types and Result alias

Extract error.rs from music-theory MCP server into fabryk-core.
Provides Error enum with common infrastructure variants (I/O, YAML,
JSON, config, not-found, path) and Result<T> alias.

No domain-specific error variants — downstream crates define their own.

Ref: Doc 0013 milestone 1.2, Audit §4.1
```
