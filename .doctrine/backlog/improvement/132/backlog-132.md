# IMP-132: Risk facet CLI verb — set/clear likelihood/impact on existing risk items

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Gap

Risk items are scaffolded at creation (`doctrine backlog new risk ...`) with an empty
`[facet]` section. The `likelihood` and `impact` fields can be authored by hand in the
TOML, but there is no CLI verb to set or clear them post-creation.

`doctrine estimate set` and `doctrine value set` exist for the `[estimate]` / `[value]`
 facets. The risk facet is the only assessable dimension missing a dedicated CLI setter.

## Why it matters now

IMP-118 (multi-dimensional priority scoring) reads `likelihood × impact` as the
**risk dimension** of the base score. Without a CLI verb, users must hand-edit TOML
to adjust a risk's assessment — friction that will increase once the score drives
`survey`/`next` ordering.

## Scope

- `doctrine risk set <ID> --likelihood <LEVEL> --impact <LEVEL>` — set both axes
- `doctrine risk clear <ID>` — clear the facet back to empty
- Validation: levels must be one of `low | medium | high | critical`
- Pure/impure split: validation pure, disk read/write impure (same seam as
  `estimate`/`value`)
- The risk item's `kind = "risk"` is the auth gate (refuse for non-risk kinds)

## Links

- Surfaced during IMP-118 dependency analysis (2026-06-21)
- Risk facet model: `src/backlog.rs` — `RiskFacet`, `RiskLevel`, `exposure()`
- Precedent: `doctrine estimate set` / `doctrine value set` (SL-118)
