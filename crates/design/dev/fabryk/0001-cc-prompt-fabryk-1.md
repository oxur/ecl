---
title: "CC Prompt: Fabryk 1.1 — Workspace Scaffold"
milestone: "1.1"
phase: 1
author: "Claude (Opus 4.5)"
created: 2026-02-03
prerequisites: None (first milestone)
governing-docs: [0009-audit, 0012-amendment, 0013-project-plan]
---

# CC Prompt: Fabryk 1.1 — Workspace Scaffold

## Context

This is the first milestone of the Fabryk extraction — a project to refactor
domain-agnostic infrastructure out of the music-theory MCP server into a set of
reusable Rust crates. See Doc 0013 (Project Plan Overview) for the full phase
breakdown and Doc 0009 (Extraction Audit) §6.1 for the workspace layout.

**Phase 1** creates `fabryk-core`, the foundation crate with zero internal Fabryk
dependencies. Milestone 1.1 sets up the entire Fabryk workspace with all 10 crate
stubs so that subsequent milestones can focus on extraction without scaffolding
overhead.

## Important: Existing ECL Workspace Stubs

The ECL repo (`~/lab/music-comp/ecl`) already contains Fabryk crate stubs from
an earlier architectural iteration. These stubs have **different content and
structure** from what the extraction plan requires:

| Crate | In Extraction Plan? | Current ECL Stub Content |
|-------|--------------------|-----------------------|
| `fabryk-core` | ✅ Yes | Has `identity.rs`, `item.rs`, `partition.rs`, `tag.rs`, `traits.rs` — **none of which match the extraction targets** |
| `fabryk-mcp` | ✅ Yes | Has `formatting.rs`, `handlers.rs`, `server.rs`, `session.rs` — **different architecture** |
| `fabryk-cli` | ✅ Yes | Has `commands.rs`, `config.rs` — **different architecture** |
| `fabryk-acl` | ✅ Yes (placeholder) | Has `enforcement.rs`, `policy.rs`, `store.rs` — **premature implementation** |
| `fabryk-content` | ✅ Yes | **Does not exist in ECL** |
| `fabryk-fts` | ✅ Yes | **Does not exist in ECL** |
| `fabryk-graph` | ✅ Yes | **Does not exist in ECL** |
| `fabryk-mcp-content` | ✅ Yes | **Does not exist in ECL** |
| `fabryk-mcp-fts` | ✅ Yes | **Does not exist in ECL** |
| `fabryk-mcp-graph` | ✅ Yes | **Does not exist in ECL** |
| `fabryk-storage` | ❌ No | Exists — from prior iteration |
| `fabryk-query` | ❌ No | Exists — from prior iteration |
| `fabryk-api` | ❌ No | Exists — from prior iteration |
| `fabryk-client` | ❌ No | Exists — from prior iteration |
| `fabryk` (umbrella) | Maybe | Exists as re-export crate |

**Decision needed before running this prompt:** Where does the Fabryk workspace
live? Two options:

1. **Separate workspace** (audit §6.1 assumes this): Create `~/lab/music-comp/fabryk/`
   as a standalone workspace. The music-theory Cargo.toml would use
   `fabryk-core = { path = "../fabryk/fabryk-core" }`. The old ECL stubs get
   removed in a separate cleanup.

2. **Within ECL workspace** (Vision doc v2 suggests this): Keep the crates under
   `~/lab/music-comp/ecl/crates/`. Gut the old stubs in place and build up the
   new content. Risk: ECL workspace compilation must still work.

**This prompt assumes Option 1 (separate workspace).** If you choose Option 2,
adjust all paths accordingly and add the ECL stub cleanup as a sub-task.

## Objective

Create the Fabryk workspace with:

- Workspace `Cargo.toml` with all 10 crate members and shared dependency versions
- One `Cargo.toml` + minimal `src/lib.rs` stub per crate
- CI configuration (`.github/workflows/ci.yml`)
- `LICENSE` (dual MIT/Apache-2.0), `README.md` skeleton
- `CLAUDE.md` with project conventions for future CC sessions
- Verify: `cargo check --workspace` succeeds

## Implementation Steps

### Step 1: Create the workspace root

```bash
mkdir -p ~/lab/music-comp/fabryk
cd ~/lab/music-comp/fabryk
git init
```

### Step 2: Create workspace `Cargo.toml`

Reference: Audit §6.1 for the workspace dependencies block.

```toml
[workspace]
members = [
    "fabryk-core",
    "fabryk-content",
    "fabryk-fts",
    "fabryk-graph",
    "fabryk-acl",
    "fabryk-mcp",
    "fabryk-mcp-content",
    "fabryk-mcp-fts",
    "fabryk-mcp-graph",
    "fabryk-cli",
]
resolver = "2"

[workspace.package]
version = "0.1.0-alpha.0"
edition = "2024"
rust-version = "1.85"
authors = ["Duncan McGreggor <duncan@oxur.org>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/oxur/fabryk"

[workspace.dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# Error handling
thiserror = "2"

# Graph
petgraph = "0.7"
rkyv = { version = "0.8", features = ["validation"] }
memmap2 = "0.9"
blake3 = "1"

# Search
tantivy = "0.22"

# Markdown
pulldown-cmark = "0.12"

# MCP
rmcp = { version = "0.1", features = ["server", "transport-io"] }

# CLI
clap = { version = "4", features = ["derive"] }

# Time
chrono = { version = "0.4", features = ["serde"] }

# Logging
twyg = "4"
log = "0.4"

# Dev dependencies
tokio-test = "0.4"
tempfile = "3"
```

