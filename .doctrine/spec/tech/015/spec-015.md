# SPEC-015: Backlog entity surface

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The backlog entity surface is doctrine's work-intake capture layer realising
**PRD-009**: the first-class home for an issue, improvement, chore, risk, or idea
the moment it surfaces, before it is scoped. It is a component of the entity engine
(SPEC-004) — five `ItemKind`s riding five engine `Kind`s over the same kind-blind
materialiser. All shared mechanism (identity, the atomic claim, id allocation,
edit-preserving status transition, and the scaffold/render pipeline) lives in the
parent container and is used here unchanged; this spec carries only what is specific
to the backlog: the five-kind discrimination, the work-item status vocabulary and
hide-set, the `Resolution` and risk `[facet]` enums, the outbound `[relationships]`
seam, and the priority-ordering projection into the `backlog_order` cordage adapter.

## Responsibilities

Mirrors the structured `responsibilities` list: bind the five item kinds onto the
engine, fix the work-item status vocabulary and hide-set, carry the kind-specific
closed enums and risk facet, hold the outbound relationships seam, mediate capture
and survey through `backlog new`/`list`, and project items into the cordage ordering
adapter.

### Five kinds, one engine

A backlog item exists in five subtypes — issue (`ISS`), improvement (`IMP`), chore
(`CHR`), risk (`RSK`), idea (`IDE`) — each a data-valued engine `Kind` with its own
tree under `.doctrine/backlog/<kind>/` and its own reservation namespace, so `ISS-001`
and `RSK-001` coexist with independent counters. The subtypes diverge only in their
prefix and whether the scaffold seeds a risk `[facet]`; every kind shares one
`backlog-NNN.{toml,md}` fileset and an `NNN-slug` symlink alias. The discrimination is
the **one-entity, one-schema** rule (PRD-009): kind is a facet on one entity, never a
fork of parallel per-kind schemas. The `ItemKind` enum is also the `backlog new`
positional (`clap::ValueEnum`) and the kebab serde of the stored `kind` field.

```text
.doctrine/backlog/<kind>/NNN/
  backlog-NNN.toml     # identity, status, resolution, tags, [facet] (risk), [relationships]
  backlog-NNN.md       # prose body
```

### Status vocabulary and list hide-set

A backlog item's status is a closed kebab-serde set — **open · triaged · started ·
resolved · closed** — seeded `open` and hand-settable, ungated as slices and ADRs
ship. The same set is the `--status` known-vocabulary the `backlog list` filter
validates against; because the enum is closed, a stored status is always
in-vocabulary, so no drift marker is possible. A backlog-local `Status::is_terminal`
predicate (resolved/closed) drives the default-list hide-set through `listing::retain`
— terminal items drop from the default list and are revealed by `--all` or any
explicit `--status`. This predicate is deliberately **not** `slice::is_terminal_status`:
backlog and slice lifecycles are independent vocabularies. The same predicate also
couples `resolution ⟺ terminal` on `edit`.

### Closed enums and the risk facet

Two further closed enums are backlog-specific. `Resolution` (fixed, done, mitigated,
accepted, expired, duplicate, wont-do, obsolete, promoted) is one generic,
kind-agnostic close-reason set — a resolution is never a close reason hidden in a
facet. The risk kind alone carries a `[facet]`: typed `RiskLevel` likelihood and
impact axes (low/medium/high/critical), an `origin` text, and `controls`. Both the
resolution and the risk axes are optional and seeded `""`, so they ride a `"" -> None`
validation seam — a tolerant `RawBacklogToml` reads them as raw strings, and a
separate `validate` pass maps empty to absent and parses any non-empty token to its
variant (erroring on an unknown one). This is the three-layer parse model the parent
container realises, specialised here for the seeded-empty optionals.

### The outbound relationships seam

