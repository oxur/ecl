---
title: "CC Prompt: Fabryk 1.7 — Music Theory Integration"
milestone: "1.7"
phase: 1
author: "Claude (Opus 4.5)"
created: 2026-02-03
prerequisites: ["1.1–1.6 completed"]
governing-docs: [0009-audit §7 Phase 2, 0013-project-plan]
---

# CC Prompt: Fabryk 1.7 — Music Theory Integration

## Context

This is the **integration milestone** — the most complex step in Phase 1. All
previous milestones built up `fabryk-core` in isolation. Now we wire it into the
music-theory MCP server, replacing local modules with `fabryk-core` imports.

**Key constraint (Doc 0013):** The music-theory MCP server must remain fully
functional after this milestone. All 25 MCP tools must work.
`cargo test --all-features` must pass in both repos.

## Objective

1. Add `fabryk-core` as a path dependency to ai-music-theory
2. Implement `ConfigProvider` for the music-theory `Config` type
3. Update all imports from local extracted modules to `fabryk_core::`
4. Remove extracted files from the music-theory repo
5. Verify: `cargo test --all-features` passes in **both** repos

## Source Files to Modify (ai-music-theory)

These files currently import from the local modules being replaced:

```
crates/server/src/config.rs          — will implement ConfigProvider
crates/server/src/lib.rs             — module declarations change
crates/server/src/state.rs           — replaced by fabryk_core::state
crates/server/src/error.rs           — replaced by fabryk_core::error
crates/server/src/util/files.rs      — replaced by fabryk_core::util::files
crates/server/src/util/paths.rs      — replaced by fabryk_core::util::paths
crates/server/src/util/mod.rs        — replaced by fabryk_core::util
crates/server/src/resources/mod.rs   — replaced by fabryk_core::resources
```

Plus every file that **imports** from these modules (essentially every `.rs`
file in the crate).

## Implementation Steps

### Step 0: Create a working branch

```bash
cd ~/lab/music-comp/ai-music-theory
git checkout -b feature/fabryk-core-integration
```

### Step 1: Add fabryk-core dependency

Edit `crates/server/Cargo.toml`:

```toml
[dependencies]
# Fabryk dependencies
fabryk-core = { path = "../../fabryk/fabryk-core" }

# ... keep all existing dependencies
```

**Important:** Adjust the relative path based on the actual directory layout.
If Fabryk is at `~/lab/music-comp/fabryk` and ai-music-theory is at
`~/lab/music-comp/ai-music-theory`, then from `crates/server/` the path is
`../../../fabryk/fabryk-core`.

Verify the path is correct:

```bash
ls $(cd ~/lab/music-comp/ai-music-theory/crates/server && \
     realpath ../../../fabryk/fabryk-core/Cargo.toml)
```

### Step 2: Implement `ConfigProvider` for music-theory `Config`

Add a trait implementation in `config.rs` (or a new file
`config_provider_impl.rs` if you prefer separation):

```rust
use fabryk_core::traits::ConfigProvider;

impl ConfigProvider for Config {
    fn project_name(&self) -> &str {
        "music-theory"
    }

    fn base_path(&self) -> fabryk_core::Result<PathBuf> {
        // Use whatever the current Config provides for data directory.
        // Read the actual Config struct to determine the right field.
        Ok(self.data_dir.clone())  // adjust field name to match actual code
    }

    fn content_path(&self, content_type: &str) -> fabryk_core::Result<PathBuf> {
        // Map content_type to the appropriate subdirectory
        Ok(self.base_path()?.join(content_type))
    }
}
```

**Read `config.rs` carefully** to understand the actual struct fields. The trait
methods should delegate to existing Config methods/fields wherever possible.

### Step 3: Audit every import site

Find all files that import from the modules we're replacing:

```bash
cd ~/lab/music-comp/ai-music-theory/crates/server/src

# Find imports of local error module
grep -rn "use crate::error" .
grep -rn "crate::Error\|crate::Result" .

# Find imports of local util module
grep -rn "use crate::util" .
grep -rn "crate::util::files\|crate::util::paths" .

# Find imports of local state module
grep -rn "use crate::state" .
grep -rn "crate::state::AppState" .

# Find imports of local resources module
grep -rn "use crate::resources" .
```

