---
title: "CC Prompt: Fabryk 1.4 — Path Utilities"
milestone: "1.4"
phase: 1
author: "Claude (Opus 4.5)"
created: 2026-02-03
prerequisites: ["1.1 Workspace scaffold", "1.2 Error types", "1.3 File utilities"]
governing-docs: [0009-audit §4.1, 0012-amendment §2f-i, 0013-project-plan]
---

# CC Prompt: Fabryk 1.4 — Path Utilities

## Context

Continuing `fabryk-core` extraction. This milestone extracts path resolution
utilities and adds the `compute_id()` function. Unlike the previous milestones,
`paths.rs` requires **parameterisation** — it currently has hardcoded
`MUSIC_THEORY_*` environment variable prefixes that must become configurable.

**Classification:** Parameterized (P) — requires env var prefix generalisation.

## Source File

```
~/lab/music-comp/ai-music-theory/crates/server/src/util/paths.rs
```

**From Audit §1:** 535 lines. Functions: `server_root()`, `project_root()`,
`expand_tilde()`. External deps: `std::path`.

## Objective

1. Extract `util/paths.rs` into `fabryk-core::util::paths`
2. Parameterise: replace hardcoded `MUSIC_THEORY_*` env var prefixes with a
   configurable project name
3. Add `compute_id()` utility to `fabryk-core::util::ids` (Amendment §2f-i)
4. Verify: path tests pass with parameterised env vars

## Implementation Steps

### Step 1: Read the source file

```bash
cat ~/lab/music-comp/ai-music-theory/crates/server/src/util/paths.rs
```

Identify all occurrences of hardcoded project-specific strings. Expect to find:

- `MUSIC_THEORY_CONFIG_DIR` or similar env var names
- `MUSIC_THEORY_DATA_DIR` or similar
- `"music-theory"` or `"ai-music-theory"` as literal strings
- Possibly hardcoded paths like `~/.config/music-theory/`

### Step 2: Design the parameterisation approach

The path functions need a project name to construct env var names and default
paths. Two options:

**Option A — Function parameter:**

```rust
/// Resolve the project root directory.
///
/// Checks (in order):
/// 1. `{PROJECT_NAME}_ROOT` environment variable
/// 2. Current working directory
pub fn project_root(project_name: &str) -> Result<PathBuf> {
    let env_var = format!("{}_ROOT", project_name.to_uppercase().replace('-', "_"));
    if let Ok(path) = std::env::var(&env_var) {
        return Ok(PathBuf::from(path));
    }
    // fallback...
}
```

**Option B — Builder/Config struct:**

```rust
pub struct PathResolver {
    project_name: String,
    env_prefix: String,
}

impl PathResolver {
    pub fn new(project_name: &str) -> Self {
        let env_prefix = project_name.to_uppercase().replace('-', "_");
        Self {
            project_name: project_name.to_string(),
            env_prefix,
        }
    }

    pub fn project_root(&self) -> Result<PathBuf> { ... }
    pub fn config_dir(&self) -> Result<PathBuf> { ... }
    pub fn data_dir(&self) -> Result<PathBuf> { ... }
}
```

**Recommendation:** Option B is cleaner if there are many path functions that
all need the project name. If there are only 2-3 functions, Option A is simpler.
Read the source and decide based on the actual API surface.

Whichever you choose, ensure the music-theory server can use it by passing
`"music-theory"` (or `"MUSIC_THEORY"`) as the project name.

### Step 3: Create `fabryk-core/src/util/paths.rs`

Copy the file, then:

1. Replace every hardcoded `"MUSIC_THEORY"` or `"music_theory"` with the
   parameterised version
2. Replace every hardcoded path fragment like `"music-theory"` with the
   project name parameter
3. Keep all the generic utility functions (`expand_tilde()`, etc.) unchanged
4. Update error messages to include the project name dynamically

### Step 4: Create `fabryk-core/src/util/ids.rs` (Amendment §2f-i)

This is a **new** utility not present in the original music-theory codebase as a
standalone function — it's inline in `graph/parser.rs`. Extract it as a reusable
utility.

