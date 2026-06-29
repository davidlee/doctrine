# Implementation Plan SL-176: Finish Axis B — slices/drift retirement

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Retire the work→backlog noun-labels `slices` / `drift` into the universal relation
grammar, completing the Axis-B collapse SL-149 began for work→canon. Three dimensions
`slices` conflated split out: **provenance** → `references(originates_from)` (a rename +
widening of shipped `scoped_from`), **fulfillment** → new `fulfils` label, **completion**
→ a `{full, partial}` degree facet on `fulfils`. The old `slices` "addressed by" reading
becomes `fulfils`' derived inbound. Design is locked through two external adversarial
passes (`design.md` decision ledger + the two "external pass" review sections); this plan
sequences the mechanism.

Governance ratification (ratifying ADR, SPEC-018, `relation-vocabulary.md`, RFC-003 close)
is **deferred to reconciliation** by locked decision — it is the design's "P5" and is NOT a
plan phase. The four phases below are implementation only.

## Sequencing & Rationale

The order is the SL-149 template (the exact prior move): **vocabulary/engine → storage →
surfaces → migration**, retain-then-cut on the dropped label. Each phase ends green; the
behaviour-preservation gate (entity-engine machinery suites green unchanged) holds across
all four.

- **PHASE-01 (engine)** lands the pure layer — `Degree`, the `Role` rename + widening,
  `Fulfils`, the `degree_bearing` column — with no storage or surfaces, so the lockstep VT
  family (enum order, accessor census, inbound names, table invariants) gates the contract
  before anything reads it. `Slices` is retained here; it is dropped only at migration. The
  one wrinkle the design flagged (G3): widening `originates_from` deliberately FLIPS shipped
  rule-contract tests (a backlog item may now author it; the target set admits SL). Those are
  **content**, not regressions — rewritten in this phase, not discovered later.

- **PHASE-02 (storage)** threads `degree` through the row/edge and resolves the two codex
  blockers about the write seam. There is **no upsert** (F1): `append_relation_row` is
  append-or-Noop, so degree is set once at author time and changed via unlink+relink; a
  conflicting-degree re-link is a hard error, the single honest extension to the seam.
  Uniqueness (F2/G2) is enforced **locally at `read_block`** — edge identity's `source` is
  one entity, so a duplicate logical edge lives in one toml; a per-entity `DuplicateEdge`
  check needs no corpus scan and no degree thread into `CatalogEdge`. The phase's gate is
  EX-5: the roleless/degreeless append path stays a strict no-op superset, so existing
  suites stay green unchanged.

- **PHASE-03 (surfaces + the blast radius)** is the largest phase and carries the two codex
  findings that grew the scope. F3: degree cannot ride a render-side side-index — the inspect
  target type itself changes (`Vec<String>` → `Vec<RelationTargetView>`), rippling outbound,
  inbound, and `--json` (heterogeneous-by-label: objects for `fulfils`, bare strings else).
  F4/F5: retiring/renaming a label is **never vocab-table-local** — live consumers in
  `priority/graph.rs` (optionality scoring), `backlog.rs` (show/json/lifecycle),
  `lazyspec.rs` map_edge, and the `commands/relation.rs` role-error string all read the old
  vocabulary. The load-bearing one is the priority re-point (R10): the optionality numbers
  must stay identical (a `fulfils` inbound credits exactly as a `slices` reference did). And
  G1(a): because the migration moves the edge to the slice end, `backlog show` can no longer
  read it from the item's own toml — it gains a derived-inbound read-path (the same machinery
  `inspect` uses), becoming corpus-aware. EN-2 re-greps the full consumer census before
  starting, because a missed consumer is a silent behaviour change.

- **PHASE-04 (migration)** is the hard cut. It is **editorial, not mechanical**: the
  prov-vs-fulfil split and the per-edge degree are human judgements recorded in a committed
  `migration-dispositions.md`, and the automated oracle proves only that those dispositions
  are faithfully APPLIED — never that they are correct (codex F6). That is why VH-1 (human
  review of the artifact) sits alongside the VT oracle. The transform is a single atomic
  apply (no valid intermediate state, since code and corpus must never disagree); the parser
  change dropping `Slices`, the rewritten corpus, and the scaffold templates land in one
  commit. Class 3 is the novel mechanic SL-149 lacked — ~63 edges *relocate* backlog→slice
  toml (a direction flip), not relabel in place.

## Notes

- **Re-census at execution, not from the plan.** Counts (`slices` 82, `scoped_from` 19,
  `drift` 7) are a 2026-06-29 snapshot; concurrent `main` authoring keeps minting fallout.
  PHASE-04 EN-2 re-snapshots live. The IMP-207 19-row list is reference, not input.
- **Residual risks carried from design (not blocking):** R9 — `originates_from` author-end
  is convention, not enforced (a reverse-authored edge is not kind-catchable); cleaned at
  reconcile if the dogfood census surfaces any. R11 — the `RelationTargetView` type ripples
  every inspect path; mitigated by `skip_if None` (degreeless byte-identical) but the
  signature churn is real.
- **Reconciliation (post-execute) owns:** the ratifying ADR (amend ADR-016/ADR-010 or a
  sibling), SPEC-018 + `relation-vocabulary.md` updates, RFC-003 close, and discharging
  IMP-207 / IMP-149. Follow-ups already captured: IMP-210 (close-cascade hint consumer),
  IMP-156 (create-time `--originates-from` flag).
