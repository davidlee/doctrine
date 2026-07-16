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
use super::project::{self, Projection, ProjectionCfg, ValueProvenance};
use super::resolve::{
    self, MalformedSupersession, ResolutionStatus, RowState, StatusMap, rater_key,
};
use super::{
    COMPARISONS_DIR, ComparisonSession, DOMAIN_ESTIMATE, DOMAIN_PRIORITY, Judgement, Response,
};

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
    /// The row's authored domain (SL-219 §2) — the key the per-domain
    /// status routing and the findings domain discriminator join on.
    pub domain: String,
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
    /// The value-domain system (SL-219 §2): compiled from the Active
    /// non-estimate rows + the authored `[value]` anchors.
    pub value: DomainSystem,
    /// The estimate-domain system (SL-219 §2): compiled from the Active
    /// estimate rows + the ROW-GATED `authored_est_cost` anchors. No est rows
    /// ⇒ an empty system (cold-start no-op by construction).
    pub estimate: DomainSystem,
    /// The VALUE system's per-class rater split (the T7 disclosure — the
    /// est-domain analog is PHASE-06's cost-source block).
    pub constraining_by_class: BTreeMap<ClassId, RaterCounts>,
    /// The ESTIMATE system's per-class rater split (SL-219 PHASE-06) — the
    /// cost-source "projected" shape's `(human, agent)` disclosure. Keyed by
    /// est-domain `ClassId` (disjoint from the value classes); `NoConstraint`
    /// rows excluded exactly as the value split is (S3 precedent).
    pub est_constraining_by_class: BTreeMap<ClassId, RaterCounts>,
    pub malformed: Vec<MalformedSupersession>,
    pub priority_domain_count: usize,
    /// The resolved-ACTIVE VALUE-domain judgements, owned (SL-217 PHASE-03;
    /// domain-scoped by SL-219 — the est rows compile in their own system and
    /// must not leak into the value recompiles). `resolve` borrows the loaded
    /// `sessions`, which drop on return — so the elicit shell cannot borrow
    /// `active` back out. Owned clones let `assemble` recompile its own
    /// baseline `ConstraintSet` from the SAME evidence, no re-resolve (DRY).
    pub active_judgements: Vec<Judgement>,
}

/// One per-domain comparison system (SL-219 design §2): the compiled
/// [`ConstraintSet`], the anchor map actually compiled into it, and its
/// projection. Two live per [`Pipeline`] (`value` / `estimate`); `resolution`
/// stays SHARED — one resolve pass over all rows, the per-domain split
/// happens at compile-input selection, not resolution (design §1).
pub(crate) struct DomainSystem {
    pub constraint_set: ConstraintSet,
    /// The anchor map compiled into `constraint_set` (SL-217 PHASE-03) — exposed
    /// so the elicit shell feeds `assemble` the identical anchors. For the
    /// estimate system this is the row-gated map, not the full candidate set.
    pub anchors: AnchorMap,
    pub projection: Projection,
}

impl DomainSystem {
    /// Compile + project one domain's system from ITS OWN rows and anchors
    /// (C2–C5 apply per system), under ITS OWN projection params (SL-219 D8:
    /// per-call params isolate the domains — value passes the shipped
    /// `VALUE_PROJECTION_PARAMS`, estimate passes `(gauge_step, gauge_center =
    /// ctx.absent)`; the caller threads both).
    fn compiled(rows: &[&Judgement], anchors: &AnchorMap, cfg: &ProjectionCfg) -> Self {
        let constraint_set = compile::compile(rows, anchors, QuarantinePolicy::Symmetric);
        let projection = project::project(&constraint_set, cfg);
        DomainSystem {
            constraint_set,
            anchors: anchors.clone(),
            projection,
        }
    }

