# ISS-326: Layering gate exempts modules with no edges from classification

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`ADR-001`'s gate (`tests/architecture_layering.rs`) requires a `[tiers]` entry
for a module only if that module **appears in an edge**. Assertion 1 reads:

```rust
// Every unit appearing in edges must be a key in map.
for (from, to) in &filtered_edges { … Violation::Unclassified … }
```

A module with **no out-edges and no in-edges** is discovered by
`discover_units` and then never checked. The reverse assertion (`StaleEntry`)
only fires for a map key with no unit — it does not force the other direction.

## Measured

`SL-248` `PHASE-02` added `src/interpretation.rs`: `out=0` by design, and
un-imported because `src/main.rs` deliberately does not declare it (the module
crosses to `doctrine-control` through the lib target instead). Deleting its
`interpretation = "leaf"` row from `layering.toml` leaves
`architecture_layering_gate` **green**. The only test that reddens is
`every_exported_item_belongs_to_a_leaf_tier_module`, which fires for a
different reason — the module is an *export*, and the export-set assertion
independently demands a leaf classification.

So the classification held, but by a mechanism that only exists because this
particular module happens to be exported. A future out-edge-free module that is
not exported would acquire a silent exemption.

## Why it matters, and why it is not urgent

`ADR-001`'s premise is that every unit carries a tier. A unit outside the tier
map is not merely unclassified — it is a module that can later grow an out-edge
in any direction, and the reviewer of that change sees no tier to reason
against.

Benign today: the corpus has no un-imported, out-edge-free module other than
`interpretation`, which is covered.

## Fix sketch

Assertion 1 already iterates `filtered_edges`. Add the direct form beside it —
every discovered unit must be a key in the map (with `main` exempt exactly as
`StaleEntry` exempts it). That is the symmetric half of the check that already
exists in the other direction, so the gate ends up asserting a bijection
between units and non-`::` map keys rather than a one-way implication.

Expect the first run to surface any currently-unclassified isolated modules;
those are the finding, not noise.

## Related

- `RSK-227` — a different gate blind spot (intra-tier concentration), not this
  one. That risk is about what the gate cannot *see* inside a tier; this is
  about a unit the gate never asks about at all.
- `SL-248` `sec-8` claims the module "is still classified in `layering.toml`
  and still walked by the gate, which discovers units from the file tree rather
  than from `mod` declarations, so it acquires no exemption by being absent from
  the binary." The first half is true (it is in `units`); the conclusion is not.
  That prose correction is owed to `SL-248`'s reconciliation brief.
