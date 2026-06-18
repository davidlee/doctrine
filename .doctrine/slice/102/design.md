# Design SL-102: SPEC-020: Estimate display rendering

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

SL-101 delivers the `EstimateFacet` model, parse, and validation. This slice adds
pure display-rendering functions that consume the normalized model and the resolved
project unit and produce human-readable output — a one-line header for normal
(`"Estimate: 2-8 espresso_shots"` or `"Estimate: none recorded"`) and detail lines
for verbose (`"  Attention spread: 4x"`, `"  Attention width: 6 espresso_shots"`).

## 2. Current State

`src/estimate.rs` exposes `EstimateFacet { lower: f64, upper: f64 }` (normalized,
validated), `EstimationConfig`, `parse_optional`, `normalise`, `validate`,
`resolve_unit`, `resolve_confidence`. No display functions exist. The eventual
caller (e.g. `doctrine slice show`) will compose these display functions into
entity output; the caller owns layout (indentation, ordering, blank lines).

## 3. Forces & Constraints

- **ADR-001**: the estimate module is a leaf (pure engine tier) — display
  functions must be pure: no clock, disk, rng, or git.
- **SPEC-020 D4**: normal display classifies nothing — no wide/risky/split-worthy
  labels.
- **FR-005 (REQ-273)**: acceptance criteria specify exact phrasing for present,
  absent, and the `lower == 0` spread-unavailable edge case.
- **POL-001**: no engineering-action-figure clankspeak — output reads as plain
  English.
- The functions must be unit-testable without project config or filesystem setup.

## 4. Guiding Principles

- Pure in, pure out. The caller resolves the unit and passes it in.
- The leaf owns display *content* (wording, number formatting); the caller owns
  display *layout* (prefix, indentation, grouping with other fields).
- No allocation ceremony: `String` and `Vec<String>` return types, no
  intermediate structs.

## 5. Proposed Design

### 5.1 System Model

New sub-module `src/estimate/display.rs`, declared via `pub(crate) mod display;`
in `src/estimate.rs`. Three public functions:

- `format_bound(f: f64) -> String` — compact bound formatting primitive.
- `format_estimate_normal(facet: Option<&EstimateFacet>, unit: &str) -> String` —
  one-line header.
- `format_estimate_verbose(facet: Option<&EstimateFacet>, unit: &str) -> Vec<String>` —
  detail lines for verbose view.

No new types; no new dependencies.

### 5.2 Interfaces & Contracts

#### `format_bound`

```rust
pub(crate) fn format_bound(f: f64) -> String;
```

Rounds to 1 decimal place: `(f * 10.0).round() / 10.0`. If the fractional part is
within `f64::EPSILON` of zero, formats as integer (`"{:.0}"`); otherwise formats
with 1 decimal (`"{:.1}"`). Examples:

| Input | Output |
|---|---|
| `0.0` | `"0"` |
| `2.0` | `"2"` |
| `2.5` | `"2.5"` |
| `3.75` | `"3.8"` |
| `2.33333` | `"2.3"` |
| `0.1` | `"0.1"` |
| `100.0` | `"100"` |

Assumes finite input (guaranteed by the parse/normalise path).

#### `format_estimate_normal`

```rust
pub(crate) fn format_estimate_normal(
    facet: Option<&EstimateFacet>,
    unit: &str,
) -> String;
```

- `Some(f)` → `"Estimate: {format_bound(lower)}-{format_bound(upper)} {unit}"`
- `None` → `"Estimate: none recorded"`

The `Option` arm hides absent from the caller — no match needed at the call site.

#### `format_estimate_verbose`

```rust
pub(crate) fn format_estimate_verbose(
    facet: Option<&EstimateFacet>,
    unit: &str,
) -> Vec<String>;
```

- `None` → `Vec::new()` (empty — caller renders only the normal line).
- `Some(f)` where `f.lower > 0.0`:
  - `"  Attention spread: {ratio}x"` (ratio = upper / lower, same format_bound
    heuristic, suffix `"x"`)
  - `"  Attention width: {format_bound(upper - lower)} {unit}"`
- `Some(f)` where `f.lower == 0.0`:
  - `"  Attention spread: ratio unavailable"`
  - `"  Attention width: {format_bound(upper - lower)} {unit}"`

Lines carry 2-space indent so the caller can prepend them directly without
prefix logic.

### 5.3 Data, State & Ownership

No state. Pure functions borrow their inputs. No heap allocation beyond the
returned `String`s.

### 5.4 Lifecycle, Operations & Dynamics

Called at display time only — no caching, no memoisation.

### 5.5 Invariants, Assumptions & Edge Cases

