// SPDX-License-Identifier: GPL-3.0-only
//! The backlog ordering adapter — the *consumer half* of cordage (SL-039).
//!
//! cordage owns the mechanism (a tree + typed DAG overlays, opaque ordering); this
//! module owns the **vocabulary**: it projects backlog items into [`OrderInput`],
//! builds two overlays (`depends_on` hard / `before` soft) plus one `OrderSpec`,
//! and reads the composed order and resolution provenance back out in domain terms
//! ([`ItemId`], [`Override`]). It performs **no sort of its own** — cordage composes
//! the order (design §5.4 I1). Pure and disk-free: it sees only `OrderInput`, never
//! a `BacklogItem` or the filesystem (the projection lives in `backlog::project`,
//! PHASE-03). Opaque cordage ids never escape a `pub(crate)` signature (§10 E4).
//!
//! Self-clearing dead-code scope: this is a leaf landed ahead of its consumer — the
//! CLI wiring (`order`/`dep_cycles`/`overrides`) lands in PHASE-03. Every item here
//! is exercised by the tests below under `cfg(test)`, so the suppression is scoped
//! to the non-test build; when PHASE-03's real consumer lands it goes unfulfilled
//! and forces its own removal (mem.pattern.lint.dead-code-expect-vs-cfg-test).
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-039 PHASE-02 pure adapter; the CLI consumer lands in PHASE-03. Tests exercise every item under cfg(test)."
    )
)]

use crate::backlog::ItemKind;
use cordage::{
    Arity, CyclePolicy, Direction, EdgeAttrs, EvictReason, Graph, GraphBuilder, NodeId, OrderLayer,
    OrderSpec, OverlayConfig, OverlayId,
};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

/// A backlog item handle in the ordering domain — kind + numeric id, the inputs to
/// the canonical ref (`RSK-002`). Opaque cordage ids map to and from this; callers
/// of the adapter speak only `ItemId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ItemId {
    kind: ItemKind,
    id: u32,
}

impl ItemId {
    /// Construct an item handle.
    pub(crate) fn new(kind: ItemKind, id: u32) -> Self {
        Self { kind, id }
    }

    /// The canonical ref (`RSK-002`) — rendered through `ItemKind::canonical_id`,
    /// the single source, so the adapter never re-derives the prefix.
    pub(crate) fn render(self) -> String {
        self.kind.canonical_id(self.id)
    }
}

impl Ord for ItemId {
    /// Canonical-id ascending — `(prefix, id)`, the tier-4 allocation tiebreak
    /// (design §5.1). `prefix` is `&'static`, so no per-compare allocation.
    fn cmp(&self, other: &Self) -> Ordering {
        (self.kind.prefix(), self.id).cmp(&(other.kind.prefix(), other.id))
    }
}

impl PartialOrd for ItemId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The projection of one backlog item into ordering inputs (design §5.4). The
/// adapter sees only this — `created` (the `YYYY-MM-DD` tier-2 tiebreak), the
/// derived `exposure` (the tier-3 within-level fallback), and the two outbound edge
/// lists in `ItemId` terms. No `BacklogItem`, no disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OrderInput {
    item: ItemId,
    created: String,
    exposure: u8,
    depends_on: Vec<ItemId>,
    before: Vec<ItemId>,
}

impl OrderInput {
    /// Construct an order input (PHASE-03's `project` is the production caller).
    pub(crate) fn new(
        item: ItemId,
        created: String,
        exposure: u8,
        depends_on: Vec<ItemId>,
        before: Vec<ItemId>,
    ) -> Self {
        Self {
            item,
            created,
            exposure,
            depends_on,
            before,
        }
    }
}

/// Why an authored edge did not constrain the final order — surfaced, never silent
/// (the honest record, design §5.6). Carries `ItemId`s and a pure reason; the
/// status/resolution naming is composed at render time (PHASE-03), keeping the
/// adapter pure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Override {
    from: ItemId,
    to: ItemId,
    reason: OverrideReason,
}

impl Override {
    /// The edge source (the authoring item, in `before`/`depends_on` orientation).
    pub(crate) fn from(self) -> ItemId {
        self.from
    }

