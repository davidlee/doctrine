# Confined orchestrator can drive the shipped nested-worker machinery

**Claim.** A **confined** (jailed-at-coord-root) `dispatch-orchestrator` can drive
the shipped dispatch worker machinery without being unjailed. This is the
empirical basis for **confined-A2 viability** — i.e. the SL-206 unjail/nomination
confinement-boundary change may be **unnecessary**.

**Evidence (SL-206 P0, 2026-07-06).** Main thread parked cwd at `.dispatch/SL-209`
(dispatch/209 @ `fa951846`), `Agent`-tool-spawned a `dispatch-orchestrator`
(no isolation ⇒ inherits cwd ⇒ jailed at coord root). Observed:
- **Confined, as designed.** `git commit --allow-empty` → `fatal: Unable to create
  '…/.git/worktrees/SL-209/index.lock': Read-only file system` (exit 128). Shared
  `.git` sealed. (Mechanism: bwrap wrap — [[mem.fact.claude.jail-ro-git-is-bwrap-wrap-passthrough-commits]].)
- **In-jail state write OK.** Write under the coord tree's own `.doctrine/state/`
  succeeded (exit 0). ⇒ arming (a DispatchRecord / marker write) is feasible
  from the confined orchestrator.
- **Nested `WorktreeCreate` fires.** The orchestrator's `Agent(isolation:worktree)`
  worker spawn fired `WorktreeCreate` (name `agent-afd0c43f…`) ⇒ the plugin
  `worktree create-fork` provisioner runs for a subagent-initiated fork.
- **Worker forks at correct base.** Worker landed on its own `dispatch/agent-…`
  branch at the SAME base HEAD `fa951846`.
- **`worker_commit` resolves + runs the full gate.** It did NOT fail for a missing
  dispatch record — it resolved the agent, ran the belt, and refused only at
  `check commit` = `commit-gate-red` because **the SL-209 base itself has a red
  `test` recipe (exit 101)**. Incidental base-health, not a confinement/resolution
  failure. (`worker_commit` is the server-side unconfined commit path, so it does
  not need the orchestrator unjailed.)

**Caveat.** The full funnel (`dispatch_import` → verify → conclude) was NOT run
end-to-end here — but those are server-side MCP (Mode B, proven), and coord-tree
state writes work. So the substrate confined-A2 needs is all present.

**Token-efficiency note (RFC-011).** The `worker_commit` red-gate refusal embedded
the entire `check commit` transcript (~295k chars: thousands of `[Prose Citation]`
warnings + the red suite). A red-gate refusal should summarize, not inline the
whole transcript.

See `.doctrine/slice/206/unjail-direction.md` §6/§7, and
[[mem.fact.claude.subagentstart-fires-nested-no-parent-discriminator]] (same probe;
the escalation finding that makes the *unjail* path costlier than confined-A2).
