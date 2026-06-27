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

/// Doctrine entity schema keys — single-source per STD-001.
pub(crate) const SCHEMA_BACKLOG: &str = "doctrine.backlog";
pub(crate) const SCHEMA_KNOWLEDGE: &str = "doctrine.knowledge";
pub(crate) const SCHEMA_ADR: &str = "doctrine.adr";
pub(crate) const SCHEMA_RFC: &str = "doctrine.rfc";
pub(crate) const SCHEMA_MEMORY: &str = "doctrine.memory";
pub(crate) const SCHEMA_PLAN: &str = "doctrine.plan";
pub(crate) const SCHEMA_PLAN_OVERVIEW: &str = "doctrine.plan.overview";

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

/// The built `doctrine` binary, resolved at RUNTIME from the running test exe.
/// SL-162 / CHR-014: never bake the path via `env!("CARGO_BIN_EXE_doctrine")` —
/// a shared target serves one artifact across namespaces/profiles, so the baked
/// path NotFounds in the namespace that did not compile it.
pub(crate) fn doctrine_bin() -> PathBuf {
    let mut p = std::env::current_exe().expect("resolve current_exe for doctrine_bin");
    p.pop(); // drop test-exe name → …/deps/
    p.pop(); // drop deps/          → …/<profile>/
    p.push(format!("doctrine{}", std::env::consts::EXE_SUFFIX));
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doctrine_bin_returns_existing_executable() {
        let path = doctrine_bin();

        // File name ends with "doctrine" (+ ".exe" on Windows).
        let name = path.file_name().expect("doctrine_bin path has a file name");
        let name_str = name
            .to_str()
            .expect("doctrine_bin file name is valid UTF-8");
        assert!(
            name_str.starts_with("doctrine"),
            "doctrine_bin file name starts with 'doctrine': {name_str}"
        );

        // The resolved path exists.
        assert!(
            path.exists(),
            "doctrine_bin path exists: {}",
            path.display()
        );

        // It is a file, not a directory.
        let meta = path.metadata().expect("doctrine_bin metadata readable");
        assert!(meta.is_file(), "doctrine_bin is a file: {}", path.display());

        // File size > 0 (non-zero binary).
        assert!(
            meta.len() > 0,
            "doctrine_bin non-zero size: {}",
            path.display()
        );
    }
}
