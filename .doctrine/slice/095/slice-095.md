# Bed in remaining relation gaps: slice related edges + governance supersedes migration + supersede verb

## Context

SL-048 shipped the cross-corpus relation contract (ADR-010): the `RELATION_RULES`
table, the `link`/`unlink` verbs, tier-1 `[[relation]]` storage migration, and
the `governed_by`/`consumes` labels. Two gaps remain:

- **IMP-082**: no `related` label for slice (or backlog items). `SL-X related SL-Y`
  fails — only governance kinds carry the `related` label. Cross-slice "related"
  edges are prose-only.
- **IMP-064** (SL-048 OD-3): governance `supersedes` stays as a typed
  `[relationships]` array — the only tier-1 field excluded from the corpus-wide
  migration. It needs migration to `[[relation]]` with `LifecycleOnly`
  `LinkPolicy`. The transactional `doctrine supersede` verb (IMP-006) must be
  built so the slot isn't inert after migration.

These are the last gaps in the relation surface for *existing* entity kinds.
Closing them beds in the foundation before knowledge records (SPEC-019) add
new relation-bearing entities.

## Scope & Objectives

1. **IMP-082 — add `related` for slice and backlog items.** One RELATION_RULES
   row: `SLICE, ISS, IMP, CHR, RSK, IDE → AnyNumbered`, `Writable`. Enables
   `doctrine link SL-X related SL-Y` and `doctrine link IMP-X related ADR-Y`.

2. **IMP-064 — migrate governance `supersedes` to `[[relation]]`.**
   - Deterministic migrator for governance `supersedes` typed arrays → 
     `[[relation]] label="supersedes" target="ADR-NNN"` rows.
   - The `superseded_by` reverse carve-out (ADR-004 §5) stays typed — it is
     verb-written, never hand-authored.
   - `LifecycleOnly` `LinkPolicy` already in `RELATION_RULES`; `link` already
     refuses it with the correct "use the transactional supersede verb" message.

3. **Build the `doctrine supersede` verb (IMP-006 for governance).**
   - Transactional: flips predecessor to `superseded` status + co-writes
     `supersedes` on successor and `superseded_by` on predecessor.
   - Scope: governance kinds first (ADR, POL, STD). Slice/knowledge supersession
     deferred to IMP-063/IMP-051.
   - Validation: refuses self-supersession, validates both refs exist, enforces
     same-kind constraint.

## Non-Goals

- **No record relation seam** — IMP-050/IMP-053 are out of scope (records don't
  exist yet; that's SL-096).
- **No POL/STD/slice supersession** — IMP-063 owns expanding `superseded` vocab
  beyond governance.
- **No `needs`/`after` changes** — deps/sequencing is untouched.
- **No tier-2/3 re-modelling** — lineage, interactions, members, review/rec edges
  stay typed.
- **No `related` for records** — records get their own relation labels
  (`relates_to`, `spawns`) in SL-096/IMP-050.

## Affected Surface

- `src/relation.rs` — add `related` RELATION_RULES row for SLICE + BACKLOG
- `src/governance.rs` — supersedes storage migration + read/write paths
- `src/relation.rs` — supersede verb (or new `src/supersede.rs`)
- `src/main.rs` — `supersede` CLI verb
- `.doctrine/adr/NNN/adr-NNN.toml` × N — governance `supersedes` typed → `[[relation]]`
- `.doctrine/policy/NNN/policy-NNN.toml` × N — same
- `.doctrine/standard/NNN/standard-NNN.toml` × N — same
- ADR-010 amendment — OD-3 resolved

## Risks & Assumptions

- **R1 (governance migration correctness):** the deterministic migrator mutates
  committed authored TOML; gated by before/after goldens + git-reversible.
- **R2 (supersede verb safety):** transactional two-entity write — partial failure
  leaves one entity in the wrong state. Mitigated: write successor first, then
  predecessor; a crash after step 1 is detectable (`supersedes` present but
  predecessor not `superseded`) and correctable.
- **R3 (ADR-004 §5 carve-out preserved):** `superseded_by` stays typed, verb-written
  only — the `[[relation]]` block carries outbound `supersedes` exclusively.
- **A1:** No governance entity currently has a non-empty `supersedes` array in this
  repo — the migrator is tested on seeded test data, not real drift.

## Verification / Closure Intent

- `doctrine link SL-X related SL-Y` succeeds; `SL-X related ADR-Y` succeeds.
- `doctrine link IMP-X related SPEC-Y` succeeds.
- `doctrine link SL-X related FREE-TEXT` is refused (`related` targets `AnyNumbered`,
  not `Unvalidated`).
- `doctrine supersede ADR-X ADR-Y` flips ADR-X to `superseded`, writes
  `[[relation]] label="supersedes" target="ADR-X"` on ADR-Y and typed
  `superseded_by = ["ADR-Y"]` on ADR-X.
- Every governance entity round-trips `show`/`show --json` byte-identical across
  the migration.
- RELATION_RULES exact-coverage invariant updated and green.
- Existing suites green; `just gate` clean.

## Summary

Two small gaps + one deferred verb. IMP-082 is one table row; IMP-064 is a
migration + the transactional verb that makes the migrated slot live. Together
they close the last open items on the existing-entity relation surface before
knowledge records (SL-096) extend it.

## Follow-Ups

- **IMP-063** — supersession for POL/STD/slice (grow `superseded` vocab)
- **IMP-051** — knowledge-record cross-kind supersession (SPEC-019 FR-006)
- **IMP-032** — supersession `validate` cross-check (ADR-010 D4; may ship here or
  as a fast-follow)
