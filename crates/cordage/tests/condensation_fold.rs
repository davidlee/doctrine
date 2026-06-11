//! SL-043 PHASE-02 (RSK-004) — the direction-resolved condensation fold.
//!
//! `evaluate` now runs ONE condensation fold per call instead of a per-node
//! `reachable` BFS. These tests are the silent-corruption guards the existing
//! suite cannot see (it carries NO cyclic SCC fixture):
//!
//! - **VT-1 / G1 (DOMINANT).** A fixture matrix `{Along, Against, None} × {Max,
//!   CountDistinct}` over ONE degraded `Reject` SCC, each cell asserting VALUE and
//!   CONTRIBUTOR identity against an independent per-node-BFS oracle built from the
//!   public `Graph::reachable`. The `None`×cyclic cell (C1) and the
//!   `Against`×cyclic cell (A-2) are the two surfaces the gate is blind to.
//! - **VT-2 / R4.** CountDistinct strict-exclusion: a cyclic SCC `a⇄b` + downstream
//!   `c`, and a diamond — each asserting `n ∉ its own witnesses` while
//!   `n ∈ its predecessors' witnesses`, no off-by-one.
//!
//! The oracle re-derives `fold_node` over `reachable` (the pre-fix definition):
//! idempotent combinators fold `{n} ∪ reachable`; `CountDistinct` folds STRICT
//! `reachable`. If the condensation fold and the oracle ever diverge, the fold is
//! wrong. Black-box: opaque builder ids, no `NodeId(_)` literals.

use std::collections::{BTreeMap, BTreeSet};

use cordage::{
    Arity, ChannelSpec, ChannelValue, Combinator, CyclePolicy, Direction, EdgeAttrs, Graph,
    GraphBuilder, NodeId, OverlayConfig, OverlayId,
};

fn reject_unbounded() -> OverlayConfig {
    OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded)
}

// ── the per-node-BFS oracle (the pre-fix evaluate definition) ─────────────────

/// `{n} ∪ reachable(n)` — the idempotent fold set.
fn idempotent_set(g: &Graph, ov: OverlayId, n: NodeId, dir: Direction) -> BTreeSet<NodeId> {
    let mut set = g.reachable(ov, n, dir);
    set.insert(n);
    set
}

/// Reproduce `fold_node`'s `Max` over a fold set: max value, min-`NodeId` argmax.
fn oracle_max(
    set: &BTreeSet<NodeId>,
    seeds: &BTreeMap<NodeId, ChannelValue>,
) -> Option<(ChannelValue, BTreeSet<NodeId>)> {
    let mut best: Option<(i64, NodeId)> = None;
    for &m in set {
        if let Some(ChannelValue::Scalar(v)) = seeds.get(&m).copied() {
            let wins = best.is_none_or(|(bv, bn)| v > bv || (v == bv && m < bn));
            if wins {
                best = Some((v, m));
            }
        }
    }
    best.map(|(v, argmax)| (ChannelValue::Scalar(v), BTreeSet::from([argmax])))
}

/// Reproduce `fold_count`: STRICT count of `Flag(true)` seeds in `reachable(n)`,
/// `None` when no present `Flag` seed (absence ≠ `Count(0)`, F45).
fn oracle_count(
    strict: &BTreeSet<NodeId>,
    seeds: &BTreeMap<NodeId, ChannelValue>,
) -> Option<(ChannelValue, BTreeSet<NodeId>)> {
    let mut present = false;
    let mut counted: BTreeSet<NodeId> = BTreeSet::new();
    for &m in strict {
        if let Some(ChannelValue::Flag(flag)) = seeds.get(&m).copied() {
            present = true;
            if flag {
                counted.insert(m);
            }
        }
    }
    present.then(|| {
        let count = u32::try_from(counted.len()).unwrap_or(u32::MAX);
        (ChannelValue::Count(count), counted)
    })
}

/// Assert the channel `evaluate` produced equals the per-node-BFS oracle over
/// every node, value AND contributor set, for one `(combinator, direction)`. The
/// nodes are passed as captured ids (opaque — no `NodeId` literal anywhere).
fn assert_matches_oracle(
    g: &Graph,
    ov: OverlayId,
    combinator: Combinator,
    direction: Direction,
    seeds: &BTreeMap<NodeId, ChannelValue>,
    all_nodes: &[NodeId],
) {
    let channel = g.evaluate(ChannelSpec::new(ov, combinator, direction), seeds);
    for &n in all_nodes {
        let expected = match combinator {
            Combinator::Max => oracle_max(&idempotent_set(g, ov, n, direction), seeds),
            Combinator::CountDistinct => oracle_count(&g.reachable(ov, n, direction), seeds),
            _ => unreachable!("matrix is Max × CountDistinct"),
        };
        match expected {
            Some((value, contrib)) => {
                assert_eq!(
                    channel.value(n),
                    Some(value),
                    "{combinator:?}/{direction:?} value at {n:?}"
                );
                assert_eq!(
                    *channel.contributors(n),
                    contrib,
                    "{combinator:?}/{direction:?} contributors at {n:?}"
                );
            }
            None => assert_eq!(
                channel.value(n),
                None,
                "{combinator:?}/{direction:?} absence at {n:?}"
            ),
        }
    }
}

