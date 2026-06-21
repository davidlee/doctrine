#![expect(unused, reason = "extraction; PHASE-03 prunes")]
// SPDX-License-Identifier: GPL-3.0-only
//! provision machine — extracted from worktree/mod.rs (SL-116 PHASE-02).

use super::allowlist::{
    Allowlist, allowlist_violations, is_withheld, parse_allowlist, select_copies,
};
use super::marker::{DISPATCH_WORKER_AGENT_TYPE, marker_present, write_marker};
use super::shared::{
    gather_fork_worktree, gather_tree_clean, is_linked_worktree, matches, resolve_commit,
    resolve_common_dir, target_dir_for_branch,
};
use crate::fsutil::{self, CopyOutcome};
use crate::git;
use crate::root;
use anyhow::{Context, bail};
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

const ALLOWLIST_FILE: &str = ".worktreeinclude";

/// Read `<root>/.worktreeinclude`; **absent ⇒ empty allowlist ⇒ copy nothing** (F2).
fn read_allowlist(root: &Path) -> anyhow::Result<Allowlist> {
    let path = root.join(ALLOWLIST_FILE);
    match fs::read_to_string(&path) {
        Ok(text) => parse_allowlist(&text).map_err(|e| anyhow::anyhow!("{}: {e}", path.display())),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Allowlist {
            patterns: Vec::new(),
        }),
        Err(e) => Err(e).with_context(|| format!("read {}", path.display())),
    }
}

/// Resolve a `git rev-parse --git-common-dir` answer (relative to `root`, or
/// absolute for a linked worktree) to a canonical path for comparison.
/// Verify `fork` is a real sibling worktree of `source`: it shares the source's
/// `git-common-dir` and is not the source itself (design §3 copy safety, B5).
fn verify_sibling_worktree(source: &Path, fork: &Path) -> anyhow::Result<()> {
    if source == fork {
        bail!("fork path is the source tree itself; refusing to provision");
    }
    let source_common = resolve_common_dir(
        source,
        &git::git_text(source, &["rev-parse", "--git-common-dir"])?,
    )?;
    let fork_common = resolve_common_dir(
        fork,
        &git::git_text(fork, &["rev-parse", "--git-common-dir"])?,
    )?;
    if source_common != fork_common {
        bail!(
            "fork {} is not a worktree of the source repo (git-common-dir differs)",
            fork.display()
        );
    }
    Ok(())
}

/// Enumerate the copy candidate set: gitignored, untracked files, NUL-delimited
/// so newline/quoted paths survive (design §3 m9).
fn enumerate_candidates(root: &Path) -> anyhow::Result<Vec<String>> {
    let raw = git::git_bytes(
        root,
        &[
            "ls-files",
            "-z",
            "--others",
            "--ignored",
            "--exclude-standard",
        ],
    )?;
    let mut out = Vec::new();
    for chunk in raw.split(|b| *b == 0) {
        if chunk.is_empty() {
            continue;
        }
        let path = std::str::from_utf8(chunk)
            .map_err(|e| anyhow::anyhow!("non-utf8 path from git ls-files: {e}"))?;
        out.push(path.to_string());
    }
    Ok(out)
}

/// `doctrine worktree provision <fork>` — the sole copier (design §3).
///
/// Runs from the SOURCE root and writes `<fork>`: read `.worktreeinclude` (absent
/// ⇒ empty) → `allowlist_violations` fail-closed → verify `<fork>` is a sibling
/// worktree → enumerate gitignored candidates → `select_copies` → safe copy,
/// skip+warn withheld → report copied/withheld (exit 0).
pub(crate) fn run_provision(path: Option<PathBuf>, fork: &Path) -> anyhow::Result<()> {
    let source = root::find(path, &root::default_markers())?;
    let source = fs::canonicalize(&source)
        .with_context(|| format!("canonicalize source root {}", source.display()))?;

    let allow = read_allowlist(&source)?;

    // Fail closed: a tier-naming pattern aborts before any copy (VT-8).
    let violations = allowlist_violations(&allow);
    if !violations.is_empty() {
        for v in &violations {
            writeln!(
                io::stderr(),
                "refusing: pattern `{}` names the withheld {} tier",
                v.pattern,
                v.tier
            )?;
        }
        bail!(
            "{} .worktreeinclude pattern(s) name a withheld tier; refusing to provision",
            violations.len()
        );
    }

    let fork =
        fs::canonicalize(fork).with_context(|| format!("canonicalize fork {}", fork.display()))?;
    verify_sibling_worktree(&source, &fork)?;

    let candidates = enumerate_candidates(&source)?;
    let selection = select_copies(&allow, &candidates);

    let withheld_target = |rel: &Path| rel.to_str().is_some_and(|s| is_withheld(s).is_some());

    let mut copied = 0usize;
    let mut skipped = 0usize;
    for rel in &selection.copy {
        match fsutil::copy_selected(&source, &fork, Path::new(rel), &withheld_target)? {
            CopyOutcome::Copied => copied += 1,
            CopyOutcome::Skipped(reason) => {
                skipped += 1;
                writeln!(io::stderr(), "skipped {rel}: {reason}")?;
            }
        }
    }
    for held in &selection.withheld {
        writeln!(io::stderr(), "withheld {} ({} tier)", held.path, held.tier)?;
    }

    // Human status to stderr (ISS-044): every consumer reuses this sole copier —
    // fork/coordinate emit a KEY=value env contract on stdout, so a "provisioned …"
    // line there pollutes it (`env $(fork …)` word-splits → rc 127). Siblings
    // (skipped/withheld) already go to stderr; this joins them.
    writeln!(
        io::stderr(),
        "provisioned {}: {copied} copied, {} withheld, {skipped} skipped",
        fork.display(),
        selection.withheld.len()
    )?;
    Ok(())
}

/// `doctrine worktree check-allowlist` — the static smell test. Nonzero exit on a
/// tier-naming pattern OR an unsupported-syntax (`!`/anchoring) pattern.
pub(crate) fn run_check_allowlist(path: Option<PathBuf>) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    let file = root.join(ALLOWLIST_FILE);
    let text = match fs::read_to_string(&file) {
        Ok(t) => t,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            writeln!(io::stdout(), "no {ALLOWLIST_FILE} — nothing to check")?;
            return Ok(());
        }
        Err(e) => return Err(e).with_context(|| format!("read {}", file.display())),
    };

    // Parse errors (`!`/anchoring/bad-glob) fail closed via `?`.
    let allow = parse_allowlist(&text).map_err(|e| anyhow::anyhow!("{}: {e}", file.display()))?;

    let violations = allowlist_violations(&allow);
    if violations.is_empty() {
        writeln!(
            io::stdout(),
            "ok — no allowlist pattern names a withheld tier"
        )?;
        return Ok(());
    }
    for v in &violations {
        writeln!(
            io::stderr(),
            "violation: pattern `{}` names the withheld {} tier",
            v.pattern,
            v.tier
        )?;
    }
    bail!(
        "{} allowlist pattern(s) name a withheld tier",
        violations.len()
    )
}
