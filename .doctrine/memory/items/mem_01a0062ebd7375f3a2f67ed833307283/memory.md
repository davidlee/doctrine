# Layering gate measures top-level modules, not sub-units

`tests/architecture_layering.rs` extracts dependency edges with the **top-level
module** of the source file as the `FROM` side and the first path component after
`crate::` as the `TO` side (`extract_edges`, ~`:162-171`). `discover_units` builds
the unit set from top-level entries only, and the map check requires every
non-`::` key to appear in it.

So `count_tangle_edges` resolves each endpoint's tier through its **top-level**
name. Sub-classification rows in `.doctrine/adr/001/layering.toml`
(`"catalog::scan" = "command"`, `"priority::graph" = "engine"`, …) constrain the
*tier* assertions but **never participate in the tangle count**.

Two consequences that decide designs:

1. Reaching a **new function inside a module the source already imports** adds no
   edge and is free.
2. Reaching into a module the source does **not** import is a new edge no matter
   how deep the target sits — and **no sub-classification row rescues it**. You
   cannot site `commands::dep_seq` at engine tier to let `backlog` call it; the
   edge is recorded as `backlog → commands`, and `commands` is command tier.

## Why it bites harder than it looks

The tangle baseline is a per-tier ratchet over cyclic edges within non-trivial
SCCs. A back edge does not cost +1 — if the two modules are in *different* SCCs it
**merges two clusters**, and every edge inside the merged component becomes
cyclic. `layering.toml` says this in its own words: *a back edge entangles a
module with the core, so measure, never predict*.

Measure before designing the call, not after: build the edge set, run Tarjan,
compare the command-tier count with and without the proposed edge.

## The repair that works

Site the shared seam **below both consumers** as its own top-level module, so both
edges point downward. `SL-238` did this twice — `src/authored_status.rs` for the
per-kind status read, `src/dep_seq_ops.rs` for the kind-neutral dep/seq
operations. `SL-204` is the precedent at scale: relocating kind identity into leaf
`kinds/` dropped `integrity` to engine and took the command tangle 99 → 76.

See [[mem.pattern.layering.direction-is-not-cohesion]] — that one says a downward
edge can still be the wrong siting. This one says a *deep* edge is still a
top-level edge.
