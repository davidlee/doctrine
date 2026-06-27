// SPDX-License-Identifier: GPL-3.0-only
//! SL-163 PHASE-02 regression guard — the shipped skill corpus (`plugins/**`)
//! carries NO repo-local couplings. POL-002 platform-independence: skills
//! materialised into a client project must rest on a contract doctrine owns, not
//! on this repo's habits.
//!
//! Two banned forms:
//!   - this repo's task-runner gate (`just check` / `just gate`) — a client has
//!     no justfile; skills invoke `doctrine check ...` instead (SL-163 the verb).
//!   - a bare `mem_<uid>` citation — a doctrine-repo-local memory uid that does
//!     not exist in a client corpus, so the reference dangles on install. The
//!     `[[mem.<dotted.key>]]` wikilinks are SANCTIONED (they ship/seed fine) and
//!     are NOT matched: `mem_` (underscore) discriminates a bare uid from both a
//!     dotted `mem.` key and the word `memory_`.
//!
//! Needles are assembled from fragments so this guard file does not self-match;
//! it also scans only `plugins/**`, never `tests/`. Rides the
//! `e2e_no_baked_paths.rs` precedent.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

mod common;

use std::path::{Path, PathBuf};

/// The forbidden shipped-surface forms, assembled from fragments so this guard
/// file does not match its own scan.
fn needles() -> [String; 3] {
    [
        format!("{} {}", "just", "check"),
        format!("{} {}", "just", "gate"),
        // Bare memory uid prefix: `mem` + `_`. Discriminates a repo-local uid
        // (`mem_019...`) from a sanctioned `[[mem.<key>]]` wikilink and from the
        // word `memory_`, neither of which contains the `mem_` substring.
        format!("{}_", "mem"),
    ]
}

/// True if any line of `path` contains `needle`. Plain-text scan (Markdown has no
/// comment syntax to exclude); the guard scans only `plugins/**`, so its own
/// documenting prose is never in range.
fn file_contains(path: &Path, needle: &str) -> bool {
    std::fs::read_to_string(path).is_ok_and(|t| t.lines().any(|l| l.contains(needle)))
}

fn all_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            all_files(&p, out);
        } else {
            out.push(p);
        }
    }
}

#[test]
fn no_shipped_couplings() {
    let root = common::repo_root();
    let needles = needles();
    let mut files = Vec::new();
    all_files(&root.join("plugins"), &mut files);

    let mut offenders: Vec<String> = Vec::new();
    for needle in &needles {
        for p in files.iter().filter(|p| file_contains(p, needle)) {
            offenders.push(format!(
                "{}: {:?}",
                p.strip_prefix(&root).unwrap_or(p).display(),
                needle
            ));
        }
    }
    offenders.sort();
    offenders.dedup();

    assert!(
        offenders.is_empty(),
        "shipped skills (plugins/**) must carry no repo-local couplings \
         (POL-002 / SL-163) — use `doctrine check {{quick|commit|gate}}` instead \
         of `just ...`, and portable prose instead of a bare `mem_` uid. \
         Offenders:\n  {}",
        offenders.join("\n  ")
    );
}
