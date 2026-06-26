# Trinary actionability

> **RESUMED + DESIGNED (2026-06-26) — RFC-008 resolved → ADR-017.** Gating is settled:
> **an inbound `needs` dependency edge on an unsettled record**, not a new
> relation/role/axis. There is no "gating-edge" to build.
>
> **Design correction (this pass).** ADR-017's premise that the `needs` work-like gate
> is *source-only* (so a work item "may target a record today") is **false in the
> current code**: `commands/dep_seq.rs` gates the **target** as work-like too, so
> `doctrine needs SL-x QUE-1` is refused today. The trinary partition is therefore
> **not** the sole engine delta — the target-admissibility gate must also widen. See
> Design Decisions below; ADR-017 prose reconciled at close.
>
> **Locked scope (three changes):** (1) **trinary partition** — unsettled record →
> non-`Terminal` `Gating` class; (2) **target-gate widening** — `needs`/`after`
> admissible targets = work-like ∪ records (source gate unchanged); (3) **estimate/value
> on records** — confirmatory design note only (already kind-agnostic; no code).
>
> **Split out:** the **`shapes`-roles** piece (semantic disambiguation, ADR-016) →
> **IDE-022** — different layer, carries its own open question. Estimate/value
> show/inspect surfacing → **IMP-183**. Outbound authoring stays a derived hub-view +
> deferred batch sugar (ADR-017 §3). See ADR-017 + `design.md`.

## Context

Source: **IMP-047**. Keystone of **RFC-007** workstream 1 (correctness).

The priority engine partitions status binary — `Workable | Terminal`
(`src/priority/partition.rs`). `blocked_by` counts a predecessor only when its
class ≠ `Terminal`; `actionable = eligible ∧ ¬blocked`; `eligible = Workable`. So
**"can block"** and **"is itself actionable work"** are fused into the one
non-terminal class.

Knowledge / governance records (PRD-010 / SPEC-019) are the forcing case. Records
are never work (*truth is not work*), so SPEC-019 D7 parks every record state as
`Terminal` — graph-present but actionability-inert — which also means a record can
**never block**. SPEC-019 D7 / NF-003 / OQ-2 record this as a known limitation
explicitly awaiting IMP-047. But the intended expressions are blocking ones:

- a **question** (`QUE`) gates the design of a slice while `open`;
- an **assumption** (`ASM`) gates an idea while `held`/`testing`;
- a **constraint** (`CON`) gates a requirement / slice while `active`;
- a **decision** (`DEC`) gates an issue while `proposed`.

In each case the record gates a downstream item but is itself not the work — the
dependent unblocks when the record reaches a terminal state.

This is a **model change**: PRD-011 / SPEC-001 canon moves first (a D-decision +
a requirement), then the engine.

## Scope & Objectives

**Three changes** (design.md holds the detail).

1. **Third status-class** in `priority::partition` — `Gating` (name settled, ADR-017):
   non-`Workable`, non-`Terminal`. It splits the two predicates the binary model
   fused: `eligible` (worklist) stays `== Workable`; `blocks` (gates via `dep`
   overlay) is `!= Terminal`, i.e. `Workable ∪ Gating`. **`channels.rs` needs no
   code change** — both predicates already read the right poles; the new variant
   slots in. A `Gating` node blocks dependents but never surfaces as work; settling
   → `Terminal` unblocks (for free). Per-kind settle boundary: ASM `held`/`testing`,
   DEC `proposed`, QUE `open`, CON `active` are `Gating`; their settled states
   `Terminal`. The VT-1 canary generalises to `workable ∪ gating ∪ terminal == vocab`.

2. **Target-admissibility gate widening** in `commands/dep_seq.rs`. Split the one
   `is_work_like` predicate into a **source** gate (unchanged — records still can't
   *author* dep/seq; ADR-017 §3) and a **target** gate `is_admissible_dep_target` =
   work-like ∪ records (ASM/DEC/QUE/CON). Governance (SPEC/ADR/POL/STD) stays
   excluded — depending on canon routes through a Revision. record→record `needs` is
   excluded for free (source gate). This is what makes `needs → <record>` authorable
   — the edge then rides the existing kind-agnostic `graph.rs` build untouched.

