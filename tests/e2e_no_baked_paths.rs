// SPDX-License-Identifier: GPL-3.0-only
//! CHR-014 / SL-162 regression guard — no source may resolve a path via the
//! compile-time `env!("CARGO_MANIFEST_DIR")` or `env!("CARGO_BIN_EXE_doctrine")`
//! macros.
//!
//! The jail shares one `CARGO_TARGET_DIR` across worktrees. A binary compiled in
//! tree W bakes W's path via `env!`, then cargo's fingerprint reuses that binary
//! when tests run from another tree — reads land at a dead/wrong path once W is
//! reaped. The fix is to resolve at RUNTIME:
//!   - `test_support::repo_root()` for the repo root (reads runtime
//!     `CARGO_MANIFEST_DIR` cargo sets to the invoking tree).
//!   - `test_support::doctrine_bin()` for the binary path (resolved from
//!     `current_exe()` sibling, profile/target-dir agnostic).
//! This guard fails if either compile-time macro creeps back in.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

mod common;

use std::path::{Path, PathBuf};

/// The forbidden macro forms, assembled from fragments so this guard file does not
/// match its own scan.
fn needles() -> [String; 2] {
    [
        format!("env!({:?})", "CARGO_MANIFEST_DIR"),
        // The `CARGO_BIN_EXE` macro is used as `CARGO_BIN_EXE_doctrine` in tests;
        // search for the common prefix, assembled from fragments so the guard's own
        // prose does not self-match.
        format!("{}_BIN_EXE", "CARGO"),
    ]
}

/// True if a non-comment line of `path` contains `needle`. Comment lines (`//`,
/// `///`, `//!`) are skipped so the prose that documents the rule does not trip it.
fn code_contains(path: &Path, needle: &str) -> bool {
    std::fs::read_to_string(path).is_ok_and(|t| {
        t.lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .any(|l| l.contains(needle))
    })
}

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn no_baked_paths() {
    let root = common::repo_root();
    let needles = needles();
    let mut files = Vec::new();
    rs_files(&root.join("src"), &mut files);
    rs_files(&root.join("tests"), &mut files);

    let mut offenders: Vec<String> = Vec::new();
    for needle in &needles {
        let mut found: Vec<String> = files
            .iter()
            .filter(|p| code_contains(p, needle))
            .map(|p| p.strip_prefix(&root).unwrap_or(p).display().to_string())
            .collect();
        offenders.append(&mut found);
    }
    offenders.sort();
    offenders.dedup();

    assert!(
        offenders.is_empty(),
        "compile-time path-baking env! macros are banned (CHR-014 / SL-162) — \
         resolve via test_support::repo_root() / test_support::doctrine_bin() \
         at runtime instead. Offenders:\n  {}",
        offenders.join("\n  ")
    );
}
