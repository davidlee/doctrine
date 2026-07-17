// SPDX-License-Identifier: GPL-3.0-only
//! `comparison::compile` — tier-2 constraint compilation (SL-213 design §2,
//! rules C1–C8). Pure leaf (ADR-001): depends only on the `wire` model plus
//! std `BTree` collections. No clock, disk, rng, or git.
//!
//! Input: the tier-1 active judgement rows (`&[&Judgement]`, borrowed — the
//! shape `resolve.rs` hands over) plus the anchor map (authored `value`
//! facets). The boundary projects each row into its typed [`PairRow`] view
//! (SL-220 design §2 — the filter seam), so the rule code below is total
//! field access and anchor rows can never compile. Output: a
//! [`ConstraintSet`] (design D12, in-memory only, never serialized):
//! equality-merged classes, the strict class digraph, per-class anchors and
//! display bounds, and the quarantine ledger. Pipeline order is pinned
//! C2 → C3 → C4.
//!
//! Determinism idiom: `BTree` collections everywhere, uid tiebreaks, no float
//! in any key (`f64` compared via `total_cmp` only). A [`ClassId`] is the
//! smallest member entity id in `BTree` order — deterministic and stable
//! across input order.

use std::collections::{BTreeMap, BTreeSet};

use super::{Judgement, RaterKind, Response};

/// A judgement row uid.
pub(crate) type RowUid = String;

/// An equality class id: the smallest member entity id in `BTree` order.
pub(crate) type ClassId = String;

/// Entity id → its authored `value` facet (the only magnitude source, D8).
/// Defined here; populated by a later SL-213 phase.
pub(crate) type AnchorMap = BTreeMap<String, f64>;

/// The strict class digraph: `(winner, loser)` → supporting row uids.
pub(crate) type EdgeMap = BTreeMap<(ClassId, ClassId), BTreeSet<RowUid>>;

/// The typed pairwise projection (SL-220 design §2): a borrowed view over an
/// order/ratio row whose pairwise payload (`b`, `response`) is present BY
/// TYPE. Constructed once at the compile boundary — the filter seam — so
/// every rule below (C1–C8) is total field access with no `Option` handling
/// inside the constraint layer; anchor rows (single subject, no pairwise
/// payload — SL-220 §1) can never enter, whatever a caller passes (RV-278
/// F-6, typed).
#[derive(Debug, Clone, Copy)]
pub(crate) struct PairRow<'a> {
    pub uid: &'a str,
    pub a: &'a str,
    pub b: &'a str,
    pub response: Response,
}

impl<'a> PairRow<'a> {
    /// The pairwise view of one row — `None` when the pairwise payload is
    /// absent (an anchor row): the row constrains nothing here.
    pub(crate) fn of(j: &'a Judgement) -> Option<Self> {
        match (j.b.as_deref(), j.response) {
            (Some(b), Some(response)) => Some(PairRow {
                uid: &j.uid,
                a: &j.a,
                b,
                response,
            }),
            _ => None,
        }
    }
}

/// One side of a class's value interval (C6). `Closed` arises only via an
/// anchor on the class itself; reachable anchors give `Open`; no anchor in
/// that direction gives `Unbounded`. Display projection only — never a
/// decision input (the joint [`ConstraintSet`] is the truth).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Bound {
    Unbounded,
    Open(f64),
    Closed(f64),
}

/// The C6 display interval for one class.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ValueBounds {
    pub lower: Bound,
    pub upper: Bound,
}

/// Why a row was quarantined (C2/C3/C4). Payloads carry enough for the
/// PHASE-06 findings: a row serving several conflicts lists EVERY pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum QuarantineReason {
    /// C3: the row's strict edge lies inside a preference cycle. `classes` =
    /// the members of that strongly-connected component, sorted.
    PreferenceCycle { classes: Vec<ClassId> },
    /// C2/C4: the row lies on a path contradicting a pair of anchors. Each
    /// pair `(x, y)` is ordered `x ⇝ y` with `anchor(x) ≤ anchor(y)`. C2
    /// pairs name the differently-anchored member entity ids (entity-id
    /// order); C4 pairs name class ids (themselves entity ids). Anchor
    /// values live in [`ConstraintSet::anchors`] / the input [`AnchorMap`].
    AnchorConflict { pairs: Vec<(String, String)> },
}

/// The tier-2 verdict for one active row (design §1: assigned by compile.rs;
/// composed with `ResolutionStatus` into a `RowState`, `resolve.rs`). PHASE-06
/// is its real, non-test consumer (`list`'s status column / `explain`'s
/// value-source block) — the PHASE-01..05 self-clearing `dead_code`
/// suppression on this type + [`ConstraintSet::status_of`] retires here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CompilationStatus {
    Constraining,
    /// `incomparable`: valid evidence, zero constraint — excluded from
    /// constraining-judgement counts (design S3).
    NoConstraint,
    Quarantined(QuarantineReason),
}

/// The C8 quarantine-policy seam — a pure input. Only the symmetric variant
/// ships (quarantine every row on a violating structure, no rank-aware
/// demotion); T7 rank-aware slots in here later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QuarantinePolicy {
    Symmetric,
}

/// The compiled constraint system (design D12). In-memory only — never
/// serialized. Post-compile the retained system is satisfiable (C5).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ConstraintSet {
    /// Entity id → its equality class. Every entity named by any active row
    /// gets a class (incomparable rows included — their entities are
    /// evidence-bearing; the rows just constrain nothing). Anchor-map-only
    /// entities get none: no row evidence, nothing to place.
    pub classes: BTreeMap<String, ClassId>,
    /// Retained strict edges (winner > loser) with supporting row uids.
    pub edges: EdgeMap,
    /// Point anchors, hoisted from anchored members onto their class.
    pub anchors: BTreeMap<ClassId, f64>,
    /// Row uid → why it was quarantined.
    pub quarantined: BTreeMap<RowUid, QuarantineReason>,
    /// C6 display bounds, one per class.
    pub bounds: BTreeMap<ClassId, ValueBounds>,
}

impl ConstraintSet {
    /// The compilation status of one active row. PHASE-06 consumer (see
    /// [`CompilationStatus`]'s doc).
    pub(crate) fn status_of(&self, judgement: &Judgement) -> CompilationStatus {
        if let Some(reason) = self.quarantined.get(&judgement.uid) {
            return CompilationStatus::Quarantined(reason.clone());
        }
        if matches!(judgement.response, Some(Response::Incomparable)) {
            return CompilationStatus::NoConstraint;
        }
        CompilationStatus::Constraining
    }
}

/// Per-rater constraining-judgement counts (design §4 S3 — the T7 rater-split
/// disclosure in `explain`'s "projected"/"gauge" value-source shapes).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct RaterCounts {
    pub human: u32,
    pub agent: u32,
}

impl RaterCounts {
    fn bump(&mut self, rater: &RaterKind) {
        match rater {
            RaterKind::Human => self.human += 1,
            RaterKind::Agent => self.agent += 1,
            // Unreachable from the pipeline: migrated rows are anchor-form by
            // the SL-220 §1 validation matrix and terminate at the claims
            // pass (design §2, RV-278 F-6) — never compile input. Kept total
            // rather than panicking on a malformed caller.
            RaterKind::Migrated => {}
        }
    }

    /// The total judgement count, human + agent (`explain`'s gauge shape:
    /// "ordered by N judgements").
    pub(crate) fn total(self) -> u32 {
        self.human + self.agent
    }
}

