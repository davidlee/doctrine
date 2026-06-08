# Repo clippy denies indexing-slicing: use .get(range) not range-indexing

`Cargo.toml [lints]` sets `clippy::indexing-slicing = "deny"`. Any panicking
index/slice in **bin/lib code** fails `just check`:

- `&v[a..]`, `&s[a..b]`, `v[i]`, `s[..n]` — all denied.
- Use `v.get(a..).unwrap_or_default()` (slices → `&[]`),
  `s.get(a..b).unwrap_or("")` (str), `v.get(i)` (element).

The gate is plain `cargo clippy` (bins/lib only, not `--all-targets`), so test
code can still index freely — the deny bites only production code.

## How to apply

When recovering a sub-slice whose bound you just computed and know is valid,
`.get(range).unwrap_or_default()` is the house idiom — the fallback is unreachable
but satisfies the lint without an `expect` (which is also denied in non-test code,
see [[mem.pattern.lint.expect-not-allow]]). For byte offsets into a `&str` (e.g.
attributing a `toml::de::Error` span to its source line), `.get(..byte)` /
`.get(byte..)` also dodges non-char-boundary panics for free.

Hit in SL-022 PHASE-03 (`registry.rs` cycle-slice recovery, `spec.rs`
`enclosing_line`). Companion to [[mem.pattern.lint.clippy-denies]],
[[mem.pattern.lint.string-build-no-push-format]],
[[mem.pattern.lint.disallowed-types-collections]],
[[mem.pattern.lint.as-conversions-ban]].
