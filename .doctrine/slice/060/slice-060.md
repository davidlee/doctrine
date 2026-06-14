# Cross-kind dep/seq capture: extend needs/after authoring beyond backlog

Realises **IMP-033**. Surfaced during SL-046 design; deferred there as a
different semantic layer (capture-side schema change, not the reference/lineage
reader).

## Context

The dep/sequence axis — hard `needs` (item→item prerequisite, payload-free) and
soft `after` (item→item sequence, per-edge `rank`) — is **authored only on
backlog items today** (`src/backlog.rs` `Relationships{needs, after, triggers}`;
verbs `backlog needs` / `backlog after`). The intent, though, is cross-kind: a
content-schema spec genuinely *needs* a storage-layer spec; coarse delivery
sequencing across slices wants an *after* preference. There is no surface to
author either.

Two facts fix the boundary of this slice:

- **These are NOT relation-vocabulary edges.** ADR-010 D1 explicitly carves the
  dep/sequence axis **out** of the unified relation contract (`RELATION_RULES` /
  the `link` writer). `needs`/`after` are typed Tier-2 **payload** edges
  (`after` carries `rank`), deliberately kept off the generic `[[relation]]`
  block. So this slice does **not** extend `link`/`RelationLabel`; it extends a
  separate typed per-kind schema + its own verbs.
- **Capture is the gap; the consumer is not yet honest either.** PRD-009 owns the
  dep/seq **capture surface** (FR-010 `needs`/`after`, REQ-097); SPEC-001 D4
  consumes it as `dep`/`seq` cordage overlays. But the only projector,
  `src/backlog_order.rs`, keys nodes on `ItemKind` (the five backlog kinds) and
  SL-046's all-entity relation reader (`outbound_for` / `integrity::KINDS`)
  **excludes** `needs`/`after`/`triggers` (ADR-010 D1). So no cross-kind dep/seq
  projection exists on *either* path — IMP-033's "the reader just admits the new
  kinds' edges" claim is optimistic and needs design scrutiny.

## Scope & Objectives

Extend the authored `needs`/`after` schema and the authoring verbs so a
**work-like entity** (slice, and per the governance decision below possibly spec)
can carry cross-kind dep/seq edges, and make at least one consumer project them
so the capture is not write-only.

Storage stays bespoke/typed (ADR-010 D1; the `rank` payload is incompatible with
the generic block). The edge target becomes a cross-kind canonical ref
(`SPEC-018`, `SL-042`), not a bare backlog id — the node identity in the
projector must generalise from `ItemKind` to a corpus-wide kind+id key
(`integrity::KINDS`).

In scope:

- Author surface: extend `needs`/`after` capture to slices (and any work-like
  kind admitted by the governance decision) — a verb seam mirroring
  `backlog needs`/`backlog after`, writing the typed edge with a cross-kind ref
  target. Edit-preserving TOML write.
- Forward-edge validation of the cross-kind target against `integrity::KINDS`
  (`needs` payload-free, target-validated; `after.to` likewise).
- Outbound-only (ADR-004) — no reverse edge authored on the target.
- At least one **honest consumer**: generalise the projector node identity from
  `ItemKind` to a corpus-wide key so the captured cross-kind edges actually feed
  a `dep`/`seq` overlay (the minimum that makes capture observable; see OQ-1).

## Non-Goals

- **Not the relation `link` writer / `RELATION_RULES`** — reference/lineage edges
  are SL-048's, shipped; dep/seq is a separate axis (ADR-010 D1).
- **Not `triggers`** — the watched-glob rider is a third axis; extending it
  cross-kind is its own concern, not folded here unless the schema move makes it
  free (OQ-4).
- **Not the actionability/blocker semantics** — SL-047's survey/next/blockers
  already mechanise overlay consumption; this slice feeds them new edges, it does
  not change the blocker mask or priority synthesis.
