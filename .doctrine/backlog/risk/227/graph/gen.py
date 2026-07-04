#!/usr/bin/env python3
"""RSK-227 — command-tier coupling map generator.

Reads the raw module-dependency dump (edges.txt) + the authoritative tier map
(layering.toml), isolates the command-tier same-tier subgraph, runs Tarjan SCC
to separate counted (in-SCC) from free (acyclic) edges, and emits two DOT views.

Regenerate edges.txt (see REGEN.md), then: `python3 gen.py`.
Render: `neato -Goverlap=prism -Gsplines=true -Gbgcolor=transparent \
         -Tsvg cmd_tangle.dot -o cmd_neato.svg`
"""
import re, sys
from pathlib import Path
from collections import defaultdict

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[4]                      # …/risk/227/graph → repo root
TOML = REPO / ".doctrine/adr/001/layering.toml"
EDGES = HERE / "edges.txt"

# --- parse tiers (top-level only; skip module::sub sub-classification) ---
tier = {}
in_tiers = False
for line in TOML.read_text().splitlines():
    s = line.strip()
    if s.startswith("[tiers]"): in_tiers = True; continue
    if s.startswith("[") and not s.startswith("[tiers]"):
        in_tiers = False; continue
    if not in_tiers: continue
    m = re.match(r'^([A-Za-z_][A-Za-z0-9_]*)\s*=\s*"(leaf|engine|command)"', s)
    if m: tier[m.group(1)] = m.group(2)

# --- parse edges; keep only edges between known units, drop self ---
edges = set()
for line in EDGES.read_text().splitlines():
    m = re.match(r'\s*([A-Za-z_]+)\s*->\s*([A-Za-z_]+)\s*$', line)
    if not m: continue
    a, b = m.group(1), m.group(2)
    if a in tier and b in tier and a != b:
        edges.add((a, b))

# --- command-tier same-tier subgraph ---
cmd = {u for u, t in tier.items() if t == "command"}
same = {(a, b) for (a, b) in edges if a in cmd and b in cmd}
outdeg = defaultdict(int); indeg = defaultdict(int)
for a, b in same: outdeg[a] += 1; indeg[b] += 1

# --- Tarjan SCC over the same-tier subgraph ---
adj = defaultdict(list)
for a, b in same: adj[a].append(b)
nodes = sorted(cmd)
idx = {}; low = {}; onstk = {}; stack = []; counter = [0]; sccs = []
sys.setrecursionlimit(10000)
def strong(v):
    idx[v] = low[v] = counter[0]; counter[0] += 1
    stack.append(v); onstk[v] = True
    for w in adj[v]:
        if w not in idx: strong(w); low[v] = min(low[v], low[w])
        elif onstk.get(w): low[v] = min(low[v], idx[w])
    if low[v] == idx[v]:
        comp = []
        while True:
            w = stack.pop(); onstk[w] = False; comp.append(w)
            if w == v: break
        sccs.append(comp)
for v in nodes:
    if v not in idx: strong(v)
big = [c for c in sccs if len(c) > 1]
scc_nodes = set().union(*big) if big else set()
tangle = [(a, b) for (a, b) in same if a in scc_nodes and b in scc_nodes]

print(f"command units: {len(cmd)}")
print(f"same-tier edges: {len(same)}")
print(f"non-trivial SCCs: {[sorted(c) for c in big]}")
print(f"tangle edges (in SCC): {len(tangle)}  (gate baseline 123)")
print(f"acyclic same-tier edges (NOT counted): {len(same) - len(tangle)}")
for u in sorted(cmd, key=lambda u: -outdeg[u])[:10]:
    tag = "[SCC]" if u in scc_nodes else "[acyclic]"
    print(f"  {u:14} out={outdeg[u]:2} in={indeg[u]:2} {tag}")

# --- emit DOT: command-tier same-tier subgraph ---
def size(u): return 0.4 + 0.13 * (outdeg[u] + indeg[u])
def color(u):
    d = outdeg[u]
    return ("#b91c1c" if d >= 10 else "#ea580c" if d >= 5 else
            "#d97706" if d >= 2 else "#65a30d" if d >= 1 else "#9ca3af")
L = ["digraph cmd {", '  rankdir=LR; bgcolor="transparent";',
     '  node [shape=circle, style="filled", fontname="Helvetica", fontsize=10, color="#00000022"];',
     '  edge [color="#33333366", arrowsize=0.6];']
for u in nodes:
    fc = "white" if outdeg[u] >= 5 else "#111"
    w = round(size(u), 2)
    L.append(f'  "{u}" [width={w}, height={w}, fillcolor="{color(u)}", '
             f'fontcolor="{fc}", label="{u}\\n{outdeg[u]}/{indeg[u]}"];')
for a, b in sorted(same):
    inscc = a in scc_nodes and b in scc_nodes
    style = ('color="#b91c1caa", penwidth=1.4' if inscc
             else 'color="#2563eb99", penwidth=1.0, style=dashed')
    L.append(f'  "{a}" -> "{b}" [{style}];')
L.append("}")
(HERE / "cmd_tangle.dot").write_text("\n".join(L))
print("\nwrote cmd_tangle.dot")