    /// The edge destination.
    pub(crate) fn to(self) -> ItemId {
        self.to
    }

    /// Why the edge was dropped.
    pub(crate) fn reason(self) -> OverrideReason {
        self.reason
    }
}

/// The kind of override (design §5.6). All three are *non-fatal* — the order is
/// still produced; the dropped edge is reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OverrideReason {
    /// A `before` edge in a soft cycle, evicted to linearize (`Evict` overlay).
    SoftCycleEvicted,
    /// A `before` edge contradicting a hard `depends_on` ordering — the dependency
    /// wins, the soft preference is dropped.
    Contradicted,
    /// An edge whose endpoint is absent from the input set — skipped at ingest.
    Dangling,
}

/// The built ordering: the cordage graph, the `NodeId → ItemId` reverse map for
/// reading results back, the two named overlay handles, and the ingest-time
/// dangling drops. Construct with [`BacklogOrder::build`].
#[derive(Debug)]
pub(crate) struct BacklogOrder {
    graph: Graph,
    by_node: BTreeMap<NodeId, ItemId>,
    depends_on_overlay: OverlayId,
    before_overlay: OverlayId,
    dangling: Vec<Override>,
}

impl BacklogOrder {
    /// Build the ordering from projected inputs. Pure; the adapter performs no sort
    /// — cordage composes the order from the two overlays.
    ///
    /// Nodes are allocated in `(exposure desc, created asc, canonical-id asc)` order
    /// so the monotonic `NodeId` carries tiers 2–4 of the order key (design §5.1):
    /// the fallback that surfaces wherever no overlay edge constrains a pair.
    ///
    /// `depends_on` is the hard prerequisite — `A.depends_on = [B]` means B must
    /// precede A, so the cordage edge is **B→A** (the single D4 flip at ingest).
    /// `before` is the soft preference — `A.before = [B]` is already src-before-dst,
    /// edge **A→B**. An edge to an absent endpoint is dropped and recorded
    /// `Dangling`.
    ///
    /// # Errors
    ///
    /// Returns an error only if cordage rejects the assembled input — an adapter
    /// bug, not a recoverable condition (design A2): ids are minted from the
    /// builder, each overlay appears once in the `OrderSpec`, and both layers
    /// traverse `Along`. Propagated as an internal error for the boundary to
    /// surface, never pattern-matched for recovery.
    pub(crate) fn build(inputs: &[OrderInput]) -> anyhow::Result<Self> {
        let mut builder = GraphBuilder::new();
        let depends_on_overlay =
            builder.overlay(OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded));
        let before_overlay =
            builder.overlay(OverlayConfig::new(CyclePolicy::Evict, Arity::Unbounded));

        let mut ordered_inputs: Vec<&OrderInput> = inputs.iter().collect();
        ordered_inputs.sort_by(|a, b| {
            b.exposure
                .cmp(&a.exposure)
                .then_with(|| a.created.cmp(&b.created))
                .then_with(|| a.item.cmp(&b.item))
        });

        let mut by_item: BTreeMap<ItemId, NodeId> = BTreeMap::new();
        let mut by_node: BTreeMap<NodeId, ItemId> = BTreeMap::new();
        for input in &ordered_inputs {
            let node = builder.node();
            by_item.insert(input.item, node);
            by_node.insert(node, input.item);
        }

        let mut dangling: Vec<Override> = Vec::new();
        for input in &ordered_inputs {
            // Present by construction (just inserted); the `else` is defensive only,
            // keeping the path panic-free.
            let Some(&src) = by_item.get(&input.item) else {
                continue;
            };
            for dep in &input.depends_on {
                match by_item.get(dep) {
                    Some(&prereq) => {
                        builder.edge(depends_on_overlay, prereq, src, EdgeAttrs::new(0, 0));
                    }
                    None => dangling.push(Override {
                        from: input.item,
                        to: *dep,
                        reason: OverrideReason::Dangling,
                    }),
                }
            }
            for successor in &input.before {
                match by_item.get(successor) {
                    Some(&dst) => builder.edge(before_overlay, src, dst, EdgeAttrs::new(0, 0)),
                    None => dangling.push(Override {
                        from: input.item,
                        to: *successor,
                        reason: OverrideReason::Dangling,
                    }),
                }
            }
        }

