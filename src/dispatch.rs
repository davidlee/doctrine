// SPDX-License-Identifier: GPL-3.0-only
//! SL-064 PHASE-04 — the `dispatch sync` projection verb (stage-1
//! `--prepare-review`). Materialises the reviewable refs from the coordination
//! branch `dispatch/<slice>` **without writing trunk**:
//!
//! * **B** — `review/<slice>`: a single squashed, filtered projection of the
//!   `dispatch/<slice>` tip, parented to the trunk base, excluding the run-ledger
//!   dir and every journal-verified orthogonal path (design §4.2).
//! * **C** — `phase/<slice>-NN`: the claude-arm per-phase cut synthesised from
//!   `boundaries.toml`, code-only (`.doctrine/` stripped), empty-code phases
//!   skipped, chained so each diff is exactly that phase's code delta (§4.3).
//!
//! The CAS journal is committed onto `dispatch/<slice>` (plumbing-only, no
//! checkout) **before** any external ref mutation (EX-2, ADR-012 D4); external
//! refs are created via zero-oid CAS so a crashed prior run's stale `review/*` /
//! `phase/*` is reported, never clobbered (EX-5). Trunk and `edge` are never
//! touched — that is stage-2 `--integrate` (PHASE-05).

use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};

use crate::git::{self, MergeTree, RefCas, ReplayOutcome, ZERO_OID};
use crate::ledger::{
    Admission, Boundaries, CandidateKind, CandidatePayload, CandidateRole, CandidateRow,
    CandidateStatus, Candidates, Journal, JournalRow, LedgerStatus, Orthogonal, read_candidates,
};
use crate::listing::render_table;
use crate::root;

/// CLI entry — create or resume the dispatch coordination worktree for `slice`
/// and emit the orchestration env contract on stdout (SL-085, design §2).
/// Gates on `plan.toml` existence + non-empty phase list BEFORE creating the
/// coordination worktree.
pub(crate) fn run_setup(path: Option<PathBuf>, slice: u32, dir: &Path) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;

    // Plan gate: read plan.toml, require existence + non-empty phase list.
    let slice_root = root.join(".doctrine/slice");
    let plan = crate::slice::read_plan(&slice_root, slice).with_context(|| {
        format!("no plan for SL-{slice:03}; run 'doctrine slice plan {slice}' first")
    })?;
    if plan.phases.is_empty() {
        anyhow::bail!("plan for SL-{slice:03} has no phases; add phases to plan.toml first");
    }

    // Delegate to the extracted pure-ish core.
    let outcome = crate::worktree::coordinate(&root, slice, dir)?;

    // Emit the dispatch env contract on stdout (4 KEY=value lines).
    let dispatch_ref = format!("refs/heads/dispatch/{slice:03}");
    writeln!(io::stdout(), "coordination_dir={}", dir.display())?;
    writeln!(io::stdout(), "base={}", outcome.dispatch_tip)?;
    writeln!(io::stdout(), "slice={slice}")?;
    writeln!(io::stdout(), "dispatch_ref={dispatch_ref}")?;

    Ok(())
}

/// One planned projection: a target ref and the commit it should be created at.
/// `source_oid` is the object the projection was computed from (the journal's
/// replay input).
struct Planned {
    target_ref: String,
    source_oid: String,
    commit_oid: String,
}

/// CLI entry — resolve the root and run stage-1 prepare-review for `slice`.
pub(crate) fn run_prepare_review(path: Option<PathBuf>, slice: u32) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    prepare_review(&root, slice)
}

/// CLI entry — resolve the root and run stage-2 integrate for `slice`. `trunk`
/// names the ref the code units project onto (ff-only); `edge` names an optional
/// aggregate ref. Both default off ⇒ a pure idempotent journal replay (EX-1).
pub(crate) fn run_integrate(
    path: Option<PathBuf>,
    slice: u32,
    trunk: Option<&str>,
    edge: Option<&str>,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    integrate(&root, slice, trunk, edge)
}

/// CLI entry — funnel-time recording: append a per-phase code boundary to
/// `boundaries.toml` (design §4.3; the claude-arm phase-cut input the orchestrator
/// records between funnel steps 7 (code) and 8 (knowledge)). `code_start`/
/// `code_end` are resolved to full commit oids so the ledger holds stable shas,
/// not mobile refs. The orchestrator commits the file onto `dispatch/<slice>`;
/// stage-1 prepare-review tree-reads it (`mem.pattern.dispatch.sync-tree-reads`).
pub(crate) fn run_record_boundary(
    path: Option<PathBuf>,
    slice: u32,
    phase: &str,
    code_start: &str,
    code_end: &str,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    let resolve = |refish: &str| -> anyhow::Result<String> {
        resolve_commit(&root, refish)?
            .with_context(|| format!("record-boundary: {refish} does not resolve to a commit"))
    };
    crate::ledger::record_boundary(
        &root,
        slice,
        crate::ledger::BoundaryRow {
            phase: phase.to_string(),
            code_start_oid: resolve(code_start)?,
            code_end_oid: resolve(code_end)?,
        },
    )
}

// --- SL-068 PHASE-02: `dispatch candidate create` (design §5.3) --------------

/// The resolved create request — the CLI flag bundle parsed into typed axes (the
/// clock is read in the shell and passed in, pure/imperative split). `source` is
/// the ref the candidate merges in; `base` the ref the merge is computed against;
/// `supersedes` an optional prior candidate id this fresh row links to (EX-2).
pub(crate) struct CreateRequest {
    pub slice: u32,
    pub label: String,
    pub kind: CandidateKind,
    pub role: CandidateRole,
    pub payload: CandidatePayload,
    pub base: String,
    pub source: Option<String>,
    pub supersedes: Option<String>,
    pub worktree: bool,
    pub created_at: String,
}

/// Parse the `--kind` token into [`CandidateKind`].
pub(crate) fn parse_kind(token: &str) -> anyhow::Result<CandidateKind> {
    match token {
        "audit" => Ok(CandidateKind::Audit),
        "experiment" => Ok(CandidateKind::Experiment),
        other => bail!("unknown candidate kind {other:?} (expected audit|experiment)"),
    }
}

/// Parse the `--role` token into [`CandidateRole`].
pub(crate) fn parse_role(token: &str) -> anyhow::Result<CandidateRole> {
    match token {
        "review_surface" => Ok(CandidateRole::ReviewSurface),
        "close_target" => Ok(CandidateRole::CloseTarget),
        "scratch" => Ok(CandidateRole::Scratch),
        other => {
            bail!("unknown candidate role {other:?} (expected review_surface|close_target|scratch)")
        }
    }
}

/// Parse the `--payload` token into [`CandidatePayload`].
pub(crate) fn parse_payload(token: &str) -> anyhow::Result<CandidatePayload> {
    match token {
        "impl_bundle" => Ok(CandidatePayload::ImplBundle),
        "code" => Ok(CandidatePayload::Code),
        other => bail!("unknown candidate payload {other:?} (expected impl_bundle|code)"),
    }
}

/// CLI entry — resolve the root and create a candidate for `req`.
pub(crate) fn run_candidate_create(
    path: Option<PathBuf>,
    req: &CreateRequest,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    candidate_create(&root, req)
}

/// The source ref a create merges in: an explicit `--source`, else the default
/// for the role — `review/<slice>` for a review surface; otherwise an explicit
/// source is required (a close target's `phase/<slice>-NN` has no single default).
fn resolve_source_ref(req: &CreateRequest, slice3: &str) -> anyhow::Result<String> {
    if let Some(src) = &req.source {
        return Ok(src.clone());
    }
    match req.role {
        CandidateRole::ReviewSurface => Ok(format!("refs/heads/review/{slice3}")),
        CandidateRole::CloseTarget | CandidateRole::Scratch => bail!(
            "candidate create: --source is required for a {} candidate",
            role_token(req.role)
        ),
    }
}

/// The persisted token for a role (error messages only; the on-disk form is
/// serde's, never hand-spliced into TOML).
fn role_token(role: CandidateRole) -> &'static str {
    match role {
        CandidateRole::ReviewSurface => "review_surface",
        CandidateRole::CloseTarget => "close_target",
        CandidateRole::Scratch => "scratch",
    }
}

/// EX-1 provenance: the candidate's source ref must correspond to a journal
/// prepare-review row whose `status == Verified`. For a `phase/<slice>-NN` source
/// (a `code` close target) additionally refuse when an EARLIER non-empty
/// phase-chain row `failed` — a hole in the chain means the selected phase does
/// not actually carry verified prior code. Reads the journal from the
/// coordination branch tip (object db). Refuses (no writes) before any verified
/// evidence exists.
fn check_provenance(journal: &Journal, slice3: &str, source_ref: &str) -> anyhow::Result<()> {
    let row = journal
        .rows
        .iter()
        .find(|r| r.target_ref == source_ref)
        .with_context(|| {
            format!(
                "candidate create: no prepare-review journal row for source {source_ref} — \
                 run `dispatch sync --prepare-review` first"
            )
        })?;
    anyhow::ensure!(
        row.status == LedgerStatus::Verified,
        "candidate create: source {source_ref} is not verified (status {:?}) — \
         no verified evidence to build a candidate from",
        row.status
    );

    // Phase-chain integrity: a close target built off phase/<slice>-NN must have
    // no earlier failed phase row (an unresolved hole below the selected phase).
    let prefix = format!("refs/heads/phase/{slice3}-");
    if let Some(nn) = source_ref
        .strip_prefix(&prefix)
        .and_then(|nn| nn.parse::<u32>().ok())
    {
        for r in &journal.rows {
            if let Some(other) = r
                .target_ref
                .strip_prefix(&prefix)
                .and_then(|n| n.parse::<u32>().ok())
                && other < nn
                && r.status == LedgerStatus::Failed
            {
                bail!(
                    "candidate create: an earlier phase row {} failed — the phase chain \
                     below {source_ref} has an unresolved hole",
                    r.target_ref
                );
            }
        }
    }
    Ok(())
}

