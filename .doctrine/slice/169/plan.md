# Implementation Plan SL-169: columns & tags read-surface wiring

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five phases over the read surface of the shared entity engine. No new verbs, no
storage, no spec amendment (SL-169 scope). The work splits cleanly along two
axes the design surfaced: (1) the conditional tags-default rule must live in one
place before any kind consumes it; (2) the per-kind column sites differ in
whether their row type already carries `tags` — a real risk axis, not cosmetic.

PHASE-01 lays the shared `listing::default_with_tags` helper and refactors
backlog onto it. PHASE-02/03/04 consume the helper to wire tags columns across
the 7 actual column sites (10 kinds). PHASE-05 closes the two non-tags SPEC-013
gaps (relation `--columns`, concept-map header casing) that share no code with
the tags work.

## Sequencing & Rationale

**PHASE-01 is the foundation gate.** `backlog.rs` already encodes the splice
inline; every other kind would otherwise copy it. Lifting it into
`listing::default_with_tags` first means PHASE-02/03/04 each collapse to two
lines, and backlog's untouched goldens become the behaviour-equivalence proof
that the helper is faithful. Nothing else may start until this lands.

**PHASE-02 before PHASE-03 — split by row readiness.** The design verified that
`GovRow`, `KnowledgeRecord`, and revision's `ListRow` already carry
`tags: Vec<String>`, while `SliceRowTuple` and `SpecListRow` do not. PHASE-02
takes the tags-ready sites first (lowest risk: column entry + helper call only)
and, crucially, lands the **governance** edit that covers four kinds
(adr/policy/standard/rfc) through `governance::run_list` in a single site —
`rfc.rs`/`adr.rs`/`policy.rs`/`standard.rs` are pure delegators and stay
untouched. PHASE-03 then does the row-field plumbing for slice and spec, where
the change is deeper (struct field + construction site) and the regression
surface larger.

**PHASE-04 isolates REC + review.** These combine three concerns — opening the
`TAGGABLE` gate (`tag.rs`), row plumbing on two `ListRow` types, and the
`show`/`--json` surfaces — and complete IMP-144's deferred read work. Keeping
them in one phase keeps the gate flip and the read wiring atomic: a kind should
never become taggable without its display surface following in the same change.
Review's list dispatch is special (derived status/await) but still routes through
`select_columns`, so the helper threads identically.

**PHASE-05 is the independent tail.** Relation `--columns` (D1) and concept-map
header casing (D5) touch disjoint files and have no tags dependency. They could
run at any point after the design lock; sequenced last only so the tags spine —
the bulk of the slice and the part with shared machinery — settles first.

**Dispatch note (not a sequencing constraint):** after PHASE-01, the file sets
of PHASE-02, PHASE-03, PHASE-04, and PHASE-05 are mutually disjoint, so they are
parallelizable under `/dispatch`. The serial order above is the safe default.

## Notes

- **Behaviour-preservation is the spine of verification.** Every phase that
  touches a default column set carries an untagged-corpus golden whose output
  must be byte-identical to pre-change — the conditional gate is what makes the
  change safe for existing fixtures (design.md Verification).
- **No renumbering.** PHASE/criterion ids are immutable; if a phase splits or a
  criterion is wrong, append — never renumber.
- **Test net to extend:** `tests/e2e_list_columns_golden.rs` (column model +
  goldens) and `tests/e2e_list_conformance.rs` (parse-conformance matrix) are
  the conformance guards; both grow in the phases that add columns and in
  PHASE-05 for relation/census.