**Note on dependency versions:** The audit lists specific versions based on the
music-theory server's current `Cargo.lock`. Before finalising, check the actual
versions in `~/lab/music-comp/ai-music-theory/Cargo.lock` and use those exact
versions to avoid version conflicts during Phase 1.7 integration. Update the
versions above to match what the music-theory project is actually using.

### Step 3: Create crate stubs

For each of the 10 crates, create:

1. `{crate}/Cargo.toml` — with `workspace = true` fields and crate-specific
   dependencies (only add dependencies that are already known; leave others for
   the extraction milestones)
2. `{crate}/src/lib.rs` — with a doc comment describing the crate's purpose

**Crate-specific details:**

| Crate | Known Dependencies | Purpose |
|-------|--------------------|---------|
| `fabryk-core` | `tokio`, `thiserror`, `serde`, `serde_yaml` | Types, traits, errors, utilities |
| `fabryk-content` | `fabryk-core`, `pulldown-cmark`, `serde`, `serde_yaml` | Markdown parsing, frontmatter |
| `fabryk-fts` | `fabryk-core`, `serde`, `serde_json`, `async-trait` | Full-text search (Tantivy feature-gated) |
| `fabryk-graph` | `fabryk-core`, `fabryk-content`, `petgraph`, `serde`, `serde_json`, `chrono` | Knowledge graph, algorithms |
| `fabryk-acl` | `fabryk-core` | Placeholder (v0.2/v0.3) |
| `fabryk-mcp` | `fabryk-core`, `rmcp`, `async-trait`, `serde`, `serde_json`, `tokio` | Core MCP infrastructure |
| `fabryk-mcp-content` | `fabryk-core`, `fabryk-content`, `fabryk-mcp`, `serde`, `serde_json` | Content + source MCP tools |
| `fabryk-mcp-fts` | `fabryk-core`, `fabryk-fts`, `fabryk-mcp`, `serde`, `serde_json` | FTS MCP tools |
| `fabryk-mcp-graph` | `fabryk-core`, `fabryk-graph`, `fabryk-mcp`, `serde`, `serde_json` | Graph MCP tools |
| `fabryk-cli` | `fabryk-core`, `clap`, `tokio` | CLI framework |

Each `Cargo.toml` should inherit from workspace where possible:

```toml
[package]
name = "fabryk-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Core types, traits, errors, and utilities for the Fabryk knowledge fabric"

[dependencies]
tokio = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true }
serde_yaml = { workspace = true }
```

For `fabryk-fts`, feature-gate Tantivy:

```toml
[features]
default = []
tantivy = ["dep:tantivy"]

[dependencies]
tantivy = { workspace = true, optional = true }
```

For `fabryk-graph`, feature-gate rkyv cache:

```toml
[features]
default = []
rkyv-cache = ["dep:rkyv", "dep:memmap2", "dep:blake3"]

[dependencies]
rkyv = { workspace = true, optional = true }
memmap2 = { workspace = true, optional = true }
blake3 = { workspace = true, optional = true }
```

### Step 4: Create project files

- `README.md` — title, one-paragraph description, crate table, "under
  construction" notice
- `LICENSE-MIT` and `LICENSE-APACHE` — standard dual license files
- `.gitignore` — standard Rust (target/, Cargo.lock for libs)
- `rust-toolchain.toml` — pin to same version as ai-music-theory
- `CLAUDE.md` — conventions doc for future CC sessions:
  - "This is the Fabryk workspace, extracted from the music-theory MCP server"
  - Key constraint: music-theory server must remain functional at every milestone
  - Pointer to governing docs in the design crate
  - `cargo check --workspace` and `cargo test --workspace` commands
  - clippy and fmt conventions

### Step 5: Create CI configuration

`.github/workflows/ci.yml`:

- Trigger on push and PR to main
- Matrix: stable + MSRV (match `rust-version` in workspace)
- Steps: `cargo check --workspace`, `cargo test --workspace`,
  `cargo clippy --workspace -- -D warnings`, `cargo fmt -- --check`
- Feature flag combinations: default, `fabryk-fts/tantivy`,
  `fabryk-graph/rkyv-cache`, all features

### Step 6: Verify

```bash
cd ~/lab/music-comp/fabryk
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

All three should pass (stubs are empty but valid).

## Exit Criteria

- [ ] Fabryk workspace exists with all 10 crate stubs
- [ ] `cargo check --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] Each crate has a `Cargo.toml` with appropriate workspace inheritance
- [ ] Feature flags configured for `fabryk-fts` (tantivy) and `fabryk-graph` (rkyv-cache)
- [ ] `README.md`, `LICENSE-*`, `.gitignore`, `rust-toolchain.toml`, `CLAUDE.md` present
- [ ] CI workflow configured
- [ ] Initial commit made on `main` branch

## Commit Message

```
feat: initialise Fabryk workspace with 10 crate stubs

Create the Fabryk workspace structure for extracting domain-agnostic
infrastructure from the music-theory MCP server.

Crates: fabryk-core, fabryk-content, fabryk-fts, fabryk-graph,
fabryk-acl, fabryk-mcp, fabryk-mcp-content, fabryk-mcp-fts,
fabryk-mcp-graph, fabryk-cli.

All crates are stubs with Cargo.toml and empty lib.rs. Workspace
dependencies are centralised. Feature flags configured for tantivy
(fabryk-fts) and rkyv-cache (fabryk-graph).

Ref: Doc 0013 milestone 1.1, Audit §6.1
```
