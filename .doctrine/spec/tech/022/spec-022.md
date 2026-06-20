# SPEC-022: Git interaction model

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The git interaction model is the **object-and-ref substrate** that dispatch stands
on. It sits beneath the whole-system root (SPEC-003) as a sibling container to the
dispatch mechanism (SPEC-012) and the orchestrator process (SPEC-021), and realises
the durable-output half of **PRD-015**. ADR-006 (worktree posture), ADR-011
(harness-agnostic spawn), and **ADR-012** (dispatch integration topology) are its
governing decisions.

It exists because the dispatch/candidate/review/integrate machinery was built
incrementally — many sound but disjoint local decisions, with no single authoritative
description of the *intended* model expressed against git's actual object + ref model
(objects, refs, merge-base, CAS, worktrees). The witnessed cost of that absence is
RV-116 (SL-127 audit): the audit skill says "repair on the candidate interaction
branch" while `/close` sources the close target from `review/<N>` — the two reconcile
only under sourcing assumptions nobody had written down. This spec writes the model
down so future slices touching dispatch stop rediscovering the same ambiguities.

**Boundary with its siblings.** SPEC-012 owns the *verb mechanism* (provision / fork /
import / land / gc, the worker-mode guard, the born-frame git seam) and SPEC-021 owns
the *process that wields them* (the funnel cadence, routing, the per-harness altitude,
the operational gotchas). This spec owns neither. It owns the **git-level contracts
those layers both depend on**: which refs exist and what each one *is*, which are
mutable and which are immutable evidence, how a projection provably and crash-safely
reaches trunk, and why the candidate layer is shaped the way it is. Verb internals are
cited here, never restated — where SPEC-012/021 already describe a mechanism, this spec
gives the object-model *rationale* underneath it and points at the owner.

The model is **retrospective**: it describes shipped behaviour (SL-064, SL-068, SL-121,
SL-126, SL-127), verified against `src/git.rs`, `src/dispatch.rs`, `src/ledger.rs`, and
`src/worktree.rs`. Requirements stay `pending`; coverage is reconciled, never inferred.

## Responsibilities

Mirrors the structured `responsibilities` list. What follows is the model, not the
mechanism: the ref taxonomy and its mutability classes, the pinned fork-point, the
two-stage projection, the CAS journal, the candidate layer, the propagation contract,
and the object-db ledger sourcing.

### The ref taxonomy and its two mutability classes

Every branch the system creates falls into one of two classes, and the class is the
load-bearing fact about it.

**Mutable refs** — advanced in the normal course of a run:

- `dispatch/<N>` — the **coordination single-source-of-truth**, living in its own
  always-on isolated worktree. The funnel's sole write target; advanced by every batch
  commit and by every journal commit. Durable across handover-resume; concurrent
  same-slice dispatch is refused at creation. Worktree-life < branch-life — the
  worktree is reaped at conclude, the branch is kept (SL-064).
- `candidate/<N>/<label>` — an **interaction branch**: an ordinary mutable git ref that
  is the *safe audit/repair surface* (SL-068). Its tip may drift; the **recorded OIDs
  are immutable** (see the candidate layer below).
- `edge` — the optional standing aggregate of all local work; contention lives only
  here, only at conclude.
- trunk (`main` / `master` / `origin/HEAD`) — foreign-owned; advanced only by
  `integrate --trunk` under fast-forward CAS.

**Immutable evidence refs (R2)** — created exactly once, never rewritten:

- `review/<N>` — the impl-bundle review unit (code + knowledge, `.doctrine/dispatch/<N>`
  and verified-orthogonal paths filtered out).
- `phase/<N>-NN` — the per-phase code cut (`.doctrine/` stripped), chained off the prior
  phase from the pinned fork-point.

Both evidence refs are created under **zero-oid CAS**: the create succeeds only if the
ref does not yet exist. A stale evidence ref from a prior run is **reported and the
command fails — never clobbered**. This is what makes them trustworthy audit inputs:
an evidence ref, once published, is a fixed object an auditor can re-derive and rely on.
Treating an evidence ref as an ordinary review branch and fixing it in place is the
SL-067 trap the candidate layer was built to close.

### The pinned fork-point invariant (RV-030 F-1)

Stage-1 projections are parented on the **pinned fork-point**
`trunk_base_B = merge-base(dispatch/<N>, trunk)` resolved at `prepare-review` time — not
the live trunk tip. The reason is exactness: a foreign commit landing on trunk between
`coordinate` and `sync` must not silently reparent the projection and distort its diff.
The live trunk tip resurfaces only at integrate's actual trunk push, under CAS.

