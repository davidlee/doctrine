`resolve_review_root` admits any tree unless this process is a dispatch worker (`DOCTRINE_WORKER`, `worktree::env_worker_set`) — the same signal `commands/guard.rs` `worker_guard` uses. The shell reads the env and hands a bool to a pure predicate. The branch-shape test (`classify_worktree_role`) leaves the review path; no host branch format is recognised (POL-002).

The review-level check stays although `worker_guard` already refuses authored review writes on the CLI, because MCP review tools call `review::run_*` directly and bypass it.

One predicate for the whole guarded set: in a worker, every verb routed through `resolve_review_root` (new, the turn verbs, prime, status, unlock) is refused; workers read via show/list. This simplifies RFC-032 D7's three-tier table, which would admit the runtime tier in any tree with a writable state tier (a worker fork included).

RFC-032 D7's second worker signal, a read-only authored tier, does not exist on Linux: spawn-confined.sh rw-binds the whole fork. The baton is a pure cache (ADR-007 D-C2; SL-268 turns in the ledger), so a review worked in a linked tree and then landed loses nothing.

Rule: one writer per RV at a time, in any admitted tree; git merge is a best-effort backstop, not a guarantee. Enforcement upgrade path: IDE-021 leases. Source: RFC-032 D7.