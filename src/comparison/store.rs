// SPDX-License-Identifier: GPL-3.0-only
//! `comparison::store` — the one impure seam (SL-213 design §1, the
//! `coverage_store` precedent): [`load_sessions`] scans `.doctrine/comparisons/
//! *.toml`, and [`load_pipeline`] composes the full pure pipeline (`resolve` →
//! `compile` → `project`) behind the single call the priority build shell
//! makes, returning every PHASE-06 surface's derived accessors alongside the
//! final `Projection`.
//!
//! Depends only on `wire` (this module's siblings) and `fs` (design §1 module
//! table) — the `StatusMap`/`AnchorMap` inputs are supplied by the caller
//! rather than built here: they require `priority::partition::status_class`
//! and `catalog::scan::ScannedEntity`, both above `comparison` in the ADR-001
//! layering (`priority` → `relation_graph`/`catalog` → … ; `priority` also
//! consumes `comparison`). Building them here would put `comparison` in a
//! two-way dependency with `priority` — a layering cycle. `priority::graph`
//! (which already depends on both `priority::partition` and `comparison`)
//! builds them and passes the finished maps in.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Context;

use super::compile::{self, AnchorMap, ClassId, ConstraintSet, QuarantinePolicy, RaterCounts};
use super::project::{self, Projection, ProjectionCfg};
use super::resolve::{
    self, MalformedSupersession, ResolutionStatus, RowState, StatusMap, rater_key,
};
use super::{COMPARISONS_DIR, ComparisonSession, DOMAIN_PRIORITY, Judgement, Response};

/// Scan `.doctrine/comparisons/*.toml` and parse every session (moved
/// verbatim from `commands/compare.rs`, SL-213 PHASE-05). A missing directory
/// yields an empty listing (not an error — the ledger is created lazily on
/// first capture). Files are read in filename order for determinism; row
/// ordering is imposed later by `resolve`'s total key, so file order is
/// immaterial to the pipeline.
pub(crate) fn load_sessions(root: &Path) -> anyhow::Result<Vec<ComparisonSession>> {
    let dir = root.join(".doctrine").join(COMPARISONS_DIR);
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => {
            return Err(
                anyhow::Error::new(e).context(format!("read comparisons dir {}", dir.display()))
            );
        }
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    paths.sort();

    let mut sessions = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("read comparison session {}", path.display()))?;
        let session = super::parse(&text)
            .with_context(|| format!("parse comparison session {}", path.display()))?;
        sessions.push(session);
    }
    Ok(sessions)
}

/// One row's OWNED render-ready summary + its joined [`RowState`] (design §4
/// S2). Owned because `Judgement` has no `Clone` (PHASE-01) — nothing borrows
/// past [`load_pipeline`]'s return. In `resolve`'s total display order
/// (`(date, session_uid, seq)`).
pub(crate) struct RowSummary {
    pub uid: String,
    pub a: String,
    pub b: String,
    pub response: Response,
    pub frame: String,
    pub rater_token: &'static str,
    pub by: Option<String>,
    pub note: Option<String>,
    pub date: String,
    pub state: RowState,
}

/// The composed pipeline artifacts PHASE-06's surfaces need beyond the final
/// [`Projection`] (design D12 — an explicit derived-accessor bundle, never a
/// `ConstraintSet` reshape): every row's joined [`RowState`] (`compare list`),
/// the compiled [`ConstraintSet`] itself (bounds + quarantine ledger for
/// `explain`/`findings`), the per-class rater split (`explain`'s T7
/// disclosure), the `MalformedSupersession` finding stream, and the raw
/// priority-domain row count (the inert-priority disclosure, `explain` only —
/// design §4 S4, NOT a finding). `Resolution::unknown_supersedes` (R2's
/// dangling-target load warning) has no PHASE-06 surface consumer yet and
/// stays un-plumbed here — a future phase's hook, not this one's scope.
pub(crate) struct Pipeline {
    pub rows: Vec<RowSummary>,
    pub constraint_set: ConstraintSet,
    pub projection: Projection,
    pub constraining_by_class: BTreeMap<ClassId, RaterCounts>,
    pub malformed: Vec<MalformedSupersession>,
    pub priority_domain_count: usize,
}

