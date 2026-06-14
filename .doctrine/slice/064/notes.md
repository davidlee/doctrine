# Notes SL-064: Coordination-branch isolation: dedicated worktree + integration-sync seam for dispatch

Durable per-slice scratchpad - tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Planning

2026-06-14: Authored the SL-064 executable plan and materialised runtime phase
tracking. The plan has seven phases: governance/OQ-D fence, coordination
worktree creation, projection plumbing and run ledger, prepare-review sync,
integrate/replay sync, source skill alignment, and end-to-end proof. Slice
status was advanced to `ready`; planning changes were committed in the same turn
under a `plan(SL-064)` commit.

Verification run for planning: `doctrine slice phases 064`,
`doctrine slice status 064 ready`, `doctrine slice list --filter
coordination-worktree`, `git diff --check`, and ASCII scan over the new plan
files. No code was changed, so `just check` was not run.

## PHASE-01 — Governance amendments and OQ-D fence (completed)

2026-06-14: Amended ADR-006 for SL-064 placement/identity refinements and pinned
the OQ-D fence. Doc-only; commit `7156f44`. Slice advanced `ready → started`.

Amendments (inline `... amendment/note (SL-064)` notes, SL-056-amendment style;
decision ids unchanged):
- **D8** — coordination branch runs in its own dedicated worktree (retires the
  "trunk in solo / delta branch in team" placement). Topology/projection/routing/
  recovery/D1-tightening remain ADR-012's — cross-referenced, not duplicated.
- **D2a** — orchestrator may run in a *linked* coordination worktree; permission
  rests on marker-absence, not `!is_linked_worktree`; "runs at the root" retired;
  `env DOCTRINE_WORKER` must not leak. `worker_mode` formula unchanged.
- **D2b** — marker-absence is a transitional assumption, not positive identity;
  the fence is defence-in-depth, NOT a proof of gc/sync impersonation coverage
  (RV-025 B3). OQ-D plan-gate pinned as binding obligations (Orchestrator-verb
  restriction + mandatory impersonation tests). IMP-065 = real positive-marker close.
- **D9** — markerless coordination-tree creation variant (same fork+provision
  ladder, no worker marker; regenerates the coordination/runtime tier).
- References + `updated = 2026-06-14`.

**D7 NOT amended** (handover F1 / EX-1) — git diff shows no `±` D7 line.

Verification: VT-1 — worker-guard suites (`e2e_worker_guard`, `e2e_worktree_{fork,
import,land,gc}`) green, 44 tests, unchanged (doc-only ⇒ trivially green; sentinel
against accidental src edit). VA-1/VA-2 by self-comparison sweep against `adr show
12` + design §5.

**OPEN → audit (F-PH01-1, VA-3 / handover F8):** the ADR-006 amendment has had
self-comparison only — it must reach **adversarial acceptance** (an inquisition
pass or `/audit`) before close, per the slice closure intent. Carry into PHASE-07
/ `/close`.
