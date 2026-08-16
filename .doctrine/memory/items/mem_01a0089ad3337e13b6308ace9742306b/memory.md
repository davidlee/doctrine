An unused private `const` or `fn` added to this crate compiles **clean** under
`cargo build` and `cargo clippy`, and is a **hard error** under the `cargo test`
compilation:

```
error: constant `ORPHAN_CONST_SL251_PROBE` is never used
  = help: to override `-D unused` add `#[expect(dead_code)]` or `#[allow(dead_code)]`
```

Verified by direct probe (SL-251 PHASE-07, 2026-08-16): append an orphan const
and fn to a `src/design_run/` file, then run all three. Build and clippy both
recompile — `Compiling doctrine` appears — and emit nothing at all. Only
`cargo test --bin doctrine --no-run` errors.

## What this does and does not mean

**It does NOT mean dead code accumulates silently.** `just check` and
`doctrine check gate` both run the test compilation, so a new dead item is
caught before it can land. There is no hole here and no backlog item is owed.

**It does mean the fast inner loop lies by omission.** An agent iterating on
`cargo build` or `cargo clippy` will not see the item that is about to red the
gate, which is the inverse of the usual expectation that clippy is the stricter
of the two. Reach for `cargo test --bin doctrine --no-run` when the question is
"is this item actually reachable".

A corollary for the `#[expect(dead_code)]` dance: an expectation is *unfulfilled*
— itself a `-D warnings` error via `unfulfilled_lint_expectations` — the moment
any reader exists **in that compilation unit**. So an item read only from
`#[cfg(test)]` code needs `#[cfg_attr(not(test), expect(dead_code, ...))]`, and an
item that gains a production reader needs the attribute **deleted outright**.
Both directions occurred one phase apart in SL-251 and each is a build error if
taken the wrong way. See [[mem_019fe235df747032a582772c4bd595bc]].

Do not re-investigate the *cause* of the build/clippy asymmetry from scratch: it
cost roughly six build cycles in SL-251 PHASE-07 and was not resolved. The
behaviour above is the actionable part.
