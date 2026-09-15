# IMP-449: Slice conformance reports a slice's own lifecycle artefacts as undeclared

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What happens

`doctrine slice conformance <id>` computes the delta between `design-target`
selectors and recorded source-deltas. Every slice necessarily writes its own
`.doctrine/` lifecycle artefacts — `slice-NNN.toml`, `notes.md`, `coverage.toml`,
the memories and backlog items it harvests, its observation records — and all of
them land in the **undeclared** cell, which `/audit` calls the highest-signal
one.

Measured on two slices:

| slice | undeclared rows | lifecycle-artefact rows | noise |
|---|---|---|---|
| `SL-259` | 25 | 18 | 72% |
| `SL-256` | 13 | 10 | 77% |

## Why it matters

The cell is read at every audit, by an agent, and re-triaged from scratch each
time. Three-quarters of it is structurally incapable of being a finding: no
`design-target` selector should ever name a slice's own `notes.md`, because a
design target names code the design commits to, and declaring the slice's
authored tier would make the registry self-referential.

The cost is not only tokens. A cell that is mostly noise trains its reader to
skim, which is the failure mode the signal exists to prevent.

## Shape of a fix

Filter `.doctrine/**` out of the conformance source-delta before the cells are
computed — or, more precisely, filter the *authored lifecycle tier* the slice
owns, so a slice that genuinely changes shipped `install/` or `memory/` content
still reports it. The boundary wants stating rather than guessing: `install/`
and `memory/` are shipped assets and belong in the signal; `.doctrine/slice/`,
`.doctrine/backlog/`, `.doctrine/knowledge/`, `.doctrine/observations/` and
`.doctrine/review/` are the lifecycle's own bookkeeping and do not.

## Provenance

Raised as `RV-366` `F-5` — the `SL-259` implementation audit — and disposed
`follow-up` there. Cross-checked against `SL-256` before being called systemic.
