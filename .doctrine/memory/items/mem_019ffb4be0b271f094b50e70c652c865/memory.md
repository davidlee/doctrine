# A move's closure is the definition's needs, not the consumer's imports

**The error mode.** Planning a symbol re-home by asking *"what does the surviving
consumer import from the dying module?"* gives you a set that is correct as an
answer to that question and **wrong as a work estimate**. It enumerates the
module's exported surface toward one caller. It does not see what the moved
*definitions themselves* reach for — private constants, private helpers, local
type aliases — none of which appear in any importer's `use` line.

**Measured (SL-254 PHASE-01, `src/worktree/`).** The design established that
`jail_prefix.rs` imports "precisely four primitives from `pretooluse.rs` and
nothing else", and concluded *"the re-home is complete and no fifth primitive is
hiding."* Both halves were true; the conclusion did not follow. Moving
`have_bwrap` also required two **private** consts it reads — `BWRAP_BIN` and
`ENV_PATH` — neither of which any consumer could name. Closure: five items, not
four.

**The second-order find is the valuable one.** `BWRAP_BIN` turned out to
duplicate a constant the *destination* already owned (`jail.rs`'s `BWRAP`), so
the move surfaced a live [[mem.pattern.doctrine.conventions]] STD-001 violation
that neither module's own review had. Taking a definition's closure walks you
into the destination's existing vocabulary, which is where duplicates live.

## What to do

1. **Ask both questions.** *What does the consumer import* sizes the interface.
   *What does each moved definition reference* sizes the work. Run the second by
   reading each moving item's body, not its signature.
2. **Diff the closure against the destination before moving.** A name in the
   closure that already exists at the destination is a duplicate to collapse, not
   a second copy to carry. That is a decision worth making at plan time, because
   it changes which call sites get touched.
3. **Expect the orphans to be compile errors, not warnings**, wherever `unused`
   or `warnings` is denied — see [[mem.pattern.lint.staged-module-unused-deny-collapses-tdd]].
   The asymmetry to plan for: an item whose sole *production* consumer leaves is
   dead in the non-test build while still live under `cfg(test)`. The fix is
   almost always to move the import down into the test module, not to reach for
   `#[expect(dead_code)]` — cf. [[mem.pattern.lint.expect-dead-code-at-item-level]]
   for when the expectation genuinely is the answer.

## Scope of the claim

This is about **surveys that stand in for work estimates**. The consumer-import
question is still the right one for proving an interface is fully re-pointed —
`SL-254`'s `EX-2` (`grep -c super::pretooluse` = 0) used exactly that and it held.
Keep it; just do not let it also answer "what moves".
