// SPDX-License-Identifier: GPL-3.0-only
//! IMP-352 — every integration test spawns the binary through one seam, and that
//! seam binds cwd.
//!
//! `worker_guard` resolves its root by walking up from CWD alone
//! (`crate::root::find(None, …)`, `src/commands/guard.rs`; RV-319 F-1 pins that it
//! ignores `-p`). A fixture that spawns the binary without binding `.current_dir()`
//! therefore roots the child in whatever tree the harness happened to stand in,
//! not the scratch root it passes to `-p`. Inside a dispatch worker fork that
//! ambient tree is marked, so the guard refuses an authored write the fixture
//! never aimed there — about ten targets went red in every marked fork, and a
//! worker consequently could not use its own test run as a green signal
//! (ISS-028, ISS-267).
//!
//! The cwd is not the only inherited signal: the env leg (`DOCTRINE_WORKER`) is
//! root-independent, so binding cwd does not reach it. `common::doctrine_cmd`
//! declares BOTH, which is why the rule below is "go through the seam" rather
//! than the weaker, easily-satisfied "call `.current_dir()` somewhere".
//!
//! Two halves, and both are needed:
//!   * a SOURCE scan — the drift is invisible at runtime, because a fixture that
//!     inherits cwd passes perfectly well in the primary tree;
//!   * a BEHAVIOURAL pin — that the seam does what the scan assumes, proved
//!     against a genuine marked linked worktree rather than asserted.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::PathBuf;

mod common;

// ---------------------------------------------------------------------------
// Half one: the source scan
// ---------------------------------------------------------------------------

/// Counts references to `doctrine_bin` in CODE, ignoring attributes — `//!` and
/// `///` comments are parsed as attributes, and this file plus several others
/// legitimately name the helper in prose. A comment is not a spawn.
struct BinRefs {
    hits: usize,
}

impl<'ast> syn::visit::Visit<'ast> for BinRefs {
    /// Doc comments desugar to `#[doc = "…"]`; skipping attributes wholesale is
    /// what keeps prose mentions from reading as violations.
    fn visit_attribute(&mut self, _: &'ast syn::Attribute) {}

    fn visit_ident(&mut self, ident: &'ast syn::Ident) {
        if ident == "doctrine_bin" {
            self.hits += 1;
        }
    }
}

/// Code references to `doctrine_bin` in `text`. An unparseable file counts zero —
/// the compiler already rejects genuinely broken sources, so failing here would
/// only produce a confusing second error.
fn bin_refs(text: &str) -> usize {
    let Ok(ast) = syn::parse_file(text) else {
        return 0;
    };
    let mut visitor = BinRefs { hits: 0 };
    syn::visit::Visit::visit_file(&mut visitor, &ast);
    visitor.hits
}

fn test_sources() -> Vec<PathBuf> {
    let dir = common::repo_root().join("tests");
    let mut out: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("read tests dir")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "rs"))
        .collect();
    out.sort();
    out
}

/// ANTI-VACUITY: the scan must catch a real spawn AND must not fire on prose.
/// Without this, the rule below would pass forever on a detector that had
/// silently stopped recognising anything.
#[test]
fn scan_separates_a_spawn_from_a_mention() {
    const VIOLATION: &str = r#"
        fn run() { let _ = std::process::Command::new(common::doctrine_bin()).output(); }
    "#;
    const WRAPPED: &str = r#"
        fn bin() -> PathBuf { common::doctrine_bin() }
    "#;
    const PROSE_ONLY: &str = r#"
        //! Resolved at runtime via `common::doctrine_bin()`, never `env!`.
        /// See doctrine_bin for why.
        fn run(dir: &Path) { let _ = common::doctrine_cmd(dir).output(); }
    "#;

    assert!(bin_refs(VIOLATION) > 0, "a direct spawn must be caught");
    assert!(
        bin_refs(WRAPPED) > 0,
        "a per-file `fn bin()` wrapper must be caught too — it is how the \
         unbound-cwd spawns hid before IMP-352"
    );
    assert_eq!(
        bin_refs(PROSE_ONLY),
        0,
        "a doc-comment mention is not a spawn; firing on prose would make the \
         rule unsatisfiable for the files that explain it"
    );
}

