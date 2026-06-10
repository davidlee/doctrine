//! Query-time traversal and channel propagation (design §5.2 contract, §5.5
//! I5/I6/I7). Pure reads over the RESOLVED per-overlay adjacency views — never
//! over the composed order `U` or `order_keys` (those degrade *order*, not
//! *reachability*, I7). Cycle-safe over degraded `Reject` views by a visited set
//! (F12/F47). `build()`/`Graph` orchestrate; this module owns the mechanism, a
//! sibling to `resolve` (build-time) — the query-time half of the same seam.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    Channel, ChannelDiagReason, ChannelDiagnostic, ChannelSpec, ChannelValue, Combinator,
    Direction, InIndex, NodeId, OutIndex, OverlayId, ValueKind,
};

/// The strict reachable set of `start` on `overlay` in `direction` (I6/F8):
/// `start` itself is excluded even when cyclically reachable. `Along` walks
/// out-edges, `Against` walks in-edges, `None` yields ∅ (F25). A foreign overlay
/// or node yields ∅ (F14). Cycle-safe: a visited set bounds the BFS over a
/// degraded `Reject` view (F12).
pub(crate) fn reachable(
    out: &OutIndex,
    incoming: &InIndex,
    overlay: OverlayId,
    start: NodeId,
    direction: Direction,
) -> BTreeSet<NodeId> {
    let mut reached: BTreeSet<NodeId> = BTreeSet::new();
    let mut visited: BTreeSet<NodeId> = BTreeSet::new();
    visited.insert(start);
    let mut frontier: VecDeque<NodeId> = VecDeque::new();
    frontier.push_back(start);
    while let Some(node) = frontier.pop_front() {
        for next in neighbours(out, incoming, overlay, node, direction) {
            // visited carries `start`, so a cycle back to it never re-adds it —
            // strictness holds without a separate guard.
            if visited.insert(next) {
                reached.insert(next);
                frontier.push_back(next);
            }
        }
    }
    reached
}

/// The spine chain of `node`: follow the single kept parent (pass-1 arity left
/// ≤1 incoming per node on an `AtMostOne` overlay) up the `incoming` view to a
/// root or a cycle re-entry, returned **root → … → node** (ancestor-first). The
/// caller has already gated on `AtMostOne`. Cycle-safe: a visited set stops a
/// surviving `Reject` cycle at re-entry (the chain ends there).
pub(crate) fn spine_path(incoming: &InIndex, overlay: OverlayId, node: NodeId) -> Vec<NodeId> {
    let mut chain = vec![node];
    let mut visited: BTreeSet<NodeId> = BTreeSet::new();
    visited.insert(node);
    let mut cur = node;
    while let Some(parent) = single_parent(incoming, overlay, cur) {
        if !visited.insert(parent) {
            break; // re-entry into a surviving cycle — stop, chain ends here
        }
        chain.push(parent);
        cur = parent;
    }
    chain.reverse();
    chain
}

/// The kept parent of `node` on `overlay`, or `None` at a root. Post pass-1
/// arity an `AtMostOne` overlay has ≤1 in-edge per node; the first (and only) is
/// it.
fn single_parent(incoming: &InIndex, overlay: OverlayId, node: NodeId) -> Option<NodeId> {
    incoming
        .get(&overlay)
        .and_then(|by_dst| by_dst.get(&node))
        .and_then(|set| set.iter().next().map(|e| e.src))
}

/// The direct successors of `node` under `direction`: out-edge `dst`s for
/// `Along`, in-edge `src`s for `Against`, nothing for `None`. A foreign overlay
/// or node yields nothing (the index lookups return `None`).
fn neighbours(
    out: &OutIndex,
    incoming: &InIndex,
    overlay: OverlayId,
    node: NodeId,
    direction: Direction,
) -> Vec<NodeId> {
    match direction {
        Direction::Along => out
            .get(&overlay)
            .and_then(|by_src| by_src.get(&node))
            .map(|set| set.iter().map(|e| e.dst).collect())
            .unwrap_or_default(),
        Direction::Against => incoming
            .get(&overlay)
            .and_then(|by_dst| by_dst.get(&node))
            .map(|set| set.iter().map(|e| e.src).collect())
            .unwrap_or_default(),
        Direction::None => Vec::new(),
    }
}

// ── channel propagation (design §5.2 contract, F15/F34) ───────────────────────

