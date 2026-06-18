# Implementation Plan SL-097: Build IMP-006 transactional supersede verb for knowledge records and wire RECORD Supersedes LifecycleOnly rule row

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases that ascend from stable foundation to behaviour change, each with
an independent green checkpoint.

## Sequencing & Rationale

**PHASE-01** establishes the two things the verb needs that don't exist today:
`RecordKind::is_terminal()` (the D2 conditional flip can't be tested without
it) and seeded `[relationships]` blocks in the four templates (the F-1
pre-flight can't find them without them). These are low-risk, purely additive
changes — no existing code paths are altered. The terminal sets are declared
alongside the status vocab arrays in `src/knowledge.rs`, keeping the D2 table
as a single source of truth. Template updates are mechanical: insert a
`[relationships]` block after `[evidence]` in each of the four `.toml` files
and update the round-trip render test.

**PHASE-02** is a structural move with a behaviour-preservation gate. It
extracts `SupersedePolicy` + `supersede_policy()` from `src/adr.rs` into a new
`src/supersede.rs` leaf module and adds the ASM/DEC/QUE/CON arms (D1). The
ADR arm is mechanically moved — no logic changes, just a crate boundary. The
existing ADR supersede suite is the proof: if it stays green, the extraction
is correct. Simultaneously, the RECORD Supersedes LifecycleOnly row (D4) joins
`RELATION_RULES` between the existing RECORD Spawns and GovernedBy rows.
Even though the verb isn't yet generalized, the rule row is inert until the
verb writes edges — and the exact-coverage invariant tests must be updated to
recognise it. Golden churn is confined to these invariants.

**PHASE-03** is the core change: generalize `run_supersede()` in `src/main.rs`
(D3) with two pure helpers (`is_record_kind`, `validate_matrix`) that delegate
to `knowledge::RecordKind` as the single source of truth. The same-kind guard
is replaced with: if both refs are records → validate against the §6 matrix;
otherwise → existing same-kind guard (ADR). The conditional terminal-status
flip (D2) uses `RecordKind::is_terminal()` from PHASE-01 — only non-terminal
predecessors get flipped. F-D idempotency is extracted to a helper for clarity.
The test plan is 10 scenarios enumerating the happy paths, error paths, and edge
cases from the design's Target Behaviour section. The phase ends with `just gate`
clean across the workspace.

## Notes

- No record migration burden: `.doctrine/knowledge/` is empty (A1).
- No CLI surface change: `doctrine supersede <NEW> <OLD>` is unchanged.
- ADR stays same-kind-only — the record-only cross-kind path is gated by
  `is_record_kind()`.
- The pure/impure split is preserved: matrix validation and terminal checks are
  pure functions in `main.rs` (no IO, clock, or disk); the verb shell handles
  parse/resolve/write.
