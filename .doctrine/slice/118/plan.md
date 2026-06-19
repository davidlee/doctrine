# Implementation Plan SL-118: Estimate facet authoring CLI verb

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases, dependency-ordered: complete the shared validation rule, build the
pure write leaf, then wire the CLI on top. The split keeps each phase under a
single concern and a single gate — the pure-layer change (PHASE-01) carries the
behaviour-preservation gate; the leaf (PHASE-02) is fully pure and golden-tested
in isolation; the command tier (PHASE-03) is the only phase touching `main.rs`.

## Sequencing & Rationale

**PHASE-01 first — validation completeness.** The codex pass (design §5) found the
CLI would accept `inf`/`nan` that the parse path drops: `estimate::validate` lacks
finiteness, and `value` has no standalone `validate`. The command tier can only
"reject exactly what parse rejects" if `validate` *is* the complete rule. This is a
small, foundational change to the shared `estimate`/`value` modules, isolated up
front so the behaviour-preservation gate (existing suites green unchanged) is a
clean checkpoint before anything builds on it. PHASE-03's parity verification
(VT-1) depends on this landing first.

**PHASE-02 next — the leaf, in isolation.** `src/facet_write.rs` is a new ADR-001
leaf with no engine/command deps, so it can be built and fully verified (VT-1..8)
without the CLI. Isolating it here lets the golden round-trip, malformed-present
fail-loud, and forward-compat tests stand on their own, and keeps the layering
test (VT-8 / design VT-13) honest — the leaf must not reach for `estimate`/`value`
(validation stays command-side). The D1 reversal (no `updated` bump) keeps this
phase clock-free and pure.

**PHASE-03 last — wire the verbs.** Depends on both: the complete `validate`
(PHASE-01) and the write leaf (PHASE-02). All `main.rs` surface lands here — the
subcommand groups, the `exact: Option<f64>` mode machine, the path-from-ref helper
factored out of `resolve_link_path`, and the Write/Read classification entry. The
acceptance proof is split deliberately (design §6/F6): the catalog round-trip
(VT-4) proves the kind-agnostic graph/map read; the slice typed-reader round-trip
(VT-5) proves the per-entity surface that catalog scan does not exercise. Dogfood
(VH-1) closes the phase.

## Notes

- **No queried data here** — phase criteria/verification live in `plan.toml`,
  runtime progress under `.doctrine/state/`.
- **History (IDE-013)** is out of scope; PHASE-02 VT-7 only pins that the writer is
  forward-compatible (unknown sub-keys survive), not that history is written.
- **value::validate wiring (PHASE-01 EX-2)** must go *into* `value::normalise`, not
  beside it — a standalone CLI-only validate would be a parallel rule (design F5).
