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

## 2. SubagentStart CANNOT be made fail-closed — it is a read-only event

**Authoritative** (`code.claude.com/docs/en/hooks.md`): **SubagentStart is a
read-only event — "no blocking or decision control."** `exit 2` only shows stderr
to the user; the subagent runs anyway. The exit-2-blocks table covers only
`PreToolUse`/`PermissionRequest`/`UserPromptSubmit`/`UserPromptExpansion`/`Stop`/
`SubagentStop`/`PreCompact`/`WorktreeCreate` — "other events" (incl. SubagentStart,
SessionStart, Setup) are non-blocking. **No documented hook event fail-closed-aborts
a subagent before it works.** Empirically confirmed: an `exit 2` no-stamp hook fired
around a benign two-step worker and it completed both steps + its final line.
Contrast **`WorktreeCreate`**, which **is** fail-closed — any non-zero exit aborts
*creation*. SubagentStart has no equivalent. (The matcher on `agent_type` —
`general-purpose`/`Explore`/`Plan`/custom — IS doc-supported, so scoping the stamp
to `dispatch-worker` is spec-blessed.)

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
