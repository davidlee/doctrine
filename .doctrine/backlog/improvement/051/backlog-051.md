# IMP-051: Scope+ship SPEC-019 FR-006: knowledge-record cross-kind supersession (IMP-006 verb, §6 matrix, Supersedes RECORD LifecycleOnly row)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

**Slice C of the SPEC-019 cut** (FR-006 / REQ-066). Follows SL-059 (Slice A) and
IMP-050 (Slice B, the relation seam). Scope when triaged:

- The transactional **IMP-006** supersede verb — cross-kind, `LifecycleOnly`,
  co-writing `supersedes`/`superseded_by` (the ADR-004 §5 carve-out) atomically,
  moving the predecessor to a terminal status valid for its own kind without
  changing kind. SPEC-019 is IMP-006's first real consumer.
- The `Supersedes` **RECORD-sourced `LifecycleOnly` rule row** whose `TargetSpec`
  is the four record kinds (cross-kind *within the family*, unlike the governance
  row's `SameKind`).
- The **§6 allowed-matrix** enforcement in the verb (not the contract table —
  `TargetSpec` cannot express a predecessor→successor constraint); reopening
  directions refused as a relation, not a supersession.

**Gated on IMP-006** (unbuilt). Stays the typed `[relationships]` carve-out,
storage-excluded from the tier-1 `[[relation]]` migration. Not dispatchable.
