# Design: SL-101 — Estimate & Value facets

## 1. Overview

Two kind-agnostic, optional entity facets shipped in one slice: **estimate** (bounded
attention burden) and **value** (a single magnitude). Both are pure-leaf modules
(ADR-001), parsed from entity TOML `[estimate]` / `[value]` tables, with project-wide
units resolved from `doctrine.toml`.

## 2. Current vs target behaviour

**Current:** no estimate or value code exists. `SliceDoc` and `Meta` carry no
facet fields. `doctrine.toml` has no `[estimation]` or `[value]` sections. The
`install/doctrine.toml.example` template documents only `[conduct]`.

**Target:** two new leaf modules (`src/estimate.rs`, `src/value.rs`) provide
pure parse/validate and impure unit resolution. `dtoml.rs` gains two config
sections. `SliceDoc` gains two optional facet fields. Estimate and value are
completely independent — an entity may carry either, both, or neither.

## 3. Module: `src/estimate.rs`

### 3.1 Pure layer — types

`estimate.rs` imports only external crates (`toml`, `serde`, `anyhow`) — no other
doctrine module. This is the ADR-001 leaf guarantee.

```rust
/// The normalised estimation facet — two finite f64 bounds.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct EstimateFacet {
    pub lower: f64,
    pub upper: f64,
}

/// Deserialisation target before normalisation. `lower`/`upper` are raw TOML
/// values so integers and floats both arrive, and non-finite values are caught.
#[derive(Debug, Clone, Deserialize)]
struct EstimateRaw {
    lower: Option<toml::Value>,
    upper: Option<toml::Value>,
    /// `#[serde(flatten)]` on a `BTreeMap` collects every key NOT matching
    /// `lower`/`upper` — the Rust field name `_extra` is never a TOML key.
    /// This is the forward-compatibility mechanism (NF-003).
    #[serde(flatten)]
    _extra: BTreeMap<String, toml::Value>,
}
```

### 3.2 Pure layer — parse & validate

```rust
/// Parse an optional `[estimate]` table. Returns `Ok(None)` absent,
/// `Ok(Some(facet))` present+valid, `Err(_)` malformed. Bakes in validation —
/// callers never hold an invalid facet.
pub(crate) fn parse_optional(
    table: Option<&toml::value::Table>
) -> anyhow::Result<Option<EstimateFacet>>;

/// Normalise raw values to finite f64. Rejects missing bounds and non-finite
/// values. Pure — callers deserialise then call this.
fn normalise(raw: EstimateRaw) -> anyhow::Result<EstimateFacet>;

/// Validate a present estimate. Pure. Violations produce sentence-case errors.
pub(crate) fn validate(facet: &EstimateFacet) -> anyhow::Result<()>;
```

**Flow:**

1. `parse_optional(None)` → `Ok(None)`
2. `parse_optional(Some(table))` → `toml::from_str` → `EstimateRaw`, then:
   - Extract `lower`/`upper` from `toml::Value`:
     - `Integer(i)` → `i as f64` (exact for ≤ 2^53)
     - `Float(f)` → `f`
     - Absent → `Err("estimate: lower is required")`
   - Check finiteness: `f.is_finite()` else `Err("estimate: lower must be finite")`
   - Construct `EstimateFacet { lower, upper }`
   - Call `validate`:
     - `lower < 0.0` → `Err("estimate: lower must be >= 0")`
     - `upper < lower` → `Err("estimate: upper must be >= lower (got lower=X, upper=Y)")`

### 3.3 Config & unit resolution

```rust
/// Project-wide estimation config, parsed from `doctrine.toml [estimation]`.
#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct EstimationConfig {
    #[serde(default)]
    pub unit: Option<String>,
    /// Default confidence bounds for display / Monte Carlo / downstream use.
    /// Stored as fractions in [0.0, 1.0]; validated: finite, in range, lower < upper.
    /// No runtime effect in this slice — purely informational until consumed.
    #[serde(default)]
    pub lower_confidence: Option<f64>,
    #[serde(default)]
    pub upper_confidence: Option<f64>,
}

pub(crate) const DEFAULT_ESTIMATION_UNIT: &str = "espresso_shots";
pub(crate) const DEFAULT_LOWER_CONFIDENCE: f64 = 0.1;
pub(crate) const DEFAULT_UPPER_CONFIDENCE: f64 = 0.9;

/// Resolve the estimation unit. Pure over config — the file read is the shell's
/// job. Empty string falls back to default.
pub(crate) fn resolve_unit(cfg: &EstimationConfig) -> String;

/// Resolve the default confidence bounds. Pure. Each bound falls back to its
/// default when absent; validated: finite, in [0.0, 1.0], lower < upper.
pub(crate) fn resolve_confidence(cfg: &EstimationConfig) -> anyhow::Result<(f64, f64)>;
```

### 3.4 Custom Deserialize for EstimateFacet

```rust
impl<'de> Deserialize<'de> for EstimateFacet {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = EstimateRaw::deserialize(d)?;
        normalise(raw).map_err(serde::de::Error::custom)
    }
}
```

This lets `SliceDoc` use `#[serde(default)] estimate: Option<EstimateFacet>` directly —
serde handles the absent-table case via `Option`, and present-table → custom
deserializer normalises + validates.

