An `all`-shaped predicate is **vacuously true over an empty set**, so any test
asserting a computed verdict can pass on zero evidence and look identical to a
test that passed on all of it.

SL-248's admission verdict: `admission(rows)` is `rows.iter().all(|v| v == Proven)`.
`admission(&[])` returns `Admitted`. The production caller hands it the shipped
row tables, so the real path is safe — but the *test* asserting
`outcome == Admitted` cannot tell the two apart.

**The rule.** A test that asserts a verdict must also assert the verdict was
reached over the evidence it names:

    assert_eq!(verdict.outcome, Admission::Admitted);
    assert_eq!(verdict.rows.len(), tables().len());   // the load-bearing one

**How to know it is load-bearing rather than decorative.** Mutate the producer
to hand the predicate an empty set (here: `verify` passing `&[]` instead of
`&tables()`). Under that mutant the outcome assertion still passes, the
"which rows refused" assertion still passes (none did), and only the count
reds. A second tell: the vacuous run is *fast* — 0.32 s against the honest
run's 21.91 s — because a verdict over no rows does no work. A cheap green on
an expensive test is the symptom.

**Sibling of, not the same as**, the unread-surface rule (an absence probe
cannot tell "held nothing" from "read nothing"). That one is about a single
observation whose failure mode is an empty read; this one is about an aggregate
whose failure mode is an empty *input*.

Applies to any `all`/`every`/`none` over a collection the test does not itself
supply: conformance verdicts, lint walks, coverage sweeps, schema validators.