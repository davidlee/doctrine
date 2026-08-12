# An equality test proves equality, never derivation

A golden test asserts `actual == expected`. It says **nothing** about where
`expected` came from. If a human authored the expectation by reading the code,
the test proves the code agrees with one person's reading of it — which is
exactly the thing under doubt during a migration.

The concrete failure: a migration author drops a line from the output, encodes
the same omission into the golden, and the test goes green. Any "totality" claim
resting on that golden (every input line has exactly one successor, every output
line exactly one predecessor) is then satisfied **vacuously**. `SL-253`'s
`RV-354` `F-3` found exactly this, against a design that claimed the golden made
both clauses machine-checked.

## The taxonomy — ask this of every authored expectation

- **Observation-backed** — a ground truth exists elsewhere (a captured
  pre-change transcript, a recorded baseline). The expectation **must be derived
  from it** by an executable transform, not transcribed. This is the only class
  where "derivation" is a meaningful word.
- **Reality-checked** — the expectation is asserted against a reality that
  *already exists when it is written*. A characterisation test written **before**
  a refactor is in this class: a transcription error fails immediately rather
  than waiting for the code to catch up with it. Safe by construction — but say
  so out loud, because the safety is invisible and a reader applying the lesson
  above will suspect it.
- **Intent-backed** — new names, new spellings, new labels. There is **no ground
  truth to derive from**; the expectation *is* the intent, and its decision
  record is its only source. Irreducibly reviewer-checked. Trying to derive it
  would be deriving intent from its own restatement.

## Getting the derivation right when you do build one

Direction matters more than the mechanism. Derive expectation **from the
pre-change ground truth, then let the code go green against it** — never
transform and diff against the live post-change output, which invites tuning the
transform until it matches whatever the code emits. The first ordering also
gives the phase a red-to-green shape it can be executed against.

The transform can be a throwaway (a stdlib-only script, not a permanent test
harness) because it fires **once**: derivation is a one-time event at migration,
whereas drift is ongoing and the golden is the right instrument for that. But it
owes four self-checks, or it derives nothing: exhaustive classification (an
unmatched input line is a hard failure, never a pass-through), exact category
counts against the baseline, single consumption of every input and single
production of every output (the totality clauses as assertions the script makes
about its own run), and rejection of duplicate outputs.

## The residual to state rather than imply

Deriving the expectation does not prove the **transformation rules** are right —
a second implementation of the same rules only moves the trusted boundary. And
the self-checks bind the observation half only: an intent-backed input to the
transform (a wrong new key, a missing label) yields a golden the counts still
accept, because the counts are over lines and that error lives inside one.

Adjacent: [[mem.pattern.testing.black-box-cli-golden]] covers golden *mechanics*
(determinism, path carving) and is silent on this question.