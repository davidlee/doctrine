#!/usr/bin/env python3
"""Prototype: SL-213 tier-3 projection — monotone placement over a feasible DAG.

Model: nodes, edges (u, v) meaning value(u) > value(v). Anchors: node -> float.
Pure order semantics. Placement rules under test:

  R1 "midpoint"  — reverse-topo greedy, midpoint(floor, ceiling)
  R2 "budgeted"  — reverse-topo greedy, floor + (ceiling-floor)/(d_up+1)
                   where d_up = longest path from node up to its ceiling anchor
                   (generalises midpoint; even spacing on chains)

Gauge (anchor-free component): value = 2*DEFAULT*(h+1)/(H+2), h = longest-path
height above sinks, H = component max height. Spread in (0, 2*DEFAULT), ordered.

Below-anchor unbounded tail: synthetic floor = max(0, ceiling - STEP*(d_down+1))
then interpolate upward (keeps positives, no manufactured negatives).
Above-anchor unbounded head: floor + STEP per level (additive gauge step).
"""

DEFAULT = 1.0
STEP = 0.25  # GAUGE_STEP candidate

from collections import defaultdict

def topo(nodes, edges):
    # Kahn, deterministic by sorted name. edges u->v (u greater).
    out = defaultdict(set); inn = defaultdict(set)
    for u, v in edges: out[u].add(v); inn[v].add(u)
    # process "lowest first": reverse topological = sinks first
    indeg = {n: len(out[n]) for n in nodes}  # count of nodes below
    ready = sorted(n for n in nodes if indeg[n] == 0)
    order = []
    while ready:
        n = ready.pop(0)
        order.append(n)
        for p in sorted(inn[n]):
            indeg[p] -= 1
            if indeg[p] == 0:
                ready.append(p); ready.sort()
    assert len(order) == len(nodes), "cycle"
    return order, out, inn

def longest_up(nodes, edges, anchors):
    """d_up[v] = longest path v -> nearest ceiling-defining anchor above, and
    hi[v] = min anchor value above. Returns (hi, d_up) maps (None if unbounded)."""
    order, out, inn = topo(nodes, edges)
    hi = {}; dup = {}
    # process top-down: reverse of sinks-first order = sources first
    for v in reversed(order):
        best = None; bestd = None
        for p in inn[v]:  # p directly above v
            cand = anchors[p] if p in anchors else hi.get(p)
            d = 1 if p in anchors else (dup.get(p) + 1 if dup.get(p) is not None else None)
            if cand is not None:
                if best is None or cand < best or (cand == best and d > bestd):
                    best, bestd = cand, d
        hi[v] = best; dup[v] = bestd
    return hi, dup

def longest_down(nodes, edges, anchors):
    """d_down[v] = longest path from v down to an anchor below (for tail floors)."""
    order, out, inn = topo(nodes, edges)
    lo = {}; ddn = {}
    for v in order:  # sinks first
        best = None; bestd = None
        for s in out[v]:
            cand = anchors[s] if s in anchors else lo.get(s)
            d = 1 if s in anchors else (ddn.get(s) + 1 if ddn.get(s) is not None else None)
            if cand is not None:
                if best is None or cand > best or (cand == best and d > bestd):
                    best, bestd = cand, d
        lo[v] = best; ddn[v] = bestd
    return lo, ddn

def height(nodes, edges):
    order, out, inn = topo(nodes, edges)
    h = {}
    for v in order:
        h[v] = 0 if not out[v] else max(h[s] for s in out[v]) + 1
    return h

def depth_below_ceiling(nodes, edges, anchors):
    """For nodes with a ceiling: longest path from the ceiling anchor down to v."""
    order, out, inn = topo(nodes, edges)
    d = {}
    for v in reversed(order):  # sources first
        cands = []
        for p in inn[v]:
            if p in anchors: cands.append(1)
            elif d.get(p) is not None: cands.append(d[p] + 1)
        d[v] = max(cands) if cands else None
    return d

def place(nodes, edges, anchors, rule="budgeted"):
    """Returns dict node -> (value, provenance)."""
    if not anchors:
        h = height(nodes, edges)
        H = max(h.values()) if h else 0
        return {n: (2 * DEFAULT * (h[n] + 1) / (H + 2), "gauge") for n in nodes}
    order, out, inn = topo(nodes, edges)
    hi, dup = longest_up(nodes, edges, anchors)
    dbc = depth_below_ceiling(nodes, edges, anchors)
    val = {}
    prov = {}
    for v in order:  # sinks (lowest) first
        if v in anchors:
            f = max((val[s] for s in out[v]), default=None)
            assert f is None or f < anchors[v], f"infeasible at {v}"
            val[v] = anchors[v]; prov[v] = "authored"
            continue
        f = max((val[s] for s in out[v]), default=None)
        c = hi[v]
        if f is None and c is None:
            # anchored component but this node sees no anchor either way:
            # disconnected-from-anchors within component impossible (component
            # connectivity is undirected) BUT order paths are directed — a node
            # can be order-incomparable to every anchor. Gauge it.
            val[v] = DEFAULT; prov[v] = "gauge"
            continue
        if f is None:
            # unbounded below: synthetic positive floor below ceiling
            d = dbc[v] or 1
            f = max(0.0, c - STEP * (d + 1))
            # then interpolate one step up from synthetic floor
            k = dup[v] or 1
            val[v] = f + (c - f) / (k + 1) if rule == "budgeted" else (f + c) / 2
            prov[v] = "projected"
            continue
        if c is None:
            val[v] = f + STEP; prov[v] = "projected"
            continue
        if rule == "budgeted":
            k = dup[v] or 1
            val[v] = f + (c - f) / (k + 1)
        else:
            val[v] = (f + c) / 2
        prov[v] = "projected"
    return {n: (val[n], prov[n]) for n in nodes}