**Serialization:** `#[derive(Serialize)]` produces compact normalised f64 values.
`None` → key omitted in TOML output. Unknown keys present at parse time are
absorbed by `_extra` on `EstimateRaw` (preventing parse errors) and dropped at
the normalise boundary — the normalised `EstimateFacet` carries only `lower`/`upper`,
so serialised output is the normalised truth. This satisfies NF-003 (tolerance,
not preservation).

## 4. Module: `src/value.rs`

### 4.1 Types

```rust
/// The value facet — a single finite f64 magnitude.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct ValueFacet {
    pub value: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct ValueRaw {
    value: Option<toml::Value>,
    #[serde(flatten)]
    _extra: BTreeMap<String, toml::Value>,
}
```

### 4.2 Parse

```rust
pub(crate) fn parse_optional(
    table: Option<&toml::value::Table>
) -> anyhow::Result<Option<ValueFacet>>;

fn normalise(raw: ValueRaw) -> anyhow::Result<ValueFacet>;
```

Identical pattern to Estimate, but simpler: one field, only finiteness check
(no range validation). Absent → `Ok(None)`. Missing value → `Err("value: value is required")`.
Non-finite → `Err("value: value must be finite")`.

### 4.3 Config

```rust
#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct ValueConfig {
    #[serde(default)]
    pub unit: Option<String>,
}

pub(crate) const DEFAULT_VALUE_UNIT: &str = "magic_beans";

pub(crate) fn resolve_unit(cfg: &ValueConfig) -> String;
```

## 5. Configuration wiring

### 5.1 `src/dtoml.rs`

```rust
use crate::estimate;
use crate::value;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct DoctrineToml {
    #[serde(default)] pub conduct: ConductConfig,
    #[serde(default)] pub verification: VerificationConfig,
    #[serde(default)] pub estimation: estimate::EstimationConfig,
    #[serde(default)] pub value: value::ValueConfig,
}
```

### 5.2 `install/doctrine.toml.example`

Add after `[conduct]` block:

```toml
# [estimation]
# unit = "espresso_shots"
# lower_confidence = 0.1
# upper_confidence = 0.9

# [value]
# unit = "magic_beans"
```

## 6. Entity-level wiring

### 6.1 `SliceDoc` (`src/slice.rs`)

```rust
// Eq dropped — f64 facets are PartialEq but not Eq.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct SliceDoc {
    id: u32,
    slug: String,
    title: String,
    status: String,
    created: String,
    updated: String,
    #[serde(default)]
    gate: Gate,
    #[serde(default)]
    estimate: Option<estimate::EstimateFacet>,
    #[serde(default)]
    value: Option<value::ValueFacet>,
}
```

`Meta` is unchanged — 4-field summary only. Facets are show-path detail.

### 6.2 `main.rs`

```rust
mod estimate;
mod value;
```

## 7. Test plan

### 7.1 `src/estimate.rs` tests

| # | Invocation | Expected | Maps to |
|---|---|---|---|
| E1 | `parse_optional(None)` | `Ok(None)` | FR-001 |
| E2 | `parse_optional(Some({lower=2, upper=8}))` | `Ok(Some({lower:2.0, upper:8.0}))` | FR-001 |
| E3 | `parse_optional(Some({lower=2.5, upper=8.0}))` | ok, float bounds | FR-001 |
| E4 | `parse_optional(Some({lower=2, upper=2}))` | ok, zero-width | FR-004 |
| E5 | missing `lower` | `Err("estimate: lower is required")` | FR-002 |
| E6 | missing `upper` | `Err("estimate: upper is required")` | FR-002 |
| E7 | `lower = nan` | `Err("estimate: lower must be finite")` | FR-001 |
| E8 | `lower = -inf` | `Err("estimate: lower must be finite")` | FR-001 |
| E9 | `lower = inf` | `Err("estimate: lower must be finite")` | FR-001 |
| E10 | `lower = -1` | `Err("estimate: lower must be >= 0")` | FR-002 |
| E11 | `lower=5, upper=2` | `Err("estimate: upper must be >= lower …")` | FR-002 |
| E12 | `resolve_unit(&EstimationConfig::default())` | `"espresso_shots"` | FR-003 |
| E13 | `resolve_unit(EstimationConfig{unit:Some("story_points"), ..})` | `"story_points"` | FR-003 |
| E14 | `resolve_unit(EstimationConfig{unit:Some(""), ..})` | `"espresso_shots"` | FR-003 |
| E15 | `resolve_confidence(&EstimationConfig::default())` | `(0.1, 0.9)` | — |
| E15a | `resolve_confidence(config with lower_confidence=0.2, upper_confidence=0.8)` | `(0.2, 0.8)` | — |
| E15b | `resolve_confidence(config with lower_confidence=nan)` | `Err("lower_confidence must be finite")` | — |
| E15c | `resolve_confidence(config with lower_confidence=0.5, upper_confidence=0.3)` | `Err("upper_confidence must be > lower_confidence")` | — |
| E15d | `resolve_confidence(config with lower_confidence=-0.1)` | `Err("lower_confidence must be in [0.0, 1.0]")` | — |
| E16 | `[estimate]` with `mode = "pert"` | parses ok, unknown key tolerated | NF-003 |
| E17 | `SliceDoc` round-trip: valid `[estimate]` | parse→serialise→parse, normalised bounds preserved | FR-004 |
| E18 | `SliceDoc` serde: no `[estimate]` table | `estimate` is `None`, serialises absent | — |
| E19 | `[estimate]` with extra keys (`mode = "pert"`) | parses ok (tolerated), serialises without extra keys | NF-003 |

