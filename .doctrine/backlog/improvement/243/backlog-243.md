# IMP-243: Fold dep/seq into scan seam

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

Surfaced during SL-194's external design review (F-ext-4). `PriorityGraph`
construction reads each entity's TOML **twice** per build:

1. `relation_graph::scan_entities` opens+parses every entity's file for outbound
   ref/lineage edges, facets, and status (the shared scan).
2. `build_from` then calls `relation_graph::dep_seq_for(root, kind, id)` per
   entity for `needs`/`after` — explicitly "NOT part of `scan_entities`"
   (`src/priority/graph.rs:359`), re-reading the same TOMLs.

## Hypothesis

The second pass is a redundant re-parse of files the scan already opened. Folding
`needs`/`after` into the `ScannedEntity` payload would eliminate a whole per-entity
parse pass on **every** priority build — `survey`, `next`, `explain`, `blockers`,
`inspect`, `findings` — not just one verb. At ~184 entities × every invocation,
this is materially more traffic than any single feature's sweep.

**Confirm first (5 min):** verify `dep_seq_for` actually re-opens/re-parses the
file vs reading a cheap cached handle. If cached, the win shrinks and this may not
be worth the blast radius.

## Blast radius / why its own slice

- Touches `scan_entities` / the `relation_graph` seam — **shared machinery**. The
  behaviour-preservation gate (AGENTS.md) requires every priority suite stay
  byte-identical when this seam changes. Non-trivial verification burden.
- Framework perf refactor, orthogonal to any one feature. Must not be bundled into
  a feature slice.

## Payoff beyond perf

Retires SL-194 F-ext-4 for free: a single shared dep/seq map means the β-family
sweep's `base`/`lo`/`hi` builds are structurally frozen by construction (β the only
difference), so SL-194's quiescent-tree precondition (R4) dissolves.

## Relations

- Originates from SL-194 external review (F-ext-4).
- Adjacent: SL-194's β-family β-sweep (the consistency motivation).
