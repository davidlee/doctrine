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
- **No optimistic locking** — version-stamping change targets + approval-retraction
  on drift. Anti-grain (doctrine surfaces drift, never hard-rejects); the narrow
  pre-flight `from`-guard covers the one real silent-clobber (design.md §9).
- **No `move`-apply automation** — requirement membership move has no existing seam
  (`spec req link`/move is the deferred SL-015 follow-on); `move` rows stage but are
  surfaced-for-manual at apply, not auto-applied (design.md §4.5, F4).
- **No `introduce`/`create`-apply automation in v1** — `spec req add`/`spec new` are
  non-transactional CLI handlers (orphan risk against one-commit apply); these rows
  stage but are surfaced-for-manual at apply. Auto-apply returns once transactional
  `spec::add_requirement`/`spec::create_spec` engine helpers exist (design.md §4.5,
  external B1/B2).
- **No per-repo conduct config for Revision** — v1 bakes the `gate` default;
  extending ADR-009's slice-state `[conduct]` table to Revision is deferred.

## Design

**Drafted + internally and externally hardened** in `design.md` (2026-06-14, scope
**C** — full structured delta payload; SL-044 done). Status stays `design` — awaiting
final lock. All nine design questions resolved (design.md §8); internal pass
integrated (§11, F1–F8); external codex pass integrated (§12, B1–B4/M1–M3). Headlines:
one change-axis kind `REV-NNN`; multi-target `[[change]]` payload rows (rows are
edges, `TypedVerbOnly`) + `primary` display-hint; reciprocity derived via
`relation_graph`, surfaced on `inspect` (ADR-004 §3 — not `show`); backlog-style
lifecycle + separate `approval` field (apply-time checkpoint, invoker-blind);
**v1 apply auto-lands `status` rows only** (rides engine-callable
`requirement::set_status`, all-or-nothing pre-flight `from`-guard);
introduce/create/move/prose surfaced-for-manual; `done` ⇒ every row landed; REC
untouched; ADR-013 + work-like-predicate `+= REV` as PHASE-01.

## Summary

One standalone work-lifecycle kind, `REV-NNN`, change axis, with an optional
spec-targeted delta payload. Unifies IDE-003 and IDE-010; unblocks
governance→work dependency modelling (SL-060) and a future staged-approve
reconcile path (SPEC-002 / SL-044).

## Follow-Ups

- `/revise` skill + workflow integration (IDE-003 tail; PHASE-06 or follow-up).
- Transactional `spec::add_requirement`/`spec::create_spec` engine helpers →
  unlocks `introduce`/`create`-apply automation (external B2; design.md §4.5).
- First-class REV↔REC relation label (governed ADR-010 addition) if a query demand
  appears — v1 links them implicitly (external B4; design.md §4.6).
- `spec req link`/move membership-mutation verb → unlocks `move`-apply automation.
- Extend ADR-009 `[conduct]` to address Revision (per-repo approval config).
- IDE-002 durable region primitive → structured prose-body anchors (`after IDE-002`).
- Close IDE-003 and IDE-010 on land.