### 7.2 `src/value.rs` tests

`value.rs` imports only external crates (`toml`, `serde`, `anyhow`) — ADR-001 leaf.
`_extra` semantics identical to EstimateRaw §3.1.

| # | Invocation | Expected | Maps to |
|---|---|---|---|
| V1 | `parse_optional(None)` | `Ok(None)` | |
| V2 | `parse_optional(Some({value=5}))` | `Ok(Some({value:5.0}))` | |
| V3 | `parse_optional(Some({value=3.5}))` | ok, float | |
| V4 | missing `value` | `Err("value: value is required")` | |
| V5 | `value = nan` | `Err("value: value must be finite")` | |
| V6 | `resolve_unit(&ValueConfig::default())` | `"magic_beans"` | |
| V7 | `[value]` with unknown keys | tolerated | |

### 7.3 `src/dtoml.rs` tests

| # | Invocation | Expected |
|---|---|---|
| D1 | `parse("")` | `EstimationConfig::default()`, `ValueConfig::default()` |
| D2 | `parse("[estimation]\nunit=\"x\"\n[value]\nunit=\"y\"")` | both set |

## 8. Verification alignment

| Requirement | How verified |
|---|---|
| FR-001 (estimate model, normalise) | Tests E1–E4, E7–E9 |
| FR-002 (validation matrix) | Tests E5–E6, E10–E11 |
| FR-003 (unit resolution) | Tests E12–E14 |
| FR-004 (round-trip) | Test E17 |
| NF-001 (non-blocking) | Structural: `rg estimate\|value src/slice.rs` — only `SliceDoc` field + deserialize, no gate/status/close references |
| NF-002 (kind-agnostic, pure) | `estimate.rs`+`value.rs` are leaves; `SliceDoc` wiring is one field each (mechanical) |
| NF-003 (forward compat) | Test E19, V7 — `#[serde(flatten)] _extra` tolerates unknown keys |

## 9. Code impact summary

```
src/estimate.rs          NEW  ~130 lines
src/value.rs             NEW   ~90 lines
src/main.rs              +2   (mod estimate; mod value;)
src/dtoml.rs             +3   (two config fields, two imports or inline)
src/slice.rs             +2   (two fields on SliceDoc)
install/doctrine.toml.example  +8  (commented sections)
```

## 10. Architecture decisions

- **D1 — Leaf modules (ADR-001).** Estimate and Value are siblings, not sub-modules of a shared "facets" parent. Each is independently testable and importable. They share no code beyond the `parse_optional → normalise → validate` pattern, which is not abstracted — the two are different enough (two-field vs one-field validation) that a shared abstraction would be heavier than the duplication.
- **D2 — Custom Deserialize.** Rather than a post-parse normalise step, each facet implements `Deserialize` directly. This keeps `SliceDoc` clean (one typed field per facet) and enforces validation at the serde boundary.
- **D3 — No template comments.** Entity TOML templates carry no commented `[estimate]`/`[value]` blocks — facets are opt-in and authored via CLI or explicit manual addition.
- **D4 — Show-path only.** Facets are `SliceDoc` fields, not `Meta` fields. List scans stay fast (4-field summary). Display and graph exposure will add their own readers in SL-102/SL-103.

## 11. Risks & assumptions

- **Assumption:** integer `as f64` is exact for all practical estimate values (≤ 2^53 ≈ 9×10^15). No lossy-cast handling needed.
- **Risk:** `toml` crate's handling of `nan`/`inf`/`-inf` may change across versions — tests E7–E9 are the canary.
- **Assumption:** `BTreeMap` for `_extra` in raw structs correctly absorbs unknown keys across `toml` crate versions (tested in E19, V7).
- **Open:** SL-102 (display) and SL-103 (graph) will need their own readers; by keeping facets out of `Meta`, we avoid coupling list performance to display/graph needs.
