# Backfill Doctrine product-spec corpus

## Context

SL-015 shipped product/tech specs as first-class entities (`doctrine spec new`,
`req add`, `show`, `validate`). The product-spec template was just restructured
into eight sections (Intent, Scope, Principles, Requirements, Success Measures,
Behaviour, Verification, Open Questions). The corpus is empty — `spec list`
returns nothing.

Doctrine has accumulated rich source material describing *what* it does and
*why*: nine evergreen specs under `doc/*`, the slice/ADR record, the source
under `src/`, and the doctrine memory corpus. None of it is captured as
product specs. This slice backfills that corpus — dogfooding the spec
machinery on Doctrine's own capabilities — with agents authoring the specs
from the existing material.

## Scope & Objectives

Author the full product-spec corpus for Doctrine's own capabilities. Approach,
in phases:

1. **Taxonomy** — derive the set of product specs that should exist: a
   capability → PRD map mined from `doc/*`, slices, ADRs, code, and memory.
   The unit is a *product capability* (what/why), not a source document — the
   mapping is not 1:1 with `doc/*` (those skew toward the *how*). The taxonomy
   and its source map are **scaffolding consumed by the backfill, not a
   persisted authored artifact** — they live in disposable runtime context
   (handover / phase sheets), not committed under `doc/*`.
2. **Exemplar** — author one PRD end-to-end against the new eight-section
   template, as the reference shape every other spec follows. Lock its quality
   bar before fanning out. Then **reconcile the `spec-product` SKILL.md to be
   exemplar-driven**: rewrite its stale "Capture" list (which still names the
   old template sections) to point at the locked exemplar as the canonical
   shape — referential, not a second prescriptive copy of the section list.
3. **Backfill** — author the remaining PRDs across the taxonomy, agents working
   from the exemplar and the source map. Each spec is *what/why* and durable;
   the *how* stays in `doc/*` / `/spec-tech`.
4. **Validate** — `doctrine spec validate` green across the corpus (no dangling
   member/interaction FKs, no duplicate labels, no orphan requirements);
   `spec show` reassembles each cleanly.

Requirements (`REQ-NNN` peer entities) are added per spec via `spec req add`,
with `--kind functional|quality`.

## Non-Goals

- **Tech specs** — `/spec-tech` and the `tech` subtype are out of scope; this
  slice is the *what/why* layer only.
- **Re-editing the template** — the restructured template is the fixed target,
  not a thing this slice revises.
- **Rewriting `doc/*`** — the evergreen specs stay as the *how*; they are a
  source, not a deliverable.
- **Slice→spec relationship wiring** — the `relationships` block stays reserved
  (v1, per `slice-019.toml`); linking slices to the new specs is later work.
- **Authoring tooling** — no new CLI verbs; use the shipped `spec` surface.
- **Re-deriving the template from the skill** — the skill is brought in line
  *with* the template/exemplar, not the reverse.

## Affected surface

- `.doctrine/spec/<subtype>/...` — the authored product specs (TOML identity +
  MD narrative + `members.toml`), created via `doctrine spec new product`.
- The taxonomy / source-map artifact — location TBD in design (authored
  slice-local doc vs `doc/*`); honour the storage rule either way.
- `install/templates/spec-product.md` — **prerequisite**: the template edit is
  currently uncommitted; commit it before authoring against it.
- `plugins/doctrine/skills/spec-product/SKILL.md` — **canonical** skill source
  (install propagates to `.doctrine/skills/` / `.claude/skills/` copies);
  reconciled to be exemplar-driven in the exemplar phase. The skill's current
  §4 example prescribes prose FR/NFR rows, colliding with the REQ-entity model —
  the reconciliation fixes this (see design D-2).

## Risks, assumptions, open questions

- **Source skew.** `doc/*` is mostly the *how* (tech/design notes). Distilling
  the *what/why* requires judgement; risk of restating mechanism as product
  intent. The exemplar sets the altitude.
- **Parallel drift.** Agents authoring specs concurrently risk inconsistent
  shape/voice. Mitigated by locking the exemplar and taxonomy first.
- **Taxonomy boundaries.** What counts as one product capability vs several
  (e.g. is the entity model a product capability or internal mechanism?) is a
  design decision, not predetermined here.
- **Resolved:** taxonomy/source-map is scaffolding consumed by the backfill —
  disposable runtime context, not a persisted authored artifact.
- **Resolved:** skill reconciliation is in scope and exemplar-driven — the skill
  becomes referential (points at the locked exemplar), not a second copy of the
  section list.
- **Assumption:** the eight-section template is final for the duration.

## Verification / closure intent

- Taxonomy reviewed and agreed before any spec is authored.
- Exemplar PRD reviewed against the template and accepted as the reference bar.
- Every capability in the taxonomy has an authored PRD; each is *what/why*, not
  *how*, and follows the exemplar's shape.
- `doctrine spec validate` clean corpus-wide; `spec show` reassembles each spec.
- `just check` green; conventions honoured (storage rule, no derived data in
  prose).
