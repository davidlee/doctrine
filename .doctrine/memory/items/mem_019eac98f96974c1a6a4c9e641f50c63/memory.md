# Self-clearing dead_code expect breaks when the leaf has a cfg(test) round-trip test

Refines `mem.pattern.lint.dead-code-self-clearing-leaf`. That pattern recommends a
module-level `#![expect(dead_code, reason=…)]` for a pure leaf built ahead of its
consumer, and notes "`#[cfg(test)]` usage does NOT count." That last clause is only
true in the *non-test* build. The sharp edge:

`expect` is fulfilled per *compilation*. Under plain `cargo clippy` / `cargo build`
(the gate; `cfg(test)` inactive) the leaf is genuinely dead → the expect is
fulfilled. But under `cargo test` (`cfg(test)` active) a round-trip test that names
the symbols is a real use → `dead_code` does NOT fire → an **unconditional**
`expect(dead_code)` becomes **unfulfilled** → `unfulfilled_lint_expectations`
errors out the test build.

So the original pattern's unconditional expect only survives when *nothing*,
including a test, uses the leaf. The moment a verification criterion demands a
serde/round-trip test over the stub (the common case for an enum landed as
vocabulary), you must scope the suppression to exactly the build where it's dead:

```rust
#[cfg_attr(not(test), expect(dead_code, reason = "… deferred consumer …"))]
```

Self-clearing property is preserved: when a real *non-test* consumer lands, the
`not(test)` expect goes unfulfilled and forces its removal — test references never
mask that. Item-level `cfg_attr` here, not module-level, since only the one stub
type is dead.

Precedent: SL-028 PHASE-03 `CoverageStatus` (`src/requirement.rs`) — landed as
vocabulary with a VT-2 round-trip naming all five variants; an unconditional
`expect` fired unfulfilled under `cargo test` until scoped `cfg_attr(not(test), …)`.
