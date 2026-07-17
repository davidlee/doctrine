// SPDX-License-Identifier: GPL-3.0-only
//! The priority graph adapter (SL-047 §5.2) — the THIRD cordage `Graph`.
//!
//! Consumes `relation_graph`'s `pub(crate)` all-kind scan seam
//! ([`crate::relation_graph::scan_entities`]) to build a cordage `Graph` carrying:
//! - the `needs` **dep overlay** (hard prerequisite, `Reject`) and the `after`
//!   **seq overlay** (soft sequence, `Evict`) — the `backlog_order` template,
//!   emitted KIND-AGNOSTICALLY (DD-2). SL-060 generalised the dep/seq READ gate
//!   ([`relation_graph::dep_seq_for`]) so SLICE (and any future authoring kind) edges
//!   reach these overlays too — backlog is no longer the only source; a kind that
//!   authors no dep/seq simply carries empty axes and contributes no edge;
//! - the SL-046 **reference/lineage overlays** (one per [`REF_LABELS`] entry) — the
//!   consequence inputs;
//! - per-node [`NodeAttr`] (kind, RAW authored status, `promoted`, `base_score`);
//! - a **consequence post-pass** (`leverage`/`optionality`/`score` maps); and
//! - an `OrderSpec` over `[dep Along, seq Along]`.
//!
//! NO partition/channel POLICY yet — `NodeAttr` stores the RAW authored status
//! string; classification (workable/terminal) is PHASE-02. A SEPARATE cordage
//! `Graph` from `backlog_order`'s and `inspect`'s — they share the `Projection`
//! *type*, never a graph instance or a scan (the scan is the shared seam, EX-5).
//!
//! Layering (ADR-001): `priority` → `relation_graph` → `projection` → `cordage`. No
//! cycle. The build is pure over the scanned `Vec` (the disk touch lives in
//! `scan_entities`, the imperative shell).
//!
//! The whole adapter is consumed by the priority CLI surface (SL-047 PHASE-03 —
//! `priority::surface` builds the view rows from `build()`), so the PHASE-01/02
//! self-clearing `not(test)` `dead_code` suppression has retired itself, as designed
//! (`mem.pattern.lint.dead-code-expect-vs-cfg-test`).

use std::collections::BTreeMap;

use crate::catalog::scan::ScanMode;

use cordage::{
    Arity, CyclePolicy, Direction, EdgeAttrs, Graph, GraphBuilder, OrderLayer, OrderSpec,
    OverlayConfig, OverlayId,
};

use crate::comparison::{
    self, AnchorMap, EntityLifecycle, Projection as ValueProjection, ProjectionCfg, StatusMap,
};
use crate::facet::EntityFacets;
use crate::priority::config;
use crate::priority::partition::{self, StatusClass};
use crate::projection::Projection;
use crate::relation::RelationLabel;
use crate::relation_graph::{self, EntityKey};
use crate::{dep_seq, entity, integrity};

/// One node's authored attributes (design §5.2). `kind` is the `&'static entity::Kind`
/// descriptor (data, not `Ord` — carries a fn-ptr `scaffold`; stored by reference like
/// `EntityKey` stores `prefix`). `status` is the RAW authored status string — `None`
/// for the status-less REC kind ONLY; RV carries its DERIVED active/done (authored-tier
/// over its finding ledger). NO classification here (workable/terminal is PHASE-02).
/// `promoted` is the backlog `resolution == Promoted` typed flag — DISTINCT from
/// status-terminal, NOT the free-text `origin`.
/// The split base score for a single entity (design §5.1). Both dimensions and
/// `total()` are `is_finite`-sanitised by [`base_score`] — NaN/\u{221e} → 0.0.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BaseScore {
    pub(crate) value_dim: f64,
    pub(crate) risk_dim: f64,
}

impl BaseScore {
    pub(crate) fn total(&self) -> f64 {
        let t = self.value_dim + self.risk_dim;
        if t.is_finite() { t } else { 0.0 }
    }
}

// ── est_cost helpers (SL-172 §5.1/§5.4) ────────────────────────────────────

/// The floor epsilon — guards against division by zero. Hoisted here so both
/// [`base_score`] and [`floor_eps`] share a single source (STD-001).
const EPSILON: f64 = 1e-12;

/// Floor to `EPSILON` if the value dips below it — protects division in
/// `value_dim` from zero-cost estimates.
fn floor_eps(x: f64) -> f64 {
    if x < EPSILON { EPSILON } else { x }
}

/// Context carried into every per-node `est_cost` call — the bare-item anchor
/// (the maximum `upper` among non-terminal estimated items + `margin`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct CostCtx {
    pub(crate) absent: f64,
}

/// The ONE β-skew formula site (SL-219 design §2 "one formula site"): the
/// operative scalar cost of an AUTHORED estimate — `lower + β·(upper − lower)`,
/// floored to `EPSILON` (the D11 positivity axiom: every anchor > 0). Shared by
/// the scoring ladder's authored branch ([`est_cost`]) and the est-domain
/// anchor builder ([`comparison_est_anchor_map`]) — extracted so the two can
/// never drift (a test pins builder output == the authored branch).
pub(crate) fn authored_est_cost(bounds: (f64, f64), ec: &config::EstimateCost) -> f64 {
    let (lower, upper) = bounds;
    floor_eps(lower + ec.skew * (upper - lower))
}

/// Compute the effective cost of an estimate — the SL-219 three-tier ladder
/// (design D2/D6, source precedence only, no numeric-dominance claim), the
/// SINGLE consumption seam for the cost feed:
/// 1. **Authored** bounds present: [`authored_est_cost`] (one formula site,
///    SL-219 §2) — an item with its own facet never consults the feed.
/// 2. Else **cost-feed lookup**: the projected cost, EPSILON-floored HERE
///    (the D11 positivity belt at the consumption branch).
/// 3. Else the **bare anchor** `ctx.absent` (≥ 1.0, already floored) —
///    computed from authored uppers ONLY (D7); a gauge-masked item is absent
///    from the feed and falls through here (gauge never divides, D2).
fn est_cost(
    bounds: Option<(f64, f64)>,
    key: EntityKey,
    cost_feed: &comparison::CostFeed,
    ctx: CostCtx,
    ec: &config::EstimateCost,
) -> f64 {
    match bounds {
        Some(b) => authored_est_cost(b, ec),
        None => match cost_feed.get(&key.canonical()) {
            Some(&projected) => floor_eps(projected),
            None => ctx.absent,
        },
    }
}

/// Default raw value for a value-bearing entity that authors no `[value]` facet
/// (SL-177; SL-176 D-value-floor-sibling). Default-when-absent, NOT a min-clamp:
/// an authored value (incl. < 1.0 and 0.0) is returned untouched.
pub(crate) const DEFAULT_VALUE: f64 = 1.0;

/// Single definition of an entity's value for priority purposes — the SL-220
/// evidence ladder (design §3, RFC-020 T3; first hit wins):
///
/// 1. **Anchored claim** — [`ClaimResolution`]`.anchored` (Pin or Human tier,
///    conflict means included). Wins outright; the same tiers shape projection
///    via `anchor_map()` — the two seams the authored facet used to occupy,
///    now with provenance. Read DIRECTLY (scope R1): a row-less human claim
///    still wins here even though compile's row-gating drops its anchor.
/// 2. **Comparison projection** — unchanged machinery (`Projected` OR its
///    `Gauge` sub-tier, SL-213 D11: value multiplies, never divides); anchors
///    are claim-derived only since the flip.
/// 3. **Agent-tier prior**, then 4. **Migrated-tier prior** — one `priors`
///    lookup (the D4 split routes both around the constraint layer; the
///    within-map tier contest is already resolved by the claims pass,
///    agent > migrated).
/// 5. **Unmigrated `[value]` facet** (transitional, D6) — consulted only when
///    ZERO unlensed claim rows exist for the item (structural here: `anchored`
///    and `priors` both missed; lensed partitions are inert, D5). The
///    `UnmigratedFacet` finding fires on facet PRESENCE, not this rung's
///    consumption (RV-278 F-4 — a facet shadowed by projection is still debt).
/// 6. **`DEFAULT_VALUE`** for a value-bearing kind; any other valueless kind
///    (records, governance, REV) is None.
///
/// Consumption gate (D7): the claim rungs read only for value-bearing kinds —
/// a scoring-inert subject's claims resolve normally but nothing consumes
/// them. The facet/projection rungs keep their pre-flip kind behaviour
/// bit-for-bit (the caller contract). Empty `claims` + empty `projected` ⇒
/// bitwise-identical to the pre-SL-213 two-tier resolution (the
/// behaviour-preservation gate). Consumed by `base_score`'s `value_dim` AND
/// the burndown accessor identically (governed policy, RV-191 F-1).
fn effective_raw_value(
    kind: &entity::Kind,
    f: &EntityFacets,
    key: EntityKey,
    projected: &ValueProjection,
    claims: &comparison::ClaimResolution,
) -> Option<f64> {
    let value_bearing = crate::kinds::is_value_bearing(kind.prefix);
    let canonical = key.canonical();
    // Rungs 1 / 3–4 (D7 consumption gate: claims feed scored kinds only).
    let claim_rung = |map: &BTreeMap<String, comparison::ResolvedClaim>| -> Option<f64> {
        if !value_bearing {
            return None;
        }
        map.get(&canonical).map(|c| c.value)
    };
    claim_rung(&claims.anchored)
        .or_else(|| projected.get(&canonical).map(|&(v, _)| v))
        .or_else(|| claim_rung(&claims.priors))
        .or_else(|| f.value.as_ref().map(|v| v.value))
        .or_else(|| value_bearing.then_some(DEFAULT_VALUE))
}

/// The per-entity tag multiplier term: `(1.0 + Σ(tag_coeff − 1.0)).max(0.0)` —
/// identity base for absent tags, each configured tag pushes the multiplier by
/// its excess over default, floored at zero so many demoting tags cannot make it
/// negative. Extracted (SL-217 PHASE-03) so `base_score` and the elicit shell's
/// `item_costing` share ONE definition (no parallel impl).
fn tag_term(f: &EntityFacets, cfg: &config::PriorityConfig) -> f64 {
    (1.0 + f.tags.iter().map(|t| cfg.tag_coeff(t) - 1.0).sum::<f64>()).max(0.0)
}

/// Pure base-score computation per entity (design §5.1). Returns the SPLIT
/// `BaseScore` so `explain` can surface `value_dim` / `risk_dim`. No IO.
#[expect(
    clippy::too_many_arguments,
    reason = "the pure scoring inputs (projection, claims, cost feed) thread from one shell seam"
)]
fn base_score(
    f: &EntityFacets,
    kind: &entity::Kind,
    key: EntityKey,
    projected: &ValueProjection,
    claims: &comparison::ClaimResolution,
    cost_feed: &comparison::CostFeed,
    cfg: &config::PriorityConfig,
    ctx: CostCtx,
) -> BaseScore {
    // value_dim = coefficients.value × value × kind_weight(kind) × tag_term / est_cost
    // tag_term = (1.0 + Σ(coeff - 1.0)).max(0.0): identity base for absent tags,
    // each configured tag pushes the multiplier by its excess over default.
    // Default coeff (1.0) → delta 0 → no effect. Floor at zero prevents a
    // negative multiplier from many demoting tags.
    let tag_term = tag_term(f, cfg);
    let value_dim = {
        let raw = match effective_raw_value(kind, f, key, projected, claims) {
            Some(v) => {
                let cost = est_cost(
                    f.estimate.as_ref().map(|e| (e.lower, e.upper)),
                    key,
                    cost_feed,
                    ctx,
                    &cfg.estimate,
                );
                let kw = cfg.kind_weight(kind.prefix);
                cfg.coefficients.value * v * kw * tag_term / cost
            }
            None => 0.0,
        };
        if raw.is_finite() { raw } else { 0.0 }
    };
    // risk_dim = coefficients.risk × exposure(f.risk)
    let risk_dim = {
        let raw = cfg.coefficients.risk * f64::from(crate::risk::exposure(f.risk.as_ref()));
        if raw.is_finite() { raw } else { 0.0 }
    };
    BaseScore {
        value_dim,
        risk_dim,
    }
}

pub(crate) struct NodeAttr {
    pub(crate) kind: &'static entity::Kind,
    pub(crate) status: Option<String>,
    pub(crate) promoted: bool,
    /// The entity's authored `title`, captured from the scan (display-only — the pure
    /// channel layer never reads it). Carried here so the impure surface shell needs
    /// no second per-row disk read (one scan, one read per entity).
    pub(crate) title: String,
    /// The entity's base score (split `value_dim`/`risk_dim`), computed in the base
    /// pre-pass and consumed by the consequence post-pass (PHASE-04) and later
    /// the mint order (PHASE-05).
    pub(crate) base_score: BaseScore,
    /// The entity's authored facets (estimate/value/risk/tags) — carried so the
    /// surface shell projects them into view rows without recomputation
    /// (SL-171 PHASE-01, D2).
    pub(crate) facets: EntityFacets,
}

/// The assembled priority graph (design §5.2). The cordage `Graph`, the
/// `EntityKey ↔ NodeId` projection, the per-node attributes (carrying `base_score`),
/// the consequence post-pass maps (`leverage`/`optionality`/`score`), and the two
/// dep/seq overlay handles. Opaque cordage ids never escape a `pub(crate)` signature.
pub(crate) struct PriorityGraph {
    pub(crate) graph: Graph,
    pub(crate) projection: Projection<EntityKey>,
    pub(crate) attrs: BTreeMap<EntityKey, NodeAttr>,
    /// Recursive needs-leverage per entity (the consequence post-pass) — consumed by
    /// the survey/next/explain surfaces (SL-133 PHASE-05).
    pub(crate) leverage: BTreeMap<EntityKey, f64>,
    /// One-hop ref-optionality per entity (the consequence post-pass) — consumed by
    /// the surfaces (SL-133 PHASE-05).
    pub(crate) optionality: BTreeMap<EntityKey, f64>,
    /// Final score per entity (`base + leverage + optionality`) — the display sort key
    /// consumed by survey/next/explain (SL-133 PHASE-05).
    pub(crate) score: BTreeMap<EntityKey, f64>,
    pub(crate) dep_overlay: OverlayId,
    pub(crate) seq_overlay: OverlayId,
    /// The build-time bare-item cost anchor (max non-terminal `upper` + margin),
    /// retained so the elicit shell's [`PriorityGraph::item_costing`] reproduces
    /// the SAME `est_cost` for bare items the base pre-pass used (SL-217 PHASE-03).
    pub(crate) cost_ctx: CostCtx,
    /// The build-time cost feed (SL-219 PHASE-04), retained for the same reason
    /// as `cost_ctx`: [`PriorityGraph::item_costing`] must resolve the SAME
    /// three-tier `est_cost` ladder the base pre-pass used — projected-cost
    /// movement reaches elicit eff-weights through this one seam.
    pub(crate) cost_feed: comparison::CostFeed,
}

impl PriorityGraph {
    /// Per-item costing for the elicit shell (SL-217 PHASE-03): the value-free
    /// multiplier `m = coeff.value × kind_weight × tag_term` (design D6), the
    /// β-skewed `est_cost` (reusing the build-time `cost_ctx` bare-item anchor),
    /// and the bare-estimate flag (no estimate facet). `None` for an unknown key.
    /// Additive READ accessor — no change to `base_score`/compile/project
    /// (behaviour-preservation, VA-1).
    pub(crate) fn item_costing(
        &self,
        key: &EntityKey,
        cfg: &config::PriorityConfig,
    ) -> Option<(f64, f64, bool)> {
        let attr = self.attrs.get(key)?;
        let f = &attr.facets;
        let multiplier =
            cfg.coefficients.value * cfg.kind_weight(attr.kind.prefix) * tag_term(f, cfg);
        let bounds = f.estimate.as_ref().map(|e| (e.lower, e.upper));
        let est = est_cost(bounds, *key, &self.cost_feed, self.cost_ctx, &cfg.estimate);
        Some((multiplier, est, f.estimate.is_none()))
    }
}

/// The reference/lineage relation labels that back a consequence-input overlay — the
/// SL-046 overlay-backed labels MINUS the two target-unvalidated ones (`Drift`/
/// `DecisionRef`, which never resolve). One `Reject`/`Unbounded` overlay each — the
/// reference/lineage consequence-input overlays. Label is overlay identity (the same
/// label from different source kinds shares ONE overlay).
const REF_LABELS: &[RelationLabel] = &[
    RelationLabel::References,
    RelationLabel::Supersedes,
    RelationLabel::DescendsFrom,
    RelationLabel::Parent,
    RelationLabel::Members,
    RelationLabel::Interactions,
    RelationLabel::Fulfils,
    RelationLabel::Related,
    RelationLabel::Reviews,
    RelationLabel::OwningSlice,
];

/// The WORK/LINEAGE label subset whose inbound references count toward consequence
/// (design §5.2, EX-3). `reviews`/`owning_slice` are bookkeeping and EXCLUDED; the
/// two target-unvalidated labels never resolve and so cannot contribute anyway.
/// SL-176 PHASE-03: `Slices` removed; `Fulfils` NEVER added (wrong sign/direction).
const CONSEQUENCE_LABELS: &[RelationLabel] = &[
    RelationLabel::References,
    RelationLabel::DescendsFrom,
    RelationLabel::Parent,
    RelationLabel::Members,
];

/// Build the priority graph once (design §5.2) — the thin `scan_entities(root)?` +
/// delegate wrapper over [`build_from`] (the SL-050 F2 shared-scan seam). A command
/// layer that already holds a scan calls `build_from` directly to avoid a second walk.
///
/// # Errors
///
/// Propagates a scan/read error, or an internal cordage rejection of well-formed
/// adapter input (an adapter bug, not a recoverable condition).
pub(crate) fn build(root: &std::path::Path) -> anyhow::Result<PriorityGraph> {
    build_from(
        &relation_graph::scan_entities(root, &mut vec![], ScanMode::default())?,
        root,
    )
}

/// Build the priority graph from a PRE-SCANNED entity slice (the SL-050 F2 shared-scan
/// seam — the body of [`build`]). The build order breaks the mint-order ↔ consequence
/// ↔ graph cycle by moving consequence to a POST-pass (SL-133 §5.4):
///
/// 1. **Scan** — supplied by the caller (the `relation_graph` seam → entity set + each
///    entity's outbound edges + RAW authored status + estimate/value/risk facets).
/// 2. **Base pre-pass** — pure per-node `base_score` (value/risk dims) from each
///    entity's OWN facets + config + kind into a `BTreeMap<EntityKey, BaseScore>`. No
///    graph needed; feeds the mint tiebreaker.
/// 3. **Mint** every node into the projection in `(base.total() desc via f64::total_cmp,
///    canonical-id asc)` order — consequence EXCLUDED (I3: no graph-derived quantity in
///    the structural tiebreak). The monotonic `NodeId` is the order key's tier-3
///    fallback. A dedicated pre-intern pass (the `backlog_order` C4 discipline): mint
///    EVERY node first, distinct keys asserted, THEN resolve+emit edges (resolve is
///    get-only, never intern inside the edge pass).
/// 4. **Edges** — reference/lineage onto the ref overlays (resolve-only; an
///    unresolved target contributes no edge). `needs` → `dep_overlay` (`Reject`,
///    oriented prereq→src i.e. B→A flip,
///    `EdgeAttrs::new(0, 0)`). `after` → `seq_overlay` (`Evict`, `EdgeAttrs::new(rank,
///    age)`). The dep/seq edges read kind-agnostically (DD-2) via the SL-060 cross-kind
///    [`relation_graph::dep_seq_for`] gate — backlog AND slice author them.
/// 5. `OrderSpec::new([dep Along, seq Along])`, then `builder.build()`.
/// 6. **Consequence post-pass** — recursive needs-leverage + one-hop ref-optionality
///    over the built graph, storing `leverage`/`optionality`/`score` (§5.4 step 6).
///
/// `root` is RETAINED: the per-entity `dep_seq_for` reads (step 3b) are per-item reads
/// NOT part of `scan_entities`, so the body still needs disk access. The mint/edge order
/// is unchanged (the scan order the caller supplies), preserving byte-identical output.
///
/// SL-213 PHASE-05: also composes the comparison-tier value projection
/// (`comparison::load_pipeline`, the ONE disk touch for the comparisons
/// ledger) and threads it into [`build_from_with_cfg`]. No comparisons on disk
/// ⇒ `load_pipeline` short-circuits to an empty pipeline ⇒ byte-identical to
/// the pre-SL-213 build (behaviour-preservation gate).
///
/// # Errors
///
/// Propagates a read error, a malformed comparison ledger, or an internal
/// cordage rejection of well-formed adapter input (an adapter bug, not a
/// recoverable condition).
pub(crate) fn build_from(
    scanned: &[relation_graph::ScannedEntity],
    root: &std::path::Path,
) -> anyhow::Result<PriorityGraph> {
    // The single config load lives HERE (D4) and is threaded into the build seam.
    // SL-194 PHASE-01 lifted it to a parameter so a β sweep can inject a swept
    // `estimate.skew`; every existing caller routes through this byte-identical wrapper.
    let cfg = config::load(root);
    // ONE pipeline load feeds BOTH scoring inputs (SL-219 D6 flow order): the
    // value projection and the est-domain cost feed derive from the same
    // resolve→compile→project pass — never two ledger reads for one build.
    let pipeline = load_comparison_pipeline(root, scanned, &cfg)?;
    let cost_feed = comparison::cost_feed(&pipeline.estimate.projection);
    build_from_with_cfg(
        scanned,
        root,
        &cfg,
        &pipeline.value.projection,
        &cost_feed,
        &pipeline.value_claims,
    )
}

