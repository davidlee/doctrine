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
    COMPARISONS_DIR, ClaimResolution, ClaimResolutionGeneric, ComparisonSession, DOMAIN_ESTIMATE,
    DOMAIN_PRIORITY, EstimatePayload, Judgement, Response, RowForm, resolve_claims,
    resolve_claims_generic,
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
    /// The pairwise partner — absent on anchor rows (single subject,
    /// SL-220 §1).
    pub b: Option<String>,
    /// The pairwise response — absent on anchor rows (SL-220 §1).
    pub response: Option<Response>,
    /// The row's authored domain (SL-219 §2) — the key the per-domain
    /// status routing and the findings domain discriminator join on.
    pub domain: String,
    pub frame: String,
    pub rater_token: &'static str,
    pub by: Option<String>,
    pub note: Option<String>,
    pub date: String,
    /// The claim-join token for an ACTIVE anchor row (`anchored` / `prior` /
    /// `conflicted`), produced HERE at the [`ClaimResolution`] join (SL-220
    /// design §2, RV-278 F-8) — never in `resolve::display_token`. `None` on
    /// pairwise rows and on non-Active anchor rows (which keep their
    /// resolution token).
    pub claim: Option<&'static str>,
    pub state: RowState,
}

impl RowSummary {
    /// The single display token: the claim join for an active anchor row,
    /// the [`RowState`] join otherwise (SL-220 design §2 — anchor rows never
    /// acquire a `CompilationStatus`).
    pub(crate) fn display_token(&self) -> String {
        self.claim
            .map_or_else(|| self.state.display_token(), str::to_string)
    }
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
    /// non-estimate PAIRWISE rows + the claim-derived anchors
    /// (`ClaimResolution::anchor_map()`, SL-220 D4/D12 — facets stopped
    /// anchoring at the flip).
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
    /// The value-domain claim resolution (SL-220 design §2): the anchor-side
    /// output of the pairwise/anchor split. Anchor rows TERMINATE here —
    /// no compile consumer reads them (RV-278 F-6). Since the PHASE-05 flip
    /// its `anchor_map()` IS the value system's compile anchor source (D4/
    /// D12), and the graph ladder reads `anchored`/`priors` directly (§3).
    pub value_claims: ClaimResolution,
    /// The estimate-domain claim resolution (SL-222 E7): the anchor-side
    /// output over estimate-anchor rows. Like `value_claims` but over
    /// `EstimatePayload` with β-skew params. Its `anchor_map()` is the
    /// Pin/Human estimate cost anchors; the resource ladder reads them.
    /// PHASE-06 consumption — read by `graph::load_comparison_pipeline`'s
    /// caller chain (the graph scoring ladder's rungs 1/3/4).
    pub estimate_claims: ClaimResolutionGeneric<EstimatePayload>,
    /// The estimate-domain bare anchor: `max_upper` over the union of (a)
    /// EVERY active unlensed estimate-anchor row's `est_upper` and (b) the
    /// facet-uppers threaded from the scan shell, + `margin`. Equals the
    /// est system's `ProjectionCfg.gauge_center` and `CostCtx.absent`.
    /// 1.0 fallback on empty-evidence corpus (§3 E7, RV-282 F-2/F-4).
    pub bare_anchor: f64,
    /// The resolved-ACTIVE VALUE-domain PAIRWISE judgements, owned (SL-217
    /// PHASE-03; domain-scoped by SL-219 — the est rows compile in their own
    /// system and must not leak into the value recompiles; anchor-free by
    /// SL-220's pairwise/anchor split — read it via
    /// [`Pipeline::active_pairwise`]). `resolve` borrows the loaded
    /// `sessions`, which drop on return — so the elicit shell cannot borrow
    /// `active` back out. Owned clones let `assemble` recompile its own
    /// baseline `ConstraintSet` from the SAME evidence, no re-resolve (DRY).
    pub active_pairwise: Vec<Judgement>,
}

impl Pipeline {
    /// The PAIRWISE view of the SL-220 §2 split: every recompiler (elicit
    /// `assemble`, tension grading) consumes THIS — never anchor rows, which
    /// terminate at [`ClaimResolution`] (RV-278 F-6).
    pub(crate) fn active_pairwise(&self) -> &[Judgement] {
        &self.active_pairwise
    }
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
#[expect(
    clippy::too_many_arguments,
    reason = "PHASE-04 pipeline wiring: load_pipeline receives 7 params — root + 5 shared (statuses, est_anchors, value_cfg, est_cfg) + 3 new PHASE-04 params (facet_uppers, margin, estimate_skew). Acceptable for an integration seam."
)]
pub(crate) fn load_pipeline(
    root: &Path,
    statuses: &StatusMap,
    est_anchors: &AnchorMap,
    value_cfg: &ProjectionCfg,
    est_cfg: &ProjectionCfg,
    facet_uppers: &BTreeMap<String, f64>,
    margin: f64,
    estimate_skew: f64,
) -> anyhow::Result<Pipeline> {
    let sessions = load_sessions(root)?;
    pipeline_from_sessions(
        &sessions,
        statuses,
        est_anchors,
        value_cfg,
        est_cfg,
        facet_uppers,
        margin,
        estimate_skew,
    )
}

