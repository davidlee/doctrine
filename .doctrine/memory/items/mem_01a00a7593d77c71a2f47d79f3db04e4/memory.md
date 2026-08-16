# The ADR-001 layering gate is blind to a module with no edges

Measured on SL-238 PHASE-01, 2026-08-16.

A **new root module** (`src/authored_status.rs`) declared in `src/main.rs` but
carrying no `use` lines — an empty shell staged so a red test suite could
compile — passed the **entire** `tests/architecture_layering.rs` suite: 25
passed, and **no `Unclassified` finding**, despite having no row in
`.doctrine/adr/001/layering.toml`.

The moment the implementation landed its `use crate::kinds/meta/entity` lines,
the same suite reported `Unclassified("authored_status")` and three tests went
red (`architecture_layering_gate`,
`the_existing_layering_gate_is_unchanged_in_verdict_over_the_root_tree`,
`the_layering_gate_runs_over_both_source_trees`).

## Why it matters

**"Gate green" does not mean "classified".** The gate reasons over the edge
graph, so a unit with no edges is not a node it can see. Two consequences:

- A phase that lands a module *shell* and defers its `layering.toml` row looks
  correct, and hands the red to whichever phase first adds an import. The
  failure surfaces one phase away from its cause.
- Any plan that sequences "add the module" and "add the layering row" as
  separate tasks is mis-sequenced. They are one unit — the row belongs in the
  commit that gives the module its first edge.

## What to do

When a slice adds a new top-level module, treat the `layering.toml` row as part
of the **first commit that gives it an import**, not as a later tidy-up, and do
not take an early green as evidence the classification is in place. Regenerate
authoritatively rather than hand-guessing:

    cargo test --test architecture_layering dump_real_graph -- --nocapture --ignored

See [[mem.pattern.lint.module-split-needs-layering-entry]] (the sub-module /
mixed-umbrella case, which fails *loudly* and is the opposite trap) and
[[mem.fact.layering.gate-measures-top-level-modules]] (granularity).
