# Review RV-309 — reconciliation of SL-021

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

SL-021 is a data-only slice — 15 tech specs (SPEC-003 through SPEC-017) authored
from existing source material (`doc/*`, PRDs, ADRs). No Rust code was changed
(only `spec-tech/SKILL.md` in PHASE-03). The audit probes:

1. **Corpus integrity** — `spec validate` clean, `spec show` reassembles each spec.
2. **Spine correctness** — `parent`/`descends_from`/`c4_level` match the locked
   design §2 taxonomy; no false containment, no silent exemption.
3. **Interactions coverage** — weighty peer edges authored where containment
   doesn't already express the relationship (design §6).
4. **Altitude gate** — specs hold retrospective present-tense shipped-how altitude;
   no current-vs-target drift, no stale mechanism (design §9).
5. **Closure readiness** — no committed scaffolding (design §7), `just check` green,
   `draft→active` flips done.

Invariants from governance: PRD-012 (tech-spec spine), PRD-013 (two-tier truth,
no coverage derivation), ADR-004 (outbound-only relations), design §9 (interim
authority rule).

## Synthesis

SL-021 is a data-only slice that backfilled 15 tech specs (SPEC-003 through
SPEC-017) from Doctrine's existing architectural source material — the
tech-side analog of SL-019's product backfill. Five phases: taxonomy → exemplar
trio → skill rework → fan-out (12 authored) → edges + validate.

The corpus is complete and clean. Every shipped-mechanism PRD is covered by
≥1 `descends_from` (PRD-010 exempt per design — mechanism unbuilt). The
relational spine (`parent`/`descends_from`/`c4_level`) matches the design §2
taxonomy with one conscious divergence (F-4): SPEC-002 was placed under the
entity engine (SPEC-004) as a component rather than under the root as a
container — architecturally correct; reconciliation is an entity-level
operation. The plan's EX-2 text is superseded by this more accurate placement.

Interactions edges (7 across 5 specs, PHASE-05) capture the weighty peer
relationships the design §6 prescribed: install↔skills, id-lifecycle→memory,
and governance-kinds→ADR-surface. The remaining 10 specs carry no weighty
peer edges — their cross-references are parent/child containment only.

One design defect surfaced (F-1): §6's example "memory uses reservation" is
factually wrong — memory sidesteps reservation via UUID identity.

`spec validate` is corpus-clean. `doctrine check gate` is green. All specs are
`active`. No scaffolding was committed (SL-019 CHARGE VII honoured).

The conformance registry is incomplete (F-3) — all phases lack source-deltas.
PHASE-03 touched code; the rest were data-only. The gap has no practical
impact but leaves a misleading `incomplete` signal. Tolerated.

## Reconciliation Brief

### Per-slice (direct edit)

- **design.md §6** (F-1): replace the inaccurate example "memory engine uses
  reservation for id allocation" with an accurate one. Candidates:
  "install uses skills distribution for gitignore negation and confirm prompt"
  or "id lifecycle references memory's UUID-identity model."

### Governance/spec (REV)

None. The SPEC-002 parent divergence (F-4) is accepted — no governance or spec
change needed. The plan criteria (EX-2) are immutable-append; document the
supersession here rather than attempting a plan edit.

## Reconciliation Outcome

### Direct edits applied
- **design.md §6** (F-1): replaced inaccurate example "memory engine uses
  reservation for id allocation" with "install uses skills distribution for
  gitignore negation and confirm prompt" and "id lifecycle references memory's
  UUID-identity model" — both backed by authored interactions edges.

### Withdrawn / tolerated / aligned
- F-2: withdrawn (duplicate of F-3)
- F-3: tolerated — conformance registry incomplete (data-only slice)
- F-4: aligned — SPEC-002 parent=SPEC-004 is architecturally correct
