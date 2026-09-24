`cargo clippy --bin doctrine` is run with `-D clippy::shadow-unrelated` (repo lint policy). The lint fires when a binding is shadowed by a *later, unrelated* binding — the common shape is a new `let` introduced at function scope while a closure or `map` already binds the same name.

Concrete case (SL-262 PHASE-01): `commands/design.rs` `apply` gained

```rust
let declared = request.checkpoint_act.as_ref().and_then(|act| act.disposition.as_ref());
```

and already held, further down in the same function,

```rust
declaration_fingerprint: request.agent_declaration.as_ref().map(|declared| { … })
```

Clippy reported `declared shadows a previous, unrelated binding` pointing at the closure parameter, and the build failed. The fix is a rename, not an `allow`: the new outer binding became `declared_review`.

Diagnosis is cheap — `cargo clippy --bin doctrine` (the `check quick`/`check gate` verbs run it for you and stop on the error). Do NOT pass `--all-targets`: that turns on the `unwrap_used`/`expect_used` denials that are legitimate in test code.

Related: [[mem.pattern.lint.expect-not-allow]], [[mem.pattern.lint.string-build-no-push-format]].