The fork-point advances by **one explicit recorded action only** — `refresh-base`, a
real `git merge --no-ff` of trunk into `dispatch/<N>` in the live coordination worktree
(SL-127). After it, the operator re-runs `prepare-review`, which recomputes the
merge-base against the advanced tip. There is no silent reparenting path; the advance is
a merge commit on the dispatch branch or it did not happen.

### The two-stage, audit-gated projection

Conclude does not push to trunk. It **projects outward in two stages**, with audit
between them (SL-064, ADR-012 D4/D5):

- **Stage-1 — `prepare-review`.** Materialise `review/<N>` + `phase/<N>-NN` and commit
  the CAS journal. **No trunk write.** Idempotent: a re-run re-pins the bundle to the
  current fork-point.
- **Audit** runs from the parent/root against the prepared refs (RV verbs refuse on a
  worktree fork). It gates the review units *before* they integrate.
- **Stage-2 — `integrate`.** Opt-in, post-audit. Fast-forward-only, expected-tip-CAS,
  reports a moved/non-ff target and halts — never force-pushes, never auto-resolves. A
  failed audit blocks trunk integration while preserving `dispatch/<N>`, `phase/*`, and
  `review/*` intact.

### The CAS journal recovery contract

Every projection is mediated by a **journal committed to `dispatch/<N>` before any
external ref mutation** (the `with_journaled_projection` bracket — owned as mechanism by
the dispatch container; the *contract* is here). Each row carries
`{target_ref, expected_old_oid, planned_new_oid, applied_new_oid, status}`. Replay is an
idempotent **3-way classification** against the live ref:

- `current == planned_new_oid` → **no-op** (already applied);
- `current == expected_old_oid` → **advance** to planned;
- otherwise → **refuse** and report a moved target.

The advance leg is **worktree-aware** (SL-121): a not-checked-out target advances by
pure `update_ref_cas` (with a post-CAS re-probe that resyncs a newly-checked-out ref);
a checked-out target advances by `merge --ff-only` in its worktree so ref, index, and
tree move together; a **non-ff advance on a checked-out ref is refused**
(`integrate-nonff-checkout`) rather than reset. A dirty checked-out target fails the
whole integrate before the first journal commit (`integrate-dirty-worktree`, zero refs
moved). Because intent is committed before action and replay is idempotent, a crashed
sync is recovered by simply re-running it.

### The candidate interaction layer (SL-068)

The candidate layer is the **safe interaction surface over immutable evidence**. An
evidence ref must not be edited in place; a `candidate/<N>/<label>` is an ordinary
branch where audit and repair happen freely. Three roles, distinguished by what they
feed: `review_surface` (the audit input, default-sourced from `review/<N>`),
`close_target` (the trunk payload, explicit `--source` required), and `scratch`.

Admission is **by immutable OID, never by ref**:

- `candidate create` performs a Doctrine no-ff 3-way merge of `--source` into `--base`,
  recording immutable `source_oid`, `base_oid`, and `merge_oid` (whose parents are
  exactly `base_oid` + `source_oid`).
- `candidate admit` validates the tip descends from `merge_oid` and pins `admitted_oid`.
- `integrate --trunk` targets `admitted_oid` — **never the live candidate tip**. If the
  candidate ref drifts after admission, `candidate status` reports it but the admitted
  OID is unchanged. If trunk has moved, integrate **refuses with guidance to create a
  superseding candidate** rather than merging at close time. There is no close-time
  merge: close only replays the journal.

### Run-ledger object-db sourcing and trunk resolution

The run ledger — `journal.toml`, `boundaries.toml`, `orthogonal.toml` under
`.doctrine/dispatch/<N>/` — is **tree-read from the `dispatch/<N>` branch tip**
(`read_path_at` against the object db), never the working filesystem, and identically in
stage-1 and stage-2. The same checkout-independent value is read everywhere; this is
what lets audit run from the root while the coordination tree is elsewhere.

Trunk is resolved by a peeled ladder `DOCTRINE_TRUNK_REF → origin/HEAD → main → master`
folded through **`freshest_descendant`** — advance only to a strict descendant, so a
stale `origin/HEAD` that is an ancestor of local `main` is overtaken, while a
genuinely-diverged candidate keeps ladder order (SL-127). At the `reconcile → done`
lifecycle crossing a structural backstop asserts
`is_ancestor(planned_new_oid, trunk_tip)` for the journal's trunk row — proving
*integration occurred* (not tree survival at tip) and fail-closing a slice that
projected but was never integrated (SL-126).