/// Per-class constraining-judgement rater split (design §4 S3) — a PURE
/// derived accessor over the compiled [`ConstraintSet`] plus the ACTIVE rows
/// it was compiled from, not a new stored field (D12: no `ConstraintSet`
/// reshape; PHASE-06's STOP-condition seam). Every active row whose
/// [`ConstraintSet::status_of`] verdict is `Constraining` is attributed to
/// every class either side belongs to: a strict edge evidences BOTH its
/// winner's and its loser's class; an `equal` merge evidences the one class
/// it created (both sides land in the same class post-merge, so the loop
/// naturally counts it once via the `BTreeSet` dedup). `NoConstraint`/
/// quarantined rows never contribute (S3: `incomparable` excluded).
pub(crate) fn constraining_counts_by_class(
    cs: &ConstraintSet,
    active: &[&Judgement],
) -> BTreeMap<ClassId, RaterCounts> {
    let mut out: BTreeMap<ClassId, RaterCounts> = BTreeMap::new();
    for &j in active {
        if !matches!(cs.status_of(j), CompilationStatus::Constraining) {
            continue;
        }
        // The typed pairwise seam (SL-220 design §2): only a row with its
        // pairwise payload evidences classes — an anchor row contributes
        // nothing, by type.
        let Some(pair) = PairRow::of(j) else {
            continue;
        };
        let classes: BTreeSet<&ClassId> = [cs.classes.get(pair.a), cs.classes.get(pair.b)]
            .into_iter()
            .flatten()
            .collect();
        for class in classes {
            out.entry(class.clone()).or_default().bump(&j.rater);
        }
    }
    out
}

/// The `rater == human` row subset (SL-218 D1) — the evidence the verdict
/// system compiles from when agent demotion is on.
pub(crate) fn human_rows<'a>(active: &[&'a Judgement]) -> Vec<&'a Judgement> {
    active
        .iter()
        .copied()
        .filter(|j| matches!(j.rater, RaterKind::Human))
        .collect()
}

/// Compile the human-rows-only subset into its own [`ConstraintSet`]
/// (SL-218 D1: excluded-from-determinacy). The subset runs the full C1–C8
/// pipeline — its own quarantine verdicts, honest per-system: a human row
/// quarantined in the full system (e.g. cycling with agent rows) may
/// legitimately survive here. Anchors are authored, so they are always
/// present in both systems.
pub(crate) fn compile_human_only(
    active: &[&Judgement],
    anchors: &AnchorMap,
    policy: QuarantinePolicy,
) -> ConstraintSet {
    compile(&human_rows(active), anchors, policy)
}

/// Compile the active set into a [`ConstraintSet`] (design §2, C1–C8).
/// Deterministic across input order; pure over its inputs.
pub(crate) fn compile(
    active: &[&Judgement],
    anchors: &AnchorMap,
    policy: QuarantinePolicy,
) -> ConstraintSet {
    // C8: the policy is a pure input; symmetric is the only shipped variant.
    let QuarantinePolicy::Symmetric = policy;

    // The filter seam (SL-220 design §2): project the typed [`PairRow`]
    // views ONCE at the boundary — everything below is total field access.
    // A row without its pairwise payload (an anchor row) constrains nothing.
    let pairs: Vec<PairRow<'_>> = active.iter().filter_map(|j| PairRow::of(j)).collect();

    // C1 vocabulary. `equal` merges; `prefer-a/b` orients a strict edge
    // winner > loser; `incomparable` constrains nothing (NoConstraint);
    // `magnitude` stays uncompiled — anchors are the only magnitude source.
    let equal_rows: Vec<&PairRow<'_>> = pairs
        .iter()
        .filter(|p| matches!(p.response, Response::Equal))
        .collect();
    let strict_rows: Vec<(&PairRow<'_>, &str, &str)> = pairs
        .iter()
        .filter_map(|p| match p.response {
            Response::PreferA => Some((p, p.a, p.b)),
            Response::PreferB => Some((p, p.b, p.a)),
            _ => None,
        })
        .collect();

    // C2 equal-vs-anchors, before class finalization.
    let mut quarantined = c2_quarantine(&equal_rows, anchors);
    let surviving_equals: Vec<&PairRow<'_>> = equal_rows
        .iter()
        .copied()
        .filter(|p| !quarantined.contains_key(p.uid))
        .collect();

    // Final classes, rebuilt exactly once from the surviving equal rows.
    let entities: BTreeSet<&str> = pairs.iter().flat_map(|p| [p.a, p.b]).collect();
    let classes = build_classes(&entities, &surviving_equals);
    let class_anchors = class_anchor_values(&classes, anchors);

    // Strict class digraph (C1).
    let mut edges: EdgeMap = BTreeMap::new();
    for (p, winner, loser) in &strict_rows {
        let (Some(w), Some(l)) = (classes.get(*winner), classes.get(*loser)) else {
            continue; // unreachable: `entities` covers every row endpoint
        };
        edges
            .entry((w.clone(), l.clone()))
            .or_default()
            .insert(p.uid.to_string());
    }

    // Pipeline order pinned (design §2): C2 → C3 → C4.
    c3_quarantine(&mut edges, &mut quarantined);
    c4_quarantine(&mut edges, &class_anchors, &mut quarantined);

    // C6 display bounds.
    let all_classes: BTreeSet<ClassId> = classes.values().cloned().collect();
    let bounds = compute_bounds(&all_classes, &edges, &class_anchors);

    let set = ConstraintSet {
        classes,
        edges,
        anchors: class_anchors,
        quarantined,
        bounds,
    };
    // C5: a compile that returns an infeasible set is a bug.
    debug_assert!(
        is_feasible(&set.edges, &set.anchors),
        "C5: compile produced an infeasible ConstraintSet"
    );
    set
}

// ---- C2: equal rows vs anchors -------------------------------------------

/// C2 (runs before class finalization): build provisional equality classes;
/// in any class holding ≥ 2 distinct anchor values, quarantine every `equal`
/// row on any path IN THE EQUALITY GRAPH between differently-anchored
/// members (violation-closure, same philosophy as C4). One rebuild suffices:
/// removing the connecting rows disconnects every conflicting anchor pair,
/// and splitting classes can only reduce merging.
fn c2_quarantine(
    equal_rows: &[&PairRow<'_>],
    anchors: &AnchorMap,
) -> BTreeMap<RowUid, QuarantineReason> {
    let entities: BTreeSet<&str> = equal_rows.iter().flat_map(|p| [p.a, p.b]).collect();
    let mut dsu = DisjointSet::new(entities.iter().copied());
    let mut adj: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut rows_by_edge: BTreeMap<(String, String), BTreeSet<RowUid>> = BTreeMap::new();
    for p in equal_rows {
        dsu.union(p.a, p.b);
        adj.entry(p.a).or_default().insert(p.b);
        adj.entry(p.b).or_default().insert(p.a);
        rows_by_edge
            .entry(vertex_pair(p.a, p.b))
            .or_default()
            .insert(p.uid.to_string());
    }

    let mut members: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for &e in &entities {
        members.entry(dsu.find(e)).or_default().push(e);
    }

    let mut pairs_by_row: BTreeMap<RowUid, BTreeSet<(String, String)>> = BTreeMap::new();
    for group in members.values() {
        let anchored: Vec<(&str, f64)> = group
            .iter()
            .filter_map(|&e| anchors.get(e).map(|&v| (e, v)))
            .collect();
        for (i, &(x, value_x)) in anchored.iter().enumerate() {
            for &(y, value_y) in anchored.iter().skip(i + 1) {
                if value_x.total_cmp(&value_y).is_eq() {
                    continue; // same anchor value: merging them is consistent
                }
                for edge in edges_on_simple_paths(&adj, x, y) {
                    let Some(uids) = rows_by_edge.get(&edge) else {
                        continue;
                    };
                    for uid in uids {
                        pairs_by_row
                            .entry(uid.clone())
                            .or_default()
                            .insert((x.to_string(), y.to_string()));
                    }
                }
            }
        }
    }

    pairs_by_row
        .into_iter()
        .map(|(uid, pairs)| {
            (
                uid,
                QuarantineReason::AnchorConflict {
                    pairs: pairs.into_iter().collect(),
                },
            )
        })
        .collect()
}

/// Every equality-graph edge (as an unordered vertex pair) lying on ANY
/// simple path between `from` and `to` — the C2 violation-closure form.
/// Exhaustive DFS over simple paths: exponential in pathological classes,
/// fine at ledger scale, and exactly the normative statement.
fn edges_on_simple_paths<'a>(
    adj: &BTreeMap<&'a str, BTreeSet<&'a str>>,
    from: &'a str,
    to: &str,
) -> BTreeSet<(String, String)> {
    let mut out = BTreeSet::new();
    let mut on_path: Vec<&str> = vec![from];
    dfs_simple_paths(adj, from, to, &mut on_path, &mut out);
    out
}

fn dfs_simple_paths<'a>(
    adj: &BTreeMap<&'a str, BTreeSet<&'a str>>,
    cur: &'a str,
    to: &str,
    on_path: &mut Vec<&'a str>,
    out: &mut BTreeSet<(String, String)>,
) {
    if cur == to {
        for step in on_path.windows(2) {
            if let [a, b] = step {
                out.insert(vertex_pair(a, b));
            }
        }
        return;
    }
    let Some(next) = adj.get(cur) else {
        return;
    };
    for &n in next {
        if on_path.contains(&n) {
            continue;
        }
        on_path.push(n);
        dfs_simple_paths(adj, n, to, on_path, out);
        on_path.pop();
    }
}