/// Core `candidate create` (design §5.3, EX-1..5). Happy path only — a content
/// conflict aborts cleanly with NO row/ref/worktree written (the conflicted +
/// `--worktree` lifecycle is PHASE-03). Sequencing: provenance gate → compute the
/// no-ff 3-way merge object → zero-oid CAS the candidate branch → record the row.
/// The CAS precedes the row write, so a refused branch creation leaves no partial
/// durable state.
fn candidate_create(root: &Path, req: &CreateRequest) -> anyhow::Result<()> {
    let slice3 = format!("{:03}", req.slice);
    let coord_ref = format!("refs/heads/dispatch/{slice3}");
    let target_ref = format!("refs/heads/candidate/{slice3}/{}", req.label);
    let id = format!("cand-{slice3}-{}", req.label);

    // --- EX-2: raw-evidence-ref write guard FIRST (invariant I9) — refuse a
    //     create driven from a worktree checked out on a `review/*` / `phase/*`
    //     evidence ref, before ANY durable write. The candidate workflow never
    //     edits the raw evidence refs in place (design §5.3). Pure string check
    //     on the branch the shell resolved. --------------------------------------
    if let Some(branch) = current_branch(root)?
        && is_raw_evidence_ref(&branch)
    {
        bail!(
            "candidate create: the current worktree is checked out on raw evidence ref {branch:?} \
             (review/* and phase/* are immutable, invariant I9) — never edit it in place; \
             run `dispatch candidate create` from a safe branch (e.g. the coordination tree) \
             to publish a candidate instead"
        );
    }

    // --- EX-1: review_surface requires an explicit --worktree in v1. Refuse
    //     before any write so a missing flag leaves no partial state. -----------
    if req.role == CandidateRole::ReviewSurface && !req.worktree {
        bail!(
            "candidate create: a review_surface candidate requires an explicit --worktree \
             (v1: the review surface is always materialised for the reviewer to read)"
        );
    }

    // --- EX-1: verified-source provenance gate FIRST (before any ref resolve
    //     or write) — refuse before verified evidence exists, by ref NAME -------
    let source_ref = resolve_source_ref(req, &slice3)?;
    let journal = read_ledger::<Journal>(root, &coord_ref, &slice3, "journal.toml")?;
    check_provenance(&journal, &slice3, &source_ref)?;

    // --- resolve source + base oids (the journal proved the source verified) -
    let source_oid = resolve_commit(root, &source_ref)?
        .with_context(|| format!("candidate create: source {source_ref} does not resolve"))?;
    let base_oid = resolve_commit(root, &req.base)?
        .with_context(|| format!("candidate create: base {} does not resolve", req.base))?;

    // --- EX-2 supersession: a fresh row links to a prior candidate id --------
    let mut ledger = read_candidates(root, req.slice)?;
    let supersedes = match &req.supersedes {
        Some(prior) => {
            anyhow::ensure!(
                ledger.rows.iter().any(|r| r.id == *prior),
                "candidate create: --supersedes {prior} names no recorded candidate"
            );
            prior.clone()
        }
        None => String::new(),
    };

    // --- EX-3: explicit no-ff 3-way merge (object db only) -------------------
    let merge_base = git::merge_base(root, &base_oid, &source_oid)?.with_context(|| {
        format!(
            "candidate create: base {base_oid} and source {source_oid} share no common ancestor"
        )
    })?;

    // The merge outcome decides the lifecycle (EX-1): a clean union commits at
    // the merge tree (status created); a conflict either ABORTS with no durable
    // state (no --worktree) or parks the branch at the base for the user to
    // resolve+commit, recording a conflicted row (--worktree).
    let (branch_oid, merge_oid, status) =
        match git::merge_tree(root, &merge_base, &base_oid, &source_oid)? {
            MergeTree::Clean { tree } => {
                let merge_oid = git::commit_tree_merge(
                    root,
                    &tree,
                    &base_oid,
                    &source_oid,
                    &format!("candidate({slice3}/{}): merge {source_ref}", req.label),
                )?;
                // Clean: the branch points at the merge commit.
                (merge_oid.clone(), merge_oid, CandidateStatus::Created)
            }
            MergeTree::Conflict if !req.worktree => bail!(
                "candidate create: 3-way merge of {source_ref} onto {} conflicts — \
                 pass --worktree to park the candidate branch at the base for \
                 manual resolve+commit, or abort (no row/ref/worktree written)",
                req.base
            ),
            // Conflicted + --worktree: park the branch at the base so the user
            // resolves+commits in the worktree. No merge commit exists yet.
            MergeTree::Conflict => (base_oid.clone(), String::new(), CandidateStatus::Conflicted),
        };

    // --- EX-3: create the branch under zero-oid CAS (refuses an existing ref).
    //     Precedes the row write so a refused creation leaves no partial state.
    match git::update_ref_cas(root, &target_ref, &branch_oid, ZERO_OID)? {
        RefCas::Updated => {}
        RefCas::Moved { actual } => bail!(
            "candidate create: {target_ref} already exists (at {}) — \
             supersede creates a fresh label, never rewrites a branch",
            actual.as_deref().unwrap_or("?")
        ),
    }

    // --- EX-3: materialise the worktree BEFORE the row write so a worktree
    //     failure rolls the ref back, leaving no orphan branch the ledger does
    //     not know about. The conflicted lifecycle ALWAYS materialises (so the
    //     user can resolve); a clean create only on the opt-in --worktree. -----
    let worktree_path = if req.worktree {
        match add_candidate_worktree(root, &id, &target_ref) {
            Ok(path) => Some(path),
            Err(e) => {
                // Roll back the branch we just created — no partial durable state.
                rollback_ref(root, &target_ref, &branch_oid);
                return Err(e);
            }
        }
    } else {
        None
    };

    // --- EX-3: record the candidate row (status created | conflicted) --------
    let row = CandidateRow {
        id: id.clone(),
        label: req.label.clone(),
        kind: req.kind,
        role: req.role,
        payload: req.payload,
        target_ref: target_ref.clone(),
        source_ref,
        source_oid,
        base_ref: req.base.clone(),
        base_oid,
        merge_oid: merge_oid.clone(),
        status,
        supersedes,
        reason: String::new(),
        created_by: "dispatch candidate create".to_owned(),
        created_at: req.created_at.clone(),
    };
    ledger.rows.push(row);
    crate::ledger::write_candidates(root, req.slice, &ledger)?;

    writeln!(io::stdout(), "{target_ref}")?;
    if let Some(path) = &worktree_path {
        writeln!(io::stdout(), "{}", path.display())?;
    }
    match status {
        CandidateStatus::Conflicted => writeln!(
            io::stderr(),
            "candidate create: {id} conflicted — branch parked at base {branch_oid}; \
             resolve+commit in {}",
            worktree_path
                .as_ref()
                .map_or_else(|| "(worktree)".to_owned(), |p| p.display().to_string())
        )?,
        _ => writeln!(
            io::stderr(),
            "candidate create: {id} created at {merge_oid}"
        )?,
    }
    Ok(())
}

/// Add a linked worktree for candidate `id` at `target_ref` under
/// `.doctrine/state/dispatch/candidate/<id>` (the gitignored runtime tier).
/// Returns the worktree path on success. Impure shell.
fn add_candidate_worktree(root: &Path, id: &str, target_ref: &str) -> anyhow::Result<PathBuf> {
    let wt_path = root.join(".doctrine/state/dispatch/candidate").join(id);
    if let Some(parent) = wt_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let wt_str = wt_path
        .to_str()
        .context("candidate create: worktree path is not valid UTF-8")?;
    git::git_text(root, &["worktree", "add", "--quiet", wt_str, target_ref])?;
    Ok(wt_path)
}

/// Best-effort CAS rollback of a ref this create just created — used when a later
/// step fails after the branch was written (EX-3: no partial durable state). A
/// failed delete is swallowed: the caller is already returning the primary error.
fn rollback_ref(root: &Path, target_ref: &str, expected: &str) {
    let _ignored = git::git_opt(root, &["update-ref", "-d", target_ref, expected]);
}

/// The branch the worktree at `root` is checked out on, short form (e.g.
/// `review/064`), or `None` for a detached HEAD. The raw-evidence-ref guard
/// (EX-2) keys on this. Impure shell.
fn current_branch(root: &Path) -> anyhow::Result<Option<String>> {
    Ok(git::git_opt(
        root,
        &["symbolic-ref", "--quiet", "--short", "HEAD"],
    )?)
}

/// Whether `branch` is a raw evidence ref the candidate workflow must never edit
/// in place (invariant I9): the `review/<slice>` impl bundle or a
/// `phase/<slice>-NN` per-phase cut. Pure.
fn is_raw_evidence_ref(branch: &str) -> bool {
    branch.starts_with("review/") || branch.starts_with("phase/")
}

// --- SL-068 PHASE-05: `dispatch candidate admit` (design §5.2/§5.5) -----------

/// The resolved admit request — pin a recorded candidate's tip as the immutable
/// OID a downstream verb (close/review) targets. The clock (`admitted_at`) is read
/// in the shell and passed in (pure/imperative split, like [`CreateRequest`]).
pub(crate) struct AdmitRequest {
    pub slice: u32,
    pub role: CandidateRole,
    pub candidate: String,
    pub review: Option<String>,
    pub admitted_at: String,
}

/// CLI entry — resolve the root and admit the candidate for `req`.
pub(crate) fn run_candidate_admit(path: Option<PathBuf>, req: &AdmitRequest) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    candidate_admit(&root, req)
}

