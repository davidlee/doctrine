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

use crate::git::{self, RefCas, ReplayOutcome, ZERO_OID};
use crate::ledger::{Boundaries, Journal, JournalRow, LedgerStatus, Orthogonal};
use crate::root;

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

    // --- plan opt-in projection rows (idempotent: skip a target already
    //     journaled by a prior/crashed run — its recorded intent is replayed) ---
    let fresh = |j: &Journal, target: &str| !j.rows.iter().any(|r| r.target_ref == target);
    if let Some(trunk_ref) = trunk.filter(|t| fresh(&journal, t)) {
        journal
            .rows
            .push(plan_trunk_row(root, &slice3, &journal, trunk_ref)?);
    }
    if let Some(edge_ref) = edge.filter(|e| fresh(&journal, e)) {
        journal.rows.push(plan_edge_row(root, &slice3, edge_ref)?);
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
