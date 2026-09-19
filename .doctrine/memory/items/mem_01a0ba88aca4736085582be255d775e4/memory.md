When proving a test actually discriminates by mutating the code or fixture and
expecting red, **assert that the mutation was applied before you read the test
result.**

A mutation that silently fails to apply produces the *same observation* as a test
that does not discriminate: green. The two are indistinguishable from the exit
code alone, and the failure mode is the dangerous direction — you conclude "the
test is weak" or, worse, "the test passed, so the code is right", when in fact
nothing was ever changed.

In SL-246 PHASE-03 two mutations silently failed to apply. In PHASE-04 a
five-row mutation battery caught both a non-discriminating fixture *and* a
genuinely wrong golden — but only because application was confirmed first.

## How

- After a `sed`/patch-style edit, re-read the mutated region and confirm the new
  bytes are there. Do not trust a zero exit status from the editing tool.
- Prefer an edit whose failure is loud (a compile error) over one whose failure is
  quiet (a string that did not match anything).
- Revert explicitly and confirm the revert, so the next row starts from a known
  state.

## Where this bites hardest

Verification rows that exist to prove a test has teeth — the `VA` criteria that
demand a **positive control** ("the same grep must find three at the branch
point"). A positive control whose mutation never applied is not a control at all,
and it is the one place the whole verification tier rests on the agent's own
discipline rather than on a tool.

Note `python3` is absent in the jail, so region splices tend to be done with
`sed`, which fails silently on a non-matching pattern — exactly the quiet failure
this pattern guards.

See [[mem.pattern.doctrine.tdd-loop]].


## Relation to the sibling pattern

[[mem.pattern.tests.mutation-needs-a-discriminating-fixture]] says a mutation
that reds nothing is *usually* a non-discriminating fixture. This pattern names
the other branch of that "usually": it may instead be a mutation that **never
applied**. Confirm application first, and only then is the fixture the suspect.
