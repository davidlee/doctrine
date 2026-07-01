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
pub(crate) fn run_import(
    path: Option<PathBuf>,
    base: &str,
    fork: Option<&str>,
    patch: Option<&Path>,
) -> anyhow::Result<()> {
    // Exactly one source. clap `conflicts_with` already rejects both-given at parse;
    // this rejects neither-given and dispatches to the arm's body.
    match (fork, patch) {
        (Some(fork), None) => run_import_fork(path, base, fork),
        (None, Some(patch)) => run_import_patch(path, base, patch),
        (Some(_), Some(_)) => bail!("import: --fork and --patch are mutually exclusive"),
        (None, None) => bail!("import: exactly one of --fork / --patch is required"),
    }
}

/// The pi/subprocess arm: import a worker's single committed fork `S` (`S^ == B`).
/// BEHAVIOUR-FROZEN (EX-4) — the body below is the pre-PHASE-05 `run_import`
/// verbatim; the `--patch` arm is strictly additive and shares only the pure
/// [`classify_import`] core.
fn run_import_fork(path: Option<PathBuf>, base: &str, fork: &str) -> anyhow::Result<()> {
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

/// The claude arm (SL-182 PHASE-05, OQ-1): import the worker's CAPTURED PATCH.
/// ro-`.git` blocks the worker's self-commit, so the fork tip stays at `B` and the
/// committed-fork path is a dead end here (`<fork>^ == B^ != B` ⇒ `MultiCommit`).
/// The delta rides the captured patch instead. Reuses [`classify_import`] UNCHANGED:
/// the `single_commit` precond is **vacuously true** (a patch carries zero commits,
/// trivially ≤ 1), so the LOAD-BEARING checks on this arm are the belt
/// (`.doctrine/`/`.claude/` reject) + `head_at_base` + `tree_clean` — all still run
/// through the pure core. Gather → pure-classify → act, same shape as the fork arm.
fn run_import_patch(path: Option<PathBuf>, base: &str, patch: &Path) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;

    // A missing/empty patch means the worker produced no capturable delta (the
    // SubagentStop capture failed or lost the tree). Report-and-halt — never launder
    // an empty import green (R-capture-lossy, design §5.4 / the funnel's halt).
    let patch_bytes =
        fs::read(patch).with_context(|| format!("read captured patch {}", patch.display()))?;
    if patch_bytes.is_empty() {
        bail!(
            "import: captured patch {} is empty — worker produced no delta; halting",
            patch.display()
        );
    }

    // --- gather: preconds 1 / 1b — HEAD == B, tracked tree clean (== fork arm) ---
    let base_sha = resolve_commit(&root, base)?;
    let head_sha = resolve_commit(&root, "HEAD")?;
    let head_at_base = matches(&base_sha, &head_sha);
    let tree_clean = gather_tree_clean(&root)?;

    // --- precond 2 — single_commit VACUOUS: a captured patch carries no commits ---
    let single_commit = true;

    // --- belt input — the patch's touched paths, via `git apply --numstat` (a DRY
    // inspection: it mutates nothing). Same hardening as the fork arm's belt so a
    // non-ASCII `.doctrine/` path emits verbatim (quotePath off) and no rename hides
    // a governance source leg (--no-renames). The capture side already hardened the
    // diff (SL-182 T2); this is belt-and-suspenders on the read-back. numstat line =
    // `<added>\t<removed>\t<path>` — the path is the segment after the last tab. ---
    // NB: `git apply` has no `--no-renames` (that is a `git diff` flag) — the capture
    // side already hardened the diff, so the patch carries no rename pairs; here we
    // only keep quotePath off so a non-ASCII path in the numstat output is verbatim.
    let numstat = git::git_text(
        &root,
        &[
            "-c",
            "core.quotePath=false",
            "apply",
            "--numstat",
            &patch.to_string_lossy(),
        ],
    )
    .with_context(|| format!("git apply --numstat {}", patch.display()))?;
    let delta_paths: Vec<String> = numstat
        .lines()
        .filter_map(|line| line.rsplit('\t').next())
        .map(str::to_owned)
        .collect();

    // --- pure classify (belt lives in the pure core, reused unchanged) ---
    match classify_import(head_at_base, tree_clean, single_commit, &delta_paths) {
        Err(refusal) => bail!("import-refused: {}", refusal.token()),
        Ok(Apply::Ok) => {}
    }

    // --- act: apply the captured patch into the index, NON-committing (ADR-006 D7).
    // Same `git apply --3way --index` as the fork arm; the orchestrator commits
    // separately. The raw bytes carry the trailing newline `git apply` requires. ---
    git::git_apply_index(&root, &patch_bytes)
        .with_context(|| format!("git apply --3way --index {}", patch.display()))?;

    writeln!(
        io::stdout(),
        "imported patch {}: delta staged (uncommitted)",
        patch.display()
    )?;
    Ok(())
}
