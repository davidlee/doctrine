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

Calibration update, SL-285 PHASE-03 (2026-10-23), first use of the new profiles:

- `review` (max): two adversarial passes (sheet, test diff) — every code
  citation spot-checked true; found real vacuity (an "absent on both sides"
  equality, a control that matched the copy leg's row). One finding was wrong
  on the evidence (it assumed claude hooks land in settings.local.json) —
  adjudicate, don't adopt.
- `implement` (high): reliable at ≤ ~4 exact rulings per run; a 6-fix run hit
  the 31-request ceiling mid-edit (edits kept; continue the `session` with
  "finish X, ≤4 requests"). Continuing the sheet author's session for later
  tasks is cheap (cached context) and keeps intent.
- `write` (off): fine for mechanical sheet updates given verbatim text.
- `read` (off): good recon (jail mounts, CLI shapes) in ~6 requests.
- Delegates write green-first verification tests; the orchestrator must still
  run negative controls (neutralise the behaviour, watch red) — delegates
  predicted green and could not tell vacuous green from real green.

Calibration update, SL-286 PHASE-01 (2026-10-24):

- Alternative to a worktree: a writable `cwd` on a subdirectory (`src/`,
  `tests/`) is accepted. The child sees only that subtree, so a change spanning
  `src/` and `tests/` takes one session per subtree. Cheaper than a fork for a
  small phase in the primary tree.
- `judgement` (high) can end its first turn after reading with `result: null`.
  Continue the session with "answer now, ≤3 more tool calls".
- Set `transcript_budget_bytes` low (~1.5–3 KB): the transcript repeats the
  final result, so a large budget doubles the tokens returned.
- `implement` again hit the 30-request ceiling on a 5-task prompt, leaving a
  complete green tree. Compile and check before re-delegating.
- Retrieve this memory before the first delegation, not at harvest.

Calibration update, IMP-522 (2026-10-24) — **supersedes the first two bullets
above** (write-root refusal; no cargo/git):

- Doctrine now has its own quotient forms (`just quotient-forms`, policy
  `~/.config/quotient/doctrine.policy.json`, server launched outside the repo).
  `write`/`implement` with `cwd` = repo root is accepted; no worktree or
  per-subtree session needed.
- The write form has cargo/rustc/mold/cc **and git** (probed: `git status`,
  `git log` work in the primary tree). Delegates can run focused tests
  themselves; a focused `cargo test --bin doctrine <filter>` took 69 s warm.
  Tell them **not to commit or stage** — git is writable there.
- Delegates share the in-tree `target/`: a concurrent orchestrator cargo run
  blocks on the build lock. Don't run cargo while a delegate is building.
- Every call must pass `cwd`: the launch dir is no longer an allowed root.
- Still true: 30-request ceiling, ≤ ~4 exact rulings per `implement` run,
  orchestrator runs negative controls and the gate.

Calibration update, SL-286 PHASE-02 (2026-10-24), first repo-root phase:

- `implement` at repo root ran ~12 runs, each ≤ ~4 rulings, red→green with its
  own focused tests; continuing one session across tasks stayed cheap until
  ~90k context peak — start a fresh session there (the sheet carries intent).
- Delegates execute rulings **literally**, including wrong ones: an over-strict
  `Current` rule and an ambiguous "render leaves" wording both shipped as
  written, and the delegate edited a design-verbatim test to match. Check each
  ruling against the design before sending; diff any test whose expected
  text encodes a design example.
- `review` (max) found real defects (a spec contradiction, undisclosed key
  loss, vacuous tests); one finding rejected on the design text. A max-thinking
  request can die on an llm-gateway timeout (`termination: error`, cause only
  in `events.jsonl`); continuing the same `session` resumes with reads intact.

Calibration update, SL-286 PHASE-03 (2026-10-24):

- Rhythm held: `judgement` sheet draft (13 requests) → orchestrator rulings →
  six `implement` runs, one task each, ~$0.15 total. A run that implements a
  whole multi-branch function *and* its tests hit the 30-request ceiling at its
  final check, leaving a green tree; split function and tests across runs.
- The `review` pass caught a blocker in an **orchestrator ruling**, not delegate
  code: a fix the design scoped to one caller was ruled onto a shared seam
  (`fsutil::write_atomic`, 75 callers), opening a symlink trust-boundary escape.
  Before ruling a change to a shared helper, count its callers and prefer an
  opt-in sibling fn.
- Ask delegates to print a notice when an environment-dependent test skips
  (e.g. chmod under root), so a green gate cannot hide a vacuous test.
