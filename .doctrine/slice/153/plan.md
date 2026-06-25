# Implementation Plan SL-153: CLI verbs for spec-internal edges (descends_from, parent, interactions)

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Close the last hand-edit-only relation edges in the corpus by adding three
`doctrine spec` verbs: `spec edit` (scalar `descends_from` / `parent`) and `spec
interactions add|remove` (the `interactions.toml` `[[edge]]` AoT). The design
(`design.md`) is locked through an internal pass (A1–A6) and an external codex
inquisition (§10 E1–E4); the plan carries those dispositions into phase criteria —
notably the parent acyclicity gate (E1), the new `remove_interaction_edges` seam
(E2, not `dep_seq::remove_after`), canonical dup-matching against on-disk rows (E3),
and reuse of `relation::lookup`/`check_target_kind` over an inline kind table (E4).

## Sequencing & Rationale

Bottom-up by seam — leaf core first, then the command shells that call down, then
distribution. Each phase ends green; the existing `spec`/`relation`/`dep_seq` suites
are the behaviour-preservation proof and stay green unchanged throughout.

- **PHASE-01 — `apply_scalar` leaf core.** The load-bearing new seam (design §9).
  Built and unit-tested in isolation at the `dep_seq` leaf before any shell depends
  on it. The risk concentrates here: edit-preserving root insert above a trailing
  `[[relation]]` block (CHR-019 worst-case), and a coherent no-op guard (equal /
  absent → no mutation). Isolating it keeps the contract — *create* an absent key,
  distinct from `apply_status`' *refuse* — provable on its own.

- **PHASE-02 — `spec edit`.** Rides PHASE-01 for the write; the phase's own weight
  is *validation before write*: tech-only `descends_from`, subtype-aware `parent`,
  existence + kind via the declared `RELATION_RULES` rows, and the pre-write
  acyclicity gate (E1) that stops a corpus state `spec validate` would later call
  HARD-invalid. Batched multi-field write-once and canonical normalization land here
  too. Comes before interactions because it exercises `apply_scalar` end-to-end.

- **PHASE-03 — `spec interactions add|remove`.** Separate surface
  (`interactions.toml`, `[[edge]]` AoT), so a separate phase. `add` rides
  `append_member`; `remove` needs the new pure `remove_interaction_edges` helper —
  the design's E2 finding is that `dep_seq::remove_after` cannot serve (wrong table /
  key / F-1 bail). Target-as-PK with canonical dup-matching against existing on-disk
  rows (E3) is the subtle correctness point.

- **PHASE-04 — shipped-memory refresh + close-out.** Documentation/distribution
  tail: the signpost memory `mem.signpost.doctrine.relating-entities` still lists
  these three as hand-edit-only; refresh it and re-embed via the shipped-memory flow.
  Final full-suite + `just gate` gate readies the slice for `/audit`.

## Notes

- **File-disjointness (dispatch).** PHASE-01 touches `src/dep_seq.rs`; PHASE-02 and
  PHASE-03 both touch `src/spec.rs` (serial — same file); PHASE-04 touches
  `memory/…` + `src/corpus.rs`. Default serial execution; PHASE-02/03 must not be
  parallelised against each other.
- **Follow-up out of scope (R2 / IMP-170).** Product `parent` (PRD→PRD) is authored
  here via an inline branch but stays undeclared in `RELATION_RULES`; the table
  honesty + PRD-parent row + VT-1 golden are the captured follow-up, not this slice.
- **Behaviour-preservation gate.** The shared machinery (`dep_seq`, the entity
  engine) keeps its existing suites green unchanged — that is the proof, per the
  project gate.