Each item carries a `[relationships]` table — forward links to `slices`, `specs`, and
`drift`, plus the priority-engine edges `needs` (hard prereq), `after` (soft
sequence, each a `{ to, rank }` edge), and `triggers` (architectural prefactor
riders). The seam is outbound-only (ADR-004); the reverse direction is derived, never
authored twice. `backlog needs`/`after` append into the seeded arrays
edit-preservingly via `toml_edit` — never a full reserialize, so comments and inert
tables survive — and **refuse** (rather than corrupt) when the seeded `[relationships]`
array is missing, directing the user to regenerate via `backlog new`.

### Capture, survey, and cross-kind grouping

`backlog new` is the capture verb: it reserves an id in the kind's namespace, scaffolds
the fileset, and prints the canonical `XXX-NNN` ref, deriving the prefix from the
engine `Kind` (the single source). `backlog list` surveys the corpus, reading each
kind tree in turn in declaration order; the cross-kind primary sort key is the
`ItemKind::ordinal` — a deterministic **grouping** (issue…idea), explicitly not a
priority claim and not the inert risk-first `KIND_PRECEDENCE` reserved for the future
multi-kind resolver. `backlog show` reassembles the identity TOML, prose, risk facet,
and outbound relationships; `backlog edit` drives the coupled status/resolution
transition.

### Priority-ordering projection

Priority order is not computed here — it is composed by **cordage** through the
`backlog_order` adapter (the consumer half of cordage, SL-039). The adapter owns only
the domain *vocabulary*: it projects items into `OrderInput`, builds a hard `needs`
overlay and a soft `after` overlay plus the risk-`exposure` (likelihood × impact)
within-level tiebreak, and reads the composed order and resolution provenance back out
as `ItemId`/`Override`. It performs no sort of its own, is pure and disk-free (it never
sees a `BacklogItem` or the filesystem), and never lets an opaque cordage id escape a
`pub(crate)` signature. `backlog order` refuses to compose when a `needs` cycle exists,
naming the members rather than emitting a wrong order.

## Concerns

- **Lockstep status vocabulary.** The `BACKLOG_STATUSES` known-set must mirror the
  `Status` enum exactly (guarded by `backlog_statuses_matches_the_variants`); a closed
  enum means a stored status is always in-vocab, so the only failure mode is the
  known-set drifting from the enum.
- **Seeded-array refusal.** The edit-preserving append into `[relationships]` must
  refuse on a missing seeded array rather than fabricate one — the alternative is the
  silent corruption hazard the F-1 guard exists to close.
- **Grouping is not priority.** `ItemKind::ordinal` (list grouping) and
  `KIND_PRECEDENCE` (the inert future resolver order) are distinct orderings; neither
  is a priority claim, which is the `backlog_order`/cordage axis.

## Hypotheses

- **One entity discriminated by a kind facet beats five schemas.** The five kinds share
  enough structure (fileset, status, relationships, reassembly) that one kind-blind
  materialiser serving all five — diverging only by prefix and the risk facet seed — is
  preferred over five parallel implementations, and keeps kind variation a facet rather
  than a fork.
- **The backlog mints ordering edges but does not own ordering.** `needs`/`after`/
  `triggers` are captured here, but the composition is delegated to cordage so the
  backlog stays a capture surface; the adapter owning only vocabulary keeps the
  priority mechanism swappable and disk-free.

## Decisions

- **D1 — the backlog status vocabulary is backlog-local.** `Status` and its
  `is_terminal` predicate are independent of the slice lifecycle; the terminal set
  drives the list hide-set through the same predicate, with no new terminal set and no
  reuse of `slice::is_terminal_status`.
- **D2 — kind-specific data is typed enums through a `"" -> None` seam, never a bag.**
  `Resolution` and the risk `[facet]` axes are closed enums; seeded-empty optionals ride
  a validation seam rather than a serde `Option`, so an unassessed axis is absent, not a
  wrong variant.
- **D3 — relationships are outbound-only and appended edit-preservingly.** The
  `[relationships]` seam stores forward links and ordering edges one-way (ADR-004),
  appended via `toml_edit` and refusing on a missing seeded array.
- **D4 — priority is composed by cordage, not by the backlog.** The `backlog_order`
  adapter projects vocabulary into cordage overlays and reads the order back; it
  performs no sort itself and never leaks an opaque cordage id.
