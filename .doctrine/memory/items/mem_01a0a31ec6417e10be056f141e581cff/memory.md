# Diagnostics name a closed vocabulary through its own `as_str`, never `Debug`

`Cargo.toml` denies `clippy::use_debug` repo-wide. So a `Display` arm — a
refusal, a cause, an error — **cannot** render an enum with `{:?}`. It needs a
`const fn as_str(self) -> &'static str`.

`src/design_run/gate.rs`'s `impl Display for Cause` states the rule at the site:
*every token is the closed vocabulary's own `as_str` — `clippy::use_debug` is
denied and a second spelling of a serde-derived token is the drift STD-001
forbids.*

## The apparent contradiction, resolved

`ChangeEvent`'s doc warns that `#[serde(rename_all)]` **plus** a hand-written
`as_str` is two sources for one wire token, and that the pair had already
drifted once — a renamed variant moved serde's token while `as_str`'s stayed,
and both halves still compiled. It solved that by **replacing** the derive with
`#[serde(try_from = "String", into = "String")]` driven off `as_str`.

That is the exception, not the pattern. `ActKind`, `Provenance`,
`InquiryLifecycle` and `ValueKind` all carry `rename_all` **and** an `as_str`.

## What to do when you add one

Write the `as_str`, keep `rename_all`, and **pin the two spellings against each
other** with a test — one round trip per member:

```rust
for kind in ValueKind::ALL {
    assert_eq!(
        serde_json::to_string(&kind).expect("a fieldless enum serialises"),
        format!("\"{}\"", kind.as_str()),
    );
}
```

That is the same guarantee `ChangeEvent`'s hand-written impls buy, for one test
instead of two impls, and it fails loudly on the rename that started all this.
It needs an `ALL` const to iterate, which is worth having anyway — every sweep
over the vocabulary should read its members from the enum rather than from
whichever ones a test author remembered.

Landed as `value_kind_tokens_match_their_serde_spelling`, `SL-259` `PHASE-05`.
Probe-verified: respelling one arm of `as_str` fails it, naming the member.

Related: [[mem.pattern.lint.clippy-denies]], [[mem.pattern.lint.expect-not-allow]].
