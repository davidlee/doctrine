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

## In a `const fn`, `.get()` is not the escape hatch — slice patterns are

`.get(range).unwrap_or_default()` is not available in a `const fn` (`Option`'s
combinators are not const), and `v[i]` is still denied there. So a compile-time
walk over a slice recurses on slice patterns instead:

```rust
const fn same_token(left: &[u8], right: &[u8]) -> bool {
    match (left, right) {
        ([], []) => true,
        ([l, ltail @ ..], [r, rtail @ ..]) => *l == *r && same_token(ltail, rtail),
        _ => false,
    }
}
```

That is the house idiom already: `design_run/change_log.rs`'s `widest` recurses
`[head, tail @ ..]` over a closed vocabulary, and SL-256's `is_subset` proof over
the `READABLE`/`EMITTABLE` rosters is built the same way. Two related walls in the
same corner: `PartialEq` is not `const`, so `a == b` on `&str`/enums will not
compile in a `const fn` (compare `as_str().as_bytes()` with the walk above), and
comparing enum **discriminants** instead trips
[[mem.pattern.lint.as-conversions-ban]].
