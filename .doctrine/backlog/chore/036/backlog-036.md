# CHR-036: Validate and tidy dispatch-mechanics memories for distribution

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Why

Project-local doctrine memory has accumulated a lot of hard-won "unfuck
dispatch edge cases" knowledge — worker-spawn/base-control footguns,
worktree-isolation gotchas, arm-routing (claude/codex/pi) quirks, and
generic git-plumbing lessons from the fork→land pipeline. Much of this is
generically reusable by anyone building or operating doctrine's dispatch
orchestrator, not just knowledge this repo needed to survive. It currently
lives only as project-local memory (trust/verification varies, written in
this repo's own CLI vocabulary) and is at risk of being lost or unfound by
future users/agents who'd benefit from it as shipped reference docs
(ADR-005 tiering: skills route, reference docs explain).

Catalog compiled 2026-07-03 by reading all 321 active memory rows and
pulling full bodies for the 18 strongest candidates. Full agent output
available in this session's transcript if needed; summary below.

## Candidate memories (grouped by theme)

### Ship as-is (generic, well-evidenced, low doctrine-vocabulary lock-in)
- `mem_019eb7263a9079d3ac4f39cae72c498a` fork-rung3-base-not-session-head
- `mem_019f212439687aa1af59e009fec943fd` unarmed-agent-worker-runs-in-coord-tree
- `mem_019ec4a71f0f7592bc07d9f5dad8efdb` claude-agent-worktree-integrates-commit-onto-parent
- `mem_019ec093bd7b71518489dd187b77f0f0` claude-worktreecreate-payload-minimal (version-pinned to 2.1.173 — reconfirm)
- `mem_019ebfb61ba870219aafc14f8dc7da3b` claude-subagentstart-worker-identity
- `mem_019ec0a5bdb274b3a7cc1d5eaf4e34c5` subagentstart-blocking-but-not-failclosable (verified 2026-06-22)
- `mem_019efa04e19377c0938e58c059507a61` / `mem_019efe28d60b7d51998f1f7912b8e7b8` WorktreeCreate hook-replace / payload cwd as control channel
- `mem_019ee40b4c9272a29c645b1a89fd2bab` pi-arm-worker-ops — strongest single candidate in the corpus
- `mem_019ede2f6b487d02b160a07dea4759c6` pi.rpc-stdin-lifecycle
- `mem_019ebed87aca75319cc25187f079eb27` landed-oracle-needs-import-receipt (git cherry patch-id oracle — generic git technique)
- `mem_019ec166d8bf7903a353688035ce38b4` gc-squash-indistinguishable-from-unlanded (pair with prior)
- `mem_019ee2a5d84077d3a93c5a3ee52af7ab` reset-keep-cant-resync-already-advanced-ref
- `mem_019ec667ccc97032b29e8dd95e19aa96` + `mem_019ec676a7a575e0926062a0830ae12c` review-branch-extraneous-deletions / filter-review-branch-per-file (merge into one doc)
- `mem_019ee41ac4c77310ae4e6ac8bca74010` close-integrate-shared-trunk-race
- `mem_019eb741539075c380783b4cff747fec` rv-verbs-refuse-on-worktree-fork (verified) — strip RV-verb specifics, keep the pattern

### Ship, needs a cleanup pass first (real lesson, currently written in doctrine's own CLI vocabulary)
- `mem_019ec6142d3b71008f2149a6d84ba981` claude-isolation-worktree-forks-orchestrator-session-head — strip the "corrects X, corrects Y" retraction narrative, keep the conclusion
- `mem_019ec65ecbc77282bad7e10a5240ad27` agent-worktree-forks-bash-cwd-head — MERGE with the one above (near-duplicate conclusion)
- `mem_019ebfd16f8e7d61bcc01d2050c9db1a` claude-agent-worktree-not-fork-provisioned — check whether the "agent_type propagation unconfirmed" spike caveat has since resolved
- `mem_019ee3a0323d704193b8c9ecf359678f` subagentstart-hook-cwd-is-worker-worktree
- `mem_019eb9bf707472b0935c4ea022bc7d61` jpi.double-op-trust-gate — version-pin caveat (pi 0.79)
- `mem_019f255d977c7f309d75a6f4dc198a1b` hand-created-worktree-provisioning — generalize past the RustEmbed specific
- `mem_019ee4bac0597bf0809caf56b0e59466` split-lineage-close-conflict-direct-land — trim `dispatch candidate create` CLI specifics, keep the "build a manual-resolve escape hatch into any auto-merge verb" lesson
- `mem_019ec88a6b43746395017bd7869e38e9` prepare-review-plumbing-desync-reverts-journal
- `mem_019ec55ef17577c1b9b21d3654a71424` sync-tree-reads-ledger-not-worktree — extract the "read from committed ref tree, not filesystem, post-teardown" principle

### Keep project-local only (doctrine's own git-plumbing internals, not user-facing troubleshooting)
`mem_019ec57abf027a139d2fd1dfb52d8f01`, `mem_019ec8841cda7ee2a522bd713fdc07de`,
`mem_019ee36939ca7a70b8aa960cb478d94c`, `mem_019ec912f7fd746284bfaef00717443e`,
`mem_019ec345c2d879f3bab52aa1dad7a401` (jail-specific), and the shared-main
concurrency pair `mem_019ec473d9f57952954f770b2abcc0ea` /
`mem_019ec470...5fb7` (mostly redundant with AGENTS.md's existing
path-limit-commit convention).

### Drop / stale risk
- `mem_019ec602fe877003ba49f92fabe63a23`, `mem_019ec5f26b7b70d3ab06b7a3ba72ed72` — explicitly RETRACTED 2026-06-14; confirm pruned/unlinked
- `mem_019ede2f99a179d2968bfadfee2843a9` — pi RPC `set_auto_retry` field name, too implementation-specific, will bitrot

## Scope of this chore

1. Walk the "ship as-is" and "needs cleanup" lists above; for each, verify
   the claim still holds (re-probe if stale-risk), rewrite doctrine-CLI-
   specific language into generic dispatch/worktree/git-plumbing language
   where flagged.
2. Merge the noted near-duplicate pairs.
3. Decide destination: doctrine's own shipped reference docs (per ADR-005
   tiering — likely `using-doctrine.md` or a new `dispatch-mechanics.md`)
   vs upstream contribution notes for Claude Agent SDK / pi RPC behavior
   that isn't doctrine-specific at all.
4. Confirm the two RETRACTED memories are actually pruned/unlinked from
   the corpus.
5. Update/retire the source memories once distilled (avoid duplicate
   sources of truth between memory and shipped docs).
