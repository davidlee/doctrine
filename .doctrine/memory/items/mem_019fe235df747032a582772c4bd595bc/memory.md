# `#[expect(dead_code)]` belongs on the item, not on the field

**Measured (SL-248 PHASE-07, `crates/doctrine-control/src/conformance.rs`).**
A module lands its production surface a phase ahead of its consumer, so it is
dead under `cfg(not(test))` and live under `cfg(test)`. Two fields of a
`pub(crate) struct Row` are read only by the next phase's harness, so they are
dead in **both** builds. The obvious move — an `#[expect(dead_code, reason =
"…")]` on each of the two fields — fails:

    error: this lint expectation is unfulfilled
    --> conformance.rs:465:9   (the field-level expect)
    = note: `-D unfulfilled-lint-expectations` implied by `-D warnings`

**Why.** rustc reports dead code at the **outermost dead item** and never
descends into it. Under `cfg(not(test))` the whole `struct Row` is dead, so the
lint fires *at the struct* — caught by the module-level blanket — and no
field-level lint is ever emitted. The field-level expectations are therefore
unfulfilled in that compilation unit, and `unfulfilled_lint_expectations` is a
hard error under `-D warnings`.

**The fix is one line up.** Put the expectation on the **item** — the struct,
the enum — not on its fields or variants:

- under `cfg(test)`, the item is live and the *field* lints fire → the
  item-level expectation catches them;
- under `cfg(not(test))`, the *item* lint fires → the same expectation catches
  that instead.

One attribute, fulfilled in both units, for opposite reasons. The same applies
to an uninhabited or never-constructed enum variant: expect on the `enum`, not
on the variant.

**Diagnosis smell.** The error points at a *narrowing* you made in good faith
and reads as if the attribute is wrong, when what is wrong is its **altitude**.
If an `expect` is unfulfilled in exactly one of the two builds, you have almost
certainly attached it below the level rustc reports at.

**Corollary — check both units.** `cargo test` and `cargo clippy` compile
different units of the same file. A green `cargo test` says nothing about
`cargo clippy`, and this class of error appears in only one of them. See
[[mem.pattern.rust.expect-dead-code-is-per-compilation-unit]] and
[[mem.pattern.lint.dead-code-derives-count-as-reads]].
