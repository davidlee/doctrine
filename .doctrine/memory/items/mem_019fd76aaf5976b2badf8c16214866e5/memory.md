## The gotcha

`?` sequences **errors**, not **failures**. When a function's failure mode is a
fail-soft `Ok` variant rather than an `Err`, a caller written as

```rust
let written = do_the_write(..)?;   // "this can't have failed, we'd have returned"
let swept   = do_the_destructive_thing(..)?;
```

has no ordering guarantee at all — the `?` is inert against the failure that
matters, and the second act runs on the assumption the first one landed.

## The instance (SL-250, RV-348 F-12)

`plan_hook` turns both of its malformed-input conditions into
`RefreshOutcome::PrintedFallback` (`src/boot.rs:1159-1168`, `:1170-1178`) — a
diagnostic complete enough to repair by hand, and deliberately not an error,
because never-clobbering a settings file doctrine cannot parse is the right
posture. `install_hook_to_file` carries it out through `Ok(..)` at `:1611`.

A design ordered write-before-evict and argued the ordering guaranteed
"activation lands before removal". With a malformed *target* and a readable
*sibling*, the write produced nothing and the sweep **succeeded** — deleting the
only working copy. The ordering was correct; the type defeated it.

## What to check

When a design argues a safety property from the ORDER of two effects, ask what
the first one returns on failure. If any failure is representable as `Ok`, the
second effect needs an explicit guard on the outcome value, not on `?`:

```rust
let landed = !matches!(written, RefreshOutcome::PrintedFallback { .. });
```

Two corollaries:

- **Report the skip.** "Did not attempt" and "nothing to do" are different
  facts; collapsing them hides why both files still carry state.
- **The dangerous case is the one that succeeds.** A test for this asserts on
  what SURVIVES. If the guard is missing the destructive act works perfectly,
  so nothing errors and only a survival assertion goes red.

Sibling shape: [[mem.pattern.doctrine.review-dispose-settle-remedy-before-disposing]]
is the ledger analogue — a receipt that reports success is not evidence the
intended thing happened.
