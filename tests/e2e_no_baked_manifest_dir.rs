// SPDX-License-Identifier: GPL-3.0-only
//! CHR-014 regression guard — no source may resolve a path via the compile-time
//! `env!("CARGO_MANIFEST_DIR")` macro.
//!
//! The jail shares one `CARGO_TARGET_DIR` across worktrees. A binary compiled in
//! tree W bakes W's path via `env!`, then cargo's fingerprint reuses that binary
//! when tests run from another tree — reads land at a dead/wrong path once W is
//! reaped (`read template /tmp/<removed-worktree>/...: No such file`). The fix is
//! to resolve the repo root at RUNTIME (`test_support::repo_root`, which reads the
//! runtime `CARGO_MANIFEST_DIR` cargo sets to the invoking tree). This guard fails
//! if the compile-time macro creeps back in.
//!
//! Scope: footgun #1 (path-baking) only. Footgun #2 (cargo fingerprint serving a
//! stale *artifact* across worktrees) is IMP-004, mitigated by `just rebuild-stale`.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

mod common;

use std::path::{Path, PathBuf};

/// The forbidden macro form, assembled from fragments so this guard file does not
/// match its own scan.
fn needle() -> String {
    format!("env!({:?})", "CARGO_MANIFEST_DIR")
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
fn no_compile_time_manifest_dir() {
    let root = common::repo_root();
    let needle = needle();
    let mut files = Vec::new();
    rs_files(&root.join("src"), &mut files);
    rs_files(&root.join("tests"), &mut files);

    let offenders: Vec<String> = files
        .iter()
        .filter(|p| code_contains(p, &needle))
        .map(|p| p.strip_prefix(&root).unwrap_or(p).display().to_string())
        .collect();

    assert!(
        offenders.is_empty(),
        "compile-time {needle} is banned (CHR-014) — resolve via test_support::repo_root() at runtime instead. Offenders:\n  {}",
        offenders.join("\n  ")
    );
}