def show(title, nodes, edges, anchors, rule="budgeted"):
    r = place(nodes, edges, anchors, rule)
    ranked = sorted(r.items(), key=lambda kv: -kv[1][0])
    print(f"\n== {title} [{rule}] ==")
    for n, (v, p) in ranked:
        a = f"  ANCHOR={anchors[n]}" if n in anchors else ""
        print(f"  {n:>4}  {v:8.4f}  {p}{a}")
    # order violations check
    vals = {n: v for n, (v, p) in r.items()}
    bad = [(u, w) for u, w in edges if vals[u] <= vals[w]]
    if bad: print("  ORDER VIOLATIONS:", bad)
    return vals

chain8 = [f"n{i}" for i in range(8)]
chain8_edges = [(f"n{i}", f"n{i+1}") for i in range(7)]  # n0 highest

print("#### Scenario 1: judgement-only chain of 8, no anchors")
show("chain8 gauge", chain8, chain8_edges, {})

print("\n#### Scenario 2: judgement-only partial order (diamond + stragglers)")
po_nodes = ["a", "b", "c", "d", "e", "f", "g"]
po_edges = [("a","b"),("a","c"),("b","d"),("c","d"),("d","e"),("f","g")]  # f>g disconnected-ish (same call)
show("partial order gauge", po_nodes, po_edges, {})

print("\n#### Scenario 3: chain8 + single mid anchor n4 = 5.0")
show("mid anchor", chain8, chain8_edges, {"n4": 5.0}, "budgeted")
show("mid anchor", chain8, chain8_edges, {"n4": 5.0}, "midpoint")

print("\n#### Scenario 4: low anchor with deep tail below (negatives risk)")
show("low anchor 0.5, 5 below", [f"m{i}" for i in range(6)],
     [(f"m{i}", f"m{i+1}") for i in range(5)], {"m0": 0.5})

print("\n#### Scenario 5: two anchors bracketing a chain (2 .. 8), crowding")
br = [f"b{i}" for i in range(6)]
show("bracket 8..2", br, [(f"b{i}", f"b{i+1}") for i in range(5)],
     {"b0": 8.0, "b5": 2.0}, "budgeted")
show("bracket 8..2", br, [(f"b{i}", f"b{i+1}") for i in range(5)],
     {"b0": 8.0, "b5": 2.0}, "midpoint")

print("\n#### Scenario 6: sparse anchor added to gauged component (before/after)")
g_nodes = ["p","q","r","s","t","u"]
g_edges = [("p","q"),("q","r"),("p","s"),("s","t"),("t","u"),("q","t")]
before = show("before (gauge)", g_nodes, g_edges, {})
after = show("after: anchor s=5.0", g_nodes, g_edges, {"s": 5.0})
ob = sorted(g_nodes, key=lambda n: -before[n]); oa = sorted(g_nodes, key=lambda n: -after[n])
print("  order before:", " > ".join(ob))
print("  order after :", " > ".join(oa))

print("\n#### Scenario 7: cross-window edge (naive-interpolation breaker)")
# u in window [2,8], v in window [2,5], edge u->v. Naive per-window interp can invert.
cw_nodes = ["A8","A5","A2","u","v"]
cw_edges = [("A8","u"),("u","v"),("v","A2"),("A5","v")]
show("cross-window", cw_nodes, cw_edges, {"A8": 8.0, "A5": 5.0, "A2": 2.0})

print("\n#### Scenario 8: incremental judgement — locality & order consistency")
i_nodes = ["x","y","z","w"]
i_edges = [("x","y"),("z","w")]
b8 = show("two islands, anchor x=4", i_nodes, i_edges, {"x": 4.0})
a8 = show("add y>z (join islands)", i_nodes, i_edges + [("y","z")], {"x": 4.0})

print("\n#### Y1: join-Y, arms 3 and 2 above common sink j (gauge)")
y1n = ["a0","a1","a2","b0","b1","j"]
y1e = [("a0","a1"),("a1","a2"),("a2","j"),("b0","b1"),("b1","j")]
show("join-Y gauge", y1n, y1e, {})

print("\n#### Y2: split-Y, arms 3 and 2 below common source h (gauge)")
y2n = ["h","c0","c1","c2","d0","d1"]
y2e = [("h","c0"),("c0","c1"),("c1","c2"),("h","d0"),("d0","d1")]
show("split-Y gauge", y2n, y2e, {})

print("\n#### Y3: sensitivity — extend short arm of Y1 by one judgement (b1>b2, b2>j)")
y3n = y1n + ["b2"]
y3e = [("a0","a1"),("a1","a2"),("a2","j"),("b0","b1"),("b1","b2"),("b2","j")]
show("join-Y extended", y3n, y3e, {})

print("\n#### Y4: user pins with a cross-judgement b0>a1 (gauge)")
show("join-Y pinned b0>a1", y1n, y1e + [("b0","a1")], {})

print("\n#### Y5: user pins with an anchor instead: b0=3.0")
show("join-Y anchor b0=3", y1n, y1e, {"b0": 3.0})

print("\n#### Y6: parallel arms bracketed by shared anchors (top=8, bottom=2)")
y6n = ["T","a1","a2","a3","b1","B"]
y6e = [("T","a1"),("a1","a2"),("a2","a3"),("a3","B"),("T","b1"),("b1","B")]
show("bracketed-Y arms 3 vs 1", y6n, y6e, {"T": 8.0, "B": 2.0})

print("\n#### Y7: pin inside bracket — add judgement a2>b1")
show("bracketed-Y pinned a2>b1", y6n, y6e + [("a2","b1")], {"T": 8.0, "B": 2.0})
