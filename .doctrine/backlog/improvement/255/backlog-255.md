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

## Progress (2026-07-13, investigation)

`doctrine check` added and wired as the intended full-suite/real-lint gate — so
the *mechanism* for both failures (narrowed suite + vacuous `check quick`) now
exists. Investigation of recent case notes (SL-210, SL-213, SL-217 drives,
2026-07-10 through 2026-07-13) confirms: **not yet adopted; the failure pattern
persists in new forms.**

### What's improved

- **`--bin doctrine` (narrowed suite) — mostly retired.** Recent workers run
  `cargo test` (full suite, not `--bin doctrine`) or rely on the server-side
  `worker_commit` gate (`doctrine check commit`). The narrowed-suite flavour
  of IMP-255 is far less common.
- **`check quick` as vacuous no-op — architecturally fixed.** `doctrine check
  quick` now explicitly no-ops when unconfigured (exit 0), rather than being
  undefined. `check commit`/`check gate` default to `just check`/`just gate`
  and work in the right context.

### What's still broken

1. **`cargo test` green ≠ clippy green.** SL-217-P02-finish: an execute agent
   ran `cargo test` only (15 tests green), called the phase done, then
   `doctrine check gate` surfaced **6 clippy errors** (integer_division,
   too_many_arguments, trivially_copy_pass_by_ref, map_or, doc-backticks).
   The handover paid for a full read+fix+re-gate loop the original agent
   should have caught. The IMP-255 "vacuous self-check" failure mode lives
   on — now through `cargo test` instead of `check quick`.

2. **`doctrine check commit` in-jail = false red.** SL-213-drive-3:
   worker_commit's gate runs `doctrine check commit` inside a marked fork;
   3 e2e ADR golden tests (`e2e_adr_cli_golden.rs`) shell the doctrine CLI
   and hit the worker-confinement write refusal. Environmental red, disjoint
   from the worker's delta. The gate runs but is unreliable inside a marked
   fork, so workers fall back to `cargo gate + server-side gate` and skip
   the full gate themselves.

3. **`doctrine check gate` in-jail = eslint noise.** SL-213-close-3: the
   eslint leg fails on every run (`web/map/node_modules/.bin/eslint:
   /usr/bin/env: bad interpreter`). Recurring noise for any Rust-only slice
   with no JS surface; forces manual adjudication on every close.

4. **worker_commit's `check commit` runs via stale `$PATH` binary.**
   Recurring (SL-210, SL-206-P13, SL-206-P14, SL-213). The gate shells
   `doctrine` from `~/.cargo/bin` — read-only, lags the coord build — so
   any phase that changes an allowlist/conformance rule false-reds. Cost:
   ~6 orchestrator tool-calls + fallback to live-worktree import per
   occurrence. Structural fix: worker_commit's gate should run its check
   via the server's in-process binary, not `$PATH`.

### Assessment

Both IMP-255 root causes still surface, just through different channels:

| Original failure | Status | New channel |
|---|---|---|
| Narrowed suite (`--bin doctrine`) | Mechanism exists, but | `cargo test`-only misses clippy (SL-217-P02) |
| Vacuous self-check (`check quick` no-op) | Mechanism exists, but | `check commit` false-reds in jail (SL-213-drive-3, SL-210) |

**Gate for closure:**
- (a) worker_commit routes its `check commit` gate through the server's
  in-process binary, not `$PATH` — removes the recurring stale-path false-red.
- (b) Worker prompt template consistently mandates `doctrine check commit`
  (not just `cargo test`) as the worker-side self-check, with jail-aware
  guidance that `check gate` includes ESLint noise.
- (c) e2e golden tests that shell the doctrine CLI are suppressed or
  marker-aware inside marked forks, so in-jail `check commit` runs green.

## Related

- RFC-011 case-notes: 8+ occurrences (pre-`check`), plus SL-210, SL-213, SL-217
  entries (post-`check`) showing the shifted failure patterns
- IMP-194 (diff-aware funnel verify), IMP-195 (inter-phase golden regression),
  IMP-208 (regression baseline carry-forward) — funnel-side verification
  improvements that complement, not replace, the worker-side mandate
- IMP-214 (make resolve_runner testable)
- IMP-209 (VT mandate structure — the sibling verify-vt false-positive problem)
- IMP-220/ISS-220 (worker_commit stale-PATH false-red)
- RFC-011 case-notes entries: `[dispatch; SL-210-drive]`, `[dispatch;
  SL-213-drive-3]`, `[close; SL-213-close-3]`, `[execute; SL-217-P02-finish]`
