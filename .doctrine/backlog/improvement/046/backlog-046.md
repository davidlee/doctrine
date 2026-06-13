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
