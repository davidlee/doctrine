// SPDX-License-Identifier: GPL-3.0-only
//! The backlog ordering adapter — the *consumer half* of cordage (SL-039).
//!
//! cordage owns the mechanism (a tree + typed DAG overlays, opaque ordering); this
//! module owns the **vocabulary**: it projects backlog items into [`OrderInput`],
//! builds two overlays (`needs` hard / `after` soft) plus one `OrderSpec`,
//! and reads the composed order and resolution provenance back out in domain terms
//! ([`ItemId`], [`Override`]). It performs **no sort of its own** — cordage composes
//! the order (design §5.4 I1). Pure and disk-free: it sees only `OrderInput`, never
//! a `BacklogItem` or the filesystem (the projection lives in `backlog::project`,
//! PHASE-03). Opaque cordage ids never escape a `pub(crate)` signature (§10 E4).
//!
//! The CLI consumer (`backlog order`/`needs`) landed in PHASE-03 (`backlog::project`,
//! `order_rows`, the set-verb cycle oracle), so the whole public surface is now
//! production-live — the PHASE-02 self-clearing `dead_code` scope removed itself per
//! plan (mem.pattern.lint.dead-code-expect-vs-cfg-test).
use crate::backlog::ItemKind;
use crate::projection::Projection;
use cordage::{
    Arity, CyclePolicy, Direction, EdgeAttrs, EvictReason, Graph, GraphBuilder, OrderLayer,
    OrderSpec, OverlayConfig, OverlayId,
};
use std::cmp::Ordering;
use std::collections::BTreeSet;

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
///
/// `after` is `(resolved `to`, rank)`: the per-edge authored `rank` rides into the
/// `after` edge's `EdgeAttrs`; the entry's index in this `Vec` supplies the `age`
/// ordinal (§5.4 — a distinct ordinal from `created`, §6 A2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OrderInput {
    item: ItemId,
    created: String,
    exposure: u8,
    needs: Vec<ItemId>,
    after: Vec<(ItemId, i32)>,
}

