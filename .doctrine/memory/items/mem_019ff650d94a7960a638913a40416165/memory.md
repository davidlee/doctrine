## The failure

A research round asked a code-map thread: *which parts of `src/worktree/` are
reusable core versus hook-serving shell?* It answered accurately for the tree as
it stands — "marker machinery survives, `fork --worker` writes it
(`fork.rs:191`)". A second thread, scoped to the target topology, listed the
marker identity system among what gets deleted.

Both went into the synthesis. Neither was collided with the other, and the
"survives" reading — the one describing the present — won by being more concrete.
It then propagated into the slice's scope document, contradicting `DEC-202`,
which had listed the disk marker as the **first** of four mechanisms to delete.

Three artefacts drifted off a correct decision. It surfaced only because the
owner read one sentence and disbelieved it.

## Why this recurs

The two question shapes look compatible and are not:

- *"what calls this?"* → a fact about the **present**. Verifiable, cheap,
  concrete, and it will always be phrased with more confidence.
- *"what should exist after the change?"* → a claim about the **target**. Held in
  a decision record, not in the code.

A researcher answering the first honestly will contradict the second whenever
the change removes a caller. That is not thread error — it is the thread doing
its job. The synthesis step is the only place the collision can happen, and it
is easy to skip because both inputs read as correct.

## The rule

When assembling a multi-thread research artefact:

1. List the mechanisms the governing decision says are being **removed**.
2. For each, find what every thread said about it. A "survives" from a
   call-graph thread against a "deleted" from a decision is a **conflict to
   resolve in the artefact**, never two bullets to file side by side.
3. Resolve it on the mechanism, not the citation count. Ask whether the thing
   can still *function* in the target topology — not merely whether something
   still calls it today.
4. Re-read the governing records before writing the synthesis. Derived artefacts
   drift off decisions; decisions rarely drift off themselves.

## The tell

A research finding that quietly **narrows** a decision's scope. Widening gets
scrutiny; narrowing reads as reassuring precision and passes.
