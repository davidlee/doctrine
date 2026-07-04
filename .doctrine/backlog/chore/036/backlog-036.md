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
future users/agents who'd benefit from it as shipped reference knowledge
(ADR-005 tiering: skills route, reference docs explain).

**Absorbs IMP-216** (closed 2026-07-04 as duplicate). IMP-216 covered the
same migration with a wider net — dispatch **+ worktree + audit + close**
traps — and proposed shipping to the *memory* tier (`mem.reference.dispatch.*`
+ a `mem.signpost.doctrine.dispatch`). CHR-036 keeps the dispatch-mechanics
focus and per-memory triage below, and folds in IMP-216's two contributions:
the audit/close close-mechanics traps (now a candidate group), and the
destination split (see **Destination — hybrid**).

### Catalog is a live snapshot, not a freeze

The candidate list below was compiled 2026-07-03 against ~321 active memory
rows. Corpus is now 326+ and **actively growing** — SL-196 / SL-198 / SL-199
(confined dispatch orchestrator, Mode B) are minting new dispatch-mechanics
memories as they land. Recent hits not yet triaged below, for example:
`mem_019f191431d2` (a `dispatch/` prefix is NOT unique to the coord tree),
`mem_019f2b670f41` (SL-182 PreToolUse jail walls only Bash/Edit/Write),
`mem_019f1a5ceef6` (dispatch arming is single-slot).

Treat the catalog as a re-derivable query result, not authored truth — re-run
`doctrine memory search dispatch` at execution time rather than trusting the
frozen uids.

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

### Close / audit mechanics (absorbed from IMP-216, silent-data-loss tier)

These are the mid-flight traps IMP-216 flagged Tier-1. They fit the *memory*
destination (agents hit them during a close/audit and `doctrine memory
retrieve dispatch`), not narrative docs:
- `mem_019ee36939ca` fold audit fix-now into journal before close_target
- `mem_019f06a18bf9` close alt: pre-FF trunk so close_target absorbs repair
- `mem_019ee41ac4c7` close-integrate on shared trunk races (also above)
- `mem_019ee4bac059` candidate can't ingest hand-resolved conflict — dead-ends
- `mem_019f096865xx` candidate worktree detached HEAD — move ref + re-admit
- `mem_019eb7415390` RV verbs refuse on a worktree fork — drive audit from parent
- `mem_019f09686501` distrust dispatch green claim — re-run suite in audit

## Destination — hybrid (decided 2026-07-04)

Split by access pattern, not one bucket for all:

- **Narrative mechanics** ("how the funnel forks/imports/lands", arm routing,
  worktree isolation model) → a shipped reference **doc** — new
  `install/dispatch-mechanics.md` (per ADR-005 "docs explain"). Read cover-to-
  cover when onboarding to dispatch.
- **Sharp mid-op traps** (close-mechanics group above, base-control footguns,
  candidate-detach) → shipped **reference memories** + a
  `mem.signpost.doctrine.dispatch` orienting entry. Retrieved *during* an
  operation via `doctrine memory retrieve dispatch`, not read up front.
- **Not doctrine-specific at all** (Claude Agent SDK / pi RPC behaviour) →
  upstream contribution notes, out of scope for the shipped corpus.

## Calibration — ongoing, not one-shot

This is a **recurring** distillation, not a single migration. The dispatch
surface is still churning (SL-196/198/199). Invest against churn:

1. **First pass, now — high-value + static only.** Ship the generic,
   well-evidenced, low-churn items: `pi-arm-worker-ops`, the git patch-id
   landed-oracle, `subagentstart-blocking-but-not-failclosable`,
   fork-rung3-base, the close-integrate race. These won't move.
2. **Defer the volatile surface** until SL-196/198/199 close — anything the
   confined-orchestrator work touches (arm placement, worktree isolation
   fallback, jail walls) will be re-authored; distilling it now = rework.
3. **Re-run periodically.** Each subsequent pass re-derives the candidate
   query, sweeps newly-minted memories, and tops up the shipped doc + signpost.

## Scope of this chore (first pass)

1. Walk the "ship as-is" list; for each **high-value + static** item, verify
   the claim still holds (re-probe if stale-risk), rewrite doctrine-CLI-
   specific language into generic dispatch/worktree/git-plumbing language.
2. Merge the noted near-duplicate pairs.
3. Stand up `install/dispatch-mechanics.md` (narrative) + a
   `mem.signpost.doctrine.dispatch` signpost (traps index); route per the
   **Destination — hybrid** split above.
4. Confirm the two RETRACTED memories are actually pruned/unlinked from
   the corpus.
5. Update/retire the source memories once distilled (avoid duplicate
   sources of truth between memory and shipped docs).
6. Leave the volatile/deferred candidates in place with a note; schedule the
   next pass after SL-196/198/199 close.

## First-pass progress (2026-07-04)

Shipped the high-value + static core:
- **`install/dispatch-mechanics.md`** — narrative reference doc (ADR-005 PULL
  tier), distilled from the static ship-as-is set: explicit fork base `B`,
  scoped verify, the patch-id landed-oracle + squash blind spot, shared-trunk
  landing races, worker-identity fencing (accident-fenced not fail-closed),
  worker self-discard traps, subprocess-arm RPC hygiene. Written in generic
  dispatch/worktree/git-plumbing language, doctrine-CLI specifics stripped.
- **`mem.signpost.doctrine.dispatch`** (`mem_019f2b93f5e1`) — shipped global
  orientation signpost: two-tier map (read the doc up front / retrieve traps
  mid-op) + a "what to retrieve when" trap-territory table.
- Wired discoverability into `mem.signpost.doctrine.reference-docs` (domain-doc
  section). Rebuilt (re-embed) → `memory sync` → boot regen; validation clean.

Source memories distilled this pass (not yet superseded — deferred to keep the
project-local originals live until the shipped set is proven):
`mem_019ee40b4c92` (pi-arm-worker-ops), `mem_019ebed87aca` (landed-oracle),
`mem_019ec166d8bf` (gc-squash blind spot), `mem_019ec0a5bdb2`
(subagentstart-not-failclosable), `mem_019eb7263a90` (fork-rung3-base),
`mem_019ee41ac4c7` (close-integrate-shared-trunk-race).

**Deferred (next pass):** promote the Tier-1 trap memories to shipped
`mem.reference.dispatch.*` (needs cross-project rewrite + re-embed); supersede
the distilled project-local originals with `--by` links; the volatile
confined-orchestrator surface (SL-196/198/199) stays local until those close.