```rust
//! ID computation utilities.
//!
//! Provides functions for generating stable, human-readable identifiers
//! from file paths. Used by GraphExtractor implementations and content
//! loaders.

use std::path::Path;
use crate::Result;

/// Compute a stable, human-readable ID from a file path.
///
/// Given a base path and a file path, produces an ID by:
/// 1. Stripping the base path prefix
/// 2. Removing the file extension
/// 3. Using the filename stem (not the full relative path) as the ID
/// 4. Normalising to lowercase kebab-case
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use fabryk_core::util::ids::compute_id;
///
/// let base = Path::new("/data/concepts");
/// let file = Path::new("/data/concepts/harmony/picardy-third.md");
/// assert_eq!(compute_id(base, file).unwrap(), "picardy-third");
///
/// let file2 = Path::new("/data/concepts/scales/Major_Scale.md");
/// assert_eq!(compute_id(base, file2).unwrap(), "major-scale");
/// ```
///
/// # Errors
///
/// Returns an error if `file_path` does not start with `base_path` or
/// if the file has no stem.
pub fn compute_id(base_path: &Path, file_path: &Path) -> Result<String> {
    let stem = file_path
        .file_stem()
        .ok_or_else(|| crate::Error::path(file_path, "file has no stem"))?
        .to_string_lossy();

    // Normalise to lowercase kebab-case
    let id = stem
        .to_lowercase()
        .replace('_', "-")
        .replace(' ', "-");

    Ok(id)
}
```

**Note:** Read the actual `compute_id` logic from `graph/parser.rs` in the
music-theory server. The implementation above is an approximation based on the
audit description. The actual logic may be more nuanced (e.g., handling nested
directories, deduplication of hyphens, etc.).

```bash
grep -n "compute_id\|fn.*id.*path" \
  ~/lab/music-comp/ai-music-theory/crates/server/src/graph/parser.rs
```

### Step 5: Update `util/mod.rs`

```rust
//! Utility modules for file operations, path handling, ID computation,
//! and common helpers.

pub mod files;
pub mod ids;
pub mod paths;
```

### Step 6: Update `fabryk-core/src/lib.rs`

No structural change needed (already exports `util`), but verify the re-exports
are appropriate. Consider adding convenience re-exports:

```rust
pub use util::ids::compute_id;
```

### Step 7: Add tests

**For paths:** Port existing tests from the source file, updating them to use
the parameterised API:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_root_from_env() {
        // Test with a custom project name
        std::env::set_var("TEST_PROJECT_ROOT", "/tmp/test-project");
        let root = project_root("test-project").unwrap();
        assert_eq!(root, PathBuf::from("/tmp/test-project"));
        std::env::remove_var("TEST_PROJECT_ROOT");
    }

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/Documents");
        assert!(!expanded.to_string_lossy().contains('~'));
    }
}
```

**For ids:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_compute_id_simple() {
        let base = Path::new("/data/concepts");
        let file = Path::new("/data/concepts/picardy-third.md");
        assert_eq!(compute_id(base, file).unwrap(), "picardy-third");
    }

    #[test]
    fn test_compute_id_normalises_underscores() {
        let base = Path::new("/data");
        let file = Path::new("/data/Major_Scale.md");
        assert_eq!(compute_id(base, file).unwrap(), "major-scale");
    }

    #[test]
    fn test_compute_id_normalises_case() {
        let base = Path::new("/data");
        let file = Path::new("/data/PicardyThird.md");
        assert_eq!(compute_id(base, file).unwrap(), "picardythird");
    }
}
```

### Step 8: Verify

```bash
cd ~/lab/music-comp/fabryk
cargo test -p fabryk-core
cargo clippy -p fabryk-core -- -D warnings
```

## Exit Criteria

- [ ] `fabryk-core/src/util/paths.rs` exists with parameterised path functions
- [ ] No hardcoded `MUSIC_THEORY` or `music-theory` strings remain
- [ ] Path functions accept a project name parameter (via function arg or struct)
- [ ] `expand_tilde()` and other generic utilities are preserved
- [ ] `fabryk-core/src/util/ids.rs` exists with `compute_id()` function
- [ ] `compute_id()` is tested with basic cases (simple path, underscores, case)
- [ ] Path tests pass with parameterised env vars
- [ ] `cargo test -p fabryk-core` passes
- [ ] `cargo clippy -p fabryk-core -- -D warnings` clean

## Commit Message

```
feat(core): extract path utilities and add compute_id

Extract util/paths.rs from music-theory MCP server into
fabryk-core::util::paths. Parameterised: hardcoded MUSIC_THEORY_*
env var prefixes replaced with configurable project name.

Added fabryk-core::util::ids::compute_id() for generating stable
human-readable IDs from file paths (Amendment §2f-i).

Ref: Doc 0013 milestone 1.4, Audit §4.1, Amendment §2f-i
```
