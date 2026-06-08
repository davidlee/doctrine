# Backfill Doctrine technical-spec corpus

## Context

SL-015 shipped product/tech specs as first-class entities. SL-019 backfilled
the **product** corpus — eleven `PRD-NNN` specs capturing Doctrine's own
*what/why* — and explicitly left tech specs out of scope. The `tech` subtype is
unused: `spec list` shows a product table only; the technical corpus is empty.

The *how* still lives where it always has — nine evergreen specs under `doc/*`
(`entity-model.md` the umbrella, plus `slices-spec`, `spec-entity-spec`,
`drift-spec`, `reservation-spec`, `relation-index`, `memory-spec`,
`install-spec`, `skills-spec`), the ADR record, and `src/`. None of it is
captured as `tech` specs. This slice backfills that corpus — the tech-side
analog of SL-019 — dogfooding the `tech` subtype on Doctrine's own architecture.

`doc/entity-model.md` is the explicit seed: it is the consolidation-direction
umbrella the per-entity notes hang off, the natural top-level architecture spec.

## Scope & Objectives

Author the Doctrine technical-spec corpus from existing source material —
`doc/*` (the durable *how*), the `PRD-NNN` product specs (the *what* each tech
spec realises), and the ADRs (load-bearing global decisions the architecture
must honour). Mirrors SL-019's shape:

1. **Taxonomy** — derive the set of `SPEC-NNN` tech specs that should exist and
   their `c4_level` placement (context / container / component / code). The unit
   is an *architectural capability* (the *how*), seeded from `entity-model.md`
   and the `doc/*` per-entity notes — not necessarily 1:1 with `doc/*` files nor
   with the eleven PRDs. Disposable scaffolding (handover / phase sheets), not a
   persisted artifact — same rule as SL-019.
2. **Exemplar** — author one tech spec end-to-end (likely the `entity-model`
   umbrella) against the tech template, locking the quality bar — `c4_level`,
   `sources` code anchors, ADR links — before fanning out. Reconcile
   `spec-tech` SKILL.md to be exemplar-driven if it is stale, as SL-019 did for
   `spec-product`.
3. **Backfill** — author the remaining tech specs from the exemplar and source
   map. Each is durable *how* (architecture, mechanism, invariants, contracts),
   not a single change's design.
4. **Edges & validate** — author tech-only `interactions.toml` spec→spec edges
   and `sources` code anchors where they carry weight; `doctrine spec validate`
   green corpus-wide (no dangling member/interaction FKs, no duplicate labels,
   no orphan requirements); `spec show` reassembles each cleanly.

Requirements are `REQ-NNN` peer entities added via
`spec req add <SPEC-ref> --kind functional|quality`.

## Non-Goals

- **Product specs** — the `PRD-NNN` corpus is a fixed *source*, not revised here.
- **Per-slice design** — a single change's current-vs-target design stays in that
  slice's `/design`. Tech specs are evergreen *how*, not change plans.
- **Rewriting `doc/*`** — the evergreen specs stay as authoritative source; this
  slice lifts their durable content into the `tech` entity surface, it does not
  delete or rewrite them. Whether `doc/*` is eventually superseded by the tech
  corpus is **out of scope** (flag as a follow-up, do not decide here).
- **Authoring tooling** — no new CLI verbs; use the shipped `spec` surface
  (`new tech`, `req add`, `show`, `validate`, `interactions.toml`).
- **PRD↔SPEC / slice↔spec relationship wiring** — the slice `relationships`
  block stays reserved (v1). Cross-corpus linkage beyond tech-only
  `interactions` is later work.
- **New ADRs** — ADRs are linked as constraints, not authored here.

## Affected surface

- `.doctrine/spec/tech/<n>/` — the authored tech specs (4-file scaffold:
  `spec-NNN.toml` identity incl. `c4_level`/`sources`, `spec-NNN.md` narrative,
  `members.toml`, `interactions.toml`), created via `doctrine spec new tech`.
- `REQ-NNN` peer entities via `spec req add`.
- `plugins/doctrine/skills/spec-tech/SKILL.md` — canonical skill source;
  reconciled to be exemplar-driven if stale (install propagates copies).
- `install/templates/spec-tech.md` — verify the tech template is the fixed
  target before authoring; commit any pending edit first (SL-019 hit this).

## Risks, assumptions, open questions

- **Taxonomy boundaries (design decision).** What is one tech spec vs several,
  and the `c4_level` of each — is `entity-model` a single *context* spec with
  per-entity *component* children, or a flatter set? Seeded by `entity-model.md`
  + `doc/*`, settled in `/design`.
- **PRD↔SPEC correspondence (open).** Whether each tech spec maps to a product
  spec, and whether that mapping is recorded (and how, given cross-subtype edges
  are out of scope) — resolve in design.
- **Source skew, inverted.** `doc/*` is already the *how*, so the SL-019 risk
  (restating mechanism as intent) inverts: the risk here is drift into
  *change-specific design* or stale mechanism. The exemplar sets the altitude.
- **`doc/*` overlap.** Tech specs and `doc/*` will describe the same mechanisms;
  duplication vs single-source-of-truth tension. In-scope: lift durable content.
  Out-of-scope: retiring `doc/*`. Flag the tension, do not resolve by deletion.
- **Parallel drift.** Concurrent authoring risks inconsistent shape/voice —
  mitigated by locking exemplar + taxonomy first, as SL-019.
- **Assumption:** the tech template and `c4_level` enum are final for the
  duration.

## Verification / closure intent

- Taxonomy + `c4_level` placement reviewed and agreed before any spec authored.
- Exemplar tech spec reviewed against the template and accepted as the bar.
- Every architectural capability in the taxonomy has an authored `SPEC-NNN`;
  each is durable *how*, follows the exemplar's shape, links its governing ADRs.
- `doctrine spec validate` clean corpus-wide; `spec show` reassembles each spec.
- `just check` green; conventions honoured (storage rule, no derived data in
  prose).

## Summary

## Follow-Ups
