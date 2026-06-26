# Trinary actionability

> **PARKED (`proposed`, 2026-06-26).** Design surfaced that the **gating-edge
> mechanism** (change 2 below) is not slice-shaped — it is an unsettled graph-model
> decision reopened by RFC-003's "structure ≠ graph-effect" ruling, routed to
> **RFC-008** for deliberation. The partition half (change 1) is settled. This slice
> resumes once RFC-008 chooses a mechanism (and likely emits an ADR). See
> `notes.md` for the engine-seam findings and the requirement RFC-008 must honour
> (*association ≠ gating*).

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

**Two coupled changes.**

1. **Third status-class** in `priority::partition` / `priority::channels` —
   provisionally `Gating` (name is OQ-1: `Gating` / `Ambient` / `Pending`) — that
   splits the two predicates the binary model fused:
   - `eligible` (appears in `survey`/`next` as work) → `Workable` only;
   - `blocks` (gates dependents via the `dep` overlay) → non-`Terminal`, i.e.
     `Workable ∪ Gating`.
   A `Gating` node blocks its dependents but never appears in the actionable
   worklist; settling to `Terminal` stops the gating and unblocks the dependent.
   The partition invariant (`workable ∪ terminal == vocab`) generalises to a
   three-way cover. For knowledge records, **no state is ever `Workable`**:
   unsettled → `Gating`, settled → `Terminal`.

2. **Gating edge into the `dep` overlay** — **DEFERRED TO RFC-008.** A record's
   unsettled state must gate the work it affects, with the record as dep
   predecessor (blocker), the B→A flip `needs` uses. *How* that edge is modelled is
   the open question RFC-008 owns: projection over the existing `Shapes` relation
   vs a distinct `gates` axis vs a `Gates` label (RFC-003 disfavours the last —
   gating is consumer graph-effect, not vocabulary). The IMP-047 "new
   `RelationLabel` + `RELATION_RULES` rows" sketch is **superseded** by that
   deliberation. Requirement: *association ≠ gating* (RFC-008). Direction
   (outbound-from-record vs dependent's `needs → record`) is RFC-008 D-c.

**Canon-first.** A SPEC-001 / PRD-011 D-decision + requirement for the third class
and the `eligible`-vs-`blocks` split land before the engine code — design drives
this, reconcile writes the spec.

**Objective:** a record (and any non-workable gating kind) can block downstream
work via the actionability graph without ever appearing in `next` as work; the
binary `Workable | Terminal` partition becomes a three-way cover; existing
behaviour for ordinary workable/terminal items is preserved.

## Non-Goals

- **Legibility of `next`/`survey`** (RFC-007 workstream 2) — folding `explain`,
  `--why`, what-if trace. Separate slice(s).
- **Epistemic-record authoring ergonomics** (RFC-007 workstream 3). Separate.
- **Risk-modelling expansion** — deferred, not blocking.
- **General cross-kind dep/seq capture (IMP-033)** beyond what gating edges need —
  this slice rides the shared "non-backlog kinds in the dep overlay" machinery but
  does not own IMP-033's full scope. Coordinate, don't absorb.
- **Backlog actionability mask (IMP-026, SPEC-001 D6)** — adjacent, separate.

## Affected surface (coarse — `/design` refines)

- `src/priority/partition.rs` — third class + generalised cover invariant
- `src/priority/channels.rs` — `eligible` vs `blocks` split; `consequence` label set
- `src/priority/graph.rs` — overlay allocation for the gating edge
- `src/priority/{surface.rs,view.rs,render.rs}` — worklist must exclude `Gating`
- `src/relation.rs` — new gating `RelationLabel`(s) + `RELATION_RULES` rows
- `.doctrine/spec/tech/001/` (SPEC-001) + PRD-011 — canon D-decision + requirement
- `.doctrine/spec/tech/019/` (SPEC-019) — D7 / NF-003 / OQ-2 revised: records become
  `Gating`, not inert (consumer of this mechanism)

## Risks / Assumptions / Open Questions

- **Behaviour-preservation gate.** The priority engine is shared machinery; the
  existing suites are the proof and must stay green for ordinary workable/terminal
  items. The three-way cover must reduce to the old binary behaviour wherever no
  `Gating` node exists.
- **OQ-1** — name of the third class (`Gating` / `Ambient` / `Pending`). → RFC-008 D-e.
- **OQ-2** — gating edge authored outbound-from-record vs dependent's `needs →
  record` (outbound fits the relation seam / ADR-004). → RFC-008 D-c.
- **Gating mechanism (the keystone OQ)** — projection-over-`shapes` vs distinct
  `gates` axis vs `Gates` label. **Routed to RFC-008** (D-a/D-b). This slice is
  parked on its resolution.
- **Coordination** — shares dep-overlay machinery with IMP-033; sequence
  after/alongside, don't fork a parallel implementation.
- **Canon-moves-first ordering** — the SPEC-001/PRD-011 change is authored through
  design→reconcile, not hand-edited ahead of the engine.

## Summary

(to be filled at close)

## Follow-Ups

- RFC-007 workstream 2 (legibility: fold `explain` into `next`/`survey`, what-if).
- RFC-007 workstream 3 (epistemic-record authoring/lifecycle).
- Revisit IMP-026 (actionability mask) and IMP-033 (cross-kind dep capture) once
  the gating machinery exists.
