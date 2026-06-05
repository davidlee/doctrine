# Repo bans `as` casts: guarded saturating float→int with expect

Repo `[lints]` denies `clippy::as_conversions`, `clippy::cast_sign_loss`,
`clippy::cast_possible_truncation`, `clippy::cast_precision_loss` (and
`indexing_slicing`). There is **no `as` anywhere in `src`** by design
(see `src/memory.rs:125`) — prefer `TryFrom`/`try_into`, `usize::try_from`, etc.

## The float→int exception

`f32`/`f64` → integer has **no safe std API** (`TryFrom` is not implemented for
floats). When you genuinely need it (e.g. `lexical::quantize`, the first such
site in the repo), use a **single guarded saturating cast**:

- Rust's float→int `as` is **saturating since 1.45**: NaN→0, out-of-range clamps
  to the int's min/max. So if the float is already finite and `>= 0`, `x as u32`
  is total, monotonic, and saturates — no manual ceiling branch needed.
- Suppress with the house style — stacked `#[expect]` + a single `reason`, never
  bare `allow` (see [[mem.pattern.lint.expect-not-allow]]):

```rust
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "saturating float→u32 (Rust >= 1.45); value is finite and >= 0; no safe std API"
)]
let q = scaled as u32;
```

Pair it with a `debug_assert!(x.is_finite())` if a non-finite input means an
upstream bug (debug explodes, release degrades) — and profile-split the test
(`#[cfg(debug_assertions)] #[should_panic]` vs `#[cfg(not(debug_assertions))]`).

Related: [[mem.pattern.lint.clippy-denies]], [[mem.pattern.lint.expect-not-allow]].