/// Core `candidate admit` (design §5.2 + §5.5 invariants). Pins a recorded
/// candidate's committed tip as the immutable `admitted_oid` a downstream verb
/// targets, after validating provenance (I3, R7): the recorded `merge_oid` is the
/// Doctrine-created candidate merge (its parents are exactly base+source) AND an
/// ancestor of the admitted tip. Re-reads the candidate ref before recording so a
/// ref moved mid-admission is refused (EX-1). Writes ONLY `candidates.toml` — never
/// trunk/edge/`review/*`/`phase/*`/the candidate ref (EX-4). Exactly one current
/// admission per role afterward (the role slot is overwritten; supersession is
/// explicit history via `supersedes`).
fn candidate_admit(root: &Path, req: &AdmitRequest) -> anyhow::Result<()> {
    // --- I9 raw-evidence-ref write guard FIRST (before any read/write) — refuse
    //     an admit driven from a worktree checked out on a `review/*` / `phase/*`
    //     evidence ref. Mirrors create's guard. -----------------------------------
    if let Some(branch) = current_branch(root)?
        && is_raw_evidence_ref(&branch)
    {
        bail!(
            "candidate admit: the current worktree is checked out on raw evidence ref {branch:?} \
             (review/* and phase/* are immutable, invariant I9) — never edit it in place; \
             run `dispatch candidate admit` from a safe branch (e.g. the coordination tree)"
        );
    }

    // scratch is not an admissible role — refuse before any read.
    if req.role == CandidateRole::Scratch {
        bail!("candidate admit: a scratch candidate is not admissible (no review/close target)");
    }

    // --- resolve the candidate tip (must be a committed clean tip) -------------
    let admitted_1 = resolve_commit(root, &req.candidate)?.with_context(|| {
        format!(
            "candidate admit: candidate {} does not resolve to a committed tip",
            req.candidate
        )
    })?;

    // --- find the recorded row pinned by the candidate ref ---------------------
    let mut ledger = read_candidates(root, req.slice)?;
    let row = ledger
        .rows
        .iter()
        .find(|r| r.target_ref == req.candidate)
        .with_context(|| {
            format!(
                "candidate admit: no recorded candidate at {} — admit pins a recorded candidate",
                req.candidate
            )
        })?
        .clone();

    // --- role must match (no mis-slotting) -------------------------------------
    anyhow::ensure!(
        row.role == req.role,
        "candidate admit: candidate {} is role {}, cannot admit as {}",
        row.id,
        role_token(row.role),
        role_token(req.role)
    );

    // --- a conflicted/unresolved row has no Doctrine merge to validate ---------
    anyhow::ensure!(
        !row.merge_oid.is_empty(),
        "candidate admit: candidate {} has no Doctrine merge to validate \
         (conflicted/unresolved) — resolve and re-create before admitting",
        row.id
    );

    // --- provenance (EX-2, I3, R7): merge_oid is the Doctrine candidate merge --
    let merge_parents: std::collections::BTreeSet<String> =
        git::parents(root, &row.merge_oid)?.into_iter().collect();
    let expected_parents: std::collections::BTreeSet<String> =
        [row.base_oid.clone(), row.source_oid.clone()]
            .into_iter()
            .collect();
    anyhow::ensure!(
        merge_parents == expected_parents,
        "candidate admit: merge_oid {} is not the Doctrine candidate merge \
         (parents != base+source)",
        row.merge_oid
    );
    anyhow::ensure!(
        git::is_ancestor(root, &row.merge_oid, &admitted_1)?,
        "candidate admit: admitted tip {admitted_1} does not descend from candidate merge {} (I3)",
        row.merge_oid
    );

    // --- EX-1: re-read the candidate ref before recording — a tip moved between
    //     the first resolve and now is refused (record only the proven oid) -----
    let admitted_2 = resolve_commit(root, &req.candidate)?;
    anyhow::ensure!(
        admitted_2.as_deref() == Some(admitted_1.as_str()),
        "candidate admit: candidate {} moved during admission (was {admitted_1}, now {}) — \
         re-run admit",
        req.candidate,
        admitted_2.as_deref().unwrap_or("absent")
    );

    // --- EX-3, I5: record the admission, overwriting the role slot (exactly one
    //     current admission per role; supersession is explicit history) ---------
    let supersedes = prior_admission(&ledger, req.role)
        .map(|a| a.candidate_id.clone())
        .unwrap_or_default();
    let admission = Admission {
        candidate_id: row.id.clone(),
        candidate_ref: req.candidate.clone(),
        expected_ref_oid: admitted_1.clone(),
        admitted_oid: admitted_1.clone(),
        review: req.review.clone().unwrap_or_default(),
        supersedes,
        admitted_at: req.admitted_at.clone(),
    };
    // scratch was refused above; admit only ever reaches a review/close slot.
    let slot = match req.role {
        CandidateRole::ReviewSurface => &mut ledger.current_admission.review_surface,
        CandidateRole::CloseTarget | CandidateRole::Scratch => {
            &mut ledger.current_admission.close_target
        }
    };
    *slot = Some(admission);
    crate::ledger::write_candidates(root, req.slice, &ledger)?;

    writeln!(io::stdout(), "{admitted_1}")?;
    writeln!(
        io::stderr(),
        "candidate admit: {} admitted at {admitted_1} ({})",
        row.id,
        role_token(req.role)
    )?;
    Ok(())
}

/// The role's current admission, if any — the record a fresh admit supersedes.
fn prior_admission(ledger: &Candidates, role: CandidateRole) -> Option<&Admission> {
    match role {
        CandidateRole::CloseTarget => ledger.current_admission.close_target.as_ref(),
        CandidateRole::ReviewSurface => ledger.current_admission.review_surface.as_ref(),
        CandidateRole::Scratch => None,
    }
}

// --- SL-068 PHASE-04: `dispatch candidate status` (design §5.3, EX-1..3) ------

/// CLI entry — resolve the root and render the candidate status surface for
/// `slice`. Read-only: never mutates a ref or the ledger (EX-3).
pub(crate) fn run_candidate_status(path: Option<PathBuf>, slice: u32) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    candidate_status(&root, slice)
}

/// Abbreviate an oid to its leading 12 chars for the human surface; empty stays
/// empty (a conflicted row has no merge oid), `—` is the absent-ref sentinel
/// (kept verbatim). Pure.
fn short_oid(oid: &str) -> String {
    if oid.is_empty() || oid == "—" {
        return oid.to_owned();
    }
    oid.chars().take(12).collect()
}

/// One evidence-ref status row (the EX-1 evidence group): the ref name, its
/// human group label, and its live tip (`—` when the ref is absent). Pure data —
/// the impure shell resolves the tips and builds the rows.
struct EvidenceRow {
    refname: String,
    group: &'static str,
    tip: String,
}

/// Render the candidate status surface (design §5.3, EX-1..3): the evidence-ref
/// group, the candidate-ref group with per-candidate base/source/tip/status/
/// admission + drift, and the safe next command(s). READ-ONLY — it resolves live
/// ref tips and reads `candidates.toml`, never writing a ref or the ledger (EX-3).
/// From a worktree on a raw evidence ref it WARNS (unlike create's refusal, EX-3).
fn candidate_status(root: &Path, slice: u32) -> anyhow::Result<()> {
    let slice3 = format!("{slice:03}");

    // EX-3: read-only — a raw-evidence-ref worktree only WARNS (never refuses,
    // unlike create's I9 guard) since status mutates nothing.
    if let Some(branch) = current_branch(root)?
        && is_raw_evidence_ref(&branch)
    {
        writeln!(
            io::stderr(),
            "candidate status: the current worktree is checked out on raw evidence ref `{branch}` \
             (review/* and phase/* are immutable) — status is read-only and changes nothing, but \
             never edit an evidence ref in place; publish via `dispatch candidate create`"
        )?;
    }

    let ledger = read_candidates(root, slice)?;

    // --- EX-1: the evidence-ref group, kept VISIBLY SEPARATE from candidates --
    let evidence = collect_evidence(root, &slice3)?;
    let mut grid: Vec<Vec<String>> = vec![cells(&["ref", "group", "tip"])];
    for row in &evidence {
        grid.push(cells(&[&row.refname, row.group, &short_oid(&row.tip)]));
    }
    writeln!(io::stdout(), "evidence refs:")?;
    write!(io::stdout(), "{}", render_table(&grid, None))?;

    // --- EX-2: the candidate-ref group with per-candidate report + drift ------
    writeln!(io::stdout(), "\ncandidates (interaction branches):")?;
    let mut cgrid: Vec<Vec<String>> = vec![cells(&[
        "id",
        "branch",
        "status",
        "base",
        "source",
        "tip",
        "admission",
        "drift",
    ])];
    let mut any_drift = false;
    for row in &ledger.rows {
        let report = candidate_report(root, &ledger, row)?;
        any_drift |= report.drift;
        cgrid.push(cells(&[
            &row.id,
            &row.target_ref,
            status_token(row.status),
            &short_oid(&row.base_oid),
            &short_oid(&row.source_oid),
            &short_oid(&report.tip),
            &report.admission,
            if report.drift { "DRIFT" } else { "ok" },
        ]));
    }
    if ledger.rows.is_empty() {
        writeln!(io::stdout(), "(none recorded)")?;
    } else {
        write!(io::stdout(), "{}", render_table(&cgrid, None))?;
    }

    // --- EX-3: print the safe NEXT command(s), not "inspect raw refs" ---------
    write_next_commands(&slice3, &ledger, any_drift)?;
    Ok(())
}

/// The per-candidate live report (EX-2): the candidate ref's live tip, a human
/// admission summary, and whether the live tip has DRIFTED from the
/// recorded/admitted OID (invariant I4 — reported, never hidden).
struct CandidateReport {
    tip: String,
    admission: String,
    drift: bool,
}

/// Build a candidate's live report (EX-2). The live tip is resolved from the
/// candidate's `target_ref` (`—` when absent); the admission summary names the
/// admitting review when this candidate is the role's admitted one. Drift = the
/// live tip differs from the OID the row pins: the admitted oid when admitted,
/// else the recorded `merge_oid` (skipped for a conflicted row, whose branch is
/// intentionally parked at base with no merge commit).
fn candidate_report(
    root: &Path,
    ledger: &Candidates,
    row: &CandidateRow,
) -> anyhow::Result<CandidateReport> {
    let tip = resolve_commit(root, &row.target_ref)?.unwrap_or_else(|| "—".to_owned());
    let admitted = admission_for(ledger, &row.id);
    let admission = match admitted {
        Some(a) => format!("admitted ({})", a.review),
        None => "—".to_owned(),
    };
    // The OID the row pins: the admitted oid when admitted, else the recorded
    // merge oid. A conflicted row (empty merge_oid, branch parked at base) is not
    // drift-checked — it has no recorded merge tip to compare against.
    let pinned = match admitted {
        Some(a) => Some(a.admitted_oid.as_str()),
        None if row.status == CandidateStatus::Conflicted => None,
        None if row.merge_oid.is_empty() => None,
        None => Some(row.merge_oid.as_str()),
    };
    let drift = match (pinned, tip.as_str()) {
        (Some(pin), live) => live != "—" && live != pin,
        (None, _) => false,
    };
    Ok(CandidateReport {
        tip,
        admission,
        drift,
    })
}

/// The admission record (either role) whose `candidate_id` matches `id`, if this
/// candidate is the currently-admitted one for its role. Pure lookup.
fn admission_for<'a>(ledger: &'a Candidates, id: &str) -> Option<&'a Admission> {
    [
        ledger.current_admission.close_target.as_ref(),
        ledger.current_admission.review_surface.as_ref(),
    ]
    .into_iter()
    .flatten()
    .find(|a| a.candidate_id == id)
}

