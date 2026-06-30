# IMP-216: Migrate project-local dispatch/worktree/audit/close edge-case memories to shipped reference knowledge

## Motivation

~46 project-local memories in this repo capture hard-won operational knowledge about
dispatch funnel mechanics, close-integration races, audit fork-ban gotchas, candidate
worktree detachment traps, and worktree infrastructure footguns. None of this is
"shipped" — every project using dispatch rediscovers it. A shipped dispatch signpost
or reference memory corpus would prevent that.

## Current state (2026-06-30 dreaming audit)

Validation clean (`doctrine memory validate` → no findings). All 46 memories active.
Most unverified (few verified by design — live fire rather than formal proof).

## Catalog: project-local dispatch/worktree/audit/close edge case memories

### Tier 1 — Dispatch Close Mechanics (CRITICAL — silent data loss risk)

| uid | key | title |
|---|---|---|
| `mem_019ee36939ca` | _(none)_ | Dispatch close: fold audit fix-now into the journal before close_target projection |
| `mem_019f06a18bf9` | `mem.pattern.dispatch.close-preff-trunk-absorbs-repair` | Dispatch close alt: pre-FF trunk so close_target absorbs the candidate-only repair |
| `mem_019ee41ac4c7` | `mem.pattern.dispatch.close-integrate-shared-trunk-race` | Close-integrate on shared trunk races repeatedly |
| `mem_019ee4bac059` | `mem.pattern.dispatch.split-lineage-close-conflict-direct-land` | Dispatch candidate verb can't ingest a hand-resolved merge conflict — close dead-ends |
| `mem_019ec912f7fd` | `mem.pattern.dispatch.close-lands-via-candidate-integrate-trunk` | Close a dispatched slice by landing the admitted close_target |
| `mem_019edd33d3b2` | `mem.fact.doctrine.close-integrate-required` | SL-102 close: --integrate without --trunk is a dry run |

### Tier 1 — Candidate Worktree Traps

| uid | key | title |
|---|---|---|
| `mem_019ee33fa551` | `mem.pattern.dispatch.candidate-worktree-detached-admit-ref` | Candidate worktree is detached; advance the branch + admit by ref |
| `mem_019f09686525` | `mem.pattern.dispatch.candidate-worktree-detached-head` | Candidate worktree is detached HEAD — move the branch ref + re-admit after repair |

### Tier 1 — Audit Gotchas

| uid | key | title |
|---|---|---|
| `mem_019eb7415390` | `mem.pattern.review.rv-verbs-refuse-on-worktree-fork` | RV review verbs refuse on a worktree fork — drive audit from parent tree |
| `mem_019ec667ccc9` | `mem.pattern.dispatch.review-branch-extraneous-deletions` | Dispatch worktree review branch carries extraneous deletions — filter at integration |
| `mem_019ee33f591d` | `mem.pattern.audit.dispatched-phase-green-but-incomplete` | Dispatched phase can land green-but-incomplete |
| `mem_019f09686501` | `mem.pattern.audit.distrust-dispatch-green-claim` | Re-run the suite in audit; distrust dispatch handover failure labels |

### Tier 2 — Dispatch Funnel / Import Mechanics

| uid | key | title |
|---|---|---|
| `mem_019eec3285e4` | `mem.pattern.dispatch.worker-fork-missing-gitignored-embed` | Dispatch worker fork omits gitignored build artifacts → spurious gate/test failures |
| `mem_019eeac33cf3` | `mem.pattern.dispatch.coord-worktree-missing-build-artifacts` | Dispatch coordination worktree omits gitignored build artifacts — funnel verify fails |
| `mem_019ec65e4690` | `mem.pattern.dispatch.worktree-import-corrupt-patch-use-checkout` | doctrine worktree import corrupts the patch under rtk; use checkout-import |
| `mem_019ebb7a25ad` | `mem.pattern.dispatch.three-way-import-onto-moved-shared-main` | Dispatch funnel import: 3-way net-diff onto moved shared main, stage-only-delta |
| `mem_019ebb430f96` | `mem.pattern.dispatch.reanchor-base-on-disjoint-head-move` | Dispatch funnel re-anchors B to moved coordination HEAD on disjointness proof |
| `mem_019ed44cbfe6` | _(none)_ | doctrine worktree import corrupt patch workaround |
| `mem_019edd5da05c` | `mem.pattern.dispatch.cherry-pick-loses-unstaged-edits` | Dispatch orchestrator manual fixes after cherry-pick must be re-staged before commit |
| `mem_019eba28977b` | `mem.pattern.dispatch.worker-verify-unset-doctrine-worker` | Dispatch worker verify gate: DOCTRINE_WORKER unset when tests mint entities |

### Tier 2 — Worktree Infrastructure