/// The disk-FREE inner half of [`load_pipeline`]: `resolve` → `compile` →
/// `project`, PLUS the PHASE-06 derived accessors every surface needs, over
/// an ALREADY-LOADED session slice. Split out so tests compose sessions in
/// memory without a filesystem round-trip (the pure/impure split the
/// project's conventions ask for).
#[expect(
    clippy::too_many_arguments,
    reason = "PHASE-04 pipeline wiring: 8 params for the core pipeline function — 5 shared domain params + 3 new PHASE-04 params (facet_uppers, margin, estimate_skew). Acceptable for a central integration seam."
)]
pub(crate) fn pipeline_from_sessions(
    sessions: &[ComparisonSession],
    statuses: &StatusMap,
    _est_anchors: &AnchorMap,
    value_cfg: &ProjectionCfg,
    est_cfg: &ProjectionCfg,
    facet_uppers: &BTreeMap<String, f64>,
    margin: f64,
    estimate_skew: f64,
) -> anyhow::Result<Pipeline> {
    if sessions.is_empty() {
        // No sessions ⇒ no claim rows ⇒ no value anchors (SL-220 D4: claims
        // are the ONLY value-anchor source since the PHASE-05 flip).
        // Bare anchor falls back to facet uppers (the old `bare_cost_anchor`
        // computation — transitional, will be PHASE-06's flip).
        let max_upper = facet_uppers.values().copied().max_by(f64::total_cmp);
        let bare_anchor = match max_upper {
            Some(mu) => mu + margin,
            None => 1.0,
        };
        return Ok(Pipeline {
            rows: Vec::new(),
            value: DomainSystem::empty(&AnchorMap::new()),
            // Row-gated (SL-219 §1): no rows ⇒ no est anchors ⇒ no system.
            estimate: DomainSystem::empty(&AnchorMap::new()),
            constraining_by_class: BTreeMap::new(),
            est_constraining_by_class: BTreeMap::new(),
            malformed: Vec::new(),
            priority_domain_count: 0,
            value_claims: ClaimResolution::default(),
            estimate_claims: ClaimResolutionGeneric::default(),
            bare_anchor,
            active_pairwise: Vec::new(),
        });
    }

    // ONE shared resolve pass over all rows (SL-219 §1: the per-domain split
    // happens at compile-input selection, not resolution).
    let resolution = resolve::resolve(sessions, statuses)?;
    // The claims pass (SL-220 design §2): its OWN selection over the
    // post-resolve rows (value-domain anchors, Active or InertLens). Its
    // anchor_map() — the Pin/Human tiers ONLY (D4 anti-laundering) — is the
    // value system's compile anchor source since the PHASE-05 flip: facets
    // stopped anchoring/shaping projection (design §3, RV-278 F-4).
    let value_claims = resolve_claims(&resolution.rows);
    // SL-222 E7: the estimate-domain claims pass — selects estimate-anchor
    // rows (Active/InertLens), folded via `EstimatePayload` with β-skew.
    let estimate_claims = resolve_claims_generic::<EstimatePayload>(
        &resolution.rows,
        DOMAIN_ESTIMATE,
        &estimate_skew,
    );
    let value_anchors = value_claims.anchor_map();

    // ── In-pipeline bare anchor (§3 E7, RV-282 F-2/F-4) ────────────────
    // Must be computed BEFORE the estimate system compiles so we can thread
    // it as the gauge_center. max_upper = max over:
    //   (a) EVERY active unlensed estimate-anchor row's est_upper — any
    //       tier, losing tiers and individual conflict rows included, lensed
    //       rows excluded, tombstoned/superseded rows already reduced by
    //       resolution;
    //   (b) the facet-uppers pure input threaded from the scan shell
    //       (transitional — the current authored-uppers source).
    // Projected costs never enter. Margin applies. Empty → 1.0 fallback.
    let max_anchor_upper = resolution
        .rows
        .iter()
        .filter(|(j, status)| {
            j.domain == DOMAIN_ESTIMATE
                && matches!(j.form, RowForm::Anchor)
                && matches!(
                    status,
                    ResolutionStatus::Active | ResolutionStatus::InertLens
                )
                && j.lens.is_none()
        })
        .filter_map(|(j, _)| j.est_upper)
        .max_by(f64::total_cmp);
    let max_facet_upper = facet_uppers.values().copied().max_by(f64::total_cmp);
    let max_upper = max_anchor_upper
        .into_iter()
        .chain(max_facet_upper)
        .max_by(f64::total_cmp);
    let bare_anchor = match max_upper {
        Some(mu) => mu + margin,
        None => 1.0,
    };
    // The est system's gauge center IS the bare anchor (the pipeline owns
    // it now). Override the caller's est_cfg gauge_center with the
    // computed value — ensures the P8 spread centres correctly.
    let est_cfg = ProjectionCfg {
        gauge_center: bare_anchor,
        ..*est_cfg
    };

    // The pairwise boundary (SL-220 §2, RV-278 F-6): anchor rows terminate
    // at `claims` and reach NO compile consumer — every downstream system
    // (compile, project, elicit's owned evidence) is anchor-free.
    let active: Vec<&Judgement> = resolution
        .rows
        .iter()
        .filter(|(j, status)| {
            matches!(status, ResolutionStatus::Active) && !matches!(j.form, RowForm::Anchor)
        })
        .map(|(j, _)| *j)
        .collect();
    // Compile-input selection (SL-219 §2): each domain compiles ONLY its own
    // rows. The value side keeps every Active non-estimate row — the exact
    // pre-split input minus the est rows (the defect this split fixes: an
    // Active estimate row used to flow into value compilation).
    let (est_active, value_active): (Vec<&Judgement>, Vec<&Judgement>) =
        active.iter().partition(|j| j.domain == DOMAIN_ESTIMATE);
    // SL-222 PHASE-06: est-domain anchors now come from
    // `estimate_claims.anchor_map()` — operative costs of Pin/Human resolved
    // claims — replacing the old authored-facet anchor builder. The caller's
    // `est_anchors` (facet-derived) is no longer the anchor source.
    let est_anchors_from_claims = estimate_claims.anchor_map();
    // Row-gating applies identically: an item anchors the est system only
    // when an est-domain row touches it.
    let gated_est_anchors: AnchorMap = est_anchors_from_claims
        .iter()
        .filter(|(item, _)| {
            est_active
                .iter()
                .any(|j| j.a == **item || j.b.as_deref() == Some(item.as_str()))
        })
        .map(|(item, &v)| (item.clone(), v))
        .collect();

    let value = DomainSystem::compiled(&value_active, &value_anchors, value_cfg);
    let estimate = DomainSystem::compiled(&est_active, &gated_est_anchors, &est_cfg);
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
            let is_anchor = matches!(j.form, RowForm::Anchor);
            // Row-status routing (SL-219 §2): `CompilationStatus` is assigned
            // by the row's OWNING domain system — the quarantine maps are
            // disjoint by construction (each row belongs to exactly one
            // domain); `RowState` joins by row uid. Anchor rows never acquire
            // one (SL-220 §2): their active display token joins from the
            // claims pass instead.
            let compilation =
                (!is_anchor && matches!(status, ResolutionStatus::Active)).then(|| {
                    if j.domain == DOMAIN_ESTIMATE {
                        estimate.constraint_set.status_of(j)
                    } else {
                        value.constraint_set.status_of(j)
                    }
                });
            let claim = (is_anchor && matches!(status, ResolutionStatus::Active))
                .then(|| {
                    if j.domain == DOMAIN_ESTIMATE {
                        claim_token_estimate(&estimate_claims, &j.a)
                    } else {
                        claim_token(&value_claims, &j.a)
                    }
                })
                .flatten();
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
                date: j.ordering_date().to_string(),
                claim,
                state: RowState::new(status.clone(), compilation),
            }
        })
        .collect();

    // Owned clones of the value-domain active evidence for the elicit shell
    // (SL-217 PHASE-03): taken while `resolution` (which the borrows come
    // from) is still alive.
    let active_pairwise: Vec<Judgement> = value_active.iter().map(|&j| j.clone()).collect();

    Ok(Pipeline {
        rows,
        value,
        estimate,
        constraining_by_class,
        est_constraining_by_class,
        malformed: resolution.malformed,
        priority_domain_count,
        value_claims,
        estimate_claims,
        bare_anchor,
        active_pairwise,
    })
}

