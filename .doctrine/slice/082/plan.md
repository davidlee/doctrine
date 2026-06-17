# Implementation Plan SL-082: Dispose of `doc/*`

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Five phases. PHASE-01 through PHASE-04 are file-disjoint across four independent
surfaces (source code, skills, install templates, memory records) and can run
concurrently. PHASE-05 must be last — it removes the `doc/` directory and runs
the final verification sweep.

## Sequencing & Rationale

**PHASE-01 ⋮ PHASE-04 → PHASE-05**

### PHASE-01 through PHASE-04: File-disjoint, parallel-capable

Each phase touches a different directory tree:

| Phase | Surface | Directory |
|---|---|---|
| PHASE-01 | Source code | `src/` |
| PHASE-02 | Skills | `plugins/doctrine/skills/` |
| PHASE-03 | Install templates | `install/` |
| PHASE-04 | Memory records | `.doctrine/memory/items/` |

No file is touched by more than one phase. No phase's output depends on
another's output. They can run in parallel if dispatched, or serially for
simplicity (four phases, each narrow in scope).

### PHASE-05: Remove and verify

The `doc/` directory can only be deleted after every reference to it is
repointed or removed. PHASE-05 gates on PHASE-01..04 completion and runs the
final `rg` sweep + `just check` + `doctrine claude install` gate. This is the
only phase that mutates the filesystem (deletion); all others are reference
edits.

## Phase detail

### PHASE-01: Source code

Six files, 10 citations. The majority are test fixtures and doc comments.
Changes are mechanical — path string updates — with one exception: S10
(`src/spec.rs:120`) adds a forward reference to SPEC-004. Risk: `src/coverage.rs`
tests may assert string equality on `MatchSource::File` values; read assertions
before editing, adjust expected values inline.

### PHASE-02: Skills

Nine files (8 SKILL.md + 1 SKILL.compact.md), 11 citations. The highest-touch
phase: every reference is prose that shapes how agents think about doctrine's
authoritative surface. Three patterns deleted:

- "evergreen specs under `doc/*`" → `.doctrine/spec/tech/` + `.doctrine/adr/`
- "author under `doc/*`" → `doctrine spec new` / `doctrine adr new` verbs
- "see `doc/*`" → see the tech spec corpus

After edits, `doctrine claude install` must run to propagate source changes
to the installed skill tree.

### PHASE-03: Install templates

Two files, three citations. Lightest phase: remove the `doc/*` glossary section,
update one governance pointer line, delete one stale HTML comment.

### PHASE-04: Memory records

Six records, 3 `memory.toml` `paths` edits + 3 `memory.md` prose edits. Scope
anchors only — no semantics change, no verification status reset. The memory
engine's `paths` field is about what the memory is *about*, not a live
dependency.

### PHASE-05: Remove and verify

`git rm -r doc/`, then the full gate: `rg` sweep, `just check`, `doctrine
claude install`. All changes from PHASE-01..05 committed in one or more
conventional commits.

## Notes

- **SL-084 cross-check:** CLAUDE.md and AGENTS.md confirmed zero `doc/`
  references. No changes needed.
- **`retrieve-memory/SKILL.md`:** `file/doc/ADR` is a generic enumeration, not
  a `doc/` path. No change needed.
- **Post-SL-021 drift:** Commit `8f0e49f` (SL-066) updated `doc/entity-model.md`
  and `doc/spec-entity-spec.md` to reflect ADR-013. Editorial resolution of
  existing open questions — no net-new architecture to rehome. Content already
  captured in ADR-013.
- **SL-021 status:** 19 tech specs authored (`draft`); content rehomed.
  SL-021's TOML still says `proposed` but the rehoming is functionally complete
  — this slice's hard prerequisite is satisfied.
- **Commit granularity:** PHASE-01..04 produce file-disjoint edits. Can commit
  per-phase or as one batch. PHASE-05 is a separate commit (directory deletion).
