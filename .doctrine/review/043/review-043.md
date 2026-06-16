# Review RV-043 — reconciliation of SL-078

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**SL-078** is a single-phase chore with two file-disjoint items:

- **CHR-006**: SPEC-010 rename sweep — correct all stale `skills install`, `skills
  list`, and `doctrine skills` references left behind after SL-056 PHASE-11
  renamed the CLI surface to `claude`. Affects `spec-010.md`, `spec-010.toml`,
  `requirement-177.toml`, `requirement-177.md`. Zero `.rs` production code impact.

- **CHR-008**: Add a unit test (`supersede_recovery_from_torn_new_only_state`) for
  the torn-state supersede recovery scenario. `run_supersede` writes NEW then OLD;
  a crash between writes leaves a detectable torn state. Re-running the same
  command naturally completes recovery through the existing flow — the test
  proves this without a dedicated recovery code path. Zero production code changes.

**Lines of attack:**

1. Verify the spec-010 sweep is complete — no stale skill references survive in
   any authored tier file.
2. Verify REQ-177 title/slug/H1 correctness.
3. Verify the recovery test exercises the torn-state scenario and passes.
4. Verify the gate (`just check`) is green — clippy zero, all tests pass.
5. Verify no production code was changed — the invariants of "spec-only + test-only".

## Synthesis

SL-078 was a clean, low-risk chore with no surprises. Both items were bounded and
disjoint — spec-only text edits (CHR-006) and a test-only addition (CHR-008).

**CHR-006** required more edits than the design map specified: two additional
`skills install` references (in the gitignore self-enforcement section and
Decision D4) were found and corrected during execution. The design's edit map
had approximate line numbers and one duplicate entry (L31-32 and L115-118 both
pointed to the same L36 reference). The verification step (`doctrine spec show
SPEC-010`) caught the completeness gap, and the additional edits were applied
before declaring done.

**CHR-008** exercised the torn-state recovery path without a dedicated recovery
code path — `run_supersede`'s existing flow handles it naturally. The test uses
`catalog::test_helpers::tmp()` and `write()` for fixtures, staying within the
test harness without new infrastructure. The key invariant proven: `push_str_if_absent`
is idempotent, so re-running supersede on the torn state completes OLD's
superseded_by entry and status flip without duplicating NEW's supersedes entry.

**Standing risks**: None. Both items are non-functional — text-only and
test-only — with zero production code impact.

**Tradeoffs**: None consciously accepted. The implementation follows the design
as written with minor line-number corrections during execution.

**Gate**: `just check` clean — 1548 tests pass, clippy zero, fmt clean.
