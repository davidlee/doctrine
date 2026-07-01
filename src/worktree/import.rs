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

// ---- git belt-hardening flags (STD-001; shared by the live-worktree gather) ------
// The claude arm imports the worker's LIVE working-tree diff, so the SAME hardening
// the fork arm's belt uses must ride at gather time — the belt cannot un-mangle a
// path the diff already C-quoted or a rename it already collapsed:
//   * quotePath off — a non-ASCII `.doctrine/` path emits verbatim, not C-quoted past
//     the `starts_with(".doctrine/")` belt;
//   * `--no-renames` — a governance-file rename shows BOTH legs (delete + add), so the
//     `.doctrine/` SOURCE cannot hide behind a same-content destination.
// (The fork arm keeps its own inline literals — BEHAVIOUR-FROZEN, EX-4 — so these are
// used only by the additive `--from-worktree` path, accepting that narrow duplication.)
const QUOTE_PATH_OFF: [&str; 2] = ["-c", "core.quotePath=false"];
const NO_RENAMES: &str = "--no-renames";
const DEV_NULL: &str = "/dev/null";

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
    from_worktree: Option<&Path>,
) -> anyhow::Result<()> {
    // Exactly one source. clap `conflicts_with` already rejects both-given at parse;
    // this rejects neither-given and dispatches to the arm's body.
    match (fork, from_worktree) {
        (Some(fork), None) => run_import_fork(path, base, fork),
        (None, Some(dir)) => run_import_from_worktree(path, base, dir),
        (Some(_), Some(_)) => bail!("import: --fork and --from-worktree are mutually exclusive"),
        (None, None) => bail!("import: exactly one of --fork / --from-worktree is required"),
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

/// The claude arm (SL-182 PHASE-05, symmetric live-import): import the worker's
/// **live** working-tree delta directly from the persisted worktree `dir`. ro-`.git`
/// blocks the worker's self-commit, so the fork tip stays at `B` and the committed-fork
/// path is a dead end here (`<fork>^ == B^ != B` ⇒ `MultiCommit`). With `create-fork`
/// as the `WorktreeCreate` hook and NO `WorktreeRemove` hook, the worker tree PERSISTS
/// on disk post-return with its diff intact (`mem_019f1a5c…`, corrected against
/// `docs/claude/hooks.md:2442`), so the orchestrator gathers the live delta itself —
/// no `SubagentStop` capture, no file hop. Reuses [`classify_import`] UNCHANGED: the
/// `single_commit` precond is **vacuously true** (a working-tree diff carries zero
/// commits, trivially ≤ 1), so the LOAD-BEARING checks on this arm are the belt
/// (`.doctrine/`/`.claude/` reject) + `head_at_base` + `tree_clean` — all still run
/// through the pure core. Gather → pure-classify → act, same shape as the fork arm.
///
/// The caller (`/dispatch-agent` funnel) reaps the worktree with `git worktree remove
/// --force` ONLY after this returns 0 (F-3): a nonzero exit halts the funnel and LEAVES
/// the tree, so a failed import never `--force`-destroys the sole copy of the delta.
fn run_import_from_worktree(path: Option<PathBuf>, base: &str, dir: &Path) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;

    // Gather the worker's LIVE working-tree delta (tracked + untracked) as one
    // applyable patch. An empty patch means the worker produced no delta — report-
    // and-halt, never launder an empty import green (design §5.4 / the funnel's halt).
    let patch_bytes = gather_worktree_patch(dir)?;
    if patch_bytes.is_empty() {
        bail!(
            "import: worker worktree {} carries no delta; halting",
            dir.display()
        );
    }

    // --- gather: preconds 1 / 1b — HEAD == B, tracked tree clean (== fork arm) ---
    let base_sha = resolve_commit(&root, base)?;
    let head_sha = resolve_commit(&root, "HEAD")?;
    let head_at_base = matches(&base_sha, &head_sha);
    let tree_clean = gather_tree_clean(&root)?;

    // --- precond 2 — single_commit VACUOUS: a working-tree diff carries no commits ---
    let single_commit = true;

    // --- belt input — the worker tree's touched paths (tracked diff + untracked adds),
    // read straight from the LIVE tree (`-C dir`) with the same hardening the fork arm's
    // belt uses (quotePath off + --no-renames) so a non-ASCII `.doctrine/` path emits
    // verbatim and no rename hides a governance source leg. ---
    let delta_paths = gather_worktree_delta_paths(dir)?;

    // --- pure classify (belt lives in the pure core, reused unchanged) ---
    match classify_import(head_at_base, tree_clean, single_commit, &delta_paths) {
        Err(refusal) => bail!("import-refused: {}", refusal.token()),
        Ok(Apply::Ok) => {}
    }

    // --- act: apply the gathered patch into the index, NON-committing (ADR-006 D7).
    // Same `git apply --3way --index` as the fork arm; the orchestrator commits
    // separately. The raw bytes carry the trailing newline `git apply` requires. ---
    git::git_apply_index(&root, &patch_bytes)
        .with_context(|| format!("git apply --3way --index from {}", dir.display()))?;

    writeln!(
        io::stdout(),
        "imported worktree {}: delta staged (uncommitted)",
        dir.display()
    )?;
    Ok(())
}

/// Gather a worker worktree's full working-tree delta as one applyable patch. The
/// **tracked** leg is `git -C <wt> diff HEAD` (staged + unstaged in one stream),
/// belt-hardened, RAW bytes (trailing newline preserved for `git apply`). The
/// **untracked** leg synthesizes an index-free `new file` hunk per untracked path via
/// `git diff --no-index /dev/null <f>` (the ro `.git` index is NEVER written — no
/// `git add`/`-N`) and concatenates it onto the same stream. Impure (git reads only,
/// all `-C <wt>`); nothing under `.git` is mutated. Relocated from the retired
/// `capture.rs` (SL-182 PHASE-05 symmetric-import amendment).
fn gather_worktree_patch(wt: &Path) -> anyhow::Result<Vec<u8>> {
    let mut patch = git::git_bytes(
        wt,
        &[
            QUOTE_PATH_OFF[0],
            QUOTE_PATH_OFF[1],
            "diff",
            NO_RENAMES,
            "HEAD",
        ],
    )
    .with_context(|| format!("git diff HEAD in {}", wt.display()))?;

    let untracked = git::git_text(wt, &["ls-files", "--others", "--exclude-standard"])
        .with_context(|| format!("list untracked in {}", wt.display()))?;
    for rel in untracked.lines().filter(|l| !l.is_empty()) {
        // `--no-index` exits 1 whenever the inputs differ (always, vs /dev/null) —
        // lenient runner keeps the stdout that carries the new-file hunk.
        let hunk = git::git_bytes_lenient(
            wt,
            &[
                QUOTE_PATH_OFF[0],
                QUOTE_PATH_OFF[1],
                "diff",
                NO_RENAMES,
                "--no-index",
                "--",
                DEV_NULL,
                rel,
            ],
        )
        .with_context(|| format!("synthesize untracked hunk for {rel} in {}", wt.display()))?;
        patch.extend_from_slice(&hunk);
    }
    Ok(patch)
}

/// The worker tree's belt input — the names of every touched path, TRACKED (`diff
/// --name-only HEAD`) plus UNTRACKED adds (`ls-files --others`), quotePath off so a
/// non-ASCII governance path is verbatim past the pure belt. Untracked paths must ride
/// the belt too: an untracked `.doctrine/foo` the worker dropped is still a governance
/// touch and must be rejected, exactly like a tracked one.
fn gather_worktree_delta_paths(wt: &Path) -> anyhow::Result<Vec<String>> {
    let tracked = git::git_text(
        wt,
        &[
            QUOTE_PATH_OFF[0],
            QUOTE_PATH_OFF[1],
            "diff",
            "--name-only",
            NO_RENAMES,
            "HEAD",
        ],
    )
    .with_context(|| format!("git diff --name-only HEAD in {}", wt.display()))?;
    let untracked = git::git_text(
        wt,
        &[
            QUOTE_PATH_OFF[0],
            QUOTE_PATH_OFF[1],
            "ls-files",
            "--others",
            "--exclude-standard",
        ],
    )
    .with_context(|| format!("list untracked in {}", wt.display()))?;
    Ok(tracked
        .lines()
        .chain(untracked.lines())
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worktree::test_helpers::{git, init_repo};
    use std::fs;

    /// VT-3 round-trip: the live-worktree gather carries BOTH a tracked change and an
    /// untracked add, and re-applies cleanly onto the base tree it was cut from —
    /// proving the index-free untracked synthesis is applyable, not just gatherable.
    /// Relocated from the retired `capture.rs` (symmetric live-import amendment).
    #[test]
    fn gather_from_worktree_captures_tracked_and_untracked_and_reapplies() {
        let tmp = tempfile::tempdir().unwrap();
        let primary = init_repo(&tmp.path().join("primary"));
        // A linked worktree off HEAD (the worker's live tree).
        let wt = tmp.path().join("wt");
        git(
            &primary,
            &["worktree", "add", "-q", wt.to_str().unwrap(), "HEAD"],
        );
        let wt = fs::canonicalize(&wt).unwrap();

        // Worker mutates a tracked file and drops an untracked one.
        fs::write(wt.join("seed"), "mutated\n").unwrap();
        fs::write(wt.join("newfile"), "brand new\n").unwrap();

        let patch = gather_worktree_patch(&wt).unwrap();
        assert!(!patch.is_empty(), "patch must be non-empty");
        let text = String::from_utf8_lossy(&patch);
        assert!(text.contains("seed"), "tracked change captured");
        assert!(text.contains("newfile"), "untracked add captured");

        // The belt-input names cover both the tracked change and the untracked add.
        let names = gather_worktree_delta_paths(&wt).unwrap();
        assert!(names.iter().any(|p| p == "seed"), "tracked path listed");
        assert!(
            names.iter().any(|p| p == "newfile"),
            "untracked path listed"
        );

        // Re-apply onto a fresh checkout of the SAME base ⇒ both deltas reconstruct.
        let target = tmp.path().join("apply");
        git(
            &primary,
            &["worktree", "add", "-q", target.to_str().unwrap(), "HEAD"],
        );
        let patch_file = tmp.path().join("captured.patch");
        fsutil::write_atomic(&patch_file, &patch).unwrap();
        git(&target, &["apply", patch_file.to_str().unwrap()]);

        assert_eq!(
            fs::read_to_string(target.join("seed")).unwrap(),
            "mutated\n"
        );
        assert_eq!(
            fs::read_to_string(target.join("newfile")).unwrap(),
            "brand new\n"
        );
    }
}
