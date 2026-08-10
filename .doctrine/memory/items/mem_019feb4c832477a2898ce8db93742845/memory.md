## The rule

If a phase builds something that **signals, deletes, or unmounts**, its *first*
commit carries the refusal that bounds it. Not its last task.

## What it cost

`SL-248` `PHASE-10` scheduled the guard as the final task, so as not to change
code under test mid-battery. Defensible in itself — and it put the floor behind
the twenty-one rows that exercise the very thing it makes safe. The battery
passed; the teardown happened *during* the task carrying the guard, before it
was committed.

Twice in that slice the operator was dropped back to their shell with no error,
during runs of a suite whose sweep kills by session. The sweep refused only its
**own** session, and the harness sits in a different one — `bwrap` at pid 1 in
session 0, the agent at pid 2 in session 0. Nothing in any liveness guard
protected either: this is the failure mode where the instrument kills the
observer.

## The practice

Before spawning a worker back into a suite that can kill processes it did not
start, **run it yourself once**. It is the discriminating experiment, it is
cheap, and losing the orchestrator's firing is much better than losing the
worker's context on top of it.

The scheduling instinct that caused this — "don't change code under test
mid-battery" — is sound in general and wrong here, because the thing being
deferred is what makes the battery survivable. When those two conflict, the
floor wins.

Sibling: [[mem.pattern.dispatch.brief-a-diagnosis-as-a-hypothesis]] — this
belongs in the *planner's* brief, not discovered by the worker.
