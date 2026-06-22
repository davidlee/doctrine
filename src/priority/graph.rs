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

use crate::facet::EntityFacets;
use crate::priority::config;
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

/// Pure base-score computation per entity (design §5.1). Returns the SPLIT
/// `BaseScore` so `explain` can surface `value_dim` / `risk_dim`. No IO.
fn base_score(f: &EntityFacets, kind: &entity::Kind, cfg: &config::PriorityConfig) -> BaseScore {
    const EPSILON: f64 = 1e-12;
    // value_dim = coefficients.value × value × kind_weight(kind) × Σtag / estimate_midpoint
    let value_dim = {
        let raw = if let Some(ref v) = f.value {
            let est_mid = match f.estimate {
                Some(ref e) => {
                    let m = f64::midpoint(e.lower, e.upper);
                    if m < EPSILON { EPSILON } else { m }
                }
                None => 1.0,
            };
            let kw = cfg.kind_weight(kind.prefix);
            cfg.coefficients.value * v.value * kw / est_mid
        } else {
            0.0
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
}

/// The reference/lineage relation labels that back a consequence-input overlay — the
/// SL-046 overlay-backed labels MINUS the two target-unvalidated ones (`Drift`/
/// `DecisionRef`, which never resolve). One `Reject`/`Unbounded` overlay each — the
/// reference/lineage consequence-input overlays. Label is overlay identity (the same
/// label from different source kinds shares ONE overlay).
const REF_LABELS: &[RelationLabel] = &[
    RelationLabel::Specs,
    RelationLabel::Requirements,
    RelationLabel::Supersedes,
    RelationLabel::DescendsFrom,
    RelationLabel::Parent,
    RelationLabel::Members,
    RelationLabel::Interactions,
    RelationLabel::Slices,
    RelationLabel::Related,
    RelationLabel::Reviews,
    RelationLabel::OwningSlice,
];

/// The WORK/LINEAGE label subset whose inbound references count toward consequence
/// (design §5.2, EX-3). `reviews`/`owning_slice` are bookkeeping and EXCLUDED; the
/// two target-unvalidated labels never resolve and so cannot contribute anyway.
const CONSEQUENCE_LABELS: &[RelationLabel] = &[
    RelationLabel::Specs,
    RelationLabel::Requirements,
    RelationLabel::Slices,
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
/// # Errors
///
/// Propagates a read error, or an internal cordage rejection of well-formed adapter
/// input (an adapter bug, not a recoverable condition).
pub(crate) fn build_from(
    scanned: &[relation_graph::ScannedEntity],
    root: &std::path::Path,
) -> anyhow::Result<PriorityGraph> {
    // 1b. Load PriorityConfig once — covers every caller including
    //      actionability_block_from (D4).
    let cfg = config::load(root);

    // 2b. Base pre-pass — compute `base_score` per node from its OWN facets + config +
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
                },
                entity.kind,
                &cfg,
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
        &cfg,
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
    })
}

