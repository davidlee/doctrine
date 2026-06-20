# IMP-046: Fresh-session probe of the literal dispatch-worker SubagentStart matcher path

From SL-056 PHASE-03 review (M2). The pivot makes the matcher-scoped `SubagentStart`
hook (`matcher: dispatch-worker`) the gate that stamps the worker-identity marker and
dissolves the σ blast-radius. PHASE-03-A proved the **mechanism** with a `*` matcher +
a `general-purpose` worker, and the official docs confirm `agent_type` matching is
supported — but the **literal `dispatch-worker` matcher path was never end-to-end
probed** (the agent-def registry needs a fresh session; a mid-session `.claude/agents/*.md`
is absent from the registry until restart).

**Do:** in a fresh session, register `install/agents/claude/dispatch-worker.md`, install
a `SubagentStart` hook with `matcher: dispatch-worker`, spawn a `dispatch-worker`
subagent at `isolation: worktree`, and confirm (a) the hook fires only for that
agent_type, (b) the marker lands in `cwd` before the worker's first command, (c) a
benign `general-purpose` subagent does NOT trigger the stamp.

**Fail-open note:** a matcher↔`DISPATCH_WORKER_AGENT_TYPE` drift makes the hook not fire
→ unstamped worker → caught by the marker-absent fail-closed privilege rule (ADR-006
D2a), not a free-writer. So this is a confidence/perf probe, not a safety blocker — but
it validates the gate the whole claude path rests on.

Refs: ADR-011 D6/D7, ADR-006 D2a/D9, SL-056 `g2-draft.md §6` (M2),
`mem.pattern.dispatch.subagentstart-blocking-but-not-failclosable`.

---

## Probe result (2026-06-20, fresh session via kill+resume)

Ran the probe with tracer hooks on `SubagentStart` and the live single clean
`dispatch-worker` stamp hook. Findings, in order:

- (a) **hook fires** for the `dispatch-worker` subagent — confirmed. ✅
- **matcher works**: `matcher: "dispatch-worker"` matched on the payload
  `agent_type` (a matcher'd tracer and a catch-all tracer both fired; a benign
  `general-purpose` subagent gets no marker). ✅
- payload `cwd` = the worker worktree, `agent_type` correct. ✅
- (b) **marker does NOT land** — the gate FAILS. ❌ Root cause is NOT the matcher
  (IMP-046's hypothesis) and NOT Defect B poison: `run_stamp_subagent` resolves the
  provision SOURCE via `root::find` on the **hook process cwd**, but the harness runs
  the hook with **process cwd = the worker worktree** (proven: hook `pwd` == payload
  `cwd`). source==fork → `verify_sibling_worktree` bails `fork path is the source
  tree itself; refusing to provision` (`src/worktree.rs:417`).

Net: the literal matcher path is sound; the auto-stamp's source-resolution is the
real defect. Routed to **ISS-011 Defect C** with the fix direction (resolve SOURCE
to the primary worktree, not the hook cwd). Harness finding recorded as
`mem.pattern.dispatch.subagentstart-hook-cwd-is-worker-worktree`. The fail-open
note holds: unstamped → caught fail-closed (ADR-006 D2a), so confidence/perf, not
safety. **Probe objective met; this item can close (finding lives in ISS-011).**

---

## VH-1 re-run (2026-06-20, post-SL-125 integration)

Re-ran the probe on `main` with the rebuilt orchestrator binary
(`~/.cargo/bin/doctrine`, built Jun 20 18:18, carries `primary_worktree` — the
SL-125 fix). Spawned a `dispatch-worker` at `isolation: worktree` plus a
`general-purpose` negative control. No hand-stamp.

- **dispatch-worker** (`agent-a08c8f12f8f91e243`): marker **present** at
  `<worker>/.doctrine/state/dispatch/worker` (0-byte presence flag) on the worker's
  first command — **auto-stamped, no hand-stamp**. ✅
- **negative control** (`general-purpose`, `agent-aa666c34663c908cf`):
  `.doctrine/state/dispatch/` absent, no marker — matcher gates correctly. ✅

**VH-1 satisfied.** SL-125 (Defect C fix) validated end-to-end: the claude-arm
auto-stamp now lands the marker FROM the primary worktree, source ≠ fork. Closes
the SL-125 deferred-acceptance caveat / FU-1 VH-1 follow-up.
