# Confined dispatch orchestrator is proven-realizable (SL-199 Mode B); RO shared .git is by-design — /drive-slice's defect was PLACEMENT (fresh fork) not permission, and the fix is coord-tree placement, NOT unconfining

Supersedes [[mem.pattern.dispatch.confined-orchestrator-cannot-write-funnel]],
which recorded the WRONG root cause for the SL-206 PHASE-05 failure. Corrected
2026-07-06 (Opus 4.8) after reading the SL-199 record.

## The proven design (SL-199 "Mode B", done)

A confined dispatch orchestrator drives the funnel with a **READ-ONLY shared
`.git` BY DESIGN** — that is the intended posture, not a defect:

- It sits in `Jail(coord-cwd)`: raw `Edit`/`Bash` reach the coordination working
  tree (its cwd, writable), but the shared `.git` is RO-walled (SL-199 design.md:54).
- Every `.git` write is performed by **server-side MCP tools running unconfined** —
  `dispatch_import` / `dispatch_conclude_phase` / `dispatch_reap`, and the worker's
  `worker_commit` — "as `worker_commit` does for a worker" (design.md:209).
  "Integrity never rests on the confined orchestrator" (:216).
- Arming works because a jailed subagent's cwd resets to its worktree root every
  call ([[mem.fact.dispatch.confined-subagent-cwd-resets-breaks-positional-arming]]),
  so SL-199 added a **coord-root positional discriminator**: the orchestrator arms
  via `dispatch arm-spawn --path .` from coord-root, and the WorktreeCreate hook
  forks the nested worker on `cwd_is_coord_root ∧ coord_in_dispatch ∧ base`
  (create.rs:200–212, 325). Built + adjudicated sound (inquisition-phase05.md).
- Nested `isolation:worktree` IS honored (F7 REFUTED); worker isolation is
  **def-pinned to the worker frontmatter**, explicitly NOT the orchestrator's
  per-call arg (design.md:328–337).

SL-199 proved the primitives; it left **VH-1 — the live integrated armed loop —
OWED / unwitnessed** (inquisition-phase05.md:52). See
[[mem.fact.dispatch.confined-orchestrator-driveloop-realizable]].

## The real /drive-slice defect — placement, not permission

`/drive-slice` §5.4 spawns the orchestrator as
`agent(orchestratorPrompt, { isolation:'worktree' })`. `isolation:'worktree'`
**forks a fresh `.worktrees/agent-<hex>`** off the spawning session's local HEAD
([[mem.pattern.dispatch.claude-isolation-worktree-forks-orchestrator-session-head]]),
so the orchestrator lands jailed to a *fresh fork*, NOT to the coord tree.
Consequences: `cwd_is_coord_root` never fires, arm writes land in the fork's tree,
and (driver cwd unparked) the fork is off the wrong base. This is a PLACEMENT bug.

**Fix direction: place the orchestrator jailed to the EXISTING coord tree**
(cwd=coord-root, no fresh fork — the SL-199 model), and let the worker frontmatter
carry `isolation`. Do NOT unconfine the orchestrator — that contradicts the locked,
proven Mode B (server-side `.git` writes, integrity-never-on-the-orchestrator).
See [[mem.pattern.dispatch.claude-arm-coord-placement]].

**Open premise (spike before redesign):** can a *workflow* `agent()` place a
subagent jailed to the existing coord tree (cwd=coord-root), or does the Workflow
harness only ever fresh-fork? If it cannot, the workflow-driver shape is
incompatible with SL-199's placement contract and the SL-206 *slice* premise
reopens (fallback: a main-thread skill sequencer), not just §5.4.

## Same RO-.git constraint, different victim

The RO shared `.git` for a jailed linked worktree is the SAME constraint behind
[[mem.pattern.dispatch.correct-worker-delta-in-place]] and
[[mem.pattern.dispatch.worker-commit-stale-path-false-red]] — there the RO victim
is the WORKER (correct; the server lands its bytes). For the orchestrator, RO
`.git` is ALSO correct — it writes via server-side tools. The earlier retracted
memory misread this shared constraint as "the orchestrator needs RW `.git`."