/// Compose the full pipeline (design §1 integration point): [`load_sessions`]
/// → [`pipeline_from_sessions`]. Empty sessions (no comparisons dir, or an
/// empty one) short-circuit before any resolve/compile/project call — the
/// behaviour-preservation gate: every existing priority suite runs with zero
/// comparison sessions on disk, so this must cost nothing beyond the one
/// directory read.
pub(crate) fn load_pipeline(
    root: &Path,
    statuses: &StatusMap,
    anchors: &AnchorMap,
    cfg: &ProjectionCfg,
) -> anyhow::Result<Pipeline> {
    let sessions = load_sessions(root)?;
    pipeline_from_sessions(&sessions, statuses, anchors, cfg)
}

/// The disk-FREE inner half of [`load_pipeline`]: `resolve` → `compile` →
/// `project`, PLUS the PHASE-06 derived accessors every surface needs, over
/// an ALREADY-LOADED session slice. Split out so tests compose sessions in
/// memory without a filesystem round-trip (the pure/impure split the
/// project's conventions ask for).
pub(crate) fn pipeline_from_sessions(
    sessions: &[ComparisonSession],
    statuses: &StatusMap,
    anchors: &AnchorMap,
    cfg: &ProjectionCfg,
) -> anyhow::Result<Pipeline> {
    if sessions.is_empty() {
        return Ok(Pipeline {
            rows: Vec::new(),
            constraint_set: compile::compile(&[], anchors, QuarantinePolicy::Symmetric),
            projection: Projection::new(),
            constraining_by_class: BTreeMap::new(),
            malformed: Vec::new(),
            priority_domain_count: 0,
        });
    }

    let resolution = resolve::resolve(sessions, statuses)?;
    let active: Vec<&Judgement> = resolution
        .rows
        .iter()
        .filter(|(_, status)| matches!(status, ResolutionStatus::Active))
        .map(|(j, _)| *j)
        .collect();
    let cs = compile::compile(&active, anchors, QuarantinePolicy::Symmetric);
    let projection = project::project(&cs, cfg);
    let constraining_by_class = compile::constraining_counts_by_class(&cs, &active);
    let priority_domain_count = resolution
        .rows
        .iter()
        .filter(|(j, _)| j.domain == DOMAIN_PRIORITY)
        .count();

    let rows = resolution
        .rows
        .iter()
        .map(|(j, status)| {
            let compilation = matches!(status, ResolutionStatus::Active).then(|| cs.status_of(j));
            RowSummary {
                uid: j.uid.clone(),
                a: j.a.clone(),
                b: j.b.clone(),
                response: j.response,
                frame: j.frame.clone(),
                rater_token: rater_key(&j.rater),
                by: j.by.clone(),
                note: j.note.clone(),
                date: j.date.clone(),
                state: RowState::new(status.clone(), compilation),
            }
        })
        .collect();

    Ok(Pipeline {
        rows,
        constraint_set: cs,
        projection,
        constraining_by_class,
        malformed: resolution.malformed,
        priority_domain_count,
    })
}

#[cfg(test)]
mod tests {
    use super::{ProjectionCfg, StatusMap, load_pipeline, load_sessions};
    use crate::comparison::{
        COMPARISON_SCHEMA, COMPARISON_VERSION, ComparisonSession, DOMAIN_VALUE, FRAME_EQUAL_EFFORT,
        Judgement, RaterKind, Response, RowForm, SessionHeader,
    };

    const CFG: ProjectionCfg = ProjectionCfg {
        gauge_step: 0.25,
        default_value: 1.0,
    };

