# SubagentStart stamp hook silently no-fires for nested-session dispatch workers

> **⚠ SUPERSEDED — the no-fire claim below does not reproduce (SL-206, 2026-07-06).**
> The original SL-068 observation (2026-06-15) is retained below as a historical
> note, but the operative claim — "`SubagentStart` silently no-fires for
> nested/child-session spawns" — is **refuted by two independent SL-206 probes**
> and must NOT be relied on (least of all as a security invariant):
> - **P1b** (agent `a418b827a5f402ec4`): a `SubagentStart(dispatch-orchestrator)`
>   hook **fired** for an `Agent`-tool spawn from a child-session main thread
>   (`CLAUDE_CODE_CHILD_SESSION=1`). → [[mem.fact.claude.subagentstart-fires-from-child-session]].
> - **P0** (nested worker `afd0c43fe5815021e`): a `SubagentStart` observer
>   **fired for a nested, subagent-initiated `dispatch-worker` spawn** — the SAME
>   matcher family this note claims no-fires. Payload carried no spawner id.
>   → [[mem.fact.claude.subagentstart-fires-nested-no-parent-discriminator]].
>
> Net: two firings, zero reproductions of the no-fire. Treat the no-fire as a
> **version-specific artifact** of the SL-068 harness build, not an invariant.
> **Do not build a security boundary (e.g. SL-206 A2 nomination) on it** — use an
> explicit intent token instead. Cited stale by `unjail-direction.md` §4.3,
> `RFC-018` §1d, and dispatch-mechanics reasoning — those should read this
> supersession, not the claim below.

---
_Historical (SL-068, 2026-06-15) — the observation as recorded; now superseded above:_

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