- **Not a backlog-schema regression** — the existing backlog `needs`/`after`
  suites are the behaviour-preservation proof; they stay green unchanged.

## Affected surface (concrete)

- `src/backlog.rs` — typed `Relationships{needs, after}` schema + the
  `needs`/`after` verb handlers (prior art to generalise or lift).
- `src/slice.rs` (and any other work-like kind module) — new authored field /
  verb wiring.
- `src/backlog_order.rs` — `ItemId{kind: ItemKind, id}` node identity; the
  `dep`/`seq` overlay build. Candidate generalisation site.
- `src/relation_graph.rs` / SL-046 reader — alternative projection path (design
  decides which carries cross-kind dep/seq).
- `src/integrity.rs` (`KINDS`) — corpus-wide id table for target validation.
- Install/gitignore only if a new authored sub-file appears (likely not — reuses
  existing per-kind TOML).

## Risks / Assumptions

- **R1 — projector identity churn.** `backlog_order.rs` is keyed on `ItemKind`
  throughout (tie-break, RSK-005 distinctness assert). Generalising to a
  corpus-wide key touches the cordage projection seam. Behaviour-preservation
  gate applies — existing ordering tests stay green.
- **R2 — two-projector ambiguity.** Unclear whether cross-kind dep/seq rides
  `backlog_order.rs` (generalised) or SL-046's all-entity reader (which ADR-010
  D1 currently excludes dep/seq from). → OQ-1, possibly an ADR amendment.
- **R3 — abstraction-gradient inversion.** A spec depending on a slice inverts the
  intended gradient (specs upstream of slices). Which kinds may `needs`/`after`
  which is project-global policy. → OQ-2, ADR-shaped.
- **A1** — `rank` stays a pairwise-edge attribute, never the item-level
  authored-priority scalar (PRD-009 §4 / PRD-011 OQ-001). Unchanged.

## Open Questions (resolve in /design)

- **OQ-1** — which projector carries cross-kind dep/seq: generalise
  `backlog_order.rs`'s `ItemKind` node key to corpus-wide, or extend SL-046's
  `outbound_for` reader (reversing ADR-010 D1's exclusion)? Likely an ADR
  amendment either way.
- **OQ-2** — which source kinds may author dep/seq onto which target kinds (the
  abstraction-gradient policy). Project-global → candidate ADR. Does a spec→slice
  `needs` ever make sense, or only same-tier / downstream edges?
- **OQ-3** — verb shape: lift `backlog needs`/`backlog after` to a generic
  cross-kind verb (`doctrine needs <SRC> <TGT>`), or add per-kind verbs mirroring
  backlog's? Coupling/cohesion call.
- **OQ-4** — is `triggers` pulled in opportunistically if the schema lift makes it
  free, or held strictly out (current scope: out)?

## Verification / closure intent

- Cross-kind `needs`/`after` authorable on a slice, target forward-validated
  against `KINDS`, round-trips through `doctrine <kind> show` (black-box golden).
- At least one consumer projects a cross-kind dep/seq edge into an overlay and
  surfaces it (a cross-kind blocker via SL-047's `blockers`, or the ordering) —
  capture is observable, not write-only.
- Backlog `needs`/`after` behaviour preserved — prior suites green unchanged.
- `just gate` clean; lint zero warnings.

## Summary

Capture-side extension of the dep/seq axis (`needs`/`after`) from backlog-only to
cross-kind work-like entities, plus the minimum consumer generalisation to make
the new edges observable. Distinct from the SL-046/047/048 relation work (that is
the reference/lineage `link` axis; ADR-010 D1 keeps dep/seq separate and typed).
Two project-global open questions (projector path, sequencing-gradient policy)
likely mint or amend an ADR in /design.

## Follow-Ups

- `triggers` cross-kind (if held out per OQ-4).
- Any ADR minted for OQ-1/OQ-2 recorded and linked `governed_by`.
