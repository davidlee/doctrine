# Revision entity: pending revise-intent and staged-delta vehicle

## Context

Two backlog items name the same gap from two angles:

- **IDE-003** (from SL-042 reconcile substrate) — a vehicle to *stage and
  approve* deltas to requirement status and spec prose before they land in the
  authored tier. Distinct from REC (the immutable record of a reconciliation
  *act*) and from RV (the adversarial review ledger, SL-040). Heritage:
  spec-driver's `RE-NNN` spec-change record.
- **IDE-010** (from SL-060 cross-kind dep/seq) — a first-class *work-lifecycle*
  entity capturing pending intent to revise a governance target (spec/ADR/POL/
  STD), so a slice can `needs` the Revision (work→work, in-axis) instead of
  `needs`-ing the governance doc (work→evergreen, gradient-inverting). This is
  why specs/ADRs were deliberately kept OUT of SL-060's `needs`/`after` source
  set.

These are **one entity at two lifecycle lenses**, not two kinds: a Revision is
born as content-light pending intent (IDE-010 — dependents anchor on it
immediately), accumulates staged deltas as it is worked (IDE-003), is approved on
the ADR-009 conduct axis, then applied (deltas land) and settled. IDE-003's
`draft→approved→applied` and IDE-010's `proposed→in-progress→done` are the same
lifecycle, content-lens vs work-lens.

Today: `REV-` is a *deferred* spec subtype (`doc/spec-entity-spec.md`), and
`doc/entity-model.md` §Adjudication leaves its home **open**, nudging it
change-side. This slice closes that question.

## Scope & Objectives

Introduce **Revision** as a standalone work-lifecycle entity kind:

- **Home** — own kind on the change axis (peer to slice/REC), NOT a spec subtype
  and NOT a slice facet. Own folder; prefix `REV-NNN` (`RE-` was spec-driver).
  Relocates the deferred `REV-` off the spec family; updates
  `doc/entity-model.md` / `doc/spec-entity-spec.md` accordingly.
- **Spine** (always present) — work lifecycle + a target edge to the governance
  entity it proposes to revise (spec/ADR/POL/STD); gates dependents; settles when
  the revision lands. Composes with IMP-047 `Gating` actionability.
- **Payload** (optional, spec-targeted only) — staged requirement-status and
  spec-prose deltas, in a sister `revision-NNN.toml` (TOML tables, never an
  embedded YAML block — storage rule); prose rationale in `revision-NNN.md`.
  ADR/POL/STD revisions carry intent + eventual prose diff, no structured payload.
- **Wiring** — the standard new-authored-entity seam (KINDS row, manifest dir,
  gitignore negation, render/show, `doctrine revision new`), per
  `mem.pattern.install.authored-entity-wiring`.

Relations authored with `doctrine link`: `governed_by` ADR-003/ADR-009/ADR-010,
`specs`/relates SPEC-002. Backlog provenance (IDE-003, IDE-010) recorded as
promotion.

## Non-Goals

- **No change to REC's shipped contract** (status-less, immutable, SL-042). REC
  composition with Revision is loose and settled in design (#1 below).
- **No reconcile-writer rewrite.** SL-044's direct `ReqStatus` write stays; how/
  whether `revise` routes through a Revision is a design question (#2), not a
  forced cutover here.
- **No `/revise` skill or `reconcile --draft` flag** in this slice's spine —
  workflow integration is downstream once the kind exists (#7 of the parcel).
- **No supersession machinery** — a Revision is *pending* intent; the completed
  lifecycle flip is a different thing.

## Open Design Questions (carry into /design)

Both upstream gates are settled (one kind; standalone `REV-NNN`, change axis,
optional payload). Remaining, design-internal:

1. **REC composition** — applied Revision `produces`/`recorded_by` a REC, or REC
   references Revision, or independent. (Lean: loose edge, REC untouched.)
2. **Reconcile-writer path** — does `revise` route through Revision (draft→
   approve→apply) or sit beside the direct path as higher-ceremony opt-in?
3. **Storage shape of the delta payload** — TOML table schema for
   requirement-flow / lifecycle / text-changes; reuse existing edges vs a
   Revision-specific vocabulary.
4. **Conduct-axis defaults** — gated vs solo self-approve by default (lean: solo
   self-approve, gating opt-in).
5. **Altitude** — does the taxonomy-home call (Revision as a change-axis kind,
   off the spec family) warrant an ADR amendment, or is the slice + entity-model
   edit sufficient?

## Summary

One standalone work-lifecycle kind, `REV-NNN`, change axis, with an optional
spec-targeted delta payload. Unifies IDE-003 and IDE-010; unblocks
governance→work dependency modelling (SL-060) and a future staged-approve
reconcile path (SPEC-002 / SL-044).

## Follow-Ups

- Wire `needs`/`after` to accept `REV-` targets (the SL-060 enabler).
- `/revise` skill + `reconcile --draft` workflow integration (IDE-003 tail).
- Close IDE-003 and IDE-010 on land.