/// Evaluate `spec` over `seeds` (design §5.2). Per node `n` (over `0..node_count`)
/// the fold set is `{n} ∪ reachable(n)` for the idempotent combinators
/// (`Any`/`All`/`Max`) and STRICT `reachable(n)` for `CountDistinct` (F34) — and
/// `reachable(_, None) = ∅` makes `Direction::None` fall out of the same
/// definition (own-seed for idempotent, always-absent for `CountDistinct`, F35).
/// The seed contract (F16/F41/F45) is enforced once up front: a foreign-`NodeId`
/// seed → `UnknownSeedNode`; a variant-mismatched seed → `SeedVariantMismatch`;
/// `UnknownSeedNode` wins (a foreign node is never variant-checked), ≤1 diagnostic
/// per node, sorted by `NodeId` (the seed `BTreeMap` is already in id order).
pub(crate) fn evaluate(
    out: &OutIndex,
    incoming: &InIndex,
    node_count: u32,
    spec: ChannelSpec,
    seeds: &BTreeMap<NodeId, ChannelValue>,
) -> Channel {
    let combinator = spec.combinator();
    let domain = seed_domain(combinator);
    let (effective, diagnostics) = vet_seeds(seeds, node_count, domain);

    let mut values: BTreeMap<NodeId, ChannelValue> = BTreeMap::new();
    let mut contributors: BTreeMap<NodeId, BTreeSet<NodeId>> = BTreeMap::new();
    for ord in 0..node_count {
        let n = NodeId(ord);
        let reach = reachable(out, incoming, spec.overlay(), n, spec.direction());
        if let Some((value, contrib)) = fold_node(combinator, n, &reach, &effective) {
            values.insert(n, value);
            if !contrib.is_empty() {
                contributors.insert(n, contrib);
            }
        }
    }
    Channel {
        values,
        contributors,
        diagnostics,
    }
}

/// Split the seed map into the **effective** seeds (known node, in-domain variant)
/// and the diagnostics for the rest. `UnknownSeedNode` is checked first, so it
/// wins over a co-located variant mismatch (F41).
fn vet_seeds(
    seeds: &BTreeMap<NodeId, ChannelValue>,
    node_count: u32,
    domain: ValueKind,
) -> (BTreeMap<NodeId, ChannelValue>, Vec<ChannelDiagnostic>) {
    let mut effective: BTreeMap<NodeId, ChannelValue> = BTreeMap::new();
    let mut diagnostics: Vec<ChannelDiagnostic> = Vec::new();
    for (&node, &value) in seeds {
        if node.0 >= node_count {
            diagnostics.push(ChannelDiagnostic {
                node,
                reason: ChannelDiagReason::UnknownSeedNode,
            });
        } else if value_kind(value) != domain {
            diagnostics.push(ChannelDiagnostic {
                node,
                reason: ChannelDiagReason::SeedVariantMismatch {
                    expected: domain,
                    actual: value_kind(value),
                },
            });
        } else {
            effective.insert(node, value);
        }
    }
    (effective, diagnostics)
}

/// Fold one node over its combinator-class fold set, or `None` if the fold set
/// holds no present effective seed (the node is then absent from `values` — no
/// combinator identity escapes, F16).
fn fold_node(
    combinator: Combinator,
    n: NodeId,
    reach: &BTreeSet<NodeId>,
    effective: &BTreeMap<NodeId, ChannelValue>,
) -> Option<(ChannelValue, BTreeSet<NodeId>)> {
    match combinator {
        Combinator::Any => fold_any(n, reach, effective),
        Combinator::All => fold_all(n, reach, effective),
        Combinator::Max => fold_max(n, reach, effective),
        Combinator::CountDistinct => fold_count(reach, effective),
    }
}

/// The idempotent fold set `{n} ∪ reachable` — `n` is never in `reach` (strict),
/// so no de-duplication is needed.
fn idempotent_members(n: NodeId, reach: &BTreeSet<NodeId>) -> impl Iterator<Item = NodeId> + '_ {
    std::iter::once(n).chain(reach.iter().copied())
}

