# RSK-227: Layering gate blind to intra-tier concentration

ADR-001's layering gate (`tests/architecture_layering.rs`) governs intra-tier
structure with a single integer per tier — `count_tangle_edges ≤ baseline`.
Concentration, fan-in/out, acyclic same-tier edges, and all sub-module coupling
within a top-level unit are unmeasured. Empirically real, currently benign.

## Risk

The only intra-tier constraint is Assertion 4: `count_tangle_edges(tier) ≤
baseline` (command baseline **123**, `layering.toml:156`; test-header comment
stale at 120). It counts directed edges *inside* non-trivial (size>1) same-tier
SCCs, over **top-level units**, monotone non-increasing. Everything else about
same-tier structure is ungoverned, so coupling rot can accumulate undetected
even while the gate stays green.

## Two distinct blind spots

1. **Inter-unit (intra-tier).** The command SCC (top-level modules: `slice`,
   `backlog`, `memory`, `governance`, `dispatch`, `rec`, `commands`, …) is
   bounded only by the scalar. Ungoverned: per-module fan-in/out; **acyclic
   same-tier edges** (trivial-SCC endpoints count 0 → free DAG fan-out); edge
   identity (Assertion 2 fires only strictly upward); **concentration** —
   offsetting a removed SCC edge with a new one passes at ≤123, so a god-module
   can grow silently.
2. **Intra-unit (sub-module).** `extract_edges` collapses `src/commands/*.rs`
   → one unit `commands`; `commands::cli`'s 15-way fan-out over its siblings
   becomes a dropped self-edge. Sub-module coupling within *any* top-level unit
   is invisible by construction — below gate granularity, not a ratchet hole.

## Empirical grounding

Whole-crate out-degree, from the `#[ignore]` `dump_real_graph` diagnostic
(command-tier sources):

| unit | out | in |
|---|---|---|
| `commands` | 58 | 3 |
| `backlog` | 28 | 7 |
| `memory` | 22 | 8 |
| `rec` | 17 | 6 |
| `priority` / `knowledge` | 16 | 3 / 4 |
| `governance` | 15 | 8 |
| `dispatch` | 14 | 1 |

**Caveat — do not misread these as a fire.** Out-degree here **conflates
tiers**: most of it is *downward* edges to pure leaves, which is the correct
thin-shell shape, not rot. `commands` out=58 is the whole `src/commands/` dir
collapsed (17 submodules summed), in=3 → nothing depends back; a dispatch layer
that is a pure source with huge fan-out is structural. Leaf sinks are healthy
(`listing` in=29/out=0, `entity` 27/0, `git` 26/0). A true concentration
measure needs **same-tier-filtered** degree — not yet computed.

## Severity: LOW (latent, not active)

No active rot in the numbers — a normal thin-shell CLI with a dispatch hub. The
gap is that the gate *cannot see* concentration, so rot *could* accrue silently
(offset-within-budget, or intra-unit like the `cli` star). Capture-and-watch,
not build-a-gate.

## Status / prior art

**Documented residual, not a regression.** SL-112 D9 (`design.md:220`): *"a
uniformly-command umbrella with internal command↔command cycles still only
ratchets, not splits — acceptable (intra-tier is out of scope)."* ADR-001
gate-edge model (`adr-001.md:120-128`): sub-classification refines direction
only, not tangle; the ratchet never consults the sub-map.

## Open question & candidate mitigations

Is the residual still acceptable now that `commands` out=58 and `backlog`
out=28 exist? Don't build a degree gate pre-emptively (false-positive churn on
the baseline, no evidence of harm). Prefer, in order:

1. **Capture + baseline + trigger** (this item). Snapshot today's degrees;
   define the condition that would justify a gate — e.g. command tangle stalls
   near 123 across N slices, or a single unit's *same-tier* fan-out crosses a
   threshold. Mirrors ADR-001's existing "promote engine to its own crate when
   churn settles" escalation.
2. **Cheap visibility.** Promote `dump_real_graph` from `#[ignore]` to a
   committed snapshot artifact — makes drift visible without a gate (report,
   not ratchet). Would need same-tier filtering to be meaningful.
3. **Structural fix (if triggered).** Per ADR-001's own note, promote the
   engine to its own workspace crate so the compiler enforces boundaries; or
   split the command tier.

Relates to ADR-001 (the gate) and SL-112 (where the residual was scoped out).

## Artifacts

`graph/` — the command-tier coupling map: portable generator (`gen.py`),
captured input (`edges.txt`), DOT + rendered SVG (`cmd_neato.svg`), and a
self-contained diagnostic page (`coupling-map.html`). `graph/REGEN.md` documents
rebuild from the `dump_real_graph` diagnostic. Confirms: two SCCs (a 23-node
core + a `{commands, mcp_server}` 2-cycle), `integrity` a symmetric 13/13 nexus,
and **35 of 149** same-tier edges acyclic → uncounted by the ratchet.
