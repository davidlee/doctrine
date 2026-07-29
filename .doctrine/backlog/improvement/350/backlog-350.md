# IMP-350: Retire story from the product altitude ladder

## Context

`ProductLevel` (`src/spec.rs:703`) ships a four-rung product ladder —
`domain | capability | feature | story` — minted as a rank-for-rank mirror of
`C4Level`. The doc comment states the intended correspondence outright:
*"feature≈component, story≈code"*.

`story` is on that ladder because `code` occupies C4 rank 3, not because a user
story is a smaller structural part of a feature. A story is a *path through* a
product, not a *part of* one: it may traverse several features or several
capabilities, and its scope varies wildly. RFC-024 records the full argument and
its second half (a cross-cutting `Scenario` entity that absorbs what `story` was
badly modelling).

This card is the cheap, separable first half. It does not wait on `Scenario`.

## Why it is cheap

- **The corpus is clean.** `product_level` is set on exactly three specs
  (PRD-016, PRD-017, PRD-018), all `capability`. Nothing is a `story`.
- **No authored governance states the ladder.** `story` appears in no spec
  prose, no requirement, and no ADR/policy/standard — only in code, one
  template, and one skill. So this needs no Revision.
- **The recursive floor already works.** `parent_rank_findings`
  (`src/registry.rs:297`) flags only `delta < 0` (inversion) and `delta > 1`
  (gap); `delta == 0` passes. Feature-parents-feature is already legal, so the
  three-rung ladder with a recursive floor needs no new rule — only that the
  property stop being an accident of a permissive check and become stated,
  tested intent.

## Decision taken — hard removal, no deprecation shim

RFC-024 Q1 asks whether retiring `story` needs a migration path. Taken here:
**hard removal**. The corpus is clean, and serde's own error names the legal
set (`unknown variant \`story\`, expected one of ...`). An accept-and-warn path
would mean keeping a deprecated variant in the enum — which is the exact thing
being removed. Any out-of-tree corpus that adopted `story` gets a parse error
that tells it what to write instead.

Ranks are unchanged: `domain=0, capability=1, feature=2`. Nothing re-ranks.

## Scope

- `src/spec.rs` — drop the `Story` variant, its `as_str` arm, its `rank` arm.
  Amend the `ProductLevel` doc comment (the `story≈code` mirror claim) and the
  `Spec::product_level` field doc to state the three-rung ladder, the recursive
  `Feature` floor, and the deliberate depth asymmetry with C4.
- `src/spec.rs` tests — drop the `story` row from
  `product_level_kebab_round_trips_every_variant`; add a negative test that
  `product_level = "story"` no longer parses; reconstruct
  `sweep_parent_rank_product_gap` (it built its gap as domain→story) as
  domain→feature, still delta 2; add product-side coverage that
  feature-parents-feature is clean (existing same-level coverage is tech-side
  only).
- `install/templates/spec-product.toml` — the ladder comment.
- `.agents/skills/spec-coverage-assessment/SKILL.md` — the ladder enumeration.
- `.doctrine/spec/product/018/spec-018.toml` — carries a stale inline copy of
  the template's ladder comment.
- Rebuild + `doctrine install` to re-embed the template and refresh the
  installed skill.

## Out of scope

- `Scenario` (RFC-024 half two) — the expensive half, unresolved.
- The C4-side question RFC-024 Q2 raises (same-level parenting is equally legal
  for tech specs — affordance or hole?). Untouched here.
- Any requirement-rollup rule (RFC-024 Q10).

## Done when

Ladder is three rungs in code, template, skill, and the one stale spec comment;
`story` fails to parse with a test proving it; feature→feature is proven clean
product-side; `doctrine validate` clean; `doctrine check gate` green.