/// The G1 fixture: a degraded `Reject` SCC `{b,c}` (b⇄c), an upstream `a→b`, and a
/// downstream `c→d`. `Along` condenses `a → {b,c} → d`; `Against` transposes it to
/// `d → {b,c} → a` — opposite fold order over the SAME SCC partition. Returns the
/// graph, the overlay, and the four captured ids (opaque — never `NodeId(_)`).
fn g1_fixture() -> (Graph, OverlayId, [NodeId; 4]) {
    let mut b = GraphBuilder::new();
    let ov = b.overlay(reject_unbounded());
    let na = b.node();
    let nb = b.node();
    let nc = b.node();
    let nd = b.node();
    b.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    b.edge(ov, nb, nc, EdgeAttrs::new(0, 0));
    b.edge(ov, nc, nb, EdgeAttrs::new(1, 0)); // b⇄c → surviving Reject SCC {b,c}
    b.edge(ov, nc, nd, EdgeAttrs::new(0, 0));
    let graph = b
        .build()
        .expect("g1 fixture builds (Reject cycle survives)");
    (graph, ov, [na, nb, nc, nd])
}

// ── VT-1 / G1 (DOMINANT): {Along, Against, None} × {Max, CountDistinct} ────────

#[test]
fn g1_matrix_max_value_and_contributors_match_per_node_bfs() {
    let (g, ov, [na, nb, nc, nd]) = g1_fixture();
    // Seed every node a distinct Scalar so the SCC's shared argmax, the min-NodeId
    // tiebreak, and the directional fold order are all exercised.
    let seeds = BTreeMap::from([
        (na, ChannelValue::Scalar(1)),
        (nb, ChannelValue::Scalar(5)),
        (nc, ChannelValue::Scalar(5)), // tie with b inside the SCC → min-NodeId argmax
        (nd, ChannelValue::Scalar(9)),
    ]);
    for direction in [Direction::Along, Direction::Against, Direction::None] {
        assert_matches_oracle(
            &g,
            ov,
            Combinator::Max,
            direction,
            &seeds,
            &[na, nb, nc, nd],
        );
    }
}

#[test]
fn g1_matrix_countdistinct_value_and_contributors_match_per_node_bfs() {
    let (g, ov, [na, nb, nc, nd]) = g1_fixture();
    let seeds = BTreeMap::from([
        (na, ChannelValue::Flag(true)),
        (nb, ChannelValue::Flag(true)),
        (nc, ChannelValue::Flag(false)), // present-false in the SCC (F45 surface)
        (nd, ChannelValue::Flag(true)),
    ]);
    for direction in [Direction::Along, Direction::Against, Direction::None] {
        assert_matches_oracle(
            &g,
            ov,
            Combinator::CountDistinct,
            direction,
            &seeds,
            &[na, nb, nc, nd],
        );
    }
}

/// The C1 silent-corruption cell, asserted directly (not only via the oracle): under
/// `Direction::None` the partition MUST dissolve to singletons even though `{b,c}`
/// is a stored SCC. If the fold grouped the stored SCC under `None`, `Max(b)` and
/// `Max(c)` would collapse to one shared value — here they must stay distinct.
#[test]
fn g1_none_does_not_group_the_stored_scc() {
    let (g, ov, [_na, nb, nc, _nd]) = g1_fixture();
    let seeds = BTreeMap::from([(nb, ChannelValue::Scalar(2)), (nc, ChannelValue::Scalar(7))]);
    let mx = g.evaluate(
        ChannelSpec::new(ov, Combinator::Max, Direction::None),
        &seeds,
    );
    // Each node folds ALONE under None → its own seed only.
    assert_eq!(mx.value(nb), Some(ChannelValue::Scalar(2)), "b folds alone");
    assert_eq!(mx.value(nc), Some(ChannelValue::Scalar(7)), "c folds alone");
    assert_eq!(*mx.contributors(nb), BTreeSet::from([nb]));
    assert_eq!(*mx.contributors(nc), BTreeSet::from([nc]));
}

/// The A-2 surface: `Against` over the cyclic fixture must fold the TRANSPOSED
/// condensation. From `a` (the source under Along), `Against` reaches nothing but
/// itself; from `d` (the sink under Along), `Against` reaches the whole SCC and `a`.
#[test]
fn g1_against_transposes_the_condensation() {
    let (g, ov, [na, nb, nc, nd]) = g1_fixture();
    let seeds = BTreeMap::from([
        (na, ChannelValue::Scalar(1)),
        (nb, ChannelValue::Scalar(5)),
        (nc, ChannelValue::Scalar(5)),
        (nd, ChannelValue::Scalar(9)),
    ]);
    let mx = g.evaluate(
        ChannelSpec::new(ov, Combinator::Max, Direction::Against),
        &seeds,
    );
    // d Against reaches {a, b, c} ∪ {d}: max is d's own 9.
    assert_eq!(mx.value(nd), Some(ChannelValue::Scalar(9)));
    assert_eq!(*mx.contributors(nd), BTreeSet::from([nd]));
    // a Against reaches nothing → own seed 1 only (NOT the SCC's 5 — forward fold
    // would corrupt this).
    assert_eq!(mx.value(na), Some(ChannelValue::Scalar(1)));
    assert_eq!(*mx.contributors(na), BTreeSet::from([na]));
}

