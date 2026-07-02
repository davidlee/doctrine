# Implementation Plan SL-189: Pi-arm boundary recording scopes to imported code commit, not funnel span

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases: **PHASE-01** builds the mechanism and proves it in Rust; **PHASE-02**
wires the pi dispatch skill to use it. The fix is deliberately small — one engine
helper reusing existing git plumbing, one new CLI mode, two skill edits — bringing
the pi arm to parity with the already-tight claude arm (design §2, §5). No claude
writer, solo-path, or conformance-reader changes.

## Sequencing & Rationale

**Why code before prose (PHASE-01 → PHASE-02).** The skill edit references
`record-delta --commit <S>`; that flag must exist and be green first, or the skill
documents a command that does not parse. PHASE-02's entrance criterion is
PHASE-01 landing.

**Why helper and CLI in one phase (PHASE-01), not split.** `single_commit_boundary`
exists solely to serve `--commit`; splitting them leaves a phase whose only artefact
is an unused function (dead-code lint, no meaningful green). Building helper +
consumer + their tests together is one cohesive red/green/refactor unit.

**TDD shape within PHASE-01.** Red: helper unit test (VT-1) against a git fixture —
derive `[S^, S]`, reject a merge, reject a root commit. Green: the helper. Red: the
SL-186 behavioural regression (VT-2) — a fixture with a refresh-base merge in `S^`
and a knowledge commit after `S`, asserting conformance sees only `S`'s own paths.
Green: wire `--commit` through `run_record_delta`. Red: arg diagnostics (VT-3) —
the clap contract. Green: the ArgGroup. Refactor. Behaviour-preservation (EX-4): the
existing `tests/e2e_slice_record_delta.rs` range coverage and the conformance suite
stay green unchanged — they are the proof the legacy path and shared machinery did
not regress (AGENTS.md behaviour-preservation gate).

**Verification modes.** PHASE-01 is fully testable (VT). PHASE-02 is prose — no test
can judge whether the skill instructs the orchestrator correctly, so it carries a VA
(agent read-back of both edited skills against the design), not a hollow VT.

## Notes

- The helper lives in the engine layer (`src/state.rs`) so both `run_record_delta`
  (now) and future adopters — `run_record_boundary` (claude), `capture_phase_boundary`
  (solo / IMP-175) — import *down* (ADR-001). This slice wires only the pi caller.
- Known deferred gap (design A2 / R4, Codex F-3): a phase split across multiple
  commits (mid-phase re-dispatch / reopen) is under-captured by a single `--commit`
  because `record_source_delta` upserts by phase. Pre-existing (the range path is
  equally last-write-wins), out of scope; the `--start/--end` escape hatch remains.
- Parallel-batch rows are batch-scoped, not per-phase (all phases in one batch share
  `S`); slice-level conformance unions rows, so it is unaffected (design §2, F-2).
