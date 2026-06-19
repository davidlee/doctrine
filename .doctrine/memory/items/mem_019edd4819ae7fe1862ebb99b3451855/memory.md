# expect(dead_code) on a `mod foo;` decl propagates into the submodule

A `#[cfg_attr(not(test), expect(dead_code, reason = "…"))]` placed on a **module
declaration** (`pub(crate) mod foo;`) sets the lint level for `foo`'s items and is
*fulfilled* by any dead item inside the submodule. So a parent file can keep a
submodule's unconsumed `pub(crate)` items clippy-clean **without editing the
submodule file**.

- Use it to respect cross-slice file boundaries — don't touch another slice's file
  just to add dead-code attrs; attach one `expect` at the `mod` line you already own.
- Self-clearing tripwire: when the module gains a live consumer, the `expect` fires
  *unfulfilled* (its own warning) and forces removal — debt cannot rot silently.
- Verify against the project gate: plain `cargo clippy` (bins/lib, NOT
  `--all-targets`) — see [[mem.pattern.lint.clippy-denies]].

**Verified — SL-107 PHASE-01.** `src/estimate.rs` carries the expect on
`pub(crate) mod display;`; `src/estimate/display.rs`'s 3 unconsumed renderers
(`format_bound`, `format_estimate_normal`, `format_estimate_verbose`) stay clean,
plain clippy zero-warn, no unfulfilled-expect. House style is `expect` not `allow`
([[mem.pattern.lint.expect-not-allow]]).
