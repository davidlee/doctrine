## The smell

A test double that returns a *queue* of canned outcomes — `scripted(vec![ok, ok,
ok], fallback)` — encodes the call sequence's **length** into every fixture that
uses it. Change the number of calls and every test that merely wanted a
successful run reds, including tests about entirely unrelated rules.

SL-248 PHASE-06 measured it: a mutation collapsing three clone executions into
one redded four tests. Two were the mutation's real targets; two were fixtures
whose script positions no longer lined up — one because it indexed `calls[3]`,
one because its setup needed a provisioning that now failed for a reason it did
not care about.

## The fix

- Default to an **always** double: one outcome, any number of calls. Tests that
  need "it worked" get "it worked" at any sequence length.
- Keep the scripted variant for the one test whose subject *is* per-call
  behaviour (which call failed, and does the refusal name it).
- Address positions by role, not index: `calls.last()` for the read-back, not
  `calls[3]`. How many calls precede it is a different test's claim.
- Let exactly one test assert the count. That test is where a sequence change
  *should* red.

## Why it matters

Entanglement reads as flakiness and burns the hour after a legitimate refactor.
It also corrupts a mutation battery's signal: extra reds are supposed to mean the
tests are entangled about *rules*, and fixture coupling makes that verdict
unreadable.
