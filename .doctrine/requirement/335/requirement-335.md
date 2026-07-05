# REQ-335: A confined-orchestrator altitude exists as a harness-neutral tier: an orchestrator running inside the coordination worktree under a cwd-confining jail with the shared object store read-only lands every worker delta and coordination write through a gated write-funnel of mediated tools (import -> conclude -> reap), never a direct coordination .git write; it reads authored/runtime state raw but performs all mutation through the funnel (reads-raw/writes-mediated), forks and confines nested workers depth-agnostically, and — because trunk-facing verbs (refresh-base/candidate/integrate) write outside the jail — reports-and-halts them to the delegating parent rather than performing them; the funnel tools route by their declared args alone, so the tier is a property of the mediated-write contract, not of any single harness.

## Statement

<!-- The requirement in full: what must hold, stated testably. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