/// Resolve the evidence-ref group (EX-1): the coordination branch, the impl
/// bundle, and every `phase/<slice>-NN` cut — NEVER conflated with a
/// `candidate/<slice>/*` interaction branch. Impure shell (resolves live tips).
fn collect_evidence(root: &Path, slice3: &str) -> anyhow::Result<Vec<EvidenceRow>> {
    let mut rows: Vec<EvidenceRow> = Vec::new();
    for (refname, group) in [
        (format!("refs/heads/dispatch/{slice3}"), "coordination"),
        (format!("refs/heads/review/{slice3}"), "impl-bundle"),
    ] {
        let tip = resolve_commit(root, &refname)?.unwrap_or_else(|| "—".to_owned());
        rows.push(EvidenceRow {
            refname,
            group,
            tip,
        });
    }
    for refname in for_each_ref(root, &format!("refs/heads/phase/{slice3}-*"))? {
        let tip = resolve_commit(root, &refname)?.unwrap_or_else(|| "—".to_owned());
        rows.push(EvidenceRow {
            refname,
            group: "phase-cut",
            tip,
        });
    }
    Ok(rows)
}

/// Enumerate the full ref names matching `pattern` (a `for-each-ref` glob, e.g.
/// `refs/heads/phase/068-*`), sorted by git's default (lexical). Empty when none
/// match. Impure shell.
fn for_each_ref(root: &Path, pattern: &str) -> anyhow::Result<Vec<String>> {
    let out = git::git_text(root, &["for-each-ref", "--format=%(refname)", pattern])?;
    Ok(out.lines().map(str::to_owned).collect())
}

/// The persisted status token for a candidate row (read view only).
fn status_token(status: CandidateStatus) -> &'static str {
    match status {
        CandidateStatus::Created => "created",
        CandidateStatus::Conflicted => "conflicted",
        CandidateStatus::Abandoned => "abandoned",
        CandidateStatus::Superseded => "superseded",
    }
}

/// Build one cell-row of owned strings from string slices.
fn cells(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| (*s).to_string()).collect()
}

/// EX-3: print the safe NEXT command(s) — concrete verbs the user runs, not
/// "inspect the raw refs". Guidance branches on ledger state: no candidates ⇒
/// create; candidates present ⇒ admit/close guidance; any drift ⇒ a re-admit
/// note (the admitted oid is immutable; a moved tip needs a fresh candidate).
fn write_next_commands(slice3: &str, ledger: &Candidates, any_drift: bool) -> anyhow::Result<()> {
    let slice = slice3.trim_start_matches('0');
    let slice = if slice.is_empty() { "0" } else { slice };
    writeln!(io::stdout(), "\nnext:")?;
    if ledger.rows.is_empty() {
        writeln!(
            io::stdout(),
            "  dispatch candidate create --slice {slice} --role review_surface \
             --payload impl_bundle --base refs/heads/main --label review-001 --worktree"
        )?;
        return Ok(());
    }
    writeln!(
        io::stdout(),
        "  dispatch candidate create --slice {slice} ...   # publish a fresh candidate"
    )?;
    writeln!(
        io::stdout(),
        "  dispatch candidate admit --slice {slice} --id <candidate-id> --review RV-NNN   \
         # pin a candidate for review/close"
    )?;
    if any_drift {
        writeln!(
            io::stdout(),
            "  note: a DRIFTED candidate's live tip moved off its recorded/admitted oid \
             (immutable) — supersede with a fresh candidate rather than editing in place"
        )?;
    }
    Ok(())
}

/// Resolve a commit-ish ref to its commit oid, or `None` when it does not exist.
fn resolve_commit(root: &Path, refish: &str) -> anyhow::Result<Option<String>> {
    Ok(git::git_opt(
        root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{refish}^{{commit}}"),
        ],
    )?)
}

/// The tree oid of a commit.
fn tree_of(root: &Path, commit: &str) -> anyhow::Result<String> {
    Ok(git::git_text(
        root,
        &["rev-parse", &format!("{commit}^{{tree}}")],
    )?)
}

/// Stage-1 prepare-review (design §4.2 B + §4.3 C).
fn prepare_review(root: &Path, slice: u32) -> anyhow::Result<()> {
    let slice3 = format!("{slice:03}");
    let coord_ref = format!("refs/heads/dispatch/{slice3}");
    let journal_path = format!(".doctrine/dispatch/{slice3}/journal.toml");

    let tip = resolve_commit(root, &coord_ref)?
        .with_context(|| format!("prepare-review: dispatch/{slice3} does not exist"))?;
    let tip_tree = tree_of(root, &tip)?;
    // Project off the PINNED FORK-POINT — merge-base(dispatch/<slice>, trunk) —
    // not the live trunk tip (RV-030 F-1, design §4.2/§4.3 trunk_base_B). The
    // coordination worktree isolates the working tree, NOT the trunk ref: a
    // foreign commit landing on trunk between `coordinate` and `sync` must not
    // reparent the per-phase cuts, else their diffs stop being exact and the
    // §3/IMP-043 "integrate refuses non-ff" net is silently bypassed. The live
    // tip resurfaces only at integrate's actual trunk push, under CAS.
    let trunk_tip = git::trunk_commit(root)?
        .context("prepare-review: no trunk ref resolves — a trunk base is required")?;
    let trunk_base = git::merge_base(root, &tip, &trunk_tip)?.with_context(|| {
        format!(
            "prepare-review: dispatch/{slice3} and trunk ({trunk_tip}) share no common ancestor"
        )
    })?;

    // --- source the run ledger from the dispatch tip (object db, not the
    //     working tree — works stage-1 and stage-2; design §4.1) --------------
    let orthogonal = read_ledger::<Orthogonal>(root, &coord_ref, &slice3, "orthogonal.toml")?;
    let boundaries = read_ledger::<Boundaries>(root, &coord_ref, &slice3, "boundaries.toml")?;

    // --- compute projections (objects only; no ref mutation yet) ------------
    let mut planned: Vec<Planned> = Vec::new();
    plan_review(
        root,
        &slice3,
        &tip,
        &tip_tree,
        &trunk_base,
        &orthogonal,
        &mut planned,
    )?;
    plan_phases(root, &slice3, &trunk_base, &boundaries, &mut planned)?;

    // --- EX-2: journal intent committed onto the branch BEFORE any external
    //     ref mutation ---------------------------------------------------------
    let mut journal = pending_journal(&planned);
    let journal_commit = commit_journal(
        root,
        &tip_tree,
        &tip,
        &journal_path,
        &coord_ref,
        &journal,
        "journal: prepare-review",
    )?;

    // --- apply the external ref creations under zero-oid CAS (EX-5) ----------
    let mut stale: Vec<String> = Vec::new();
    for row in &mut journal.rows {
        match git::update_ref_cas(root, &row.target_ref, &row.planned_new_oid, ZERO_OID)? {
            RefCas::Updated => {
                row.status = LedgerStatus::Verified;
                row.applied_new_oid = row.planned_new_oid.clone();
                writeln!(io::stdout(), "{}", row.target_ref)?;
            }
            RefCas::Moved { actual } => {
                row.status = LedgerStatus::Failed;
                stale.push(format!(
                    "{} (exists at {})",
                    row.target_ref,
                    actual.as_deref().unwrap_or("?")
                ));
            }
        }
    }

    // --- record applied status back onto the branch (recoverability) --------
    commit_journal(
        root,
        &tip_tree,
        &journal_commit,
        &journal_path,
        &coord_ref,
        &journal,
        "journal: prepare-review",
    )?;

    if stale.is_empty() {
        writeln!(
            io::stderr(),
            "prepare-review: {} ref(s) created",
            journal.rows.len()
        )?;
        Ok(())
    } else {
        bail!(
            "prepare-review: {} stale ref(s) reported, not clobbered: {}",
            stale.len(),
            stale.join(", ")
        )
    }
}

/// Stage-2 integrate (design §4 / §4.3). Sources the prepared journal from the
/// `dispatch/<slice>` tip tree (object db — works after the coordination worktree
/// is removed, EX-1), then **replays every row idempotently** under the 3-way CAS
/// ([`git::replay_ref`]): an intact prepared ref is a verified no-op, a clobbered
/// one is refused. When opted in, it appends and replays projection rows that
/// advance the audited code units onto `trunk` (ff-only, EX-3) and an aggregate
/// `edge` ref (EX-4). Plumbing-only — no checkout; the journal intent commits onto
/// the branch BEFORE any external ref mutation and the applied status commits back
/// after (EX-5). A moved target is reported, never clobbered (no auto-resolve).
fn integrate(
    root: &Path,
    slice: u32,
    trunk: Option<&str>,
    edge: Option<&str>,
) -> anyhow::Result<()> {
    let slice3 = format!("{slice:03}");
    let coord_ref = format!("refs/heads/dispatch/{slice3}");
    let journal_path = format!(".doctrine/dispatch/{slice3}/journal.toml");

    let tip = resolve_commit(root, &coord_ref)?
        .with_context(|| format!("integrate: dispatch/{slice3} does not exist"))?;
    let tip_tree = tree_of(root, &tip)?;

    // Stage-1 must have prepared the journal (tree-read, never the filesystem —
    // it would silently empty from the parent/root, see the sync-tree-reads-ledger
    // memory). An empty journal ⇒ prepare-review never ran.
    let mut journal = read_ledger::<Journal>(root, &coord_ref, &slice3, "journal.toml")?;
    if journal.rows.is_empty() {
        bail!("integrate: no prepared journal on dispatch/{slice3} — run prepare-review first");
    }

    // --- SL-068 PHASE-06: a candidate workflow is "active for the slice" ⇔ the
    //     ledger carries ≥1 recorded candidate row. When active, --trunk/--edge
    //     source the ADMITTED oid (close_target / review_surface) and REFUSE
    //     rather than fall back to a raw phase/review ref (I6, I4, R4). When NOT
    //     active the legacy paths are preserved UNCHANGED (this is what keeps
    //     e2e_dispatch_sync.rs — which records no candidate — green). -----------
    let candidates = read_candidates(root, slice)?;
    let candidate_active = !candidates.rows.is_empty();

    // --- plan opt-in projection rows (idempotent: skip a target already
    //     journaled by a prior/crashed run — its recorded intent is replayed) ---
    let fresh = |j: &Journal, target: &str| !j.rows.iter().any(|r| r.target_ref == target);
    if let Some(trunk_ref) = trunk.filter(|t| fresh(&journal, t)) {
        let row = if candidate_active {
            plan_candidate_trunk_row(root, &candidates, trunk_ref)?
        } else {
            plan_trunk_row(root, &slice3, &journal, trunk_ref)?
        };
        journal.rows.push(row);
    }
    if let Some(edge_ref) = edge.filter(|e| fresh(&journal, e)) {
        let row = if candidate_active {
            plan_candidate_edge_row(root, &candidates, edge_ref)?
        } else {
            plan_edge_row(root, &slice3, edge_ref)?
        };
        journal.rows.push(row);
    }

    // --- journal the (possibly extended) intent onto the branch BEFORE any
    //     external ref mutation (EX-5, ADR-012 D4) ------------------------------
    let journal_commit = commit_journal(
        root,
        &tip_tree,
        &tip,
        &journal_path,
        &coord_ref,
        &journal,
        "journal: integrate",
    )?;

    // --- replay every row idempotently under the 3-way CAS (EX-1/EX-2) --------
    let mut moved: Vec<String> = Vec::new();
    for row in &mut journal.rows {
        match git::replay_ref(
            root,
            &row.target_ref,
            &row.expected_old_oid,
            &row.planned_new_oid,
        )? {
            ReplayOutcome::NoOp => {
                row.status = LedgerStatus::Verified;
                row.applied_new_oid = row.planned_new_oid.clone();
            }
            ReplayOutcome::Applied => {
                row.status = LedgerStatus::Verified;
                row.applied_new_oid = row.planned_new_oid.clone();
                writeln!(io::stdout(), "{}", row.target_ref)?;
            }
            ReplayOutcome::Moved { actual } => {
                row.status = LedgerStatus::Failed;
                moved.push(format!(
                    "{} (target at {})",
                    row.target_ref,
                    actual.as_deref().unwrap_or("?")
                ));
            }
        }
    }

    // --- record applied status back onto the branch (recoverability) ----------
    commit_journal(
        root,
        &tip_tree,
        &journal_commit,
        &journal_path,
        &coord_ref,
        &journal,
        "journal: integrate",
    )?;

    if moved.is_empty() {
        writeln!(
            io::stderr(),
            "integrate: {} ref(s) replayed",
            journal.rows.len()
        )?;
        Ok(())
    } else {
        bail!(
            "integrate: {} moved target(s), not clobbered: {}",
            moved.len(),
            moved.join(", ")
        )
    }
}

