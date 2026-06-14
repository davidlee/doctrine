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

use crate::git::{self, RefCas};
use crate::ledger::{Boundaries, Journal, JournalRow, LedgerStatus, Orthogonal};
use crate::root;

/// The all-zero oid — the CAS `expected_old` for a ref *creation* (`update-ref`
/// refuses if the ref already exists).
const ZERO_OID: &str = "0000000000000000000000000000000000000000";

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
    let trunk_base = git::trunk_commit(root)?
        .context("prepare-review: no trunk ref resolves — a trunk base is required")?;

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
        &tip,
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
        &journal_commit,
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
/// impl tip tree; `parent`/`expected_old` are the branch's current tip. Returns
/// the new branch commit oid.
fn commit_journal(
    root: &Path,
    base_tree: &str,
    parent: &str,
    journal_path: &str,
    coord_ref: &str,
    journal: &Journal,
    expected_old: &str,
) -> anyhow::Result<String> {
    let body = journal.to_toml()?;
    let tree = git::tree_with_file(root, base_tree, journal_path, &body)?;
    let commit = git::commit_tree(root, &tree, parent, "journal: prepare-review")?;
    match git::update_ref_cas(root, coord_ref, &commit, expected_old)? {
        RefCas::Updated => Ok(commit),
        RefCas::Moved { actual } => bail!(
            "prepare-review: dispatch branch moved under us (expected {expected_old}, found {})",
            actual.as_deref().unwrap_or("?")
        ),
    }
}
