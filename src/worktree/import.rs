#![expect(unused, reason = "extraction; PHASE-03 prunes")]
// SPDX-License-Identifier: GPL-3.0-only
//! import machine — extracted from worktree/mod.rs (SL-116 PHASE-02).

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

/// The two coordination/runtime tier prefixes the import belt rejects. The
/// `.claude/` tier is wholly gitignored, so its leg only ever catches a
/// *force-added* path — parity with `.doctrine/`, not a special case (PHASE-07).
const DOCTRINE_PREFIX: &str = ".doctrine/";

const CLAUDE_PREFIX: &str = ".claude/";

/// Verdict of the PURE import classifier: apply the delta, or fail closed with a
/// distinct named refusal token. The shell ([`run_import`]) gathers the FACTS and
/// acts on this verdict — never the other way round (ADR-001 leaf, gather →
/// pure-classify → act).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Apply {
    /// All preconds + the belt hold ⇒ the orchestrator may `git apply` the delta.
    Ok,
}

/// The exhaustive v1 import refusal set (stationary-head case only). Each fails
/// closed with a distinct token; never auto-merge / auto-resolve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Refusal {
    /// Coordination `HEAD != B` — the orchestrator's base moved (re-dispatch).
    HeadMoved,
    /// Tracked tree dirty (`git status --porcelain --untracked-files=no` nonempty).
    TreeUnclean,
    /// `<fork>` carries more than one non-merge commit (`S^ != B`).
    MultiCommit,
    /// The `B..<fork>` delta touches a `.doctrine/` (coordination/runtime) path.
    DoctrineTouch,
    /// The `B..<fork>` delta force-touches a `.claude/` path.
    ClaudeTouch,
}

impl Refusal {
    /// The distinct named token each refusal fails closed with (the property the
    /// VT-2 goldens assert, not a proxy).
    pub(crate) fn token(self) -> &'static str {
        match self {
            Refusal::HeadMoved => "head-moved",
            Refusal::TreeUnclean => "tree-unclean",
            Refusal::MultiCommit => "multi-commit",
            Refusal::DoctrineTouch => "doctrine-touch",
            Refusal::ClaudeTouch => "claude-touch",
        }
    }
}

/// PURE import classifier (no git / disk / env — ADR-001 leaf, CLAUDE.md
/// pure/imperative split). Takes the gathered FACTS and returns the verdict:
///
/// * `head_at_base` — coordination `HEAD == B` (ref-equality, resolved in the shell)
/// * `tree_clean`   — tracked tree clean (`--untracked-files=no` porcelain empty)
/// * `single_commit`— `<fork>^ == B` (exactly one non-merge commit S on the fork)
/// * `delta_paths`  — the `B..<fork>` name-only, TRACKED-files-only diff paths
///
/// Precond order matches the funnel: HEAD → tree → single-commit → belt. The belt
/// prefix-matching lives HERE (pure) — `.doctrine/` then `.claude/`, prefix-match
/// both tiers with no special-casing.
pub(crate) fn classify_import(
    head_at_base: bool,
    tree_clean: bool,
    single_commit: bool,
    delta_paths: &[String],
) -> Result<Apply, Refusal> {
    if !head_at_base {
        return Err(Refusal::HeadMoved);
    }
    if !tree_clean {
        return Err(Refusal::TreeUnclean);
    }
    if !single_commit {
        return Err(Refusal::MultiCommit);
    }
    for path in delta_paths {
        if path.starts_with(DOCTRINE_PREFIX) {
            return Err(Refusal::DoctrineTouch);
        }
        if path.starts_with(CLAUDE_PREFIX) {
            return Err(Refusal::ClaudeTouch);
        }
    }
    Ok(Apply::Ok)
}