fn vertex_pair(a: &str, b: &str) -> (String, String) {
    if a <= b {
        (a.to_string(), b.to_string())
    } else {
        (b.to_string(), a.to_string())
    }
}

// ---- classes ---------------------------------------------------------------

/// Union–find over entity ids with a deterministic "smaller root wins" rule:
/// every union keeps the lexicographically smaller root, so the final root of
/// a set IS its smallest member — the [`ClassId`] scheme for free.
#[derive(Debug)]
struct DisjointSet {
    parent: BTreeMap<String, String>,
}

impl DisjointSet {
    fn new<'a>(entities: impl Iterator<Item = &'a str>) -> Self {
        Self {
            parent: entities.map(|e| (e.to_string(), e.to_string())).collect(),
        }
    }

    fn find(&self, entity: &str) -> String {
        let mut cur = entity;
        while let Some(parent) = self.parent.get(cur) {
            if parent.as_str() == cur {
                break;
            }
            cur = parent;
        }
        cur.to_string()
    }

    fn union(&mut self, a: &str, b: &str) {
        let root_a = self.find(a);
        let root_b = self.find(b);
        if root_a == root_b {
            return;
        }
        let (small, large) = if root_a < root_b {
            (root_a, root_b)
        } else {
            (root_b, root_a)
        };
        self.parent.insert(large, small);
    }
}

fn build_classes(
    entities: &BTreeSet<&str>,
    surviving_equals: &[&PairRow<'_>],
) -> BTreeMap<String, ClassId> {
    let mut dsu = DisjointSet::new(entities.iter().copied());
    for p in surviving_equals {
        dsu.union(p.a, p.b);
    }
    entities
        .iter()
        .map(|&e| (e.to_string(), dsu.find(e)))
        .collect()
}

/// Hoist entity anchors onto their class. Post-C2 no class holds two
/// distinct anchor values; the smallest anchored member's value wins the
/// (bitwise-equal) tie via `or_insert`.
fn class_anchor_values(
    classes: &BTreeMap<String, ClassId>,
    anchors: &AnchorMap,
) -> BTreeMap<ClassId, f64> {
    let mut out: BTreeMap<ClassId, f64> = BTreeMap::new();
    for (entity, &value) in anchors {
        let Some(class) = classes.get(entity) else {
            continue; // anchor on an entity with no row evidence: no class
        };
        let slot = out.entry(class.clone()).or_insert(value);
        debug_assert!(
            slot.total_cmp(&value).is_eq(),
            "post-C2 invariant broken: class `{class}` holds two distinct anchor values"
        );
    }
    out
}

// ---- C3: preference-cycle quarantine ---------------------------------------

/// C3 (D3): SCCs over the strict class digraph; every within-SCC edge's
/// supporting rows are quarantined (`PreferenceCycle`), self-loops included
/// (a strict row whose endpoints merged into one class). External
/// member-level edges are untouched; no member equality, no condensation
/// edges.
fn c3_quarantine(edges: &mut EdgeMap, quarantined: &mut BTreeMap<RowUid, QuarantineReason>) {
    let nodes: BTreeSet<ClassId> = edges
        .keys()
        .flat_map(|(u, v)| [u.clone(), v.clone()])
        .collect();
    let adj = adjacency(edges);
    let component = kosaraju(&nodes, &adj);

    let mut scc_members: BTreeMap<usize, Vec<ClassId>> = BTreeMap::new();
    for (node, &id) in &component {
        scc_members.entry(id).or_default().push(node.clone());
    }

    let cyclic: Vec<(ClassId, ClassId)> = edges
        .keys()
        .filter(|(u, v)| component.get(u) == component.get(v))
        .cloned()
        .collect();
    for key in cyclic {
        let Some(rows) = edges.remove(&key) else {
            continue;
        };
        let Some(id) = component.get(&key.0) else {
            continue; // unreachable: every edge endpoint is a node
        };
        let classes = scc_members.get(id).cloned().unwrap_or_default();
        for uid in rows {
            quarantined.insert(
                uid,
                QuarantineReason::PreferenceCycle {
                    classes: classes.clone(),
                },
            );
        }
    }
}

/// Kosaraju SCC assignment over the strict class digraph. Deliberate
/// duplication (design §1): cordage carries Tarjan machinery, but
/// `scc_partition` (crates/cordage/src/query.rs) and `pass2_cycles`
/// (crates/cordage/src/resolve.rs) are private and `OverlayId`/`NodeId(u32)`
/// shaped, and `CycleDiagnostic` has private fields with no public
/// constructor — widening cordage's public API for this non-entity string
/// graph is not sanctioned, so a small local SCC pass lives here instead.
fn kosaraju(
    nodes: &BTreeSet<ClassId>,
    adj: &BTreeMap<ClassId, BTreeSet<ClassId>>,
) -> BTreeMap<ClassId, usize> {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut finish_order: Vec<&str> = Vec::new();
    for n in nodes {
        dfs_finish_order(n, adj, &mut seen, &mut finish_order);
    }
    let reversed = transpose(adj);
    let mut component: BTreeMap<ClassId, usize> = BTreeMap::new();
    let mut next_id = 0_usize;
    for &n in finish_order.iter().rev() {
        if !component.contains_key(n) {
            dfs_assign(n, &reversed, &mut component, next_id);
            next_id += 1;
        }
    }
    component
}

fn dfs_finish_order<'a>(
    v: &'a str,
    adj: &'a BTreeMap<ClassId, BTreeSet<ClassId>>,
    seen: &mut BTreeSet<&'a str>,
    finish_order: &mut Vec<&'a str>,
) {
    if !seen.insert(v) {
        return;
    }
    if let Some(next) = adj.get(v) {
        for w in next {
            dfs_finish_order(w, adj, seen, finish_order);
        }
    }
    finish_order.push(v);
}

