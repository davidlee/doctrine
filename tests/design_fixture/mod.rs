// SPDX-License-Identifier: GPL-3.0-only
//! A started design run in a throwaway tree — the bootstrap the design-run e2e
//! crates share with `e2e_claude_install`.
//!
//! **Why this is not in `tests/common/`.** Only top-level `tests/*.rs` files are
//! compiled as crates; a subdirectory is inert until some crate declares
//! `mod …;`. So this module costs only the crates that opt in, whereas
//! `tests/common/` is already declared by ~30 e2e crates and anything added
//! there is compiled by all of them. `common` is for genuinely universal
//! helpers; a concern-specific bootstrap belongs in its own sibling
//! (`mem.pattern.tests.shared-helper-placement`).
//!
//! **Precondition.** An including crate must also declare `mod common;` — this
//! module resolves `crate::common::{doctrine_bin, SLICE_DIR}`. A crate that
//! forgets fails to compile, so the coupling cannot rot silently.
//!
//! **Not named `design_run`.** Every `tests/e2e_design_*.rs` already binds that
//! name to the `#[path]`-included leaf `src/design_run/mod.rs`.
//!
//! Model-reading accessors deliberately stay OUT of here. A crate that needs
//! them adds a second inherent `impl` block on [`DesignRun`] locally — the type
//! is defined in a module of that same crate, so that is legal and keeps the
//! `design_run` leaf out of crates that only drive the CLI.

#![allow(
    dead_code,
    reason = "shared bootstrap: not every includer uses every item"
)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// The slice every fixture designs.
pub(crate) const SLICE: &str = "SL-233";
/// Its zero-padded directory name.
pub(crate) const SLICE_NUMBER: &str = "233";

/// A started design run in a throwaway tree.
pub(crate) struct DesignRun {
    _tmp: tempfile::TempDir,
    pub(crate) root: PathBuf,
    /// Learned from `design start`'s own output, so no test re-types the state
    /// path (STD-001: the path has exactly one source, `design_snapshot_path`).
    pub(crate) snapshot: PathBuf,
    pub(crate) uid: String,
}

impl DesignRun {
    /// A run with no authored document — the cold start.
    pub(crate) fn start() -> DesignRun {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(root.join(crate::common::SLICE_DIR).join(SLICE_NUMBER)).unwrap();
        let out = run(&root, &["design", "start", SLICE, "-p", "."]);
        let uid = out
            .split_whitespace()
            .nth(1)
            .expect("`design start` names the run uid")
            .to_owned();
        let snapshot = out
            .lines()
            .find_map(|line| line.strip_prefix("snapshot "))
            .map(|path| root.join(path))
            .expect("`design start` names the snapshot path");
        DesignRun {
            _tmp: tmp,
            root,
            snapshot,
            uid,
        }
    }

    pub(crate) fn show(&self, extra: &[&str]) -> String {
        let mut args = vec!["design", "show", SLICE, "-p", "."];
        args.extend_from_slice(extra);
        run(&self.root, &args)
    }

    pub(crate) fn resume(&self, extra: &[&str]) -> String {
        let mut args = vec!["design", "resume", SLICE, "-p", "."];
        args.extend_from_slice(extra);
        run(&self.root, &args)
    }
}

/// Seed the authored slice record a `governance-confirmed` act needs to be
/// admissible (SL-244 `EX-10`).
///
/// `ObservedFact::GovernanceEdges` is projected from the slice's own outbound
/// relations, and a slice record the shell **cannot read** is an unobservable
/// fact, which `sec-3` fixes as CHANGED — so the act is refused rather than
/// recorded. Every design e2e fixture creates the slice *directory* and nothing
/// in it, which was enough until the projection went live: a ladder that records
/// the act needs a record to project from.
///
/// Both tiers, because `slice::relation_edges` reads through the show-path
/// reader and that reader wants the `.md` as well as the `.toml` — the
/// projection itself only reads the `.toml`, so the second write is a
/// precondition of the *reader*, not of the fact.
///
/// **No `[[relation]]` rows.** An edgeless slice is an *observable* fact (the
/// empty projection), which is all a ladder needs; a fixture whose subject is
/// the edge set moving writes its own rows.
pub(crate) fn seed_slice_record(root: &Path, slice_number: &str) {
    let dir = root.join(crate::common::SLICE_DIR).join(slice_number);
    std::fs::create_dir_all(&dir).unwrap();
    let id: u32 = slice_number.parse().expect("the slice number is numeric");
    std::fs::write(
        dir.join(format!("slice-{slice_number}.toml")),
        format!(
            "id      = {id}\n\
             slug    = \"fixture\"\n\
             title   = \"Fixture\"\n\
             status  = \"started\"\n\
             created = \"2026-01-01\"\n\
             updated = \"2026-01-01\"\n"
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join(format!("slice-{slice_number}.md")),
        "# Fixture\n\nA throwaway slice record, seeded so the governance edge set is observable.\n",
    )
    .unwrap();
}

/// Spawn the built binary rooted at `root`, asserting success; return stdout.
pub(crate) fn run(root: &Path, args: &[&str]) -> String {
    let out = spawn(root, args);
    assert!(
        out.status.success(),
        "doctrine {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// Spawn the built binary rooted at `root`, asserting failure; return stderr.
pub(crate) fn fail(root: &Path, args: &[&str]) -> String {
    let out = spawn(root, args);
    assert!(
        !out.status.success(),
        "doctrine {args:?} unexpectedly succeeded: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn spawn(root: &Path, args: &[&str]) -> std::process::Output {
    let bin = crate::common::doctrine_bin();
    Command::new(&bin)
        .args(args)
        .current_dir(root)
        .env("PATH", path_leading_with(bin.parent()))
        .env_remove("DOCTRINE_WORKER")
        .output()
        .expect("spawn doctrine")
}

/// `PATH` with the binary-under-test's directory first.
///
/// A runbook step's `verify` argv names `doctrine` and is resolved by the OS
/// like any other executable — the design's "arbitrary executable" contract
/// admits no special case for its own name (SL-233 PHASE-16 §4). So a fixture
/// that did not do this would spawn whatever `doctrine` the developer happens to
/// have installed, and check that instead of the code under test.
fn path_leading_with(dir: Option<&Path>) -> std::ffi::OsString {
    let existing = std::env::var_os("PATH").unwrap_or_default();
    let entries = dir
        .map(Path::to_path_buf)
        .into_iter()
        .chain(std::env::split_paths(&existing));
    std::env::join_paths(entries).expect("join PATH entries")
}
