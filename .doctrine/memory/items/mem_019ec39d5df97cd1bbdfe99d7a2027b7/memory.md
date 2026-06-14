# Shared CARGO_TARGET_DIR makes a worktree audit false-RED; touch+per-suite to defeat

Worktree audit false-RED: a coord worktree shares `CARGO_TARGET_DIR` with main via
the jail redirect (`~/.cargo/doctrine-target-jail/debug`, mem.pattern.build.jail-target-redirect),
so test binaries compiled from **main's** source shadow the fork's → a suite RED that
is not a logic defect.

Surfaced in the SL-056 reconciliation audit (RV-016): two findings (F-1 deterministic,
F-2 flaky) were raised on this artifact and **withdrawn** once caught — the slice was in
fact green per-suite on a fresh compile.

To trust ANY test result in a shared-target worktree:
- `touch` the test file(s) to force a fresh recompile from the fork's source;
- run suites **individually** (or one focused `--test a --test b …` group), NEVER bare
  `cargo test --workspace` — it thrashes the shared target and folds in foreign reds;
- always `env -u DOCTRINE_WORKER` (tests mint entities,
  mem.pattern.dispatch.worker-verify-unset-doctrine-worker).

Cousin of [[mem.pattern.dispatch.worktree-removal-stale-manifest-dir-false-red]] (a
deleted-fork CARGO_MANIFEST_DIR false-red) — same family: stale build artifacts in a
worktree masquerading as a RED.
