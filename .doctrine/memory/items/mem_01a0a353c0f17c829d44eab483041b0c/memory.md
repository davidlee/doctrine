## The question

A `const _: () = assert!(f(TABLE) <= BOUND);` is the house idiom here for
*proved rather than asserted* (`design_run/gate.rs`, `change_log.rs`,
`commands/design.rs`). Its helper `f` usually has no other reader. So: does
deleting the assert break the build, or silently orphan the helper?

**It depends on the module, and the two live examples answer differently.**

## `design_run` — the proof does NOT defend itself

`src/design_run/mod.rs` carries a module-wide

```rust
#![cfg_attr(not(test), expect(dead_code, reason = "SL-233 PHASE-03/04 land the first non-test callers"))]
```

so in a non-test build dead code is *expected*, not denied. That is why
`change_log.rs`'s comment on the `is_subset(&EMITTABLE, &READABLE)` proof is
right: delete it and `cargo check` passes; `cargo test --bin doctrine` then
fails to compile, because the exemption is `not(test)`-scoped and lapses there
while both roster tests live in a different compilation unit
(`tests/e2e_design_state.rs`).

## `commands` — the proof DOES defend itself

`src/commands/design.rs` has no such exemption, so the crate's denied `unused`
applies in the ordinary build. Deleting `widest_canonical_id`'s assert gives, at
`cargo check`:

```
error: function `decimal_digits` is never used
error: function `widest_canonical_id` is never used
```

Probed directly (SL-259 PHASE-06 `P5`), which refuted the phase sheet's own risk
entry — it had assumed the `EMITTABLE` fragility generalised.

## The rule

Before writing "do not clean this up, nothing will catch it" on a const proof,
**check the enclosing module for a `cfg_attr(not(test), expect(dead_code))`** and
probe it. The sentence is a claim about the module's lint posture, not about
const proofs, and the two are routinely confused. Helper visibility does not
decide it either — private `fn`s are caught in `commands` and exempt in
`design_run`.

Related: [[mem.pattern.rust.expect-dead-code-is-per-compilation-unit]],
[[mem.pattern.lint.dead-code-derives-count-as-reads]],
[[mem.pattern.lint.dead-code-staged-ahead-cfg-test]].
