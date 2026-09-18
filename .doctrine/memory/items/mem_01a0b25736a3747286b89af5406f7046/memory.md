# `<kind> show` is not a cheap per-entity read

The intuition is that `doctrine adr show ADR-004` reads one entity's two files
while `doctrine inspect ADR-004` walks the whole corpus, so `show` must be the
cheap one. It is the other way round.

Measured warm on the doctrine tree, three runs each:

| command | time |
|---|---|
| `doctrine adr show ADR-004` | 0.36 s |
| `doctrine inspect ADR-004` | 0.21 s |

`inspect` does a full corpus scan, builds the relation graph, derives inbound
edges and appends a priority actionability block — and still beats reading one
ADR by 40%.

## Why

A table-format `show` renders a value line and an estimate line, and gets each
from a helper that independently calls
`priority::graph::load_comparison_pipeline_for_root` —
`src/priority/surface.rs:1004` and `:1037`. So one entity read loads the whole
comparison pipeline **twice**. Every kind pairs the two calls: governance
(ADR/POL/STD/RFC), spec (PRD/SPEC), slice, backlog, revision, concept-map.

The pure variants that would share one load already sit beside them
(`value_line_from_pipeline`, `estimate_line_from_pipeline`), and the doc comment
on the first names the hazard outright. The `show` paths simply do not use them.

`IMP-460` carries the fix.

## What to do with this

**Do not cost a design on the assumption that `show` is per-entity and cheap.**
Two consequences that have already bitten:

- Adding derived, corpus-scale content to a `show` is not the cost objection it
  looks like — the scan is already paid. Object to it on surface grounds
  (unconditional output change, goldens moving) if you object at all.
- A second corpus-scale read inside an already-scanning command is easy to add
  without noticing. Check whether a pipeline or scan is already in hand.

Related: `IMP-459` — the corpus scan separately reads, parses and validates all
360 knowledge records and keeps only their relation edges.