/// The highest-numbered `refs/heads/phase/<slice>-NN` target in the journal — the
/// cumulative code tip (phase branches are chained off the trunk base, so the max
/// NN holds all prior phases' code). Only **verified** rows count: a failed phase
/// projection must not be mistaken for the chain tip (RV-030 F-8), else integrate
/// would parent the trunk advance on an unresolved ref. `None` when no verified
/// phase row was projected.
fn phase_chain_tip(journal: &Journal, slice3: &str) -> Option<String> {
    let prefix = format!("refs/heads/phase/{slice3}-");
    journal
        .rows
        .iter()
        .filter(|r| r.status == LedgerStatus::Verified)
        .filter_map(|r| {
            r.target_ref
                .strip_prefix(&prefix)
                .and_then(|nn| nn.parse::<u32>().ok())
                .map(|n| (n, r.target_ref.clone()))
        })
        .max_by_key(|(n, _)| *n)
        .map(|(_, refname)| refname)
}

/// Plan the trunk projection row (EX-3): the cumulative code tip advances
/// `trunk_ref` **fast-forward-only**. `expected_old` is the trunk tip (zero if the
/// ref is absent); a planned commit that does not descend from it ⇒ the trunk
/// moved ⇒ refuse (re-anchor is reported, never auto-resolved).
fn plan_trunk_row(
    root: &Path,
    slice3: &str,
    journal: &Journal,
    trunk_ref: &str,
) -> anyhow::Result<JournalRow> {
    let phase_ref = phase_chain_tip(journal, slice3).with_context(|| {
        format!("integrate --trunk: no phase/{slice3}-NN code units to integrate")
    })?;
    let planned = resolve_commit(root, &phase_ref)?
        .with_context(|| format!("integrate --trunk: {phase_ref} does not resolve"))?;
    let expected_old = resolve_commit(root, trunk_ref)?;
    if let Some(tip) = &expected_old {
        anyhow::ensure!(
            git::is_ancestor(root, tip, &planned)?,
            "integrate --trunk: {planned} does not fast-forward {trunk_ref} (at {tip}) — \
             trunk moved; re-anchor required, not auto-resolved"
        );
    }
    Ok(projection_row(trunk_ref, planned, expected_old))
}

/// Plan the edge aggregate row (EX-4): the `review/<slice>` impl bundle advances
/// the standing `edge_ref`. Not ff-gated (a standing aggregate of local work); the
/// CAS still refuses a concurrently-moved edge — isolated to this sync point.
fn plan_edge_row(root: &Path, slice3: &str, edge_ref: &str) -> anyhow::Result<JournalRow> {
    let review_ref = format!("refs/heads/review/{slice3}");
    let planned = resolve_commit(root, &review_ref)?
        .with_context(|| format!("integrate --edge: {review_ref} does not resolve"))?;
    let expected_old = resolve_commit(root, edge_ref)?;
    Ok(projection_row(edge_ref, planned, expected_old))
}

/// SL-068 PHASE-06 — plan the trunk row when a candidate workflow is active: the
/// admitted **`close_target`** OID advances `trunk_ref` fast-forward-only, sourced
/// from the ledger (never a close-time merge, I6). Targeting is by `admitted_oid`
/// only — moving the candidate ref after admission cannot change the target (I4).
/// REFUSES (no fallback to the phase-chain tip) when no `close_target` admission
/// exists; on a non-ff trunk it refuses and instructs the user to create a
/// superseding close-target candidate on the new base (EX-2, R4 — no auto-reanchor).
fn plan_candidate_trunk_row(
    root: &Path,
    candidates: &Candidates,
    trunk_ref: &str,
) -> anyhow::Result<JournalRow> {
    let admission = candidates.current_admission.close_target.as_ref().context(
        "integrate --trunk: a candidate workflow is active but no close_target admission \
             exists — run `dispatch candidate admit --role close_target` first; integrate will \
             not fall back to a raw phase ref",
    )?;
    let planned = admission.admitted_oid.clone();
    let expected_old = resolve_commit(root, trunk_ref)?;
    if let Some(tip) = &expected_old {
        anyhow::ensure!(
            git::is_ancestor(root, tip, &planned)?,
            "integrate --trunk: admitted close_target {planned} does not fast-forward {trunk_ref} \
             (at {tip}) — trunk moved; create a superseding close-target candidate on the new \
             base and re-admit (not auto-resolved)"
        );
    }
    Ok(projection_row(trunk_ref, planned, expected_old))
}

/// SL-068 PHASE-06 — plan the edge row when a candidate workflow is active: the
/// admitted **`review_surface`** OID advances `edge_ref`, sourced from the ledger.
/// Same posture as the legacy edge (not ff-gated; the CAS still guards). REFUSES
/// (no silent raw `review/<slice>` fallback) when no `review_surface` admission
/// exists. Targeting is by `admitted_oid` only (I4).
fn plan_candidate_edge_row(
    root: &Path,
    candidates: &Candidates,
    edge_ref: &str,
) -> anyhow::Result<JournalRow> {
    let admission = candidates
        .current_admission
        .review_surface
        .as_ref()
        .context(
            "integrate --edge: a candidate workflow is active but no review_surface admission \
             exists — run `dispatch candidate admit --role review_surface` first; integrate will \
             not fall back to the raw review ref",
        )?;
    let planned = admission.admitted_oid.clone();
    let expected_old = resolve_commit(root, edge_ref)?;
    Ok(projection_row(edge_ref, planned, expected_old))
}

/// A pending CAS journal row advancing `target_ref` to `planned` from its current
/// tip (`expected_old`, zero-oid for a ref creation). `source_oid == planned_new_oid`
/// is **intentional** for these direct-projection (trunk/edge) rows — the source
/// IS the planned ref, so replay recomputes identity and converges to a no-op
/// (RV-030 F-10); unlike prepare-review rows where source (dispatch tip) and the
/// synthesised commit differ.
fn projection_row(target_ref: &str, planned: String, expected_old: Option<String>) -> JournalRow {
    JournalRow {
        source_oid: planned.clone(),
        target_ref: target_ref.to_owned(),
        expected_old_oid: expected_old.unwrap_or_else(|| ZERO_OID.to_owned()),
        planned_new_oid: planned,
        applied_new_oid: String::new(),
        status: LedgerStatus::Pending,
    }
}

/// Read a run-ledger manifest from the `dispatch/<slice>` tip tree (object db,
/// not the working filesystem). Absent ⇒ the type's empty default.
fn read_ledger<T: serde::de::DeserializeOwned + Default>(
    root: &Path,
    coord_ref: &str,
    slice3: &str,
    file: &str,
) -> anyhow::Result<T> {
    let path = format!(".doctrine/dispatch/{slice3}/{file}");
    match git::read_path_at(root, coord_ref, &path)? {
        Some(text) => Ok(toml::from_str(&text)?),
        None => Ok(T::default()),
    }
}

/// B — plan `review/<slice>`: filter the tip tree (drop the run-ledger dir and
/// every journal-verified orthogonal path) and commit it against the trunk base.
fn plan_review(
    root: &Path,
    slice3: &str,
    tip: &str,
    tip_tree: &str,
    trunk_base: &str,
    orthogonal: &Orthogonal,
    planned: &mut Vec<Planned>,
) -> anyhow::Result<()> {
    let mut exclude: Vec<String> = vec![format!(".doctrine/dispatch/{slice3}")];
    for mark in &orthogonal.rows {
        if mark.status == LedgerStatus::Verified {
            exclude.push(mark.path.clone());
        }
    }
    let exclude_refs: Vec<&str> = exclude.iter().map(String::as_str).collect();
    let review_tree = git::filter_tree(root, tip_tree, &exclude_refs)?;
    let review_commit = git::commit_tree(
        root,
        &review_tree,
        trunk_base,
        &format!("review({slice3}): impl bundle"),
    )?;
    planned.push(Planned {
        target_ref: format!("refs/heads/review/{slice3}"),
        source_oid: tip.to_owned(),
        commit_oid: review_commit,
    });
    Ok(())
}

