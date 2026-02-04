---
title: "CC Prompt: Fabryk 1.3 — File Utilities"
milestone: "1.3"
phase: 1
author: "Claude (Opus 4.5)"
created: 2026-02-03
prerequisites: ["1.1 Workspace scaffold", "1.2 Error types"]
governing-docs: [0009-audit §4.1, 0013-project-plan]
---

# CC Prompt: Fabryk 1.3 — File Utilities

## Context

Continuing `fabryk-core` extraction. Milestone 1.2 established the error types.
This milestone extracts the async file utilities — already fully generic
(Classification: G, no changes needed).

## Source File

```
~/lab/music-comp/ai-music-theory/crates/server/src/util/files.rs
```

**From Audit §1 (File Inventory):** 624 lines. Functions: `find_file_by_id()`,
`find_all_files()`, `FindOptions`. External deps: `tokio::fs`.

**Classification:** Generic (G) — no domain coupling.

Also check:

```
~/lab/music-comp/ai-music-theory/crates/server/src/util/mod.rs
```

This is a 4-line module export file. Read it to understand the module structure.

## Objective

1. Extract `util/files.rs` into `fabryk-core::util::files`
2. Set up the `util` module structure in fabryk-core
3. Ensure all existing tests pass in the new location
4. Verify: `cargo test -p fabryk-core` passes

## Implementation Steps

### Step 1: Read the source files

```bash
cat ~/lab/music-comp/ai-music-theory/crates/server/src/util/files.rs
cat ~/lab/music-comp/ai-music-theory/crates/server/src/util/mod.rs
```

Understand the public API. Expect to find:

- `FindOptions` struct — configurable file search parameters (extension filter,
  recursive flag, etc.)
- `find_file_by_id()` — locate a file by its stem/ID in a directory tree
- `find_all_files()` — collect all matching files from a directory tree
- Possibly other helper functions

### Step 2: Create the module structure

```
fabryk-core/src/
├── error.rs       (from milestone 1.2)
├── util/
│   ├── mod.rs
│   └── files.rs
└── lib.rs
```

### Step 3: Copy and adapt `files.rs`

Copy the file into `fabryk-core/src/util/files.rs`.

Review for any coupling:

- **Import paths:** Replace any `crate::error::` or `crate::Error` imports with
  `crate::error::Error` / `crate::Result` (should already work since we
  extracted the same error module)
- **Domain references:** Scan for any music-theory-specific constants, path
  assumptions, or hardcoded strings. Given the G classification, there should
  be none, but verify.
- **Dependencies:** Ensure all required deps are in `fabryk-core/Cargo.toml`.
  This file likely needs `tokio` with the `fs` feature. It may also use `log`
  for debug logging.

### Step 4: Create `util/mod.rs`

```rust
//! Utility modules for file operations, path handling, and common helpers.

pub mod files;
```

### Step 5: Update `fabryk-core/src/lib.rs`

```rust
//! Fabryk Core — shared types, traits, errors, and utilities.
//!
//! This crate provides the foundational types used across all Fabryk crates.
//! It has no internal Fabryk dependencies (dependency level 0).

pub mod error;
pub mod util;

pub use error::{Error, Result};
```

### Step 6: Update `fabryk-core/Cargo.toml`

Add any new dependencies that `files.rs` requires. At minimum:

```toml
[dependencies]
thiserror = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
serde_yaml = { workspace = true }
tokio = { workspace = true }
log = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
tokio-test = { workspace = true }
```

**Note:** Check whether `files.rs` uses `tokio::fs` (async) or `std::fs` (sync).
If async, `tokio` is needed as a runtime dependency. If the tests use
`#[tokio::test]`, `tokio` is needed in dev-dependencies at minimum.

### Step 7: Run tests

```bash
cd ~/lab/music-comp/fabryk
cargo test -p fabryk-core
cargo clippy -p fabryk-core -- -D warnings
```

If the original file has tests that depend on fixtures or test data, create
equivalent fixtures in `fabryk-core/tests/` or use `tempfile` to generate them
dynamically.

## Exit Criteria

- [ ] `fabryk-core/src/util/files.rs` exists with all file utility functions
- [ ] `FindOptions`, `find_file_by_id()`, `find_all_files()` are public exports
- [ ] No domain-specific code (no music-theory references)
- [ ] All original tests pass in the new location
- [ ] `cargo test -p fabryk-core` passes
- [ ] `cargo clippy -p fabryk-core -- -D warnings` clean

## Commit Message

```
feat(core): extract file utilities

Extract util/files.rs from music-theory MCP server into
fabryk-core::util::files. Provides async file discovery utilities:
find_file_by_id(), find_all_files(), FindOptions.

Already fully generic — no changes needed.

Ref: Doc 0013 milestone 1.3, Audit §4.1
```