3. **estimate/value on records — confirmatory note (no code).** Already kind-agnostic
   (`estimate set ASM-001` works; round-trips — `RawRecordToml` has no
   `deny_unknown_fields`). Currently **inert** for scoring (a record's base doesn't
   propagate — leverage flows dependent→prereq, records have no dep predecessors). A VT
   pins the round-trip. `risk` stays excluded (kind-gated + `[facet]` table-name collides
   with knowledge's typed kind-facet). Surfacing in show/inspect → IMP-183.

**Canon-first.** A SPEC-001 / PRD-011 D-decision + requirement for the third class
and the `eligible`-vs-`blocks` split land before the engine code — design drives
this, reconcile writes the spec.

**Objective:** a record can block downstream work via `needs` without ever appearing
in `next` as work; the binary `Workable | Terminal` partition becomes a three-way
cover; existing behaviour for ordinary workable/terminal items is preserved.

## Non-Goals

- **Legibility of `next`/`survey`** (RFC-007 workstream 2) — folding `explain`,
  `--why`, what-if trace. Separate slice(s).
- **Epistemic-record authoring ergonomics** (RFC-007 workstream 3). Separate.
- **Risk-modelling expansion** — deferred, not blocking.
- **General cross-kind dep/seq capture (IMP-033)** beyond what gating edges need —
  this slice rides the shared "non-backlog kinds in the dep overlay" machinery but
  does not own IMP-033's full scope. Coordinate, don't absorb.
- **Backlog actionability mask (IMP-026, SPEC-001 D6)** — adjacent, separate.

## Affected surface (design-target)

- `src/priority/partition.rs` — `Gating` class + `gating` set on `KindPartition` +
  generalised three-way cover canary; per-kind knowledge settle boundary.
- `src/commands/dep_seq.rs` — source/target gate split; `is_admissible_dep_target`
  (work-like ∪ records); refusal message; refusal/admission tests.
- `.doctrine/spec/tech/001/` (SPEC-001) + PRD-011 — canon D-decision + requirement
  (third class; `eligible`-vs-`blocks` split; records-as-`needs`-targets).
- `.doctrine/spec/tech/019/` (SPEC-019) — D7 / NF-003 / OQ-2 revised: records become
  `Gating` (unsettled) / `Terminal` (settled), not all-inert.

**Explicitly NOT touched** (the elegance of ADR-017): `channels.rs` (predicates
already correct), `graph.rs` (kind-agnostic `needs` build already emits the edge),
`surface.rs`/`view.rs`/`render.rs` (`Gating` excluded via `eligible == Workable`,
no worklist change), `relation.rs` (shapes-roles split to IDE-022).

## Risks / Assumptions / Open Questions

- **Behaviour-preservation gate.** The priority engine is shared machinery; the
  existing suites are the proof and must stay green for ordinary workable/terminal
  items. The three-way cover must reduce to the old binary behaviour wherever no
  `Gating` node exists. Two existing knowledge canary tests **flip by design**
  (consumer revision, not regression): `every_knowledge_status_classifies_terminal_never_workable`
  and the `knowledge_partitions_cover_the_real_vocabularies` canary form.
- **ADR-017 premise correction** (RESOLVED this pass) — target gate is NOT source-only;
  change 2 (gate widening) restores ADR-017's intent. ADR-017 prose reconciled at close.
- **OQ-1 (name)** — `Gating` adopted (ADR-017; cosmetic). Closed.
- **Gating mechanism** — settled by ADR-017 (inbound `needs`). Closed.
- **Coordination with IMP-033** — gate widening overlaps IMP-033's cross-tier dep/seq
  scope. SL-158 widens *only* to records (not full cross-tier); coordinate, don't fork.
- **Canon-moves-first ordering** — the SPEC-001/PRD-011 change is authored through
  design→reconcile, not hand-edited ahead of the engine.
- **Assumption (low risk, VT-pinned)** — records are scanned graph nodes and
  `ensure_ref_resolves` accepts `.doctrine/knowledge/...` (existing graph.rs tests
  seed knowledge trees; admissibility VT confirms `needs → QUE` resolves + emits).

## Summary

(to be filled at close)

## Follow-Ups

- **IDE-022** — `shapes`-roles (semantic disambiguation, ADR-016), split from this slice.
- **IMP-183** — surface estimate/value in show/inspect for all estimable kinds.
- ADR-017 prose reconciliation (the source-only premise correction) — at close.
- Outbound gating hub-view + deferred batch sugar (ADR-017 §3).
- RFC-007 workstream 2 (legibility: fold `explain` into `next`/`survey`, what-if).
- RFC-007 workstream 3 (epistemic-record authoring/lifecycle).
- Revisit IMP-026 (actionability mask) and IMP-033 (full cross-kind dep capture).
