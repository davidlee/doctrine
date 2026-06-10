# Implementation Plan SL-021: Backfill Doctrine technical-spec corpus

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Backfill the technical-spec corpus — the *how* of Doctrine's shipped
architecture — as `tech` entities, the tech-side analog of SL-019's product
backfill. The design is locked (`design.md`, D1 re-locked per the inquisition's
F1-A remedy: a whole-system **context** root "Doctrine", mechanism containers
beneath it, thin capability components). This plan turns design §8's phase shape
into five executable phases.

The corpus is *data*, not code: the work is authoring TOML+MD spec entities and
their relational spine (`descends_from`, `parent`, `interactions`), with
`doctrine spec validate` and `spec show` as the integrity gates and the user as
the altitude gate. No new CLI verbs (slice non-goal); the only code touched is
the embed refresh in PHASE-03.

## Sequencing & Rationale

Five phases, each a hard gate for the next — the sequencing exists to lock
*shape* before *volume*, the SL-019 lesson:

- **PHASE-01 Taxonomy first** — the roster and per-spec source map are confirmed
  and the boundary calls settled *before* a single spec is authored. The roster
  is disposable scaffolding (gitignored), never committed (SL-019 CHARGE VII).
  Gated by user agreement: fan-out against a wrong taxonomy is the expensive
  mistake.

- **PHASE-02 Exemplar trio before fan-out** — author exactly the three C4 shapes
  (root → container → component) end-to-end and lock them as the bar. The root
  is the hardest shape: a whole-system synthesis that must stay at altitude
  (names the parts and their composition, never restates a container's how) and
  is anchor-free (REQ-085). `entity-model.md`'s durable content lives on the
  entity-engine *container*, not the root — the F1-A correction. Locking the
  trio with the user prevents the inverted-SL-019 skew (sliding into
  change-specific design or stale mechanism) from propagating across ~13 specs.

- **PHASE-03 Skill rework after the trio** — the SKILL.md predates the SL-022
  spine; reworking it *after* the exemplar exists lets it point at concrete
  shapes rather than describe them abstractly (the SL-019 PHASE-03 pattern). The
  embed-refresh ritual (`skills install` + `touch src/skills.rs` + rebuild) is a
  known footgun — a lone edit is invisible until the embedding crate recompiles.

- **PHASE-04 Fan-out, top-down, capped** — the remaining specs, authored from the
  locked exemplar + reconciled skill + source map. Top-down because a child's
  `parent` must resolve, so containers land before their components (waves).
  Width is capped for reservation-contention and voice-consistency reasons
  carried from SL-019; the concrete fan-out mechanism (Workflow vs serial
  `/execute`) is a `/phase-plan` call, not design- or plan-locked.

- **PHASE-05 Edges, validate, coverage audit last** — peer `interactions` edges
  only where containment doesn't already say it; the SPEC-001/SPEC-002 `parent`
  retrofit (a mechanical single-field add, F5, now pointing at a *true*
  container under the root); the corpus-wide integrity gates; the
  capability-coverage audit (every shipped-mechanism PRD reachable, exemptions
  named); and the `draft`→`active` flips. This is where SL-022's checks get
  exercised for real corpus-wide.

## Notes

- **Requirement status stays `pending`** throughout (D4) — the initial value of
  the authored/normative requirement-status enum, asserting authored status not
  derived coverage (PRD-013 two-tier). No coverage tables, no status derivation.
- **`doc/*` interim authority (F4)** — once a tech spec captures an architecture
  it is authoritative for it; the lifted `doc/*` content is demoted to
  seed/pointer. Physical retirement of `doc/*` is out of scope (flag as a
  `/close` follow-up).
- **Binary-path trap** — all authoring uses `cargo run --` or the
  `cargo metadata`-resolved binary; `./target/debug` is stale in the jail.