        builder.order_spec(OrderSpec::new(vec![
            OrderLayer::new(depends_on_overlay, Direction::Along),
            OrderLayer::new(before_overlay, Direction::Along),
        ]));

        let graph = builder.build().map_err(|e| {
            anyhow::anyhow!(
                "backlog_order: cordage rejected well-formed adapter input (internal bug): {e:?}"
            )
        })?;

        Ok(Self {
            graph,
            by_node,
            depends_on_overlay,
            before_overlay,
            dangling,
        })
    }

    /// The composed total order in `ItemId` terms — cordage's level-then-`NodeId`
    /// order mapped back through the reverse index.
    pub(crate) fn ordered(&self) -> Vec<ItemId> {
        self.graph
            .ordered()
            .iter()
            .filter_map(|node| self.by_node.get(node).copied())
            .collect()
    }

    /// The diagnosed `depends_on` cycles — each an `ItemId` set (an authoring error
    /// to surface; the order is still produced, design §5.5).
    pub(crate) fn dep_cycles(&self) -> Vec<BTreeSet<ItemId>> {
        self.graph
            .provenance()
            .cycles()
            .iter()
            .filter(|cycle| cycle.overlay() == self.depends_on_overlay)
            .map(|cycle| {
                cycle
                    .nodes()
                    .iter()
                    .filter_map(|node| self.by_node.get(node).copied())
                    .collect()
            })
            .collect()
    }

    /// The dropped soft edges (design §5.6): `before` edges evicted on the soft
    /// overlay — by an intra-overlay cycle (`SoftCycleEvicted`) or a contradiction
    /// with a hard dependency (`Contradicted`) — plus the ingest-time `Dangling`
    /// drops. `ArityViolation` cannot arise on an `Unbounded` overlay (A5), so it
    /// contributes nothing.
    pub(crate) fn overrides(&self) -> Vec<Override> {
        let mut out: Vec<Override> = self
            .graph
            .provenance()
            .evictions()
            .iter()
            .filter(|evicted| evicted.overlay() == self.before_overlay)
            .filter_map(|evicted| {
                let from = self.by_node.get(&evicted.edge().src()).copied()?;
                let to = self.by_node.get(&evicted.edge().dst()).copied()?;
                let reason = match evicted.reason() {
                    EvictReason::IntraOverlayCycle => OverrideReason::SoftCycleEvicted,
                    EvictReason::UnionCycleVsLayer => OverrideReason::Contradicted,
                    EvictReason::ArityViolation => return None,
                };
                Some(Override { from, to, reason })
            })
            .collect();
        out.extend(self.dangling.iter().copied());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rsk(id: u32) -> ItemId {
        ItemId {
            kind: ItemKind::Risk,
            id,
        }
    }

    fn iss(id: u32) -> ItemId {
        ItemId {
            kind: ItemKind::Issue,
            id,
        }
    }

    fn inp(
        item: ItemId,
        created: &str,
        exposure: u8,
        depends_on: Vec<ItemId>,
        before: Vec<ItemId>,
    ) -> OrderInput {
        OrderInput::new(item, created.to_string(), exposure, depends_on, before)
    }

    fn pos(order: &[ItemId], item: ItemId) -> usize {
        order.iter().position(|x| *x == item).unwrap()
    }

    // --- T1: ItemId renders the canonical ref through the single source ---

    #[test]
    fn item_id_renders_canonical_ref() {
        assert_eq!(rsk(2).render(), "RSK-002");
        assert_eq!(iss(7).render(), "ISS-007");
        assert_eq!(ItemId::new(ItemKind::Chore, 11).render(), "CHR-011");
    }

    // --- T3: VT-2 dependency ordering ---

    #[test]
    fn depends_on_orders_the_prerequisite_first() {
        let a = rsk(1);
        let b = rsk(2);
        let inputs = vec![
            inp(a, "2026-06-01", 0, vec![b], vec![]), // A depends_on B
            inp(b, "2026-06-01", 0, vec![], vec![]),
        ];
        let order = BacklogOrder::build(&inputs).unwrap().ordered();
        assert!(
            pos(&order, b) < pos(&order, a),
            "B (prerequisite) precedes A; got {order:?}"
        );
    }

    // --- T3: VT-8 determinism under input permutation ---

    #[test]
    fn order_and_overrides_are_identical_under_input_permutation() {
        let a = rsk(1);
        let b = rsk(2);
        let c = iss(7);
        let d = iss(8);
        let forward = vec![
            inp(a, "2026-06-02", 4, vec![b], vec![]), // a depends_on b
            inp(b, "2026-06-01", 0, vec![], vec![]),
            inp(c, "2026-06-03", 8, vec![d], vec![d]), // c depends_on d AND c before d → evict
            inp(d, "2026-06-04", 8, vec![], vec![]),
        ];
        let reverse: Vec<OrderInput> = forward.iter().rev().cloned().collect();
        let fwd = BacklogOrder::build(&forward).unwrap();
        let rev = BacklogOrder::build(&reverse).unwrap();
        // Both halves of VT-8: the composed order AND the override set are stable
        // under input permutation — the allocation key, not arrival order, decides.
        assert_eq!(fwd.ordered(), rev.ordered());
        assert!(!fwd.overrides().is_empty(), "fixture must evict an edge");
        assert_eq!(fwd.overrides(), rev.overrides());
    }

    #[test]
    fn no_edges_falls_to_allocation_order() {
        // exposure desc, then created asc, then canonical-id asc.
        let hi = rsk(1); // exposure 9
        let mid = iss(7); // exposure 0, ISS < RSK
        let lo = rsk(3); // exposure 0, later created
        let inputs = vec![
            inp(lo, "2026-06-05", 0, vec![], vec![]),
            inp(mid, "2026-06-01", 0, vec![], vec![]),
            inp(hi, "2026-06-09", 9, vec![], vec![]),
        ];
        let order = BacklogOrder::build(&inputs).unwrap().ordered();
        assert_eq!(order, vec![hi, mid, lo]);
    }

    // --- T4: VT-5 dependency cycle named in ItemIds ---

    #[test]
    fn dependency_cycle_is_named_in_item_ids() {
        let a = rsk(1);
        let b = rsk(2);
        let inputs = vec![
            inp(a, "2026-06-01", 0, vec![b], vec![]),
            inp(b, "2026-06-01", 0, vec![a], vec![]),
        ];
        let cycles = BacklogOrder::build(&inputs).unwrap().dep_cycles();
        assert_eq!(cycles, vec![BTreeSet::from([a, b])]);
    }

    // --- T5: VT-3 a before contradicting a dependency is overridden ---

    #[test]
    fn before_contradicting_a_dependency_is_overridden() {
        let a = rsk(1);
        let b = rsk(2);
        let inputs = vec![
            inp(a, "2026-06-01", 0, vec![b], vec![b]), // A depends_on B AND A before B
            inp(b, "2026-06-01", 0, vec![], vec![]),
        ];
        let built = BacklogOrder::build(&inputs).unwrap();
        let order = built.ordered();
        // Dependency wins: B precedes A.
        assert!(pos(&order, b) < pos(&order, a), "dep wins; got {order:?}");
        let overrides = built.overrides();
        assert!(
            overrides.iter().any(|o| o.from() == a
                && o.to() == b
                && o.reason() == OverrideReason::Contradicted),
            "the before A→B edge is overridden as Contradicted; got {overrides:?}"
        );
    }

    // --- T5: VT-6 a soft before cycle is evicted, not fatal ---

    #[test]
    fn soft_before_cycle_is_evicted_not_fatal() {
        let x = rsk(1);
        let y = rsk(2);
        let inputs = vec![
            inp(x, "2026-06-01", 0, vec![], vec![y]), // X before Y
            inp(y, "2026-06-01", 0, vec![], vec![x]), // Y before X
        ];
        let built = BacklogOrder::build(&inputs).unwrap();
        assert_eq!(built.ordered().len(), 2, "order still produced");
        let overrides = built.overrides();
        assert_eq!(overrides.len(), 1, "one edge evicted; got {overrides:?}");
        assert_eq!(overrides[0].reason(), OverrideReason::SoftCycleEvicted);
        let (from, to) = (overrides[0].from(), overrides[0].to());
        assert!(
            (from == x && to == y) || (from == y && to == x),
            "the evicted edge is between X and Y"
        );
    }

    // --- T5: dangling endpoint dropped and recorded ---

    #[test]
    fn dangling_edge_endpoint_is_dropped_and_recorded() {
        let a = rsk(1);
        let ghost = rsk(9);
        let inputs = vec![inp(a, "2026-06-01", 0, vec![ghost], vec![])];
        let built = BacklogOrder::build(&inputs).unwrap();
        assert_eq!(built.ordered(), vec![a], "the ghost is not a node");
        let overrides = built.overrides();
        assert_eq!(overrides.len(), 1);
        assert_eq!(overrides[0].reason(), OverrideReason::Dangling);
        assert_eq!(overrides[0].from(), a);
        assert_eq!(overrides[0].to(), ghost);
    }

    // --- T6: VT-4 exposure breaks ties within a level ---

    #[test]
    fn exposure_breaks_ties_within_a_level() {
        let hi = rsk(1);
        let lo = iss(7);
        let inputs = vec![
            inp(lo, "2026-06-01", 0, vec![], vec![]),
            inp(hi, "2026-06-01", 12, vec![], vec![]),
        ];
        let order = BacklogOrder::build(&inputs).unwrap().ordered();
        assert!(
            pos(&order, hi) < pos(&order, lo),
            "high exposure precedes baseline at equal level; got {order:?}"
        );
    }

    // --- T6: §10 A1 regression — exposure is within-level, never a cross-level lift ---

    #[test]
    fn independent_baseline_is_not_buried_behind_a_blocked_high_exposure_item() {
        let top = rsk(1);
        let mid = rsk(2);
        let bottom = rsk(3);
        let free = iss(7);
        let inputs = vec![
            inp(top, "2026-06-01", 16, vec![mid], vec![]),
            inp(mid, "2026-06-01", 0, vec![bottom], vec![]),
            inp(bottom, "2026-06-01", 0, vec![], vec![]),
            inp(free, "2026-06-01", 0, vec![], vec![]),
        ];
        let order = BacklogOrder::build(&inputs).unwrap().ordered();
        // The free baseline (level 0) is NOT buried behind top (level 2) despite
        // top's maximal exposure — exposure is a within-level fallback, not a lift.
        assert!(
            pos(&order, free) < pos(&order, top),
            "independent baseline not buried; got {order:?}"
        );
        // The dependency chain still holds.
        assert!(pos(&order, bottom) < pos(&order, mid) && pos(&order, mid) < pos(&order, top));
    }

    // --- T6: a before edge (a level) beats exposure (a within-level fallback) ---

    #[test]
    fn a_before_edge_beats_exposure() {
        let hi = rsk(1);
        let lo = rsk(2);
        let inputs = vec![
            inp(hi, "2026-06-01", 16, vec![], vec![]),
            inp(lo, "2026-06-01", 0, vec![], vec![hi]), // lo before hi
        ];
        let order = BacklogOrder::build(&inputs).unwrap().ordered();
        assert!(
            pos(&order, lo) < pos(&order, hi),
            "the before edge beats hi's exposure; got {order:?}"
        );
    }

    // --- T7: VT-10 no pub(crate) signature leaks an opaque cordage id ---

    #[test]
    fn no_pub_crate_signature_leaks_a_cordage_id() {
        // Mechanical, textual scope: every source line whose first token is
        // `pub(crate)` (a fn/struct/field signature) must not name an opaque cordage
        // id. cordage's own `pub` tokens stay free; private fields (`by_node`,
        // `depends_on_overlay`) are not `pub(crate)` and are intentionally exempt.
        let src = include_str!("backlog_order.rs");
        for (idx, line) in src.lines().enumerate() {
            if line.trim_start().starts_with("pub(crate)") {
                assert!(
                    !line.contains("NodeId") && !line.contains("OverlayId"),
                    "line {}: pub(crate) signature leaks an opaque cordage id: {line}",
                    idx + 1
                );
            }
        }
    }
}
