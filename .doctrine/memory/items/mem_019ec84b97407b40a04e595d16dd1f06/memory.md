# SubagentStart stamp hook silently no-fires for nested-session dispatch workers

On the claude `/dispatch` arm, the matcher-scoped `SubagentStart` hook
(`doctrine worktree marker --stamp-subagent`, matcher `dispatch-worker`) does
**not fire** when the orchestrator is itself a nested/child Claude session
(`CLAUDE_CODE_CHILD_SESSION=1`) and the worker is spawned via the `Agent` tool.
No marker is written, no error surfaces — silent. `worktree verify-worker` then
correctly refuses `unstamped`.

**The mechanism is sound** — `doctrine worktree marker --stamp-subagent` works
when invoked manually with the live payload shape
`{"cwd":"<worker-worktree>","agent_type":"dispatch-worker"}` (it provisions the
D9 allowlist + writes `.doctrine/state/dispatch/worker`). Only the harness-side
hook *delivery* fails for nested sessions.

**Why it is still safe to proceed (orchestrator post-hoc stamp):**
- Worker-mode DURING execution comes from the worker's `export DOCTRINE_WORKER=1`
  self-arm (the env leg of `env_worker_set`), independent of the marker — confirm
  it was active (the worker hits the worker-fork authored-write refusal on
  entity-minting e2e tests, see
  [[mem.pattern.dispatch.worker-verify-unset-doctrine-worker]]).
- The real fence is the import R-5 belt (`worktree import` rejects any
  `.doctrine/`/`.claude/` touch), run trusted-side regardless of the marker — the
  marker "fails open" by design.
- base==B is independently provable (`S^ == B`); `verify-worker` only blocks on
  the missing marker, not the base.

**How to apply:** in a nested-session claude dispatch run, after each worker
returns and before `import`, the orchestrator (trusted) stamps the worker
worktree itself:
`echo '{"cwd":"<wt>","agent_type":"dispatch-worker"}' | doctrine worktree marker --stamp-subagent`
then `verify-worker --base <B> --dir <wt>` passes and the funnel proceeds. The
stamp is post-spawn so it does not gate execution — the env self-arm does. User
approved this posture for the SL-068 run (2026-06-15). Root-cause fix (make the
hook fire for nested sessions, or capture the live payload schema) is a separate
follow-up.