/// Positive control: the detector works against the REAL seam, not just fixtures.
/// If `common/mod.rs` ever stopped referencing `doctrine_bin`, the rule below
/// would pass vacuously across a tree that had moved on.
#[test]
fn scan_finds_the_seam_it_polices() {
    let files = test_sources();
    assert!(
        files.len() > 80,
        "sanity: expected to scan the whole tests tree, found {}",
        files.len()
    );
    let seam = common::repo_root().join("tests/common/mod.rs");
    let text = std::fs::read_to_string(&seam).expect("read the spawn seam");
    assert!(
        bin_refs(&text) > 0,
        "tests/common/mod.rs must be the one place that resolves the binary path"
    );
}

/// The rule: `doctrine_cmd` is the only way an integration test spawns the
/// binary. Keyed on `doctrine_bin` rather than on the absence of `.current_dir()`
/// because the path resolver is unavoidable — a spawn cannot dodge it — whereas a
/// `.current_dir()` scan is satisfied by binding cwd on one spawn in a file and
/// forgetting it on the next.
#[test]
fn every_test_spawns_the_binary_through_the_seam() {
    let offenders: Vec<String> = test_sources()
        .into_iter()
        .filter(|p| p.file_name().is_some_and(|n| n != "common"))
        .filter(|p| bin_refs(&std::fs::read_to_string(p).expect("read test source")) > 0)
        .map(|p| {
            p.strip_prefix(common::repo_root())
                .unwrap_or(&p)
                .display()
                .to_string()
        })
        .collect();

    assert!(
        offenders.is_empty(),
        "these tests resolve the binary path themselves instead of going through \
         `common::doctrine_cmd(cwd)`. A raw spawn inherits the harness's cwd, so \
         the worker-mode guard roots the child in the ambient tree rather than the \
         scratch root the fixture operates on — which refuses every authored write \
         inside a dispatch worker fork (IMP-352):\n  {}",
        offenders.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// Half two: the behavioural pin
// ---------------------------------------------------------------------------

/// What the scan assumes, proved: cwd decides the guard's verdict, so a fixture
/// rooted in its own scratch tree writes freely even when a marked worker fork is
/// what the harness is standing in — while the SAME command rooted in that fork is
/// still refused.
///
/// Both arms go through `doctrine_cmd`, differing only in the root handed to it.
/// That keeps the rule above absolute (no allowlist) and isolates the variable
/// under test to cwd alone: the env leg is stripped on both arms, so the refusal
/// can only be the marker leg.
///
/// The fork is a GENUINE linked worktree carrying the marker
/// (`common::marked_linked_fork` self-validates both legs) — `resolve_mode`
/// requires `is_linked && marker_present`, so a marker file dropped in a bare
/// tempdir would never be refused and this test would prove nothing.
#[test]
fn cwd_decides_the_guard_verdict_for_an_identical_write() {
    let src = tempfile::tempdir().expect("tempdir");
    common::init_repo(src.path());

    let holder = tempfile::tempdir().expect("tempdir");
    let fork = holder.path().join("fork");
    common::marked_linked_fork(src.path(), &fork, "imp352-wkr");

    let scratch = tempfile::tempdir().expect("tempdir");

    // Arm A — rooted in the marked fork (what an unbound cwd inherits inside a
    // dispatch worker): the marker leg fires and the write is refused.
    let refused = common::doctrine_cmd(&fork)
        .args(["backlog", "new", "issue", "probe", "-p"])
        .arg(scratch.path())
        .output()
        .expect("spawn doctrine");
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        !refused.status.success(),
        "a write rooted in a marked fork must still be refused — confinement is \
         not what IMP-352 relaxed; stderr: {stderr}"
    );
    assert!(
        stderr.contains("signal: marker"),
        "the refusal must be the MARKER leg (the env leg is stripped on both \
         arms), else this test is measuring the wrong signal; stderr: {stderr}"
    );

    // Arm B — the identical write, rooted in the scratch tree it actually targets.
    let allowed = common::doctrine_cmd(scratch.path())
        .args(["backlog", "new", "issue", "probe", "-p"])
        .arg(scratch.path())
        .output()
        .expect("spawn doctrine");
    assert!(
        allowed.status.success(),
        "a fixture rooted in its own scratch tree must write regardless of the \
         harness's tree; stderr: {}",
        String::from_utf8_lossy(&allowed.stderr)
    );
    assert!(
        scratch.path().join(".doctrine/backlog/issue/001").is_dir(),
        "the write must actually land in the scratch root"
    );
}
