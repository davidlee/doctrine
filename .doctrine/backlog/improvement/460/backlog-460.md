# IMP-460: Every kind's show loads the comparison pipeline twice

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What happens

A `<kind> show` in table format renders a value line and an estimate line. It
gets them from two sibling helpers:

```rust
let value_line    = priority::surface::show_value_line(&root, …)?;
let estimate_line = priority::surface::show_estimate_line(&root, …)?;
```

Each begins `let pipeline = graph::load_comparison_pipeline_for_root(root)?`
(`src/priority/surface.rs:1004`, `:1037`) — a corpus-scale load. So one entity
read loads the whole comparison pipeline **twice**.

The pure variants that would share one load already exist beside them —
`value_line_from_pipeline` (`:1015`) and `estimate_line_from_pipeline` — and
`value_line_from_pipeline`'s own doc comment names the hazard: *"for callers
that render many entities in one scan and must not reload the pipeline per
entity."* The `show` paths are not that caller, but they are its neighbour: two
loads where one would do.

## Where

Every call site pairs them, so every one doubles:

- `src/governance.rs:521` — `adr` / `policy` / `standard` / `rfc`
- `src/spec.rs:1567` — `prd` / `spec`
- `src/slice.rs:2353`
- `src/backlog.rs:1969`
- `src/revision.rs:813`
- `src/concept_map.rs:1111`

## Measured

Warm, this tree, three runs each:

| command | time |
|---|---|
| `doctrine adr show ADR-004` | 0.36 s |
| `doctrine inspect ADR-004` | 0.21 s |

`inspect` does a full corpus scan, builds the relation graph, derives inbound
edges and appends a priority actionability block — and is still **40% faster**
than reading one ADR. That inversion is the symptom.

Not proven to be entirely the double load; the second load is the obvious
suspect and the cheap thing to eliminate first. Measure again after.

## The fix

Load once per `show` and pass the pipeline to both pure variants. The seam
exists; the call sites just do not use it. Six sites, one shape.

## Related

- `SL-246` — found this while checking whether a per-entity `show` could afford
  an inbound-record count. It can: the scan is already being paid, twice. The
  slice does not fix it.
- `IMP-459` — the other corpus-scan waste found on the same pass.
