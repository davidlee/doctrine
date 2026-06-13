# REQ-248: `worktree fork --base B --branch N --dir P [--worker]` creates the worktree, provisions it (sole copier, withheld excluded), stamps the worker marker before any spawn window, and emits the per-worktree env contract — Orchestrator-classed, with compensating rollback that names any leftover on failure.

## Statement

<!-- The requirement in full: what must hold, stated testably. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