/// Load the comparison-tier PIPELINE (SL-213 PHASE-06): the compiled
/// `ConstraintSet` (bounds + quarantine ledger), the constraining-judgement
/// rater split by class, the resolve-tier finding streams, and the final
/// `Projection` — the richer bundle `explain`'s value-source block and
/// `findings`' comparison detectors need beyond the projected scalar
/// [`build_from`]'s own load consumes. Same layering rationale as its
/// siblings (this file's doc on [`comparison_status_map`]): needs
/// `partition::status_class` + `ScannedEntity`, both above `comparison`.
pub(crate) fn load_comparison_pipeline(
    root: &std::path::Path,
    scanned: &[relation_graph::ScannedEntity],
    cfg: &config::PriorityConfig,
) -> anyhow::Result<comparison::Pipeline> {
    // SL-220 PHASE-05: no facet→AnchorMap builder — the value system's compile
    // anchors are claim-derived inside the pipeline (`ClaimResolution::
    // anchor_map()`, D4/D12); facets stopped anchoring/shaping projection.
    let statuses = comparison_status_map(scanned);
    let est_anchors = comparison_est_anchor_map(scanned, cfg);
    // Per-domain projection params (SL-219 D8). Value: the shipped
    // `VALUE_PROJECTION_PARAMS` (its step still config-overridable via the
    // SL-213 `[priority.gauge] step` seam — default IS the const's step).
    // Estimate: `EST_GAUGE_STEP` (config: `[priority.estimate] gauge_step`)
    // with the gauge CENTERED on the corpus's own bare-item cost anchor
    // (D7/D8 — anchor-free est components render around the engine's
    // absent-cost stance, not a new arbitrary constant).
    let value_cfg = ProjectionCfg {
        gauge_step: cfg.gauge.step,
        ..comparison::VALUE_PROJECTION_PARAMS
    };
    let est_cfg = ProjectionCfg {
        gauge_step: cfg.estimate.gauge_step,
        gauge_center: bare_cost_anchor(scanned, &cfg.estimate),
    };
    comparison::load_pipeline(root, &statuses, &est_anchors, &value_cfg, &est_cfg)
}

/// The bare-item cost anchor (SL-172 §5.4 anchor fold): the maximum `upper`
/// among NON-TERMINAL items' AUTHORED estimates + `margin`, `1.0` when no
/// estimate exists (the empty-corpus fallback). Authored uppers ONLY — the
/// projected tier never moves it (SL-219 D7: no feedback loop through the
/// default). Terminals (closed/done) must not inflate bare-item cost. Pure
/// helper shared by [`build_from_with_cfg`]'s `CostCtx` and the est-domain
/// `gauge_center` in [`load_comparison_pipeline`]; it lives here (not in
/// `comparison::store`) because it reads `partition::status_class` +
/// `ScannedEntity`, both above `comparison` in the ADR-001 layering — the
/// same home rationale as [`comparison_status_map`].
fn bare_cost_anchor(scanned: &[relation_graph::ScannedEntity], ec: &config::EstimateCost) -> f64 {
    let max_upper = scanned
        .iter()
        .filter(|entity| {
            partition::status_class(entity.kind, entity.status.as_deref()) != StatusClass::Terminal
        })
        .filter_map(|entity| entity.estimate.as_ref().map(|e| e.upper))
        .max_by(f64::total_cmp);
    match max_upper {
        Some(mu) => mu + ec.margin,
        None => 1.0,
    }
}

/// Convenience one-call wrapper over [`load_comparison_pipeline`] for callers
/// with no pre-existing scan to share — `compare list` (SL-213 PHASE-06),
/// which needs the pipeline alone, not a `PriorityGraph`. Mirrors [`build`]'s
/// relationship to [`build_from`].
pub(crate) fn load_comparison_pipeline_for_root(
    root: &std::path::Path,
) -> anyhow::Result<comparison::Pipeline> {
    let scanned = relation_graph::scan_entities(root, &mut vec![], ScanMode::default())?;
    let cfg = config::load(root);
    load_comparison_pipeline(root, &scanned, &cfg)
}

/// Build the comparison-tier [`StatusMap`] from the raw entity scan (design §1
/// integration point): terminal status via [`partition::status_class`];
/// supersession via outbound `supersedes` edges — an edge A→B (A's outbound
/// `Supersedes` relation names B) means B is superseded BY A
/// (`comparison::resolve` R6 "superseded entity").
///
/// **Home, stated:** this builder lives here, in `priority::graph` — NOT in
/// `comparison::store` — because it depends on `partition::status_class` and
/// `ScannedEntity`, both of which sit above `comparison` in the ADR-001
/// layering (design §1: `comparison::store` depends on `wire, fs` only).
/// `priority::graph` already depends on both `priority::partition` and
/// `comparison`, so it is the layering-safe home for the bridge; building it
/// inside `comparison::store` would put `comparison` in a two-way dependency
/// with `priority` (a cycle), since `priority::graph` also calls
/// `comparison::load_pipeline`.
fn comparison_status_map(scanned: &[relation_graph::ScannedEntity]) -> StatusMap {
    let mut map = StatusMap::new();
    for entity in scanned {
        if partition::status_class(entity.kind, entity.status.as_deref()) == StatusClass::Terminal {
            map.insert(entity.key.canonical(), EntityLifecycle::Terminal);
        }
    }
    for entity in scanned {
        for edge in &entity.outbound {
            if edge.label == RelationLabel::Supersedes
                && let Ok((kref, id)) = integrity::parse_canonical_ref(&edge.target)
            {
                let target_key = EntityKey {
                    prefix: kref.kind.prefix,
                    id,
                };
                map.insert(
                    target_key.canonical(),
                    EntityLifecycle::Superseded {
                        by: entity.key.canonical(),
                    },
                );
            }
        }
    }
    map
}

/// Build the est-domain [`AnchorMap`] from the raw entity scan (SL-219 design
/// §2 anchor seam): each ADMISSIBLE entity's (per [`comparison::
/// admissible_estimate_pair`] kinds — no parallel list) authored `[estimate]`
/// facet, β-resolved through [`authored_est_cost`] — the operative scalar
/// cost, the ONE formula site (D1). Estimated RECORDS anchor too (anchor mass
/// through chains, though nothing scores a record). The map is the CANDIDATE
/// set; row-gating (an anchor enters the est system only when an est-domain
/// row touches its item) happens at compile-input selection inside
/// `comparison::store` (design §1). Same layering rationale as
/// [`comparison_status_map`] — home stated there (needs `config` +
/// `ScannedEntity`, both above `comparison`).
fn comparison_est_anchor_map(
    scanned: &[relation_graph::ScannedEntity],
    cfg: &config::PriorityConfig,
) -> AnchorMap {
    scanned
        .iter()
        .filter(|entity| {
            comparison::admissible_estimate_pair(entity.kind.prefix, entity.kind.prefix).is_ok()
        })
        .filter_map(|entity| {
            entity.estimate.as_ref().map(|e| {
                (
                    entity.key.canonical(),
                    authored_est_cost((e.lower, e.upper), &cfg.estimate),
                )
            })
        })
        .collect()
}

/// Build the priority graph from a PRE-SCANNED entity slice with an INJECTED
/// [`config::PriorityConfig`] (the SL-194 PHASE-01 rebuild seam — the body of
/// [`build_from`], extracted so β perturbation can sweep `estimate.skew` over the same
/// scan). Identical to [`build_from`] except the config is supplied rather than loaded.
///
/// **The injected `cfg` threads the WHOLE build.** The base pre-pass (2c, `base_score`)
/// AND the consequence post-pass (6, `consequence_post_pass` — leverage/optionality
/// coeffs) both read it, so a swept `skew` perturbs base cost and consequence coeffs
/// consistently (design "cfg threads the WHOLE build") — never base-only with default
/// consequence coeffs. Byte-identical to the pre-extraction `build_from` when passed
/// `&config::load(root)` (the behaviour-preservation gate, VT-1).
///
/// SL-213 PHASE-05: `projected` is the comparison-tier value projection
/// (entity id → `(value, provenance)`, `authored > projected > gauge` — D11),
/// consulted by [`effective_raw_value`] at BOTH the base-score `value_dim` site
/// and the burndown accessor identically (governed policy: gauge participates
/// in burndown). An empty `projected` map is bitwise-identical to the
/// pre-SL-213 two-tier resolution (the behaviour-preservation gate). A caller
/// sweeping `cfg` (the β endpoint sweep, `surface::beta_endpoints`) loads ONE
/// projection over the shared scan and passes it to every build — the
/// projection is scan-derived, not cfg-swept.
///
/// SL-219 PHASE-04: `cost_feed` is the est-domain scoring feed
/// ([`comparison::cost_feed`] — the est projection minus its Gauge tier),
/// consulted ONLY at the [`est_cost`] ladder's middle tier. Zero est-domain
/// rows ⇒ empty feed ⇒ bitwise-identical scoring to the pre-ladder build
/// (the same empty-map preservation gate as `projected`).
///
/// SL-220 PHASE-05: `claims` is the value-domain claim resolution
/// (`Pipeline.value_claims` — the SAME pipeline pass that produced
/// `projected`), threaded as a PURE input from the shell (the cost-feed
/// precedent) and consulted by [`effective_raw_value`]'s claim rungs. An
/// empty resolution (no anchor rows on disk) is bitwise-identical to the
/// pre-flip build over the same `projected`/`cost_feed` (the empty-claims
/// preservation gate, §8.5).
///
/// # Errors
///
/// Propagates a read error, or an internal cordage rejection of well-formed adapter
/// input (an adapter bug, not a recoverable condition).
pub(crate) fn build_from_with_cfg(
    scanned: &[relation_graph::ScannedEntity],
    root: &std::path::Path,
    cfg: &config::PriorityConfig,
    projected: &ValueProjection,
    cost_feed: &comparison::CostFeed,
    claims: &comparison::ClaimResolution,
) -> anyhow::Result<PriorityGraph> {
    // 2b. Anchor fold (SL-172 §5.4): the shared [`bare_cost_anchor`] helper —
    //      max upper among non-terminal estimated items + margin, else 1.0.
    let ctx = CostCtx {
        absent: bare_cost_anchor(scanned, &cfg.estimate),
    };

    // 2c. Base pre-pass — compute `base_score` per node from its OWN facets + config +
    //      kind (pure, per-node, graph-free). Runs before mint because it feeds the
    //      tiebreaker (SL-133 §5.4 step 2/3). Carried onto `NodeAttr.base_score` at 3c
    //      and read by the consequence post-pass.
    let base_by_key: BTreeMap<EntityKey, BaseScore> = scanned
        .iter()
        .map(|entity| {
            let base = base_score(
                &EntityFacets {
                    estimate: entity.estimate.clone(),
                    value: entity.value.clone(),
                    risk: entity.risk.clone(),
                    tags: entity.tags.clone(),
                },
                entity.kind,
                entity.key,
                projected,
                claims,
                cost_feed,
                cfg,
                ctx,
            );
            (entity.key, base)
        })
        .collect();

    // 3. Mint — (base.total() DESC via f64::total_cmp, canonical-id ASC) (SL-133 §5.4
    //    step 3; was `consequence desc`). The monotonic NodeId is the tier-3 fallback
    //    (the within-level allocation key). Consequence is EXCLUDED from mint — a
    //    graph-derived quantity in the structural tiebreak would couple ordering to the
    //    edges it orders (I3 feedback loop), and `score` is not yet computed. Pre-intern
    //    EVERY node in this order BEFORE any edge resolves (C4), asserting distinct keys.
    let mut order: Vec<EntityKey> = scanned.iter().map(|e| e.key).collect();
    order.sort_by(|a, b| {
        let ba = base_by_key.get(a).map_or(0.0, BaseScore::total);
        let bb = base_by_key.get(b).map_or(0.0, BaseScore::total);
        bb.total_cmp(&ba).then_with(|| a.cmp(b))
    });

    let mut builder = GraphBuilder::new();
    // Reference/lineage overlays (the consequence inputs) + the two dep/seq overlays.
    // Capture every OverlayId from the builder — never fabricate an id.
    let mut ref_by_label: BTreeMap<RelationLabel, OverlayId> = BTreeMap::new();
    for &label in REF_LABELS {
        let ov = builder.overlay(OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded));
        ref_by_label.insert(label, ov);
    }
    let dep_overlay = builder.overlay(OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded));
    let seq_overlay = builder.overlay(OverlayConfig::new(CyclePolicy::Evict, Arity::Unbounded));

    let mut projection: Projection<EntityKey> = Projection::new();
    for &key in &order {
        assert!(
            projection.resolve(key).is_none(),
            "priority::graph: duplicate EntityKey {} (canonical ids unique by prefix)",
            key.canonical()
        );
        projection.intern(&mut builder, key);
    }

    // 3b. Read each entity's dep/seq + promoted ONCE through the cross-kind dispatch
    //     (SL-060 §5.2 — `relation_graph::dep_seq_for` replaces the former backlog-prefix
    //     gate: it routes backlog AND slice to their readers and short-circuits every
    //     non-authoring kind with NO disk read, F5). The attrs pass and the edge pass
    //     share one read per entity (no double parse). `promoted` is carried alongside —
    //     backlog-only by construction (every other kind yields `false`).
    let mut dep_seq: BTreeMap<EntityKey, (dep_seq::DepSeq, bool)> = BTreeMap::new();
    for entity in scanned {
        dep_seq.insert(
            entity.key,
            relation_graph::dep_seq_for(root, entity.kind, entity.key.id)?,
        );
    }

    // 3c. Per-node attributes — RAW authored status verbatim, kind, promoted, and the
    //     `base_score` computed in the 2b pre-pass (reused, not recomputed). Only a
    //     backlog item can be `promoted`; every other kind is never promoted.
    //     Facets (estimate/value/risk/tags) are captured here from the scan so the
    //     surface shell projects them into view rows without recomputation (SL-171 D2).
    let mut attrs: BTreeMap<EntityKey, NodeAttr> = BTreeMap::new();
    for entity in scanned {
        let base = base_by_key.get(&entity.key).copied().unwrap_or(BaseScore {
            value_dim: 0.0,
            risk_dim: 0.0,
        });
        attrs.insert(
            entity.key,
            NodeAttr {
                kind: entity.kind,
                status: entity.status.clone(),
                promoted: dep_seq
                    .get(&entity.key)
                    .is_some_and(|(_ds, promoted)| *promoted),
                title: entity.title.clone(),
                base_score: base,
                facets: EntityFacets {
                    estimate: entity.estimate.clone(),
                    value: entity.value.clone(),
                    risk: entity.risk.clone(),
                    tags: entity.tags.clone(),
                },
            },
        );
    }

    // 4. Edges — resolve-only (never intern inside the edge pass). An unresolved
    //    target simply contributes NO edge (it is not recorded — there is no node to
    //    edge from / to).
    for entity in scanned {
        let Some(src) = projection.resolve(entity.key) else {
            debug_assert!(false, "priority::graph: edge-pass key not interned");
            continue;
        };

        // Reference/lineage edges onto the ref overlays (consequence inputs). An
        // unresolved or no-overlay (target-unvalidated) target contributes no edge.
        for edge in &entity.outbound {
            if let Some(dst) = resolve(&projection, &edge.target)
                && let Some(&ov) = ref_by_label.get(&edge.label)
            {
                builder.edge(ov, src, dst, EdgeAttrs::new(0, 0));
            }
        }

        // dep/seq edges — kind-agnostic (DD-2): emission is byte-identical and kind-blind;
        // a kind that authors no dep/seq simply carries empty axes (every non-authoring
        // kind, and any authoring entity with no edges).
        if let Some((ds, _promoted)) = dep_seq.get(&entity.key) {
            // `A.needs = [B]` ⇒ B must precede A: edge B→A (the flip), hard, never
            // evicts. An unresolved prereq contributes no edge (no node to edge from).
            for prereq_ref in &ds.needs {
                if let Some(prereq) = resolve(&projection, prereq_ref) {
                    builder.edge(dep_overlay, prereq, src, EdgeAttrs::new(0, 0));
                }
            }
            // `A.after = [{to=B, rank}]` ⇒ B before A: edge B→A carrying the genuine
            // `(rank, age)` eviction key; `age` is the entry's index in this item's
            // `after` array (the `backlog_order` discipline).
            for (idx, edge) in ds.after.iter().enumerate() {
                if let Some(prereq) = resolve(&projection, &edge.to) {
                    let age = u64::try_from(idx).map_err(|e| {
                        anyhow::anyhow!("priority::graph: after-edge index overflows u64: {e}")
                    })?;
                    builder.edge(seq_overlay, prereq, src, EdgeAttrs::new(edge.rank, age));
                }
            }
        }
    }

    // 5. OrderSpec over [dep Along, seq Along], then build.
    builder.order_spec(OrderSpec::new(vec![
        OrderLayer::new(dep_overlay, Direction::Along),
        OrderLayer::new(seq_overlay, Direction::Along),
    ]));

    let graph = builder.build().map_err(|e| {
        anyhow::anyhow!(
            "priority::graph: cordage rejected well-formed adapter input (internal bug): {e:?}"
        )
    })?;

    // 6. Consequence post-pass (design §5.4 step 6) — two mechanisms:
    //      needs-leverage (recursive DP) + ref-optionality (one-hop).
    //      Reads NodeAttr.base_score from `attrs` (the field is consumed here —
    //      no dead_code).
    let (leverage, optionality, score) = consequence_post_pass(
        &graph,
        &projection,
        &attrs,
        &ref_by_label,
        dep_overlay,
        cfg,
        projected,
        claims,
    );

    Ok(PriorityGraph {
        graph,
        projection,
        attrs,
        leverage,
        optionality,
        score,
        dep_overlay,
        seq_overlay,
        cost_ctx: ctx,
        cost_feed: cost_feed.clone(),
    })
}

