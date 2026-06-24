// SPDX-License-Identifier: GPL-3.0-only
//! Test-only helpers shared across the lib unit tests and the integration tests.
//!
//! CHR-014: resolve the repo root at RUNTIME, never via the compile-time
//! `env!("CARGO_MANIFEST_DIR")` macro. The jail shares one `CARGO_TARGET_DIR` across
//! worktrees, so a binary compiled in tree W (with W's path baked by `env!`) can be
//! reused when tests run from another tree — pointing reads at a dead/wrong path once
//! W is reaped. Cargo sets `CARGO_MANIFEST_DIR` in the test process's *runtime* env to
//! the invoking tree, so a runtime read is always correct regardless of which tree
//! compiled the binary.
//!
//! One source: declared `#[cfg(test)] mod test_support;` in `main.rs` for the lib unit
//! tests, and `#[path]`-included by `tests/common/mod.rs` for the integration tests
//! (separate compilation units that cannot see `cfg(test)` items in the lib).

use std::path::PathBuf;

/// The repo root, resolved at runtime. Prefers cargo's runtime `CARGO_MANIFEST_DIR`
/// (set to the invoking tree); falls back to walking up from the CWD to the directory
/// holding `Cargo.toml`, for the rare non-cargo-driven run.
pub(crate) fn repo_root() -> PathBuf {
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        return PathBuf::from(dir);
    }
    let mut cur = std::env::current_dir().expect("resolve current dir");
    loop {
        if cur.join("Cargo.toml").is_file() {
            return cur;
        }
        if !cur.pop() {
            panic!("repo_root: no runtime CARGO_MANIFEST_DIR and no Cargo.toml ancestor of CWD");
        }
    }
}
