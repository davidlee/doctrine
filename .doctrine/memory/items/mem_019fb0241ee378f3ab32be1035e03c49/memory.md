A criterion of the form *"each test was observed RED before being made green"* is
satisfiable in two very different senses, and only one of them is evidence.

**Weak (a payload refusal).** Write the test, write nothing else, run it. It fails
with `unknown field 'concerns'` or `unresolved import` — the wire does not exist
yet. That red proves the test *runs*; it proves nothing about the behaviour the
test is named for, because no implementation was ever given the chance to get it
wrong.

**Strong (a wrong admission).** Land the whole mechanism — types, wire,
serialisation, the happy path — and deliberately **omit the guard**. Now run the
test. It fails with the system cheerfully doing the wrong thing: a stale proposal
accepted and its changes applied, a delegate's payload advancing the run's stage.
*Then* write the refusal. The same test goes red-for-the-right-reason and then
green, in the natural order.

## The practice

1. implement the mechanism and its happy path;
2. run the guarded tests — they must fail by **succeeding at the wrong thing**;
3. capture that output verbatim (it is the negative-control evidence);
4. write the guard;
5. re-run — green.

Costs one extra build. Buys evidence that the guard is what closed the hole, which
is the whole claim a negative control makes.

## Where it does not apply

A test asserting that a feature *exists* (the happy path itself) has no wrong
admission available — its only possible red is structural. Say so rather than
implying otherwise: an honest "this red is a compile failure, and no stronger red
exists for this test" is worth more than a reconstruction staged after the fact.

## Provenance

SL-233 PHASE-12 could only produce payload-refusal reds for its four refusal tests
and had to rebuild wrong-admission evidence afterwards, disclosed as a weakness.
PHASE-10 inverted the task order on purpose and got both wrong-admission reds in
the natural sequence. See the SL-233 `notes.md` § *Learned* entries for both.