// ── VT-2 / R4: CountDistinct strict-exclusion (no off-by-one) ─────────────────

#[test]
fn r4_countdistinct_strict_in_a_cyclic_scc_with_downstream() {
    // SCC a⇄b, plus b→c downstream. Seed a, b, c all true. STRICT reach excludes
    // self; the SCC witness set is shared pre-subtraction, each member removes
    // itself.
    let mut bld = GraphBuilder::new();
    let ov = bld.overlay(reject_unbounded());
    let na = bld.node();
    let nb = bld.node();
    let nc = bld.node();
    bld.edge(ov, na, nb, EdgeAttrs::new(0, 0));
    bld.edge(ov, nb, na, EdgeAttrs::new(1, 0)); // a⇄b SCC
    bld.edge(ov, nb, nc, EdgeAttrs::new(0, 0));
    let g = bld.build().expect("a⇄b + downstream builds");

    let seeds = BTreeMap::from([
        (na, ChannelValue::Flag(true)),
        (nb, ChannelValue::Flag(true)),
        (nc, ChannelValue::Flag(true)),
    ]);
    let cd = g.evaluate(
        ChannelSpec::new(ov, Combinator::CountDistinct, Direction::Along),
        &seeds,
    );

    // a STRICT reach = {b, c} → counts b,c, NOT a. n ∉ own witnesses.
    assert_eq!(cd.value(na), Some(ChannelValue::Count(2)));
    assert_eq!(*cd.contributors(na), BTreeSet::from([nb, nc]));
    assert!(!cd.contributors(na).contains(&na), "a ∉ its own witnesses");
    // b STRICT reach = {a, c} → counts a,c, NOT b.
    assert_eq!(cd.value(nb), Some(ChannelValue::Count(2)));
    assert_eq!(*cd.contributors(nb), BTreeSet::from([na, nc]));
    assert!(!cd.contributors(nb).contains(&nb), "b ∉ its own witnesses");
    // a ∈ b's witnesses (its predecessor in the SCC), b ∈ a's witnesses.
    assert!(cd.contributors(nb).contains(&na), "a ∈ b's witnesses");
    assert!(cd.contributors(na).contains(&nb), "b ∈ a's witnesses");
    // c is a sink (STRICT reach empty) → absent, not Count(0).
    assert_eq!(cd.value(nc), None, "c reaches nothing strictly → absent");
}

#[test]
fn r4_countdistinct_diamond_counts_shared_grandparent_once() {
    // Diamond g→p1→n, g→p2→n. Against from n reaches {p1,p2,g}; g counted ONCE
    // (set-union accumulator, R3). No SCC, but proves the diamond no-op and strict
    // self-exclusion on the same fold path.
    let mut bld = GraphBuilder::new();
    let ov = bld.overlay(reject_unbounded());
    let ng = bld.node();
    let np1 = bld.node();
    let np2 = bld.node();
    let nn = bld.node();
    bld.edge(ov, ng, np1, EdgeAttrs::new(0, 0));
    bld.edge(ov, ng, np2, EdgeAttrs::new(0, 0));
    bld.edge(ov, np1, nn, EdgeAttrs::new(0, 0));
    bld.edge(ov, np2, nn, EdgeAttrs::new(0, 0));
    let g = bld.build().expect("diamond builds");

    let seeds = BTreeMap::from([
        (ng, ChannelValue::Flag(true)),
        (np1, ChannelValue::Flag(true)),
        (np2, ChannelValue::Flag(true)),
    ]);
    let cd = g.evaluate(
        ChannelSpec::new(ov, Combinator::CountDistinct, Direction::Against),
        &seeds,
    );
    // n Against reach = {p1, p2, g} → all three counted, g once (diamond no-op).
    assert_eq!(cd.value(nn), Some(ChannelValue::Count(3)));
    assert_eq!(*cd.contributors(nn), BTreeSet::from([ng, np1, np2]));
    assert!(!cd.contributors(nn).contains(&nn), "n ∉ its own witnesses");
    // p1 Against reach = {g} → g counted, NOT p1.
    assert_eq!(cd.value(np1), Some(ChannelValue::Count(1)));
    assert_eq!(*cd.contributors(np1), BTreeSet::from([ng]));
    assert!(cd.contributors(nn).contains(&np1), "p1 ∈ n's witnesses");
}
