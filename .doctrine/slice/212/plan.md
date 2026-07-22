# Implementation Plan SL-212: Ingest hand-resolved trunk merge

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

The design (`design.md`, locked after three adversarial codex passes; RV-289
findings folded and self-verified) delivers IMP-127: a `candidate ingest` verb
that adopts an operator's hand-resolved `(base, source)` merge as a conflicted
candidate's `merge_oid`, gated by *provenance and content* so an arbitrary tree
cannot reach trunk. The central predicate is pure: `parents(R) == [base, source]`
ordered **and** `diff(R.tree, T_c) ⊆ C` (byte-wise, rename-fold OFF), where `T_c`
/ `C` come from git's own `merge-tree`. Publication stays FF-only and untouched.

The plan decomposes along the design's seams so each phase is independently
green-able and the risky mechanisms land behind their own regression tests.

## Sequencing & Rationale

**PHASE-01 — substrate.** Everything ingest reads or stores, all additive or
behaviour-preserving: the two backward-compatible `CandidateRow` fields, the
atomic `store` (D7), and the byte-safe git read helpers (`MergeTree` conflict
stages, `changed_paths` on `--no-renames`, `merge_base_all`,
`custom_merge_driver_paths`). It rides the existing `diff_doctrine_paths` seam
rather than adding a parallel diff — one byte-safe primitive, thin wrappers
(DRY / ADR-001). Landing first means later phases build on a stable, tested base
and the behaviour-preservation gate (existing suites green unchanged) is proven
early.

**PHASE-02 — the predicate.** The pure `validate_ingest_provenance` is the
soundness heart and has zero git dependency, so it is isolated as its own phase
and exhaustively unit-tested (reversed/single parent, arbitrary-tree `D ⊄ C`,
advisory markers, happy). Pure-first keeps the correctness core out of the
imperative shell (AGENTS.md pure/imperative split) and lets 03/04 consume a
proven judge. It is file-disjoint from 01 and could run in parallel under
dispatch.

**PHASE-03 — create-arm materialisation.** The staging half (D2, the pass-2
reversal): the conflict+`--worktree` arm materialises merge-tree's own output —
`read-tree --reset -u T_c` (the *exact* projection, RV-289 F-2) + unmerged stage
rewrite + `MERGE_HEAD` — so create-staging and ingest-validation are the same
engine by construction, immune to `branch.mergeOptions` (the engine-immunity
regression test is the proof). Ordered *after* 01 because it needs the conflict
stage table and the guard helpers. Only this one arm changes; every other arm
and the admit/integrate suites stay green.

**PHASE-04 — the verb.** `run_candidate_ingest` and its wiring, consuming 01's
reads and 02's judge. Two RV-289 fixes are load-bearing here: the coordination
guard keys on the **candidate path** (not the linked-worktree test, which the
coord tree itself trips — F-1), and the write-once `merge_oid` is enforced by the
fail-closed pre-state select (D5), not a field-level rewrite. This is where the
CLI surface, guard classification, and goldens land.

**PHASE-05 — behaviour surface + governance acceptance.** The user-facing and
doc consequences (status prescription, de-absolutising "Doctrine-created" prose),
the conflict taxonomy and crash/atomicity coverage, and the REV-030/ADR-012
§Verification cases realised as executable checks — the governance payload that
proves the slice did what its ADR amendment promised.

## Notes

- **Two `/plan`→`/phase-plan` pins carried from the design review.** (1) The
  candidate-path guard must *share* the `dispatch.rs:1497`
  `.doctrine/state/dispatch/candidate` constant, not re-inline it (STD-001).
  (2) Pin how the coordination `root` is derived so a candidate cwd cannot yield
  a stale `dispatch_dir(root, slice)` ledger path — the mechanism behind F-1.
- **Projection plumbing (OQ-1)** is settled in principle (`read-tree --reset -u`
  → stage rewrite → `MERGE_HEAD`); PHASE-03 pins the exact worktree-private
  `git rev-parse --git-path` / `GIT_INDEX_FILE` locations. A memory records that
  production git plumbing uses a git-dir `ScratchIndex` for throwaway
  `GIT_INDEX_FILE` (tempfile is dev-only) — reuse that pattern, do not invent.
- **Deferred, not solved (R-4 / IMP-305):** full crash≡resume restage after a
  partial staging is a referenced follow-up; this slice bounds durability to
  atomic-store + a durable row before the worktree.
- VT `test_file` targets name `src/dispatch.rs` for the validator/verb and their
  inline tests; if `/execute` relocates a symbol, append a corrected VT (ids are
  immutable — never renumber).
