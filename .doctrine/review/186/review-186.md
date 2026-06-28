# Review RV-186 — reconciliation of SL-173

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Audit surface:** `refs/heads/review/173` (candidate interaction branch, dispatched
to `dispatch/SL-173`). Single-phase slice; one file touched (`src/backlog.rs`).

**Lines of attack:**
1. **Conformance** — does the delta match `design-target` (src/backlog.rs)? Conformance
   report: 0 undeclared, 0 undelivered, 1 conformant → clean.
2. **Design fidelity** — do the CLI flags, filter logic, `norm_ref` helper, and seam
   placement match `design.md` exactly? D1 (normalized match), predicate (OR within
   flag, AND across axes), retain placement (after `--kind`, before `any_tagged`).
3. **Test coverage** — do the unit tests cover all VT-1 through VT-6 criteria from
   `plan.toml`? Check for explicit VT-5 (`--by sequence` with filters) and VT-6
   (`--json`/`--columns` with filters) tests.
4. **Behaviour-preservation gate** — 2728 existing tests pass; clippy clean; no
   changes outside `src/backlog.rs`.

## Synthesis

SL-173 adds repeatable `--after <REF>` and `--needs <REF>` edge filters to
`doctrine backlog list`. Single phase, single file (`src/backlog.rs`), TDD
driven. The implementation is complete and correct:

- **Design fidelity**: all design decisions are implemented. D1 normalized match
  (`norm_ref` using cross-kind `parse_canonical_ref`, verbatim fallback).
  Predicate correctly implements OR within a flag, AND across axes. Retain
  placement is after `--kind`, before `any_tagged` (preserving dynamic tags-column
  visibility).
- **Conformance**: clean — 0 undeclared, 0 undelivered, 1 conformant path.
- **Exit criteria**: all five (EX-1 through EX-5) met. CLI flags wired, `norm_ref`
  pure, retains placed correctly, hide-set/ordering/JSON/columns unchanged,
  clippy clean.
- **Verification**: VT-1 through VT-4 explicitly tested (8 new unit tests). VT-5
  and VT-6 are preserved by the unchanged ordering/rendering code paths (no
  dedicated filtered-set tests, but existing suites pass — tolerated, F-1, F-2).
- **Behaviour-preservation**: 2728 tests pass, 0 failures, 0 warnings.

**Standing risks**: none. The change is additive — no existing API surface
modified, no shared-axis changes, no I/O changes. The edge filters are pure over
the in-memory `BacklogItem.relationships`.

**Tradeoffs accepted**: VT-5/VT-6 explicit filtered-set tests deferred to
nice-to-have (F-1, F-2). The code paths they would exercise are unchanged and
existing tests pass.

## Reconciliation Brief

No spec or governance changes required. No per-slice design edits needed.
All design decisions are reflected in implementation. Both findings (F-1, F-2)
are tolerated with rationale — no remediation required.

## Reconciliation Outcome

All findings were withdrawn or tolerated with rationale. No writes needed.
Reconcile pass complete — handoff to /close.