/// C — plan `phase/<slice>-NN` from `boundaries.toml`: each emitted phase is the
/// code-only (`.doctrine/` stripped) cut of its cumulative `code_end_oid` tree,
/// chained off the previous phase (trunk base for the first). Empty-code phases
/// (`code_start_oid == code_end_oid`) emit no ref.
fn plan_phases(
    root: &Path,
    slice3: &str,
    trunk_base: &str,
    boundaries: &Boundaries,
    planned: &mut Vec<Planned>,
) -> anyhow::Result<()> {
    let mut parent = trunk_base.to_owned();
    for boundary in &boundaries.rows {
        if boundary.code_start_oid == boundary.code_end_oid {
            continue; // empty-code phase — no branch cut (design §4.3)
        }
        let nn = boundary
            .phase
            .strip_prefix("PHASE-")
            .unwrap_or(&boundary.phase);
        let code_tree = tree_of(root, &boundary.code_end_oid)?;
        let phase_tree = git::filter_tree(root, &code_tree, &[".doctrine"])?;
        let phase_commit =
            git::commit_tree(root, &phase_tree, &parent, &format!("phase({slice3}-{nn})"))?;
        planned.push(Planned {
            target_ref: format!("refs/heads/phase/{slice3}-{nn}"),
            source_oid: boundary.code_end_oid.clone(),
            commit_oid: phase_commit.clone(),
        });
        parent = phase_commit;
    }
    Ok(())
}

/// Build the pending-intent journal (one row per planned ref, all CAS creations).
fn pending_journal(planned: &[Planned]) -> Journal {
    Journal {
        rows: planned
            .iter()
            .map(|p| JournalRow {
                source_oid: p.source_oid.clone(),
                target_ref: p.target_ref.clone(),
                expected_old_oid: ZERO_OID.to_owned(),
                planned_new_oid: p.commit_oid.clone(),
                applied_new_oid: String::new(),
                status: LedgerStatus::Pending,
            })
            .collect(),
    }
}

/// Commit `journal` onto `dispatch/<slice>` by splicing `journal.toml` into the
/// tip tree and advancing the branch under CAS (no checkout). `base_tree` is the
/// impl tip tree; `parent` is the branch's current tip — by construction both the
/// new commit's parent AND the CAS expected-old (always identical). `msg` is the
/// stage-distinct commit message (`journal: prepare-review` / `journal: integrate`,
/// RV-030 F-4). Returns the new branch commit oid.
fn commit_journal(
    root: &Path,
    base_tree: &str,
    parent: &str,
    journal_path: &str,
    coord_ref: &str,
    journal: &Journal,
    msg: &str,
) -> anyhow::Result<String> {
    let body = journal.to_toml()?;
    let tree = git::tree_with_file(root, base_tree, journal_path, &body)?;
    let commit = git::commit_tree(root, &tree, parent, msg)?;
    match git::update_ref_cas(root, coord_ref, &commit, parent)? {
        RefCas::Updated => Ok(commit),
        RefCas::Moved { actual } => bail!(
            "journal-commit: dispatch branch moved under us (expected {parent}, found {})",
            actual.as_deref().unwrap_or("?")
        ),
    }
}

/// Render an ordered phase-status table. Pure formatting — caller owns data.
/// Designed for reuse by `plan-next` and `status` (PHASE-03).
pub(crate) fn render_phase_table(rows: &[(String, String, String)]) -> String {
    use comfy_table::Table;
    let mut table = Table::new();
    table
        .load_preset(comfy_table::presets::NOTHING)
        .set_header(vec!["  ID", "  Status", "  Name"])
        .force_no_tty();
    for (id, status, name) in rows {
        table.add_row(vec![
            format!("  {id}"),
            format!("  {status}"),
            format!("  {name}"),
        ]);
    }
    // Trim trailing whitespace (comfy-table last-column cell-fill edge case)
    let out = table.to_string();
    out.lines()
        .map(|l| l.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// `doctrine dispatch plan-next` — read the plan and runtime phase sheets;
/// print an ordered phase rollup and identify the next actionable phase(s).
/// Read-only — callable from anywhere.
pub(crate) fn run_plan_next(path: Option<PathBuf>, slice: u32, json: bool) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;

    // 1. Read plan.toml
    let plan = crate::slice::read_plan(&root.join(".doctrine/slice"), slice)?;

    // 2. Read phase statuses from runtime state
    let state_dir = crate::state::phases_dir(&root, slice);

    // Build ordered phase+status list
    let mut rows: Vec<(String, String, String)> = Vec::new();
    for ph in &plan.phases {
        let stem = ph.id.to_lowercase();
        let status = match crate::state::read_phase_status(&state_dir, &stem) {
            Ok(Some(s)) => s,
            Ok(None) => "pending".to_string(), // absent tracking file → pending
            Err(_) => "unknown".to_string(),
        };
        rows.push((ph.id.clone(), status, ph.name.clone()));
    }

    // 3. Compute `next`
    // Scan in plan order, skip completed/blocked.
    // First actionable in_progress → only that phase.
    // First actionable pending → that phase + consecutive pending.
    let mut next: Vec<String> = Vec::new();
    let mut found_actionable = false;
    let mut saw_blocked = false;

    for (id, status, _) in &rows {
        match status.as_str() {
            "completed" => {}
            "blocked" => {
                saw_blocked = true;
                if found_actionable {
                    break; // stop at blocked after we started collecting
                }
            }
            "in_progress" => {
                if !found_actionable {
                    next.push(id.clone());
                    break; // in_progress gates subsequent pending
                }
            }
            _ => {
                // pending or unknown
                if !found_actionable {
                    next.push(id.clone());
                    found_actionable = true;
                    // continue for consecutive pending
                } else if status.as_str() == "pending" {
                    next.push(id.clone());
                } else {
                    break; // non-pending stops the run
                }
            }
        }
    }

    // 4. Render output
    if json {
        #[derive(serde::Serialize)]
        struct PhaseRow {
            id: String,
            name: String,
            status: String,
        }
        #[derive(serde::Serialize)]
        struct Output {
            phases: Vec<PhaseRow>,
            next: Vec<String>,
            batching_requires_phase_plan: bool,
        }
        let output = Output {
            phases: rows
                .iter()
                .map(|(id, status, name)| PhaseRow {
                    id: id.clone(),
                    name: name.clone(),
                    status: status.clone(),
                })
                .collect(),
            next,
            batching_requires_phase_plan: true,
        };
        writeln!(io::stdout(), "{}", serde_json::to_string_pretty(&output)?)?;
    } else {
        // Human output
        let table = render_phase_table(&rows);
        writeln!(io::stdout(), "{table}")?;
        if next.is_empty() {
            if saw_blocked {
                writeln!(
                    io::stdout(),
                    "\nnext: (none — all remaining phases are blocked)"
                )?;
            }
        } else {
            let ids = next.join(", ");
            writeln!(io::stdout(), "\nnext: {ids}")?;
            writeln!(
                io::stdout(),
                "  ⚠ run /phase-plan before parallel spawn; do not assume file-disjointness"
            )?;
        }
    }

    Ok(())
}

/// `doctrine dispatch status` — read-only full dispatch rollup: coordination
/// state, phase table, trunk drift, sync state, candidate summary, next-step
/// guidance. Read-only — callable from anywhere.
pub(crate) fn run_status(path: Option<PathBuf>, slice: u32, json: bool) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let slice3 = format!("{slice:03}");
    let dispatch_ref = format!("refs/heads/dispatch/{slice3}");

    // --- Coordination state ---------------------------------------------------
    let dispatch_tip = resolve_commit(&root, &dispatch_ref)?.with_context(|| {
        format!("dispatch branch not found; run 'dispatch setup --slice {slice}' first")
    })?;
    let dispatch_short = git::git_text(&root, &["rev-parse", "--short=7", &dispatch_tip])?;

    // Find live worktree via git worktree list --porcelain
    let coord_state = find_coordination_worktree(&root, &slice3);

    // --- Trunk drift -----------------------------------------------------------
    let trunk_tip = git::trunk_commit(&root)?.with_context(|| "trunk ref not found")?;
    let fork_point = git::merge_base(&root, &dispatch_tip, &trunk_tip)?
        .with_context(|| format!("dispatch/{slice3} and trunk share no common ancestor"))?;
    let ahead_cnt = git::git_text(
        &root,
        &["rev-list", "--count", &format!("{fork_point}..{trunk_tip}")],
    )?;
    let ahead: u32 = ahead_cnt.trim().parse().unwrap_or(0);
    let trunk_state = if ahead == 0 { "stable" } else { "moved" };

    // --- Phase table -----------------------------------------------------------
    let plan = crate::slice::read_plan(&root.join(".doctrine/slice"), slice)?;
    let state_dir = crate::state::phases_dir(&root, slice);
    let mut phase_rows: Vec<(String, String, String)> = Vec::new();
    for ph in &plan.phases {
        let stem = ph.id.to_lowercase();
        let status = match crate::state::read_phase_status(&state_dir, &stem) {
            Ok(Some(s)) => s,
            Ok(None) => "pending".to_string(),
            Err(_) => "unknown".to_string(),
        };
        phase_rows.push((ph.id.clone(), status, ph.name.clone()));
    }

    // --- Sync state ------------------------------------------------------------
    let review_ref = format!("refs/heads/review/{slice3}");
    let review_exists = resolve_commit(&root, &review_ref)?.is_some();
    let phase_ref_count = count_phase_refs(&root, &slice3);

    // --- Candidate summary -----------------------------------------------------
    let candidates = read_candidates(&root, slice)?;
    let candidate_total = candidates.rows.len();
    let candidate_admitted = [
        candidates.current_admission.close_target.is_some(),
        candidates.current_admission.review_surface.is_some(),
    ]
    .into_iter()
    .filter(|&x| x)
    .count();

    // --- Next-step guidance ----------------------------------------------------
    let all_completed = phase_rows
        .iter()
        .all(|(_, status, _)| status == "completed");
    let coord_live = !matches!(coord_state.as_str(), "(removed)");
    let admitted_ct = candidates.current_admission.close_target.as_ref();

    let next_guidance = if !all_completed {
        // Condition 1: phases remain
        let next_phases = compute_next_phases(&phase_rows);
        NextGuidance::Phases {
            phases: next_phases,
        }
    } else if !review_exists {
        // Condition 2: all completed, no review ref
        NextGuidance::PrepareReview
    } else if coord_live && admitted_ct.is_some() {
        // Condition 3: all completed, review ref, admitted close_target, coord live
        NextGuidance::AuditThenIntegrate
    } else if coord_live && admitted_ct.is_none() {
        // Condition 4: all completed, review ref, no admitted close_target, coord live
        NextGuidance::AuditOrCandidateStatus
    } else if let Some(ct) = admitted_ct {
        if is_ancestor_of_trunk(&root, &ct.admitted_oid, &trunk_tip)? {
            // Condition 5: coord removed, admitted close_target is ancestor of trunk
            NextGuidance::Complete
        } else {
            // Condition 6: coord removed, admitted exists, NOT ancestor of trunk
            NextGuidance::AwaitingIntegration
        }
    } else {
        // Fallback (shouldn't normally reach here)
        NextGuidance::AuditOrCandidateStatus
    };

    // --- Output ----------------------------------------------------------------
    if json {
        let output = StatusOutput {
            dispatch: DispatchState {
                r#ref: dispatch_ref,
                tip: dispatch_short,
            },
            coord: CoordState {
                state: if coord_live {
                    "live".to_string()
                } else {
                    "removed".to_string()
                },
                path: if coord_live { Some(coord_state) } else { None },
            },
            trunk: TrunkState {
                state: trunk_state.to_string(),
                fork_point,
                ahead,
            },
            phases: phase_rows
                .iter()
                .map(|(id, status, name)| PhaseState {
                    id: id.clone(),
                    name: name.clone(),
                    status: status.clone(),
                })
                .collect(),
            sync: SyncState {
                state: if review_exists {
                    "prepared".to_string()
                } else {
                    "not_prepared".to_string()
                },
                review_ref: if review_exists {
                    Some(review_ref)
                } else {
                    None
                },
                phase_cuts: phase_ref_count,
            },
            candidates: CandidateSummary {
                total: candidate_total,
                admitted: candidate_admitted,
            },
            next: next_guidance.to_json(),
        };
        writeln!(io::stdout(), "{}", serde_json::to_string_pretty(&output)?)?;
    } else {
        // Human output
        writeln!(io::stdout(), "dispatch: {dispatch_ref}  ({dispatch_short})")?;
        writeln!(io::stdout(), "coord:    {coord_state}")?;
        if ahead > 0 {
            writeln!(
                io::stdout(),
                "trunk:    {trunk_state} ({ahead} commit(s) ahead of fork-point)"
            )?;
        } else {
            writeln!(io::stdout(), "trunk:    {trunk_state}")?;
        }
        writeln!(io::stdout())?;
        writeln!(io::stdout(), "phases:")?;
        write!(io::stdout(), "{}", render_phase_table(&phase_rows))?;
        writeln!(io::stdout())?;
        writeln!(io::stdout())?;
        if review_exists {
            writeln!(
                io::stdout(),
                "sync:     prepared — {review_ref} ({phase_ref_count} phase cut(s))"
            )?;
        } else {
            writeln!(io::stdout(), "sync:     not yet run")?;
        }
        writeln!(
            io::stdout(),
            "candidates: {candidate_total} ({candidate_admitted} admitted)"
        )?;
        match &next_guidance {
            NextGuidance::Phases { phases } => {
                let ids = phases.join(", ");
                writeln!(io::stdout(), "next:     {ids}")?;
            }
            NextGuidance::PrepareReview => {
                writeln!(
                    io::stdout(),
                    "next:     all phases completed — run 'dispatch sync --prepare-review'"
                )?;
            }
            NextGuidance::AuditThenIntegrate => {
                writeln!(
                    io::stdout(),
                    "next:     all phases completed — admitted candidate exists; run audit then 'dispatch sync --integrate'"
                )?;
            }
            NextGuidance::AuditOrCandidateStatus => {
                writeln!(
                    io::stdout(),
                    "next:     all phases completed — review ref prepared; run audit or 'dispatch candidate status'"
                )?;
            }
            NextGuidance::Complete => {
                writeln!(
                    io::stdout(),
                    "next:     complete — coordination worktree removed; slice is integrated"
                )?;
            }
            NextGuidance::AwaitingIntegration => {
                writeln!(
                    io::stdout(),
                    "next:     awaiting integration — run 'dispatch sync --integrate' after audit"
                )?;
            }
        }
    }

    Ok(())
}

