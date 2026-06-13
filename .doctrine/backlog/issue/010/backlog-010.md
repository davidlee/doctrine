# ISS-010: Read-side CLI does not surface slice outbound tier-1 relations: inspect and slice show return empty despite authored [[relation]] rows

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`inspect --help` advertises "its authored **outbound** relations", but for a
slice with authored tier-1 `[[relation]]` rows both read surfaces return empty:

- `doctrine inspect SL-NNN --json` → `"outbound": []`
- `doctrine slice show SL-NNN --json` → `"relationships": { specs: [], requirements: [], supersedes: [] }`
- `slice show` / `inspect` table views render no outbound/relationships section.

Reproduced on **SL-047** (done; rows `specs`→PRD-011/SPEC-001,
`requirements`→REQ-073/075/076) and **SL-057** (`specs`→SPEC-002/PRD-013,
`governed_by`→ADR-003/ADR-009). `validate` reads the rows and reports clean, so
the rows are well-formed and legal — the gap is purely **read-side rendering**:
`read_block` (`src/relation.rs`) is not wired into the `inspect` outbound view or
the `slice show` relationships render for the post-SL-048 `[[relation]]` storage.
The authored edges are invisible at every read surface, so structural relations
are write-only in practice.

Fix: wire `read_block` tier-1 output into `inspect` (outbound) and `slice show`
(relationships), table + json. Check the new `governed_by` axis renders (additive,
SL-048 §5.2). Companion to IMP-048 (write verb), IMP-049 (agent guidance),
ISS-009 (stale scaffold) — the relation surface is half-wired end to end.
Surfaced while scoping SL-057.