/// The claim-join display token for an ACTIVE anchor row's item (SL-220
/// design §2, RV-278 F-8): `conflicted` when the item's winning tier
/// disagrees internally, else `anchored` (Pin/Human) / `prior`
/// (Agent/Migrated). Item-level — every active anchor row on the item wears
/// its resolved state; the row's own tier stays visible via `rater_token`.
fn claim_token(claims: &ClaimResolution, item: &str) -> Option<&'static str> {
    if let Some(claim) = claims.anchored.get(item) {
        return Some(if claim.conflict.is_some() {
            "conflicted"
        } else {
            "anchored"
        });
    }
    claims.priors.get(item).map(|claim| {
        if claim.conflict.is_some() {
            "conflicted"
        } else {
            "prior"
        }
    })
}

/// Estimate-domain analog of [`claim_token`]: resolves display tokens for
/// estimate-anchor rows via the estimate claims pass.
fn claim_token_estimate(
    claims: &ClaimResolutionGeneric<EstimatePayload>,
    item: &str,
) -> Option<&'static str> {
    if let Some(claim) = claims.anchored.get(item) {
        return Some(if claim.conflict.is_some() {
            "conflicted"
        } else {
            "anchored"
        });
    }
    claims.priors.get(item).map(|claim| {
        if claim.conflict.is_some() {
            "conflicted"
        } else {
            "prior"
        }
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
        COMPARISON_SCHEMA, COMPARISON_VERSION, ClaimResolution, ComparisonSession, DOMAIN_ESTIMATE,
        DOMAIN_VALUE, FRAME_COST_ANCHOR, FRAME_EQUAL_EFFORT, FRAME_MORE_WORK, FRAME_VALUE_ANCHOR,
        Judgement, QuarantinePolicy, QuarantineReason, RaterKind, Response, RowForm, SessionHeader,
        VALUE_PROJECTION_PARAMS, ValueProvenance, cost_feed, pipeline_from_sessions,
    };
    use crate::priority::config::EST_GAUGE_STEP;
    use std::collections::BTreeMap;

    const CFG: ProjectionCfg = VALUE_PROJECTION_PARAMS;

    /// The default test skew (matches `EstimateCost::default().skew`).
    const EST_SKEW: f64 = 0.65;
    /// The default test margin (matches `EstimateCost::default().margin`).
    const EST_MARGIN: f64 = 1.0;

    /// A convenience empty facet-uppers map for tests that don't exercise
    /// the bare-anchor dimension (behaviour preservation).
    fn empty_facet_uppers() -> BTreeMap<String, f64> {
        BTreeMap::new()
    }

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
            b: Some(b.to_string()),
            response: Some(Response::PreferA),
            domain: DOMAIN_VALUE.to_string(),
            frame: FRAME_EQUAL_EFFORT.to_string(),
            form: RowForm::Order,
            magnitude: None,
            supersedes: None,
            lens: None,
            rater: RaterKind::Human,
            by: None,
            note: None,
            date: Some("2026-07-11".to_string()),
            observed_at: None,
            basis: None,
            est_lower: None,
            est_upper: None,
            admission: None,
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
            &CFG,
            &EST_CFG,
            &empty_facet_uppers(),
            EST_MARGIN,
            EST_SKEW,
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
            &CFG,
            &EST_CFG,
            &empty_facet_uppers(),
            EST_MARGIN,
            EST_SKEW,
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
    /// est anchor map (the VT-1 harness). SL-222 PHASE-06: the old explicit
    /// `est_anchors` map is DEAD — anchors are now claim-derived from
    /// anchor-form rows. This helper ignores the anchor map (the `_` prefix
    /// on the pipeline parameter means the map doesn't reach compile). Callers
    /// that need anchored est behaviour should use `pipeline_with_est_anchors`
    /// instead, or add `est_anchor_row` + a gate row to the judgements vec.
    fn pipeline_of(judgements: Vec<Judgement>, est_anchors: &AnchorMap) -> super::Pipeline {
        let sessions = vec![session("s1", "2026-07-11", judgements)];
        pipeline_from_sessions(
            &sessions,
            &StatusMap::new(),
            est_anchors,
            &CFG,
            &EST_CFG,
            &empty_facet_uppers(),
            EST_MARGIN,
            EST_SKEW,
        )
        .unwrap()
    }

    /// Like `pipeline_of` but converts `est_anchors` entries into actual
    /// anchor-form + gate rows so the est system is properly anchored.
    /// Used by tests that depend on anchored cost behaviour (the PHASE-06
    /// class-b churn fix).
    fn pipeline_with_est_anchors(
        judgements: Vec<Judgement>,
        est_anchors: &AnchorMap,
    ) -> super::Pipeline {
        let mut all = judgements;
        for (i, (item, &cost)) in est_anchors.iter().enumerate() {
            all.push(est_anchor_row(&format!("a-{item}"), item, cost, cost));
            // Gate row: Order-form so it enters est_active (anchor rows
            // are excluded from the compile-active set).
            all.push(Judgement {
                uid: format!("ag-{item}"),
                seq: i as u32 + 100,
                a: item.to_string(),
                b: None,
                response: Some(Response::PreferA),
                domain: DOMAIN_ESTIMATE.to_string(),
                frame: FRAME_MORE_WORK.to_string(),
                form: RowForm::Order,
                magnitude: None,
                supersedes: None,
                lens: None,
                rater: RaterKind::Agent,
                by: None,
                note: None,
                date: Some("2026-07-17".to_string()),
                observed_at: None,
                basis: None,
                est_lower: None,
                est_upper: None,
                admission: None,
            });
        }
        let sessions = vec![session("s1", "2026-07-11", all)];
        pipeline_from_sessions(
            &sessions,
            &StatusMap::new(),
            est_anchors,
            &CFG,
            &EST_CFG,
            &empty_facet_uppers(),
            EST_MARGIN,
            EST_SKEW,
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
        );
        let value_entities: Vec<&String> = pipeline.value.constraint_set.classes.keys().collect();
        assert_eq!(value_entities, ["SL-100", "SL-200"], "value rows only");
        let est_entities: Vec<&String> = pipeline.estimate.constraint_set.classes.keys().collect();
        assert_eq!(est_entities, ["SL-100", "SL-300"], "estimate rows only");
        // The owned value evidence excludes the est row (elicit recompiles
        // the value baseline from this — no cross-domain leak).
        let active_uids: Vec<&str> = pipeline
            .active_pairwise()
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

        // PHASE-06: anchors now come from anchor-form claim rows. Use
        // `pipeline_with_est_anchors` so SL-100 and SL-300 get anchor rows
        // (and gate rows). SL-400's anchor stays excluded because no real
        // est-domain row touches it (its gate row IS an est-domain row, so
        // the claim rows must use the same mechanism).
        // Actually: we only add anchor+gate rows for the two that have
        // real est rows. The cold-start sub-test uses pipeline_of (ignores
        // anchors) to show no est rows ⇒ empty system.
        let pipeline = pipeline_with_est_anchors(
            vec![est_judgement("e1", "SL-300", "SL-100")],
            // Only the items that appear in `e1` get gate rows:
            &[
                ("SL-100".to_string(), cost((1.0, 3.0))),
                ("SL-300".to_string(), cost((5.0, 9.0))),
            ]
            .into_iter()
            .collect(),
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
        // pipeline_of ignores the anchors map (PHASE-06), so this still works.
        let cold = pipeline_of(vec![judgement("v1", "SL-100", "SL-200")], &est_anchors);
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
        equal_row.response = Some(Response::Equal);
        // PHASE-06: use pipeline_with_est_anchors so the anchors become
        // anchor-form claim rows (with gate rows for row-gating).
        let pipeline = pipeline_with_est_anchors(vec![equal_row], &est_anchors);
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
        // PHASE-06: use pipeline_with_est_anchors so anchors come from claim rows.
        let pipeline =
            pipeline_with_est_anchors(vec![est_judgement("e1", "SL-100", "SL-300")], &est_anchors);
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
        // PHASE-06: use pipeline_with_est_anchors so anchors come from claim rows.
        let pipeline = pipeline_with_est_anchors(
            vec![
                judgement("v1", "SL-100", "SL-200"),
                judgement("v2", "SL-200", "SL-500"),
                judgement("v3", "SL-500", "SL-100"),
                est_judgement("e1", "SL-100", "SL-300"),
            ],
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
    /// The in-memory pipeline over one est-domain session with the given cost
    /// anchors, est projection params, facet uppers, and margin (the VT-2
    /// harness; the value system stays empty — the domains are independent by
    /// PHASE-02). Override params are for the bare-anchor test fixtures.
    fn est_pipeline_with_uppers(
        judgements: Vec<Judgement>,
        est_anchors: &AnchorMap,
        est_cfg: &ProjectionCfg,
        facet_uppers: &BTreeMap<String, f64>,
        margin: f64,
    ) -> super::Pipeline {
        let mut all = judgements;
        for (i, (item, &cost)) in est_anchors.iter().enumerate() {
            all.push(est_anchor_row(&format!("a-{item}"), item, cost, cost));
            all.push(Judgement {
                uid: format!("ag-{item}"),
                seq: i as u32 + 100,
                a: item.to_string(),
                b: None,
                response: Some(Response::PreferA),
                domain: DOMAIN_ESTIMATE.to_string(),
                frame: FRAME_MORE_WORK.to_string(),
                form: RowForm::Order,
                magnitude: None,
                supersedes: None,
                lens: None,
                rater: RaterKind::Agent,
                by: None,
                note: None,
                date: Some("2026-07-17".to_string()),
                observed_at: None,
                basis: None,
                est_lower: None,
                est_upper: None,
                admission: None,
            });
        }
        let sessions = vec![session("s1", "2026-07-11", all)];
        pipeline_from_sessions(
            &sessions,
            &StatusMap::new(),
            est_anchors,
            &CFG,
            est_cfg,
            facet_uppers,
            margin,
            EST_SKEW,
        )
        .unwrap()
    }

    /// Build an est pipeline from `judgements` and an old-style `est_anchors`
    /// map. SL-222 PHASE-06: the anchors map is no longer a direct parameter
    /// — anchors are claim-derived from anchor-form rows. We convert each
    /// map entry into an `est_anchor_row` (degenerate lower=upper=operative)
    /// and add a gate row so the claim enters the gated est system.
    fn est_pipeline(
        judgements: Vec<Judgement>,
        est_anchors: &AnchorMap,
        est_cfg: &ProjectionCfg,
    ) -> super::Pipeline {
        let mut all = judgements;
        for (i, (item, &cost)) in est_anchors.iter().enumerate() {
            all.push(est_anchor_row(&format!("a-{item}"), item, cost, cost));
            // Gate row: Order-form so it enters est_active (anchor rows
            // are excluded from the compile-active set).
            all.push(Judgement {
                uid: format!("ag-{item}"),
                seq: i as u32 + 100,
                a: item.to_string(),
                b: None,
                response: Some(Response::PreferA),
                domain: DOMAIN_ESTIMATE.to_string(),
                frame: FRAME_MORE_WORK.to_string(),
                form: RowForm::Order,
                magnitude: None,
                supersedes: None,
                lens: None,
                rater: RaterKind::Agent,
                by: None,
                note: None,
                date: Some("2026-07-17".to_string()),
                observed_at: None,
                basis: None,
                est_lower: None,
                est_upper: None,
                admission: None,
            });
        }
        est_pipeline_with_uppers(all, est_anchors, est_cfg, &empty_facet_uppers(), EST_MARGIN)
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
    /// ABSENT from the cost feed: gauge renders, never divides. The bare
    /// anchor is computed in-pipeline: max over facet uppers + margin.
    #[test]
    fn gauge_component_present_in_projection_absent_from_feed() {
        // Feed facet uppers so the computed bare anchor = 10.0 + 1.0 = 11.0.
        let facet_uppers: BTreeMap<String, f64> =
            [("SL-100".to_string(), 10.0)].into_iter().collect();
        let pipeline = est_pipeline_with_uppers(
            vec![
                est_judgement("e1", "SL-100", "SL-200"),
                est_judgement("e2", "SL-500", "SL-600"),
            ],
            &anchors_of(&[("SL-100", 5.0)]),
            &EST_CFG,
            &facet_uppers,
            EST_MARGIN,
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

    // ---- SL-222 §8.4 bare-anchor test helpers ---------------------------------

    /// An estimate-domain anchor row (cost-anchor frame, est_lower/est_upper).
    /// Builds from a value-style `anchor_row` then overrides the domain-specific
    /// fields so there's no code duplication with the SL-220 helpers above.
    fn est_anchor_row(uid: &str, item: &str, lower: f64, upper: f64) -> Judgement {
        Judgement {
            uid: uid.to_string(),
            seq: 0,
            a: item.to_string(),
            b: None,
            response: None,
            domain: DOMAIN_ESTIMATE.to_string(),
            frame: FRAME_COST_ANCHOR.to_string(),
            form: RowForm::Anchor,
            magnitude: None,
            est_lower: Some(lower),
            est_upper: Some(upper),
            supersedes: None,
            lens: None,
            rater: RaterKind::Human,
            by: None,
            note: None,
            date: Some("2026-07-17".to_string()),
            observed_at: None,
            basis: None,
            admission: None,
        }
    }

    /// Overwrite a judgement's rater to Agent (for losing-tier scenarios).
    fn as_agent(mut j: Judgement) -> Judgement {
        j.rater = RaterKind::Agent;
        j
    }

    /// Add a lens on an anchor row (so resolution marks it `InertLens`).
    fn with_lens(mut j: Judgement, lens: &str) -> Judgement {
        j.lens = Some(lens.to_string());
        j
    }

    /// Build a bare-anchor pipeline via `pipeline_from_sessions` using
    /// an explicit session slices, facet uppers, and margin. Overrides the
    /// est_cfg gauge_center to a dummy — the pipeline overwrites it anyway.
    fn bare_pipeline(
        sessions: Vec<ComparisonSession>,
        facet_uppers: &BTreeMap<String, f64>,
        margin: f64,
    ) -> super::Pipeline {
        pipeline_from_sessions(
            &sessions,
            &StatusMap::new(),
            &AnchorMap::new(),
            &CFG,
            &EST_CFG,
            facet_uppers,
            margin,
            EST_SKEW,
        )
        .unwrap()
    }

    // ---- SL-222 §8.4 bare-anchor test battery --------------------------------

    /// 1. Domination property: with several active estimate anchor rows (mixed
    /// tiers), pipeline.bare_anchor >= every active asserted est_upper + margin.
    #[test]
    fn bare_anchor_dominates_all_active_estimate_uppers() {
        // Three estimate anchor rows on three different items, mixed tiers:
        // human (SL-100, upper=8.0), agent (SL-200, upper=15.0 — losing tier
        // but still contributes its upper), human (SL-300, upper=3.0).
        let pipeline = bare_pipeline(
            vec![session(
                "s1",
                "2026-07-17",
                vec![
                    est_anchor_row("h1", "SL-100", 2.0, 8.0),
                    as_agent(est_anchor_row("a1", "SL-200", 5.0, 15.0)),
                    est_anchor_row("h2", "SL-300", 1.0, 3.0),
                ],
            )],
            &empty_facet_uppers(),
            EST_MARGIN,
        );
        // max_upper = 15.0, margin = 1.0 → bare_anchor = 16.0
        assert!((pipeline.bare_anchor - 16.0).abs() < EPS);
        // Domination: bare_anchor >= every row's est_upper + margin
        for (item, upper) in [("SL-100", 8.0), ("SL-200", 15.0), ("SL-300", 3.0)] {
            assert!(
                pipeline.bare_anchor >= upper + EST_MARGIN,
                "bare_anchor {:.4} < {item} upper {upper:.4} + margin {EST_MARGIN:.4}",
                pipeline.bare_anchor
            );
        }
        // Confirm the agent row's upper IS the max (losing-tier inclusion).
        assert!((pipeline.bare_anchor - 15.0 - EST_MARGIN).abs() < EPS);
    }

    /// 2. Losing-tier inclusion: a row whose tier LOSES resolution (an agent
    /// row beaten by a human row on the same item) still contributes its
    /// est_upper to the bare-anchor fold. The est_upper of the losing tier
    /// is part of the max computation (the bare-anchor filter works on ALL
    /// Active rows directly, NOT the claim resolution — claims fold losing
    /// tiers out, but the bare_anchor max includes them).
    #[test]
    fn bare_anchor_includes_losing_tier_upper() {
        // Same item SL-100: two human anchors (upper=10.0, upper=8.0) and one
        // agent anchor (upper=20.0). All rows must be in SEPARATE sessions
        // (different raters share an identity key otherwise, R3 supersedes).
        // Claim resolution Human-wins with mean operative from humans only.
        // But bare_anchor should still see the agent's 20.0 as max.
        let pipeline = bare_pipeline(
            vec![
                session(
                    "s1",
                    "2026-07-17",
                    vec![est_anchor_row("h1", "SL-100", 1.0, 10.0)],
                ),
                session(
                    "s2",
                    "2026-07-17",
                    vec![est_anchor_row("h2", "SL-100", 2.0, 8.0)],
                ),
                session(
                    "s3",
                    "2026-07-17",
                    vec![as_agent(est_anchor_row("a1", "SL-100", 15.0, 20.0))],
                ),
            ],
            &empty_facet_uppers(),
            EST_MARGIN,
        );
        // max_upper = 20.0 (agent's), + margin = 21.0
        assert!(
            (pipeline.bare_anchor - 21.0).abs() < EPS,
            "bare_anchor={}",
            pipeline.bare_anchor
        );
        // The claim on SL-100 is anchored (Human wins) — the agent row is
        // IGNORED by the claim fold (winning tier = Human, losing tiers
        // contribute nothing — not even to priors). The anchored claim
        // only reflects the two Human rows' mean.
        assert!(pipeline.estimate_claims.anchored.contains_key("SL-100"));
        assert!(
            !pipeline.estimate_claims.priors.contains_key("SL-100"),
            "priors empty: losing-tier Agent rows vanish from the fold"
        );
        let claim = &pipeline.estimate_claims.anchored["SL-100"];
        // The claim's operative is the mean of the two human rows:
        // lower = (1.0+2.0)/2=1.5, upper = (10.0+8.0)/2=9.0,
        // operative = 1.5 + 0.65*(9.0-1.5) = 1.5 + 4.875 = 6.375
        let expected_op = 1.5 + 0.65 * (9.0 - 1.5);
        assert!(
            (claim.operative - expected_op).abs() < 1e-12,
            "operative={}",
            claim.operative
        );
        // The bare anchor (21.0) is NOT the same as the anchored claim's
        // operative (6.375) — evidence that bare_anchor reads ALL rows.
        assert!(
            (pipeline.bare_anchor - claim.operative).abs() > 10.0,
            "bare_anchor and claim operative should differ"
        );
    }

    /// 3. Conflict-row inclusion: individual rows of a same-tier conflict each
    /// contribute their uppers (not just the winning mean). Two human rows on
    /// the same item disagree (upper=20.0, upper=2.0); the max = 20.0.
    /// Rows are in DIFFERENT sessions so R3 doesn't mark one as superseded.
    #[test]
    fn bare_anchor_includes_conflict_individual_uppers() {
        let pipeline = bare_pipeline(
            vec![
                session(
                    "s1",
                    "2026-07-17",
                    vec![est_anchor_row("h1", "SL-100", 5.0, 20.0)],
                ),
                session(
                    "s2",
                    "2026-07-17",
                    vec![est_anchor_row("h2", "SL-100", 1.0, 2.0)],
                ),
            ],
            &empty_facet_uppers(),
            EST_MARGIN,
        );
        // max_upper = 20.0 (h1's), h2's 2.0 doesn't affect max. Bare anchor = 21.0.
        assert!(
            (pipeline.bare_anchor - 21.0).abs() < EPS,
            "bare_anchor={}",
            pipeline.bare_anchor
        );
        // The conflict flag is set (the pipeline resolved Human-tier conflict).
        assert!(
            pipeline.estimate_claims.anchored["SL-100"]
                .conflict
                .is_some(),
            "same-tier disagreement should produce conflict"
        );
    }

    /// 4. Lensed exclusion: a lensed estimate anchor row's upper does NOT enter
    /// the fold. A lensed row resolves `InertLens` which the bare-anchor filter
    /// excludes (the filter requires `.lens.is_none()`).
    #[test]
    fn bare_anchor_excludes_lensed_upper() {
        // Two rows on different items: SL-100 human (upper=8.0), SL-200 lensed
        // human (upper=50.0). The lensed 50.0 must NOT enter the max.
        let pipeline = bare_pipeline(
            vec![session(
                "s1",
                "2026-07-17",
                vec![
                    est_anchor_row("h1", "SL-100", 2.0, 8.0),
                    with_lens(est_anchor_row("l1", "SL-200", 10.0, 50.0), "user-value"),
                ],
            )],
            &empty_facet_uppers(),
            EST_MARGIN,
        );
        // max_upper = 8.0 only (50.0 excluded), + margin = 9.0
        assert!((pipeline.bare_anchor - 9.0).abs() < EPS);
        // The lensed row resolves inert(lens) — confirm via RowSummary.
        let l1 = row_of(&pipeline, "l1");
        assert_eq!(l1.display_token(), "inert(lens)");
    }

    /// 5. Tombstone/supersession reduction: a superseded or tombstoned row's
    /// upper does not enter (resolution already reduced it). The superseding
    /// row's upper DOES enter.
    #[test]
    fn bare_anchor_excludes_superseded_and_tombstoned_uppers() {
        // Four rows:
        //   h1: SL-100, upper=100.0, superseded by h2 (h2.supersedes = "h1")
        //   h2: SL-100, upper=10.0  ← the superseder, ACTIVE
        //   h3: SL-200, upper=50.0, tombstoned (via tombstones list)
        //   h4: SL-300, upper=5.0, ACTIVE
        let mut h2 = est_anchor_row("h2", "SL-100", 1.0, 10.0);
        h2.supersedes = Some("h1".to_string());
        let h1 = est_anchor_row("h1", "SL-100", 50.0, 100.0);
        let h3 = est_anchor_row("h3", "SL-200", 20.0, 50.0);
        let h4 = est_anchor_row("h4", "SL-300", 2.0, 5.0);

        // Tombstone h3 via the session's tombstones list.
        let s = session("s1", "2026-07-17", vec![h1, h2, h3, h4]);
        let s = crate::comparison::ComparisonSession {
            tombstones: vec![crate::comparison::Tombstone {
                uid: "t1".to_string(),
                seq: 0,
                target: "h3".to_string(),
                date: "2026-07-17".to_string(),
                note: None,
            }],
            ..s
        };

        let pipeline = bare_pipeline(vec![s], &empty_facet_uppers(), EST_MARGIN);
        // Active uppers: h2=10.0, h4=5.0. Max = 10.0 + 1.0 = 11.0.
        assert!((pipeline.bare_anchor - 11.0).abs() < EPS);
        // Confirm h1 is superseded, h3 is tombstoned.
        assert_eq!(row_of(&pipeline, "h1").display_token(), "superseded→h2");
        assert_eq!(row_of(&pipeline, "h3").display_token(), "tombstoned");
        // h2 and h4 are active.
        assert_eq!(row_of(&pipeline, "h2").display_token(), "anchored");
        assert_eq!(row_of(&pipeline, "h4").display_token(), "anchored");
    }

    /// 6. Facet-uppers input: the transitional facet-uppers pure input
    /// participates in the max alongside anchor-row uppers.
    #[test]
    fn bare_anchor_facet_uppers_join_anchor_uppers() {
        let facet_uppers: BTreeMap<String, f64> =
            [("SL-100".to_string(), 30.0)].into_iter().collect();
        // Anchor upper = 8.0, facet upper = 30.0. Max = 30.0 + 1.0 = 31.0.
        let pipeline = bare_pipeline(
            vec![session(
                "s1",
                "2026-07-17",
                vec![est_anchor_row("h1", "SL-100", 2.0, 8.0)],
            )],
            &facet_uppers,
            EST_MARGIN,
        );
        assert!((pipeline.bare_anchor - 31.0).abs() < EPS);

        // Reverse: anchor upper dominates.
        let facet_low: BTreeMap<String, f64> = [("SL-100".to_string(), 2.0)].into_iter().collect();
        let pipeline2 = bare_pipeline(
            vec![session(
                "s1",
                "2026-07-17",
                vec![est_anchor_row("h1", "SL-100", 2.0, 50.0)],
            )],
            &facet_low,
            EST_MARGIN,
        );
        assert!((pipeline2.bare_anchor - 51.0).abs() < EPS);
    }

    /// 7. Empty corpus: no anchor rows, no facet uppers -> bare_anchor == 1.0
    /// fallback (margin semantics preserved as the previous computation had).
    #[test]
    fn bare_anchor_empty_corpus_falls_back_to_one() {
        let pipeline = bare_pipeline(vec![], &empty_facet_uppers(), EST_MARGIN);
        assert!((pipeline.bare_anchor - 1.0).abs() < EPS);

        // Even with a session with no estimate anchor rows, no facets:
        let pipeline2 = bare_pipeline(
            vec![session(
                "s1",
                "2026-07-17",
                vec![judgement("j1", "SL-100", "SL-200")],
            )],
            &empty_facet_uppers(),
            EST_MARGIN,
        );
        assert!((pipeline2.bare_anchor - 1.0).abs() < EPS);
    }

    /// 8. Pinned one-site equality: `bare_anchor` becomes the est system's
    /// `gauge_center`, which means an anchor-free component (P8) is spread
    /// centred on `bare_anchor`. Add an anchor-free pair ordered by an est
    /// judgement and verify the midpoint ≈ `bare_anchor`.
    #[test]
    fn bare_anchor_equals_absent_and_gauge_center() {
        // Four estimate anchor rows on different items (max_upper=25.0) plus
        // an anchor-free pair (SL-500, SL-600) connected by an est judgement.
        let pipeline = bare_pipeline(
            vec![session(
                "s1",
                "2026-07-17",
                vec![
                    est_anchor_row("h1", "SL-100", 2.0, 8.0),
                    est_anchor_row("h2", "SL-200", 3.0, 12.0),
                    as_agent(est_anchor_row("a1", "SL-300", 10.0, 25.0)),
                    est_anchor_row("h3", "SL-400", 1.0, 5.0),
                    // Anchor-free pair — no anchor touches SL-500/SL-600.
                    est_judgement("e1", "SL-500", "SL-600"),
                ],
            )],
            &empty_facet_uppers(),
            EST_MARGIN,
        );
        // max_upper = 25.0, + 1.0 = 26.0
        let expected = 26.0;
        assert!((pipeline.bare_anchor - expected).abs() < EPS);
        // P8 gauge spread centres on bare_anchor. With gauge_step=0.25 (the
        // EST_GAUGE_STEP shipped value), two anchor-free nodes spread 1 step
        // apart with midpoint = gauge_center.
        let (hi_got, hi_prov) = pipeline.estimate.projection["SL-500"];
        let (lo_got, lo_prov) = pipeline.estimate.projection["SL-600"];
        assert_eq!(hi_prov, Gauge);
        assert_eq!(lo_prov, Gauge);
        let midpoint = (hi_got + lo_got) / 2.0;
        assert!(
            (midpoint - expected).abs() < EPS,
            "gauge midpoint {midpoint:.4} ≠ bare_anchor {expected:.4}"
        );
    }

    /// 9. Behaviour preservation pin: facets but zero estimate anchor rows ->
    /// bare_anchor equals the old facet-derived computation (pin the expected
    /// value explicitly in the test).
    #[test]
    fn bare_anchor_facet_only_falls_back_to_facet_upper() {
        let facet_uppers: BTreeMap<String, f64> =
            [("ISS-001".to_string(), 12.0), ("ISS-002".to_string(), 7.0)]
                .into_iter()
                .collect();
        // No estimate anchor rows at all, only value pairwise rows.
        let pipeline = bare_pipeline(
            vec![session(
                "s1",
                "2026-07-17",
                vec![judgement("j1", "SL-100", "SL-200")],
            )],
            &facet_uppers,
            EST_MARGIN,
        );
        // Old facet-derived computation: max facet upper (12.0) + margin (1.0) = 13.0
        assert!((pipeline.bare_anchor - 13.0).abs() < EPS);

        // No facets, no anchor rows: 1.0 fallback.
        let pipeline2 = bare_pipeline(
            vec![session(
                "s1",
                "2026-07-17",
                vec![judgement("j1", "SL-100", "SL-200")],
            )],
            &empty_facet_uppers(),
            EST_MARGIN,
        );
        assert!((pipeline2.bare_anchor - 1.0).abs() < EPS);
    }

    // ---- SL-220 PHASE-03: the claims pass wiring (design §2) -----------------

    /// A live human value-anchor row (SL-220 §1): single subject, no pairwise
    /// payload, `value-anchor` frame, magnitude payload.
    fn anchor_row(uid: &str, item: &str, magnitude: f64) -> Judgement {
        let mut j = judgement(uid, item, "unused");
        j.b = None;
        j.response = None;
        j.form = RowForm::Anchor;
        j.frame = FRAME_VALUE_ANCHOR.to_string();
        j.magnitude = Some(magnitude);
        j
    }

    fn row_of<'a>(pipeline: &'a super::Pipeline, uid: &str) -> &'a super::RowSummary {
        pipeline
            .rows
            .iter()
            .find(|r| r.uid == uid)
            .unwrap_or_else(|| panic!("row {uid} present"))
    }

    /// The claims pass is wired (design §2) and anchor rows terminate at it:
    /// they reach NO compile consumer AS ROWS (RV-278 F-6 — the value system
    /// equals a direct compile over ONLY the pairwise rows), the pairwise
    /// view excludes them, and their RowSummary joins the claim token instead
    /// of any `CompilationStatus`. Since the PHASE-05 flip their RESOLVED
    /// Pin/Human magnitudes DO feed compile — as the claim-derived
    /// `AnchorMap` (D4/D12), never as rows; agent claims stay laundering-proof
    /// (absent from the compiled anchors).
    #[test]
    fn anchor_rows_terminate_at_claims_and_never_reach_compile() {
        let mut agent_anchor = anchor_row("a1", "SL-300", 2.0);
        agent_anchor.rater = RaterKind::Agent;
        let with_anchors = pipeline_of(
            vec![
                judgement("j1", "SL-100", "SL-200"),
                anchor_row("h1", "SL-100", 6.0),
                agent_anchor,
            ],
            &AnchorMap::new(),
        );

        // The claim resolution rides the Pipeline, routed by tier (D4).
        assert_eq!(with_anchors.value_claims.anchored["SL-100"].operative, 6.0);
        assert_eq!(with_anchors.value_claims.priors["SL-300"].operative, 2.0);
        // The pairwise view is anchor-free (the SL-220 split).
        let pairwise: Vec<&str> = with_anchors
            .active_pairwise()
            .iter()
            .map(|j| j.uid.as_str())
            .collect();
        assert_eq!(pairwise, ["j1"]);
        // F-6, store path: the compiled value system is EXACTLY a direct
        // compile over the pairwise rows + the claim-derived anchor map —
        // anchor rows contributed no compile ROW.
        let claim_anchors = with_anchors.value_claims.anchor_map();
        assert_eq!(
            claim_anchors.get("SL-100"),
            Some(&6.0),
            "the human claim anchors the compile (the flip, D12)"
        );
        assert!(
            !claim_anchors.contains_key("SL-300"),
            "the agent claim never enters the anchor map (D4)"
        );
        assert_eq!(with_anchors.value.anchors, claim_anchors);
        let pairwise_refs: Vec<&Judgement> = with_anchors.active_pairwise().iter().collect();
        let direct =
            super::compile::compile(&pairwise_refs, &claim_anchors, QuarantinePolicy::Symmetric);
        assert_eq!(with_anchors.value.constraint_set, direct);
        // RowSummary join: claim tokens, never a CompilationStatus.
        let h1 = row_of(&with_anchors, "h1");
        assert_eq!(h1.b, None);
        assert_eq!(h1.response, None);
        assert_eq!(h1.state.compilation, None);
        assert_eq!(h1.claim, Some("anchored"));
        assert_eq!(h1.display_token(), "anchored");
        let a1 = row_of(&with_anchors, "a1");
        assert_eq!(a1.state.compilation, None);
        assert_eq!(a1.display_token(), "prior");
        // The pairwise row keeps the RowState join untouched.
        assert_eq!(row_of(&with_anchors, "j1").display_token(), "active");
    }

    /// Claim display tokens beyond the happy path: cross-session same-tier
    /// disagreement renders `conflicted` on every contributing row; a lensed
    /// anchor row keeps its RESOLUTION token (`inert(lens)` — the claim
    /// tokens are Active-only); a superseded anchor row keeps its
    /// supersession token.
    #[test]
    fn anchor_row_tokens_cover_conflict_lens_and_supersession() {
        let mut lensed = anchor_row("l1", "SL-100", 9.0);
        lensed.lens = Some("user-value".to_string());
        let s1 = session(
            "s1",
            "2026-07-11",
            vec![
                anchor_row("p", "SL-100", 4.0),
                anchor_row("q", "SL-100", 8.0), // R3-revises `p` (same identity)
                lensed,
            ],
        );
        let s2 = session("s2", "2026-07-11", vec![anchor_row("r", "SL-100", 2.0)]);
        let pipeline = pipeline_from_sessions(
            &[s1, s2],
            &StatusMap::new(),
            &AnchorMap::new(),
            &CFG,
            &EST_CFG,
            &empty_facet_uppers(),
            EST_MARGIN,
            EST_SKEW,
        )
        .unwrap();
        // q (8.0) and r (2.0) are concurrent → conflicted, mean 5.0.
        let claim = &pipeline.value_claims.anchored["SL-100"];
        assert_eq!(claim.operative, 5.0);
        assert!(claim.conflict.is_some());
        assert_eq!(row_of(&pipeline, "q").display_token(), "conflicted");
        assert_eq!(row_of(&pipeline, "r").display_token(), "conflicted");
        // Non-Active anchor rows keep their resolution tokens.
        assert_eq!(row_of(&pipeline, "p").display_token(), "superseded→q");
        assert_eq!(row_of(&pipeline, "l1").claim, None);
        assert_eq!(row_of(&pipeline, "l1").display_token(), "inert(lens)");
        // …and the lensed partition still resolved (RV-278 F-2).
        let key = ("user-value".to_string(), "SL-100".to_string());
        assert_eq!(pipeline.value_claims.lensed[&key].operative, 9.0);
    }

    /// The empty-claims property (design §8.5, pipeline level): a corpus with
    /// no anchor rows carries an empty ClaimResolution, an EMPTY compiled
    /// anchor map (claims are the only value-anchor source post-flip), and a
    /// value system bitwise-identical to a direct anchor-free compile+project
    /// over the same rows — the claims pass perturbs nothing.
    #[test]
    fn corpus_without_anchor_rows_scores_identically_with_empty_claims() {
        let rows = vec![
            judgement("j1", "SL-100", "SL-200"),
            judgement("j2", "SL-200", "SL-300"),
        ];
        let pipeline = pipeline_of(rows, &AnchorMap::new());
        assert_eq!(pipeline.value_claims, ClaimResolution::default());
        assert!(pipeline.value.anchors.is_empty(), "no claims ⇒ no anchors");

        let rows = [
            judgement("j1", "SL-100", "SL-200"),
            judgement("j2", "SL-200", "SL-300"),
        ];
        let refs: Vec<&Judgement> = rows.iter().collect();
        let direct = super::compile::compile(&refs, &AnchorMap::new(), QuarantinePolicy::Symmetric);
        assert_eq!(pipeline.value.constraint_set, direct);
        assert_eq!(
            pipeline.value.projection,
            super::project::project(&direct, &CFG)
        );
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
                            eq.response = Some(Response::Equal);
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