## Concerns

- **Crash-safety is the design centre.** Journal-before-mutation + idempotent 3-way
  replay means any sync is re-runnable; the model never force-pushes and never
  auto-resolves a moved target. Recovery is re-run, not repair.
- **Evidence trust by construction.** Zero-oid CAS creation + report-not-clobber is what
  makes `review/*` / `phase/*` reconstructable audit inputs; the candidate layer exists
  so that trust is never violated by an in-place fix.
- **`boundaries.toml` must be committed for phase cuts to materialise (ISS-039).** The
  object-db tree-read is correct by design; the live operational gap is that the funnel
  (claude arm) does not always commit `boundaries.toml` to `dispatch/<N>`, so the
  tree-read returns empty and `prepare-review` projects **0 phase cuts**. This is a
  funnel implementation gap, not a model defect; the model's constraint is stated here so
  the funnel can be held to it. Tracked as ISS-039.
- **The repair→integrate propagation burden is the operator's (see D1).** The most
  common way to lose a fix is to repair the review-surface candidate and then create the
  close-target from `review/<N>`.
- **Concurrency is bounded, not eliminated.** Documented residual races (dirt introduced
  mid-merge, labeling races on compatible concurrent advances, the None-leg
  untracked-collision risk) are reported and tested, not closed; CAS guarantees
  content-safety, not absence of contention (SL-121 §7, IMP-122).

## Hypotheses

- **Refs are the right unit of durable evidence.** Immutable, CAS-created branches are
  assumed to be a better audit substrate than ad-hoc patches or tags because they are
  reconstructable and diffable with ordinary git.
- **OID-pinned admission beats ref-tracking.** Targeting `admitted_oid` rather than a
  live tip is assumed safer than auto-propagation: it makes "what lands on trunk" an
  explicit, immutable choice at the cost of an operator propagation step.
- **A pinned fork-point is worth the staleness it admits.** Parenting on `trunk_base_B`
  rather than the live tip is assumed to be the right trade: exact diffs now, an explicit
  `refresh-base` later, over silent reparenting.

## Decisions

- **D1 — The repair→integrate propagation contract (the RV-116 resolution).** A repair
  committed on a review-surface candidate does **not** auto-flow to the close-target;
  this decoupling is *intentional*, a direct consequence of admit-by-OID. To land a
  candidate-branch repair on trunk, the operator must source the close-target from the
  repaired candidate (`--source refs/heads/candidate/<N>/<label>`), or cherry-pick the
  fix onto the close-target and re-admit. The `/close` default `--source review/<N>` is
  the legacy straight-through path: correct only when no repair happened on the
  candidate. Both paths are legitimate; the spec's job is to make the choice explicit so
  a fix is never silently dropped. (Born from RV-116; the model and the code already
  agree — the gap was that this was never written down.)
- **D2 — Sibling, not child of SPEC-012.** The git model is the *substrate* SPEC-012
  (mechanism) and SPEC-021 (process) both build on, so it is a sibling container under
  SPEC-003, not a component beneath the mechanism. Parenting it under SPEC-012 would
  invert the dependency (substrate under mechanism) and bury the ref taxonomy under verb
  prose. SPEC-012 and SPEC-021 carry outbound `uses` edges into this spec.
- **D3 — Cite mechanism, never restate it.** Where SPEC-012 owns a verb internal (the
  `with_journaled_projection` bracket, tree-filtering, fork/import/land/gc) or SPEC-021
  owns a process (the funnel cadence, the operational gotchas), this spec states the
  object-model contract and rationale and points at the owner. The boundary is: *what a
  ref is and why* lives here; *how a verb manipulates it* lives there.

## References

- ADR-006 (worktree posture), ADR-011 (harness-agnostic spawn), ADR-012 (dispatch
  integration topology) — governing decisions.
- SPEC-012 (dispatch & worktree mechanism), SPEC-021 (orchestrator process) — sibling
  containers that build on this model.
- SL-064 (coordination-branch isolation + integration-sync), SL-068 (dispatch
  candidates), SL-121 (worktree-aware integrate), SL-126 (structural close-gate),
  SL-127 (dispatch base freshness) — the slices that shipped the model.
- RV-030 F-1 (pinned fork-point), RV-116 (the repair→integrate gap that bore this spec),
  ISS-039 (boundaries.toml not committed), IMP-122 (None-leg race), IMP-127
  (split-lineage hand-resolve), IMP-128 (this spec's origin).