    /// An empty system (the sessions-empty short-circuit): nothing compiled,
    /// nothing projected — no resolve/compile/project cost beyond the empty
    /// calls the pre-split pipeline already made.
    fn empty(anchors: &AnchorMap) -> Self {
        DomainSystem {
            constraint_set: compile::compile(&[], anchors, QuarantinePolicy::Symmetric),
            anchors: anchors.clone(),
            projection: Projection::new(),
        }
    }
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
    value_anchors: &AnchorMap,
    est_anchors: &AnchorMap,
    value_cfg: &ProjectionCfg,
    est_cfg: &ProjectionCfg,
) -> anyhow::Result<Pipeline> {
    let sessions = load_sessions(root)?;
    pipeline_from_sessions(
        &sessions,
        statuses,
        value_anchors,
        est_anchors,
        value_cfg,
        est_cfg,
    )
}

/// The disk-FREE inner half of [`load_pipeline`]: `resolve` → `compile` →
/// `project`, PLUS the PHASE-06 derived accessors every surface needs, over
/// an ALREADY-LOADED session slice. Split out so tests compose sessions in
/// memory without a filesystem round-trip (the pure/impure split the
/// project's conventions ask for).
pub(crate) fn pipeline_from_sessions(
    sessions: &[ComparisonSession],
    statuses: &StatusMap,
    value_anchors: &AnchorMap,
    est_anchors: &AnchorMap,
    value_cfg: &ProjectionCfg,
    est_cfg: &ProjectionCfg,
) -> anyhow::Result<Pipeline> {
    if sessions.is_empty() {
        return Ok(Pipeline {
            rows: Vec::new(),
            value: DomainSystem::empty(value_anchors),
            // Row-gated (SL-219 §1): no rows ⇒ no est anchors ⇒ no system.
            estimate: DomainSystem::empty(&AnchorMap::new()),
            constraining_by_class: BTreeMap::new(),
            est_constraining_by_class: BTreeMap::new(),
            malformed: Vec::new(),
            priority_domain_count: 0,
            active_judgements: Vec::new(),
        });
    }

    // ONE shared resolve pass over all rows (SL-219 §1: the per-domain split
    // happens at compile-input selection, not resolution).
    let resolution = resolve::resolve(sessions, statuses)?;
    let active: Vec<&Judgement> = resolution
        .rows
        .iter()
        .filter(|(_, status)| matches!(status, ResolutionStatus::Active))
        .map(|(j, _)| *j)
        .collect();
    // Compile-input selection (SL-219 §2): each domain compiles ONLY its own
    // rows. The value side keeps every Active non-estimate row — the exact
    // pre-split input minus the est rows (the defect this split fixes: an
    // Active estimate row used to flow into value compilation).
    let (est_active, value_active): (Vec<&Judgement>, Vec<&Judgement>) =
        active.iter().partition(|j| j.domain == DOMAIN_ESTIMATE);
    // Row-gating (SL-219 §1, mem.fact.comparison.anchor-attachment-row-gated-
    // per-system): an authored estimate enters the est system only when an
    // est-domain row touches its item.
    let gated_est_anchors: AnchorMap = est_anchors
        .iter()
        .filter(|(item, _)| est_active.iter().any(|j| j.a == **item || j.b == **item))
        .map(|(item, &v)| (item.clone(), v))
        .collect();

    let value = DomainSystem::compiled(&value_active, value_anchors, value_cfg);
    let estimate = DomainSystem::compiled(&est_active, &gated_est_anchors, est_cfg);
    let constraining_by_class =
        compile::constraining_counts_by_class(&value.constraint_set, &value_active);
    // SL-219 PHASE-06: the est system's own rater split, for the cost-source
    // "projected" shape (NoConstraint excluded, same as the value split).
    let est_constraining_by_class =
        compile::constraining_counts_by_class(&estimate.constraint_set, &est_active);
    let priority_domain_count = resolution
        .rows
        .iter()
        .filter(|(j, _)| j.domain == DOMAIN_PRIORITY)
        .count();

    let rows = resolution
        .rows
        .iter()
        .map(|(j, status)| {
            // Row-status routing (SL-219 §2): `CompilationStatus` is assigned
            // by the row's OWNING domain system — the quarantine maps are
            // disjoint by construction (each row belongs to exactly one
            // domain); `RowState` joins by row uid.
            let compilation = matches!(status, ResolutionStatus::Active).then(|| {
                if j.domain == DOMAIN_ESTIMATE {
                    estimate.constraint_set.status_of(j)
                } else {
                    value.constraint_set.status_of(j)
                }
            });
            RowSummary {
                uid: j.uid.clone(),
                a: j.a.clone(),
                b: j.b.clone(),
                response: j.response,
                domain: j.domain.clone(),
                frame: j.frame.clone(),
                rater_token: rater_key(&j.rater),
                by: j.by.clone(),
                note: j.note.clone(),
                date: j.date.clone(),
                state: RowState::new(status.clone(), compilation),
            }
        })
        .collect();

    // Owned clones of the value-domain active evidence for the elicit shell
    // (SL-217 PHASE-03): taken while `resolution` (which the borrows come
    // from) is still alive.
    let active_judgements: Vec<Judgement> = value_active.iter().map(|&j| j.clone()).collect();

    Ok(Pipeline {
        rows,
        value,
        estimate,
        constraining_by_class,
        est_constraining_by_class,
        malformed: resolution.malformed,
        priority_domain_count,
        active_judgements,
    })
}

