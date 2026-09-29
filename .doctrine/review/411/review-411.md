# Review RV-411 — code-review of CHR-172

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Sample-based test quality audit (CHR-172). Depth over breadth: a stratified
sample of test modules, each read in full.

### Baseline (measured 2026-09-29, `cargo test -p doctrine`, libtest `--report-time`)

- ~9,039 test executions, 2m04s wall (mostly compile); 166 s summed test time.
- Only 11 tests exceed 1 s. Outliers: `e2e_show_equivalence`
  `show_matches_the_kind_verb_in_table` 19.3 s; `e2e_codex_install` 6–10.6 s
  each (×4); `e2e_claude_install` `design_prompts_have_no_consumer_outside_the_design_run`
  7.3 s; `e2e_doctor_golden` ~1.8–2.3 s (×3).
- 440 test names execute more than once: `#[path]`-included modules carry
  their `#[cfg(test)]` suites into every including binary
  (`common::test_support::tests` ×106, `design_run::tests` ×11).

### Sample (stratum → unit)

| stratum | unit |
|---|---|
| leaf / pure | `src/priority/graph.rs` (70; ISS-008 flake report) |
| engine, largest | `src/memory.rs` (249) |
| engine, ledger | `src/review/tests.rs` (133) |
| impure shell (real git) | `src/git.rs` (130) |
| rendering / governance text | `src/boot.rs` (195) |
| CLI integration | `tests/e2e_design_state.rs` (64 + includes) |
| golden | `tests/e2e_review_golden.rs` (48) |
| measured-slow | `e2e_show_equivalence`, `e2e_codex_install`, `e2e_claude_install`, `e2e_doctor_golden` |
| structural guard | `tests/architecture_layering.rs` |
| helpers | `tests/common/mod.rs`, `src/test_support.rs` |
| separate crate | `crates/doctrine-control/src/conformance.rs` (186) |

### Lines of attack

- **Brittle** — asserts on incidental text/ordering/formatting/private shape
  beyond the behaviour claimed.
- **Vacuous** — can't fail, asserts on setup, tautologies, `is_ok()` only,
  over-mocked seams, test that re-derives the expected value from the code.
- **Slow** — needless process spawn, real git/disk where a pure seam exists,
  repeated heavy fixtures, duplicate execution.
- **Intent-opaque** — name/body doesn't say what behaviour is pinned; setup
  noise drowning the assertion; archaeology comments (slice/RV ids) instead of
  a behaviour statement.
- **Parallel implementation** — hand-rolled fixtures where a helper exists.
- **Non-hermetic** — reads the live corpus, env, clock, or host paths.

## Synthesis

**Overall:** acceptable — with a vacuity problem worth fixing now.

**Synopsis.** The suite is fast (only 11 of ~9,000 tests exceed 1 s) and mostly
behaviour-focused. The best areas — the refusal tests in review/tests.rs, the
merge-planner half of boot.rs, the ambient-surface tests in memory.rs, and the
capsule verdict algebra — are rigorous. They pin exact refusal variants, check
that ledger bytes are unchanged after a refusal, and include positive controls.
Where the suite is weak, it is weak in the same four ways across modules:

1. **Tests that have stopped testing.** Their fixtures drifted away from the
   code, or they never called it: F-1, F-9, F-10, F-11, F-12, F-7, F-15, F-19.
   Two of these hid real production defects: draft expiry (ISS-500) and the
   sha1-only zero OID, the object id used for compare-and-swap ref updates
   (ISS-501).
2. **Non-hermetic fixtures.** They inherit the host's git config and hooks,
   read `DOCTRINE_*` env, walk the live corpus, or run `npx` over the network
   (F-2, F-6, F-7, F-8, F-13, F-20). Because fixtures are copied rather than
   shared (F-17), none of this can be fixed in one place.
3. **Structural guard gaps.** The ADR-001 layering gate misses macro-embedded
   and sub-tier edges (F-4). The capsule suite is red on the owner's host and
   no gate runs it (F-13).
4. **Cost and noise.** `#[path]` includes compile and run suites repeatedly
   (F-5). Goldens are copied in full (F-14). Near-duplicate tests should be
   table-driven (F-18). Test names record history, not behaviour (F-21).

Leverage order: F-2 plus F-17 together (one hermetic fixture layer), then the
vacuity cluster, then F-4. Most slow-test cost goes away with F-6 and F-8.

Not raised: ISS-008 (a reported flake in `priority::graph`) looks stale. The
test was renamed, and 90 stressed runs showed no failure. Recommend closing it.
One reviewer saw `rendered_artifact_matches_the_embedded_bytes` fail; it passed
12/12 in the baseline run, so no finding.

**Haiku:**

> green bar, empty loop —
> the draft expires in the past
> that never arrives
