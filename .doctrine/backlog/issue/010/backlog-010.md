# ISS-010: Read-side CLI does not surface slice outbound tier-1 relations: inspect and slice show return empty despite authored [[relation]] rows

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`inspect --help` advertises "its authored **outbound** relations", but for a
slice with authored tier-1 `[[relation]]` rows both read surfaces return empty:

- `doctrine inspect SL-NNN --json` → `"outbound": []`
- `doctrine slice show SL-NNN --json` → `"relationships": { specs: [], requirements: [], supersedes: [] }`
- `slice show` / `inspect` table views render no outbound/relationships section.

Reproduced on **SL-047** (done; 5 rows: `specs`→PRD-011/SPEC-001,
`requirements`→REQ-073/075/076) and **SL-057** (`specs`→SPEC-002/PRD-013,
`governed_by`→ADR-003/ADR-009). `validate` reports clean (read-tolerant; does not
disconfirm).

**Root cause is reader-side, not render.** Diagnosis narrows it:
- `inspect SPEC-002 --json` outbound **works** — but spec relations are TYPED
  fields (`descends_from`, `members.toml`), not `[[relation]]` rows.
- `inspect` on slice (SL-047/SL-057) and backlog (IMP-047) → `outbound: []`.
- `slice show` render is correct: `format_show`/`show_json` (`src/slice.rs`
  ~1201/1254) build relationships from the `tier1` arg via
  `relation::targets_for`. Output empty ⇒ `tier1` is empty ⇒
  `slice::relation_edges`→`relation::tier1_edges`→`read_block` (`src/relation.rs`
  ~547/494) returns **no edges** for authored slice `[[relation]]` rows.

So the generic `[[relation]]`-row read path yields nothing for the slice (and
likely backlog) source — the rows are present and legal but not parsed into
edges. Likely SL-048 wired the reader/render for SPEC's typed axes but the
generic-row consumer for slice/backlog sources never fully landed ("built ahead
of consumers"). Structural relations are write-only in practice.

Fix: make `read_block`/`tier1_edges` actually yield slice (+ backlog) `[[relation]]`
edges, surfaced in `inspect` outbound and `slice show` relationships (table+json);
check `governed_by` renders (additive, SL-048 §5.2). Companion to IMP-048 (write
verb), IMP-049 (agent guidance), ISS-009 (stale scaffold) — the relation surface
is half-wired end to end. Surfaced while scoping SL-057.