impl OrderInput {
    /// Construct an order input (PHASE-03's `project` is the production caller).
    pub(crate) fn new(
        item: ItemId,
        created: String,
        exposure: u8,
        needs: Vec<ItemId>,
        after: Vec<(ItemId, i32)>,
    ) -> Self {
        Self {
            item,
            created,
            exposure,
            needs,
            after,
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
    /// The edge source — the predecessor. Uniform across every reason (both
    /// authored edges flip B→A, and dangling drops adopt the same orientation):
    /// `from` should have preceded `to`; it didn't, because of [`Override::reason`].
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
    /// An `after` edge in a soft cycle, evicted to linearize (`Evict` overlay).
    SoftCycleEvicted,
    /// An `after` edge contradicting a hard `needs` ordering — the dependency
    /// wins, the soft preference is dropped.
    Contradicted,
    /// An edge whose endpoint is absent from the input set — skipped at ingest.
    Dangling,
}

/// The built ordering: the cordage graph, the `ItemId ↔ NodeId` projection for
/// reading results back, the two named overlay handles, and the ingest-time
/// dangling drops. Construct with [`BacklogOrder::build`].
#[derive(Debug)]
pub(crate) struct BacklogOrder {
    graph: Graph,
    projection: Projection<ItemId>,
    needs_overlay: OverlayId,
    after_overlay: OverlayId,
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
    /// Both authored edges point at predecessors and flip **B→A** (uniform
    /// src-before-dst, every layer `Along`, §5.1). `needs` is the hard prerequisite
    /// — `A.needs = [B]` means B must precede A, so the cordage edge is **B→A**,
    /// `EdgeAttrs::new(0, 0)` (hard edges never evict). `after` is the soft
    /// preference — `A.after = [{to=B, rank}]` means A comes after B, so the cordage
    /// edge is also **B→A**, carrying `EdgeAttrs::new(rank, age)` where `age` is the
    /// edge's index in the item's `after` array (the genuine `(rank, age, src, dst)`
    /// eviction key, §5.4). An edge to an absent endpoint is dropped and recorded
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
        let needs_overlay =
            builder.overlay(OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded));
        let after_overlay =
            builder.overlay(OverlayConfig::new(CyclePolicy::Evict, Arity::Unbounded));

        let mut ordered_inputs: Vec<&OrderInput> = inputs.iter().collect();
        ordered_inputs.sort_by(|a, b| {
            b.exposure
                .cmp(&a.exposure)
                .then_with(|| a.created.cmp(&b.created))
                .then_with(|| a.item.cmp(&b.item))
        });

        // Dedicated pre-intern pass: mint EVERY input's node first, in the sorted
        // order above, so the monotonic NodeId carries tiers 2–4 of the order key
        // (C4 — the gate's three-step shape). `intern` is mint-or-get; the inputs
        // are distinct `ItemId`s by construction (backlog ids unique, RSK-005), so
        // this asserts the precondition — a duplicate would silently reuse a node
        // and corrupt the tie-break (VT-3).
        let mut projection: Projection<ItemId> = Projection::new();
        for input in &ordered_inputs {
            assert!(
                projection.resolve(input.item).is_none(),
                "backlog_order: duplicate ItemId {} in inputs (ids must be distinct, RSK-005)",
                input.item.render()
            );
            projection.intern(&mut builder, input.item);
        }

        let mut dangling: Vec<Override> = Vec::new();
        for input in &ordered_inputs {
            // Present by construction (just interned); the `else` is defensive only,
            // keeping the path panic-free. Resolve is get-only — NEVER intern inside
            // the edge loop, which would mint in dependency-reference order (a
            // tie-break regression).
            let Some(src) = projection.resolve(input.item) else {
                continue;
            };
            for dep in &input.needs {
                match projection.resolve(*dep) {
                    // `A.needs=[B]` ⇒ B before A: edge B→A, hard edges never evict.
                    Some(prereq) => {
                        builder.edge(needs_overlay, prereq, src, EdgeAttrs::new(0, 0));
                    }
                    // The missing predecessor is `from`, the dependent `to` —
                    // the uniform B→A orientation `overrides()` reports (the
                    // evicted paths read src→from, dst→to identically).
                    None => dangling.push(Override {
                        from: *dep,
                        to: input.item,
                        reason: OverrideReason::Dangling,
                    }),
                }
            }
            for (idx, (to, rank)) in input.after.iter().enumerate() {
                match projection.resolve(*to) {
                    // `A.after=[{to=B, rank}]` ⇒ B before A: edge B→A (the flip),
                    // carrying the genuine `(rank, age)` eviction key; `age` is the
                    // entry's index in this item's `after` array (§5.4, A7).
                    Some(prereq) => {
                        let age = u64::try_from(idx).map_err(|e| {
                            anyhow::anyhow!("backlog_order: after-edge index overflows u64: {e}")
                        })?;
                        builder.edge(after_overlay, prereq, src, EdgeAttrs::new(*rank, age));
                    }
                    // The missing predecessor is `from`, the dependent `to` —
                    // matching the B→A orientation of the evicted paths.
                    None => dangling.push(Override {
                        from: *to,
                        to: input.item,
                        reason: OverrideReason::Dangling,
                    }),
                }
            }
        }

        builder.order_spec(OrderSpec::new(vec![
            OrderLayer::new(needs_overlay, Direction::Along),
            OrderLayer::new(after_overlay, Direction::Along),
        ]));

        let graph = builder.build().map_err(|e| {
            anyhow::anyhow!(
                "backlog_order: cordage rejected well-formed adapter input (internal bug): {e:?}"
            )
        })?;

        Ok(Self {
            graph,
            projection,
            needs_overlay,
            after_overlay,
            dangling,
        })
    }

    /// The composed total order in `ItemId` terms — cordage's level-then-`NodeId`
    /// order mapped back through the reverse index.
    pub(crate) fn ordered(&self) -> Vec<ItemId> {
        self.graph
            .ordered()
            .iter()
            .filter_map(|node| self.projection.key_of(*node))
            .collect()
    }

    /// The diagnosed `needs` cycles — each an `ItemId` set (an authoring error
    /// to surface; the order is still produced, design §5.5).
    pub(crate) fn dep_cycles(&self) -> Vec<BTreeSet<ItemId>> {
        self.graph
            .provenance()
            .cycles()
            .iter()
            .filter(|cycle| cycle.overlay() == self.needs_overlay)
            .map(|cycle| self.projection.remap_set(cycle.nodes()))
            .collect()
    }

