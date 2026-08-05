`#[expect(lint)]` is checked **per compilation unit**, and the design e2e suites
are separate units that `#[path]`-include the same source. So an item marked
`#[expect(dead_code)]` because nothing in `src/` reads it will fail an e2e binary
that *does* read it — `error: this lint expectation is unfulfilled`, under
`-D warnings` — while the `src` build stays correct. Same source, two answers.

Observed on `AcceptanceAttestation::authority()` (SL-244 PHASE-05 `T13`, `F59`).

**`cfg_attr(not(test), expect(...))` does not rescue it.** That idiom exists for
the phase-staging case ([[mem.pattern.lint.dead-code-staged-ahead-cfg-test]]),
and it works when a *unit* test reads the item. Here the src unit-test build has
no reader either, so stripping the expect under `cfg(test)` just moves the
failure — the fix would be a unit test written only to keep a marker honest.

**Route around it instead.** Where the item is a plain accessor over a serde
type, assert the **persisted** value:

```rust
assert_eq!(
    serde_json::to_value(&record.acceptance).unwrap()["authority"],
    serde_json::to_value(AcceptanceAuthority::User).unwrap(),
);
```

Both sides come from the type's own serde form, so no token is re-typed
(STD-001), it touches no production source, and what is asserted is what a later
reader of the snapshot actually finds — arguably the better assertion.

**The general rule:** an accessor carrying `expect(dead_code)` is *not available*
to an e2e suite. Read through serde, or accept that giving it a reader means
editing the marker in production source.

This is the third face of the trap SL-244 PHASE-05 kept meeting: (1) an `expect`
whose subject gains a reader must be **deleted**, not left; (2) `cfg(test)`
strips the suppression, so a gated declaration needs a test reader in the same
task; (3) this one — the reader and the marker can live in different units.
