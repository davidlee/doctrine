A behaviour-preservation gate ("the existing suite is the proof — it must stay
green **unedited**") is usually discharged with `git diff --stat` over the file
holding the risk-set tests. That is weak evidence, and in two distinct ways.

## 1. The diff must be taken against the phase base, not the working tree

`git diff` compares the working tree to the index. Once the earlier movement's
commits have landed — the normal case when a second worker picks up a phase — a
bare `git diff <path>` returns **empty**, and empty reads exactly like "nothing
was edited". It is not evidence; it is the absence of a question.

    git diff --stat <first-phase-commit>^..HEAD -- <path>

## 2. A line count sees deletions, not edits

Even against the right base, `N insertions, 2 deletions` invites the inference
"both deletions are production, therefore no test was touched". The inference
does not hold: an **insertion inside an existing test body** is an edit — a
loosened assertion, a widened tolerance, an extra early return — and it lands in
the same insertion count as a brand-new test appended below.

Pin the named items instead. Extract each risk-set item's full body from both
revisions by brace balance and compare the strings:

    git show <base>:<path>   # slice from `fn <name>(` to its balanced close
    git show HEAD:<path>     # same
    # assert identical; report the byte length as the receipt

That produces a claim of the right shape — *this oracle and these two callers
are byte-identical across the phase* — rather than a claim about the file that
happens to contain them. It also survives the file being reformatted around
them.

## Where the boundary comes from

Which items form the risk set is a **design decision, not a judgement call at
gate time**. Phase-plan it (SL-249 PHASE-04's `D1` fixed it at one file's
`mod tests`), so the gate is a lookup rather than an argument.
