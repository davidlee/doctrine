# REV REV-046 — Adopt dispatch execution capsules at implementation cutover

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

RFC-025's capsule spike establishes a narrower and simpler **target** authority
model for dispatch: a persistent trusted control-plane orchestrator launches a
fresh headless worker in a fresh execution capsule for each phase; the worker has
no canonical repository authority; the control plane harvests, verifies,
normalizes, and admits by immutable identity under a durable journal and CAS.

That target makes a cluster of incumbent mechanisms unnecessary: worktree marker
identity, `DOCTRINE_WORKER`, SubagentStart stamping, base-by-placement, the gated
`worker_commit` exception, per-harness arm routing and altitude, coordination-
worktree placement, patch import, and the nominated-unjailed orchestrator
choreography. The mechanism census records **15 primary DELETE rows**, alongside
13 transformations, 17 keeps, and one row scoped to solo worktrees. It does not
authorize deleting present-tense governance before an implementation exists.

This Revision therefore stages **target-state revise intent** while preserving
the incumbent/target distinction. It remains `proposed`, unapplied, and
unapproved until the gates below clear. ADR-011 and the existing dispatch specs
remain authoritative for shipped dispatch until the capsule cutover.

## Evidence boundary

The scope comes from RFC-025's go/no-go, not from a flat reading of the census:

- the measured confinement result is **Linux/bwrap only** on two client fixture
  shapes; it is not a macOS result;
- conflict/staleness **refusal** is evidenced by the spike; resolution and
  admission are designed, not newly evidenced, by DEC-137 (QUE-202);
- bundle ingestion is settled by DEC-135, with hostile bytes snapshotted into
  owned storage and imported through a fresh quarantine repository (QUE-200);
- the client interpretation policy is settled by DEC-136 as a required
  `[interpretation]` block in `.doctrine/doctrine.toml`, resolved from the
  contracted base and only restrictable by phase contracts (QUE-201);
- the evidence is sixteen hazard rows with a capsule-model boundary or recorded
  dissolution plus two incumbent regression legs — never “16/16”; and
- the real-agent capsule phase is one measured run, not a performance comparison
  or production-readiness assessment.

The Revision retains its hard `needs` edges to QUE-200, QUE-201, and QUE-202.
All three are now terminal and supply DEC-135, DEC-136, and DEC-137
respectively. Their answers belong in the authored target contract before
approval; this scope does not promote the spike's refusal evidence into an
unmeasured admission claim.

## Governance boundary

### Product altitude — PRD-015 (primary, `modify`)

PRD-015 already owns the correct capability: concurrent work in isolated units,
a trusted canonical-mutation boundary, reviewable results, audit-gated
integration, crash recovery, harness neutrality, and policy neutrality. No new
product spec is needed.

Revise its worktree-specific promises into evergreen isolation-unit/capsule
language. In particular, REQ-296–304 must be swept rather than assumed preserved:
the target worker may write Doctrine state and commits **inside the capsule**, but
none are canonical until trusted-side admission. “Private working tree”,
coordination-tier absence, source-only workers, coordination-branch recovery,
and marker-absence fail-closed wording cannot survive unchanged. The product
outcomes — exact reviewable deltas, explicit admission, audit-gated integration,
and crash-safe recovery — do survive.

Solo `/execute` worktrees remain a supported non-dispatch isolation mode. This
Revision does not migrate solo execution to capsules.

### Target technical boundary — new capsule container

Author a new **Dispatch execution capsules** technical container under SPEC-003,
descending from PRD-015. It owns the harness-neutral capsule contract:

`provision → launch → notify/inspect → harvest/freeze → verify in a separate
capsule → normalize → admit/integrate → close → explicit cleanup`.

It also owns the control-plane/capsule authority split, fresh phase transaction
lifecycle, contracted base and interpretation policy, the live-work versus
admission-journal versus forensic-exhibit boundary (DEC-133 and DEC-137), and
the platform-backend contract. Per
DEC-136 it extends the shared `.doctrine/doctrine.toml` parser with a required
`[interpretation]` block for capsule dispatch, resolves it once from the
contracted base, and permits work contracts only monotonic restriction. The
Linux backend may name bwrap as measured. The cross-platform contract states
equivalent properties, not an unmeasured macOS mechanism.

For v0 the frozen source capsule is the rescue payload. It is never
automatically destroyed before an incorporating result is integrated and
closed; formal repair always uses a fresh capsule. Capacity handling begins
with a conspicuous configurable free-space warning and manual intervention,
not a new archive, pre-reservation, backpressure, or eviction subsystem.

The new container and its requirements are not created by this scoping chore.
They are authored when QUE-200–202 are settled and then added to this Revision's
touched set before approval.

### SPEC-012 — retain and narrow (`modify`)

SPEC-012 remains the worktree mechanism owner for solo `/execute` and remains the
incumbent dispatch mechanism authority until cutover. Narrow it after cutover:

- keep worktree provisioning needed by solo execution, `land`, and the
  ancestry-based solo leg of `gc`;
- retire dispatch-only marker identity, marker remedies, branch-point/import
  machinery, `fork --worker`, per-worktree env delivery, and patch-id admission
  machinery that the capsule journal replaces;
- transform `.worktreeinclude` lessons into the new capsule provisioning
  contract without pretending the two mechanisms are identical; and
- preserve the born-frame/git seam and pure/imperative split where they remain
  independently owned.

The requirement sweep must explicitly disposition REQ-189–196 and REQ-248–252.
Rows retained for solo worktrees must say so; dispatch-only rows retire only at
cutover.

### SPEC-021 — revise and move beneath the capsule container (`modify`)

Preserve the process obligations that transfer: ordered lifecycle, report-and-
halt, verification before admission, knowledge after confirmed canonical code,
durable funnel position, verb legality, and the single-next-action oracle.