fn dfs_assign(
    v: &str,
    adj: &BTreeMap<ClassId, BTreeSet<ClassId>>,
    component: &mut BTreeMap<ClassId, usize>,
    id: usize,
) {
    if component.contains_key(v) {
        return;
    }
    component.insert(v.to_string(), id);
    if let Some(next) = adj.get(v) {
        for w in next {
            dfs_assign(w, adj, component, id);
        }
    }
}

// ---- C4: anchor-conflict quarantine ----------------------------------------

/// Edge → every ordered anchor pair it serves.
type ViolationMap = BTreeMap<(ClassId, ClassId), BTreeSet<(ClassId, ClassId)>>;

/// C4, normative path form (D4): for every ordered anchor pair `(x, y)` with
/// a directed path `x ⇝ y` and `anchor(x) ≤ anchor(y)`, every retained edge
/// `(u, v)` with `u ∈ forward-reach(x)∪{x}` and `v ∈ reverse-reach(y)∪{y}`
/// lies on a directed path from `x` to `y`. Exact post-C3: the retained
/// graph is a DAG, so the walk `x ⇝ u → v ⇝ y` is a simple path. Returns
/// edge → every pair it serves (a row quarantined once lists every pair).
fn anchor_violations_path_form(
    edges: &EdgeMap,
    class_anchors: &BTreeMap<ClassId, f64>,
) -> ViolationMap {
    let adj = adjacency(edges);
    let reversed = transpose(&adj);
    let mut out: ViolationMap = BTreeMap::new();
    for (x, anchor_x) in class_anchors {
        let forward = reach_including(&adj, x);
        for (y, anchor_y) in class_anchors {
            if x == y || anchor_x.total_cmp(anchor_y).is_gt() || !forward.contains(y) {
                continue;
            }
            let reverse = reach_including(&reversed, y);
            for (u, v) in edges.keys() {
                if forward.contains(u) && reverse.contains(v) {
                    out.entry((u.clone(), v.clone()))
                        .or_default()
                        .insert((x.clone(), y.clone()));
                }
            }
        }
    }
    out
}

/// C4, forced-floor / forced-ceiling two-pass — the crossing-set
/// implementation. `ceiling(u)` = min anchor over ancestors∪self (`u` is
/// forced strictly below every upstream anchor); `floor(v)` = max anchor
/// over descendants∪self (`v` is forced strictly above every downstream
/// anchor). An edge `(u, v)` lies on a violating path ⇔ the intervals cross:
/// `ceiling(u) ≤ floor(v)`. Pure order comparison, no gap arithmetic. Must
/// agree with the path form — debug-asserted on every compile and pinned by
/// a golden test.
fn anchor_violations_two_pass(
    edges: &EdgeMap,
    class_anchors: &BTreeMap<ClassId, f64>,
) -> BTreeSet<(ClassId, ClassId)> {
    let adj = adjacency(edges);
    let reversed = transpose(&adj);
    let nodes: BTreeSet<ClassId> = edges
        .keys()
        .flat_map(|(u, v)| [u.clone(), v.clone()])
        .collect();
    let mut ceiling: BTreeMap<ClassId, f64> = BTreeMap::new();
    let mut floor: BTreeMap<ClassId, f64> = BTreeMap::new();
    for n in &nodes {
        let upstream = reach_including(&reversed, n)
            .into_iter()
            .filter_map(|c| class_anchors.get(&c).copied());
        if let Some(m) = total_min(upstream) {
            ceiling.insert(n.clone(), m);
        }
        let downstream = reach_including(&adj, n)
            .into_iter()
            .filter_map(|c| class_anchors.get(&c).copied());
        if let Some(m) = total_max(downstream) {
            floor.insert(n.clone(), m);
        }
    }
    edges
        .keys()
        .filter(|(u, v)| {
            matches!(
                (ceiling.get(u), floor.get(v)),
                (Some(c), Some(f)) if c.total_cmp(f).is_le()
            )
        })
        .cloned()
        .collect()
}

/// Apply C4 over the post-C3 (DAG) edges. Single pass suffices: quarantining
/// edges only removes paths — it can never create a new anchor-pair
/// violation (invariant, debug-asserted below and property-tested).
fn c4_quarantine(
    edges: &mut EdgeMap,
    class_anchors: &BTreeMap<ClassId, f64>,
    quarantined: &mut BTreeMap<RowUid, QuarantineReason>,
) {
    let by_pair = anchor_violations_path_form(edges, class_anchors);
    let crossing = anchor_violations_two_pass(edges, class_anchors);
    debug_assert_eq!(
        by_pair.keys().cloned().collect::<BTreeSet<_>>(),
        crossing,
        "C4: normative path form and floor/ceiling two-pass disagree"
    );
    for key in crossing {
        let Some(rows) = edges.remove(&key) else {
            continue;
        };
        let pairs: Vec<(String, String)> = by_pair
            .get(&key)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default();
        for uid in rows {
            quarantined.insert(
                uid,
                QuarantineReason::AnchorConflict {
                    pairs: pairs.clone(),
                },
            );
        }
    }
    debug_assert!(
        anchor_violations_two_pass(edges, class_anchors).is_empty(),
        "C4: a violation survived the single quarantine pass"
    );
}

// ---- graph helpers -----------------------------------------------------------

fn adjacency(edges: &EdgeMap) -> BTreeMap<ClassId, BTreeSet<ClassId>> {
    let mut adj: BTreeMap<ClassId, BTreeSet<ClassId>> = BTreeMap::new();
    for (winner, loser) in edges.keys() {
        adj.entry(winner.clone()).or_default().insert(loser.clone());
    }
    adj
}

fn transpose(adj: &BTreeMap<ClassId, BTreeSet<ClassId>>) -> BTreeMap<ClassId, BTreeSet<ClassId>> {
    let mut out: BTreeMap<ClassId, BTreeSet<ClassId>> = BTreeMap::new();
    for (from, tos) in adj {
        for to in tos {
            out.entry(to.clone()).or_default().insert(from.clone());
        }
    }
    out
}

fn reach_including(adj: &BTreeMap<ClassId, BTreeSet<ClassId>>, start: &str) -> BTreeSet<ClassId> {
    let mut out: BTreeSet<ClassId> = BTreeSet::new();
    out.insert(start.to_string());
    let mut stack: Vec<String> = vec![start.to_string()];
    while let Some(cur) = stack.pop() {
        let Some(next) = adj.get(&cur) else {
            continue;
        };
        for n in next {
            if out.insert(n.clone()) {
                stack.push(n.clone());
            }
        }
    }
    out
}

fn total_min(values: impl Iterator<Item = f64>) -> Option<f64> {
    values.reduce(|a, b| if a.total_cmp(&b).is_le() { a } else { b })
}

fn total_max(values: impl Iterator<Item = f64>) -> Option<f64> {
    values.reduce(|a, b| if a.total_cmp(&b).is_ge() { a } else { b })
}

// ---- C6: bounds ---------------------------------------------------------------

