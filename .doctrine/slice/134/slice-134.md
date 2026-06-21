# Risk facet CLI verb — set/clear likelihood/impact

## Context

Risk items are scaffolded with an empty `[facet]` section. The `likelihood`
and `impact` fields can be hand-edited in the TOML, but no CLI verb exists
to set or clear them post-creation. `doctrine estimate set` and `doctrine
value set` exist for the other assessable facets — risk is the gap.

IMP-118 (SL-133) reads `likelihood × impact` as the risk dimension of the
priority base score. Without a CLI verb, users must hand-edit TOML to
adjust risk assessment.

## Scope & Objectives

- `doctrine risk set <ID> --likelihood <LEVEL> --impact <LEVEL>` — set both axes
- `doctrine risk clear <ID>` — clear the facet back to empty
- Validation: levels must be `low | medium | high | critical`
- Refuse for non-risk item kinds (the `kind = "risk"` auth gate)
- Pure/impure split: validation pure, disk read/write impure — same seam
  as `estimate`/`value`
- Echo on success (matching `estimate set`/`value set` pattern)

## Non-Goals

- No `risk show` or `risk list` — facet display belongs in the show path
  (SL-132 covers estimate/value display; risk display is analogous but
  deferred)
- No risk history tracking
- Risk facet model (`RiskFacet`, `RiskLevel`, `exposure()`) already exists
  in `src/backlog.rs:382` — reused as-is

## Terrain

| File | Change |
|------|--------|
| `src/backlog.rs:382` | `RiskFacet`, `RiskLevel` — existing model, no change |
| `src/backlog.rs:507` | `exposure()` — existing, no change |
| `src/commands/facet.rs` | New `RiskSetArgs` / `RiskClearArgs` + `run_risk_set` / `run_risk_clear` — follow estimate/value pattern |
| `src/facet_write.rs` | Reuse `apply_set` / `apply_clear` for risk facet |
| `src/main.rs` | Register `Risk` subcommand with `set`/`clear` |

## Dependencies

- Precedent: `doctrine estimate set` / `doctrine value set` (SL-118)
- `src/commands/facet.rs` — existing pattern for set/clear verbs
- `src/facet_write.rs` — existing TOML write leaf
