# SubagentStart hook is sync-blocking but un-failclosable; exit 2 does not abort the subagent

Empirically established (claude-code, in-jail) by a live hook+subagent timing
experiment for SL-056, extending
`mem.pattern.dispatch.claude-subagentstart-worker-identity`. Two facts the SL-056
design needed and was guessing about.

## 1. SubagentStart is sync-blocking — stamp lands before the worker's first command

A `SubagentStart` **command** hook runs synchronously and **gates the subagent's
execution until the hook process exits** (Claude Code hooks are sync by default;
`async: true` opts out). Proven by scaling an artificial `sleep` in the hook: with
`sleep 3s` the worker's first command timestamp was `+7.0s` after HOOK_START; with
`sleep 10s` it was `+13.7s` — the worker–start lag tracks the hook duration **1:1**
and is strictly **after** the hook's exit. The marker the hook stamps into `cwd` was
`present` at the worker's first action both runs. ⇒ The "created-but-unstamped"
window is **NOT a worker-execution race**: a SubagentStart-stamp is mechanically
guaranteed present before any worker command — **when the hook succeeds**.

## 2. SubagentStart CANNOT be made fail-closed — `exit 2` does not abort the subagent

The only exit status that blocks a gated action in Claude Code hooks is **`exit 2`**
(other nonzero codes are non-blocking errors). At `SubagentStart`, `exit 2` does
**not** prevent the subagent from running: a hook that refused to stamp, emitted a
stderr reason, and exited 2 still let a benign worker execute its tool and return a
result (the hook fired + exited 2 around it). Contrast **`WorktreeCreate`**, which
**is** fail-closed — a no-output hook aborts *creation* (`WorktreeCreate hook
failed: no output`). SubagentStart has no equivalent.

⇒ The stamp-before-worker guarantee (fact 1) holds **only on hook success**. On a
stamp failure the worker proceeds **unstamped and un-gateable by the hook**. So a
SubagentStart-stamp worker-identity scheme is **accident-fenced, not fail-closed**:
the real fence against an unstamped worker is the **import belt + `DOCTRINE_WORKER`
worker-mode guard + the pre-distilled prompt**, never the hook's exit status. This
is why the SL-056 claude path is the ADR-011 D6 *O3-red* altitude, not the
fail-closed arm — and why the WorktreeCreate `run_create_fork` (fail-closed-capable
but lacking type/path in its payload, see
`mem.pattern.dispatch.claude-worktreecreate-payload-minimal-no-type-no-path`)
remains the only fail-closed-capable claude seam, deferred until its payload grows.
