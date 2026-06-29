# REV REV-016 — reconcile SL-176

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Reconcile of **SL-176** (Finish Axis B), discharging **RV-192 finding F-5** (governance
ratification, deferred by design to reconcile). The substantive ratification — **ADR-018**
— is authored directly (the REV change grammar has no create-ADR action; ADR-016 set the
direct-authoring precedent). This REV carries the one spec-truth change and records the
rest of the reconcile landing.

## Reconcile narrative (SL-176)

- **[RV-192 F-5] SPEC-018 — `modify` (this row).** The Cross-corpus relation contract still
  transcribed the pre-SL-176 `references` role set and named no completion facet. Updated to
  match the corpus as migrated and ratified by ADR-018:
  - role `scoped_from` → **`originates_from`** (renamed in place; widened to
    `{SL + backlog}` sources and `Kinds(BACKLOG + SL)` targets);
  - the new **`fulfils`** label (SL → backlog) with its non-keyed **`Option<Degree>
    {full, partial}`** facet, `None ≡ Full`, degree excluded from edge identity;
  - companion **`relation-vocabulary.md`** refreshed: `Slices` row retired, `scoped_from`
    row renamed, `Fulfils` + degree added to the work→artefact class.

  Authoritative vocabulary stays in code (`RELATION_RULES`, `src/relation.rs`); the spec
  points, never transcribes (storage rule). Surfaced-for-manual at apply; landed by hand.

### Landed alongside this REV (recorded here, not REV change rows)

- **ADR-018** authored + accepted — ratifies RFC-003 § "Finish Axis B"; composes with
  ADR-016/010/004 and **partially reverses ADR-016 §2** (completion is relational via the
  degree facet). `SL-176 governed_by ADR-018` linked.
- **RFC-003** → `resolved` (`rfc status`) — its Finish-Axis-B decision is discharged by
  ADR-018; the work→canon half stayed ratified by ADR-016.
- **IMP-207** (19-row `spawned_from` retcon) and **IMP-149** (`slices` ambiguity) →
  `resolved`. IMP-210 (close-cascade hint) and IMP-156 (`--spawn-from` flag) confirmed open
  follow-ups.
- **SL-176 design.md** — one-line xref to the OQ-2 bare-entity-drift carve-out (RV-192 F-4
  optional polish).