- Input `f64` values are finite (parse/normalise guarantees this).
- `unit` is a non-empty string (the caller resolves the default). An empty
  `unit` produces grammatically odd output but does not panic.
- Zero-width estimate (`lower == upper`): normal renders `"Estimate: 2-2 shots"`;
  verbose shows spread `1x`, width `0 shots`.
- `lower == 0, upper > 0`: spread unavailable, width shown.
- `lower == 0, upper == 0`: spread unavailable, width `0 shots`.

## 6. Open Questions & Unknowns

None — foundations resolved during design conversation.

## 7. Decisions, Rationale & Alternatives

- **D1 — Display sub-module, not inline.** `src/estimate/display.rs` with a
  one-line `pub mod display` declaration. No file moves, no import churn. The
  separation keeps the module surface scannable and avoids a long file.
  Alternative: inline at the bottom of `estimate.rs` (rejected — separation is
  cheap and future display additions won't bloat the parent).

- **D2 — Three functions, not one enum-driven.** `format_bound`,
  `format_estimate_normal`, `format_estimate_verbose` — each single-purpose. The
  caller composes. Alternative: single `FormatMode` enum (rejected — couples the
  caller's mode choice into the leaf; the leaf should not know about "normal" vs
  "verbose" as concepts, only produce the right output for each).

- **D3 — `Vec<String>` for verbose, not a struct.** The caller prints lines in
  order; a struct with optional spread would require the caller to pattern-match
  on the `lower == 0` edge case that the leaf already handled. Returned lines are
  self-contained. Alternative: `VerboseDetail` struct (rejected — adds a type for
  a single call site's convenience).

- **D4 — Bound formatting: truncate to 1 decimal, strip trailing zero.**
  `format_bound` rounds to 1 decimal place and omits `.0`. The rounding is
  display-only — `EstimateFacet` holds full `f64` precision unchanged. For the
  precision range we deal with (bounded integer-like attention estimates), 1
  decimal place is sufficient; if fractional estimates become common, precision
  can be lifted later without breaking the callers.

## 8. Risks & Mitigations

- **R1 — Floating-point rounding surprises.** `format_bound` uses
  `(f * 10.0).round() / 10.0` then checks `(rounded - rounded.trunc()).abs() <
  f64::EPSILON`. This can misclassify values very close to an integer (e.g.
  `0.999999999999` → `"1"`). Mitigation: bounds are authored as TOML numbers and
  normalised from integer/float sources; the parse path preserves the exact TOML
  value in `f64`. The EPSILON check handles the residual rounding from the
  rounding step itself, not the original value.
  Severity: low.

## 9. Quality Engineering & Validation

Tests live in `src/estimate/display.rs` — a `#[cfg(test)] mod tests` block at the
bottom of the file, alongside the display functions. Test cases:

**VT-1 `format_bound`** — each row in the table at §5.2, plus
`2.000000000001` → `"2"` (epsilon noise near integer).

**VT-2 `format_estimate_normal` present** — integer bounds (`lower=2, upper=8`)
produces `"Estimate: 2-8 espresso_shots"`.

**VT-3 `format_estimate_normal` absent** — `None` produces
`"Estimate: none recorded"`.

**VT-4 `format_estimate_normal` float bounds** — `lower=2.5, upper=8.0` produces
`"Estimate: 2.5-8 espresso_shots"`.

**VT-5 `format_estimate_verbose` absent** — `None` returns empty vec.

**VT-6 `format_estimate_verbose` normal** — `lower=2, upper=8` returns
`["  Attention spread: 4x", "  Attention width: 6 espresso_shots"]`.

**VT-7 `format_estimate_verbose` lower-zero** — `lower=0, upper=5` returns
`["  Attention spread: ratio unavailable", "  Attention width: 5 espresso_shots"]`.

**VT-8 zero-width estimate** — `lower=5, upper=5`: normal shows `"Estimate: 5-5
shots"`, verbose shows spread `1x`, width `0 shots`.

## 10. Review Notes

### Adversarial review (2026-06-19)

- **F1** — Verbose-line 2-space indent is layout, not content, but matches the
  project's existing indent convention (`format_show` uses `"  "`). Keep.
- **F2** — Authored fractional values like `2.05` round to `2` due to inherent
  f64 binary representation (the stored value is `2.0499...`). Any 1-decimal
  approach would produce the same result. Academic for attention-burden estimates.
- **F4** — Added VT-1 epsilon-noise test case.

### Design notes

- Polished bound formatting edges (VT-8) — zero-width estimates must render
  cleanly.
- Ratio formatting reuses the `format_bound` heuristic, appending `"x"` suffix.
- The `unit` parameter is always passed — if the shell hasn't resolved it yet,
  the caller handles that; the leaf does not default.