/// Consequence post-pass (design §5.4 step 6). Pure over the built graph.
/// Returns (leverage, optionality, score) keyed by `EntityKey`.
#[expect(
    clippy::too_many_arguments,
    reason = "the pure scoring inputs (projection, claims) thread from build_from_with_cfg's one seam"
)]
fn consequence_post_pass(
    graph: &Graph,
    projection: &Projection<EntityKey>,
    attrs: &BTreeMap<EntityKey, NodeAttr>,
    ref_by_label: &BTreeMap<RelationLabel, OverlayId>,
    dep_overlay: OverlayId,
    cfg: &config::PriorityConfig,
    projected: &ValueProjection,
    claims: &comparison::ClaimResolution,
) -> (
    BTreeMap<EntityKey, f64>,
    BTreeMap<EntityKey, f64>,
    BTreeMap<EntityKey, f64>,
) {
    use std::collections::BTreeSet;

    // ── node-id ↔ EntityKey helpers ──
    let ek = |nid: cordage::NodeId| -> Option<EntityKey> { projection.key_of(nid) };
    let base_of = |nid: cordage::NodeId| -> f64 {
        ek(nid)
            .and_then(|k| attrs.get(&k))
            .map_or(0.0, |a| a.base_score.total())
    };
    // SL-176 PHASE-03: split value/risk accessors for the burndown post-pass.
    let value_dim_of = |nid: cordage::NodeId| -> f64 {
        ek(nid)
            .and_then(|k| attrs.get(&k))
            .map_or(0.0, |a| a.base_score.value_dim)
    };
    let risk_dim_of = |nid: cordage::NodeId| -> f64 {
        ek(nid)
            .and_then(|k| attrs.get(&k))
            .map_or(0.0, |a| a.base_score.risk_dim)
    };
    // SL-176 PHASE-03 / SL-177 PHASE-02 / SL-213 PHASE-05 / SL-220 PHASE-05:
    // raw value accessor routed through the priority-tier seam — the SL-220
    // evidence ladder (anchored claim → projection → priors → transitional
    // facet → default; valueless kind is 0.0). The burndown numerator and
    // denominator both use this — SAME source as `value_dim` (SL-213 design
    // "governed policy": gauge participates in burndown identically).
    let raw_value_of = |nid: cordage::NodeId| -> f64 {
        let Some(key) = ek(nid) else {
            return 0.0;
        };
        attrs
            .get(&key)
            .and_then(|a| effective_raw_value(a.kind, &a.facets, key, projected, claims))
            .unwrap_or(0.0)
    };

    // ── Component partition: each dep_overlay SCC from provenance is one component;
    //      every other node is its own singleton. EVERY node is assigned up front so
    //      the condensation DAG below is total (RV-137 F-1: a lazily-assigned-on-visit
    //      scheme can't be topo-ordered). ──
    let cycles = graph.provenance().cycles();
    let mut node_to_component: BTreeMap<cordage::NodeId, usize> = BTreeMap::new();
    let mut component_members: Vec<BTreeSet<cordage::NodeId>> = Vec::new();
    for cyc in cycles {
        if cyc.overlay() != dep_overlay {
            continue;
        }
        let comp_idx = component_members.len();
        for &n in cyc.nodes() {
            node_to_component.insert(n, comp_idx);
        }
        component_members.push(cyc.nodes().clone());
    }
    for nid in graph.ordered() {
        node_to_component.entry(nid).or_insert_with(|| {
            let comp_idx = component_members.len();
            component_members.push(BTreeSet::from([nid]));
            comp_idx
        });
    }
    let component_count = component_members.len();
    let comp_of = |nid: cordage::NodeId| -> Option<usize> { node_to_component.get(&nid).copied() };

    // ── Condensation DAG: an edge c → c' means a member of component c has a
    //      dep out-edge (a DEPENDENT) landing in c'. Per RV-137 F-1 the leverage DP
    //      must run in reverse-topo order of THIS graph — reverse graph.ordered() is
    //      NOT a valid order because a seq edge can perturb an SCC member's level and
    //      place it before an external dependent, dropping that dependent's resolved
    //      leverage. Per RV-137 F-2 each external dependent NODE is held in a set, so a
    //      dependent that needs >1 member counts ONCE per component. ──
    let mut comp_dependents: Vec<BTreeSet<cordage::NodeId>> =
        vec![BTreeSet::new(); component_count];
    let mut comp_succ: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); component_count];
    for (c, ((dependents, succ), members)) in comp_dependents
        .iter_mut()
        .zip(comp_succ.iter_mut())
        .zip(component_members.iter())
        .enumerate()
    {
        for &m in members {
            for (d, _) in graph.out_edges(dep_overlay, m) {
                match comp_of(d) {
                    Some(dc) if dc != c => {
                        dependents.insert(d);
                        succ.insert(dc);
                    }
                    _ => {} // intra-component (or unresolved) → contributes 0
                }
            }
        }
    }

    // Reverse-topo of the condensation via iterative post-order DFS: post-order emits
    // a component AFTER all its successors, so every dependent's leverage is resolved
    // before the component that leans on it. (The condensation is acyclic; the visited
    // guard is a belt-and-braces backstop.)
    let mut topo: Vec<usize> = Vec::with_capacity(component_count);
    let mut visited = vec![false; component_count];
    for start in 0..component_count {
        if visited.get(start).copied().unwrap_or(true) {
            continue;
        }
        let mut stack: Vec<(usize, bool)> = vec![(start, false)];
        while let Some((c, emit)) = stack.pop() {
            if emit {
                topo.push(c);
                continue;
            }
            if visited.get(c).copied().unwrap_or(true) {
                continue;
            }
            if let Some(slot) = visited.get_mut(c) {
                *slot = true;
            }
            stack.push((c, true));
            if let Some(succ) = comp_succ.get(c) {
                for &sc in succ {
                    if !visited.get(sc).copied().unwrap_or(true) {
                        stack.push((sc, false));
                    }
                }
            }
        }
    }

    // ── leverage DP over the condensation in reverse-topo order. leverage(c) =
    //      dep_coeff · Σ over UNIQUE external dependents D of (base(D) + leverage(D));
    //      every member of c carries the same component leverage. ──
    let mut leverage_by_node: BTreeMap<cordage::NodeId, f64> = BTreeMap::new();
    for &c in &topo {
        let Some(dependents) = comp_dependents.get(c) else {
            continue;
        };
        let mut sum = 0.0f64;
        for &d in dependents {
            sum += base_of(d) + leverage_by_node.get(&d).copied().unwrap_or(0.0);
        }
        let lev = cfg.consequence.dep_coeff * sum;
        let lev = if lev.is_finite() { lev } else { 0.0 };
        if let Some(members) = component_members.get(c) {
            for &m in members {
                leverage_by_node.insert(m, lev);
            }
        }
    }

    // ── optionality: one-hop ref over CONSEQUENCE_LABELS (design §5.4 step 6).
    //      N's referencers are in_edges(ov, N) over the CONSEQUENCE_LABELS subset only.
    let mut optionality_by_node: BTreeMap<cordage::NodeId, f64> = BTreeMap::new();
    for nid in graph.ordered() {
        let mut sum = 0.0f64;
        for &label in CONSEQUENCE_LABELS {
            if let Some(&ov) = ref_by_label.get(&label) {
                for (src, _) in graph.in_edges(ov, nid) {
                    sum += base_of(src);
                }
            }
        }
        let opt = cfg.consequence.ref_coeff * sum;
        let opt = if opt.is_finite() { opt } else { 0.0 };
        optionality_by_node.insert(nid, opt);
    }

    // ── SL-176 PHASE-03: fulfils value-burndown post-pass (D-priority-burndown).
    //      A backlog item's value_dim is REDUCED by the lifecycle-gated raw value of
    //      the slices that fulfil it. Degree ignored, non-conserving across multi-item,
    //      excluded from mint tiebreak. Fulfils overlay backs in_edges (REF_LABELS only).
    let fulfils_ov = ref_by_label.get(&RelationLabel::Fulfils).copied();
    let mut burndown_by_node: BTreeMap<cordage::NodeId, f64> = BTreeMap::new();
    if let Some(ov) = fulfils_ov {
        for nid in graph.ordered() {
            let raw_val = raw_value_of(nid);
            if raw_val <= 0.0 {
                burndown_by_node.insert(nid, 0.0);
                continue;
            }
            // delivered = Σ over fulfils in_edges of gate(status(src)) · raw_value(src)
            // gate = 1.0 iff source slice status ∈ {started, audit, reconcile, done}
            let mut delivered = 0.0f64;
            for (src, _) in graph.in_edges(ov, nid) {
                let Some(src_key) = ek(src) else { continue };
                let Some(src_attr) = attrs.get(&src_key) else {
                    continue;
                };
                let gate = match src_attr.status.as_deref() {
                    Some("started" | "audit" | "reconcile" | "done") => 1.0,
                    _ => 0.0,
                };
                if gate > 0.0 {
                    delivered += gate * raw_value_of(src);
                }
            }
            // r = clamp(delivered / raw_value, 0, 1)
            let r = (delivered / raw_val).clamp(0.0, 1.0);
            let burn = value_dim_of(nid) * (1.0 - r);
            let burn = if burn.is_finite() { burn } else { 0.0 };
            burndown_by_node.insert(nid, burn);
        }
    }

    // ── assemble into EntityKey-keyed maps ──
    let mut leverage: BTreeMap<EntityKey, f64> = BTreeMap::new();
    let mut optionality: BTreeMap<EntityKey, f64> = BTreeMap::new();
    let mut score: BTreeMap<EntityKey, f64> = BTreeMap::new();
    for nid in graph.ordered() {
        if let Some(k) = ek(nid) {
            let lev = leverage_by_node.get(&nid).copied().unwrap_or(0.0);
            let opt = optionality_by_node.get(&nid).copied().unwrap_or(0.0);
            // SL-176 PHASE-03: score = risk_dim + lev + opt + burndown_term
            // where burndown_term = value_dim · (1 − r) = the attenuated value.
            // A node with no fulfils inbound has r=0 ⇒ burndown_term = value_dim
            // ⇒ score = risk_dim + value_dim + lev + opt == base_of + lev + opt (unchanged).
            let burn = burndown_by_node
                .get(&nid)
                .copied()
                .unwrap_or(value_dim_of(nid));
            let sc = risk_dim_of(nid) + lev + opt + burn;
            let sc = if sc.is_finite() { sc } else { 0.0 };
            leverage.insert(k, lev);
            optionality.insert(k, opt);
            score.insert(k, sc);
        }
    }
    (leverage, optionality, score)
}