| uid | key | title |
|---|---|---|
| `mem_019ed6532173` | `mem.fact.sl-085.dispatch-ref-lifecycle-gap` | Dispatch coordination branch may be GCd before audit — no lifecycle guard |
| `mem_019ec473d9f5` | `mem.system.dispatch.orchestrator-on-shared-main-contention-cost` | Dispatch orchestrator on shared main pays concurrency cost |
| `mem_019eacb95c39` | `mem.concept.dispatch.gitignored-tier-partition` | Gitignored-tier partition for worktree dispatch (ADR-006 D9) |
| `mem_019ed624cc9c` | _(none)_ | SL-085: worktree coordinate fails to find committed plan.toml on provisioned worktree |
| `mem_019ee083a8ce` | `mem.pattern.dispatch.trunk-ref-unpushed-local` | Dispatch worktree branches off trunk (origin/HEAD default), not HEAD |
| `mem_019ebed87aca` | `mem.pattern.dispatch.landed-oracle-needs-import-receipt` | Dispatch fork landed-oracle: all unsound; use git patch-id check (git cherry) |
| `mem_019ec166d8bf` | `mem.pattern.dispatch.gc-squash-indistinguishable-from-unlanded` | gc squash-merge indistinguishable from never-landed fork |
| `mem_019f01df17c0` | `mem.pattern.git.worktree-porcelain-prunable-trails-branch` | git worktree porcelain: prunable trails branch — block-accumulate |
| `mem_019ee2a5d840` | `mem.pattern.dispatch.reset-keep-cant-resync-already-advanced-ref` | git reset --keep cannot resync a worktree whose branch already advanced |
| `mem_019ef48ff180` | `mem.pattern.worktree.primary-tree-resolver-and-contextual-review-fork-ban` | Primary-tree resolver reuse and contextual review fork-ban |
| `mem_019f0715428b` | _(none)_ | Pre-warm dispatch worker fork target via reflink copy |
| `mem_019f0746d1aa` | _(none)_ | Dispatch integrate --edge advance leg is not FF-gated |

### Tier 3 — Claude/pi Arm Specifics

| uid | key | title |
|---|---|---|
| `mem_019edf7cbf87` | `mem.pattern.dispatch.claude-arm-coord-placement` | claude-arm dispatch: place coordination worktree inside cwd-jail |
| `mem_019ec4a71f0f` | `mem.pattern.dispatch.claude-agent-worktree-integrates-commit-onto-parent` | Claude dispatch-agent worker commit integrates onto parent branch |
| `mem_019ef99bfeee` | `mem.pattern.dispatch.claude-arm-isolation-fallback` | Claude-arm dispatch worker stamps worker marker on coord tree |
| `mem_019ec6142d3b` | `mem.pattern.dispatch.claude-agent-worktree-forks-session-head` | Claude Agent isolation:worktree forks session local HEAD under baseRef=head |
| `mem_019ec65ecbc7` | `mem.pattern.dispatch.agent-worktree-forks-bash-cwd-head` | claude Agent isolation worktree forks Bash-tool cwd HEAD |
| `mem_019ebfd16f8e` | `mem.pattern.dispatch.claude-agent-worktree-harness-born` | Claude Agent worktree is harness-born, not fork-provisioned |
| `mem_019f01e2f7d2` | `mem.pattern.dispatch.claude-worker-no-per-worktree-env` | Claude dispatch worker cannot get per-worktree env via hooks |
| `mem_019ee28ee9ee` | `mem.signpost.doctrine.dispatch-claude-arm-wrong-base` | Claude dispatch arm wrong-base risk under shared-clone contention |
| `mem_019ee40b4c92` | `mem.pattern.dispatch.pi-arm-worker-ops` | pi-arm dispatch worker operational footguns |

### Tier 3 — Non-Dispatchable / Boundary

| uid | key | title |
|---|---|---|
| `mem_019eb7263a90` | `mem.pattern.dispatch.fork-rung3-base-not-session-head` | Dispatch worker must fork rung-3 from explicit base B |
| `mem_019eb5eb158f` | `mem.pattern.dispatch.authoring-entities-not-dispatchable` | Authoring doctrine entities cannot be fanned out via /dispatch |
| `mem_019f03427411` | _(none)_ | Fork-landed phase leaves an unbound source-delta |
| `mem_019ec345c2d8` | `mem.pattern.dispatch.nested-bwrap-userns-confines-worker` | Nested bwrap userns confines dispatch worker OS floor |
| `mem_019ec0a5bdb2` | `mem.pattern.dispatch.subagent-start-hook-unfailable` | SubagentStart hook sync-blocking but un-failclosable |

## Migration plan

### Goals
1. A **shipped dispatch signpost** (`mem.signpost.doctrine.dispatch`) that maps the
   operational territory and links to detailed reference memories.
2. **Reference-tier dispatch memories** covering close mechanics, candidate worktree
   traps, and audit gotchas (Tier 1).
3. Infrastructure/arm-specific memories stay project-local but earn durable keys and
   cross-links.

### Approach
- Tier 1 memories → promote to reference (non-signpost) shipped memories with durable
  `mem.reference.dispatch.*` keys.
- Tier 2 memories → triage: universal parts promote, project-specific (rtk, jail,
  embed artifacts) stay local with durable keys.
- Tier 3 memories → stay project-local; ensure durable keys and cross-links to
  shipped reference.
- Add a `mem.signpost.doctrine.dispatch` orienting memory.
- After migration, supersede or repurpose the project-local originals.

### Related
- **SL-178** — exemplar: promoting `mem_019f075f` (close drift-discharge via accept REC)
  to a shipped memory; same pattern IMP-216 describes.
- IMP-127 (ingest hand-resolved merge conflict — the close-dead-ends fix)
- IMP-169 (recognize manual/external dispatch integration)
- IMP-187 (candidate worktree stage generated embed assets)
- ISS-038 (dirty/shared trunk phantom index)

## Acceptance criteria (sketch)
- `doctrine memory retrieve dispatch` returns an orienting signpost
- Dispatched-slice close, audit fork-ban, and candidate detach traps are reference-tier
- Project-local originals are superseded with `--by` links to shipments
- Validation clean after migration