/// C6: per class, lower = max anchor strictly below (forward-reachable —
/// edges point winner → loser), upper = min anchor strictly above
/// (reverse-reachable); `Closed` only via an anchor on the class itself.
fn compute_bounds(
    all_classes: &BTreeSet<ClassId>,
    edges: &EdgeMap,
    class_anchors: &BTreeMap<ClassId, f64>,
) -> BTreeMap<ClassId, ValueBounds> {
    let adj = adjacency(edges);
    let reversed = transpose(&adj);
    let mut out: BTreeMap<ClassId, ValueBounds> = BTreeMap::new();
    for class in all_classes {
        let bounds = if let Some(&a) = class_anchors.get(class) {
            ValueBounds {
                lower: Bound::Closed(a),
                upper: Bound::Closed(a),
            }
        } else {
            let below = total_max(
                reach_including(&adj, class)
                    .into_iter()
                    .filter_map(|c| class_anchors.get(&c).copied()),
            );
            let above = total_min(
                reach_including(&reversed, class)
                    .into_iter()
                    .filter_map(|c| class_anchors.get(&c).copied()),
            );
            ValueBounds {
                lower: below.map_or(Bound::Unbounded, Bound::Open),
                upper: above.map_or(Bound::Unbounded, Bound::Open),
            }
        };
        out.insert(class.clone(), bounds);
    }
    out
}

// ---- C5: feasibility ------------------------------------------------------------

