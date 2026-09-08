# ISS-448: Unpinned toolchain reds the gate on untouched files

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What happened

`SL-256` PHASE-01 ran `doctrine check gate` on a tree whose only edits were in
`src/design_run/` and `tests/`. The gate failed at the `lint` recipe — **before
tests run at all** — with four clippy errors in files the phase never touched:

- `src/estimate/display.rs:20,41,77` — `clippy::assert_is_empty` on
  `debug_assert!(!unit.is_empty())`.
- `src/map_server/state.rs:71` — `clippy::double_must_use`, fired from inside
  the `async_trait` macro expansion on `trait DotRenderer`.

`git status` confirmed both files byte-identical to `HEAD`. Clippy lints are
per-item, so nothing in `design_run/` could change how `estimate/display.rs`
lints.

## Cause

The toolchain moved under the repo. Clippy was `0.1.99 (948ec4e1ec 2026-09-04)`;
the tree's last commit was three weeks older. New lints — or newly macro-aware
ones — turn previously-clean code into `-D warnings` errors with no source
change. This is `IMP-273` (*Pin one exact Rust toolchain across dev + CI*)
arriving as a concrete cost rather than a hypothetical.

## Why it is worth an issue of its own

The blast radius is not the four lints. It is that **any** slice, on **any**
phase, can have its exit gate red-lit by code it did not write, at a moment when
the implementor's cheapest move is to widen their diff past the slice's fenced
selectors to get green. That silently defeats `doctrine slice conformance`, which
exists to catch exactly that widening. The failure mode is procedural, not
cosmetic.

It also lands at the worst point in the recipe: `lint` runs before `test`, so a
toolchain bump hides the test result entirely.

## What was done at SL-256

Fixed in passing on the user's explicit call, not silently:

- the three asserts became `debug_assert_ne!(unit, "", "…")`, which is what the
  lint suggests and is better code — the failure now shows the value;
- the `async_trait` one became `#[expect(clippy::double_must_use, reason = …)]`.
  `allow` was the first attempt and is not available: `clippy::allow_attributes`
  is denied here, which is the machine enforcement behind
  `mem.pattern.lint.expect-not-allow`. Note the residual — an `expect` is itself
  checked, so on a toolchain that stops firing `double_must_use` it goes
  unfulfilled and errors the other way. Unpinned, that cuts both directions.

Both files were added to `SL-256`'s selector list as `scope-relevant` with a
note saying they are not slice substance, so the conformance report declares the
widening rather than being surprised by it.

## What is actually being asked for

Not the four lints — those are cleared. Either:

1. pin the toolchain (`rust-toolchain.toml`) so a bump is a deliberate, reviewed
   change with its own slice — this is `IMP-273`, and this issue is the evidence
   for it; or
2. decide the gate should tolerate pre-existing lint debt outside the slice's
   selectors, and give implementors a sanctioned route to a scoped gate.

(1) is the obvious one. (2) is recorded because it is the alternative someone
will reach for under time pressure, and it should be chosen rather than drifted
into.
