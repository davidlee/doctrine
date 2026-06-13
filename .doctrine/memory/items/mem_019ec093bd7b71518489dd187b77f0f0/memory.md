# Claude Code WorktreeCreate hook payload carries no type, no target path, no base — use SubagentStart to stamp worker identity

Empirically probed on the live harness (SL-056 PHASE-02 O3 spike). The deployed
WorktreeCreate behaviour **contradicts the published hook docs** — verify
empirically, do not trust the docs here.

**WorktreeCreate** fires for an `isolation: worktree` Agent spawn and **replaces
default creation, fail-closed** (a no-stdout hook → `WorktreeCreate hook failed: no
output` → creation aborted). Mid-session settings-hook edits take effect **without
restart**; mid-session `.claude/agents/*.md` registration does **not** (registry is
session-start-loaded). The actual payload is only:

```json
{"session_id","transcript_path","cwd","hook_event_name","name":"agent-<hex>"}
```

`name` is the agent *instance* id — **not** the type. There is **no
`subagent_type`/`agent_type`, no `worktree_path`, and no `base_path`/`ref`**. So a
hook **cannot gate on agent type, cannot learn the target dir, and cannot see the
base commit** at WorktreeCreate. (The default-created worktree's HEAD was not the
orchestrator's HEAD — the base is opaque and not orchestrator-controlled.)

**SubagentStart** is the usable worker-identity seam instead. Its payload **does**
carry `agent_type` (matcher-scopable) and **`cwd` = the worker's worktree path**:

```json
{"session_id","transcript_path","cwd":"<worktree>","agent_id","agent_type","hook_event_name":"SubagentStart"}
```

So a `SubagentStart` hook with `matcher: <agent-name>` can **provision + stamp a
marker into `cwd`** — scoped cleanly to the dispatch worker, no blast radius on
benign isolated subagents. It fires *after* worktree creation ⇒ a **fail-open
created-but-unstamped window** (accident-fenced + prompt-enforced, not malice-proof).

**Why:** the agnostic-fork-marker design (ADR-011 / SL-056) hoped for a fail-closed
WorktreeCreate `create-fork` that gates on `subagent_type` and creates at the
payload path; that is **not buildable** against this payload. **How to apply:** for
claude dispatch-worker identity, use **SubagentStart-stamp** (matcher-scoped) over
Claude's default worktree creation; treat WorktreeCreate as observe/replace-only
with no type/path/base. Revisit if a future harness version enriches the payload or
an IDE-004 env channel lands. See [[mem.pattern.dispatch.claude-subagentstart-worker-identity]]
and [[mem.pattern.dispatch.claude-agent-worktree-not-fork-provisioned]].
