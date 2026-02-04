---
title: "CC Prompt: Fabryk 1.6 — Resource Serving"
milestone: "1.6"
phase: 1
author: "Claude (Opus 4.5)"
created: 2026-02-03
prerequisites: ["1.1–1.5 completed"]
governing-docs: [0009-audit §4.1, 0013-project-plan]
---

# CC Prompt: Fabryk 1.6 — Resource Serving

## Context

Continuing `fabryk-core` extraction. This is the last extraction milestone before
integration. The `resources/mod.rs` module handles MCP resource serving — making
files available as resources through the MCP protocol. It's already fully generic.

**Classification:** Generic (G) — no domain coupling.

## Source File

```
~/lab/music-comp/ai-music-theory/crates/server/src/resources/mod.rs
```

**From Audit §1:** 50+ lines. Functions: `serve_resource()`. No external deps
beyond what's already in `fabryk-core`.

## Objective

1. Extract `resources/mod.rs` into `fabryk-core::resources`
2. Ensure the resource serving functions work with `AppState<C: ConfigProvider>`
3. Verify: resource serving tests pass

## Implementation Steps

### Step 1: Read the source file

```bash
cat ~/lab/music-comp/ai-music-theory/crates/server/src/resources/mod.rs
```

Understand:
- What `serve_resource()` does (likely reads a file and returns its content)
- What parameters it takes (likely a path or resource ID, plus app state)
- Whether it references `Config` directly or goes through `AppState`
- Whether it has any music-theory-specific logic

### Step 2: Assess the coupling

Given the small size (50+ lines) and G classification, this should be
straightforward. Check for:

- References to `crate::config::Config` → replace with generic `C: ConfigProvider`
- References to `crate::state::AppState` → should work with `AppState<C>`
- Hardcoded content type assumptions → generalise if present
- MCP-specific types (rmcp) → if present, this might belong in `fabryk-mcp`
  instead of `fabryk-core`. **Read the file carefully before deciding
  placement.**

### Step 3: Decide on placement

The audit places this in `fabryk-core`, but if the module depends on MCP/rmcp
types, it should go in `fabryk-mcp` instead (to avoid pulling rmcp into the core
crate). Make this judgment call based on the actual imports:

- **If it uses only `std`/`tokio` types:** → `fabryk-core::resources` ✓
- **If it uses `rmcp` types:** → defer to Phase 5 (`fabryk-mcp`) and skip this
  milestone (document the decision and move on to 1.7)

### Step 4: Create `fabryk-core/src/resources.rs` (or `resources/mod.rs`)

If the module is small (~50 lines), a single file `resources.rs` is fine.
If it's larger or has sub-modules, use a directory.

Copy the file and adapt:

- Update any `Config` references to use `C: ConfigProvider` generics
- Update any `AppState` references to use `AppState<C>`
- Ensure error types use `fabryk_core::Error`

### Step 5: Update `fabryk-core/src/lib.rs`

```rust
pub mod error;
pub mod resources;
pub mod state;
pub mod traits;
pub mod util;

pub use error::{Error, Result};
pub use traits::ConfigProvider;
```

### Step 6: Add or port tests

```bash
# Check if the source file has tests
grep -n "#\[cfg(test)\]" \
  ~/lab/music-comp/ai-music-theory/crates/server/src/resources/mod.rs
```

If tests exist, port them. If not, add basic tests verifying the public API.

### Step 7: Verify

```bash
cd ~/lab/music-comp/fabryk
cargo test -p fabryk-core
cargo clippy -p fabryk-core -- -D warnings
```

Also verify the full workspace still compiles (in case the resources module
introduced new dependencies):

```bash
cargo check --workspace
```

## Exit Criteria

- [ ] Resource serving module extracted to `fabryk-core` (or deferred to
      `fabryk-mcp` with documented rationale if it depends on rmcp)
- [ ] No domain-specific code
- [ ] Generic over `C: ConfigProvider` where config access is needed
- [ ] Tests pass
- [ ] `cargo test -p fabryk-core` passes
- [ ] `cargo clippy -p fabryk-core -- -D warnings` clean
- [ ] `cargo check --workspace` still passes

## Commit Message

```
feat(core): extract resource serving module

Extract resources/mod.rs from music-theory MCP server into
fabryk-core::resources. Provides generic resource serving utilities.

Already fully generic — no changes needed.

Ref: Doc 0013 milestone 1.6, Audit §4.1
```

## Notes

If this module turns out to be trivially small (< 20 lines, just a re-export
or a thin wrapper), consider whether it warrants its own module or should be
folded into an existing module. Document the decision either way.