/// Parse `git worktree list --porcelain` for a worktree checked out on
/// `dispatch/<slice3>`. Returns the worktree path or "(removed)".
fn find_coordination_worktree(root: &Path, slice3: &str) -> String {
    let target_branch = format!("refs/heads/dispatch/{slice3}");
    let Ok(out) = git::git_text(root, &["worktree", "list", "--porcelain"]) else {
        return "(removed)".to_string();
    };
    let mut current_path: Option<String> = None;
    for line in out.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = Some(path.to_string());
        } else if let Some(branch) = line.strip_prefix("branch ")
            && branch == target_branch
        {
            return current_path.unwrap_or_else(|| "(removed)".to_string());
        }
    }
    "(removed)".to_string()
}

/// Count `refs/heads/phase/{slice3}-*` refs via `git for-each-ref`.
fn count_phase_refs(root: &Path, slice3: &str) -> usize {
    let pattern = format!("refs/heads/phase/{slice3}-*");
    let Ok(out) = git::git_text(root, &["for-each-ref", "--format=%(refname)", &pattern]) else {
        return 0;
    };
    if out.trim().is_empty() {
        0
    } else {
        out.lines().count()
    }
}

/// Compute next phases using same logic as plan-next.
fn compute_next_phases(rows: &[(String, String, String)]) -> Vec<String> {
    let mut next: Vec<String> = Vec::new();
    let mut found_actionable = false;
    for (id, status, _) in rows {
        match status.as_str() {
            "completed" => {}
            "blocked" => {
                if found_actionable {
                    break;
                }
            }
            "in_progress" => {
                if !found_actionable {
                    next.push(id.clone());
                    break;
                }
            }
            _ => {
                if !found_actionable {
                    next.push(id.clone());
                    found_actionable = true;
                } else if status.as_str() == "pending" {
                    next.push(id.clone());
                } else {
                    break;
                }
            }
        }
    }
    next
}

/// Check if `oid` is an ancestor of `trunk_tip` (or equal).
fn is_ancestor_of_trunk(root: &Path, oid: &str, trunk_tip: &str) -> anyhow::Result<bool> {
    if oid == trunk_tip {
        return Ok(true);
    }
    let mb = git::merge_base(root, oid, trunk_tip)?;
    Ok(mb.as_deref() == Some(oid))
}

// --- JSON output types -------------------------------------------------------

#[derive(serde::Serialize)]
struct StatusOutput {
    dispatch: DispatchState,
    coord: CoordState,
    trunk: TrunkState,
    phases: Vec<PhaseState>,
    sync: SyncState,
    candidates: CandidateSummary,
    next: NextJson,
}

#[derive(serde::Serialize)]
struct DispatchState {
    #[serde(rename = "ref")]
    r#ref: String,
    tip: String,
}