/// `doctrine worktree import --base <B> --fork <branch>` — mechanizes the dispatch
/// funnel's deterministic stationary-head import as ONE fail-closed verb (design
/// §5, ADR-006 D7: import ≠ commit). Runs at the coordination root.
///
/// Gather → pure-classify → act, patterned after [`run_branch_point_check`]:
/// 1. gather the FACTS (HEAD==B via [`resolve_commit`]/[`matches`]; tracked-tree
///    cleanliness; `<fork>^ == B`; the `B..<fork>` name-only tracked diff),
/// 2. [`classify_import`] returns the verdict (the belt lives in the pure core),
/// 3. on `Ok`, `git apply --3way --index` the SAME name-only diff NON-committing —
///    the orchestrator commits separately. Under both preconds the patch applies
///    onto the exact tree it was cut from ⇒ cannot conflict (apply-conflict is NOT
///    a v1 refusal). NO runtime receipt is stamped — landed-ness is derived from
///    durable git later, never a pre-commit gitignored flag that would survive a
///    crash and lie "landed".
///
/// Gather the tracked-tree cleanliness fact for [`run_import`] / [`run_land`].
/// Delegates to the single leaf predicate [`git::tree_clean`] (SL-121 §2.3 lifted
/// it to git.rs so the integrate dirty pre-gate + the §2.5 race re-check share the
/// Orchestrator-classed; refused under worker-mode by `worker_guard` (the verb is
/// the orchestrator's, never a worker's).
pub(crate) fn run_import(path: Option<PathBuf>, base: &str, fork: &str) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;

    // --- gather: precond 1 — HEAD == B (ref-equality on resolved shas) ---
    let base_sha = resolve_commit(&root, base)?;
    let head_sha = resolve_commit(&root, "HEAD")?;
    let head_at_base = matches(&base_sha, &head_sha);

    // --- gather: precond 1b — tracked tree clean (untracked deliberately excluded) ---
    let tree_clean = gather_tree_clean(&root)?;

    // --- gather: precond 2 — S^ == B (exactly one non-merge commit on the fork) ---
    // `<fork>^` = S's first parent, peeled to a commit. A merge or multi-commit
    // history (or a fork that does not resolve) ⇒ parent != B ⇒ not single-commit,
    // never a panic — `git_opt` yields None on a non-resolving ref.
    let fork_parent = git::git_opt(
        &root,
        &["rev-parse", "--verify", &format!("{fork}^^{{commit}}")],
    )?;
    let single_commit = fork_parent
        .as_deref()
        .is_some_and(|p| matches(p, &base_sha));

    // --- gather: belt input — B..<fork> name-only, TRACKED-files-only diff ---
    // Two hardening flags, both gating the belt's malice-containment (SL-056 §7):
    //   * `-c core.quotePath=false` — git's default quotePath=true C-quotes any
    //     path with a non-ASCII byte (".doctrine/\303\251…"), so the pure
    //     prefix-match `starts_with(".doctrine/")` would MISS and the governance
    //     file would ride back. Pin it off so the real path is emitted verbatim.
    //   * `--no-renames` — default rename detection collapses a governance
    //     DELETION paired with a same-content add elsewhere into a single
    //     destination line, hiding the `.doctrine/` SOURCE from the belt. Off ⇒
    //     both legs (delete + add) appear as themselves.
    let diff = git::git_text(
        &root,
        &[
            "-c",
            "core.quotePath=false",
            "diff",
            "--name-only",
            "--no-renames",
            &format!("{base}..{fork}"),
        ],
    )?;
    let delta_paths: Vec<String> = diff.lines().map(str::to_owned).collect();

    // --- pure classify ---
    match classify_import(head_at_base, tree_clean, single_commit, &delta_paths) {
        Err(refusal) => bail!("import-refused: {}", refusal.token()),
        Ok(Apply::Ok) => {}
    }

    // --- act: apply the SAME diff into the index, NON-committing (ADR-006 D7) ---
    // `git apply --3way --index` writes the index from the coordination root; under
    // both preconds the patch applies onto the exact tree it was cut from.
    // `--no-renames` keeps the apply view consistent with the belt's: a rename
    // is two real legs (delete + add), which `git apply` handles directly (a
    // pure-rename header carries no hunk for apply to act on).
    // Capture the diff as RAW BYTES (not `git_text`, whose `.trim()` strips the
    // trailing newline `git apply` requires — ISS-032). The name-only belt above
    // can trim freely; this apply stream cannot.
    let patch = git::git_bytes(&root, &["diff", "--no-renames", &format!("{base}..{fork}")])?;
    git::git_apply_index(&root, &patch)
        .with_context(|| format!("git apply --3way --index {base}..{fork}"))?;

    writeln!(
        io::stdout(),
        "imported {base}..{fork}: delta staged (uncommitted)"
    )?;
    Ok(())
}
