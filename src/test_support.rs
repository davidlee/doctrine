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

/// True when running inside a dispatch worker — the `DOCTRINE_WORKER` env leg, which
/// since SL-254 `DEC-207` is the whole of worker identity. Authored-write e2e goldens
/// early-return on this so a worker's own `cargo test` reflects delta health, not the
/// worker-mode guard's (correct) refusals. The server-side commit gate UNSETS the env
/// for its run, so the goldens still execute there — coverage is preserved; only the
/// worker's manual run skips.
///
/// `is_some()` is a deliberately broader test than `env_worker_set()`'s exact `= "1"`:
/// any `DOCTRINE_WORKER` value conservatively skips a golden. (SL-225 #2, DEC-003.)
///
/// The `WORKER_MARKER_REL` carve-out this used to carry (a documented duplicate of
/// `marker.rs`'s marker path, needed because the integration-test crate compiles
/// separately — CHR-014) is gone with the marker: there is no longer a second place
/// for the path to drift out of lockstep with. The NAME is retained despite now being
/// a slight misnomer — it has 112 call sites across 33 test files, and renaming them
/// would be a large diff for no behavioural gain (EX-6: the helper contains the blast
/// radius).
// Consumed only by the separately-compiled integration-test crate (via the `#[path]`
// include in `tests/common/mod.rs`), never by the bin's own `#[cfg(test)]` unit tests —
// so it reads as dead in the bin build. Same cross-crate carve-out as `common`'s
// `#![allow(dead_code)]` (SL-162 D5).
#[allow(dead_code)]
pub(crate) fn under_worker_marker() -> bool {
    worker_env_says_worker(std::env::var_os("DOCTRINE_WORKER").as_deref())
}

/// The pure core of [`under_worker_marker`]: the shell reads the env, this decides.
/// Split out so the decision stays testable — `set_var` is banned crate-wide, so a
/// test cannot mutate the ambient `DOCTRINE_WORKER` to drive the cases.
fn worker_env_says_worker(value: Option<&std::ffi::OsStr>) -> bool {
    value.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    // SL-254 PHASE-05: this was `worker_marker_at_reads_env_and_marker_legs`, a
    // three-case table over (root, env_set). The marker-file leg is gone (`DEC-207`
    // makes `DOCTRINE_WORKER` the whole of worker identity), so the root argument and
    // its two marker rows have no subject left. What survives — and is what the 112
    // call sites depend on — is that the skip-helper keys off `DOCTRINE_WORKER`
    // presence, deliberately BROADER than `marker::env_worker_set`'s exact `= "1"`
    // (SL-225 #2): any value at all conservatively skips an authored-write golden.
    #[test]
    fn under_worker_marker_treats_any_doctrine_worker_value_as_worker() {
        use std::ffi::OsStr;

        assert!(!worker_env_says_worker(None), "unset ⇒ not under a worker");
        assert!(
            worker_env_says_worker(Some(OsStr::new("1"))),
            "`1` ⇒ under a worker"
        );
        assert!(
            worker_env_says_worker(Some(OsStr::new("0"))),
            "broader than `env_worker_set`: even `0` conservatively skips"
        );
        assert!(
            worker_env_says_worker(Some(OsStr::new(""))),
            "broader than `env_worker_set`: even empty conservatively skips"
        );
    }

    /// The shell wired to the pure core above reads the ambient `DOCTRINE_WORKER`
    /// (not some other name) — the one part `set_var`'s crate-wide ban leaves
    /// observable only against the ambient environment.
    #[test]
    fn under_worker_marker_reads_the_doctrine_worker_variable() {
        assert_eq!(
            under_worker_marker(),
            std::env::var_os("DOCTRINE_WORKER").is_some(),
            "the skip-helper's verdict must track `DOCTRINE_WORKER` presence"
        );
    }

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