/// The scoring cost feed shape: canonical entity id → fed (non-Gauge)
/// projected cost. Produced by [`cost_feed`]; threaded by the shells into
/// `graph::build_from_with_cfg` (SL-219 design §2 flow).
pub(crate) type CostFeed = BTreeMap<String, f64>;

/// The scoring cost feed (SL-219 design §2 flow, D2): the ESTIMATE-domain
/// projection MINUS its Gauge tier. The executable tier rule:
///
/// | projection case | provenance | fed? |
/// |---|---|---|
/// | P3 anchored class (incl. members merged in without their own facet) | `Authored` | yes — covers the merge-hoisted bare member; items with their own authored facet never consult the feed downstream (ladder order), so redundant entries are inert |
/// | P4/P5/P6 placements | `Projected` | yes |
/// | P7 no-floor-no-ceiling, P8 anchor-free component | `Gauge` | **no** — gauge renders, never divides (D2) |
///
/// Fed costs are strictly positive (D11 positivity axiom: the est feasible
/// region is order constraints + point anchors + `c > 0`; every anchor > 0
/// because `authored_est_cost` floors at EPSILON, and P6's clamp keeps
/// interpolants under a positive ceiling above zero). Pure; the SOLE consumer
/// is `graph::est_cost`'s ladder — the single consumption seam (D6).
pub(crate) fn cost_feed(est_projection: &Projection) -> CostFeed {
    est_projection
        .iter()
        .filter(|(_, (_, provenance))| *provenance != ValueProvenance::Gauge)
        .map(|(entity, &(cost, _))| (entity.clone(), cost))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{AnchorMap, Projection, ProjectionCfg, StatusMap, load_pipeline, load_sessions};
    use crate::comparison::{
        COMPARISON_SCHEMA, COMPARISON_VERSION, ComparisonSession, DOMAIN_ESTIMATE, DOMAIN_VALUE,
        FRAME_EQUAL_EFFORT, FRAME_MORE_WORK, Judgement, QuarantineReason, RaterKind, Response,
        RowForm, SessionHeader, VALUE_PROJECTION_PARAMS, ValueProvenance, cost_feed,
        pipeline_from_sessions,
    };
    use crate::priority::config::EST_GAUGE_STEP;

    const CFG: ProjectionCfg = VALUE_PROJECTION_PARAMS;

    /// The est-domain projection params the VT-2 fixtures run under (SL-219
    /// D8): the shipped `EST_GAUGE_STEP` and a gauge center standing in for
    /// the fixture corpus's bare-item anchor `ctx.absent` (`max_upper +
    /// margin` — the caller-computed D7 quantity; `graph.rs` owns the real
    /// fold, this module receives the finished params).
    const EST_CENTER: f64 = 11.0;
    const EST_CFG: ProjectionCfg = ProjectionCfg {
        gauge_step: EST_GAUGE_STEP,
        gauge_center: EST_CENTER,
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
        let pipeline = load_pipeline(
            dir.path(),
            &StatusMap::new(),
            &Default::default(),
            &Default::default(),
            &CFG,
            &EST_CFG,
        )
        .unwrap();
        assert!(pipeline.value.projection.is_empty());
        assert!(pipeline.estimate.projection.is_empty());
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
        let pipeline = load_pipeline(
            root,
            &StatusMap::new(),
            &Default::default(),
            &Default::default(),
            &CFG,
            &EST_CFG,
        )
        .unwrap();
        let projected = &pipeline.value.projection;
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

    // ---- SL-219 VT-1: DomainSystem split & estimate compile -------------------

    /// An estimate-domain judgement: `more-work` frame, winner = the COSTLIER
    /// side (`prefer-a` ⇒ `c_a > c_b`, design D5).
    fn est_judgement(uid: &str, costlier: &str, cheaper: &str) -> Judgement {
        let mut j = judgement(uid, costlier, cheaper);
        j.domain = DOMAIN_ESTIMATE.to_string();
        j.frame = FRAME_MORE_WORK.to_string();
        j
    }

    /// The in-memory pipeline over one session of `judgements` with the given
    /// anchor maps (the VT-1 harness).
    fn pipeline_of(
        judgements: Vec<Judgement>,
        value_anchors: &AnchorMap,
        est_anchors: &AnchorMap,
    ) -> super::Pipeline {
        let sessions = vec![session("s1", "2026-07-11", judgements)];
        pipeline_from_sessions(
            &sessions,
            &StatusMap::new(),
            value_anchors,
            est_anchors,
            &CFG,
            &EST_CFG,
        )
        .unwrap()
    }

    /// The DomainSystem split: each domain compiles ONLY its own rows — an
    /// Active estimate row no longer flows into value compilation (the
    /// pre-SL-219 defect), and value rows never enter the estimate system.
    #[test]
    fn each_domain_system_compiles_only_its_own_rows() {
        let pipeline = pipeline_of(
            vec![
                judgement("v1", "SL-100", "SL-200"),
                est_judgement("e1", "SL-100", "SL-300"),
            ],
            &AnchorMap::new(),
            &AnchorMap::new(),
        );
        let value_entities: Vec<&String> = pipeline.value.constraint_set.classes.keys().collect();
        assert_eq!(value_entities, ["SL-100", "SL-200"], "value rows only");
        let est_entities: Vec<&String> = pipeline.estimate.constraint_set.classes.keys().collect();
        assert_eq!(est_entities, ["SL-100", "SL-300"], "estimate rows only");
        // The owned value evidence excludes the est row (elicit recompiles
        // the value baseline from this — no cross-domain leak).
        let active_uids: Vec<&str> = pipeline
            .active_judgements
            .iter()
            .map(|j| j.uid.as_str())
            .collect();
        assert_eq!(active_uids, ["v1"]);
    }

    /// Est anchors are the β-resolved `authored_est_cost` points (design D1),
    /// ROW-GATED per system: an authored estimate enters the est system only
    /// when an est-domain row touches its item; no est rows ⇒ an empty est
    /// system (cold-start no-op).
    #[test]
    fn est_anchors_from_authored_est_cost_are_row_gated() {
        let ec = crate::priority::config::EstimateCost::default();
        let cost = |bounds| crate::priority::graph::authored_est_cost(bounds, &ec);
        let est_anchors: AnchorMap = [
            ("SL-100".to_string(), cost((1.0, 3.0))),
            ("SL-300".to_string(), cost((5.0, 9.0))),
            ("SL-400".to_string(), cost((2.0, 4.0))), // no row touches it
        ]
        .into_iter()
        .collect();

        let pipeline = pipeline_of(
            vec![est_judgement("e1", "SL-300", "SL-100")],
            &AnchorMap::new(),
            &est_anchors,
        );
        // Gated map: SL-400 (row-less) never enters the compiled system.
        let gated: Vec<&String> = pipeline.estimate.anchors.keys().collect();
        assert_eq!(gated, ["SL-100", "SL-300"], "row-gated anchor set");
        // The compiled class anchors carry the authored_est_cost points.
        let cs = &pipeline.estimate.constraint_set;
        let class_of = |e: &str| cs.classes.get(e).unwrap();
        assert_eq!(cs.anchors.get(class_of("SL-100")), Some(&cost((1.0, 3.0))));
        assert_eq!(cs.anchors.get(class_of("SL-300")), Some(&cost((5.0, 9.0))));

        // Cold start: no est rows ⇒ empty est system despite candidate anchors.
        let cold = pipeline_of(
            vec![judgement("v1", "SL-100", "SL-200")],
            &AnchorMap::new(),
            &est_anchors,
        );
        assert!(cold.estimate.anchors.is_empty(), "no rows ⇒ no est anchors");
        assert!(cold.estimate.constraint_set.classes.is_empty());
        assert!(cold.estimate.projection.is_empty());
    }

    /// C2 in the estimate system: an `equal` sizing row merging two
    /// differently COST-anchored items is quarantined as an anchor conflict.
    #[test]
    fn c2_cost_anchor_merge_conflict_quarantines_the_equal_row() {
        let est_anchors: AnchorMap = [("SL-100".to_string(), 1.0), ("SL-300".to_string(), 5.0)]
            .into_iter()
            .collect();
        let mut equal_row = est_judgement("e1", "SL-100", "SL-300");
        equal_row.response = Response::Equal;
        let pipeline = pipeline_of(vec![equal_row], &AnchorMap::new(), &est_anchors);
        assert!(
            matches!(
                pipeline.estimate.constraint_set.quarantined.get("e1"),
                Some(QuarantineReason::AnchorConflict { .. })
            ),
            "C2 cost-anchor merge conflict quarantines the row: {:?}",
            pipeline.estimate.constraint_set.quarantined
        );
        assert!(pipeline.value.constraint_set.quarantined.is_empty());
    }

    /// C4 stale-estimate violation-closure golden (design D1): sizing evidence
    /// ordering a cheap-anchored item ABOVE a dear-anchored one contradicts the
    /// β-resolved costs — the row is quarantined naming the anchor pair
    /// `(x, y)` with `anchor(x) ≤ anchor(y)`, and its RowState joins from the
    /// OWNING (estimate) system.
    #[test]
    fn c4_stale_estimate_violation_closure_golden() {
        let est_anchors: AnchorMap = [
            ("SL-100".to_string(), 1.0), // cheap anchor…
            ("SL-300".to_string(), 5.0),
        ]
        .into_iter()
        .collect();
        // …but the sizing row says SL-100 is MORE work than SL-300.
        let pipeline = pipeline_of(
            vec![est_judgement("e1", "SL-100", "SL-300")],
            &AnchorMap::new(),
            &est_anchors,
        );
        assert_eq!(
            pipeline.estimate.constraint_set.quarantined.get("e1"),
            Some(&QuarantineReason::AnchorConflict {
                pairs: vec![("SL-100".to_string(), "SL-300".to_string())],
            }),
            "the C4 closure names the contradicted anchor pair"
        );
        let row = pipeline.rows.iter().find(|r| r.uid == "e1").unwrap();
        assert_eq!(row.domain, DOMAIN_ESTIMATE);
        assert_eq!(row.state.display_token(), "quarantined(anchors)");
    }

    /// The two quarantine maps are DISJOINT by row uid (each row belongs to
    /// exactly one domain; `CompilationStatus` is assigned by its owning
    /// domain's compile), and each row's state joins from its own system.
    #[test]
    fn quarantine_maps_are_disjoint_and_rowstate_joins_by_owning_domain() {
        // Value system: a C3 preference cycle (v1..v3 all quarantined).
        // Estimate system: a C4 anchor conflict (e1 quarantined).
        let est_anchors: AnchorMap = [("SL-100".to_string(), 1.0), ("SL-300".to_string(), 5.0)]
            .into_iter()
            .collect();
        let pipeline = pipeline_of(
            vec![
                judgement("v1", "SL-100", "SL-200"),
                judgement("v2", "SL-200", "SL-500"),
                judgement("v3", "SL-500", "SL-100"),
                est_judgement("e1", "SL-100", "SL-300"),
            ],
            &AnchorMap::new(),
            &est_anchors,
        );
        let value_uids: Vec<&String> = pipeline.value.constraint_set.quarantined.keys().collect();
        let est_uids: Vec<&String> = pipeline
            .estimate
            .constraint_set
            .quarantined
            .keys()
            .collect();
        assert_eq!(value_uids, ["v1", "v2", "v3"], "value cycle quarantined");
        assert_eq!(est_uids, ["e1"], "est conflict quarantined");
        assert!(
            value_uids.iter().all(|uid| !est_uids.contains(uid)),
            "quarantine maps disjoint by row uid"
        );
        // RowState joins by owning domain: v-rows cite the cycle, e1 the anchors.
        for uid in ["v1", "v2", "v3"] {
            let row = pipeline.rows.iter().find(|r| r.uid == uid).unwrap();
            assert_eq!(row.state.display_token(), "quarantined(cycle)");
        }
        let e1 = pipeline.rows.iter().find(|r| r.uid == "e1").unwrap();
        assert_eq!(e1.state.display_token(), "quarantined(anchors)");
    }

    // ---- SL-219 PHASE-03 VT-2: est projection goldens & the cost feed --------

    use ValueProvenance::{Authored, Gauge, Projected};

    const EPS: f64 = 1e-4;

    /// The in-memory pipeline over one est-domain session with the given cost
    /// anchors and est projection params (the VT-2 harness; the value system
    /// stays empty — the domains are independent by PHASE-02).
    fn est_pipeline(
        judgements: Vec<Judgement>,
        est_anchors: &AnchorMap,
        est_cfg: &ProjectionCfg,
    ) -> super::Pipeline {
        let sessions = vec![session("s1", "2026-07-11", judgements)];
        pipeline_from_sessions(
            &sessions,
            &StatusMap::new(),
            &AnchorMap::new(),
            est_anchors,
            &CFG,
            est_cfg,
        )
        .unwrap()
    }

    fn anchors_of(pairs: &[(&str, f64)]) -> AnchorMap {
        pairs.iter().map(|&(e, v)| (e.to_string(), v)).collect()
    }

    /// Assert an entity's projected cost (approx 1e-4) and provenance.
    fn cost_golden(p: &Projection, e: &str, expected: f64, prov: ValueProvenance) {
        let (got, got_prov) = *p.get(e).unwrap_or_else(|| panic!("missing entity {e}"));
        assert!(
            (got - expected).abs() < EPS,
            "{e}: got {got:.4}, want {expected:.4}"
        );
        assert_eq!(got_prov, prov, "{e} provenance");
        assert!(!got.is_nan(), "{e} is NaN");
    }

    /// Anchored chain golden: two cost anchors bracket a sizing chain; the
    /// interior interpolates (P4), the anchors are exact (P3), and every
    /// placement is FED at its projected cost.
    #[test]
    fn est_projection_anchored_chain_golden() {
        let pipeline = est_pipeline(
            vec![
                est_judgement("e1", "SL-100", "SL-200"),
                est_judgement("e2", "SL-200", "SL-300"),
                est_judgement("e3", "SL-300", "SL-400"),
            ],
            &anchors_of(&[("SL-100", 8.0), ("SL-400", 2.0)]),
            &EST_CFG,
        );
        let p = &pipeline.estimate.projection;
        cost_golden(p, "SL-100", 8.0, Authored);
        cost_golden(p, "SL-200", 6.0, Projected);
        cost_golden(p, "SL-300", 4.0, Projected);
        cost_golden(p, "SL-400", 2.0, Authored);

        let feed = cost_feed(p);
        assert_eq!(feed.len(), 4, "every non-Gauge placement is fed");
        for (entity, &(cost, _)) in p {
            assert_eq!(feed.get(entity), Some(&cost), "{entity} fed at its cost");
        }
    }

    /// P5 above-top-anchor: an item evidenced MORE work than the costliest
    /// anchor takes one additive `EST_GAUGE_STEP` off that floor — Projected,
    /// fed.
    #[test]
    fn est_projection_p5_above_top_anchor_golden() {
        let pipeline = est_pipeline(
            vec![est_judgement("e1", "SL-100", "SL-200")],
            &anchors_of(&[("SL-200", 5.0)]),
            &EST_CFG,
        );
        let p = &pipeline.estimate.projection;
        cost_golden(p, "SL-200", 5.0, Authored);
        cost_golden(p, "SL-100", 5.0 + EST_GAUGE_STEP, Projected);
        assert_eq!(
            cost_feed(p).get("SL-100"),
            Some(&(5.0 + EST_GAUGE_STEP)),
            "the P5 head is fed"
        );
    }

    /// P6 compression below a small anchor — the §3 documented artifact: three
    /// items evidenced strictly below a 0.5 anchor are FORCED into (0, 0.5) by
    /// the evidence + the D11 positivity axiom; the even spacing inside the
    /// interval is convention (interpolation over the clamped synthetic
    /// floor). All three are Projected, fed, and strictly positive.
    #[test]
    fn est_projection_p6_compression_below_small_anchor_golden() {
        let pipeline = est_pipeline(
            vec![
                est_judgement("e1", "SL-100", "SL-200"),
                est_judgement("e2", "SL-200", "SL-300"),
                est_judgement("e3", "SL-300", "SL-400"),
            ],
            &anchors_of(&[("SL-100", 0.5)]),
            &EST_CFG,
        );
        let p = &pipeline.estimate.projection;
        cost_golden(p, "SL-100", 0.5, Authored);
        cost_golden(p, "SL-200", 0.375, Projected);
        cost_golden(p, "SL-300", 0.25, Projected);
        cost_golden(p, "SL-400", 0.125, Projected);
        let feed = cost_feed(p);
        for e in ["SL-200", "SL-300", "SL-400"] {
            let cost = feed[e];
            assert!(cost > 0.0 && cost < 0.5, "{e} forced into (0, 0.5): {cost}");
        }
    }

    /// D2: a gauge component is PRESENT in the est projection (render-only,
    /// centered on the corpus's own bare anchor — the est `gauge_center`) and
    /// ABSENT from the cost feed: gauge renders, never divides.
    #[test]
    fn gauge_component_present_in_projection_absent_from_feed() {
        let pipeline = est_pipeline(
            vec![
                est_judgement("e1", "SL-100", "SL-200"),
                est_judgement("e2", "SL-500", "SL-600"),
            ],
            &anchors_of(&[("SL-100", 5.0)]),
            &EST_CFG,
        );
        let p = &pipeline.estimate.projection;
        // The anchor-free island spreads around the bare anchor (P8, H = 1).
        cost_golden(p, "SL-500", 2.0 * EST_CENTER * 2.0 / 3.0, Gauge);
        cost_golden(p, "SL-600", 2.0 * EST_CENTER / 3.0, Gauge);
        let (hi, _) = p["SL-500"];
        let (lo, _) = p["SL-600"];
        assert!(
            ((hi + lo) / 2.0 - EST_CENTER).abs() < EPS,
            "gauge centers on the corpus's own bare anchor"
        );

        let feed = cost_feed(p);
        let fed: Vec<&String> = feed.keys().collect();
        assert_eq!(fed, ["SL-100", "SL-200"], "only the anchored component");
        assert!(!feed.contains_key("SL-500") && !feed.contains_key("SL-600"));
    }

    /// `EST_GAUGE_STEP` sensitivity sweep (design §6.4): across the step grid
    /// the sizing order stays respected (order-safety), every placement keeps
    /// its provenance tier (so the FED SET is step-invariant), and every fed
    /// cost stays strictly positive (D11).
    #[test]
    fn est_gauge_step_sweep_order_and_provenance_invariant() {
        // P5 head above the anchor + P6 tail below it, in one chain.
        let rows = || {
            vec![
                est_judgement("e1", "SL-100", "SL-200"),
                est_judgement("e2", "SL-200", "SL-300"),
                est_judgement("e3", "SL-300", "SL-400"),
            ]
        };
        let anchors = anchors_of(&[("SL-200", 0.5)]);
        let order = ["SL-100", "SL-200", "SL-300", "SL-400"];
        let mut step_milli = 5_u32;
        while step_milli <= 1000 {
            let cfg = ProjectionCfg {
                gauge_step: f64::from(step_milli) / 1000.0,
                gauge_center: EST_CENTER,
            };
            let p = est_pipeline(rows(), &anchors, &cfg).estimate.projection;
            for pair in order.windows(2) {
                let (w, _) = p[pair[0]];
                let (l, _) = p[pair[1]];
                assert!(
                    w > l,
                    "order {}>{} broke at step {step_milli}",
                    pair[0],
                    pair[1]
                );
            }
            assert_eq!(p["SL-200"].1, Authored, "anchor provenance at {step_milli}");
            for e in ["SL-100", "SL-300", "SL-400"] {
                assert_eq!(p[e].1, Projected, "{e} provenance at {step_milli}");
            }
            for (e, cost) in cost_feed(&p) {
                assert!(
                    cost > 0.0,
                    "fed {e} not positive at step {step_milli}: {cost}"
                );
            }
            step_milli += 50;
        }
    }

    /// D11 positivity property over DETERMINISTICALLY generated est ledgers
    /// (the `project.rs` enumeration idiom — no rng): every response
    /// assignment over four entities' six pairs (4^6: absent / more-work
    /// either way / equal) crossed with three positive anchor configs. Every
    /// fed cost is strictly positive and non-NaN.
    #[test]
    fn cost_feed_fed_costs_positive_property() {
        const PAIRS: [(&str, &str); 6] = [
            ("SL-100", "SL-200"),
            ("SL-100", "SL-300"),
            ("SL-100", "SL-400"),
            ("SL-200", "SL-300"),
            ("SL-200", "SL-400"),
            ("SL-300", "SL-400"),
        ];
        let configs: [Vec<(&str, f64)>; 3] = [
            vec![],
            vec![("SL-100", 0.5), ("SL-400", 0.01)],
            vec![("SL-100", 8.0), ("SL-200", 3.0), ("SL-400", 1.0)],
        ];
        for cfg in configs {
            let anchors = anchors_of(&cfg);
            for mask in 0..4_u32.pow(6) {
                let mut rows = Vec::new();
                for (i, (a, b)) in PAIRS.iter().enumerate() {
                    let uid = format!("j{i}");
                    match (mask / 4_u32.pow(u32::try_from(i).unwrap())) % 4 {
                        1 => rows.push(est_judgement(&uid, a, b)),
                        2 => rows.push(est_judgement(&uid, b, a)),
                        3 => {
                            let mut eq = est_judgement(&uid, a, b);
                            eq.response = Response::Equal;
                            rows.push(eq);
                        }
                        _ => {}
                    }
                }
                if rows.is_empty() {
                    continue;
                }
                let p = est_pipeline(rows, &anchors, &EST_CFG).estimate.projection;
                for (e, cost) in cost_feed(&p) {
                    assert!(
                        cost > 0.0 && !cost.is_nan(),
                        "fed {e} violates positivity under mask {mask}: {cost}"
                    );
                }
            }
        }
    }
}