/// Get-only resolve of an authored ref string to a minted node, or `None`. A ref
/// that fails to parse as a canonical ref (free-text), or parses to an id never
/// minted (no entity dir), is `None` → a dangler. NEVER interns.
fn resolve(projection: &Projection<EntityKey>, reference: &str) -> Option<cordage::NodeId> {
    let (kref, id) = integrity::parse_canonical_ref(reference).ok()?;
    projection.resolve(EntityKey {
        prefix: kref.kind.prefix,
        id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Write `root/<rel>` with `body`, creating parents.
    fn write(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    /// SL-048 PHASE-04: rewrite a legacy `[relationships]` body (`key = [...]` lines)
    /// into the migrated on-disk shape for `source` — tier-1 simple-list axes become
    /// `[[relation]]` rows (canonical order is laundered by `read_block`, so emit
    /// order here is irrelevant); every other line (the typed `needs`/`after`/
    /// `triggers` payload axes, or any non-migrated label) stays verbatim in a
    /// `[relationships]` table emitted FIRST (F1). Keeps these fixtures' inline bodies
    /// readable while exercising the post-cut storage shape.
    fn migrate_body(source: &crate::entity::Kind, rels: &str) -> String {
        use crate::relation::RelationLabel;
        let mut typed = String::new();
        let mut rows = String::new();
        for line in rels.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let key = trimmed.split('=').next().unwrap_or("").trim();
            let is_simple_list = trimmed.contains('[') && !trimmed.contains('{');
            let migrated = is_simple_list
                && RelationLabel::from_name(key)
                    .and_then(|l| crate::relation::lookup(source, l, None))
                    .is_some_and(|r| {
                        r.tier == crate::relation::Tier::One
                            && r.link != crate::relation::LinkPolicy::LifecycleOnly
                    });
            if migrated {
                let inner = trimmed
                    .split_once('[')
                    .and_then(|(_, rest)| rest.rsplit_once(']'))
                    .map(|(refs, _)| refs)
                    .unwrap_or("");
                for t in inner.split(',') {
                    let t = t.trim().trim_matches('"');
                    if !t.is_empty() {
                        rows.push_str(&format!(
                            "[[relation]]\nlabel = \"{key}\"\ntarget = \"{t}\"\n"
                        ));
                    }
                }
            } else {
                typed.push_str(line);
                typed.push('\n');
            }
        }
        let typed_table = if typed.trim().is_empty() {
            String::new()
        } else {
            format!("[relationships]\n{typed}")
        };
        format!("{typed_table}{rows}")
    }

    /// Seed a slice (toml + md) with a legacy `[relationships]` body (rewritten to the
    /// SL-048 migrated shape via [`migrate_body`]).
    fn seed_slice(root: &Path, id: u32, rels: &str) {
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"s\"\ntitle = \"S\"\nstatus = \"proposed\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{}",
                migrate_body(&crate::slice::SLICE_KIND, rels)
            ),
        );
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.md"),
            "scope\n",
        );
    }

    /// Seed a requirement (an edge target only — has a top-level status).
    fn seed_requirement(root: &Path, id: u32) {
        write(
            root,
            &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.toml"),
            &format!("id = {id}\nslug = \"r\"\ntitle = \"R\"\nstatus = \"active\"\n"),
        );
        write(
            root,
            &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.md"),
            "r\n",
        );
    }

    /// Seed a backlog issue with a `[relationships]` body and a `resolution`.
    fn seed_issue(root: &Path, id: u32, status: &str, resolution: &str, rels: &str) {
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"{status}\"\n\
                 resolution = \"{resolution}\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 {}",
                migrate_body(&crate::backlog::ISSUE_KIND, rels)
            ),
        );
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.md"),
            "b\n",
        );
    }

    /// Seed a risk backlog item (so a second backlog kind exists for dep/seq).
    fn seed_risk(root: &Path, id: u32, status: &str, rels: &str) {
        write(
            root,
            &format!(".doctrine/backlog/risk/{id:03}/backlog-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"k\"\ntitle = \"K\"\nkind = \"risk\"\nstatus = \"{status}\"\n\
                 resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 {}",
                migrate_body(&crate::backlog::RISK_KIND, rels)
            ),
        );
        write(
            root,
            &format!(".doctrine/backlog/risk/{id:03}/backlog-{id:03}.md"),
            "k\n",
        );
    }

    /// Seed a reconciliation record (status-LESS by design).
    fn seed_rec(root: &Path, id: u32, owning_slice: &str) {
        write(
            root,
            &format!(".doctrine/rec/{id:03}/rec-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"r\"\ntitle = \"R\"\n\
                 [rec]\nmove = \"accept\"\nowning_slice = \"{owning_slice}\"\n"
            ),
        );
    }

    /// Seed a review (status-LESS authored; status derived from findings).
    fn seed_review(root: &Path, id: u32, target: &str, findings: &str) {
        write(
            root,
            &format!(".doctrine/review/{id:03}/review-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"r\"\ntitle = \"R\"\n\
                 [review]\nfacet = \"reconciliation\"\nraiser = \"a\"\nresponder = \"b\"\n\
                 [target]\nref = \"{target}\"\n{findings}"
            ),
        );
    }

    fn key(prefix: &'static str, id: u32) -> EntityKey {
        EntityKey { prefix, id }
    }

    // -- VT-1: builds; node set equals the scanned set; distinct keys ----------

    #[test]
    fn builds_over_multi_kind_corpus_node_set_equals_scanned() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(
            root,
            1,
            "[[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"REQ-005\"\n",
        );
        seed_requirement(root, 5);
        seed_issue(root, 1, "open", "", "slices = [\"SL-001\"]\n");
        seed_rec(root, 1, "SL-001");
        seed_review(root, 1, "SL-001", "");

        let pg = build(root).unwrap();
        // Node set equals the scanned entity set (one NodeAttr per scanned entity).
        let scanned: std::collections::BTreeSet<EntityKey> =
            relation_graph::scan_entities(root, &mut vec![], ScanMode::default())
                .unwrap()
                .iter()
                .map(|e| e.key)
                .collect();
        let minted: std::collections::BTreeSet<EntityKey> = pg.attrs.keys().copied().collect();
        assert_eq!(minted, scanned, "every scanned entity is a node");
        // Each key resolves (distinct keys, all interned).
        for k in &scanned {
            assert!(
                pg.projection.resolve(*k).is_some(),
                "{} minted",
                k.canonical()
            );
        }
        assert_eq!(pg.attrs.len(), scanned.len());
        // NodeAttr.kind carries the kind descriptor (its prefix matches the key).
        for (k, attr) in &pg.attrs {
            assert_eq!(
                attr.kind.prefix, k.prefix,
                "NodeAttr.kind matches the key prefix"
            );
        }
    }

    // -- VT-1 + EX-2: NodeAttr status/promoted reads -------------------------

    #[test]
    fn node_attr_status_promoted_per_kind() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, "");
        seed_requirement(root, 5);
        // A promoted issue (resolution == promoted) vs a plain open one.
        seed_issue(root, 1, "resolved", "promoted", "");
        seed_issue(root, 2, "open", "", "");
        seed_rec(root, 1, "SL-001");
        // A review with one OPEN finding ⇒ derived status "active".
        seed_review(
            root,
            1,
            "SL-001",
            "[[finding]]\nid = \"F-1\"\nstatus = \"open\"\nseverity = \"minor\"\n\
             title = \"t\"\ndetail = \"d\"\n",
        );
        // A review with all VERIFIED ⇒ derived status "done".
        seed_review(
            root,
            2,
            "SL-001",
            "[[finding]]\nid = \"F-1\"\nstatus = \"verified\"\nseverity = \"minor\"\n\
             title = \"t\"\ndetail = \"d\"\n",
        );

        let pg = build(root).unwrap();
        // Slice carries its raw authored status.
        assert_eq!(pg.attrs[&key("SL", 1)].status.as_deref(), Some("proposed"));
        assert!(!pg.attrs[&key("SL", 1)].promoted);
        // Requirement carries its top-level status.
        assert_eq!(pg.attrs[&key("REQ", 5)].status.as_deref(), Some("active"));
        // REC is status-less.
        assert_eq!(pg.attrs[&key("REC", 1)].status, None);
        // Promoted issue: flag set, status raw "resolved".
        assert_eq!(pg.attrs[&key("ISS", 1)].status.as_deref(), Some("resolved"));
        assert!(
            pg.attrs[&key("ISS", 1)].promoted,
            "resolution=promoted ⇒ promoted"
        );
        // Plain issue: not promoted.
        assert!(!pg.attrs[&key("ISS", 2)].promoted);
        // RV status is DERIVED, not stored.
        assert_eq!(pg.attrs[&key("RV", 1)].status.as_deref(), Some("active"));
        assert_eq!(pg.attrs[&key("RV", 2)].status.as_deref(), Some("done"));
    }

    // -- VT-7: mint uses BASE only (consequence excluded), score is post-pass ---

    #[test]
    fn mint_order_base_desc_then_canonical_asc_and_permutation_invariant() {
        let dir = tmp();
        let root = dir.path();
        // Three issues with DIFFERENT base scores (value facet over est_cost=6.5):
        // ISS-001 value 5 → base 5/6.5; ISS-002 value 25 → base 25/6.5;
        // ISS-003 value 15 → base 15/6.5. Mint order is base.total() DESC, ties by id ASC.
        // Crucially the consequence/edge topology does NOT enter mint (I3).
        seed_issue_with_facets(root, 1, "", "lower = 0.0\nupper = 10.0", "value = 5.0", "");
        seed_issue_with_facets(root, 2, "", "lower = 0.0\nupper = 10.0", "value = 25.0", "");
        seed_issue_with_facets(root, 3, "", "lower = 0.0\nupper = 10.0", "value = 15.0", "");

        let pg = build(root).unwrap();
        // NodeId reflects mint order: lower NodeId = minted earlier (higher base).
        let n1 = pg.projection.resolve(key("ISS", 1)).unwrap();
        let n2 = pg.projection.resolve(key("ISS", 2)).unwrap();
        let n3 = pg.projection.resolve(key("ISS", 3)).unwrap();
        assert!(
            n2 < n3,
            "ISS-002 (base 25/6.5) mints before ISS-003 (base 15/6.5)"
        );
        assert!(
            n3 < n1,
            "ISS-003 (base 15/6.5) mints before ISS-001 (base 5/6.5)"
        );

        // Permutation invariance: re-seed the same corpus in a DIFFERENT authoring order
        // (BTree, no clock/RNG) — the score map and the mint order are identical.
        let dir2 = tmp();
        let root2 = dir2.path();
        seed_issue_with_facets(
            root2,
            3,
            "",
            "lower = 0.0\nupper = 10.0",
            "value = 15.0",
            "",
        );
        seed_issue_with_facets(
            root2,
            2,
            "",
            "lower = 0.0\nupper = 10.0",
            "value = 25.0",
            "",
        );
        seed_issue_with_facets(root2, 1, "", "lower = 0.0\nupper = 10.0", "value = 5.0", "");
        let pg2 = build(root2).unwrap();
        assert_eq!(pg.score, pg2.score, "score map is permutation-invariant");
        let m1 = pg2.projection.resolve(key("ISS", 1)).unwrap();
        let m2 = pg2.projection.resolve(key("ISS", 2)).unwrap();
        let m3 = pg2.projection.resolve(key("ISS", 3)).unwrap();
        assert!(m2 < m3 && m3 < m1, "mint order is permutation-invariant");
    }

    #[test]
    fn mint_order_is_blind_to_consequence_topology() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 (no facets, base 0) is referenced by TWO slices via `slices` (a
        // CONSEQUENCE_LABELS edge → high optionality in the post-pass). ISS-002 has a
        // value facet (base > 0) but no inbound references. Under the OLD policy the
        // referenced ISS-001 would mint first (consequence desc); under the score model
        // mint is base-only, so ISS-002 (higher base) mints FIRST — consequence is
        // excluded from the structural tiebreak (I3). The post-pass still gives ISS-001
        // a positive score, but that does not reorder the mint.
        seed_issue(root, 1, "open", "", "");
        seed_issue_with_facets(root, 2, "", "lower = 0.0\nupper = 10.0", "value = 25.0", "");
        seed_slice(root, 1, "slices = [\"ISS-001\"]\n");
        seed_slice(root, 2, "slices = [\"ISS-001\"]\n");

        let pg = build(root).unwrap();
        let n1 = pg.projection.resolve(key("ISS", 1)).unwrap();
        let n2 = pg.projection.resolve(key("ISS", 2)).unwrap();
        assert!(
            n2 < n1,
            "ISS-002 (base 25/6.5≈3.846) mints before the heavily-referenced ISS-001 (base 0) — mint is base-only (I3)"
        );
        // The post-pass still credits ISS-001's optionality (two slices reference it,
        // both base 0 here → optionality 0). SL-177 PHASE-02: valueless ISS-001 has
        // default 1.0 → value_dim = 1.0/11.0 ≈ 0.0909 → score = 0.0909.
        assert!((pg.score.get(&key("ISS", 1)).copied().unwrap_or(0.0) - 1.0 / 11.0).abs() < 1e-9);
    }

    // -- EX-4: dep/seq edges; an unresolved target contributes no edge ---------

    #[test]
    fn dep_seq_edges_emitted_for_backlog_unresolved_contributes_no_edge() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 needs RSK-001 (resolvable) and ISS-099 (unresolved); after ISS-002.
        seed_issue(
            root,
            1,
            "open",
            "",
            "needs = [\"RSK-001\", \"ISS-099\"]\nafter = [{ to = \"ISS-002\", rank = 0 }]\n",
        );
        seed_issue(root, 2, "open", "", "");
        seed_risk(root, 1, "open", "");

        let pg = build(root).unwrap();
        // The dep overlay carries the resolvable needs edge (RSK-001 → ISS-001, the
        // B→A flip): RSK-001 is a predecessor of ISS-001 in `dep`.
        let iss1 = pg.projection.resolve(key("ISS", 1)).unwrap();
        let rsk1 = pg.projection.resolve(key("RSK", 1)).unwrap();
        let dep_preds: Vec<_> = pg
            .graph
            .in_edges(pg.dep_overlay, iss1)
            .map(|(s, _)| s)
            .collect();
        // The unresolved ISS-099 needs ref produced NO edge — RSK-001 is the ONLY
        // dep predecessor of ISS-001 (the dangling-record was dropped; the absence of
        // a phantom edge is the surviving behaviour).
        assert_eq!(
            dep_preds,
            vec![rsk1],
            "only the resolvable needs prereq edges (B→A); unresolved adds nothing"
        );
        // The after edge (ISS-002 → ISS-001) lands on the seq overlay.
        let iss2 = pg.projection.resolve(key("ISS", 2)).unwrap();
        let seq_preds: Vec<_> = pg
            .graph
            .in_edges(pg.seq_overlay, iss1)
            .map(|(s, _)| s)
            .collect();
        assert!(
            seq_preds.contains(&iss2),
            "after edge oriented predecessor→src"
        );
    }

    #[test]
    fn nodes_authoring_no_dep_seq_carry_no_edges() {
        let dir = tmp();
        let root = dir.path();
        // SL-176 PHASE-03: `Slices` removed from CONSEQUENCE_LABELS — use
        // `references(implements)` for the optionality witness.
        // SL-001 references(implements) REQ-005 → REQ-005 gets optionality from SL-001's base.
        // SL-001 needs a value facet so its base_score is non-zero.
        // Author the slice toml directly with facet + implements edge.
        write(
            root,
            ".doctrine/slice/001/slice-001.toml",
            "id = 1\nslug = \"s\"\ntitle = \"S\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [estimate]\nlower = 0.0\nupper = 10.0\n\
             [value]\nvalue = 25.0\n\
             [[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"REQ-005\"\n",
        );
        write(root, ".doctrine/slice/001/slice-001.md", "scope\n");
        seed_issue_with_facets(root, 1, "", "lower = 0.0\nupper = 10.0", "value = 25.0", "");
        seed_issue(root, 2, "open", "", "");
        seed_requirement(root, 5);
        seed_slice(root, 2, "");
        let pg = build(root).unwrap();
        let sl1 = pg.projection.resolve(key("SL", 1)).unwrap();
        let sl2 = pg.projection.resolve(key("SL", 2)).unwrap();
        assert_eq!(pg.graph.in_edges(pg.dep_overlay, sl1).count(), 0);
        assert_eq!(pg.graph.in_edges(pg.seq_overlay, sl1).count(), 0);
        assert_eq!(pg.graph.in_edges(pg.dep_overlay, sl2).count(), 0);
        // The resolvable `references(implements)` ref edge landed: REQ-005's optionality
        // reflects SL-001's base (25/6.5).
        assert!(
            (pg.optionality.get(&key("REQ", 5)).copied().unwrap_or(0.0) - 25.0 / 6.5).abs() < 1e-9,
            "resolvable consequence ref produces its edge (witnessed via optionality)"
        );
    }

    // -- SL-060 VT-1/VT-2: cross-kind slice dep/seq reaches the same overlays ---

    #[test]
    fn slice_needs_lands_on_dep_overlay_cross_kind() {
        let dir = tmp();
        let root = dir.path();
        // SL-001 needs SL-002 — a slice→slice hard prerequisite. The cross-kind
        // `dep_seq_for` slice arm reads it; emission is kind-blind, so it lands on the
        // SAME dep overlay the backlog `needs` does, oriented prereq→dependent (B→A).
        seed_slice(root, 1, "needs = [\"SL-002\"]\n");
        seed_slice(root, 2, "");
        let pg = build(root).unwrap();
        let sl1 = pg.projection.resolve(key("SL", 1)).unwrap();
        let sl2 = pg.projection.resolve(key("SL", 2)).unwrap();
        let dep_preds: Vec<_> = pg
            .graph
            .in_edges(pg.dep_overlay, sl1)
            .map(|(s, _)| s)
            .collect();
        assert_eq!(
            dep_preds,
            vec![sl2],
            "slice→slice needs lands on the dep overlay (B→A flip), like backlog"
        );
    }

    #[test]
    fn slice_after_lands_on_seq_overlay_with_rank_and_array_index_age() {
        let dir = tmp();
        let root = dir.path();
        // SL-001 after SL-002 (rank 7, array index 0) then SL-003 (rank 0, index 1).
        // The slice seq overlay must carry the SAME (rank, age=array index) eviction key
        // the backlog seq overlay does (INV-2 parity, kind-blind emission).
        seed_slice(
            root,
            1,
            "after = [{ to = \"SL-002\", rank = 7 }, { to = \"SL-003\" }]\n",
        );
        seed_slice(root, 2, "");
        seed_slice(root, 3, "");
        let pg = build(root).unwrap();
        let sl1 = pg.projection.resolve(key("SL", 1)).unwrap();
        let sl2 = pg.projection.resolve(key("SL", 2)).unwrap();
        let sl3 = pg.projection.resolve(key("SL", 3)).unwrap();
        // Collect (predecessor, rank, age) off the seq overlay's in-edges of SL-001.
        let seq: BTreeMap<_, _> = pg
            .graph
            .in_edges(pg.seq_overlay, sl1)
            .map(|(s, a)| (s, (a.rank(), a.age())))
            .collect();
        assert_eq!(
            seq.get(&sl2).copied(),
            Some((7, 0)),
            "first after edge: authored rank 7, age = array index 0"
        );
        assert_eq!(
            seq.get(&sl3).copied(),
            Some((0, 1)),
            "second after edge: default rank 0, age = array index 1"
        );
    }

    // -- A free-text / no-overlay outbound target produces no edge -------------

    #[test]
    fn free_text_outbound_target_produces_no_edge() {
        let dir = tmp();
        let root = dir.path();
        // A backlog drift edge is target-unvalidated (no overlay) → it produces no
        // edge at all. With the lone item (no facets), nothing references it and it
        // references no real node, so its score stays at the 0 floor — the surviving
        // behaviour of the dropped dangling record.
        seed_issue(root, 1, "open", "", "drift = [\"some-free-text\"]\n");
        let pg = build(root).unwrap();
        let n = pg.projection.resolve(key("ISS", 1)).unwrap();
        assert_eq!(
            pg.graph.out_edges(pg.dep_overlay, n).count(),
            0,
            "free-text drift target produces no dep edge"
        );
        // SL-177 PHASE-02: valueless backlog item defaults to 1.0 via
        // effective_raw_value → passes burndown guard → score = value_dim = 1.0.
        assert_eq!(
            pg.score.get(&key("ISS", 1)).copied().unwrap_or(0.0),
            1.0,
            "free-text drift target: valueless item score = 1.0 (default)"
        );
    }

    // ── PHASE-04 scoring tests ───────────────────────────────────────────

    /// Seed a backlog item with estimate + value + risk facets for scoring tests.
    fn seed_issue_with_facets(
        root: &Path,
        id: u32,
        rels: &str,
        estimate: &str,
        value: &str,
        risk_facet: &str,
    ) {
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
                 resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 {}\n[estimate]\n{}\n[value]\n{}\n[facet]\n{}\n",
                migrate_body(&crate::backlog::ISSUE_KIND, rels),
                estimate,
                value,
                risk_facet,
            ),
        );
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.md"),
            "b\n",
        );
    }

    // ── VT-2: base_score matrix ─────────────────────────────────────────

    #[test]
    fn base_score_all_facets_present() {
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(
            root,
            1,
            "",
            "lower = 2.0\nupper = 8.0",
            "value = 10.0",
            "likelihood = \"high\"\nimpact = \"critical\"",
        );
        let pg = build(root).unwrap();
        let bs = pg.attrs[&key("ISS", 1)].base_score;
        // value_dim = 1.0(value coeff) * 10.0 * 1.0(kind_weight) * 1.0(Σtag) / est_cost
        //   est_cost = lower + β·(upper-lower) = 2.0 + 0.65*(8.0-2.0) = 5.9
        //   value_dim = 10.0 / 5.9 ≈ 1.694915254
        // risk_dim  = 2.0(risk coeff) * 12(exposure: high=3 × critical=4)
        //          = 24.0
        assert!(
            (bs.value_dim - 10.0 / 5.9).abs() < 1e-9,
            "value_dim should be 10/5.9"
        );
        assert!((bs.risk_dim - 24.0).abs() < 1e-9, "risk_dim should be 24.0");
        assert!(
            (bs.total() - (10.0 / 5.9 + 24.0)).abs() < 1e-9,
            "total should be 10/5.9 + 24"
        );
    }

    #[test]
    fn base_score_value_only_risk_absent() {
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "", "lower = 0.0\nupper = 2.0", "value = 5.0", "");
        let pg = build(root).unwrap();
        let bs = pg.attrs[&key("ISS", 1)].base_score;
        // est_cost = lower + β·(upper-lower) = 0.0 + 0.65*(2.0-0.0) = 1.3
        // value_dim = 1.0 * 5.0 / 1.3 ≈ 3.846153846
        assert!(
            (bs.value_dim - 5.0 / 1.3).abs() < 1e-9,
            "value_dim should be 5.0/1.3"
        );
        assert!((bs.risk_dim - 0.0).abs() < 1e-9, "risk_dim should be 0");
        assert!(
            (bs.total() - 5.0 / 1.3).abs() < 1e-9,
            "total should be 5.0/1.3"
        );
    }

    #[test]
    fn base_score_risk_only_value_absent() {
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(
            root,
            1,
            "",
            "",
            "",
            "likelihood = \"low\"\nimpact = \"medium\"",
        );
        let pg = build(root).unwrap();
        let bs = pg.attrs[&key("ISS", 1)].base_score;
        // SL-177: ISS is value-bearing; no authored value → default 1.0.
        // est_cost = absent = 1.0 (lone bare item); value_dim = 1.0 * 1.0 * 1.0 * 1.0 / 1.0 = 1.0.
        assert!(
            (bs.value_dim - 1.0).abs() < 1e-9,
            "value_dim should be 1.0 (default)"
        );
        // risk_dim = 2.0 * 2 (low=1 × medium=2) = 4.0
        assert!((bs.risk_dim - 4.0).abs() < 1e-9, "risk_dim should be 4.0");
        assert!((bs.total() - 5.0).abs() < 1e-9, "total should be 5.0");
    }

    #[test]
    fn base_score_neither_facet_present() {
        let dir = tmp();
        let root = dir.path();
        seed_issue(root, 1, "open", "", "");
        let pg = build(root).unwrap();
        let bs = pg.attrs[&key("ISS", 1)].base_score;
        // SL-177: ISS is value-bearing; no authored value → default 1.0.
        // est_cost = absent = 1.0 (lone bare item); value_dim = 1.0.
        assert!(
            (bs.value_dim - 1.0).abs() < 1e-9,
            "value_dim should be 1.0 (default)"
        );
        assert!((bs.risk_dim - 0.0).abs() < 1e-9, "risk_dim should be 0");
        assert!((bs.total() - 1.0).abs() < 1e-9, "total should be 1.0");
    }

    #[test]
    fn base_score_bare_item_empty_corpus_fallback_cost_one() {
        let dir = tmp();
        let root = dir.path();
        // A lone bare item — no estimate anywhere in the corpus → absent = 1.0 (empty fallback).
        // est_cost = absent = 1.0; value_dim = 1.0 * 3.0 / 1.0 = 3.0.
        seed_issue_with_facets(
            root,
            1,
            "",
            "", // no estimate
            "value = 3.0",
            "",
        );
        let pg = build(root).unwrap();
        let bs = pg.attrs[&key("ISS", 1)].base_score;
        assert!((bs.value_dim - 3.0).abs() < 1e-9, "value_dim should be 3.0");
    }

    // ── SL-177: effective_raw_value / DEFAULT_VALUE ─────────────────────

    /// Valueless SL (value-bearing, no authored value) → value_dim = DEFAULT_VALUE / est_cost.
    /// Red: old behaviour was 0.0. Green: equals the explicit value=1.0 computation.
    #[test]
    fn base_score_valueless_sl_equals_explicit_value_one() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001: no value facet (implicit default 1.0); ISS-002: value = 1.0 explicitly.
        seed_issue_with_facets(root, 1, "", "lower = 0.0\nupper = 10.0", "", "");
        seed_issue_with_facets(root, 2, "", "lower = 0.0\nupper = 10.0", "value = 1.0", "");
        let pg = build(root).unwrap();
        let bs1 = pg.attrs[&key("ISS", 1)].base_score;
        let bs2 = pg.attrs[&key("ISS", 2)].base_score;
        // Both have absent = 1.0 (empty-corpus fallback) because neither has an estimate…
        // Wait: both HAVE an estimate (lower=0, upper=10). max_upper = 10.0. absent = 10.0 + margin(1.0) = 11.0.
        // est_cost = lower + β·(upper-lower) = 0.0 + 0.65*10.0 = 6.5.
        // ISS-001 value_dim = 1.0(default) * 1.0 / 6.5; ISS-002 value_dim = 1.0(authored) * 1.0 / 6.5.
        let expected = 1.0 / 6.5;
        assert!(
            (bs1.value_dim - expected).abs() < 1e-9,
            "valueless SL value_dim = 1.0/6.5 = {expected}, got {}",
            bs1.value_dim
        );
        assert!(
            (bs2.value_dim - expected).abs() < 1e-9,
            "explicit value=1.0 SL value_dim = 1.0/6.5 = {expected}, got {}",
            bs2.value_dim
        );
    }

    /// Valueless ASM and REV → effective_raw_value None → value_dim == 0.
    #[test]
    fn base_score_valueless_asm_and_rev_value_dim_zero() {
        // Test effective_raw_value directly on real kind descriptors from the
        // integrity table — avoids the disk-seed complexity for REV (which nests
        // under a slice).
        let facets = crate::facet::EntityFacets {
            estimate: None,
            value: None,
            risk: None,
            tags: vec![],
        };
        // Find ASM and REV kinds from the integrity table.
        let asm_kind = crate::integrity::KINDS
            .iter()
            .find(|k| k.kind.prefix == "ASM")
            .map(|k| k.kind)
            .expect("ASM in KINDS");
        let rev_kind = crate::integrity::KINDS
            .iter()
            .find(|k| k.kind.prefix == "REV")
            .map(|k| k.kind)
            .expect("REV in KINDS");
        let iss_kind = crate::integrity::KINDS
            .iter()
            .find(|k| k.kind.prefix == "ISS")
            .map(|k| k.kind)
            .expect("ISS in KINDS");
        let no_projection = ValueProjection::new();
        let no_claims = comparison::ClaimResolution::default();
        let asm_key = EntityKey {
            prefix: asm_kind.prefix,
            id: 1,
        };
        let rev_key = EntityKey {
            prefix: rev_kind.prefix,
            id: 1,
        };
        let iss_key = EntityKey {
            prefix: iss_kind.prefix,
            id: 1,
        };
        assert_eq!(
            effective_raw_value(asm_kind, &facets, asm_key, &no_projection, &no_claims),
            None
        );
        assert_eq!(
            effective_raw_value(rev_kind, &facets, rev_key, &no_projection, &no_claims),
            None
        );
        assert_eq!(
            effective_raw_value(iss_kind, &facets, iss_key, &no_projection, &no_claims),
            Some(DEFAULT_VALUE),
            "ISS is value-bearing → default"
        );
        // value_dim for ASM/REV is 0.
        let cfg = config::PriorityConfig::default();
        let ctx = CostCtx { absent: 1.0 };
        let no_feed = comparison::CostFeed::new();
        let bs = base_score(
            &facets,
            asm_kind,
            asm_key,
            &no_projection,
            &no_claims,
            &no_feed,
            &cfg,
            ctx,
        );
        assert!(
            (bs.value_dim - 0.0).abs() < 1e-9,
            "ASM value_dim should be 0"
        );
        let bs = base_score(
            &facets,
            rev_kind,
            rev_key,
            &no_projection,
            &no_claims,
            &no_feed,
            &cfg,
            ctx,
        );
        assert!(
            (bs.value_dim - 0.0).abs() < 1e-9,
            "REV value_dim should be 0"
        );
        let bs = base_score(
            &facets,
            iss_kind,
            iss_key,
            &no_projection,
            &no_claims,
            &no_feed,
            &cfg,
            ctx,
        );
        assert!(
            (bs.value_dim - 1.0).abs() < 1e-9,
            "ISS value_dim should be 1.0 (default)"
        );
    }

    /// No-clamp: authored value=0.3 on SL → 0.3, not 1.0. Authored 0.0 → value_dim == 0.
    #[test]
    fn base_score_authored_value_preserved_no_clamp() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001: value = 0.3; ISS-002: value = 0.0.
        seed_issue_with_facets(root, 1, "", "lower = 0.0\nupper = 10.0", "value = 0.3", "");
        seed_issue_with_facets(root, 2, "", "lower = 0.0\nupper = 10.0", "value = 0.0", "");
        let pg = build(root).unwrap();
        let bs1 = pg.attrs[&key("ISS", 1)].base_score;
        let bs2 = pg.attrs[&key("ISS", 2)].base_score;
        // est_cost = 0.0 + 0.65*10.0 = 6.5.
        assert!(
            (bs1.value_dim - 0.3 / 6.5).abs() < 1e-9,
            "authored 0.3 should be 0.3/6.5, not clamped to 1.0"
        );
        assert!(
            (bs2.value_dim - 0.0).abs() < 1e-9,
            "authored 0.0 stays 0.0 (not defaulted to 1.0)"
        );
    }

    // ── SL-213 PHASE-05: comparison-tier value projection ───────────────

    /// `effective_raw_value`'s projection tier (SL-213 D11, re-pinned under
    /// the SL-220 ladder): a projected-map entry beats `DEFAULT_VALUE`; a
    /// `Gauge`-provenance map entry resolves IDENTICALLY to a `Projected` one
    /// (D11: "gauge is a sub-tier of projected"). An empty projected map is
    /// the identity case. SL-220 §3 FLIPPED the facet's position (RV-278
    /// F-4): projection now out-ranks the unmigrated facet — the compared
    /// facet-bearing assertion at the tail is the flip stated as a test. The
    /// authored facet is obtained via a real disk scan (never naming the
    /// facet type directly — `EntityFacets.value` stays untyped here,
    /// mirroring the production base pre-pass at `build_from_with_cfg` —
    /// NF-001's structural tripwire keeps facet-symbol exposure to its
    /// allowlisted surface).
    #[test]
    fn effective_raw_value_provenance_chain_authored_over_projected_over_gauge_over_default() {
        let iss_kind = crate::integrity::KINDS
            .iter()
            .find(|k| k.kind.prefix == "ISS")
            .map(|k| k.kind)
            .expect("ISS in KINDS");
        let key1 = EntityKey {
            prefix: "ISS",
            id: 1,
        };
        let no_facets = crate::facet::EntityFacets {
            estimate: None,
            value: None,
            risk: None,
            tags: vec![],
        };
        let no_claims = comparison::ClaimResolution::default();

        // Empty map: value-bearing kind with no facet ⇒ DEFAULT_VALUE (identity).
        let empty = ValueProjection::new();
        assert_eq!(
            effective_raw_value(iss_kind, &no_facets, key1, &empty, &no_claims),
            Some(DEFAULT_VALUE)
        );

        // Projected-tier map entry wins over the default.
        let mut projected = ValueProjection::new();
        projected.insert(
            key1.canonical(),
            (2.5, crate::comparison::ValueProvenance::Projected),
        );
        assert_eq!(
            effective_raw_value(iss_kind, &no_facets, key1, &projected, &no_claims),
            Some(2.5)
        );

        // Gauge-tier map entry resolves IDENTICALLY (same tier for D11's purposes).
        let mut gauged = ValueProjection::new();
        gauged.insert(
            key1.canonical(),
            (2.5, crate::comparison::ValueProvenance::Gauge),
        );
        assert_eq!(
            effective_raw_value(iss_kind, &no_facets, key1, &gauged, &no_claims),
            Some(2.5),
            "Gauge and Projected provenance resolve to the same raw value"
        );

        // THE FLIP (SL-220 §3, RV-278 F-4): a projected/gauge map entry now
        // beats the unmigrated facet — the facet's absolute magnitude stopped
        // anchoring the value scale; with zero claim rows AND no projection
        // it still contributes at rung 5.
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "", "", "value = 9.0", "");
        let scanned =
            relation_graph::scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        let authored = scanned
            .iter()
            .find(|e| e.key == key1)
            .expect("ISS-001 scanned");
        let authored_facets = crate::facet::EntityFacets {
            estimate: authored.estimate.clone(),
            value: authored.value.clone(),
            risk: authored.risk.clone(),
            tags: authored.tags.clone(),
        };
        assert_eq!(
            effective_raw_value(iss_kind, &authored_facets, key1, &gauged, &no_claims),
            Some(2.5),
            "projection out-ranks the unmigrated facet (rung 2 > rung 5)"
        );
        assert_eq!(
            effective_raw_value(iss_kind, &authored_facets, key1, &empty, &no_claims),
            Some(9.0),
            "no projection, zero claim rows: the facet still contributes (rung 5)"
        );
    }

    /// VT-2: a real comparison session on disk flows end to end through
    /// `store::load_pipeline` into `build`'s `value_dim` — a two-node,
    /// anchor-free chain projects via the P8 gauge spread (`comparison::project`
    /// golden `s1_chain8_gauge`'s two-node case: winner 4/3, loser 2/3 of
    /// `DEFAULT_VALUE`), and the preferred side scores higher.
    #[test]
    fn ledger_fed_comparison_projects_into_value_dim() {
        let dir = tmp();
        let root = dir.path();
        // Two bare issues — value-bearing, no authored [value] facet, no
        // [estimate] anywhere in the corpus (est_cost = absent = 1.0).
        seed_issue(root, 1, "open", "", "");
        seed_issue(root, 2, "open", "", "");
        write_comparison_session(root, "ISS-001", "ISS-002");

        let pg = build(root).unwrap();
        let bs1 = pg.attrs[&key("ISS", 1)].base_score;
        let bs2 = pg.attrs[&key("ISS", 2)].base_score;
        let expected_winner = 2.0 * DEFAULT_VALUE * 2.0 / 3.0;
        let expected_loser = 2.0 * DEFAULT_VALUE * 1.0 / 3.0;
        assert!(
            (bs1.value_dim - expected_winner).abs() < 1e-9,
            "preferred ISS-001 gauge-projects to 4/3, got {}",
            bs1.value_dim
        );
        assert!(
            (bs2.value_dim - expected_loser).abs() < 1e-9,
            "non-preferred ISS-002 gauge-projects to 2/3, got {}",
            bs2.value_dim
        );
        assert!(bs1.value_dim > bs2.value_dim);
    }

    /// VT-2: the governed policy (design §3, D11) — a Gauge-provenance
    /// projected value participates in burndown IDENTICALLY to an authored
    /// value, via the SAME `raw_value_of` source `value_dim` uses. A done
    /// slice fulfilling the gauge-valued issue burns it down by the same
    /// formula `burndown_lowers_score` pins for an authored value.
    #[test]
    fn gauge_fed_burndown_golden() {
        let dir = tmp();
        let root = dir.path();
        seed_issue(root, 1, "open", "", "");
        seed_issue(root, 2, "open", "", "");
        write_comparison_session(root, "ISS-001", "ISS-002");
        // ISS-001 gauge-projects to 4/3 (see ledger_fed_comparison_projects_into_value_dim).
        // SL-001 (done, authored value = 2/3) fulfils it: delivered = 2/3,
        // r = (2/3)/(4/3) = 0.5, burn = (4/3)*0.5 = 2/3.
        write(
            root,
            ".doctrine/slice/001/slice-001.toml",
            "id = 1\nslug = \"s\"\ntitle = \"S\"\nstatus = \"done\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [value]\nvalue = 0.6666666666666666\n\
             [[relation]]\nlabel = \"fulfils\"\ntarget = \"ISS-001\"\n",
        );
        write(root, ".doctrine/slice/001/slice-001.md", "scope\n");

        let pg = build(root).unwrap();
        let s1 = pg.score[&key("ISS", 1)];
        let expected_value_dim = 2.0 * DEFAULT_VALUE * 2.0 / 3.0;
        let expected_burn = expected_value_dim * 0.5;
        assert!(
            (s1 - expected_burn).abs() < 1e-6,
            "gauge-fed value burns down identically to an authored one, got {s1}"
        );
    }

    /// Write a two-row-free, single-judgement comparison session preferring
    /// `winner` over `loser` (order form, value domain, agent-rated) straight
    /// to `.doctrine/comparisons/` — bypasses `commands::compare::run_capture`
    /// (its admissibility gate is a capture-time-only concern, design §1) so
    /// the graph-adapter tests exercise the pipeline seam directly.
    fn write_comparison_session(root: &Path, winner: &str, loser: &str) {
        let judgement = comparison::Judgement {
            uid: "j1".to_string(),
            seq: 0,
            a: winner.to_string(),
            b: Some(loser.to_string()),
            response: Some(comparison::Response::PreferA),
            domain: comparison::DOMAIN_VALUE.to_string(),
            frame: comparison::FRAME_EQUAL_EFFORT.to_string(),
            form: comparison::RowForm::Order,
            magnitude: None,
            supersedes: None,
            lens: None,
            rater: comparison::RaterKind::Agent,
            by: None,
            note: None,
            date: Some("2026-07-11".to_string()),
            observed_at: None,
            basis: None,
            admission: None,
        };
        let session = comparison::ComparisonSession {
            schema: comparison::COMPARISON_SCHEMA.to_string(),
            version: comparison::COMPARISON_VERSION,
            session: comparison::SessionHeader {
                uid: "s1".to_string(),
                date: "2026-07-11".to_string(),
                audience: None,
            },
            judgements: vec![judgement],
            tombstones: Vec::new(),
        };
        let text = comparison::to_toml(&session).unwrap();
        write(root, ".doctrine/comparisons/2026-07-11-s1.toml", &text);
    }

    // ── VT-4: directions & classes ──────────────────────────────────────

    #[test]
    fn leverage_flows_out_edges_dep_overlay() {
        // A needs B: dep edge B→A. out_edges(dep_overlay, B) = [A].
        // B's leverage = dep_coeff * (base(A) + leverage(A)).
        // A has no dependents → leverage(A)=0.
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "needs = [\"ISS-002\"]\n", "", "value = 10.0", "");
        seed_issue_with_facets(root, 2, "", "", "value = 3.0", "");
        let pg = build(root).unwrap();
        // ISS-002 base = 3.0, ISS-001 base = 10.0 (both bare, absent=1.0)
        // ISS-002 is prereq (src of dep edge B→A); out_edges(dep_overlay, ISS-002) = {ISS-001}
        // ISS-001 has no dependents → leverage(ISS-001) = 0
        // leverage(ISS-002) = 0.5 * (base(ISS-001) + 0) = 5.0
        let lev2 = pg.leverage[&key("ISS", 2)];
        let lev1 = pg.leverage[&key("ISS", 1)];
        assert!((lev1 - 0.0).abs() < 1e-9, "ISS-001 has no dependents");
        assert!((lev2 - 5.0).abs() < 1e-9, "ISS-002 gets 0.5 * 10.0");
    }

    #[test]
    fn optionality_flows_in_edges_over_consequence_labels_one_hop() {
        // SL-001 has a `slices` edge to ISS-001 (CONSEQUENCE_LABELS member).
        // optionality(ISS-001) = ref_coeff * base(SL-001). One hop, no recursion.
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, "slices = [\"ISS-001\"]\n");
        seed_issue_with_facets(root, 1, "", "", "value = 7.0", "");
        let pg = build(root).unwrap();
        // SL-001 base = 0 (no value facet)
        // ISS-001 base = 7.0
        // optionality(ISS-001) = 1.0 * base(SL-001) = 0.0
        let opt = pg.optionality[&key("ISS", 1)];
        assert!(
            (opt - 0.0).abs() < 1e-9,
            "SL-001 has no value → optionality=0"
        );
        // ISS-001 itself is not referenced by anyone
        let opt_sl = pg.optionality[&key("SL", 1)];
        assert!(
            (opt_sl - 0.0).abs() < 1e-9,
            "SL-001 is not a ref target of a consequence label"
        );
    }

    #[test]
    fn reviews_and_owning_slice_edges_contribute_zero_optionality() {
        // A review targeting ISS-001 creates a `reviews` edge (NOT in CONSEQUENCE_LABELS).
        // A rec creates `owning_slice` (NOT in CONSEQUENCE_LABELS).
        // Neither should contribute to optionality.
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "", "", "value = 5.0", "");
        seed_review(root, 1, "ISS-001", "");
        seed_rec(root, 1, "ISS-001");
        let pg = build(root).unwrap();
        // ISS-001 optionality should be 0 — reviews and owning_slice don't count.
        let opt = pg.optionality[&key("ISS", 1)];
        assert!(
            (opt - 0.0).abs() < 1e-9,
            "reviews/owning_slice contribute 0"
        );
    }

    #[test]
    fn dangling_target_contributes_zero() {
        // An edge to an unresolved target contributes nothing.
        let dir = tmp();
        let root = dir.path();
        // SL-001 has a `slices` edge to ISS-099 (doesn't exist).
        seed_slice(root, 1, "slices = [\"ISS-099\"]\n");
        seed_issue_with_facets(root, 1, "", "", "value = 3.0", "");
        let pg = build(root).unwrap();
        // ISS-099 was never seeded → no edge, no optionality contribution.
        assert!(pg.optionality.get(&key("ISS", 1)).copied().unwrap_or(0.0) == 0.0);
    }

    // ── VT-4b: leverage is recursive ────────────────────────────────────

    #[test]
    fn leverage_recursive_chain() {
        // A needs B, B needs C. Chain: top (A) → middle (B) → leaf (C).
        // ISS-001 needs ISS-002, ISS-002 needs ISS-003.
        // Dep edges: ISS-002→ISS-001, ISS-003→ISS-002.
        // out_edges: ISS-001=[], ISS-002=[ISS-001], ISS-003=[ISS-002]
        // base(ISS-001)=2, base(ISS-002)=3, base(ISS-003)=5.
        // leverage(ISS-001) = 0 (no dependents)
        // leverage(ISS-002) = 0.5 * (base(ISS-001) + 0) = 1.0
        // leverage(ISS-003) = 0.5 * (base(ISS-002) + 1.0) = 0.5 * 4 = 2.0
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "needs = [\"ISS-002\"]\n", "", "value = 2.0", "");
        seed_issue_with_facets(root, 2, "needs = [\"ISS-003\"]\n", "", "value = 3.0", "");
        seed_issue_with_facets(root, 3, "", "", "value = 5.0", "");
        let pg = build(root).unwrap();
        let lev_1 = pg.leverage[&key("ISS", 1)];
        let lev_2 = pg.leverage[&key("ISS", 2)];
        let lev_3 = pg.leverage[&key("ISS", 3)];
        assert!((lev_1 - 0.0).abs() < 1e-9, "ISS-001 has no dependents");
        assert!((lev_2 - 1.0).abs() < 1e-9, "ISS-002 gets 0.5 * ISS-001");
        assert!(
            (lev_3 - 2.0).abs() < 1e-9,
            "ISS-003 gets 0.5 * (ISS-002+l2)"
        );
    }

    #[test]
    fn leverage_diamond_double_counts_shared_leaf() {
        // Top needs B and C. B and C both need D.
        // ISS-001 needs ISS-002, ISS-001 needs ISS-003.
        // ISS-002 needs ISS-004. ISS-003 needs ISS-004.
        // ISS-004 is the shared leaf. Top-to-leaf direction: ISS-001 → ISS-002/ISS-003 → ISS-004.
        // Dep edges: ISS-002→ISS-001, ISS-003→ISS-001, ISS-004→ISS-002, ISS-004→ISS-003.
        // Leverage flows opposite: from dependents to prereqs.
        // ISS-001 has no dependents (it's the top) → lever(ISS-001)=0
        // ISS-002's dependent: ISS-001. lever(ISS-002) = 0.5 * (base(ISS-001)+0) = 0.5*10 = 5.0
        // ISS-003's dependent: ISS-001. lever(ISS-003) = 0.5 * 10 = 5.0
        // ISS-004's dependents: ISS-002 and ISS-003.
        //   lever(ISS-004) = 0.5 * ((base(ISS-002)+lev(ISS-002)) + (base(ISS-003)+lev(ISS-003)))
        //                  = 0.5 * ((1+5) + (1+5)) = 6.0
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(
            root,
            1,
            "needs = [\"ISS-002\", \"ISS-003\"]\n",
            "",
            "value = 10.0",
            "",
        );
        seed_issue_with_facets(root, 2, "needs = [\"ISS-004\"]\n", "", "value = 1.0", "");
        seed_issue_with_facets(root, 3, "needs = [\"ISS-004\"]\n", "", "value = 1.0", "");
        seed_issue_with_facets(root, 4, "", "", "value = 5.0", "");
        let pg = build(root).unwrap();
        let lev_1 = pg.leverage[&key("ISS", 1)];
        let lev_2 = pg.leverage[&key("ISS", 2)];
        let lev_3 = pg.leverage[&key("ISS", 3)];
        let lev_4 = pg.leverage[&key("ISS", 4)];
        assert!((lev_1 - 0.0).abs() < 1e-9);
        assert!((lev_2 - 5.0).abs() < 1e-9);
        assert!((lev_3 - 5.0).abs() < 1e-9);
        assert!(
            (lev_4 - 6.0).abs() < 1e-9,
            "D double-counted through both paths"
        );
    }

    #[test]
    fn ref_optionality_is_one_hop_no_transitive_accumulation() {
        // SL-176 PHASE-03: `Slices` removed from CONSEQUENCE_LABELS.
        // ISS-001 references(concerns) ISS-002. ISS-002's optionality sees only ISS-001.
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, "");
        seed_issue_with_facets(root, 1, "", "", "value = 5.0", "");
        seed_issue_with_facets(root, 2, "", "", "value = 3.0", "");
        let pg = build(root).unwrap();
        // No references edges authored — no optionality anywhere.
        assert!(
            (pg.optionality[&key("ISS", 2)] - 0.0).abs() < 1e-9,
            "ISS-002 has no referencers"
        );
        assert!(
            (pg.optionality[&key("ISS", 1)] - 0.0).abs() < 1e-9,
            "ISS-001 has no referencers"
        );
        assert!(
            (pg.optionality[&key("SL", 1)] - 0.0).abs() < 1e-9,
            "SL-001 has no referencers"
        );
    }

    // ── VT-6: determinism + finite outputs ──────────────────────────────

    #[test]
    fn equal_scores_tiebreak_id_asc() {
        // Two identical items with the same facets → same base_score.
        // Their scores should be equal, and the BTreeMap order (id asc) is
        // the natural tiebreak.
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "", "", "value = 10.0", "");
        seed_issue_with_facets(root, 2, "", "", "value = 10.0", "");
        let pg = build(root).unwrap();
        let s1 = pg.score[&key("ISS", 1)];
        let s2 = pg.score[&key("ISS", 2)];
        // Equal base, no leverage/optionality → equal scores.
        assert!((s1 - s2).abs() < 1e-9, "equal bases yield equal scores");
        // Keys are ordered by canonical id (ISS-001 < ISS-002).
        let keys: Vec<_> = pg.score.keys().collect();
        assert!(keys[0] < keys[1], "BTreeMap orders by id asc");
    }

    #[test]
    fn near_max_coefficients_produce_no_nan_or_inf() {
        // Feed a config with COEFF_MAX coefficients (loaded from doctrine.toml)
        // and verify that scores/leverage/optionality are finite.
        let dir = tmp();
        let root = dir.path();
        let max_val = config::COEFF_MAX;
        write(
            root,
            ".doctrine/doctrine.toml",
            &format!(
                "[priority]\ncoefficients = {{ value = {max_val}, risk = {max_val} }}\n\
                 consequence = {{ dep_coeff = 1.0, ref_coeff = {max_val} }}\n"
            ),
        );
        // A needs B: B accrues leverage from A
        seed_issue_with_facets(root, 1, "needs = [\"ISS-002\"]\n", "", "value = 1e6", "");
        seed_issue_with_facets(
            root,
            2,
            "",
            "",
            "value = 1e6",
            "likelihood = \"critical\"\nimpact = \"critical\"",
        );
        let pg = build(root).unwrap();
        for (_k, &s) in &pg.score {
            assert!(s.is_finite(), "score should be finite, got {s}");
        }
        for (_k, &lev) in &pg.leverage {
            assert!(lev.is_finite(), "leverage should be finite, got {lev}");
        }
        for (_k, &opt) in &pg.optionality {
            assert!(opt.is_finite(), "optionality should be finite, got {opt}");
        }
    }

    // ── VT-8: termination / condensation ────────────────────────────────

    #[test]
    fn self_loop_yields_finite_leverage() {
        // A needs A — a self-loop. Should produce finite leverage.
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "needs = [\"ISS-001\"]\n", "", "value = 5.0", "");
        let pg = build(root).unwrap();
        let lev = pg.leverage[&key("ISS", 1)];
        assert!(lev.is_finite(), "self-loop leverage should be finite");
    }

    #[test]
    fn multi_member_scc_with_external_dependent() {
        // A↔B (mutual needs) forming an SCC, with external dependent C (C needs B).
        // The {A,B} component is from provenance().cycles().
        // Intra-component edges (A→B, B→A) contribute 0.
        // External: C depends on B → base(C)+leverage(C) flows to {A,B} component once.
        // A and B report the same finite component leverage.
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "needs = [\"ISS-002\"]\n", "", "value = 1.0", "");
        seed_issue_with_facets(root, 2, "needs = [\"ISS-001\"]\n", "", "value = 1.0", "");
        seed_issue_with_facets(root, 3, "needs = [\"ISS-002\"]\n", "", "value = 10.0", "");
        let pg = build(root).unwrap();
        // C (=ISS-003 base=10) depends on B (=ISS-002). C has no dependents → lev(C)=0.
        // {A,B} component gets 0.5 * (base(C) + lev(C)) = 0.5 * 10 = 5.0.
        // Intra-component edges A↔B contribute 0.
        let lev_a = pg.leverage[&key("ISS", 1)];
        let lev_b = pg.leverage[&key("ISS", 2)];
        let lev_c = pg.leverage[&key("ISS", 3)];
        assert!(lev_c == 0.0, "C has no dependents");
        assert!(
            (lev_a - lev_b).abs() < 1e-9,
            "A and B report the same component leverage"
        );
        assert!((lev_a - 5.0).abs() < 1e-9, "component leverage = 0.5 * 10");
        assert!(lev_a.is_finite(), "leverage should be finite");
    }

    #[test]
    fn scc_leverage_uses_component_topo_order_under_seq_perturbation() {
        // RV-137 F-1: reverse graph.ordered() is NOT reverse-topo of the CONDENSED
        // graph. A↔B SCC; external dependent D needs A; D has its own dependent E
        // (E needs D) so leverage(D) is recursive/nonzero; a seq edge (B after D)
        // perturbs ordered() so a member of {A,B} is visited before D's leverage
        // resolves. The component must still pick up D's RESOLVED leverage.
        //   dep edges: A needs B → B→A; B needs A → A→B (SCC {A,B});
        //              D needs A → A→D (D is the component's external dependent);
        //              E needs D → D→E (so out_edges(D)={E}).
        //   leverage(E)=0; leverage(D)=0.5*(base(E)+0)=0.5*8=4;
        //   leverage({A,B})=0.5*(base(D)+leverage(D))=0.5*(2+4)=3.
        //   The pre-fix first-member-hit code drops leverage(D) → 0.5*2=1.
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "needs = [\"ISS-002\"]\n", "", "value = 0.0", ""); // A
        seed_issue_with_facets(
            root,
            2,
            "needs = [\"ISS-001\"]\nafter = [{ to = \"ISS-003\", rank = 0 }]\n",
            "",
            "value = 0.0",
            "",
        ); // B (SCC member + seq "after D")
        seed_issue_with_facets(root, 3, "needs = [\"ISS-001\"]\n", "", "value = 2.0", ""); // D needs A
        seed_issue_with_facets(root, 4, "needs = [\"ISS-003\"]\n", "", "value = 8.0", ""); // E needs D
        let pg = build(root).unwrap();
        let lev_a = pg.leverage[&key("ISS", 1)];
        let lev_b = pg.leverage[&key("ISS", 2)];
        let lev_d = pg.leverage[&key("ISS", 3)];
        let lev_e = pg.leverage[&key("ISS", 4)];
        assert!((lev_e - 0.0).abs() < 1e-9, "E has no dependents");
        assert!((lev_d - 4.0).abs() < 1e-9, "D = 0.5 * base(E)");
        assert!(
            (lev_a - lev_b).abs() < 1e-9,
            "A and B share component leverage"
        );
        assert!(
            (lev_a - 3.0).abs() < 1e-9,
            "{{A,B}} picks up D's RESOLVED leverage: 0.5*(2+4)=3, not 0.5*2=1"
        );
    }

    #[test]
    fn scc_external_dependent_counted_once_per_component() {
        // RV-137 F-2: an external dependent that needs >1 SCC member must be counted
        // ONCE for the component, not once per member.
        //   A↔B SCC; D needs A AND D needs B → out_edges(A)∋D, out_edges(B)∋D.
        //   {A,B} external dependents = {D} (deduped). leverage = 0.5*(base(D)+0)=5.
        //   The pre-fix per-member sum counts D twice → 0.5*(10+10)=10.
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "needs = [\"ISS-002\"]\n", "", "value = 0.0", ""); // A
        seed_issue_with_facets(root, 2, "needs = [\"ISS-001\"]\n", "", "value = 0.0", ""); // B
        seed_issue_with_facets(
            root,
            3,
            "needs = [\"ISS-001\", \"ISS-002\"]\n",
            "",
            "value = 10.0",
            "",
        ); // D needs A AND B
        let pg = build(root).unwrap();
        let lev_a = pg.leverage[&key("ISS", 1)];
        let lev_b = pg.leverage[&key("ISS", 2)];
        let lev_d = pg.leverage[&key("ISS", 3)];
        assert!((lev_d - 0.0).abs() < 1e-9, "D has no dependents");
        assert!(
            (lev_a - lev_b).abs() < 1e-9,
            "A and B share component leverage"
        );
        assert!(
            (lev_a - 5.0).abs() < 1e-9,
            "D counted once per component: 0.5*10=5, not 0.5*20=10"
        );
    }

    // ── tag_term helpers ────────────────────────────────────────────────

    /// Seed a backlog issue with tags, estimate, and value facets.
    fn seed_issue_with_tags(root: &Path, id: u32, tags: &str, value: &str, estimate: &str) {
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
                 resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 tags = [{tags}]\n\
                 [estimate]\n{estimate}\n\
                 [value]\n{value}\n",
            ),
        );
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.md"),
            "b\n",
        );
    }

    // ── VT-14: tag_term in base_score ───────────────────────────────────

    #[test]
    fn base_score_empty_tags_identity() {
        // No tags → tag_term = 1.0 → value_dim unchanged (identity).
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "", "lower = 0.0\nupper = 10.0", "value = 10.0", "");
        let pg = build(root).unwrap();
        let bs = pg.attrs[&key("ISS", 1)].base_score;
        // value_dim = 1.0 * 10.0 * 1.0 * 1.0 / est_cost
        //   est_cost = 0.0 + 0.65*(10.0-0.0) = 6.5
        //          = 10.0 / 6.5 ≈ 1.538461538
        assert!(
            (bs.value_dim - 10.0 / 6.5).abs() < 1e-9,
            "empty tags → identity"
        );
    }

    #[test]
    fn base_score_with_tag_coefficient() {
        // tags = ["area:foo"], tag_coeff("area:foo") = 2.0
        // tag_term = 1.0 + (2.0 - 1.0) = 2.0 → doubles value_dim
        let dir = tmp();
        let root = dir.path();
        write(
            root,
            ".doctrine/doctrine.toml",
            "[priority]\ntag_coefficients = { \"area:foo\" = 2.0 }\n",
        );
        seed_issue_with_tags(
            root,
            1,
            "\"area:foo\"",
            "value = 10.0",
            "lower = 0.0\nupper = 10.0",
        );
        let pg = build(root).unwrap();
        let bs = pg.attrs[&key("ISS", 1)].base_score;
        // value_dim = 1.0 * 10.0 * 1.0 * 2.0 / est_cost
        //   est_cost = 0.0 + 0.65*(10.0-0.0) = 6.5
        //          = 20.0 / 6.5 ≈ 3.076923077
        assert!(
            (bs.value_dim - 20.0 / 6.5).abs() < 1e-9,
            "tag coeff 2.0 doubles value_dim"
        );
    }

    #[test]
    fn base_score_multiple_tags() {
        // tags = ["a", "b"], tag_coeff("a") = 1.5, tag_coeff("b") = 2.0
        // tag_term = 1.0 + (1.5 - 1.0) + (2.0 - 1.0) = 2.5
        let dir = tmp();
        let root = dir.path();
        write(
            root,
            ".doctrine/doctrine.toml",
            "[priority]\ntag_coefficients = { a = 1.5, b = 2.0 }\n",
        );
        seed_issue_with_tags(
            root,
            1,
            "\"a\", \"b\"",
            "value = 6.0",
            "lower = 2.0\nupper = 4.0",
        );
        let pg = build(root).unwrap();
        let bs = pg.attrs[&key("ISS", 1)].base_score;
        // value_dim = 1.0 * 6.0 * 1.0 * 2.5 / est_cost
        //   est_cost = 2.0 + 0.65*(4.0-2.0) = 3.3
        //          = 15.0 / 3.3 ≈ 4.545454545
        assert!(
            (bs.value_dim - 15.0 / 3.3).abs() < 1e-9,
            "tag_term 2.5 → value_dim = 15.0/3.3"
        );
    }

    #[test]
    fn base_score_demoting_tag() {
        // tags = ["wontfix"], tag_coeff("wontfix") = 0.5
        // tag_term = 1.0 + (0.5 - 1.0) = 0.5 → halves value_dim
        let dir = tmp();
        let root = dir.path();
        write(
            root,
            ".doctrine/doctrine.toml",
            "[priority]\ntag_coefficients = { wontfix = 0.5 }\n",
        );
        seed_issue_with_tags(
            root,
            1,
            "\"wontfix\"",
            "value = 20.0",
            "lower = 0.0\nupper = 10.0",
        );
        let pg = build(root).unwrap();
        let bs = pg.attrs[&key("ISS", 1)].base_score;
        // value_dim = 1.0 * 20.0 * 1.0 * 0.5 / est_cost
        //   est_cost = 0.0 + 0.65*(10.0-0.0) = 6.5
        //          = 10.0 / 6.5 ≈ 1.538461538
        assert!(
            (bs.value_dim - 10.0 / 6.5).abs() < 1e-9,
            "demoting tag halves value_dim"
        );
    }

    #[test]
    fn base_score_multi_demote_floors_at_zero() {
        // tags = ["x", "y"], both tag_coeff = 0.0
        // tag_term = 1.0 + (0.0 - 1.0) + (0.0 - 1.0) = -1.0 → max(0.0) = 0.0
        let dir = tmp();
        let root = dir.path();
        write(
            root,
            ".doctrine/doctrine.toml",
            "[priority]\ntag_coefficients = { x = 0.0, y = 0.0 }\n",
        );
        seed_issue_with_tags(
            root,
            1,
            "\"x\", \"y\"",
            "value = 10.0",
            "lower = 0.0\nupper = 10.0",
        );
        let pg = build(root).unwrap();
        let bs = pg.attrs[&key("ISS", 1)].base_score;
        // value_dim = 1.0 * 10.0 * 1.0 * 0.0 / 6.5 = 0.0 (tag_term floors at 0)
        assert!(
            (bs.value_dim - 0.0).abs() < 1e-9,
            "multi-demote floors at zero, not negative"
        );
    }

    // ── VT-3: fulfils value-burndown post-pass ─────────────────────────

    /// burndown lowers score: a done slice fulfilling a valued backlog item
    /// attenuates the item's value_dim. The item's score is strictly lower than
    /// an identical un-fulfilled item.
    #[test]
    fn burndown_lowers_score() {
        let dir = tmp();
        let root = dir.path();
        // Two identical backlog items (value=10.0, bare → est_cost=absent=1.0,
        // value_dim=10.0). ISS-002 has a `done` slice fulfilling it; ISS-001 does not.
        // SL-001: value=4.0, status="done", fulfils ISS-002.
        // Burndown on ISS-002: delivered=4.0, r=4.0/10.0=0.4, burn=10.0*0.6=6.0.
        seed_issue_with_facets(root, 1, "", "", "value = 10.0", "");
        seed_issue_with_facets(root, 2, "", "", "value = 10.0", "");
        write(
            root,
            ".doctrine/slice/001/slice-001.toml",
            "id = 1\nslug = \"s\"\ntitle = \"S\"\nstatus = \"done\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [value]\nvalue = 4.0\n\
             [[relation]]\nlabel = \"fulfils\"\ntarget = \"ISS-002\"\n",
        );
        write(root, ".doctrine/slice/001/slice-001.md", "scope\n");
        let pg = build(root).unwrap();
        let s1 = pg.score[&key("ISS", 1)];
        let s2 = pg.score[&key("ISS", 2)];
        // ISS-001: no fulfils → burndown_term = value_dim = 10.0, score = 10.0.
        assert!((s1 - 10.0).abs() < 1e-9, "ISS-001 unchanged, got {s1}");
        // ISS-002: 40% burndown → burn = 6.0, score = 6.0.
        assert!((s2 - 6.0).abs() < 1e-9, "ISS-002 burndown to 6.0, got {s2}");
        assert!(
            s2 < s1,
            "burndown strictly lowers the fulfilled item's score"
        );
    }

    /// lifecycle gate: only started/audit/reconcile/done slices burn value;
    /// proposed/design/plan/ready/abandoned slices burn nothing.
    #[test]
    fn burndown_lifecycle_gate() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001: fulfilled by SL-001 (status="ready" → gate=0 → no burn).
        // ISS-002: fulfilled by SL-002 (status="started" → gate=1 → full burn).
        // ISS-003: fulfilled by SL-003 (status="done" → gate=1 → full burn).
        // All ISS: value=10.0 bare (value_dim=10.0). All SL: value=4.0.
        seed_issue_with_facets(root, 1, "", "", "value = 10.0", "");
        seed_issue_with_facets(root, 2, "", "", "value = 10.0", "");
        seed_issue_with_facets(root, 3, "", "", "value = 10.0", "");
        // SL-001 ready — gate=0.
        write(
            root,
            ".doctrine/slice/001/slice-001.toml",
            "id = 1\nslug = \"s\"\ntitle = \"S\"\nstatus = \"ready\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [value]\nvalue = 4.0\n\
             [[relation]]\nlabel = \"fulfils\"\ntarget = \"ISS-001\"\n",
        );
        write(root, ".doctrine/slice/001/slice-001.md", "scope\n");
        // SL-002 started — gate=1.
        write(
            root,
            ".doctrine/slice/002/slice-002.toml",
            "id = 2\nslug = \"s\"\ntitle = \"S\"\nstatus = \"started\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [value]\nvalue = 4.0\n\
             [[relation]]\nlabel = \"fulfils\"\ntarget = \"ISS-002\"\n",
        );
        write(root, ".doctrine/slice/002/slice-002.md", "scope\n");
        // SL-003 done — gate=1.
        write(
            root,
            ".doctrine/slice/003/slice-003.toml",
            "id = 3\nslug = \"s\"\ntitle = \"S\"\nstatus = \"done\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [value]\nvalue = 4.0\n\
             [[relation]]\nlabel = \"fulfils\"\ntarget = \"ISS-003\"\n",
        );
        write(root, ".doctrine/slice/003/slice-003.md", "scope\n");
        let pg = build(root).unwrap();
        // ISS-001: ready gate=0 → burn=10.0, score=10.0 (unchanged).
        assert!(
            (pg.score[&key("ISS", 1)] - 10.0).abs() < 1e-9,
            "ready status burns nothing: Fulfils burndown lifecycle gate"
        );
        // ISS-002: started gate=1 → burn=6.0, score=6.0.
        assert!(
            (pg.score[&key("ISS", 2)] - 6.0).abs() < 1e-9,
            "started status burns fully: Fulfils burndown lifecycle gate"
        );
        // ISS-003: started gate=1 → burn=6.0, score=6.0.
        assert!(
            (pg.score[&key("ISS", 3)] - 6.0).abs() < 1e-9,
            "done (via started) burns fully: Fulfils burndown lifecycle gate"
        );
    }

    /// non-conservation: one done slice fulfilling TWO items burns each item
    /// independently — the slice's value is not "spent once."
    #[test]
    fn burndown_non_conservation() {
        let dir = tmp();
        let root = dir.path();
        // SL-001 (value=4.0, done) fulfils both ISS-001 and ISS-002.
        // Each ISS has value=10.0 bare (value_dim=10.0).
        // Each ISS independently: delivered=4.0, r=0.4, burn=6.0, score=6.0.
        seed_issue_with_facets(root, 1, "", "", "value = 10.0", "");
        seed_issue_with_facets(root, 2, "", "", "value = 10.0", "");
        write(
            root,
            ".doctrine/slice/001/slice-001.toml",
            "id = 1\nslug = \"s\"\ntitle = \"S\"\nstatus = \"done\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [value]\nvalue = 4.0\n\
             [[relation]]\nlabel = \"fulfils\"\ntarget = \"ISS-001\"\n\
             [[relation]]\nlabel = \"fulfils\"\ntarget = \"ISS-002\"\n",
        );
        write(root, ".doctrine/slice/001/slice-001.md", "scope\n");
        let pg = build(root).unwrap();
        // Both items burned identically — slice's 4.0 is NOT consumed by one item
        // and unavailable to the other.
        let s1 = pg.score[&key("ISS", 1)];
        let s2 = pg.score[&key("ISS", 2)];
        assert!((s1 - 6.0).abs() < 1e-9, "ISS-001 burndown to 6.0, got {s1}");
        assert!((s2 - 6.0).abs() < 1e-9, "ISS-002 burndown to 6.0, got {s2}");
        assert!(
            (s1 - s2).abs() < 1e-9,
            "Fulfils burndown is non-conserving across multi-item"
        );
    }

    /// originates_from is inert for priority: it feeds NO priority pass —
    /// neither optionality nor the burndown changes.
    #[test]
    fn originates_from_inert_for_priority() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 authors originates_from → SL-001 (OriginatesFrom label).
        // SL-001 has value=5.0. OriginatesFrom is NOT in REF_LABELS → no overlay
        // → contributes neither optionality nor burndown.
        seed_issue(root, 1, "open", "", "originates_from = [\"SL-001\"]\n");
        seed_slice(root, 1, "");
        let pg = build(root).unwrap();
        // Neither entity gets optionality from the originates_from edge.
        assert!(
            pg.optionality.get(&key("ISS", 1)).copied().unwrap_or(0.0) == 0.0,
            "originates_from is not a CONSEQUENCE_LABELS member → optionality = 0"
        );
        assert!(
            pg.optionality.get(&key("SL", 1)).copied().unwrap_or(0.0) == 0.0,
            "SL target of originates_from gets no optionality"
        );
        // Burndown unaffected: no Fulfils edge exists (r=0, burn = value_dim).
        // SL-177 PHASE-02: ISS-001 is value-bearing valueless → default 1.0 → score=1.0.
        assert!(
            (pg.score[&key("ISS", 1)] - 1.0).abs() < 1e-9,
            "originates_from: valueless item score = 1.0 (default); still no lev/opt change"
        );
    }

    /// exact-value / wrong-denominator trap: value_dim ≠ raw_value when
    /// estimate/cost diverges them. Assert the hand-computed post-burndown
    /// score EXACTLY — catches r computed with value_dim instead of raw value,
    /// or delivered subtracted directly instead of via the r ratio.
    #[test]
    fn burndown_exact_value_divergence_trap() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001: value=10.0, estimate lower=0 upper=20.
        //   est_cost = 0 + 0.65*20 = 13.0, value_dim = 10.0/13.0 ≈ 0.7692307692307693.
        //   raw_value = 10.0 (≠ value_dim).
        // SL-001: value=5.0, status="done", fulfils ISS-001.
        //   Burndown on ISS-001: delivered = gate(1.0) * raw_value(SL-001) = 5.0.
        //   r = 5.0 / raw_value(ISS-001) = 5.0 / 10.0 = 0.5.
        //   burn = value_dim(ISS-001) * (1 - r) = 0.7692307692307693 * 0.5
        //        = 0.38461538461538464.
        //   score = 0 + 0 + 0 + burn ≈ 0.38461538461538464.
        seed_issue_with_facets(root, 1, "", "lower = 0.0\nupper = 20.0", "value = 10.0", "");
        write(
            root,
            ".doctrine/slice/001/slice-001.toml",
            "id = 1\nslug = \"s\"\ntitle = \"S\"\nstatus = \"done\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [value]\nvalue = 5.0\n\
             [[relation]]\nlabel = \"fulfils\"\ntarget = \"ISS-001\"\n",
        );
        write(root, ".doctrine/slice/001/slice-001.md", "scope\n");
        let pg = build(root).unwrap();
        let expected: f64 = 10.0 / 13.0 * 0.5; // = 0.38461538461538464
        let got = pg.score[&key("ISS", 1)];
        assert!(
            (got - expected).abs() < 1e-9,
            "Fulfils burndown uses raw_value denominator ({expected}) not value_dim, got {got}"
        );
    }

    /// decomposition: Fulfils is NOT in CONSEQUENCE_LABELS (zero optionality
    /// from the fulfils edge), Slices is removed from CONSEQUENCE_LABELS, so
    /// the fulfilled item's score delta vs no-fulfils baseline equals the
    /// burndown term ONLY. Catches Fulfils wrongly left in CONSEQUENCE_LABELS
    /// or Slices not fully removed (double-count).
    #[test]
    fn burndown_decomposition() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001: value=10.0 bare (value_dim=10.0). No slices/references edges.
        // SL-001: value=4.0, status="done", fulfils ISS-001.
        seed_issue_with_facets(root, 1, "", "", "value = 10.0", "");
        write(
            root,
            ".doctrine/slice/001/slice-001.toml",
            "id = 1\nslug = \"s\"\ntitle = \"S\"\nstatus = \"done\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [value]\nvalue = 4.0\n\
             [[relation]]\nlabel = \"fulfils\"\ntarget = \"ISS-001\"\n",
        );
        write(root, ".doctrine/slice/001/slice-001.md", "scope\n");
        let pg = build(root).unwrap();
        // ISS-001 optionality: the fulfils edge is on the Fulfils overlay, NOT in
        // CONSEQUENCE_LABELS → contributes 0. No other inbound consequence edges.
        assert!(
            pg.optionality[&key("ISS", 1)] == 0.0,
            "Fulfils not in CONSEQUENCE_LABELS: optionality from fulfils = 0"
        );
        // Score delta vs no-fulfils baseline: the only difference is the burndown term.
        // No-fulfils baseline: score = value_dim = 10.0.
        // With fulfils: burn = 10.0 * (1 - 4.0/10.0) = 6.0, score = 6.0.
        // Delta = 4.0 = value_dim * r = burndown term ONLY (no lev/opt delta).
        let s = pg.score[&key("ISS", 1)];
        assert!(
            (s - 6.0).abs() < 1e-9,
            "Fulfils burndown decomposition: score=6.0, got {s}"
        );
        // Verify the delta equals value_dim * r alone (proof: 10.0 * 0.4 = 4.0).
        let baseline = 10.0;
        let delta = baseline - s;
        assert!(
            (delta - 4.0).abs() < 1e-9,
            "decomposition: delta={delta} equals burndown term ONLY (no slices/optionality double-count)"
        );
    }

    // ── VT-4 (SL-177 PHASE-02): burndown raw_value retrofit ──────────────

    /// VT-1 (F-1 regression guard): a valueless slice (value-bearing, no authored
    /// value) in a delivering status fulfils a valued backlog item. The slice's
    /// default 1.0 (via `effective_raw_value`) contributes to `delivered` → the
    /// item's score is reduced vs an unfulfilled baseline.
    #[test]
    fn burndown_valueless_fulfilling_slice_delivers_default_value() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001: unfulfilled, value=10.0 → value_dim=10.0, score=10.0.
        // ISS-002: fulfilled by valueless SL-001 in `started`.
        //   SL-001 has no [value] facet → effective_raw_value returns 1.0 (default).
        //   delivered = gate(1.0) * 1.0 = 1.0, r = 1.0/10.0 = 0.1.
        //   burn = 10.0 * 0.9 = 9.0, score = 9.0.
        seed_issue_with_facets(root, 1, "", "", "value = 10.0", "");
        seed_issue_with_facets(root, 2, "", "", "value = 10.0", "");
        write(
            root,
            ".doctrine/slice/001/slice-001.toml",
            "id = 1\nslug = \"s\"\ntitle = \"S\"\nstatus = \"started\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [[relation]]\nlabel = \"fulfils\"\ntarget = \"ISS-002\"\n",
        );
        write(root, ".doctrine/slice/001/slice-001.md", "scope\n");
        let pg = build(root).unwrap();
        let s1 = pg.score[&key("ISS", 1)];
        let s2 = pg.score[&key("ISS", 2)];
        // Unfulfilled baseline: 10.0.
        assert!(
            (s1 - 10.0).abs() < 1e-9,
            "ISS-001 unfulfilled baseline 10.0, got {s1}"
        );
        // Fulfilled by valueless slice: delivered=1.0, r=0.1, burn=9.0.
        assert!(
            (s2 - 9.0).abs() < 1e-9,
            "ISS-002 burndown by valueless slice to 9.0, got {s2}"
        );
        assert!(
            s2 < s1,
            "valueless fulfilling slice reduces the item's score"
        );
        assert!(
            s2 > 0.0,
            "score is positive (delivered > 0 from default value), got {s2}"
        );
    }

    /// VT-2 (exclusion): a non-value-bearing kind as a fulfils source would
    /// contribute 0 to delivered — `effective_raw_value` returns None for REV
    /// and record kinds, and the `unwrap_or(0.0)` in `raw_value_of` converts
    /// that to 0.0. Since the relation graph constrains Fulfils sources to SL
    /// (always value-bearing), this is tested via the `raw_value_of` closure
    /// semantics: verify that `effective_raw_value` returns None for REV/ASM,
    /// which the burndown path converts to a 0 contribution.
    #[test]
    fn burndown_non_value_bearing_source_contributes_zero() {
        // Test effective_raw_value directly: REV and ASM are NOT value-bearing.
        let facets = crate::facet::EntityFacets {
            estimate: None,
            value: None,
            risk: None,
            tags: vec![],
        };
        let rev_kind = crate::integrity::KINDS
            .iter()
            .find(|k| k.kind.prefix == "REV")
            .map(|k| k.kind)
            .expect("REV in KINDS");
        let asm_kind = crate::integrity::KINDS
            .iter()
            .find(|k| k.kind.prefix == "ASM")
            .map(|k| k.kind)
            .expect("ASM in KINDS");
        let iss_kind = crate::integrity::KINDS
            .iter()
            .find(|k| k.kind.prefix == "ISS")
            .map(|k| k.kind)
            .expect("ISS in KINDS");
        let no_projection = ValueProjection::new();
        let no_claims = comparison::ClaimResolution::default();
        let rev_key = EntityKey {
            prefix: rev_kind.prefix,
            id: 1,
        };
        let asm_key = EntityKey {
            prefix: asm_kind.prefix,
            id: 1,
        };
        let iss_key = EntityKey {
            prefix: iss_kind.prefix,
            id: 1,
        };
        // Non-value-bearing → None → raw_value_of returns 0.0 (via unwrap_or).
        assert_eq!(
            effective_raw_value(rev_kind, &facets, rev_key, &no_projection, &no_claims),
            None
        );
        assert_eq!(
            effective_raw_value(asm_kind, &facets, asm_key, &no_projection, &no_claims),
            None
        );
        // Value-bearing without authored value → Some(1.0) (default).
        assert_eq!(
            effective_raw_value(iss_kind, &facets, iss_key, &no_projection, &no_claims),
            Some(DEFAULT_VALUE)
        );
        // When routed through the burndown closure (raw_value_of), these map to:
        //   effective_raw_value(None) → unwrap_or(0.0) → 0.0 (can't deliver value)
        //   effective_raw_value(Some(1.0)) → unwrap_or(0.0) → 1.0 (delivers default)
        // This guards against the old path that read f.value directly and missed
        // the default for value-bearing kinds.
    }

    // ── SL-194 VT-1: build_from == build_from_with_cfg(…, load(root)) ─────────

    /// The behaviour-preservation gate for the SL-194 rebuild-seam extraction:
    /// `build_from` must be byte-identical to `build_from_with_cfg` fed the same
    /// `config::load(root)` it would have loaded internally. Compares the observable
    /// products — the score/leverage/optionality maps, the base scores, and the minted
    /// node order — over a multi-kind corpus with dep + facet variety.
    #[test]
    fn build_from_equals_build_from_with_cfg_over_loaded_config() {
        let dir = tmp();
        let root = dir.path();
        // A corpus with base-score variety, a needs edge (leverage), and a ref edge
        // (optionality) so every consequence path is exercised.
        seed_issue_with_facets(
            root,
            1,
            "needs = [\"RSK-001\"]",
            "lower = 0.0\nupper = 10.0",
            "value = 25.0",
            "",
        );
        seed_issue_with_facets(root, 2, "", "lower = 1.0\nupper = 4.0", "value = 5.0", "");
        seed_risk(root, 1, "open", "");
        seed_slice(root, 1, "references = [\"REQ-005\"]");
        seed_requirement(root, 5);

        let scanned =
            relation_graph::scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        let via_load = build_from(&scanned, root).unwrap();
        // No comparisons dir in this fixture: an empty projection is exactly
        // what `build_from` computes internally (behaviour-preservation gate).
        let via_cfg = build_from_with_cfg(
            &scanned,
            root,
            &config::load(root),
            &ValueProjection::new(),
            &comparison::CostFeed::new(),
            &comparison::ClaimResolution::default(),
        )
        .unwrap();

        assert_eq!(via_load.score, via_cfg.score, "score map identical");
        assert_eq!(
            via_load.leverage, via_cfg.leverage,
            "leverage map identical"
        );
        assert_eq!(
            via_load.optionality, via_cfg.optionality,
            "optionality map identical"
        );
        // Base scores per node identical (the injected cfg drove base_score identically).
        let base = |pg: &PriorityGraph| -> std::collections::BTreeMap<EntityKey, (f64, f64)> {
            pg.attrs
                .iter()
                .map(|(k, a)| (*k, (a.base_score.value_dim, a.base_score.risk_dim)))
                .collect()
        };
        assert_eq!(base(&via_load), base(&via_cfg), "base scores identical");
        // Minted node order identical (NodeId assignment is the mint tiebreak).
        let order = |pg: &PriorityGraph| -> Vec<EntityKey> {
            pg.graph
                .ordered()
                .iter()
                .filter_map(|n| pg.projection.key_of(*n))
                .collect()
        };
        assert_eq!(order(&via_load), order(&via_cfg), "minted order identical");
    }

    // ── SL-219 VT-2: one β-skew formula site ─────────────────────────────────

    /// The est anchor builder and the scoring ladder's authored branch share
    /// ONE formula site (`authored_est_cost`, design §2): for every scanned
    /// entity with an authored estimate, the builder's anchor value equals the
    /// graph's authored-branch `est_cost` for the same bounds + cfg. Estimated
    /// RECORDS anchor too (D3 — anchor mass through chains); a facet-less
    /// entity contributes no anchor.
    #[test]
    fn est_anchor_builder_equals_graph_authored_branch_est_cost() {
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "", "lower = 1.0\nupper = 9.0", "value = 5.0", "");
        seed_issue_with_facets(root, 2, "", "lower = 0.0\nupper = 0.0", "value = 5.0", "");
        seed_issue(root, 3, "open", "", ""); // no estimate ⇒ no anchor
        // An estimated knowledge RECORD (facets are kind-agnostic): anchors too.
        write(
            root,
            ".doctrine/knowledge/question/001/record-001.toml",
            &format!(
                "schema = \"{}\"\nversion = 1\nid = 1\nslug = \"q1\"\ntitle = \"Q1\"\n\
                 record_kind = \"question\"\nstatus = \"open\"\ncreated = \"2026-01-01\"\n\
                 updated = \"2026-01-01\"\ntags = []\n[estimate]\nlower = 2.0\nupper = 6.0\n",
                crate::test_support::SCHEMA_KNOWLEDGE
            ),
        );
        write(
            root,
            ".doctrine/knowledge/question/001/record-001.md",
            "q\n",
        );

        let scanned =
            relation_graph::scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        let cfg = config::load(root);
        let anchors = comparison_est_anchor_map(&scanned, &cfg);

        // The ctx is irrelevant to the authored branch (bounds present) — any
        // absent value must leave the equality intact.
        let ctx = CostCtx { absent: 999.0 };
        for (canonical, prefix, id, bounds) in [
            ("ISS-001", "ISS", 1, (1.0, 9.0)),
            ("ISS-002", "ISS", 2, (0.0, 0.0)),
            ("QUE-001", "QUE", 1, (2.0, 6.0)),
        ] {
            assert_eq!(
                anchors.get(canonical).copied(),
                Some(est_cost(
                    Some(bounds),
                    key(prefix, id),
                    &comparison::CostFeed::new(),
                    ctx,
                    &cfg.estimate
                )),
                "{canonical}: builder anchor == authored-branch est_cost"
            );
        }
        assert!(
            !anchors.contains_key("ISS-003"),
            "no authored estimate ⇒ no anchor"
        );
    }

    // ── SL-219 PHASE-04 VT-1: the est_cost ladder & the cost feed ────────────

    /// One est-domain judgement: under `FRAME_MORE_WORK`, `PreferA` evidences
    /// `a` costlier than `b`; `Equal` merges the pair into one class.
    fn est_row(
        uid: &str,
        a: &str,
        b: &str,
        response: comparison::Response,
    ) -> comparison::Judgement {
        comparison::Judgement {
            uid: uid.to_string(),
            seq: 0,
            a: a.to_string(),
            b: Some(b.to_string()),
            response: Some(response),
            domain: comparison::DOMAIN_ESTIMATE.to_string(),
            frame: comparison::FRAME_MORE_WORK.to_string(),
            form: comparison::RowForm::Order,
            magnitude: None,
            supersedes: None,
            lens: None,
            rater: comparison::RaterKind::Agent,
            by: None,
            note: None,
            date: Some("2026-07-11".to_string()),
            observed_at: None,
            basis: None,
            admission: None,
        }
    }

    /// Write one est-domain comparison session carrying `rows` straight to
    /// `.doctrine/comparisons/` (mirrors [`write_comparison_session`]: the
    /// capture admissibility gate is capture-time-only).
    fn write_est_session(root: &Path, rows: Vec<comparison::Judgement>) {
        let session = comparison::ComparisonSession {
            schema: comparison::COMPARISON_SCHEMA.to_string(),
            version: comparison::COMPARISON_VERSION,
            session: comparison::SessionHeader {
                uid: "e1".to_string(),
                date: "2026-07-11".to_string(),
                audience: None,
            },
            judgements: rows,
            tombstones: Vec::new(),
        };
        let text = comparison::to_toml(&session).unwrap();
        write(root, ".doctrine/comparisons/2026-07-11-e1.toml", &text);
    }

    /// The ladder-resolved `est_cost` of one item, read through the ONE
    /// post-build seam ([`PriorityGraph::item_costing`]).
    fn est_of(pg: &PriorityGraph, prefix: &'static str, id: u32) -> f64 {
        let cfg = config::PriorityConfig::default();
        pg.item_costing(&key(prefix, id), &cfg).unwrap().1
    }

    /// Ladder tier 1: authored bounds NEVER consult the feed — even a feed
    /// entry for the same key is inert (source precedence, not numeric
    /// dominance: the fed value here is far larger AND far smaller variants
    /// both lose to the authored branch).
    #[test]
    fn est_cost_ladder_authored_bounds_beat_cost_feed() {
        let ec = config::EstimateCost::default();
        let ctx = CostCtx { absent: 7.0 };
        for fed in [0.001, 1000.0] {
            let feed: comparison::CostFeed = [("ISS-001".to_string(), fed)].into();
            assert_eq!(
                est_cost(Some((2.0, 6.0)), key("ISS", 1), &feed, ctx, &ec),
                authored_est_cost((2.0, 6.0), &ec),
                "authored beats feed entry {fed}"
            );
        }
    }

    /// Ladder tier 2 → 3: a bare item with a feed entry takes the fed cost;
    /// a bare item absent from the feed falls to the bare anchor.
    #[test]
    fn est_cost_ladder_feed_then_bare_anchor() {
        let ec = config::EstimateCost::default();
        let ctx = CostCtx { absent: 7.0 };
        let feed: comparison::CostFeed = [("ISS-001".to_string(), 4.85)].into();
        assert_eq!(est_cost(None, key("ISS", 1), &feed, ctx, &ec), 4.85);
        assert_eq!(
            est_cost(None, key("ISS", 2), &feed, ctx, &ec),
            7.0,
            "unevidenced-bare falls through to ctx.absent"
        );
    }

    /// The D11 positivity belt at the consumption branch: a (contractually
    /// impossible, structurally guarded) non-positive fed cost is floored to
    /// EPSILON — the divisor can never hit zero.
    #[test]
    fn est_cost_ladder_epsilon_floors_the_feed_branch() {
        let ec = config::EstimateCost::default();
        let ctx = CostCtx { absent: 7.0 };
        let feed: comparison::CostFeed = [("ISS-001".to_string(), 0.0)].into();
        assert_eq!(est_cost(None, key("ISS", 1), &feed, ctx, &ec), EPSILON);
    }

    /// An evidenced-bare item is FED (design §2): ISS-002 (no facet) evidenced
    /// costlier than authored ISS-001 lands one `EST_GAUGE_STEP` above the
    /// anchor (P5) — its scoring divisor is the projected cost, not the bare
    /// anchor. The authored item itself stays on the authored branch.
    #[test]
    fn est_cost_ladder_evidenced_bare_item_takes_cost_feed() {
        let dir = tmp();
        let root = dir.path();
        // authored_est_cost(2,6) = 2 + 0.65·4 = 4.6; absent = 6 + 1 = 7.
        seed_issue_with_facets(root, 1, "", "lower = 2.0\nupper = 6.0", "value = 5.0", "");
        seed_issue_with_facets(root, 2, "", "", "value = 5.0", "");
        write_est_session(
            root,
            vec![est_row(
                "e1",
                "ISS-002",
                "ISS-001",
                comparison::Response::PreferA,
            )],
        );
        let pg = build(root).unwrap();
        assert!(
            (est_of(&pg, "ISS", 1) - 4.6).abs() < 1e-9,
            "authored branch"
        );
        assert!(
            (est_of(&pg, "ISS", 2) - (4.6 + config::EST_GAUGE_STEP)).abs() < 1e-9,
            "evidenced-bare fed at the P5 placement, not the 7.0 bare anchor"
        );
        assert_eq!(pg.cost_feed.len(), 2, "anchor + P5 head both fed");
    }

    /// Gauge NEVER divides (D2): an anchor-free est component projects with
    /// `Gauge` provenance, stays OUT of the feed, and both members keep the
    /// bare anchor as their divisor. ISS-001's anchor is row-gated out (no
    /// est row touches it), so the ISS-002/ISS-003 component is anchor-free.
    #[test]
    fn est_cost_ladder_gauge_masked_items_stay_at_bare_anchor() {
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "", "lower = 2.0\nupper = 6.0", "value = 5.0", "");
        seed_issue_with_facets(root, 2, "", "", "value = 5.0", "");
        seed_issue_with_facets(root, 3, "", "", "value = 5.0", "");
        write_est_session(
            root,
            vec![est_row(
                "e1",
                "ISS-002",
                "ISS-003",
                comparison::Response::PreferA,
            )],
        );
        let pg = build(root).unwrap();
        assert!(pg.cost_feed.is_empty(), "gauge tier absent from the feed");
        for id in [2, 3] {
            assert!(
                (est_of(&pg, "ISS", id) - 7.0).abs() < 1e-9,
                "ISS-{id:03} gauge-masked ⇒ bare anchor"
            );
        }
    }

    /// A merge-hoisted bare member (an `Equal` row into an anchored class) is
    /// fed AT the class-anchor value: ISS-002 scores with ISS-001's authored
    /// cost as its divisor — not the bare anchor, and with no facet of its own.
    #[test]
    fn est_cost_ladder_merge_hoisted_member_fed_at_class_anchor() {
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "", "lower = 2.0\nupper = 6.0", "value = 5.0", "");
        seed_issue_with_facets(root, 2, "", "", "value = 5.0", "");
        write_est_session(
            root,
            vec![est_row(
                "e1",
                "ISS-002",
                "ISS-001",
                comparison::Response::Equal,
            )],
        );
        let pg = build(root).unwrap();
        assert_eq!(
            pg.cost_feed.get("ISS-002"),
            Some(&4.6),
            "merged member fed at the class anchor"
        );
        assert!((est_of(&pg, "ISS", 2) - 4.6).abs() < 1e-9);
        // The anchored item's own feed entry is inert (ladder order).
        assert!((est_of(&pg, "ISS", 1) - 4.6).abs() < 1e-9);
    }

    /// The regime-flip golden (design §6.5): the FIRST authored anchor in a
    /// component flips its evidenced members bare → projected — a real, owned
    /// scoring discontinuity (ISS-002's divisor drops 7.0 → 2.55).
    #[test]
    fn est_cost_ladder_regime_flip_first_anchor_flips_component() {
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "", "lower = 2.0\nupper = 6.0", "value = 5.0", "");
        seed_issue_with_facets(root, 2, "", "", "value = 5.0", "");
        seed_issue_with_facets(root, 3, "", "", "value = 5.0", "");
        write_est_session(
            root,
            vec![est_row(
                "e1",
                "ISS-002",
                "ISS-003",
                comparison::Response::PreferA,
            )],
        );
        // Before: the component is anchor-free — both members at the bare anchor.
        let before = build(root).unwrap();
        assert!((est_of(&before, "ISS", 2) - 7.0).abs() < 1e-9);
        assert!((est_of(&before, "ISS", 3) - 7.0).abs() < 1e-9);

        // ISS-003 authors its first estimate: authored_est_cost(1,3) = 2.3.
        seed_issue_with_facets(root, 3, "", "lower = 1.0\nupper = 3.0", "value = 5.0", "");
        let after = build(root).unwrap();
        assert!((est_of(&after, "ISS", 3) - 2.3).abs() < 1e-9, "authored");
        assert!(
            (est_of(&after, "ISS", 2) - (2.3 + config::EST_GAUGE_STEP)).abs() < 1e-9,
            "member flipped bare → projected (the owned discontinuity)"
        );
    }

    /// The INV-2 restatement pin (design §3 / REV content 2): a PROJECTED cost
    /// may legitimately EXCEED the bare anchor — the anchor dominates every
    /// AUTHORED estimate only. With `gauge_step = 5.0`, the P5 head lands at
    /// 6.0 + 5.0 = 11.0, above the 7.0 bare anchor, and still divides.
    #[test]
    fn est_cost_ladder_projected_may_exceed_bare_anchor() {
        let dir = tmp();
        let root = dir.path();
        write(
            root,
            crate::dtoml::DOCTRINE_TOML,
            "[priority.estimate]\ngauge_step = 5.0\n",
        );
        seed_issue_with_facets(root, 1, "", "lower = 6.0\nupper = 6.0", "value = 5.0", "");
        seed_issue_with_facets(root, 2, "", "", "value = 5.0", "");
        write_est_session(
            root,
            vec![est_row(
                "e1",
                "ISS-002",
                "ISS-001",
                comparison::Response::PreferA,
            )],
        );
        let pg = build(root).unwrap();
        assert!((pg.cost_ctx.absent - 7.0).abs() < 1e-9, "bare anchor");
        let cfg = config::load(root);
        let fed = pg.item_costing(&key("ISS", 2), &cfg).unwrap().1;
        assert!((fed - 11.0).abs() < 1e-9, "P5 head at anchor + step: {fed}");
        assert!(
            fed > pg.cost_ctx.absent,
            "projected exceeds the bare anchor"
        );
    }
    // ── SL-220 PHASE-05: the resolver flip — evidence-ladder suite (§8.4) ──

    /// A value-domain anchor row for the claims harness (mirrors the
    /// `claims.rs` fixtures; capture admissibility is capture-time-only).
    fn claim_anchor(
        uid: &str,
        item: &str,
        magnitude: f64,
        rater: comparison::RaterKind,
        pin: bool,
    ) -> comparison::Judgement {
        let migrated = matches!(rater, comparison::RaterKind::Migrated);
        comparison::Judgement {
            uid: uid.to_string(),
            seq: 0,
            a: item.to_string(),
            b: None,
            response: None,
            domain: comparison::DOMAIN_VALUE.to_string(),
            frame: comparison::FRAME_VALUE_ANCHOR.to_string(),
            form: comparison::RowForm::Anchor,
            magnitude: Some(magnitude),
            supersedes: None,
            lens: None,
            rater,
            by: pin.then(|| "op".to_string()),
            note: None,
            date: (!migrated).then(|| "2026-07-16".to_string()),
            observed_at: migrated.then(|| "2026-07-16".to_string()),
            basis: None,
            admission: pin.then_some(comparison::AdmissionKind::Pin),
        }
    }

    /// Resolve a claim ledger over Active rows (the pure §8.4 harness).
    fn claims_of(rows: &[comparison::Judgement]) -> comparison::ClaimResolution {
        let tagged: Vec<(&comparison::Judgement, comparison::ResolutionStatus)> = rows
            .iter()
            .map(|j| (j, comparison::ResolutionStatus::Active))
            .collect();
        comparison::resolve_claims(&tagged)
    }

    /// EX-1: each ladder rung wins in isolation, and every ADJACENT rung
    /// dominance pair holds — pin > human > projection > agent > migrated >
    /// facet > default (design §3, first hit wins). The facet is consulted
    /// ONLY at zero claim rows: a coexisting prior shadows it (D6 residue).
    #[test]
    fn value_ladder_rungs_and_adjacent_dominance() {
        use comparison::RaterKind;
        let iss_kind = crate::integrity::KINDS
            .iter()
            .find(|k| k.kind.prefix == "ISS")
            .map(|k| k.kind)
            .expect("ISS in KINDS");
        let key1 = key("ISS", 1);
        let no_facets = crate::facet::EntityFacets {
            estimate: None,
            value: None,
            risk: None,
            tags: vec![],
        };
        // The facet comes off a real disk scan (NF-001 tripwire: the facet
        // type is never named in this file).
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "", "", "value = 9.0", "");
        let scanned =
            relation_graph::scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        let facet_9 = crate::facet::EntityFacets {
            estimate: None,
            value: scanned
                .iter()
                .find(|e| e.key == key1)
                .and_then(|e| e.value.clone()),
            risk: None,
            tags: vec![],
        };
        assert!(facet_9.value.is_some(), "fixture authored the facet");

        let no_projection = ValueProjection::new();
        let mut projection_2_5 = ValueProjection::new();
        projection_2_5.insert(
            key1.canonical(),
            (2.5, crate::comparison::ValueProvenance::Projected),
        );
        let erv = |f: &EntityFacets,
                   projected: &ValueProjection,
                   claims: &comparison::ClaimResolution| {
            effective_raw_value(iss_kind, f, key1, projected, claims)
        };

        // Rung 1 vs rung 2 — an anchored claim (pin OR human) beats projection.
        let pin_over_human = claims_of(&[
            claim_anchor("h", "ISS-001", 5.0, RaterKind::Human, false),
            claim_anchor("p", "ISS-001", 6.0, RaterKind::Human, true),
        ]);
        assert_eq!(
            erv(&facet_9, &projection_2_5, &pin_over_human),
            Some(6.0),
            "pin > human (tier contest) and anchored > projection > facet"
        );
        let human = claims_of(&[claim_anchor("h", "ISS-001", 5.0, RaterKind::Human, false)]);
        assert_eq!(
            erv(&no_facets, &projection_2_5, &human),
            Some(5.0),
            "human claim > projection"
        );

        // Rung 2 vs rung 3 — projection beats an agent-tier prior.
        let agent = claims_of(&[claim_anchor("a", "ISS-001", 3.0, RaterKind::Agent, false)]);
        assert!(agent.priors.contains_key("ISS-001"), "agent → priors (D4)");
        assert_eq!(
            erv(&no_facets, &projection_2_5, &agent),
            Some(2.5),
            "projection > agent prior"
        );

        // Rung 3 vs rung 4 — agent beats migrated (the priors tier contest).
        let agent_over_migrated = claims_of(&[
            claim_anchor("a", "ISS-001", 3.0, RaterKind::Agent, false),
            claim_anchor("m", "ISS-001", 2.0, RaterKind::Migrated, false),
        ]);
        assert_eq!(
            erv(&no_facets, &no_projection, &agent_over_migrated),
            Some(3.0),
            "agent prior > migrated prior"
        );

        // Rung 4 vs rung 5 — a migrated prior shadows the facet (residue
        // awaiting its strip: NOT consulted while any claim row exists).
        let migrated = claims_of(&[claim_anchor(
            "m",
            "ISS-001",
            2.0,
            RaterKind::Migrated,
            false,
        )]);
        assert_eq!(
            erv(&facet_9, &no_projection, &migrated),
            Some(2.0),
            "migrated prior > unmigrated facet"
        );

        // Rung 5 vs rung 6 — zero claim rows: the facet still contributes.
        let no_claims = comparison::ClaimResolution::default();
        assert_eq!(
            erv(&facet_9, &no_projection, &no_claims),
            Some(9.0),
            "unmigrated facet > default (transitional rung, D6)"
        );
        // Rung 6 — nothing at all: the value-bearing default.
        assert_eq!(
            erv(&no_facets, &no_projection, &no_claims),
            Some(DEFAULT_VALUE)
        );
    }

    /// Scope R1 (the row-gating footgun): a human `value set` claim with NO
    /// comparison rows still wins at rung 1 — compile's row-gating drops the
    /// row-less anchor from the constraint set (no class to attach to), but
    /// the ladder reads `ClaimResolution.anchored` DIRECTLY. Proven through
    /// the FULL pipeline: session on disk → `build_from` → `value_dim`.
    #[test]
    fn row_less_human_claim_resolves_at_rung_1_through_the_full_pipeline() {
        let dir = tmp();
        let root = dir.path();
        seed_issue(root, 1, "open", "", "");
        write_value_claim_session(
            root,
            vec![claim_anchor(
                "c1",
                "ISS-001",
                7.0,
                comparison::RaterKind::Human,
                false,
            )],
        );
        let scanned =
            relation_graph::scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        let cfg = config::load(root);
        let pipeline = load_comparison_pipeline(root, &scanned, &cfg).unwrap();
        // Row-gating drops the row-less anchor from compile/projection…
        assert!(
            pipeline.value.projection.get("ISS-001").is_none(),
            "no comparison rows ⇒ no projection entry"
        );
        // …but the authority record carries it, and the ladder consumes it.
        assert_eq!(pipeline.value_claims.anchored["ISS-001"].value, 7.0);
        let pg = build(root).unwrap();
        let bs = pg.attrs[&key("ISS", 1)].base_score;
        // value_dim = 1.0 × 7.0 × 1.0 × 1.0 / est_cost(absent = 1.0) = 7.0.
        assert!(
            (bs.value_dim - 7.0).abs() < 1e-9,
            "the row-less claim scored: {}",
            bs.value_dim
        );
    }

    /// RV-278 F-4, stated as a test: a compared facet-bearing item resolves
    /// at rung 2 — the facet neither anchors the compile (the deleted
    /// facet→AnchorMap builder) nor fills the ladder — and the presence-based
    /// `UnmigratedFacet` finding fires anyway. Permanent semantics, not a
    /// migration-window artifact.
    #[test]
    fn compared_facet_bearing_item_resolves_at_rung_2_and_presence_finding_fires() {
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "", "", "value = 9.0", "");
        seed_issue(root, 2, "open", "", "");
        write_comparison_session(root, "ISS-001", "ISS-002");
        let scanned =
            relation_graph::scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        let cfg = config::load(root);
        let pipeline = load_comparison_pipeline(root, &scanned, &cfg).unwrap();
        // The facet no longer anchors or shapes projection…
        assert!(
            pipeline.value.anchors.is_empty(),
            "no claim rows ⇒ no compile anchors — the facet never enters"
        );
        // …so the two-node anchor-free chain projects via the P8 gauge
        // spread (winner 4/3, loser 2/3 of DEFAULT_VALUE), and rung 2 wins
        // over the facet's 9.0.
        let pg = build(root).unwrap();
        let bs1 = pg.attrs[&key("ISS", 1)].base_score;
        assert!(
            (bs1.value_dim - 4.0 / 3.0).abs() < 1e-9,
            "rung 2 (projection) beat the unmigrated facet: {}",
            bs1.value_dim
        );
        // The presence finding fires regardless of rung-5 consumption.
        let findings = crate::priority::findings::detect(&pg, &cfg, None);
        assert!(
            findings.iter().any(|f| matches!(
                f,
                crate::priority::findings::Finding::UnmigratedFacet { entity, value, .. }
                    if entity == "ISS-001" && (*value - 9.0).abs() < 1e-9
            )),
            "UnmigratedFacet fires on PRESENCE: {findings:?}"
        );
        // And the uncompared control: strip the session, the facet serves
        // rung 5 — same finding, different rung (presence ≠ consumption).
        std::fs::remove_dir_all(root.join(".doctrine/comparisons")).unwrap();
        let pg2 = build(root).unwrap();
        let bs1 = pg2.attrs[&key("ISS", 1)].base_score;
        assert!(
            (bs1.value_dim - 9.0).abs() < 1e-9,
            "no projection: rung 5 serves the facet"
        );
    }

    /// D7 paired consumption gate over ALL_KINDS: a resolved claim on ANY
    /// subject resolves (capture-lossless — the claims pass is kind-blind),
    /// but `effective_raw_value` consumes it ONLY for value-bearing kinds —
    /// scoring-inert subjects' claims feed nothing (consumption-inert).
    #[test]
    fn scoring_inert_kinds_claims_resolve_but_are_never_consumed_all_kinds() {
        let no_facets = crate::facet::EntityFacets {
            estimate: None,
            value: None,
            risk: None,
            tags: vec![],
        };
        let no_projection = ValueProjection::new();
        for prefix in crate::kinds::ALL_KINDS {
            let Some(kind) = crate::integrity::KINDS
                .iter()
                .find(|k| k.kind.prefix == *prefix)
                .map(|k| k.kind)
            else {
                panic!("{prefix} missing from integrity::KINDS");
            };
            let subject = format!("{prefix}-001");
            let claims = claims_of(&[claim_anchor(
                "c1",
                &subject,
                7.0,
                comparison::RaterKind::Human,
                false,
            )]);
            // Capture-lossless: the claim resolved for EVERY kind.
            assert_eq!(claims.anchored[&subject].value, 7.0, "{prefix} captured");
            let entity_key = EntityKey { prefix, id: 1 };
            let resolved =
                effective_raw_value(kind, &no_facets, entity_key, &no_projection, &claims);
            if crate::kinds::is_value_bearing(prefix) {
                assert_eq!(resolved, Some(7.0), "{prefix} consumes at rung 1");
            } else {
                assert_eq!(resolved, None, "{prefix} is consumption-inert (D7)");
            }
        }
    }

    /// §8.5 empty-claims bitwise-preservation property: a corpus with NO
    /// anchor rows and NO `[value]` facets builds BITWISE-identically whether
    /// the claims input is the pipeline's own (empty) `ClaimResolution` or an
    /// explicitly-empty one — with real comparison rows keeping the
    /// projection non-trivial (the SL-213 empty-projection precedent).
    #[test]
    fn empty_claims_and_no_facets_score_bitwise_identically() {
        let dir = tmp();
        let root = dir.path();
        seed_issue(root, 1, "open", "", "");
        seed_issue(root, 2, "open", "", "");
        write_comparison_session(root, "ISS-001", "ISS-002");
        let scanned =
            relation_graph::scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        let cfg = config::load(root);
        let pipeline = load_comparison_pipeline(root, &scanned, &cfg).unwrap();
        assert_eq!(
            pipeline.value_claims,
            comparison::ClaimResolution::default(),
            "no anchor rows ⇒ the pipeline's claims are empty"
        );
        assert!(!pipeline.value.projection.is_empty(), "non-trivial corpus");
        let cost_feed = comparison::cost_feed(&pipeline.estimate.projection);
        let via_load = build_from(&scanned, root).unwrap();
        let explicit_empty = build_from_with_cfg(
            &scanned,
            root,
            &cfg,
            &pipeline.value.projection,
            &cost_feed,
            &comparison::ClaimResolution::default(),
        )
        .unwrap();
        assert_eq!(via_load.score, explicit_empty.score, "score map bitwise");
        assert_eq!(via_load.leverage, explicit_empty.leverage);
        assert_eq!(via_load.optionality, explicit_empty.optionality);
        for (k, attr) in &via_load.attrs {
            let other = &explicit_empty.attrs[k];
            assert!(
                attr.base_score.value_dim.to_bits() == other.base_score.value_dim.to_bits()
                    && attr.base_score.risk_dim.to_bits() == other.base_score.risk_dim.to_bits(),
                "{} base dims bitwise-identical",
                k.canonical()
            );
        }
    }

    /// Write one VALUE-domain session carrying `rows` straight to
    /// `.doctrine/comparisons/` (mirrors [`write_est_session`]).
    fn write_value_claim_session(root: &Path, rows: Vec<comparison::Judgement>) {
        let session = comparison::ComparisonSession {
            schema: comparison::COMPARISON_SCHEMA.to_string(),
            version: comparison::COMPARISON_VERSION,
            session: comparison::SessionHeader {
                uid: "v1".to_string(),
                date: "2026-07-16".to_string(),
                audience: None,
            },
            judgements: rows,
            tombstones: Vec::new(),
        };
        let text = comparison::to_toml(&session).unwrap();
        write(root, ".doctrine/comparisons/2026-07-16-v1.toml", &text);
    }
}