/// C5: the retained system is satisfiable iff the strict class digraph is
/// acyclic and no anchored-pair violation remains (each class holds at most
/// one anchor value by construction — asserted in `class_anchor_values`).
/// Over the dense reals those order conditions are sufficient as well as
/// necessary: values can always be squeezed into the open intervals a
/// consistent order dictates.
fn is_feasible(edges: &EdgeMap, class_anchors: &BTreeMap<ClassId, f64>) -> bool {
    let nodes: BTreeSet<ClassId> = edges
        .keys()
        .flat_map(|(u, v)| [u.clone(), v.clone()])
        .collect();
    let adj = adjacency(edges);
    let component = kosaraju(&nodes, &adj);
    let acyclic = edges
        .keys()
        .all(|(u, v)| component.get(u) != component.get(v));
    acyclic && anchor_violations_two_pass(edges, class_anchors).is_empty()
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{
        AnchorMap, Bound, CompilationStatus, ConstraintSet, EdgeMap, QuarantinePolicy,
        QuarantineReason, ValueBounds, anchor_violations_path_form, anchor_violations_two_pass,
        compile, compile_human_only,
    };
    use crate::comparison::{
        DOMAIN_VALUE, FRAME_EQUAL_EFFORT, Judgement, RaterKind, Response, RowForm,
    };

    // ---- fixtures -----------------------------------------------------------

    fn row(uid: &str, a: &str, b: &str, response: Response) -> Judgement {
        Judgement {
            uid: uid.to_string(),
            seq: 0,
            a: a.to_string(),
            b: Some(b.to_string()),
            response: Some(response),
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
            admission: None,
        }
    }

    fn prefer(uid: &str, winner: &str, loser: &str) -> Judgement {
        row(uid, winner, loser, Response::PreferA)
    }

    fn equal(uid: &str, a: &str, b: &str) -> Judgement {
        row(uid, a, b, Response::Equal)
    }

    fn anchors(pairs: &[(&str, f64)]) -> AnchorMap {
        pairs.iter().map(|&(e, v)| (e.to_string(), v)).collect()
    }

    fn compiled(rows: &[Judgement], anchors: &AnchorMap) -> ConstraintSet {
        let refs: Vec<&Judgement> = rows.iter().collect();
        compile(&refs, anchors, QuarantinePolicy::Symmetric)
    }

    fn edge_map(edges: &[(&str, &str, &str)]) -> EdgeMap {
        let mut out: EdgeMap = EdgeMap::new();
        for &(w, l, uid) in edges {
            out.entry((w.to_string(), l.to_string()))
                .or_default()
                .insert(uid.to_string());
        }
        out
    }

    fn cycle(classes: &[&str]) -> QuarantineReason {
        QuarantineReason::PreferenceCycle {
            classes: classes.iter().map(|c| c.to_string()).collect(),
        }
    }

    fn conflict(pairs: &[(&str, &str)]) -> QuarantineReason {
        QuarantineReason::AnchorConflict {
            pairs: pairs
                .iter()
                .map(|&(x, y)| (x.to_string(), y.to_string()))
                .collect(),
        }
    }

    fn edge_keys(cs: &ConstraintSet) -> Vec<(String, String)> {
        cs.edges.keys().cloned().collect()
    }

    fn class_of<'a>(cs: &'a ConstraintSet, entity: &str) -> &'a str {
        cs.classes.get(entity).expect("entity has a class")
    }

    // ---- C1 vocabulary --------------------------------------------------------

    #[test]
    fn c1_prefer_a_builds_strict_edge_winner_over_loser() {
        let rows = [prefer("j1", "A", "B")];
        let cs = compiled(&rows, &anchors(&[]));
        assert_eq!(
            edge_keys(&cs),
            vec![("A".to_string(), "B".to_string())],
            "one edge A > B"
        );
        assert_eq!(class_of(&cs, "A"), "A");
        assert_eq!(class_of(&cs, "B"), "B");
        assert!(cs.quarantined.is_empty());
    }

    #[test]
    fn c1_prefer_b_orients_the_edge_to_the_second_entity() {
        let rows = [row("j1", "A", "B", Response::PreferB)];
        let cs = compiled(&rows, &anchors(&[]));
        assert_eq!(edge_keys(&cs), vec![("B".to_string(), "A".to_string())]);
    }

    #[test]
    fn c1_equal_merges_classes_and_class_id_is_smallest_member() {
        let rows = [equal("e1", "B", "A"), equal("e2", "B", "C")];
        let cs = compiled(&rows, &anchors(&[]));
        assert_eq!(class_of(&cs, "A"), "A");
        assert_eq!(class_of(&cs, "B"), "A");
        assert_eq!(class_of(&cs, "C"), "A");
    }

    #[test]
    fn c1_incomparable_is_no_constraint_and_builds_no_edge() {
        let rows = [row("j1", "A", "B", Response::Incomparable)];
        let cs = compiled(&rows, &anchors(&[]));
        assert!(cs.edges.is_empty());
        assert!(cs.quarantined.is_empty());
        // Entities are still evidence-bearing: they get classes.
        assert_eq!(class_of(&cs, "A"), "A");
        assert_eq!(
            cs.status_of(&rows[0]),
            CompilationStatus::NoConstraint,
            "incomparable is excluded from constraining-judgement counts"
        );
    }

    #[test]
    fn c1_anchor_lands_on_the_merged_class() {
        let rows = [equal("e1", "A", "B")];
        let cs = compiled(&rows, &anchors(&[("B", 3.0)]));
        assert_eq!(cs.anchors.get("A"), Some(&3.0));
        assert_eq!(cs.anchors.len(), 1);
    }

    #[test]
    fn c1_anchor_without_row_evidence_gets_no_class() {
        let rows = [prefer("j1", "A", "B")];
        let cs = compiled(&rows, &anchors(&[("Z", 9.0)]));
        assert!(!cs.classes.contains_key("Z"));
        assert!(cs.anchors.is_empty());
    }

    #[test]
    fn c1_magnitude_stays_uncompiled_pure_order_semantics() {
        let mut ratio = row("j1", "A", "B", Response::PreferA);
        ratio.form = RowForm::Ratio;
        ratio.magnitude = Some(2.5);
        let rows = [ratio];
        let cs = compiled(&rows, &anchors(&[]));
        // The response compiles as order evidence; the magnitude never
        // becomes an anchor or a bound (anchors are the only magnitude
        // source, D8).
        assert_eq!(edge_keys(&cs), vec![("A".to_string(), "B".to_string())]);
        assert!(cs.anchors.is_empty());
        assert_eq!(
            cs.bounds.get("A"),
            Some(&ValueBounds {
                lower: Bound::Unbounded,
                upper: Bound::Unbounded,
            })
        );
    }

    // ---- VT-1: C3 preference cycles ------------------------------------------

    #[test]
    fn c3_obligation_1_triangle_quarantined_external_edge_retained() {
        // A > B > C > A plus A > D: exactly the cycle rows are quarantined,
        // A > D is retained, and B–D / C–D stay unconstrained.
        let rows = [
            prefer("j1", "A", "B"),
            prefer("j2", "B", "C"),
            prefer("j3", "C", "A"),
            prefer("j4", "A", "D"),
        ];
        let cs = compiled(&rows, &anchors(&[]));
        let reason = cycle(&["A", "B", "C"]);
        assert_eq!(cs.quarantined.get("j1"), Some(&reason));
        assert_eq!(cs.quarantined.get("j2"), Some(&reason));
        assert_eq!(cs.quarantined.get("j3"), Some(&reason));
        assert_eq!(cs.quarantined.len(), 3);
        assert_eq!(
            edge_keys(&cs),
            vec![("A".to_string(), "D".to_string())],
            "the external member-level edge A > D is untouched"
        );
        assert_eq!(cs.status_of(&rows[3]), CompilationStatus::Constraining);
        assert_eq!(
            cs.bounds.get("D"),
            Some(&ValueBounds {
                lower: Bound::Unbounded,
                upper: Bound::Unbounded,
            }),
            "no anchors anywhere: D is unconstrained in value"
        );
    }

    #[test]
    fn c3_two_cycle_quarantines_both_rows() {
        let rows = [prefer("j1", "A", "B"), prefer("j2", "B", "A")];
        let cs = compiled(&rows, &anchors(&[]));
        let reason = cycle(&["A", "B"]);
        assert_eq!(cs.quarantined.get("j1"), Some(&reason));
        assert_eq!(cs.quarantined.get("j2"), Some(&reason));
        assert!(cs.edges.is_empty());
    }

    #[test]
    fn c3_runs_before_c4_cycle_edge_also_in_anchor_conflict() {
        // Edge A > B both sits in a cycle AND lies on an anchor-violating
        // path (anchor(A)=1 ≤ anchor(C)=2 via A > B > C). C3 first: the
        // cycle rows go PreferenceCycle, and with them gone there is no
        // remaining path A ⇝ C — so j3 SURVIVES. Had C4 run first, j3 would
        // have been quarantined: the pipeline order is observable here.
        let rows = [
            prefer("j1", "A", "B"),
            prefer("j2", "B", "A"),
            prefer("j3", "B", "C"),
        ];
        let cs = compiled(&rows, &anchors(&[("A", 1.0), ("C", 2.0)]));
        let reason = cycle(&["A", "B"]);
        assert_eq!(cs.quarantined.get("j1"), Some(&reason));
        assert_eq!(cs.quarantined.get("j2"), Some(&reason));
        assert!(!cs.quarantined.contains_key("j3"));
        assert_eq!(edge_keys(&cs), vec![("B".to_string(), "C".to_string())]);
    }

    #[test]
    fn c3_strict_edge_within_an_equality_class_is_a_self_loop_cycle() {
        let rows = [equal("e1", "A", "B"), prefer("j1", "A", "B")];
        let cs = compiled(&rows, &anchors(&[]));
        assert_eq!(cs.quarantined.get("j1"), Some(&cycle(&["A"])));
        assert!(!cs.quarantined.contains_key("e1"));
        assert!(cs.edges.is_empty());
    }

    // ---- VT-2: C4 anchor conflicts --------------------------------------------

    #[test]
    fn c4_single_row_conflict_quarantined() {
        // anchor(A)=1 ≤ anchor(B)=2 yet A > B: a one-edge violation.
        let rows = [prefer("j1", "A", "B")];
        let cs = compiled(&rows, &anchors(&[("A", 1.0), ("B", 2.0)]));
        assert_eq!(cs.quarantined.get("j1"), Some(&conflict(&[("A", "B")])));
        assert!(cs.edges.is_empty());
    }

    #[test]
    fn c4_chain_conflict_quarantines_every_edge_on_the_path() {
        let rows = [prefer("j1", "A", "B"), prefer("j2", "B", "C")];
        let cs = compiled(&rows, &anchors(&[("A", 1.0), ("C", 3.0)]));
        let reason = conflict(&[("A", "C")]);
        assert_eq!(cs.quarantined.get("j1"), Some(&reason));
        assert_eq!(cs.quarantined.get("j2"), Some(&reason));
        assert!(cs.edges.is_empty());
    }

    #[test]
    fn c4_parallel_paths_between_one_pair_quarantine_all_paths() {
        let rows = [
            prefer("j1", "A", "B"),
            prefer("j2", "B", "D"),
            prefer("j3", "A", "C"),
            prefer("j4", "C", "D"),
        ];
        let cs = compiled(&rows, &anchors(&[("A", 1.0), ("D", 2.0)]));
        let reason = conflict(&[("A", "D")]);
        for uid in ["j1", "j2", "j3", "j4"] {
            assert_eq!(cs.quarantined.get(uid), Some(&reason), "row {uid}");
        }
        assert!(cs.edges.is_empty());
    }

    #[test]
    fn c4_one_edge_serving_several_conflicts_lists_every_pair() {
        // Two violating pairs (A, C) and (B, C) share the edge M > C: it is
        // quarantined once and its finding lists both pairs.
        let rows = [
            prefer("j1", "A", "M"),
            prefer("j2", "B", "M"),
            prefer("j3", "M", "C"),
        ];
        let cs = compiled(&rows, &anchors(&[("A", 1.0), ("B", 1.5), ("C", 5.0)]));
        assert_eq!(cs.quarantined.get("j1"), Some(&conflict(&[("A", "C")])));
        assert_eq!(cs.quarantined.get("j2"), Some(&conflict(&[("B", "C")])));
        assert_eq!(
            cs.quarantined.get("j3"),
            Some(&conflict(&[("A", "C"), ("B", "C")]))
        );
        assert!(cs.edges.is_empty());
    }

    #[test]
    fn c4_equal_anchor_values_across_distinct_classes_still_violate() {
        // A strict edge demands strictly-greater; equal anchors forbid it.
        let rows = [prefer("j1", "A", "B")];
        let cs = compiled(&rows, &anchors(&[("A", 2.0), ("B", 2.0)]));
        assert_eq!(cs.quarantined.get("j1"), Some(&conflict(&[("A", "B")])));
    }

    #[test]
    fn c4_consistent_anchored_chain_is_retained() {
        let rows = [prefer("j1", "A", "B"), prefer("j2", "B", "C")];
        let cs = compiled(&rows, &anchors(&[("A", 5.0), ("C", 1.0)]));
        assert!(cs.quarantined.is_empty());
        assert_eq!(cs.edges.len(), 2);
    }

    #[test]
    fn c4_path_form_matches_floor_ceiling_on_branching_goldens() {
        // Golden 1: diamond — parallel paths between one violating pair.
        let diamond = edge_map(&[
            ("A", "B", "j1"),
            ("B", "D", "j2"),
            ("A", "C", "j3"),
            ("C", "D", "j4"),
        ]);
        let diamond_anchors = anchors(&[("A", 1.0), ("D", 2.0)]);
        // Golden 2: one edge serving several conflicts.
        let shared = edge_map(&[("A", "M", "j1"), ("B", "M", "j2"), ("M", "C", "j3")]);
        let shared_anchors = anchors(&[("A", 1.0), ("B", 1.5), ("C", 5.0)]);
        // Golden 3: violating branch beside a retained branch.
        let mixed = edge_map(&[
            ("X", "P", "j1"),
            ("P", "Y", "j2"),
            ("P", "Q", "j3"),
            ("Q", "Z", "j4"),
        ]);
        let mixed_anchors = anchors(&[("X", 1.0), ("Y", 5.0), ("Z", 0.5)]);

        for (edges, anchor_map) in [
            (&diamond, &diamond_anchors),
            (&shared, &shared_anchors),
            (&mixed, &mixed_anchors),
        ] {
            let path_form: BTreeSet<(String, String)> =
                anchor_violations_path_form(edges, anchor_map)
                    .into_keys()
                    .collect();
            let two_pass = anchor_violations_two_pass(edges, anchor_map);
            assert_eq!(path_form, two_pass, "violation computations must agree");
        }
        // And the mixed golden pins WHICH edges violate: only the X ⇝ Y
        // branch; the Q ⇝ Z branch is order-consistent (1 > 0.5).
        let mixed_violations = anchor_violations_two_pass(&mixed, &mixed_anchors);
        let expected: BTreeSet<(String, String)> = [
            ("X".to_string(), "P".to_string()),
            ("P".to_string(), "Y".to_string()),
        ]
        .into_iter()
        .collect();
        assert_eq!(mixed_violations, expected);
    }

    #[test]
    fn c4_single_pass_leaves_no_residual_violation() {
        // Invariant: quarantining edges only removes paths, never creates a
        // new anchor-pair violation — re-detecting over the retained set
        // finds nothing.
        let rows = [
            prefer("j1", "A", "M"),
            prefer("j2", "B", "M"),
            prefer("j3", "M", "C"),
            prefer("j4", "C", "D"),
        ];
        let cs = compiled(&rows, &anchors(&[("A", 1.0), ("B", 1.5), ("C", 5.0)]));
        assert!(anchor_violations_two_pass(&cs.edges, &cs.anchors).is_empty());
    }

    // ---- VT-3: C2 equal-vs-anchors ---------------------------------------------

    #[test]
    fn c2_equality_path_between_differently_anchored_members_quarantined() {
        // A(1) = M = B(2) merges two anchors: both connecting rows go via
        // equality-graph violation-closure; the pendant M = W is NOT on any
        // A ⇝ B path and survives. Classes are rebuilt once from survivors.
        let rows = [
            equal("e1", "A", "M"),
            equal("e2", "M", "B"),
            equal("e3", "M", "W"),
        ];
        let cs = compiled(&rows, &anchors(&[("A", 1.0), ("B", 2.0)]));
        let reason = conflict(&[("A", "B")]);
        assert_eq!(cs.quarantined.get("e1"), Some(&reason));
        assert_eq!(cs.quarantined.get("e2"), Some(&reason));
        assert!(!cs.quarantined.contains_key("e3"));
        assert_eq!(class_of(&cs, "A"), "A");
        assert_eq!(class_of(&cs, "B"), "B");
        assert_eq!(class_of(&cs, "M"), "M");
        assert_eq!(class_of(&cs, "W"), "M");
        assert_eq!(cs.anchors.get("A"), Some(&1.0));
        assert_eq!(cs.anchors.get("B"), Some(&2.0));
    }

    #[test]
    fn c2_same_anchor_value_merge_is_retained() {
        let rows = [equal("e1", "A", "B")];
        let cs = compiled(&rows, &anchors(&[("A", 1.0), ("B", 1.0)]));
        assert!(cs.quarantined.is_empty());
        assert_eq!(class_of(&cs, "B"), "A");
        assert_eq!(cs.anchors.get("A"), Some(&1.0));
    }

    #[test]
    fn c2_parallel_equality_paths_all_quarantined() {
        // Two disjoint equality paths A–M–B and A–N–B between the same
        // conflicting anchors: closure takes every row on every path.
        let rows = [
            equal("e1", "A", "M"),
            equal("e2", "M", "B"),
            equal("e3", "A", "N"),
            equal("e4", "N", "B"),
        ];
        let cs = compiled(&rows, &anchors(&[("A", 1.0), ("B", 2.0)]));
        let reason = conflict(&[("A", "B")]);
        for uid in ["e1", "e2", "e3", "e4"] {
            assert_eq!(cs.quarantined.get(uid), Some(&reason), "row {uid}");
        }
        for entity in ["A", "B", "M", "N"] {
            assert_eq!(class_of(&cs, entity), entity, "classes rebuilt apart");
        }
    }

    // ---- C5: feasibility ---------------------------------------------------------

    /// Independent satisfiability checker (Kahn + BFS — deliberately not the
    /// Kosaraju/reach machinery compile uses): the retained digraph must be
    /// acyclic, every reachable anchored pair strictly decreasing.
    fn independently_feasible(cs: &ConstraintSet) -> bool {
        let nodes: BTreeSet<&str> = cs
            .edges
            .keys()
            .flat_map(|(u, v)| [u.as_str(), v.as_str()])
            .collect();
        let mut indegree: BTreeMap<&str, usize> = nodes.iter().map(|&n| (n, 0)).collect();
        for (_, v) in cs.edges.keys() {
            if let Some(d) = indegree.get_mut(v.as_str()) {
                *d += 1;
            }
        }
        let mut ready: Vec<&str> = indegree
            .iter()
            .filter(|&(_, &d)| d == 0)
            .map(|(&n, _)| n)
            .collect();
        let mut placed = 0_usize;
        while let Some(n) = ready.pop() {
            placed += 1;
            for (u, v) in cs.edges.keys() {
                if u.as_str() == n
                    && let Some(d) = indegree.get_mut(v.as_str())
                {
                    *d -= 1;
                    if *d == 0 {
                        ready.push(v.as_str());
                    }
                }
            }
        }
        if placed != nodes.len() {
            return false; // a cycle survived: infeasible
        }
        for (x, anchor_x) in &cs.anchors {
            let mut seen: BTreeSet<&str> = BTreeSet::new();
            let mut stack = vec![x.as_str()];
            while let Some(cur) = stack.pop() {
                for (u, v) in cs.edges.keys() {
                    if u.as_str() == cur && seen.insert(v.as_str()) {
                        stack.push(v.as_str());
                    }
                }
            }
            for y in seen {
                if let Some(anchor_y) = cs.anchors.get(y)
                    && !anchor_x.total_cmp(anchor_y).is_gt()
                {
                    return false; // a violating anchored pair survived
                }
            }
        }
        true
    }

    #[test]
    fn c5_feasibility_property_over_enumerated_ledgers() {
        // Deterministic generator — no rng, no proptest: every response
        // assignment over four entities' six pairs (4^6 ledgers), crossed
        // with three anchor configurations. compile's own debug asserts
        // (C4 agreement, C5 feasibility) also run on every case.
        const ENTITY_PAIRS: [(&str, &str); 6] = [
            ("A", "B"),
            ("A", "C"),
            ("A", "D"),
            ("B", "C"),
            ("B", "D"),
            ("C", "D"),
        ];
        let anchor_configs = [
            anchors(&[]),
            anchors(&[("A", 1.0), ("D", 2.0)]),
            anchors(&[("A", 2.0), ("C", 2.0), ("B", 5.0)]),
        ];
        for cfg in &anchor_configs {
            for mask in 0..4_u32.pow(6) {
                let mut rows = Vec::new();
                for (i, (a, b)) in ENTITY_PAIRS.iter().enumerate() {
                    let uid = format!("j{i}");
                    match (mask / 4_u32.pow(i as u32)) % 4 {
                        1 => rows.push(row(&uid, a, b, Response::PreferA)),
                        2 => rows.push(row(&uid, a, b, Response::PreferB)),
                        3 => rows.push(row(&uid, a, b, Response::Equal)),
                        _ => {}
                    }
                }
                let cs = compiled(&rows, cfg);
                assert!(
                    independently_feasible(&cs),
                    "infeasible ConstraintSet for mask {mask} cfg {cfg:?}"
                );
                // Post-C2 invariant: no class holds two distinct anchors.
                let mut per_class: BTreeMap<&String, f64> = BTreeMap::new();
                for (entity, value) in cfg {
                    let Some(class) = cs.classes.get(entity) else {
                        continue;
                    };
                    if let Some(prev) = per_class.insert(class, *value) {
                        assert!(
                            prev.total_cmp(value).is_eq(),
                            "class with two anchor values for mask {mask}"
                        );
                    }
                }
            }
        }
    }

    // ---- C6: bounds ------------------------------------------------------------

    #[test]
    fn c6_lower_is_max_anchor_below_upper_is_min_anchor_above() {
        // A(10) > M, B(6) > M, M > D(1), M > E(3): M's interval is the
        // tightest pair — Open(3) below, Open(6) above.
        let rows = [
            prefer("j1", "A", "M"),
            prefer("j2", "B", "M"),
            prefer("j3", "M", "D"),
            prefer("j4", "M", "E"),
        ];
        let cs = compiled(
            &rows,
            &anchors(&[("A", 10.0), ("B", 6.0), ("D", 1.0), ("E", 3.0)]),
        );
        assert!(cs.quarantined.is_empty());
        assert_eq!(
            cs.bounds.get("M"),
            Some(&ValueBounds {
                lower: Bound::Open(3.0),
                upper: Bound::Open(6.0),
            })
        );
    }

    #[test]
    fn c6_closed_only_via_anchor_on_the_class_itself() {
        let rows = [prefer("j1", "A", "B")];
        let cs = compiled(&rows, &anchors(&[("A", 7.0)]));
        assert_eq!(
            cs.bounds.get("A"),
            Some(&ValueBounds {
                lower: Bound::Closed(7.0),
                upper: Bound::Closed(7.0),
            })
        );
        assert_eq!(
            cs.bounds.get("B"),
            Some(&ValueBounds {
                lower: Bound::Unbounded,
                upper: Bound::Open(7.0),
            })
        );
    }

    #[test]
    fn c6_isolated_unanchored_class_is_unbounded_both_sides() {
        let rows = [row("j1", "A", "B", Response::Incomparable)];
        let cs = compiled(&rows, &anchors(&[]));
        let unbounded = ValueBounds {
            lower: Bound::Unbounded,
            upper: Bound::Unbounded,
        };
        assert_eq!(cs.bounds.get("A"), Some(&unbounded));
        assert_eq!(cs.bounds.get("B"), Some(&unbounded));
    }

    // ---- C8 / status / determinism -----------------------------------------------

    #[test]
    fn c8_status_of_assigns_all_three_compilation_statuses() {
        let rows = [
            prefer("j1", "A", "B"),
            row("j2", "C", "D", Response::Incomparable),
            prefer("j3", "X", "Y"),
            prefer("j4", "Y", "X"),
        ];
        let cs = compiled(&rows, &anchors(&[]));
        assert_eq!(cs.status_of(&rows[0]), CompilationStatus::Constraining);
        assert_eq!(cs.status_of(&rows[1]), CompilationStatus::NoConstraint);
        assert!(matches!(
            cs.status_of(&rows[2]),
            CompilationStatus::Quarantined(QuarantineReason::PreferenceCycle { .. })
        ));
    }

    #[test]
    fn compile_is_deterministic_across_input_order() {
        let rows = [
            prefer("j1", "A", "B"),
            prefer("j2", "B", "C"),
            equal("e1", "C", "D"),
            row("j3", "A", "D", Response::Incomparable),
        ];
        let anchor_map = anchors(&[("D", 4.0)]);
        let forward: Vec<&Judgement> = rows.iter().collect();
        let backward: Vec<&Judgement> = rows.iter().rev().collect();
        let cs1 = compile(&forward, &anchor_map, QuarantinePolicy::Symmetric);
        let cs2 = compile(&backward, &anchor_map, QuarantinePolicy::Symmetric);
        assert_eq!(cs1, cs2);
    }

    // ---- human-only subset compile (SL-218 PHASE-01 VT-3) ----------------------

    fn agent_prefer(uid: &str, winner: &str, loser: &str) -> Judgement {
        Judgement {
            rater: RaterKind::Agent,
            ..prefer(uid, winner, loser)
        }
    }

    /// VT-3: a human row caught in an agent-involved preference cycle is
    /// quarantined in the FULL system but retained in the human-only system —
    /// the subset compile runs its own C2–C4 rather than inheriting the full
    /// system's quarantine verdicts.
    #[test]
    fn human_only_retains_human_row_from_agent_involved_cycle() {
        // Cycle A>B>C>A: the A>B leg is human testimony, the rest agent.
        let rows = [
            prefer("h1", "A", "B"),
            agent_prefer("g1", "B", "C"),
            agent_prefer("g2", "C", "A"),
        ];
        let refs: Vec<&Judgement> = rows.iter().collect();
        let anchor_map = anchors(&[]);

        let full = compile(&refs, &anchor_map, QuarantinePolicy::Symmetric);
        assert!(
            matches!(
                full.status_of(&rows[0]),
                CompilationStatus::Quarantined(QuarantineReason::PreferenceCycle { .. })
            ),
            "full system: the human leg rides the cycle quarantine"
        );

        let human = compile_human_only(&refs, &anchor_map, QuarantinePolicy::Symmetric);
        assert_eq!(
            human.status_of(&rows[0]),
            CompilationStatus::Constraining,
            "human-only system: no cycle, the row constrains"
        );
        assert_eq!(edge_keys(&human), vec![("A".to_string(), "B".to_string())]);
        assert!(human.quarantined.is_empty());
        // Agent-row entities never entered the subset: no class for C.
        assert!(!human.classes.contains_key("C"));
    }

    /// VT-3: the subset still quarantines on its OWN violations — a human-only
    /// cycle does not survive the human-only compile.
    #[test]
    fn human_only_runs_its_own_quarantine_passes() {
        let rows = [
            prefer("h1", "X", "Y"),
            prefer("h2", "Y", "X"),
            agent_prefer("g1", "X", "Z"),
        ];
        let refs: Vec<&Judgement> = rows.iter().collect();
        let human = compile_human_only(&refs, &anchors(&[]), QuarantinePolicy::Symmetric);
        assert!(
            matches!(
                human.status_of(&rows[0]),
                CompilationStatus::Quarantined(QuarantineReason::PreferenceCycle { .. })
            ),
            "human-only 2-cycle quarantined by the subset's own C3"
        );
    }
}