This will produce a comprehensive list of every file and line that needs
updating.

### Step 4: Update imports systematically

For each file, replace local imports with `fabryk_core` imports:

| Old Import | New Import |
|-----------|-----------|
| `use crate::error::{Error, Result}` | `use fabryk_core::{Error, Result}` |
| `use crate::error::Error` | `use fabryk_core::Error` |
| `crate::Result<T>` | `fabryk_core::Result<T>` |
| `use crate::util::files::{find_file_by_id, ...}` | `use fabryk_core::util::files::{find_file_by_id, ...}` |
| `use crate::util::paths::{server_root, ...}` | `use fabryk_core::util::paths::{server_root, ...}` |
| `use crate::state::AppState` | `use fabryk_core::state::AppState` |
| `use crate::resources::serve_resource` | `use fabryk_core::resources::serve_resource` |

**Approach:** Use a combination of `sed` and manual review. Don't blindly
find-and-replace — some files may have local re-exports or aliases that need
careful handling.

Suggested order:
1. Start with `error.rs` imports (most pervasive)
2. Then `util` imports
3. Then `state` imports
4. Then `resources` imports
5. Check `lib.rs` and `mod.rs` files last (module declarations)

### Step 5: Handle `AppState` genericisation

The current `AppState` in music-theory likely holds more than just config
(search backend, graph state, etc.). After this milestone, the music-theory
codebase should:

- **Use `fabryk_core::state::AppState<Config>`** for the config wrapper
- **Keep domain-specific state** (search, graph) as additional fields or in
  a wrapper struct

Two approaches:

**Option A — Extend AppState locally:**

```rust
// In ai-music-theory, create a local wrapper
use fabryk_core::state::AppState as CoreAppState;

pub struct AppState {
    core: CoreAppState<Config>,
    search: Arc<RwLock<Option<SearchBackend>>>,
    graph: Arc<RwLock<Option<GraphState>>>,
}

impl AppState {
    pub fn config(&self) -> &Config {
        self.core.config()
    }
    // ... delegate + add search/graph accessors
}
```

**Option B — Use AppState<Config> directly, pass search/graph separately:**

The other subsystems receive search and graph state through their own
parameters rather than through AppState. This is cleaner but requires more
call-site changes.

**Recommendation:** Choose based on how deeply `AppState` is threaded through
the codebase. If it's passed to dozens of functions, Option A minimises churn.
If it's only used in a few places, Option B is cleaner for the long term.

### Step 6: Handle path utility parameterisation

After milestone 1.4, path functions require a project name parameter. Update
call sites in music-theory:

```rust
// Before (hardcoded)
let root = server_root()?;

// After (parameterised — adjust based on 1.4 design choice)
let root = server_root("music-theory")?;
// Or:
let resolver = PathResolver::new("music-theory");
let root = resolver.server_root()?;
```

Find all call sites:

```bash
grep -rn "server_root\|project_root\|config_dir\|data_dir" \
  ~/lab/music-comp/ai-music-theory/crates/server/src/
```

### Step 7: Remove extracted files from music-theory

Only after imports are updated and the crate compiles:

```bash
cd ~/lab/music-comp/ai-music-theory/crates/server/src

# Remove extracted files
rm error.rs
rm util/files.rs
rm util/paths.rs
rm util/mod.rs       # if util/ is now empty
rmdir util/          # if empty
rm resources/mod.rs
rmdir resources/     # if empty
rm state.rs          # only the parts that moved; keep if still has domain state
```

**Be careful with `state.rs`:** If it still contains domain-specific state
(search backend, graph state), don't delete the whole file. Only remove the
parts that were extracted and keep the domain-specific wrapper.

Update `lib.rs` to remove module declarations for deleted files:

```rust
// Remove these lines from lib.rs:
// mod error;        ← now from fabryk_core
// mod util;         ← now from fabryk_core
// mod resources;    ← now from fabryk_core
// mod state;        ← if fully replaced; keep if domain wrapper remains
```

