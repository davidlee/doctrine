# REQ-249: `worktree import --base B --fork BR` funnels a single distilled worker delta: refuse unless HEAD==B, the tree is clean, and S^==B; reject a `.doctrine/`/`.claude/` belt touch in the B..S tracked diff; then `git apply --3way --index` (the commit is separate, no runtime receipt). Orchestrator-classed, dispatch-only.

## Statement

<!-- The requirement in full: what must hold, stated testably. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