/// Gather the present-true witnesses among `members`' effective `Flag` seeds, or
/// `None` if `members` holds no present `Flag` seed at all (so the caller can
/// distinguish absence from a present-all-false fold, F45). Shared by `Any` (over
/// `{n} ∪ reachable`) and `CountDistinct` (over strict `reachable`): both reduce
/// to "which reachable members carry a true flag", differing only in fold set and
/// output projection.
fn true_witnesses(
    members: impl Iterator<Item = NodeId>,
    effective: &BTreeMap<NodeId, ChannelValue>,
) -> Option<BTreeSet<NodeId>> {
    let mut present = false;
    let mut witnesses: BTreeSet<NodeId> = BTreeSet::new();
    for m in members {
        if let Some(ChannelValue::Flag(flag)) = effective.get(&m).copied() {
            present = true;
            if flag {
                witnesses.insert(m);
            }
        }
    }
    present.then_some(witnesses)
}

/// `Any`: OR of present `Flag` seeds; contributors = the present-true witnesses
/// (F43). A present-all-false fold set is `Flag(false)`, real data (F45).
fn fold_any(
    n: NodeId,
    reach: &BTreeSet<NodeId>,
    effective: &BTreeMap<NodeId, ChannelValue>,
) -> Option<(ChannelValue, BTreeSet<NodeId>)> {
    true_witnesses(idempotent_members(n, reach), effective)
        .map(|trues| (ChannelValue::Flag(!trues.is_empty()), trues))
}

/// `All`: AND of present `Flag` seeds; contributors = the present-false seeds if
/// the result is false, else the present-true set (F43).
fn fold_all(
    n: NodeId,
    reach: &BTreeSet<NodeId>,
    effective: &BTreeMap<NodeId, ChannelValue>,
) -> Option<(ChannelValue, BTreeSet<NodeId>)> {
    let mut present = false;
    let mut result = true;
    let mut trues: BTreeSet<NodeId> = BTreeSet::new();
    let mut falses: BTreeSet<NodeId> = BTreeSet::new();
    for m in idempotent_members(n, reach) {
        if let Some(ChannelValue::Flag(flag)) = effective.get(&m).copied() {
            present = true;
            if flag {
                trues.insert(m);
            } else {
                result = false;
                falses.insert(m);
            }
        }
    }
    if !present {
        return None;
    }
    let contributors = if result { trues } else { falses };
    Some((ChannelValue::Flag(result), contributors))
}

/// `Max`: maximum of present `Scalar` seeds; contributors = the single argmax,
/// min-`NodeId` among the maximal (F21).
fn fold_max(
    n: NodeId,
    reach: &BTreeSet<NodeId>,
    effective: &BTreeMap<NodeId, ChannelValue>,
) -> Option<(ChannelValue, BTreeSet<NodeId>)> {
    let mut best: Option<(i64, NodeId)> = None;
    for m in idempotent_members(n, reach) {
        if let Some(ChannelValue::Scalar(value)) = effective.get(&m).copied() {
            let supersedes = match best {
                None => true,
                Some((best_value, best_node)) => {
                    value > best_value || (value == best_value && m < best_node)
                }
            };
            if supersedes {
                best = Some((value, m));
            }
        }
    }
    best.map(|(value, argmax)| (ChannelValue::Scalar(value), BTreeSet::from([argmax])))
}

/// `CountDistinct`: count of STRICT-reachable `Flag(true)` seeds via a set-union
/// accumulator (diamonds are a no-op, R3); contributors = the counted set. A
/// present-all-false strict fold set is `Count(0)`, real data ≠ absence (F45).
fn fold_count(
    reach: &BTreeSet<NodeId>,
    effective: &BTreeMap<NodeId, ChannelValue>,
) -> Option<(ChannelValue, BTreeSet<NodeId>)> {
    true_witnesses(reach.iter().copied(), effective).map(|counted| {
        let count = u32::try_from(counted.len()).unwrap_or(u32::MAX);
        (ChannelValue::Count(count), counted)
    })
}

/// The seed/output `ValueKind` a combinator consumes (`Any`/`All`/`CountDistinct`
/// take `Flag`, `Max` takes `Scalar`).
fn seed_domain(combinator: Combinator) -> ValueKind {
    match combinator {
        Combinator::Any | Combinator::All | Combinator::CountDistinct => ValueKind::Flag,
        Combinator::Max => ValueKind::Scalar,
    }
}

/// The discriminant of a [`ChannelValue`].
fn value_kind(value: ChannelValue) -> ValueKind {
    match value {
        ChannelValue::Flag(_) => ValueKind::Flag,
        ChannelValue::Scalar(_) => ValueKind::Scalar,
        ChannelValue::Count(_) => ValueKind::Count,
    }
}
