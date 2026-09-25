# ISS-481: needs: null silently no-ops against the published contract

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The defect

`doctrine design contract --format prompt` publishes the row:

```
needs   [id(inq-)]   sparse   (omit persists · null clears)
```

Sending `"needs": null` in a `Declaration` changes nothing **and reports success**.
`declare_node` matches only `Sparse::Value`:

```rust
// src/design_run/run.rs:1449
if let Sparse::Value(declared) = declaration.needs_declaration() {
```

so `Sparse::Null` falls through as a no-op. The other two sparse fields on the
same struct are wired correctly: `question` goes through `Sparse::apply`, which
handles `Null` (`src/design_run/submission.rs:31-58`), and `parent` has explicit
`Value` / `Null` arms. `needs` is the one whose `null` arm was never connected.

## Why it matters

This is `RFC-031` T1's disease in miniature: success reported for input the engine
ignored. An agent following the published contract believes it cleared the edge set
while the edges persist, so the resulting map is silently wrong — and the failure
is invisible, because the run reports a clean apply.

The only way to clear `needs` today is `[]`, which is not what the contract says
and which an agent has no reason to guess.

## Evidence

- `src/design_run/run.rs:1449` — the `Sparse::Value`-only match
- `src/design_run/submission.rs:31-58` — `Sparse::{Omitted,Null,Value}`; `Null` is
  documented as "the value is cleared"
- `doctrine design contract --format prompt` — the published row
- Found by read-only recon 2026-09-25. No existing test exercises `needs: null`.

## Shape of a fix

Handle `Sparse::Null` in the same block: clear to the empty set and emit
`NeedsRemoved` rows for the difference — exactly what `[]` already does, so the
rows machinery needs nothing new. Test at both layers: the unit arm, and an e2e
`declare` → `needs: null` → assert cleared *and* rows emitted (the `ISS-450`
failure mode was a missing row, not a missing mutation).

## Candidate co-scope

`IMP-469` — the map-growth / gate-invalidation item, same surface.