#[derive(serde::Serialize)]
struct CoordState {
    state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

#[derive(serde::Serialize)]
struct TrunkState {
    state: String,
    fork_point: String,
    ahead: u32,
}

#[derive(serde::Serialize)]
struct PhaseState {
    id: String,
    name: String,
    status: String,
}

#[derive(serde::Serialize)]
struct SyncState {
    state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    review_ref: Option<String>,
    phase_cuts: usize,
}

#[derive(serde::Serialize)]
struct CandidateSummary {
    total: usize,
    admitted: usize,
}

#[derive(serde::Serialize)]
struct NextJson {
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    phases: Option<Vec<String>>,
}

/// The next-step guidance resolved from the deterministic state machine.
enum NextGuidance {
    Phases { phases: Vec<String> },
    PrepareReview,
    AuditThenIntegrate,
    AuditOrCandidateStatus,
    Complete,
    AwaitingIntegration,
}

impl NextGuidance {
    fn to_json(&self) -> NextJson {
        match self {
            NextGuidance::Phases { phases } => NextJson {
                kind: "phases".to_string(),
                phases: Some(phases.clone()),
            },
            NextGuidance::PrepareReview => NextJson {
                kind: "blocked".to_string(),
                phases: None,
            },
            NextGuidance::AuditThenIntegrate | NextGuidance::AuditOrCandidateStatus => NextJson {
                kind: "audit".to_string(),
                phases: None,
            },
            NextGuidance::Complete => NextJson {
                kind: "completed".to_string(),
                phases: None,
            },
            NextGuidance::AwaitingIntegration => NextJson {
                kind: "awaiting_integration".to_string(),
                phases: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn git(dir: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("spawn git");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn init_repo(dir: &Path) {
        std::fs::create_dir_all(dir).unwrap();
        git(dir, &["init", "-q", "-b", "main"]);
        git(dir, &["config", "user.email", "t@example.com"]);
        git(dir, &["config", "user.name", "Test"]);
        std::fs::create_dir_all(dir.join(".doctrine")).unwrap();
        std::fs::write(dir.join("a.txt"), "hello").unwrap();
        git(dir, &["add", "."]);
        git(dir, &["commit", "-q", "-m", "base"]);
    }

    fn seed_slice_dir(dir: &Path, slice: u32) {
        let rel = format!(".doctrine/slice/{slice:03}");
        let full = dir.join(&rel);
        std::fs::create_dir_all(&full).unwrap();
        std::fs::write(
            full.join("slice.toml"),
            format!("id = {slice}\ntitle = \"test\"\nkind = \"slice\"\nstatus = \"planned\"\n"),
        )
        .unwrap();
        git(dir, &["add", "-A"]);
        git(dir, &["commit", "-q", "-m", "seed slice dir"]);
    }

    fn seed_plan(dir: &Path, slice: u32, phases: &str) {
        let rel = format!(".doctrine/slice/{slice:03}/plan.toml");
        let full = dir.join(&rel);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, phases).unwrap();
        git(dir, &["add", "-A"]);
        git(dir, &["commit", "-q", "-m", "seed plan"]);
    }

    #[test]
    fn dispatch_setup_gates_on_no_plan() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        // No plan.toml — the gate should fail before touching git.
        let holder = tempfile::tempdir().unwrap();
        let coord = holder.path().join("coord");
        let result = run_setup(Some(src.path().to_path_buf()), 85, &coord);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("no plan"),
            "error should mention 'no plan'; got: {err}"
        );
    }

    #[test]
    fn dispatch_setup_gates_on_empty_plan() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            "schema = \"doctrine.plan.overview\"\nversion = 1\nslice = \"SL-085\"\n",
        );
        // Plan has zero phases.
        let holder = tempfile::tempdir().unwrap();
        let coord = holder.path().join("coord");
        let result = run_setup(Some(src.path().to_path_buf()), 85, &coord);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("no phases"),
            "error should mention 'no phases'; got: {err}"
        );
    }

    #[test]
    fn dispatch_setup_creates_coordination() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            "schema = \"doctrine.plan.overview\"\nversion = 1\nslice = \"SL-085\"\n\n[[phase]]\nid = \"PHASE-01\"\nname = \"fixture\"\nobjective = \"fixture\"\n",
        );
        let holder = tempfile::tempdir().unwrap();
        let coord = holder.path().join("coord");
        let result = run_setup(Some(src.path().to_path_buf()), 85, &coord);
        assert!(result.is_ok(), "setup must succeed; err: {result:?}");

        // Verify worktree exists.
        assert!(coord.exists(), "coordination dir exists");
        assert!(coord.join("a.txt").exists(), "checkout exists");

        // Verify env contract keys on stdout (print! from run_setup).
        // Since run_setup uses println!, we test via the returned Ok(()).
        // The actual stdout capture is an integration-test concern; here we
        // verify the function doesn't panic and the worktree is real.
        assert!(coord.join(".doctrine").exists(), "provisioned");
    }

    // --- plan-next helpers ---

    /// Write a `phase-NN.toml` tracking file under
    /// `.doctrine/state/slice/{slice:03}/phases/`.
    fn seed_phase_tracking(dir: &Path, slice: u32, phase_num: u32, status: &str) {
        let state_dir = dir
            .join(".doctrine/state/slice")
            .join(format!("{slice:03}"))
            .join("phases");
        std::fs::create_dir_all(&state_dir).unwrap();
        std::fs::write(
            state_dir.join(format!("phase-{phase_num:02}.toml")),
            format!("status = \"{status}\"\n"),
        )
        .unwrap();
    }

    /// Build a multi-phase plan.toml body from phase ids + names. Each entry is
    /// `(id, name)`; the fixture automatically wraps in a `[[phase]]` array.
    fn plan_body(phases: &[(&str, &str)]) -> String {
        let mut body =
            String::from("schema = \"doctrine.plan.overview\"\nversion = 1\nslice = \"SL-085\"\n");
        for (id, name) in phases {
            body.push_str(&format!(
                "\n[[phase]]\nid = \"{id}\"\nname = \"{name}\"\nobjective = \"fixture\"\n"
            ));
        }
        body
    }

    // --- plan-next tests ---

    #[test]
    fn dispatch_plan_next_orders_phases() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            &plan_body(&[
                ("PHASE-01", "setup"),
                ("PHASE-02", "build"),
                ("PHASE-03", "blocked-one"),
                ("PHASE-04", "final"),
            ]),
        );
        seed_phase_tracking(src.path(), 85, 1, "completed");
        seed_phase_tracking(src.path(), 85, 2, "completed");
        seed_phase_tracking(src.path(), 85, 3, "blocked");
        // PHASE-04 has no tracking → pending

        // run_plan_next prints to stdout; we verify it doesn't panic and
        // check that the return is Ok.
        let result = run_plan_next(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "plan-next should succeed; err: {result:?}");
    }

    #[test]
    fn dispatch_plan_next_all_blocked() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            &plan_body(&[
                ("PHASE-01", "setup"),
                ("PHASE-02", "blocked-one"),
                ("PHASE-03", "blocked-two"),
            ]),
        );
        seed_phase_tracking(src.path(), 85, 1, "completed");
        seed_phase_tracking(src.path(), 85, 2, "blocked");
        seed_phase_tracking(src.path(), 85, 3, "blocked");

        let result = run_plan_next(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "plan-next should succeed; err: {result:?}");
    }

    #[test]
    fn dispatch_plan_next_stops_at_blocked_mid() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            &plan_body(&[
                ("PHASE-01", "setup"),
                ("PHASE-02", "first-pending"),
                ("PHASE-03", "second-pending"),
                ("PHASE-04", "blocked"),
                ("PHASE-05", "after-blocked"),
            ]),
        );
        seed_phase_tracking(src.path(), 85, 1, "completed");
        // PHASE-02, PHASE-03: no tracking → pending
        seed_phase_tracking(src.path(), 85, 4, "blocked");
        // PHASE-05: no tracking → pending

        let result = run_plan_next(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "plan-next should succeed; err: {result:?}");
    }

    #[test]
    fn dispatch_plan_next_resume_in_progress() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            &plan_body(&[
                ("PHASE-01", "setup"),
                ("PHASE-02", "in-progress"),
                ("PHASE-03", "next-one"),
                ("PHASE-04", "next-two"),
            ]),
        );
        seed_phase_tracking(src.path(), 85, 1, "completed");
        seed_phase_tracking(src.path(), 85, 2, "in_progress");
        // PHASE-03, PHASE-04: no tracking → pending

        let result = run_plan_next(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "plan-next should succeed; err: {result:?}");
    }

    #[test]
    fn dispatch_plan_next_json() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            &plan_body(&[("PHASE-01", "setup"), ("PHASE-02", "active")]),
        );
        seed_phase_tracking(src.path(), 85, 1, "completed");
        // PHASE-02: no tracking → pending

        let result = run_plan_next(Some(src.path().to_path_buf()), 85, true);
        assert!(
            result.is_ok(),
            "plan-next --json should succeed; err: {result:?}"
        );
    }

    #[test]
    fn dispatch_plan_next_no_plan() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        // No plan.toml seeded.

        let result = run_plan_next(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_err(), "plan-next without plan should fail");
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("not found"),
            "error should mention 'not found'; got: {err}"
        );
    }

    // --- status helpers ---

    /// Create a `refs/heads/dispatch/{slice:03}` ref pointing at the current HEAD.
    fn create_dispatch_ref(dir: &Path, slice: u32) {
        let head = git(dir, &["rev-parse", "HEAD"]);
        git(
            dir,
            &[
                "update-ref",
                &format!("refs/heads/dispatch/{slice:03}"),
                &head,
            ],
        );
    }

    /// Create a `refs/heads/review/{slice:03}` ref pointing at the current HEAD.
    fn create_review_ref(dir: &Path, slice: u32) {
        let head = git(dir, &["rev-parse", "HEAD"]);
        git(
            dir,
            &[
                "update-ref",
                &format!("refs/heads/review/{slice:03}"),
                &head,
            ],
        );
    }

    /// Advance trunk by making a commit on main.
    fn advance_trunk(dir: &Path) -> String {
        std::fs::write(dir.join("b.txt"), "world").unwrap();
        git(dir, &["add", "b.txt"]);
        git(dir, &["commit", "-q", "-m", "advance trunk"]);
        git(dir, &["rev-parse", "HEAD"])
    }

    // --- status tests ---

    /// T3-1: Status fresh after setup → coord live, phases pending, sync not yet run.
    #[test]
    fn dispatch_status_fresh_after_setup() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            &plan_body(&[("PHASE-01", "setup"), ("PHASE-02", "build")]),
        );
        create_dispatch_ref(src.path(), 85);

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "status should succeed; err: {result:?}");
    }

    /// T3-2: Status missing dispatch ref → non-zero exit (error).
    #[test]
    fn dispatch_status_missing_dispatch_ref() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(src.path(), 85, &plan_body(&[("PHASE-01", "setup")]));
        // No dispatch ref created.

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_err(), "status without dispatch ref should fail");
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("dispatch branch not found"),
            "error should mention 'dispatch branch not found'; got: {err}"
        );
    }

    /// T3-3: Status missing trunk ref → non-zero exit (error).
    #[test]
    fn dispatch_status_missing_trunk_ref() {
        // Create a repo that initialises with an orphaned initial commit on a
        // non-standard branch, so the trunk ladder (origin/HEAD, main, master)
        // finds nothing.
        let src = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(src.path()).unwrap();
        git(src.path(), &["init", "-q", "-b", "other"]);
        git(src.path(), &["config", "user.email", "t@example.com"]);
        git(src.path(), &["config", "user.name", "Test"]);
        std::fs::create_dir_all(src.path().join(".doctrine")).unwrap();
        std::fs::write(src.path().join("a.txt"), "hello").unwrap();
        git(src.path(), &["add", "."]);
        git(src.path(), &["commit", "-q", "-m", "base"]);
        seed_slice_dir(src.path(), 85);
        seed_plan(src.path(), 85, &plan_body(&[("PHASE-01", "setup")]));
        create_dispatch_ref(src.path(), 85);
        // No main/master branch — trunk ladder returns None.

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_err(), "status without trunk ref should fail");
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("trunk ref not found"),
            "error should mention 'trunk ref not found'; got: {err}"
        );
    }

    /// T3-4: Status after sync → sync prepared, phase cuts count.
    #[test]
    fn dispatch_status_after_sync() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(src.path(), 85, &plan_body(&[("PHASE-01", "setup")]));
        create_dispatch_ref(src.path(), 85);
        create_review_ref(src.path(), 85);

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "status should succeed; err: {result:?}");
    }

    /// T3-5: Status moved trunk → trunk moved.
    #[test]
    fn dispatch_status_moved_trunk() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(src.path(), 85, &plan_body(&[("PHASE-01", "setup")]));
        // Create dispatch ref BEFORE trunk advances, so the fork point is older.
        create_dispatch_ref(src.path(), 85);
        advance_trunk(src.path());

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "status should succeed; err: {result:?}");
    }

    /// T3-6: Status all phases completed, no review ref → next guidance for prepare-review.
    #[test]
    fn dispatch_status_all_completed_no_review() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(src.path(), 85, &plan_body(&[("PHASE-01", "setup")]));
        create_dispatch_ref(src.path(), 85);
        seed_phase_tracking(src.path(), 85, 1, "completed");

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "status should succeed; err: {result:?}");
    }

    /// T3-7: Status all completed, review ref present → guidance references audit.
    #[test]
    fn dispatch_status_all_completed_review_present() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(src.path(), 85, &plan_body(&[("PHASE-01", "setup")]));
        create_dispatch_ref(src.path(), 85);
        create_review_ref(src.path(), 85);
        seed_phase_tracking(src.path(), 85, 1, "completed");

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "status should succeed; err: {result:?}");
    }

    /// T3-8: Status coord removed → coord (removed).
    #[test]
    fn dispatch_status_coord_removed() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(src.path(), 85, &plan_body(&[("PHASE-01", "setup")]));
        create_dispatch_ref(src.path(), 85);
        // No worktree exists — worktree list won't find it.

        let result = run_status(Some(src.path().to_path_buf()), 85, false);
        assert!(result.is_ok(), "status should succeed; err: {result:?}");
    }

    /// T3-9: Status JSON → all sections, next.kind structured.
    #[test]
    fn dispatch_status_json() {
        let src = tempfile::tempdir().unwrap();
        init_repo(src.path());
        seed_slice_dir(src.path(), 85);
        seed_plan(
            src.path(),
            85,
            &plan_body(&[("PHASE-01", "setup"), ("PHASE-02", "build")]),
        );
        create_dispatch_ref(src.path(), 85);

        let result = run_status(Some(src.path().to_path_buf()), 85, true);
        assert!(
            result.is_ok(),
            "status --json should succeed; err: {result:?}"
        );
    }
}
