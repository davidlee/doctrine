# REQ-251: `worktree gc --fork BR` reaps worktree+branch+target-dir only when the fork commit provably landed via `git cherry` (ancestry OR every-commit patch-id); an idempotent re-run completes or names the leftover; no runtime receipt is trusted. Orchestrator-classed.

## Statement

<!-- The requirement in full: what must hold, stated testably. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
