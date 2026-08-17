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

Test code is exempt: the edge collector sets `skip_cfg_test` and returns early on
a `#[cfg(test)]` item or mod body, so a `crate::commands::…` path inside
`mod tests` records no edge at all. A production-only grep is therefore not the
same assertion as the gate, and the gate is the one that counts.

## Why it bites harder than it looks

The tangle baseline is a per-tier ratchet over cyclic edges within non-trivial
SCCs. A back edge does not cost +1 — if the two modules are in *different* SCCs it
**merges two clusters**, and every edge inside the merged component becomes
cyclic. `layering.toml` says this in its own words: *a back edge entangles a
module with the core, so measure, never predict*.

Measure before designing the call, not after: build the edge set, run Tarjan,
compare the command-tier count with and without the proposed edge.

## Two repairs, and which one the seam admits

**Relocate**, when the shared seam has no dependency on either consumer's tier.
Site it **below both** as its own top-level module, so both edges point downward.
`SL-238` did this for the per-kind status read (`src/authored_status.rs`), whose
consumers are a footer shell and a doctor check and whose own reads are all
leaf-ward. `SL-204` is the precedent at scale: relocating kind identity into leaf
`kinds/` dropped `integrity` to engine and took the command tangle 99 → 76.

**Invert**, when it does not. A seam that legitimately consumes command-tier
policy cannot be relocated below its consumers — moving it makes the offending
edge *upward*, which is worse than the cycle it was introduced to remove and which
no sub-classification row launders either. Have the module that already depends
downward **supply** the operation instead: a struct of `fn` pointers filled at the
higher tier and threaded through the lower one's entry point. See
[[mem.pattern.lint.back-edge-tangle-inject-fnptr]].

`SL-238` met both seams and answered them differently, which is the useful part.
Its dep/seq operations resolve refs, consult per-kind terminality policy through
command-tier `priority::partition`, and echo — so an engine-tier `dep_seq_ops`
module was **drafted and withdrawn** (`design.md` §6, *"The alternative, and why it
lost"*), and `backlog` receives the operations by injection. Do not read the
`authored_status` relocation as the general answer; ask first whether the seam's
own dependencies point downward.

See [[mem.pattern.layering.direction-is-not-cohesion]] — that one says a downward
edge can still be the wrong siting. This one says a *deep* edge is still a
top-level edge.