Retire or transform the mechanisms tied to the incumbent topology: arm routing,
one-landing-per-worktree-base, coordination-worktree placement, marker/hook
identity, per-harness altitude, patch-import fallbacks, nominated orchestration,
and direct coordination-tree commits. Reconcile REQ-287–295 and REQ-335 plus
REQ-384–387 individually; do not bulk-retire the state-machine investment that
the census says transfers.

At target state SPEC-021 is a component of the new capsule container rather than
of SPEC-012. The parent move and prose/requirement revision land together.

### SPEC-022 — preserve the Git substrate (`modify`)

SPEC-022 remains a sibling container under SPEC-003. Preserve immutable OIDs,
candidate admission, journal-before-mutation, idempotent CAS replay, audit-gated
integration, trunk resolution, and the no-force/no-auto-resolve envelope.

Revise only the topology-dependent parts: ref population, contracted-base
refresh, capsule-result provenance inputs, admission-journal sourcing,
live-work/forensic-exhibit lifecycle, and the working-tree-specific legs of
projection. Per DEC-137, same-base conflict/staleness resolution extracts the
existing candidate engine behind an explicit capsule-provenance seam; it does
not introduce a second conflict system or carry the incumbent coordination
journal forward as scaffolding. Clean candidates and hand-resolved candidates
are verified in fresh capsules before immutable admission and expected-tip
integration.

### ADR-006 — separate dispatch cutover from surviving solo worktrees (`modify`)

Amend the dispatch clauses of the worktree posture: worker-sole-writer wording,
marker/env enforcement, the raw-tree residual, pre-distilled worker context,
funnel cadence, coordination placement, dispatch provisioning, and candidate
interaction. Preserve the policy-agnostic stance, trunk-side ID allocation,
storage-tier merge safety, and solo `/execute` worktree semantics that still
apply. The capsule worker has broad **local** authority; the preserved invariant
is that canonical mutation belongs to the control plane.

### ADR-008 — replace repo-local dispatch confinement machinery (`modify`)

Preserve the independently useful in-tree build target, no-mid-dispatch-install
fact, and deferred cache posture. Recast bwrap from a codex/pi enhancement into
the measured Linux capsule backend. Retire the nominated-unjailed orchestrator
mechanism and ADR-008's `worker_commit` note only at capsule cutover. Do not state
or imply that macOS Seatbelt is already selected or measured.

Egress allowlisting and non-Git build-input provisioning do **not** land here;
DEC-129 and IMP-397 own that separate follow-on.

### ADR-011 — incumbent until cutover, then superseded for dispatch (`modify`)

ADR-011 remains present-tense authority while the Claude arm uses an in-session
`Agent`, disk marker, hook stamp, and gated worker commit. The target capsule ADR
will replace its dispatch authority with uniform headless subprocess launch and
OS-boundary identity. Add the eventual supersession/status row only when that
target ADR exists and the implementation cutover is ready; scoping does not make
shipped history false early.

### ADR-012 — keep admission and CAS, replace coordination topology (`modify`)

Preserve candidate admission by immutable OID, explicit operator resolution,
two-stage audit gating, journal-before-mutation, expected-tip CAS, and
report-never-auto-resolve. Replace coordination-worktree and mandatory
`dispatch/*`/per-phase-ref assumptions only where the new admission journal and
human-facing evidence views provide the same durable guarantees. The exact
population of `review/*` and `phase/*` remains a target-design question, not a
DELETE-by-count conclusion.

## Cutover and apply gates

Before this Revision may be approved or applied:

1. QUE-200, QUE-201, and QUE-202 are settled and their decisions are reflected
   in the target ADR/spec and requirement statements.
2. The new capsule ADR and technical container exist and are included in the
   Revision's touched set.
3. A planned implementation proves uniform headless worker launch, explicit
   contracted bases, Linux/bwrap confinement, trusted-side ingestion,
   separate-capsule verification, normalization, durable admission journaling,
   and crash-safe admission/integration.
4. The implementation names the exact compatibility/cutover point at which no
   dispatch run can still depend on marker, hook, `worker_commit`, worktree
   import, or coordination-worktree mechanisms.
5. Every current requirement listed above receives an explicit keep, transform,
   retire, or solo-scoped disposition; no mechanism is retired solely because a
   census row says DELETE.
6. The final prose keeps Linux evidence and macOS outstanding work visibly
   separate.

## Deliberately outside this Revision

- macOS sandbox-backend selection and re-measurement;
- IMP-397 / QUE-204 egress allowlisting and non-Git build inputs;
- retention duration, quotas, or project/slice/machine policy hierarchy beyond
  DEC-133/DEC-137's live-work/journal/exhibit separation;
- automated capacity reservation, throughput backpressure, capsule eviction,
  and a separate content-addressed rescue archive;
- migration of solo `/execute` worktrees to capsules;
- production optimisations such as overlays, snapshots, reflinks, shared caches,
  or remote execution; and
- implementation itself — this Revision is the governance boundary that a later
  slice must satisfy, not the slice.

## Drafting sources

- RFC-025 and its mechanism census, go/no-go, red-team, and probe evidence;
- DEC-133 (durable admission journal versus expiring forensic exhibit);
- DEC-134 (persistent control-plane orchestration, fresh headless phase worker);
- DEC-135 (bundle ingestion and its parent-side protocol);
- DEC-136 (`doctrine.toml` interpretation policy and contract refinement);
- DEC-137 (same-base candidate admission, frozen source capsules, and fresh
  repair transactions);
- DEC-129 and IMP-397 for the explicit egress/build-input exclusion; and
- the settled outputs of QUE-200, QUE-201, and QUE-202.
