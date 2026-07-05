# Workflow agent(isolation:worktree) fires the doctrine WorktreeCreate hook; payload cwd = session working-dir

**A `agent(isolation:'worktree')` spawned from inside a dynamic Workflow script
FIRES the doctrine `WorktreeCreate` hook** (`doctrine worktree create-fork`) — the
runtime does NOT silently fork off session HEAD. Verified live 2026-07-05 (Opus 4.8
harness) via a `DOCTRINE_BIN` logging shim; CHR-039 probe SQ1/SQ2.

**The payload `cwd` tracks the driver's working directory**, not a fixed repo root:
- driver shell cwd = `/workspace/doctrine` (primary) → payload `cwd` = same → fork
  minted at `.worktrees/wf_<runid>-N`, detached at HEAD, no branch (Passthrough).
- driver shell cwd = `.dispatch/SL-198` (a `dispatch/198` coord tree) → payload
  `cwd` = that coord root → fork nested under `.dispatch/SL-198/.worktrees/`.

**Consequence (IDE-031).** The confined-fork trigger
`cwd_is_coord_root ∧ coord_in_dispatch ∧ base` (`src/worktree/create.rs`) is
**reachable from a workflow**: launch the driver session cwd'd in a coord tree on
`dispatch/<n>` with `base` armed, and the workflow's isolated spawn forks at `base`
on the dispatch branch. Confined-without-base is a benign Passthrough (create.rs).

The workflow SCRIPT has no Bash so it can't `arm-spawn` itself — but it doesn't need
to: it spawns ONE confined orchestrator, which arms + nested-spawns workers from its
own Bash turns. See [[mem.fact.dispatch.confined-orchestrator-driveloop-realizable]]
and [[mem.fact.dispatch.confined-subagent-cwd-resets-breaks-positional-arming]].
Related: [[mem.fact.workflow.isolated-fork-reaches-doctrine-mcp]]. Context: CHR-039,
RFC-011; findings `.doctrine/rfc/011/chr-039-findings.md`.
