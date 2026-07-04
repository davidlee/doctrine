# IMP-255: Dispatch worker must run full cargo test, not --bin doctrine

## Problem

Dispatch workers consistently run `cargo test --bin doctrine` (or `check quick`,
which is an unconfigured no-op), missing integration test binaries under
`tests/`. The worker self-reports green, but the funnel verify beat catches
real regressions that the worker's narrowed suite never ran.

The funnel doctrine is "self-reports are advisory; trust only the verify beat"
— which is correct, but each miss burns a full worker run (~15–85k tokens) that
is structurally blind to integration-binary regressions.

Witnessed 8+ times in the RFC-011 case notes:
- SL-191-P02: worker ran `--bin doctrine install`, missed e2e golden in `tests/`
  → full re-dispatch
- SL-191-P05: worker attributed e2e reds to "ambient marker," one was real
  gate-induced regression → caught one funnel round late
- SL-191-P06: `--bin doctrine` missed sync golden in `tests/`
- SL-176-P03: worker reported `check quick` green — it's an unconfigured no-op;
  7 real clippy errors landed silently
- SL-176-P03: worker's `verify-vt` gate falsely passed (keywords matched prose,
  not tests)
- SL-196: worker inspected primary-tree src/ instead of worktree copy

Two distinct failures:
1. **Narrowed suite** — `--bin doctrine` doesn't run `tests/` integration
   binaries, leaving golden/behaviour regressions invisible to the worker.
2. **Vacuous self-check** — `check quick` is an unconfigured no-op by default;
   the real lint check (`check gate` / clippy) is never run.

## Fix direction

- **Worker prompt template**: mandate `cargo test` (full suite) and
  `doctrine check gate` (real lint), not `--bin doctrine` / `check quick`.
- **Verify that the worker ran the right commands**: the funnel's `check
  regression` beat already catches the gaps, but should also assert the
  worker's self-reported gate command is the real one.
- **`just test-full` recipe**: a single command that always runs the full suite
  regardless of workspace shape, so workers don't have to guess.
- Consider a funnel pre-spawn check that warns if the worker prompt scopes to
  `--bin doctrine` with `tests/` files in the declared scope.

## Related

- RFC-011 case-notes: 8+ occurrences
- IMP-194 (diff-aware funnel verify), IMP-195 (inter-phase golden regression),
  IMP-208 (regression baseline carry-forward) — funnel-side verification
  improvements that complement, not replace, the worker-side mandate
- IMP-214 (make resolve_runner testable)
- IMP-209 (VT mandate structure — the sibling verify-vt false-positive problem)