    /// The dropped soft edges (design §5.6): `after` edges evicted on the soft
    /// overlay — by an intra-overlay cycle (`SoftCycleEvicted`) or a contradiction
    /// with a hard `needs` ordering (`Contradicted`) — plus the ingest-time
    /// `Dangling` drops. `ArityViolation` cannot arise on an `Unbounded` overlay
    /// (A5), so it contributes nothing.
    pub(crate) fn overrides(&self) -> Vec<Override> {
        let mut out: Vec<Override> = self
            .graph
            .provenance()
            .evictions()
            .iter()
            .filter(|evicted| evicted.overlay() == self.after_overlay)
            .filter_map(|evicted| {
                let from = self.projection.key_of(evicted.edge().src())?;
                let to = self.projection.key_of(evicted.edge().dst())?;
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
        needs: Vec<ItemId>,
        after: Vec<(ItemId, i32)>,
    ) -> OrderInput {
        OrderInput::new(item, created.to_string(), exposure, needs, after)
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

    // --- T4: VT-2 `needs` ordering (the B→A flip) ---

    #[test]
    fn needs_orders_the_prerequisite_first() {
        let a = rsk(1);
        let b = rsk(2);
        let inputs = vec![
            inp(a, "2026-06-01", 0, vec![b], vec![]), // A needs B
            inp(b, "2026-06-01", 0, vec![], vec![]),
        ];
        let order = BacklogOrder::build(&inputs).unwrap().ordered();
        assert!(
            pos(&order, b) < pos(&order, a),
            "B (prerequisite) precedes A; got {order:?}"
        );
    }

    // --- T5(b): VT-2/VT-3 `after` orders two otherwise-unordered items (the flip) ---

    #[test]
    fn after_orders_the_predecessor_first() {
        let a = rsk(1);
        let b = rsk(2);
        let inputs = vec![
            inp(a, "2026-06-01", 0, vec![], vec![(b, 0)]), // A after B ⇒ B before A
            inp(b, "2026-06-01", 0, vec![], vec![]),
        ];
        let order = BacklogOrder::build(&inputs).unwrap().ordered();
        assert!(
            pos(&order, b) < pos(&order, a),
            "B (predecessor) precedes A under the after flip; got {order:?}"
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
            inp(a, "2026-06-02", 4, vec![b], vec![]), // a needs b
            inp(b, "2026-06-01", 0, vec![], vec![]),
            inp(c, "2026-06-03", 8, vec![d], vec![]), // c needs d (edge d→c)
            inp(d, "2026-06-04", 8, vec![], vec![(c, 0)]), // d after c (edge c→d) → contradicts, evict
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

    // --- T5(a): VT-3 an `after` contradicting a `needs` is overridden ---

    #[test]
    fn after_contradicting_a_need_is_overridden() {
        let a = rsk(1);
        let b = rsk(2);
        let inputs = vec![
            inp(a, "2026-06-01", 0, vec![b], vec![]), // A needs B ⇒ edge B→A (B before A)
            inp(b, "2026-06-01", 0, vec![], vec![(a, 0)]), // B after A ⇒ edge A→B — contradicts
        ];
        let built = BacklogOrder::build(&inputs).unwrap();
        let order = built.ordered();
        // The hard `needs` wins: B precedes A.
        assert!(pos(&order, b) < pos(&order, a), "need wins; got {order:?}");
        let overrides = built.overrides();
        // The contradicting `after` edge A→B (the predecessor-flip src=A, dst=B) is dropped.
        assert!(
            overrides.iter().any(|o| o.from() == a
                && o.to() == b
                && o.reason() == OverrideReason::Contradicted),
            "the after edge A→B is overridden as Contradicted; got {overrides:?}"
        );
    }

    // --- T5: VT-3 a soft `after` cycle is evicted, not fatal ---

    #[test]
    fn soft_after_cycle_is_evicted_not_fatal() {
        let x = rsk(1);
        let y = rsk(2);
        let inputs = vec![
            inp(x, "2026-06-01", 0, vec![], vec![(y, 0)]), // X after Y ⇒ edge Y→X
            inp(y, "2026-06-01", 0, vec![], vec![(x, 0)]), // Y after X ⇒ edge X→Y — cycle
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

    // --- VT-6 mechanism (adapter half): a higher-`rank` after edge survives a soft
    // cycle; the strictly lower-`rank` edge is the one evicted (proves the genuine
    // `(rank, age, src, dst)` key, not the retired (0,0) stand-in). Full CLI VT-6 in
    // PHASE-03; the eviction mechanism is unit-proven here. ---

    #[test]
    fn lower_rank_after_edge_is_the_one_evicted_in_a_soft_cycle() {
        let x = rsk(1);
        let y = rsk(2);
        let inputs = vec![
            // X after Y, rank 5 ⇒ edge Y→X (rank 5) — the durable, high-rank preference.
            inp(x, "2026-06-01", 0, vec![], vec![(y, 5)]),
            // Y after X, rank 1 ⇒ edge X→Y (rank 1) — the weaker edge, evicted first.
            inp(y, "2026-06-01", 0, vec![], vec![(x, 1)]),
        ];
        let built = BacklogOrder::build(&inputs).unwrap();
        let overrides = built.overrides();
        assert_eq!(overrides.len(), 1, "one edge evicted; got {overrides:?}");
        assert_eq!(overrides[0].reason(), OverrideReason::SoftCycleEvicted);
        // The evicted edge is the weaker X→Y (rank 1): src=X, dst=Y. The surviving
        // Y→X (rank 5) keeps Y before X in the order.
        assert_eq!(
            (overrides[0].from(), overrides[0].to()),
            (x, y),
            "the lower-rank edge X→Y is evicted; got {overrides:?}"
        );
        let order = built.ordered();
        assert!(
            pos(&order, y) < pos(&order, x),
            "the surviving high-rank Y→X edge keeps Y before X; got {order:?}"
        );
    }

    // --- VT-6 mechanism (age half): equal-rank soft cycle, the LOWER-`age`
    // (lower array-index) edge is evicted. Proves `age` is wired from the entry's
    // index, not a constant — the missing half of the `(rank, age, src, dst)` key.
    //
    // Fixture: X=RSK-001, Y=RSK-002, Z=RSK-003, equal exposure/created ⇒ NodeIds
    // allocate canonical-id ascending: node(X) < node(Y) < node(Z). The cycle:
    //   X.after = [(Y, 0)]        ⇒ edge Y→X, age 0
    //   Y.after = [(Z, 0), (X, 0)] ⇒ edge Z→Y age 0 (clean, Z precedes Y) and the
    //                                cycle-closing edge X→Y at index 1 ⇒ age 1
    // Both cycle edges share rank 0. The eviction key is (rank, age, src, dst): with
    // equal rank, `age` decides BEFORE (src,dst). Y→X has age 0 < X→Y's age 1, so
    // Y→X is evicted. DISCRIMINATION: were `age` a constant, the tiebreak would fall
    // through to (src,dst) — X→Y has the smaller src (node(X) < node(Y)) and would
    // be evicted instead. The asserted victim Y→X flips iff age is genuinely wired.

    #[test]
    fn lower_age_after_edge_is_the_one_evicted_in_an_equal_rank_soft_cycle() {
        let x = rsk(1);
        let y = rsk(2);
        let z = rsk(3);
        let inputs = vec![
            inp(x, "2026-06-01", 0, vec![], vec![(y, 0)]), // X after Y ⇒ edge Y→X, age 0
            // Z padding at index 0 (age 0, no cycle), cycle edge X at index 1 (age 1).
            inp(y, "2026-06-01", 0, vec![], vec![(z, 0), (x, 0)]),
            inp(z, "2026-06-01", 0, vec![], vec![]),
        ];
        let built = BacklogOrder::build(&inputs).unwrap();
        let overrides = built.overrides();
        assert_eq!(overrides.len(), 1, "one edge evicted; got {overrides:?}");
        assert_eq!(overrides[0].reason(), OverrideReason::SoftCycleEvicted);
        // The lower-age edge Y→X (src=Y, dst=X, age 0) is evicted; the higher-age
        // X→Y (age 1) survives. A constant age would evict X→Y instead (smaller src).
        assert_eq!(
            (overrides[0].from(), overrides[0].to()),
            (y, x),
            "the lower-age edge Y→X is evicted; got {overrides:?}"
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
        // a.needs=[ghost] ⇒ ghost should have preceded a, but ghost is absent:
        // the missing predecessor is `from`, the dependent `a` is `to` — the
        // uniform B→A orientation shared with the evicted paths.
        assert_eq!(overrides[0].from(), ghost);
        assert_eq!(overrides[0].to(), a);
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

    // --- T6: an after edge (a level) beats exposure (a within-level fallback) ---

    #[test]
    fn an_after_edge_beats_exposure() {
        let hi = rsk(1);
        let lo = rsk(2);
        let inputs = vec![
            inp(hi, "2026-06-01", 16, vec![], vec![(lo, 0)]), // hi after lo ⇒ edge lo→hi
            inp(lo, "2026-06-01", 0, vec![], vec![]),
        ];
        let order = BacklogOrder::build(&inputs).unwrap().ordered();
        assert!(
            pos(&order, lo) < pos(&order, hi),
            "the after edge (lo→hi) beats hi's exposure; got {order:?}"
        );
    }

    // --- VT-3: the distinct-key precondition fires on a duplicate ItemId. The
    // pre-intern pass asserts each input's id is absent before interning; a
    // duplicate (a corpus invariant violation, RSK-005) must panic, never silently
    // reuse a node and corrupt the tie-break. ---

    #[test]
    #[should_panic(expected = "duplicate ItemId")]
    fn duplicate_item_id_in_inputs_trips_the_precondition() {
        let a = rsk(1);
        let inputs = vec![
            inp(a, "2026-06-01", 0, vec![], vec![]),
            inp(a, "2026-06-02", 0, vec![], vec![]), // same ItemId — must fire
        ];
        let _ = BacklogOrder::build(&inputs);
    }

    // --- T7: VT-10 no pub(crate) signature leaks an opaque cordage id ---

    #[test]
    fn no_pub_crate_signature_leaks_a_cordage_id() {
        // Mechanical, textual scope: every source line whose first token is
        // `pub(crate)` (a fn/struct/field signature) must not name an opaque cordage
        // id. cordage's own `pub` tokens stay free; private fields (`by_node`,
        // `needs_overlay`/`after_overlay`) are not `pub(crate)` and are exempt.
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
