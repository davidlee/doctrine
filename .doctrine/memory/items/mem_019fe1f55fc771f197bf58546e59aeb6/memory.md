## The gotcha

`anyhow::Error`'s `Display` renders **only the outermost context**. So the
repo's standard refusal idiom —

```rust
some_pure_fn(..)
    .map_err(|refused| anyhow::anyhow!("{refused}"))
    .with_context(|| format!("checkpoint {subject} declares ..."))?
```

— produces an error whose `to_string()` is *the context line alone*. The typed
refusal underneath is in the chain, not in `Display`.

A test written the obvious way,

```rust
assert!(error.to_string().contains(&expected_refusal));  // WRONG
```

fails even though the production code is correct. Worse is the near-miss: if
the context line happens to contain the substring being asserted (a field name,
an id), the assertion **passes without ever reading the cause** — so the test
that was supposed to pin the refusal's identity pins nothing.

## The fix

Use the **alternate** flag, which renders the chain joined by `": "`:

```rust
let error = format!("{error:#}");
assert!(error.contains(&expected_refusal), "...: {error}");
```

This is also what the user is shown, so the assertion is over the real surface
rather than an internal fragment of it.

## Where it bites

Anywhere a shell converts a typed refusal to `anyhow` and adds context — which
in this repo is the dominant shape at `plan_checkpoints`, `adoptable`, and the
`RecordKind::from_str` arm. Cost the SL-249 PHASE-06 worker one red/green cycle
on a test that was already asserting the right thing.

## Related

`mem.pattern.doctrine.error-mapping-by-string-parsing-is-fragile` covers the
opposite direction (don't *parse* the string); this one is about *reading* it in
a test.
