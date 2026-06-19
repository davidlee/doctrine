# Implementation Plan SL-026: lazyspec read-only projection

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

One deliverable — a locked JSON wire format plus its producer — sequenced into
four phases along a single seam: **pure first, impure last, fixtures in the
middle**. The mapping (the contract this slice owns) is pure over a `Corpus`
data struct, so it is built and unit-tested with zero I/O before any loader or
command exists. The impure shell (loaders, CLI, serialization) and the drift
canary (golden + conformance) land last, once the fixture infra they consume is
in place. Design reference: `design.md` §5; charges sealed in RV-093.

## Sequencing & Rationale

**Why pure-core first (PHASE-01).** `project` is the heart — status map, edge
map, plan-rollup rule, ordering, inline body assembly (§5.3). Because it is pure
over `Corpus`, every mapping invariant (INV-1/2/4/5/6/7, totality, the
`PhaseRollup::total()`/`anomalies()` plan rule) is testable from hand-built
in-memory values, with no fixtures and no disk. Landing it first pins the wire
schema and de-risks the mapping before the mechanical plumbing. The structs are
unused until PHASE-04 wires the command, so a self-clearing module-level
`#![expect(dead_code, …)]` (R3) holds the gate green.

**Why the fixture seam is its own phase (PHASE-02).** The golden corpus must be
built through the real loaders over a temp tree, and the test-support seam is
mostly already there (`catalog::test_helpers`: `seed_slice`/`seed_adr`/
`seed_requirement`/`relation_rows`). The remaining gaps — promoting SL-027's
`write_fixture` to `pub(crate)` (R7, no re-triplicated backlog TOML), a
`seed_spec`, and `seed_adr`'s generic-edge gap (RV-093 F-2) — are a
behaviour-preservation-gated edit (existing backlog + spec suites must stay green
unchanged). Isolating them keeps that gate clean and gives PHASE-03/04 a stable
fixture vocabulary to build on. Promoting `write_fixture` is a `/consult`-grade
visibility move (§9) — do not improvise its final home at execute.

**Why the shell is split from the command (PHASE-03 → PHASE-04).** PHASE-03
widens `spec::render`/`read_members`/`read_interactions` to `pub(crate)` (a
second behaviour-preservation-gated edit on `spec.rs`) and composes the existing
readers + `relation::tier1_edges` into `load_corpus` — pure reader composition,
no new read logic, no per-kind edge reach-in (D7). It ends green with a loader
test over a PHASE-02-seeded tree. PHASE-04 then wires the `export lazyspec`
command, clears the dead_code expect, and lands the **drift canary**: the
golden file (R1) plus table-driven conformance over kinds *and* fields (R2 — the
SL-025 audit miss: envelope-parity ≠ surface-parity). Keeping the loader's
behaviour-preservation edit out of the command/conformance phase keeps each
green point focused.

**Linear, not parallel.** The dependency chain is strict
(01 → 02 → {03, 04}; 04 needs 03), so phases run serially. PHASE-02 touches test
support (`backlog.rs`, `catalog::test_helpers`, `spec.rs` test seam); PHASE-01
and the new `src/lazyspec.rs` are disjoint from it, but PHASE-02's entrance
wants PHASE-01's `Corpus` shape settled, so order over overlap.

## Notes

- **OQ-4 resolved (this plan):** v1 bodies are raw prose-tier `.md` for
  slice/adr/backlog; specs keep both-tier output via `render()` (already). The
  lossy-by-design v1 ethos (§4) favours the simplest faithful body; structured
  TOML (acceptance_criteria/c4_level/risk facet) is dropped from the viewer and
  recoverable in a later slice if the viewer needs it. Flagged for pushback.
- **Out of scope (unchanged):** piece-4 (the `../lazyspec` fork) and the
  deferred post-scope node kinds (IMP-105). Dangling outbound edges to deferred
  kinds are accepted under `validate_ignore` (§5.5, D8).
- The plan is not higher authority than `design.md` or `/canon`: if execution
  surfaces a substantive design problem, re-enter `/design`, don't improvise.
