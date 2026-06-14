# Implementation Plan SL-064: Coordination-branch isolation: dedicated worktree + integration-sync seam for dispatch

Prose companion to `plan.toml`. Narrative only - no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md section reference forms. -->

## Overview

SL-064 ships the accepted ADR-012 topology and the remaining ADR-006 placement
amendments in three layers:

1. Lock the governance boundary and OQ-D fence.
2. Build the coordination-tree and projection mechanisms in code.
3. Rewire the dispatch/worktree skills and prove the full lifecycle end to end.

The phase boundary deliberately keeps the new projection machinery below the CLI
until its git plumbing and committed run ledger are testable on their own. The
sync verb is then split by ADR-012's two stages: prepare-review first, integrate
and replay second. Skill prose lands after those command surfaces exist, so the
skills cite real behavior rather than aspirational shell recipes.

## Sequencing & Rationale

PHASE-01 is first because SL-064 has two governance owners. ADR-006 owns the
placement and identity refinement (D2a/D8/D9); ADR-012 owns topology, routing,
projection, audit ordering, the D1 tightening, **and the D7 projection-semantics**
— so ADR-006's D7 is left unchanged (funnel discipline preserved), not re-amended
here, to avoid double-owning what ADR-012 already carries. The plan-gate for OQ-D
also belongs up front: the slice must not accidentally turn marker absence into a
claimed identity proof while it adds a markerless coordination tree. OQ-D
restriction + impersonation tests therefore span the **whole** Orchestrator verb
class — creation (PHASE-02), prepare-review (PHASE-04), and integrate (PHASE-05,
the trunk-writing verb) — not just creation.

PHASE-02 isolates the coordination branch before projection exists. The
coordination worktree is the inner-loop foundation: once the funnel writes
`dispatch/<slice>` instead of the session tree, the shared-main contention
surfaces are structurally out of the per-batch path. Creation distinguishes two
branch-exists cases: a **live worktree** on `dispatch/<slice>` means a concurrent
same-slice run and is refused; the branch existing **without** a live worktree is
a handover-resume and reattaches the same branch (design §1 resume stability).

PHASE-03 builds the reusable primitives: filtered tree composition, commit-tree,
CAS ref update, and the committed run ledger. This keeps the most delicate git
work out of command-flow code and gives prepare-review and integrate the same
tested substrate. It also owns the **funnel-time recording surface** that writes
`boundaries.toml`/`orthogonal.toml` — a tested verb, not skill-prose appends, so
stage-1 synthesis (C's phase cut, B's orthogonal exclusion) consumes
machine-written OIDs rather than hand-authored ones. Because the primitives extend
`src/git.rs` — the born-frame capture seam — the phase carries an explicit
behaviour-preservation gate on the `forget.*.v1` byte-reproduction.

PHASE-04 implements stage-1 prepare-review. It materializes the exact objects
audit will inspect, without writing trunk. This is the first externally visible
projection point and the right place to wire `dispatch sync` into the
Orchestrator-verb guard.

PHASE-05 implements stage-2 integration and replay. It is separate from
prepare-review because it has different failure semantics: target refs may have
moved, audit has already happened, and the command must be idempotent after
partial application.

PHASE-06 updates the source skills after the commands are real. This rewrites
the human/agent operating loop from "the coordination branch is the deliverable"
to "coordination is the SSoT, review/phase refs are the deliverables, audit gates
those refs, and integration is explicit." The post-audit trigger is pinned to
`/close` (not `/dispatch`): conclude stops at prepare-review + worktree removal,
audit runs from parent/root against the prepared refs, and only then does `/close`
invoke `dispatch sync --integrate`.

PHASE-07 is the system proof. The earlier phases cover pieces; this phase proves
the user-visible claim of SL-064: dispatch runs in a dedicated coordination
worktree, leaves reviewable refs, keeps unreviewed code off trunk by default,
removes only the worktree directory at conclude, and preserves behavior of the
existing funnel suites.

PHASE-08 is the **§8 claude-arm base-correctness thread** — appended after the
§1–§7 integration-sync thread and deliberately independent of it. It exists
because the claude `/dispatch` arm first shipped unusable: a worker forked off
`origin/HEAD` (behind local trunk) developed against a stale tree, caught only
late at `import`. The mid-session `babd656` detour wrongly concluded the claude
isolated-worker base was opaque/uncontrollable (option X). A controlled
marker-commit test (2026-06-14) disproved that and locked **option Y**: under
`baseRef='head'` the worker forks the spawning orchestrator session's local HEAD,
so the base is controlled by **orchestrator placement** (run the session inside the
`dispatch/<slice>` coordination tree), not a ref-redirect — no `WorktreeCreate`
hook needed for base control.

The phase ships two legs (DD-11) plus their governance correction. Leg 1 is
correctness-by-default: the installer writes `worktree.baseRef='head'` into the
same gitignored `.claude/settings.local.json` the HookSpec merge already owns, so
the imposition is per-operator, never team-committed (the §8.3 layer wall). Leg 2
is the fail-loud belt: a post-spawn `verify-worker` verb that refuses+reports a
residual wrong base **before** `import`, never a silent wrong-base landing. It
keeps the established pure/impure split — a `classify_worker_verify` classifier
(ADR-001 leaf, no git/disk) behind a `run_verify_worker` shell that reads the
marker from the worker worktree's withheld tier and computes base==B via
`merge-base --is-ancestor` through the rtk-bypassing `src/git.rs` runner. Because
the orchestrator only regains control when the `Agent` tool returns (§8.1), the
verb is honestly post-worker: it prevents a wrong *import*, not the wasted worker
run — pre-worker fail-closed remains the deferred `WorktreeCreate` nicety (IMP-072),
no longer justified as a base-control need under Y.

The ADR-011 amendment is in place (D3/D5/D6/D7) because ADR-011 owns the
base-pinning cell and altitude table — a correction-with-authority, consistent
with SL-064 amending ADR-006 and authoring ADR-012. The VH harness criterion is
already satisfied and is carried as evidence, not re-probed; every other criterion
is doctrine-unit-testable.

## Notes

- `IMP-065` is not implemented here. SL-064 ships the transitional markerless
  coordination path with the OQ-D fence and impersonation tests; the positive
  coordination marker remains the dedicated follow-up.
- `IMP-041` is resolved by the new lifecycle: worker/coordination worktree
  directories are cleanup targets, while `dispatch/*`, `review/*`, and `phase/*`
  refs survive until their integration lifecycle says otherwise.
- `IMP-043` has two distinct closures, one per thread, not a contradiction: the
  per-batch *re-anchor* demotes to sync time (§1–§7, PHASE-05/07) via CAS refusal
  and replay; the deferred *content-base assertion* (the D5 worst case) closes in
  PHASE-08 via the `verify-worker` `merge-base --is-ancestor` check.
- `IMP-052` (orchestrator post-spawn marker check) is promoted prose → verb in
  PHASE-08, folded into `verify-worker`.
- `IMP-072` (`WorktreeCreate` pre-worker fail-closed arm) stays deferred and is
  reframed by option Y: it is a fail-closability nicety only, no longer a
  base-control mechanism, and carries the σ blast-radius cost explicitly.