/// Consequence post-pass (design §5.4 step 6). Pure over the built graph.
/// Returns (leverage, optionality, score) keyed by `EntityKey`.
fn consequence_post_pass(
    graph: &Graph,
    projection: &Projection<EntityKey>,
    attrs: &BTreeMap<EntityKey, NodeAttr>,
    ref_by_label: &BTreeMap<RelationLabel, OverlayId>,
    dep_overlay: OverlayId,
    cfg: &config::PriorityConfig,
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

    // ── assemble into EntityKey-keyed maps ──
    let mut leverage: BTreeMap<EntityKey, f64> = BTreeMap::new();
    let mut optionality: BTreeMap<EntityKey, f64> = BTreeMap::new();
    let mut score: BTreeMap<EntityKey, f64> = BTreeMap::new();
    for nid in graph.ordered() {
        if let Some(k) = ek(nid) {
            let lev = leverage_by_node.get(&nid).copied().unwrap_or(0.0);
            let opt = optionality_by_node.get(&nid).copied().unwrap_or(0.0);
            let bs = base_of(nid);
            let sc = bs + lev + opt;
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
                    .and_then(|l| crate::relation::lookup(source, l))
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
        seed_slice(root, 1, "requirements = [\"REQ-005\"]\n");
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
        // Three issues with DIFFERENT base scores (value facet over a fixed estimate
        // mid of 5.0): ISS-001 value 5 → base 1.0; ISS-002 value 25 → base 5.0;
        // ISS-003 value 15 → base 3.0. Mint order is base.total() DESC, ties by id ASC:
        // ISS-002 (5.0) < ISS-003 (3.0) < ISS-001 (1.0) by NodeId. Crucially the
        // consequence/edge topology does NOT enter mint (I3) — none of these author a
        // work/lineage edge onto another, so only base ranks them.
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
            "ISS-002 (base 5.0) mints before ISS-003 (base 3.0)"
        );
        assert!(
            n3 < n1,
            "ISS-003 (base 3.0) mints before ISS-001 (base 1.0)"
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
            "ISS-002 (base 5.0) mints before the heavily-referenced ISS-001 (base 0) — mint is base-only (I3)"
        );
        // The post-pass still credits ISS-001's optionality (two slices reference it,
        // both base 0 here → optionality 0), proving score is a separate display tier.
        assert_eq!(pg.score.get(&key("ISS", 1)).copied().unwrap_or(0.0), 0.0);
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
        // These slices author NO needs/after — the cross-kind `dep_seq_for` reads their
        // empty axes; no panic, no dep/seq edge. (SL-060: slices CAN author dep/seq now,
        // but an entity that authors none contributes none.) A FACETED issue references
        // REQ-005 via the `requirements` consequence label, so the resolved ref edge is
        // observable through REQ-005's post-pass OPTIONALITY (ref_coeff · base(referrer)).
        // A faceted ISS-001 (base = value 25 / mid 5 = 5.0) references ISS-002 via the
        // `slices` consequence label; ISS-002's one-hop optionality is
        // ref_coeff(1.0) · base(ISS-001) = 5.0 when the ref edge resolved. SL-001/SL-002
        // author NO needs/after — their dep/seq axes are empty (no panic, no edge).
        seed_issue_with_facets(
            root,
            1,
            "slices = [\"ISS-002\"]\n",
            "lower = 0.0\nupper = 10.0",
            "value = 25.0",
            "",
        );
        seed_issue(root, 2, "open", "", "");
        seed_slice(root, 1, "requirements = [\"REQ-005\"]\n");
        seed_requirement(root, 5);
        seed_slice(root, 2, "");
        let pg = build(root).unwrap();
        let sl1 = pg.projection.resolve(key("SL", 1)).unwrap();
        let sl2 = pg.projection.resolve(key("SL", 2)).unwrap();
        assert_eq!(pg.graph.in_edges(pg.dep_overlay, sl1).count(), 0);
        assert_eq!(pg.graph.in_edges(pg.seq_overlay, sl1).count(), 0);
        assert_eq!(pg.graph.in_edges(pg.dep_overlay, sl2).count(), 0);
        // The resolvable `slices` ref edge landed: ISS-002's optionality reflects it.
        assert!(
            (pg.optionality.get(&key("ISS", 2)).copied().unwrap_or(0.0) - 5.0).abs() < 1e-9,
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
        assert_eq!(
            pg.score.get(&key("ISS", 1)).copied().unwrap_or(0.0),
            0.0,
            "free-text drift target produces no edge → score floor 0"
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
        // value_dim = 1.0(value coeff) * 10.0 * 1.0(kind_weight, default) * 1.0(Σtag) / 5.0(mid)
        //          = 2.0
        // risk_dim  = 2.0(risk coeff) * 12(exposure: high=3 × critical=4)
        //          = 24.0
        assert!((bs.value_dim - 2.0).abs() < 1e-9, "value_dim should be 2.0");
        assert!((bs.risk_dim - 24.0).abs() < 1e-9, "risk_dim should be 24.0");
        assert!((bs.total() - 26.0).abs() < 1e-9, "total should be 26.0");
    }

    #[test]
    fn base_score_value_only_risk_absent() {
        let dir = tmp();
        let root = dir.path();
        seed_issue_with_facets(root, 1, "", "lower = 0.0\nupper = 2.0", "value = 5.0", "");
        let pg = build(root).unwrap();
        let bs = pg.attrs[&key("ISS", 1)].base_score;
        // value_dim = 1.0 * 5.0 / ((0+2)/2 → EPSILON because < EPSILON) = 5.0 / 1e-12 ≈ 5e12
        // Actually: mid = (0+2)/2 = 1.0. So value_dim = 5.0 / 1.0 = 5.0
        assert!((bs.value_dim - 5.0).abs() < 1e-9, "value_dim should be 5.0");
        assert!((bs.risk_dim - 0.0).abs() < 1e-9, "risk_dim should be 0");
        assert!((bs.total() - 5.0).abs() < 1e-9, "total should be 5.0");
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
        assert!((bs.value_dim - 0.0).abs() < 1e-9, "value_dim should be 0");
        // risk_dim = 2.0 * 2 (low=1 × medium=2) = 4.0
        assert!((bs.risk_dim - 4.0).abs() < 1e-9, "risk_dim should be 4.0");
        assert!((bs.total() - 4.0).abs() < 1e-9, "total should be 4.0");
    }

    #[test]
    fn base_score_neither_facet_present() {
        let dir = tmp();
        let root = dir.path();
        seed_issue(root, 1, "open", "", "");
        let pg = build(root).unwrap();
        let bs = pg.attrs[&key("ISS", 1)].base_score;
        assert!((bs.value_dim - 0.0).abs() < 1e-9, "value_dim should be 0");
        assert!((bs.risk_dim - 0.0).abs() < 1e-9, "risk_dim should be 0");
        assert!((bs.total() - 0.0).abs() < 1e-9, "total should be 0");
    }

    #[test]
    fn base_score_absent_estimate_uses_midpoint_one() {
        let dir = tmp();
        let root = dir.path();
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
        // value_dim = 1.0 * 3.0 / 1.0 = 3.0
        assert!((bs.value_dim - 3.0).abs() < 1e-9, "value_dim should be 3.0");
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
        // ISS-002 base = 3.0, ISS-001 base = 10.0
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
        // A has slices→B, B has slices→C. C's optionality should only see B, not A.
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, "slices = [\"ISS-001\"]\n");
        seed_issue_with_facets(root, 1, "slices = [\"ISS-002\"]\n", "", "value = 5.0", "");
        seed_issue_with_facets(root, 2, "", "", "value = 3.0", "");
        let pg = build(root).unwrap();
        // ISS-002 is the downstream target of ISS-001's slices edge.
        // ISS-002's optionality should be ref_coeff * base(ISS-001) = 5.0
        let opt_iss2 = pg.optionality[&key("ISS", 2)];
        assert!(
            (opt_iss2 - 5.0).abs() < 1e-9,
            "ISS-002 gets optionality from ISS-001"
        );
        // ISS-001's optionality is 0 (only SL-001 references it, but SL-001 has value=0)
        let opt_iss1 = pg.optionality[&key("ISS", 1)];
        assert!(
            (opt_iss1 - 0.0).abs() < 1e-9,
            "ISS-001 has no valued referencers"
        );
        // SL-001 has zero optionality
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
            "doctrine.toml",
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
}
