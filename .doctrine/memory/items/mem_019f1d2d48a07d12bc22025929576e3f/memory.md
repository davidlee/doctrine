# macOS pretooluse consumer wires to Seatbelt via cfg-split probe_backend

The claude `PreToolUse` hook consumer (`doctrine worktree pretooluse`,
`worktree::pretooluse`, command tier) reaches the SL-183 macOS Seatbelt jailer
through a `cfg`-split `probe_backend`. Landed SL-183 PHASE-04 (commit `b2cd1000`).

## The wiring
- `#[cfg(not(target_os = "macos"))] probe_backend(_cwd)` — Linux: `bwrap` on PATH
  ⇒ `Backend::Bwrap`, else `Deny{bwrap-unavailable}` (unchanged).
- `#[cfg(target_os = "macos")] probe_backend(cwd)` — builds `RealEnv { main_root }`
  (the leaf's injected `ResolveEnv` seam) and returns
  `seatbelt_backend(resolve_inputs(cwd, &main_root, &env))`. `Ok ⇒
  Seatbelt(ResolvedMac)` (Bash wrapped into `sandbox-exec`); `Err ⇒ Deny{reason}` —
  fail-closed on every `cwd`→policy branch (POL-002, F-B4), never pass-through.
- `main_root` = realpath'd `CLAUDE_PROJECT_DIR` anchor (factored into
  `project_anchor()`, shared with `cwd_is_project_worktree`). Legit because this
  module IS the claude-specific PreToolUse handler (ADR-011 per-harness altitude) —
  the claude env var is an existing contract, not new coupling.

## Why cfg-split, not runtime
`bwrap` is Linux-only, Seatbelt macOS-only. `cfg` mirrors jail.rs's "Seatbelt
`Jailer` never built on Linux" gating and keeps the mac-only `getconf`/`resolve_inputs`
off Linux CI. The pure routing downstream (`select_jailer` → `decide_bash`) is
arm-neutral over whichever `Backend` is returned.

## Load-bearing gotchas
- **Pre-SL-183 the consumer was Seatbelt-blind:** `probe_backend` only emitted
  `Bwrap`/`Deny{bwrap-unavailable}`, so on macOS every worktree-subagent Bash denied
  `bwrap-unavailable` and the jailer (built + unit-tested in jail.rs by PHASE-03) was
  **dead code from the hook entry**. If you see the jailer "not firing" on macOS,
  check that this wiring is present.
- **Backend resolved LAZILY on the Bash path only.** `decide_write` walls Edit/Write
  on `pathcheck` and ignores the backend; running `probe_backend` on Edit/Write would
  spawn git/getconf AND mkdir `<wt>/.tmp` (a `resolve_inputs` side effect) on the hot
  hook path (INV-1) for a discarded result. `run_pretooluse` gates it to
  `tool_name == Bash`; non-Bash gets a never-read placeholder `Backend::Deny`.
- **Containment needs a PROVISIONED policy.** A bare Agent-tool `isolation:worktree`
  subagent lands in `.claude/worktrees/agent-<id>` (harness git, not `create-fork`) ⇒
  no policy ⇒ `resolve_inputs` branch (e) `PolicyMissing` ⇒ Deny. To get containment
  (wrap, not deny) a policy must exist at
  `<main>/.doctrine/state/dispatch/jail/<basename(cwd)>.toml`.
- **Cross-arm asymmetry:** the macOS arm fails CLOSED on absent policy (Deny); the
  Linux `pretooluse` policy path (`resolve_provisioned_policy`) defaults to a Default
  floor. Deliberate. Sibling: [[mem.pattern.dispatch.seatbelt-insitu-subagent-nesting]],
  [[mem.pattern.dispatch.jail-resolve-inputs-injected-env]],
  [[mem.pattern.dispatch.claude-worktree-subagent-bwrap-confinement]].
