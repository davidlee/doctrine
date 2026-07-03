# Docs-first, then probe: don't thrash empirically on documented harness semantics

When correctness depends on Claude Code harness behavior (hook firing,
`isolation:worktree` subagents, env injection, `CLAUDE_ENV_FILE`, payload
fields, hook exit-code semantics), do **both, in order**:

1. **Read the official docs first** via the docs cache / `docs-ground-truth-source`
   to get the *intended contract*.
2. **Confirm with a minimal live probe** — the deployed harness has diverged from
   the docs before.

The two failure modes are opposite and both real (SL-056, doctrine):

- **Guessing** burned the design across 7 inquisitions; a live experiment
  overturned three inferred "facts" (SessionStart does NOT fire for subagents;
  `CLAUDE_CODE_CHILD_SESSION` is on both worker and orchestrator; the Agent tool
  DOES return an `agentId`).
- **Over-probing** the other way: thrashing empirically on hook exit-code
  semantics drew two corrections on `exit 2` — the docs state it plainly
  (PreToolUse-style events fail open; only a named event set blocks). User steer:
  *"you have a tool for reading the official anthropic docs — you should do it
  more often."*

Docs give the contract; the probe confirms the deployed version matches — faster
and more authoritative than probing blind. This is the *ordering* discipline that
complements the probe technique catalog in
[[mem.pattern.claude.harness-introspection]] and the source in
[[mem.fact.claude.docs-ground-truth-source]].
