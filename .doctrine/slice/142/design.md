# Design: Wire tag coefficients into priority scoring

SL-142 · `design` · 2026-06-22

## Overview

Wire entity tags through the scoring data path so the `× Σ tag_coefficients` term
in ADR-015's `value_dim` formula takes effect. Tags are already stored on entities
(SL-136); `tag_coeff()` exists dead in `config.rs`. This design bridges the gap:
`read_facets` → `EntityFacets` → `base_score`.

## Data path

### 1. `EntityFacets` (`src/facet.rs`)

Add `tags: Vec<String>` and remove the "Extended in later slices: tags (SL-136)" comment.

```rust
pub(crate) struct EntityFacets {
    pub estimate: Option<EstimateFacet>,
    pub value: Option<ValueFacet>,
    pub risk: Option<RiskFacet>,
    pub tags: Vec<String>,              // NEW — wired from entity TOML tags; empty vec = identity
}
```

### 2. `read_facets` (`src/catalog/scan.rs`)

Return a 4-tuple `(estimate, value, risk, tags: Vec<String>)`.

Parse from the already-parsed TOML table (no extra I/O), reading raw string
values — tags are already normalised at rest by SL-136:

```rust
let tags: Vec<String> = table
    .get("tags")
    .and_then(|v| v.as_array())
    .map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    })
    .unwrap_or_default();
```

**Normalisation note:** Tags at rest are already normalised by SL-136
(`tag::normalize_tag`). `read_facets` reads raw TOML values and passes
through unmodified — no re-normalisation needed. An absent `tags` key →
empty vec → identity in the formula.

### 3. `ScannedEntity` (`src/catalog/scan.rs`)

Add `pub(crate) tags: Vec<String>` after the `risk` field.

### 4. Call site: `build_from` (`src/priority/graph.rs`)

The inline `EntityFacets { ... }` constructor at line ~248 gains `tags: entity.tags.clone()`.

## Formula change

### 5. `base_score` (`src/priority/graph.rs`)

Current:
```rust
let kw = cfg.kind_weight(kind.prefix);
cfg.coefficients.value * v.value * kw / est_mid
```

After:
```rust
let kw = cfg.kind_weight(kind.prefix);
// tag_term = (1.0 + Σ(coeff - 1.0)).max(0.0): identity base for absent tags,
// each configured tag pushes the multiplier by its excess over default.
// Default coeff (1.0) → delta 0 → no effect. Floor at zero prevents a
// negative multiplier from many demoting tags (F-7).
let tag_term = (1.0 + f.tags
    .iter()
    .map(|t| cfg.tag_coeff(t) - 1.0)
    .sum::<f64>())
    .max(0.0);
cfg.coefficients.value * v.value * kw * tag_term / est_mid
```

**Edge case:** empty `f.tags` → `1.0 + 0.0` = identity (×1.0). A single tag
with default coeff (1.0) → `1.0 + 0.0` = identity. Multiple default tags
→ `1.0 + n×0.0` = identity. Adding tags never shifts score unless their
coeffs differ from the 1.0 default.

### 6. `tag_coeff` (`src/priority/config.rs`)

Remove `#[cfg_attr(not(test), expect(dead_code, reason = "consumed SL-136 (tags)"))]`.

## Verification

### New unit tests (`src/priority/graph.rs`)

Model on existing `base_score_all_facets_present`:

| Test | Tags | Coeff | tag_term | Effect on value_dim |
|------|------|-------|----------|---------------------|
| `base_score_empty_tags_identity` | `[]` | — | `1.0 + 0.0 = 1.0` | unchanged (×1.0 identity) |
| `base_score_with_tag_coefficient` | `["area:foo"]` | `"area:foo" → 2.0` | `1.0 + (2.0-1.0) = 2.0` | doubles |
| `base_score_multiple_tags` | `["a", "b"]` | `"a" → 1.5, "b" → 2.0` | `1.0 + 0.5 + 1.0 = 2.5` | ×2.5 |
| `base_score_demoting_tag` | `["wontfix"]` | `"wontfix" → 0.5` | `1.0 + (0.5-1.0) = 0.5` | halves (demotion works) |
| `base_score_multi_demote_floors_at_zero` | `["x", "y"]` | both → 0.0 | `(1.0 -1.0 -1.0).max(0) = 0.0` | zeroes (floored, not negative) |

### Existing tests unchanged

All current `base_score_*` tests use entities without tags (empty `EntityFacets.tags`).
With `tag_term = 1.0 + Σ(coeff - 1.0)`, empty tags produce `tag_term == 1.0`,
the identity. No assertion updates needed.

### Golden tests

The identity semantics (`tag_term = 1.0` for empty/default tags) mean scoring for
untagged entities is unchanged — no golden shift from the formula change alone.
If the test corpus contains tag-bearing entities, goldens may shift and need updating.

### Gate

`just gate` must pass with zero warnings. The `dead_code` removal is the only lint
change.

## Risks

- **IMP-109 adjacency:** `read_facets` still re-parses TOML already parsed by
  `status_and_title_for`. This PR does not fix that — it's a pre-existing concern
  tracked separately. Adding tags to the existing `read_facets` is correct per
  current architecture.
- No test data carries tags in the current corpus → identity semantics (×1.0)
  means goldens are unaffected by this PR. Goldens only change if tag-bearing
  entities are present in the corpus.

## Follow-ups

- RFC-002 item B: seed `[priority.tag_coefficients]` in `doctrine.toml`.