    fn write(root: &std::path::Path, rel: &str, body: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    fn judgement(uid: &str, a: &str, b: &str) -> Judgement {
        Judgement {
            uid: uid.to_string(),
            seq: 0,
            a: a.to_string(),
            b: b.to_string(),
            response: Response::PreferA,
            domain: DOMAIN_VALUE.to_string(),
            frame: FRAME_EQUAL_EFFORT.to_string(),
            form: RowForm::Order,
            magnitude: None,
            supersedes: None,
            lens: None,
            rater: RaterKind::Human,
            by: None,
            note: None,
            date: "2026-07-11".to_string(),
        }
    }

    fn session(uid: &str, date: &str, judgements: Vec<Judgement>) -> ComparisonSession {
        ComparisonSession {
            schema: COMPARISON_SCHEMA.to_string(),
            version: COMPARISON_VERSION,
            session: SessionHeader {
                uid: uid.to_string(),
                date: date.to_string(),
                audience: None,
            },
            judgements,
            tombstones: Vec::new(),
        }
    }

    // ---- VT-3: load_sessions ------------------------------------------------

    #[test]
    fn missing_comparisons_dir_is_an_empty_load() {
        let dir = tempfile::tempdir().unwrap();
        let sessions = load_sessions(dir.path()).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn empty_comparisons_dir_is_an_empty_load() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".doctrine/comparisons")).unwrap();
        let sessions = load_sessions(dir.path()).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn sessions_load_in_filename_order() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // Write in reverse-filename order; loaded sessions must come back sorted.
        write(
            root,
            ".doctrine/comparisons/2026-07-11-zzz.toml",
            &crate::comparison::to_toml(&session("last", "2026-07-11", vec![])).unwrap(),
        );
        write(
            root,
            ".doctrine/comparisons/2026-07-09-aaa.toml",
            &crate::comparison::to_toml(&session("first", "2026-07-09", vec![])).unwrap(),
        );
        write(
            root,
            ".doctrine/comparisons/2026-07-10-mmm.toml",
            &crate::comparison::to_toml(&session("middle", "2026-07-10", vec![])).unwrap(),
        );
        let sessions = load_sessions(root).unwrap();
        let uids: Vec<&str> = sessions.iter().map(|s| s.session.uid.as_str()).collect();
        assert_eq!(uids, vec!["first", "middle", "last"]);
    }

    #[test]
    fn non_toml_files_in_the_dir_are_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(root, ".doctrine/comparisons/README.md", "not a session\n");
        let sessions = load_sessions(root).unwrap();
        assert!(sessions.is_empty());
    }

    // ---- load_projection ----------------------------------------------------

    #[test]
    fn no_sessions_on_disk_is_an_empty_projection() {
        let dir = tempfile::tempdir().unwrap();
        let pipeline =
            load_pipeline(dir.path(), &StatusMap::new(), &Default::default(), &CFG).unwrap();
        assert!(pipeline.projection.is_empty());
        assert!(pipeline.rows.is_empty());
        assert_eq!(pipeline.priority_domain_count, 0);
    }

    #[test]
    fn pipeline_composes_through_to_a_projection() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(
            root,
            ".doctrine/comparisons/2026-07-11-s1.toml",
            &crate::comparison::to_toml(&session(
                "s1",
                "2026-07-11",
                vec![judgement("j1", "SL-100", "SL-200")],
            ))
            .unwrap(),
        );
        let pipeline = load_pipeline(root, &StatusMap::new(), &Default::default(), &CFG).unwrap();
        let projected = &pipeline.projection;
        // Anchor-free two-node chain: the P8 gauge spread places both ends.
        assert!(projected.contains_key("SL-100"));
        assert!(projected.contains_key("SL-200"));
        let (winner, _) = projected["SL-100"];
        let (loser, _) = projected["SL-200"];
        assert!(winner > loser, "the preferred side outranks the other");
        // The one row joins to Active + Constraining ⇒ "active" (design §4 S2).
        assert_eq!(pipeline.rows.len(), 1);
        assert_eq!(pipeline.rows[0].state.display_token(), "active");
    }
}