### Step 8: Verify compilation

```bash
cd ~/lab/music-comp/ai-music-theory
cargo check --all-features
```

Fix any compilation errors. Common issues:
- Missing re-exports (things that were `pub use crate::error::Error` in lib.rs)
- Type mismatches (fabryk_core::Error vs. local Error if not all references updated)
- Trait bounds (functions expecting `Config` now need `C: ConfigProvider`)

### Step 9: Run the full test suite

```bash
# Music theory tests
cd ~/lab/music-comp/ai-music-theory
cargo test --all-features
cargo clippy --all-features -- -D warnings

# Fabryk tests (sanity check — should still pass)
cd ~/lab/music-comp/fabryk
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

### Step 10: Functional verification

If possible, start the MCP server and verify basic functionality:

```bash
cd ~/lab/music-comp/ai-music-theory
cargo run --features fts,graph -- serve
```

Verify at minimum:
- Server starts without errors
- Health tool responds
- A content query works (e.g., `list_concepts`)
- Search works (e.g., `search_concepts`)

### Step 11: Verify no duplicate code

```bash
# Check that the extracted functions don't exist in both repos
grep -rn "fn find_file_by_id" \
  ~/lab/music-comp/ai-music-theory/crates/server/src/
# Should return nothing

grep -rn "fn find_file_by_id" \
  ~/lab/music-comp/fabryk/fabryk-core/src/
# Should return the function definition
```

Repeat for other key functions: `find_all_files`, `server_root`, `project_root`,
`expand_tilde`, `compute_id`, `serve_resource`.

## Exit Criteria

- [ ] ai-music-theory depends on `fabryk-core` (path dependency)
- [ ] `ConfigProvider` implemented for music-theory `Config`
- [ ] All imports updated from local modules to `fabryk_core::`
- [ ] Extracted files removed from music-theory repo (error.rs, util/*, resources/*, state.rs parts)
- [ ] No duplicate code between repos for extracted functionality
- [ ] `cargo test --all-features` passes in ai-music-theory
- [ ] `cargo test --workspace` passes in fabryk
- [ ] `cargo clippy --all-features -- -D warnings` clean in ai-music-theory
- [ ] `cargo clippy --workspace -- -D warnings` clean in fabryk
- [ ] All 25 MCP tools functional (manual or automated verification)

## Phase 1 Exit Criteria (from Doc 0013)

After this milestone, all Phase 1 criteria should be met:

- [x] `fabryk-core` compiles and all tests pass
- [ ] ai-music-theory depends on `fabryk-core` for errors, utilities, paths, state
- [ ] No duplicate code between repos for extracted files
- [ ] `cargo clippy` clean in both repos

## Commit Message

```
feat: integrate fabryk-core into ai-music-theory

Add fabryk-core as path dependency. Implement ConfigProvider for
music-theory Config. Update all imports from local error, util,
state, and resources modules to fabryk_core::.

Remove extracted files from music-theory repo. All 25 MCP tools
verified functional.

BREAKING: Internal module paths changed. No public API change.

Ref: Doc 0013 milestone 1.7, Audit §7 Phase 2
```

## Risk Mitigation

This milestone has the highest risk in Phase 1 because it touches every file in
the music-theory codebase. To reduce risk:

1. **Compile early and often** — don't try to update all imports in one shot.
   Do it module by module: error first, then util, then state, then resources.
   Compile after each group.

2. **Keep the old files initially** — rather than deleting files upfront, first
   update all imports to point to `fabryk_core`. Once the crate compiles and
   tests pass with the old files still present (unused), then delete them.
   This way, if something goes wrong, you can revert easily.

3. **Git commit at checkpoints** — commit after each successful compile:
   - "wip: update error imports to fabryk_core"
   - "wip: update util imports to fabryk_core"
   - "wip: update state imports to fabryk_core"
   - "wip: remove extracted files"
   Then squash into the final commit.

4. **Run the MCP server** — don't just rely on tests. Actually start the server
   and hit a few tools to verify runtime behaviour.
