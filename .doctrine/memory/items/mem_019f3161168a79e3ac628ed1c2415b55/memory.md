# Frontmatter isolation honored for nested subagent spawns

**Claude Code 2.1.198 honors `isolation: worktree` declared in a subagent def's
FRONTMATTER for a NESTED spawn** (a subagent spawning a subagent) — the nested
worker lands in its own git worktree, not the parent's cwd. Verified live: a
`general-purpose` orchestrator (cwd = a linked coord worktree) nested-spawned a
def whose isolation was frontmatter-only (NO Agent call-param) → the worker got an
isolated worktree, marker landed inside it, coord tree stayed clean.

**Why it matters — determinism.** Passing `isolation:"worktree"` as an Agent
**call-param** is the LLM's per-spawn responsibility, and models (even Opus)
intermittently omit it → the worker silently runs in-place in the parent tree with
no fork ([[mem_019f2d4d50427d61bc562373551d4df3]] — the omission failure mode).
Frontmatter is declarative and can't be forgotten. **Rule: anything that must be
deterministic and can ride the agent-def frontmatter or tool-surface SHOULD —
never leave worker isolation to a per-call param.** (User directive, 2026-07-05.)

- Base-control is unaffected by frontmatter-vs-call: a `WorktreeCreate` hook (if
  present) creates the worktree and controls base+branch regardless
  ([[mem_019ec6142d3b71008f2149a6d84ba981]]); the hook fires on 2.1.198
  ([[mem_019f23713d5b7552b9f99f81c08fafe8]]).
- **Caveat: agent-def registry is frozen at harness launch.** A new/edited def
  needs a full reload before it registers (a mid-session spawn of an unregistered
  type errors `Agent type '<x>' not found`).

Evidence: SL-199 PHASE-05 findings F9/F12 (`.doctrine/state/ex3-scaffold.md`).
