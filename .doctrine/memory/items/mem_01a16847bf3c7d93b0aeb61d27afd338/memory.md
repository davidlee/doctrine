Observed SL-285 (2026-10-23).

- `pi_agent` with `profile=write|implement` and `cwd=<repo root>` is refused
  `view-overlaps-protected`: the root contains the server's project profile
  layer `<launch dir>/.quotient/profiles` (checked even when absent). Run
  writable delegates in a linked worktree **below** the root
  (`doctrine worktree fork … --dir .worktrees/<x>`), then land it.
- The write sandbox has **no cargo/rustc and no git** (the worktree's `.git`
  points outside the view). Delegates can edit, not verify. Tell them so in the
  prompt — otherwise they burn the 30-request ceiling probing for a toolchain.
  The orchestrator runs fmt/clippy/tests/gate and sends errors back by
  continuing the same `session` (cheap: a few requests).
- Prebuild the fork (`cargo test --no-run`) before the first delegate: a cold
  `target/` otherwise eats the wall-clock ceiling.
- Calibration: `judgement`/`review` (high/max thinking) gave accurate sheets and
  real defects; `implement` is reliable given exact rulings, and broad tasks hit
  the request ceiling — split runs to ≤ ~6 edits each. Read-only reviewers need
  the diff written to a gitignored file in the tree (no git).
- Negative controls are cheap to run yourself: neutralise the fix with a
  temporary `if false &&`/`if false { … } else { … }` (a plain deletion trips
  `unused`/`dead_code` deny and you get a compile error, not a red test).
