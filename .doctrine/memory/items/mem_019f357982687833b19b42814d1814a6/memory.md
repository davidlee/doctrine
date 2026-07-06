# [RETRACTED — WRONG ROOT CAUSE] Superseded by mem.pattern.dispatch.confined-orchestrator-placement-not-permission

> **DO NOT TRUST THIS MEMORY'S ORIGINAL CLAIM.** It asserted the confined dispatch
> orchestrator "cannot drive the funnel" and "needs RW `.git`". **That is FALSE.**
> Superseded 2026-07-06. Read
> [[mem.pattern.dispatch.confined-orchestrator-placement-not-permission]] instead.

## Why it was wrong

The confined orchestrator's **RO shared `.git` is BY DESIGN** — the proven SL-199
"Mode B" (slice done) has it write coord `.git` via **server-side MCP tools**
running unconfined (`dispatch_import`/`conclude`/`reap`, and the worker's
`worker_commit`), never directly. "Integrity never rests on the confined
orchestrator" (SL-199 design.md:216). The orchestrator arms via a coord-root
positional discriminator (`cwd_is_coord_root`, create.rs:200–212) — built + sound.

This memory's original claim contradicted knowledge **already recorded correctly**
at the time it was written:
[[mem.fact.dispatch.confined-orchestrator-driveloop-realizable]],
[[mem.pattern.dispatch.claude-isolation-worktree-forks-orchestrator-session-head]],
[[mem.pattern.dispatch.claude-arm-coord-placement]]. It was an outlier, not a discovery.

## The actual defect it misdiagnosed

The SL-206 PHASE-05 `/drive-slice` failure was **PLACEMENT, not permission**: §5.4
spawns the orchestrator with `isolation:'worktree'`, which forks a *fresh*
worktree instead of jailing it to the existing coord tree. The fix is coord-tree
placement, **NOT** unconfining the orchestrator (which would contradict locked,
proven Mode B). Full corrected account in the successor memory and SL-206 notes.md
FINDING 3 (corrected).
