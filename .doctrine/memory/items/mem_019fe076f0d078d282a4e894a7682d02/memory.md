# Staging a type ahead of its consumer: check the derives before reaching for `expect(dead_code)`

[[mem.pattern.lint.dead-code-staged-ahead-cfg-test]] says to gate an item that
lands a phase before its consumer with
`#[cfg_attr(not(test), expect(dead_code, reason = "…"))]`, so it can ship
without a blanket suppression. Sound — but its precondition is that **nothing**
reads the item under `cfg(not(test))`, and a `struct` with value-semantics
derives does not meet it.

**Measured (SL-248 PHASE-02, `src/interpretation.rs`).** `InterpretationPolicy`
derives `Debug, Clone, PartialEq, Eq`. Its fields' only *hand-written*
production readers were two tasks away, so the attribute went on the struct.
rustc rejected it immediately:

    error: this lint expectation is unfulfilled
    `-D unfulfilled-lint-expectations` implied by `-D warnings`

The **derived `Clone` and `PartialEq` impls read every field**, so the fields
are live from the moment the type exists. `Debug` alone does *not* count —
rustc deliberately ignores it for dead-code purposes — so "a derive keeps it
alive" is not a reliable rule of thumb in either direction.

**How to decide, cheaply.** Do not reason about it; compile. Add the item
without the attribute and see whether `dead_code` actually fires. Under
`warnings = "deny"` both outcomes are hard errors, so the compiler answers in
one cycle either way — and an unfulfilled `expect` is the *more* confusing
error of the two, because it reads as if the lint config is wrong rather than
the premise.

**Where the original pattern still applies:** free functions, consts, and
structs carrying no derive beyond `Debug` — items with genuinely no reader
until the next phase. See also
[[mem.pattern.lint.dead-code-blanket-masks-siblings]] and
[[mem.pattern.lint.expect-not-allow]].
