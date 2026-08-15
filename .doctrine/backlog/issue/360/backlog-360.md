# ISS-360: Batch declare refuses an in-batch parent chain

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

A batched `declare` refuses when a node's parent is **also in the same batch** and
the chain runs two levels deep — declaring `A`, then `B` with `parent = A`, then
`C` with `parent = B` in one submission. Parent resolution appears to run against
the pre-batch state rather than against the batch's own accumulating result.

## Why it matters

The inquiry map's entire value is its **structure** — parent edges and blocking
edges. `RFC-026` E8.3 measured a real run at nine nodes and **zero edges of either
kind**, and this is one of the mechanical reasons why: the natural way to author a
tree is to declare it top-down in one batch, and that is exactly the shape the
engine refuses. The workaround — one submission per level — costs a revision per
level and is not obvious, so the path of least resistance is a flat map.

That makes this issue evidence in `QUE-218` (*Does the inquiry map earn a semantic
tier?*): the "under-featured vs under-used" fork cannot be read cleanly while a
mechanical obstacle sits in front of the feature.

## Evidence

- `019fc523-c616` — batch declare refused an in-batch parent chain two levels deep

## Shape of a fix

Resolve parents against the batch's running projection, not the pre-batch snapshot
— i.e. fold declarations in submission order and resolve each against the state
after its predecessors. A cycle check is then needed on the folded result, which
the flat form gets for free today.

## References

- `QUE-218` — does the inquiry map earn a semantic tier? (this is a confound in the evidence)
- `RFC-026` E8.3 — nine nodes, zero edges
- `DEC-063` — design-run mutation uses atomic sparse JSON declarations
