# Implementation Plan SL-046: Cross-kind relation graph spine: all-entity adapter + related/inbound query

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases, bottom-up along the ADR-001 layers: extract the shared leaf, build
the extraction seam on top of it, assemble the engine that consumes both, then
wire the command. Each phase ends green and is independently testable; the order
is dependency order, not convenience.

The slice is a **reader** (design §1). Nothing here authors a new relation or a
stored reverse field — the whole risk surface is "do we project the
already-authored outbound relations faithfully and derive inbound correctly."
That shapes the two load-bearing gates below.

## Sequencing & Rationale

**PHASE-01 — leaf + gate first.** `Projection<K>` is the one piece genuinely
shared by both adapters (D3), so it lands first; everything downstream rides it.
It is paired with the `backlog_order` swap deliberately, not deferred: the swap
is the **behaviour-preservation gate**, and a gate is only worth anything if it
runs before the code that depends on the thing it guards. If the `Projection`
primitive is subtly wrong (mint order, mint-or-get semantics), the byte-exact
`backlog order` golden catches it here — before three more consumers are built on
a broken leaf. The mint-sequence contract (C4) is the most-likely-to-break thing
in the whole slice; isolating it in the first phase keeps the blast radius small.

**PHASE-02 — extraction seam before the engine that calls it.** The six per-kind
`relation_edges` accessors + `outbound_for` dispatch are pure data extraction
with no graph dependency, so they precede `relation_graph`. Two reasons to keep
this its own phase rather than folding it into PHASE-03: (1) the six accessors
are **file-disjoint** (each edits its own kind's module) and parallelise cleanly
under dispatch, whereas PHASE-03 is a single new engine file — different
concurrency shapes; (2) "outbound correctness per kind" is a self-contained test
surface that does not need a graph to exist.

The **layering constraint** is the subtle part and the reason the vocab types
(`RelationLabel`, `RelationEdge`) are placed in PHASE-02, *below* the engine
modules: `relation_graph` calls `slice::relation_edges`, `spec::relation_edges`,
… so it **imports** every per-kind module. If `RelationEdge` lived in
`relation_graph`, each kind module would have to import `relation_graph` back to
name the return type — a cycle, forbidden by ADR-001. The types are pure data, so
they belong in the leaf tier (the `projection` leaf, or `integrity` beside
`KINDS`); the engine and all six kinds then depend *downward* only. The phase
fixes the home before any accessor names the type.

**PHASE-03 — the engine, once both inputs exist.** `relation_graph` needs the
projection (PHASE-01) and `outbound_for` (PHASE-02) in hand. It owns the scan,
the overlay map, and the `inspect` query. This is where the derivation invariants
live (I1–I4): inbound from `in_edges`, never a stored reverse field;
direct-only/one-hop so no acyclicity is assumed; `provenance()` left untouched so
the benign symmetric-`related` cycle diagnostic cannot leak. The `superseded_by`
fixture (C8/R3) and the dedupe claim (C3) are discharged here because this is the
first phase where a graph exists to query.

**PHASE-04 — command last.** Pure wiring + render over a finished engine. Split
from PHASE-03 because the render surface is its own contract (uniform
list/show/render, SL-025; SL-045's `spec req list` is the template to reuse, not
re-invent) and its own test shape (black-box CLI goldens + `--json` conformance),
distinct from the engine's in-process invariant tests.

**Why not fewer phases.** Folding 01+02 loses the isolated gate. Folding 02+03
couples the file-disjoint accessors to the single engine file (worse for
dispatch) and the layering decision gets made implicitly mid-engine instead of
deliberately. Folding 03+04 mixes in-process invariant tests with black-box CLI
goldens. The four seams are the four distinct risk/test/concurrency profiles.

**Dispatch note (informational, not plan scope).** PHASE-01 is the serial gate
everything builds on. PHASE-02's six accessors are largely file-disjoint and
parallelise. PHASE-03 and PHASE-04 are serial single-file phases downstream.
`/dispatch` (or `/execute` solo) decides parallelism per phase from this shape.

## Notes

- **Forward-coupling, out of scope (design §7 D4).** SL-046's `RelationLabel` +
  the `OverlayId → Label` map are the *seed* of ADR-010's code-authoritative
  vocabulary. SL-048 extends that same enum + legal-set table and owns the
  divergence test — it must not fork a parallel vocabulary. The vocabulary must
  cover all six edge-authoring kinds (the C1 fix lands that breadth here). Neither
  changes SL-046 code; recorded so the successor cannot fork the contract.
- **cordage is consumed unchanged** (SPEC-001 D1 locked, REQ-079) — no doctrine
  vocabulary enters the crate; the relation *kind* lives in overlay identity (D2).
- **Repo bans** ride every phase: no `HashMap`/`HashSet` (BTree), no `as` casts
  (guarded conversions), no indexing-slicing (`.get`), no `std::env::var`
  (`var_os`); `cargo clippy` zero warnings, `just check` before each commit.
