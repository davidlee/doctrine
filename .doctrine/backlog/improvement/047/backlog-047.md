# IMP-047: Trinary actionability: decouple blocking from eligibility so unsettled records (and kinds) gate work without being actionable

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Why

The priority engine (PRD-011 / SPEC-001, shipped SL-046/047) uses a **binary**
status partition: `Workable | Terminal` (`priority/partition.rs`). `blocked_by`
counts a predecessor only when its class ≠ `Terminal`; `actionable = eligible ∧
¬blocked`; `eligible = Workable`. So "can block" and "is itself actionable work" are
**coupled** — the only non-terminal class is `Workable`, which is also `eligible` and
lands in `next`. There is no class for *"this gates work but is not itself something
you pick up."*

Knowledge records (PRD-010 / SPEC-019) are the forcing case. They are never work
(*truth is not work*), so SPEC-019 D7 currently parks every record state as
`Terminal` — graph-present but actionability-inert — which also means a record can
**never block**. SPEC-019 OQ-2 records that limitation. But the intended expressions
are exactly blocking ones:

- a **spike** should answer this **question** (`QUE`), which needs answering to
  unblock the **design of a slice** — `QUE` gates `SL` while `open`;
- an **assumption** (`ASM`) should be verified before we address that **idea**
  (`IDE`) — `ASM` gates `IDE` while `held`/`testing`;
- a **constraint** (`CON`) prohibits meeting a **requirement** (`REQ`) or executing a
  **slice** (`SL`) while it holds — `CON` gates while `active`;
- a **decision** (`DEC`) needs to be made before we can pick up an **issue** (`ISS`) —
  `DEC` gates `ISS` while `proposed`.

In every case the record gates a downstream item but is itself not the work — the
work is the spike / the verification / the deciding. The dependent unblocks when the
record reaches a terminal state (`answered` / `validated` / `waived`/`retired` /
`accepted`|`rejected`).

## What

Two coupled changes:

1. **A third status-class** in `priority::partition` / `priority::channels` —
   provisionally `Gating` (name TBD: `Gating` / `Ambient` / `Pending`) — that splits
   the two predicates the binary model fused:
   - `eligible` (appears in `survey`/`next` as work) — `Workable` only;
   - `blocks` (gates dependents via the `dep` overlay) — non-`Terminal`, i.e.
     `Workable ∪ Gating`.
   A `Gating` node blocks its dependents but never appears in the actionable worklist;
   when it settles to `Terminal` it stops gating and the dependent unblocks. The
   partition invariant (`workable ∪ terminal == vocab`) generalises to a three-way
   cover. For knowledge records, **no state is ever `Workable`**: unsettled →
   `Gating`, settled → `Terminal`.

2. **A gating edge into the `dep` overlay.** The record authors an outbound
   `gates`/`blocks` edge to what it affects (PRD-010 §3 "a record shows what it
   affects"); cordage allocates it to the `dep` overlay with the record as predecessor
   (blocker) and the target as successor (blocked), the same B→A flip `needs` uses.
   This is the `RelationLabel` + `RELATION_RULES` + overlay-allocation extension
   SPEC-019's relation seam already flags — here with the priority-graph semantics
   attached. (Whether the edge is authored on the record outbound, or as the
   dependent's `needs → record`, is a design call; outbound-from-record fits the
   relation seam and ADR-004.)

The spawned-work path stays valid and composes: the spike that answers `QUE` is still
a backlog item that `QUE` links to (PRD-010 §6); now `QUE` *also* gates the slice
directly, so the graph shows both "what work answers this" and "what this blocks."

## Scope / coordination

- Priority engine: `priority/partition.rs` (third class + invariant), `channels.rs`
  (`eligible` vs `blocks` split, `consequence` label set), `graph.rs` (overlay
  allocation), and the SPEC-001/PRD-011 canon (a D-decision + a requirement; this is a
  **model change**, so the canon moves first).
- Relation contract: new gating `RelationLabel`(s) + `RELATION_RULES` rows for the
  record source-group and any other gating source (this overlaps SPEC-019's relation
  extension and **IMP-033** cross-kind dep/seq capture — shares the "non-backlog kinds
  in the dep overlay" machinery; sequence after / alongside it).
- Consumer: SPEC-019 D7 / NF-003 / OQ-2 are revised by this — records become `Gating`,
  not inert. Update SPEC-019 once the mechanism lands (kept as the target-intent note
  meanwhile).

Motivated by PRD-010 / SPEC-019; changes PRD-011 / SPEC-001; overlaps IMP-033,
relates to IMP-026 (backlog triggers actionability mask, SPEC-001 D6).
