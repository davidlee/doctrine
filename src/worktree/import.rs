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
pub(crate) const DOCTRINE_PREFIX: &str = ".doctrine/";

pub(crate) const CLAUDE_PREFIX: &str = ".claude/";

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

/// The verdict token when the POST-import (post-apply) tree fails `doctrine check
/// prove` — unformatted or lint-red (STD-001). Distinct from the pre-apply
/// [`Refusal`] belt: this gate runs AFTER `git apply` on the claude arm.
const POST_IMPORT_UNCLEAN: &str = "post-import-unclean";

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
    /// The delta touches a path NO design-target selector of the `--slice` scope
    /// declares (SL-180 PHASE-02). Only reachable when `selectors` is non-empty.
    UndeclaredScope,
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
            Refusal::UndeclaredScope => "undeclared-scope",
        }
    }

    /// The human-facing `Refused.detail` for this refusal (SL-224 PHASE-01). Only
    /// [`Refusal::UndeclaredScope`] carries a detail: it recomputes the undeclared set
    /// from the SAME pure predicate the belt used ([`crate::conformance::undeclared_paths`])
    /// and renders the shared runnable remediation ([`crate::conformance::undeclared_detail`])
    /// naming the slice id. Every other refusal returns an EMPTY detail — the bare token
    /// already fully describes it. Called on the refusal VALUE at the `dispatch_import`
    /// funnel arm so `Refusal` need not be named in prod (it is a `#[cfg(test)]` re-export).
    /// `Refusal` stays a fieldless `Copy` enum — the detail is computed here, never stored.
    pub(crate) fn scope_detail(
        self,
        slice: u32,
        selectors: &[String],
        delta_paths: &[String],
    ) -> String {
        match self {
            Refusal::UndeclaredScope => {
                let paths: Vec<&str> = delta_paths.iter().map(String::as_str).collect();
                let undeclared = crate::conformance::undeclared_paths(selectors, &paths);
                crate::conformance::undeclared_detail(slice, &undeclared)
            }
            _ => String::new(),
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
/// both tiers with no special-casing. The scope leg runs LAST, AFTER the prefix
/// legs: a `.doctrine/`/`.claude/` path always yields its tier refusal, never
/// `UndeclaredScope` (SL-180 PHASE-02). `selectors` are the `--slice` scope's
/// design-target selectors; an EMPTY `selectors` makes the scope leg a no-op —
/// byte-for-byte the pre-PHASE-02 belt (VA-3).
pub(crate) fn classify_import(
    head_at_base: bool,
    tree_clean: bool,
    single_commit: bool,
    delta_paths: &[String],
    selectors: &[String],
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
    // Scope leg (LAST): reject iff a `--slice` scope was supplied AND some delta
    // path is declared by no design-target selector. Reuses the pure conformance
    // predicate (single match implementation).
    if !selectors.is_empty() {
        let paths: Vec<&str> = delta_paths.iter().map(String::as_str).collect();
        if !crate::conformance::undeclared_paths(selectors, &paths).is_empty() {
            return Err(Refusal::UndeclaredScope);
        }
    }
    Ok(Apply::Ok)
}

/// On an `UndeclaredScope` refusal, print the id-bearing remediation to stdout
/// before the caller bails (EX-3). Re-derives the undeclared set from the SAME
/// pure predicate the belt used (single source), and hands it back so the caller
/// can name the paths in its error too. Renders via the SAME
/// [`crate::conformance::undeclared_detail`] formatter the MCP arm emits (SL-224
/// PHASE-01 EX-2) — one runnable `doctrine slice selector add SL-NNN <path>
/// --intent design-target` per path, id-bearing so it parses.
fn report_undeclared_scope(
    slice: Option<u32>,
    selectors: &[String],
    delta_paths: &[String],
) -> anyhow::Result<Vec<String>> {
    let paths: Vec<&str> = delta_paths.iter().map(String::as_str).collect();
    let undeclared = crate::conformance::undeclared_paths(selectors, &paths);
    write!(
        io::stdout(),
        "{}",
        undeclared_scope_report(slice, &undeclared)
    )?;
    Ok(undeclared)
}

/// The PURE rendering of the undeclared-scope report block (impurity — the stdout
/// write — stays in [`report_undeclared_scope`]). On `Some(id)` it IS the MCP
/// arm's [`crate::conformance::undeclared_detail`] — ONE id-bearing formatter,
/// so each line is a runnable `doctrine slice selector add SL-NNN <path> --intent
/// design-target` (SL-224 PHASE-01 EX-2). The `None` arm is type-reachable but
/// practically unreachable (the `UndeclaredScope` refusal fires only with
/// non-empty selectors, which in practice means `--slice` was given); it falls
/// back to an honest id-less path list — invents no fake id, never `unwrap`s.
fn undeclared_scope_report(slice: Option<u32>, undeclared: &[String]) -> String {
    if let Some(id) = slice {
        crate::conformance::undeclared_detail(id, undeclared)
    } else {
        let mut lines = vec![format!(
            "{} path(s) declared by no design-target selector:",
            undeclared.len()
        )];
        lines.extend(undeclared.iter().map(|path| format!("  {path}")));
        let mut out = lines.join("\n");
        out.push('\n');
        out
    }
}

/// Run the pure classifier and, on `UndeclaredScope`, print the diagnostic +
/// bail with a message naming the offending paths; on any other refusal, bail
/// with the bare token (the pre-PHASE-02 shape). Shared by both import arms so
/// the scope reporting has ONE implementation.
fn classify_or_report(
    slice: Option<u32>,
    head_at_base: bool,
    tree_clean: bool,
    single_commit: bool,
    delta_paths: &[String],
    selectors: &[String],
) -> anyhow::Result<()> {
    match classify_import(
        head_at_base,
        tree_clean,
        single_commit,
        delta_paths,
        selectors,
    ) {
        Err(Refusal::UndeclaredScope) => {
            let undeclared = report_undeclared_scope(slice, selectors, delta_paths)?;
            bail!(
                "import-refused: {} ({})",
                Refusal::UndeclaredScope.token(),
                undeclared.join(", ")
            );
        }
        Err(refusal) => bail!("import-refused: {}", refusal.token()),
        Ok(Apply::Ok) => Ok(()),
    }
}

/// The POST-apply prove-gate verdict (PURE). SEPARATE from the pre-apply
/// [`classify_import`]/[`Refusal`] belt: that belt runs BEFORE `git apply`; this
/// runs AFTER, on the post-import tree, over a single boolean fact (`prove` exited
/// clean?). Pure so VT-1 pins the halt semantics without spawning a child. A red
/// (unformatted OR lint-fail) delta HALTS — the orchestrator NEVER auto-fixes
/// (auto-fix hides the compliance signal + can't repair clippy/layering reds, and
/// violates ADR-012 sole-writer: land-or-reject, never rewrite).
fn import_prove_verdict(prove_clean: bool) -> Result<(), &'static str> {
    if prove_clean {
        Ok(())
    } else {
        Err(POST_IMPORT_UNCLEAN)
    }
}

/// Resolve the `prove` cadence argv IN-PROCESS (never spawn the `doctrine` binary —
/// PATH-independent by design) and run it on the coord `root`, inheriting stdio.
/// Returns whether it exited clean. Reuses the exact `load_config` + `resolve_check`
/// pair the `check` verb uses (both `pub(crate)`, layering-legal — no cycle).
fn run_prove_on(root: &Path) -> anyhow::Result<bool> {
    let cfg = crate::coverage_store::load_config(root)?;
    let argv = match crate::verify::resolve_check(&cfg, crate::verify::CheckKind::Prove) {
        crate::verify::CheckPlan::Run(argv) => argv,
        crate::verify::CheckPlan::Empty(_) => {
            bail!("import: [verification].prove is empty — cannot run the post-import prove gate")
        }
        // Prove never resolves to Noop (only `quick` is Noop-when-unset); guard the
        // invariant defensively rather than panic.
        crate::verify::CheckPlan::Noop(_) => {
            bail!(
                "import: [verification].prove resolved to a no-op — the post-import prove gate is inert"
            )
        }
    };
    // `Run` is non-empty by construction (INV-2); guard defensively rather than panic.
    let Some((program, rest)) = argv.split_first() else {
        bail!("import: resolved prove argv is empty — cannot spawn the post-import prove gate")
    };
    let status = std::process::Command::new(program)
        .args(rest)
        .current_dir(root)
        .status()
        .with_context(|| format!("spawning the post-import prove gate: {}", argv.join(" ")))?;
    Ok(status.success())
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
    slice: Option<u32>,
    selectors: &[String],
) -> anyhow::Result<()> {
    // Exactly one source. clap `conflicts_with` already rejects both-given at parse;
    // this rejects neither-given and dispatches to the arm's body.
    match (fork, from_worktree) {
        (Some(fork), None) => run_import_fork(path, base, fork, slice, selectors),
        (None, Some(dir)) => run_import_from_worktree(path, base, dir, slice, selectors),
        (Some(_), Some(_)) => bail!("import: --fork and --from-worktree are mutually exclusive"),
        (None, None) => bail!("import: exactly one of --fork / --from-worktree is required"),
    }
}

/// The pi/subprocess arm: import a worker's single committed fork `S` (`S^ == B`).
/// BEHAVIOUR-FROZEN (EX-4) — the body below is the pre-PHASE-05 `run_import`
/// verbatim; the `--patch` arm is strictly additive and shares only the pure
/// [`classify_import`] core.
fn run_import_fork(
    path: Option<PathBuf>,
    base: &str,
    fork: &str,
    slice: Option<u32>,
    selectors: &[String],
) -> anyhow::Result<()> {
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

    // --- pure classify (+ scope reporting on UndeclaredScope) ---
    classify_or_report(
        slice,
        head_at_base,
        tree_clean,
        single_commit,
        &delta_paths,
        selectors,
    )?;

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
fn run_import_from_worktree(
    path: Option<PathBuf>,
    base: &str,
    dir: &Path,
    slice: Option<u32>,
    selectors: &[String],
) -> anyhow::Result<()> {
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

    // --- pure classify (belt lives in the pure core; scope leg reported here) ---
    classify_or_report(
        slice,
        head_at_base,
        tree_clean,
        single_commit,
        &delta_paths,
        selectors,
    )?;

    // --- act: apply the gathered patch into the index, NON-committing (ADR-006 D7).
    // Same `git apply --3way --index` as the fork arm; the orchestrator commits
    // separately. The raw bytes carry the trailing newline `git apply` requires. ---
    git::git_apply_index(&root, &patch_bytes)
        .with_context(|| format!("git apply --3way --index from {}", dir.display()))?;

    // --- reject-and-halt prove gate (SL-191 PHASE-05): run the NON-mutating
    // `prove` cadence on the POST-import tree. A red (unformatted OR lint-fail)
    // delta HALTS — the delta is staged but NOT committed; the orchestrator fixes
    // the delta / re-dispatches, NEVER auto-fixes (ADR-012 sole-writer). ---
    let prove_clean = run_prove_on(&root)?;
    import_prove_verdict(prove_clean).map_err(|tok| {
        anyhow::anyhow!(
            "import halted: {tok} — the post-import tree fails `doctrine check prove` \
             (unformatted or lint-red); delta staged but NOT committed. Fix the delta / \
             re-dispatch (never auto-fix; ADR-012 sole-writer)."
        )
    })?;

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
pub(crate) fn gather_worktree_delta_paths(wt: &Path) -> anyhow::Result<Vec<String>> {
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

    // --- VT-4: classify_import with selectors (the pure scope leg) ---

    #[test]
    fn classify_import_undeclared_delta_path_is_undeclared_scope() {
        let selectors = vec!["src/**".to_string()];
        let delta = vec!["docs/readme.md".to_string()];
        assert_eq!(
            classify_import(true, true, true, &delta, &selectors),
            Err(Refusal::UndeclaredScope)
        );
    }

    #[test]
    fn classify_import_doctrine_path_is_doctrine_touch_even_when_undeclared() {
        // Prefix legs precede the scope leg: a `.doctrine/` path is DoctrineTouch,
        // never UndeclaredScope, though no selector declares it either.
        let selectors = vec!["src/**".to_string()];
        let delta = vec![".doctrine/state/x".to_string()];
        assert_eq!(
            classify_import(true, true, true, &delta, &selectors),
            Err(Refusal::DoctrineTouch)
        );
    }

    #[test]
    fn classify_import_empty_selectors_is_ok_noop() {
        // Empty selectors ⇒ the scope leg never fires (byte-for-byte old belt).
        let delta = vec!["anything/at/all".to_string()];
        assert_eq!(
            classify_import(true, true, true, &delta, &[]),
            Ok(Apply::Ok)
        );
    }

    #[test]
    fn classify_import_fully_declared_delta_is_ok() {
        let selectors = vec!["src/**".to_string()];
        let delta = vec!["src/a.rs".to_string(), "src/b.rs".to_string()];
        assert_eq!(
            classify_import(true, true, true, &delta, &selectors),
            Ok(Apply::Ok)
        );
    }

    // --- VT-1 (SL-191 PHASE-05): the post-apply prove-gate verdict is pure ---

    #[test]
    fn import_prove_verdict_halts_on_unclean_with_the_specific_token() {
        // A red (unformatted / lint-fail) post-import tree HALTS with the exact
        // token — assert the token, not merely `is_err()`.
        assert_eq!(
            import_prove_verdict(false),
            Err("post-import-unclean"),
            "an unclean post-import tree halts with POST_IMPORT_UNCLEAN"
        );
        // A clean tree lands.
        assert_eq!(import_prove_verdict(true), Ok(()));
    }

    // --- VT-5: run_import_from_worktree fed `--slice`-resolved selectors ---

    /// Stand up a coordination primary at HEAD == base with a linked worker
    /// worktree that mutated the tracked `seed`; returns `(tmp, primary, wt)`.
    fn worker_tree_touching_seed() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let primary = init_repo(&tmp.path().join("primary"));
        // The claude arm's post-import prove gate (SL-191 PHASE-05) spawns the
        // resolved `prove` argv on the coord root. This temp coord has no justfile,
        // so pin an always-clean prove override (`true`) in `.doctrine/doctrine.toml`
        // — untracked, so the `tree_clean` precond (tracked-only) is unaffected.
        fs::create_dir_all(primary.join(".doctrine")).unwrap();
        fs::write(
            primary.join(".doctrine/doctrine.toml"),
            "[verification]\nprove = [\"true\"]\n",
        )
        .unwrap();
        let wt = tmp.path().join("wt");
        git(
            &primary,
            &["worktree", "add", "-q", wt.to_str().unwrap(), "HEAD"],
        );
        let wt = fs::canonicalize(&wt).unwrap();
        fs::write(wt.join("seed"), "mutated\n").unwrap();
        (tmp, primary, wt)
    }

    #[test]
    fn run_import_from_worktree_refuses_an_undeclared_worker_path() {
        let (_tmp, primary, wt) = worker_tree_touching_seed();
        // No design-target selector declares `seed`.
        let selectors = vec!["docs/**".to_string()];
        let err = run_import_from_worktree(Some(primary), "HEAD", &wt, Some(224), &selectors)
            .expect_err("an undeclared worker path must be refused");
        let msg = err.to_string();
        assert!(
            msg.contains("import-refused: undeclared-scope"),
            "carries the token: {msg}"
        );
        assert!(msg.contains("seed"), "names the offending path: {msg}");
    }

    // The id-bearing remediation the CLI arm emits goes to STDOUT (via
    // `undeclared_detail`), not the bail message above; assert the CLI arm's PURE
    // renderer routes through the SAME `undeclared_detail` the MCP arm uses so the
    // hint carries the `SL-NNN` id and is runnable (SL-224 PHASE-01 EX-2).
    #[test]
    fn cli_undeclared_scope_report_is_the_id_bearing_shared_formatter() {
        let undeclared = vec!["src/leaked.rs".to_string()];
        let rendered = undeclared_scope_report(Some(224), &undeclared);
        assert_eq!(
            rendered,
            crate::conformance::undeclared_detail(224, &undeclared),
            "the CLI arm renders via the shared id-bearing formatter"
        );
        assert!(
            rendered.contains(
                "doctrine slice selector add SL-224 src/leaked.rs --intent design-target"
            ),
            "the emitted remediation is id-bearing and runnable: {rendered}"
        );
    }

    // The type-reachable `None` arm (non-empty selectors, no `--slice`) is honest:
    // it names the paths without inventing a fake `SL-NNN` id.
    #[test]
    fn cli_undeclared_scope_report_none_arm_omits_the_id() {
        let undeclared = vec!["src/leaked.rs".to_string()];
        let rendered = undeclared_scope_report(None, &undeclared);
        assert!(
            rendered.contains("src/leaked.rs"),
            "names the path: {rendered}"
        );
        assert!(
            !rendered.contains("SL-"),
            "no fabricated id on the None arm: {rendered}"
        );
    }

    #[test]
    fn run_import_from_worktree_stages_a_declared_worker_delta() {
        let (_tmp, primary, wt) = worker_tree_touching_seed();
        // The literal selector declares `seed` ⇒ conformant ⇒ the delta stages.
        let selectors = vec!["seed".to_string()];
        run_import_from_worktree(Some(primary.clone()), "HEAD", &wt, Some(224), &selectors)
            .expect("a fully-declared worker delta must import");
        // The delta landed in the coord index, NON-committing.
        let staged = std::process::Command::new("git")
            .arg("-C")
            .arg(&primary)
            .args(["diff", "--cached", "--name-only"])
            .output()
            .unwrap();
        let names = String::from_utf8_lossy(&staged.stdout);
        assert!(
            names.contains("seed"),
            "seed staged into the index: {names}"
        );
    }
}
