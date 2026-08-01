# DEC-102: Abstract probe rows

**Decision.** RFC-025 `probe-specs.md` H-rows are **abstract**. Each fixture
([[two-spike-fixtures]]) instantiates a row in its own idiom; the row id and its
expected kill boundary are shared.

H11 is "hostile build-time code writes outside the workspace" — instantiated as
`build.rs` on the Rust fixture, as `postinstall` on the TypeScript one. Same
row, same expected boundary, two instantiations.

## Why

It makes **altitude a measurement instead of an assertion**:

| outcome | classification |
|---|---|
| boundary holds under both instantiations | `model-level` — ports unchanged; justifies the REV |
| holds under one only | `client-local` — and that divergence is itself a finding |
| row instantiable in one ecosystem only | recorded as such; no portability claim made |

The alternative — me hand-authoring an `altitude` column — would put my
judgement where the spike is supposed to put evidence. The whole point of
running probes is to stop asserting things that can be measured.

## Consequence for the matrix

Row count stays at 16. `matrix.tsv` gains `fixture`, `vector-class`,
`instantiation`, and `altitude` columns; `altitude` is **computed from
results**, not authored.

Phase ids and criteria ids are immutable in doctrine; H-row ids inherit the same
discipline here — instantiations are added, rows are never renumbered.

## Related

- [[two-spike-fixtures]] — the fixtures that instantiate.
- [[interpretation-surface]] — the `vector-class` column's vocabulary.
- POL-002 — why the altitude distinction exists at all.
