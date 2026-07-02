# Notes SL-190: Dispatch orchestrator state-visibility verbs

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Audit close-out (RV-234)

Dispatched via the claude arm (6 phases, serial), audited on candidate
`candidate/190/review-001` (tip `0eca671a`). Implementation faithful to the
RV-214-hardened design; four findings, all minor/nit, none gating. See RV-234
`## Synthesis` for the full closure story. Phase sheets carried no filled
Decisions/Findings — dispatch records via source-delta, so the audit synthesis is
the durable record.

**Durable harvest:**
- CHR-034 — dispatch funnel should `cargo fmt --check` before committing phase cuts
  (the impl bundle carried rustfmt drift; RV-234 F-3).
- Base-staleness → integration conflict (RV-234 F-2) is the known
  `mem.signpost.doctrine.dispatch-claude-arm-wrong-base` class; the pre-dispatch
  `git fetch . edge:main` ritual. No new memory needed.

**Reconcile handoff (RV-234 brief):** one per-slice edit — design.md §"Layering &
code impact" table (drop the stale cli.rs row; add guard.rs + reconcile.rs). No REV
(no governance/spec finding).

**Close handoff — candidate admit is BLOCKED (IMP-127).** Because review/190 forked
from a pre-SL-180 base, `dispatch candidate create` hit the conformance.rs conflict
and parked `cand-190-review-001` with `status=conflicted, merge_oid=""`. Per IMP-127
(known defect; mem.pattern.dispatch.split-lineage-close-conflict-direct-land) a
conflicted create cannot be admitted — there is no verb to feed the manual resolution
back in. The conflict IS resolved: candidate branch `candidate/190/review-001` tip
`0eca671a` is the correct no-ff merge of review/190 onto main (union of SL-180 +
SL-190 conformance.rs additions), builds clean, gate green, all suites pass. **/close
must direct-land** using this resolved tip (the documented SL-104 escape): from a
worktree, reproduce `git merge --no-ff review/190`, take the resolution equal to
`0eca671a`, verify the delta is only SL-190's additions (no SL-180 reversion), apply
to main, `close(SL-190)`, `slice status 190 done`. Do NOT rely on
`candidate admit`/`sync --integrate` for this slice.
