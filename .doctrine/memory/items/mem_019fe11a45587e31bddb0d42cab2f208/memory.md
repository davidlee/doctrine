# A module staged ahead of its consumer has no per-task red/green

[[mem.pattern.lint.dead-code-module-level-vs-cfg-attr]] tells you to gate a
whole module landed a phase early with
`#![cfg_attr(not(test), expect(dead_code, reason = "…"))]`. That is right, and
it has a consequence the pattern does not state.

Under `cargo test` the `cfg_attr` is **stripped** — `cfg(test)` is on, so the
expectation does not apply. This workspace sets `unused = "deny"`, so *every*
item the tests do not yet reach is a hard error. A module whose production
surface lands complete therefore **does not compile at all** until the last test
that names the last item exists.

**Measured (SL-248 PHASE-04, `crates/doctrine-control/src/backend.rs`).** The
phase sheet sequenced fourteen tasks as red/green/refactor cycles. After the
task that landed the types, `cargo test` produced **69 errors**, all of the form
`constant X is never used` / `associated items new, host and inner are never
used` — including constants that *are* referenced, by a `const` slice that is
itself dead, so the whole chain reads dead. There was no partial-green state to
stand on: the first successful compile of the phase came after ~30 tests, and
everything passed at once.

## What to do instead

1. **Sequence by task, but expect one compile.** Write each task's tests in
   order — the ordering still buys you the design pressure — and accept that the
   compiler is silent until the last one lands. Do not chase the intermediate
   error list; it is noise about items whose tests you have not written yet.
2. **Recover the red phase by mutation, not by ordering.** Everything passing on
   the first compile is not evidence. Snapshot the green file, then break one
   rule at a time and confirm the intended test — and *only* that test — reds.
   See [[mem.pattern.tests.guard-needs-a-discriminating-difference]]: a mutation
   battery answers exactly its question, "what would this have printed if the
   guard were deleted?"
3. **Restore by copy, never `git checkout`**
   ([[mem.pattern.git.revert-control-by-edit-not-checkout]]).

## What this is not

Not the `unfulfilled` failure of
[[mem.pattern.lint.dead-code-derives-count-as-reads]]. That one is about the
*non-test* build, where derives make fields live and the `expect` goes
unfulfilled. This is the mirror build: `cfg(test)` on, attribute gone, `unused`
denied. The module-level `cfg_attr(not(test), …)` spelling took ~15 new types
without a fight here; the cost it carries is the loss of intermediate compiles.
