# RFC-011 case notes — token-inefficiency & incidental complexity

Running log. Each entry: context · friction · root cause · token cost (rough).
Captured during informal subagent-orchestration of SL-166 PHASE-04/05
(orchestrator = main agent, workers = subagents in the shared worktree).

## 2026-06-27 — SL-166 orchestration session (orientation)

- **Runtime phase-state is per-worktree, but handover asserted "PRIMARY-rooted".**
  Handover (SL-166) said lifecycle/registry verbs "resolve to the PRIMARY
  registry" even from the fork. False for phase status: `.doctrine/state/` is
  gitignored and per-worktree, so the worktree's PHASE-03=`completed` flip never
  reached primary (primary still showed `planned`). Cost: ~2 extra tool calls to
  reconcile primary-vs-worktree state before trusting either. Root cause:
  handover conflated authored-registry writes (record-delta → committed TOML)
  with gitignored runtime state. Both are "doctrine verbs" but route to different
  tiers. A worker onboarding cold would mis-target lifecycle flips.

- **CLI command-shape guesses cost round-trips.** `doctrine paths SL-166`
  (suggested in boot.md "useful commands") → `unrecognized subcommand 'paths'`.
  `doctrine slice status SL-166` → wants `<ID> <STATE>` (it's a setter, not a
  reader) AND a numeric id (`SL-166` → "invalid digit"). `doctrine status 166`
  → "unexpected argument". Three failed invocations before finding phase status
  via raw `grep` of the runtime toml. Root cause: id-form inconsistency (some
  verbs take `166`, prose/commits take `SL-166`) + reader/writer overload on
  `status` + a stale "useful commands" hint in boot.md. Each miss = one wasted
  call + its error payload in context.

- **Handover is large (105 lines) and duplicated across two copies** (primary
  `.doctrine/slice/166/handover.md` stale @ PHASE-03, worktree copy fresh @
  PHASE-04). A `diff` was needed to discover which was current. Cost: ~1 large
  read + 1 diff. Root cause: handover.md committed into the tree (so it forks
  with the branch) while also being a per-phase mutable doc — two live copies,
  no freshness marker except mtime.
