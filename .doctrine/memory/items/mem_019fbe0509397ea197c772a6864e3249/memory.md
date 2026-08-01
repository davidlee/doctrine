## The trap

Doctrine has a growing family of tests that assert over **authored data** rather
than over code — golden corpora, fixture trees, and (SL-233 PHASE-09) an
evaluation kit of TOML the test scores from. TDD on one of these produces a
tempting first RED: write the test, run it, watch it fail with

```
kit artefact …/rubric.toml is not readable: No such file or directory
```

That is a real RED and it satisfies the letter of a negative-control criterion.
It is nearly worthless as evidence. **All it proves is that the test opens the
file.** Every assertion downstream of the read is untested — the loop that never
executed, the comparison that would have passed vacuously over an empty set, the
`is_subset` on two empty collections. Data-driven assertions fail *soft*: a
`BTreeSet` that ends up empty because a key was misspelled makes
`is_disjoint`, `is_subset`, and `all()` all return `true`.

## The move

After the kit is green, **mutate the data to violate each load-bearing property,
one at a time, and confirm the intended assertion is the one that fires** — by
message, not merely by exit code. Then revert the mutation and re-run green.

PHASE-09's two, for shape:

| mutation | assertion that fired |
|---|---|
| added a delivery evidence key to the classification signal's `admissible_evidence` | the set-disjointness assertion — the criterion's actual property |
| lifted the known-bad transcript to the known-good one's top band in one class | the strict-ordering assertion, naming the class and both scores |

Cheap — two edits and two runs — and it is the difference between "the file is
there" and "the property holds and would be caught if it stopped holding".

## Reverting the mutation

Copy the file aside first, restore by copy, and `diff` the restore. Do not rely
on `sed -i` to undo its own edit: the reverse pattern often does not match what
the forward pattern produced, and the tree is left subtly wrong while still
green. If the data is untracked, `git checkout` is not available as a safety net.

Related: [[mem.pattern.harness.grep-negative-needs-positive-control]] is the same
error in a different instrument — a negative result that proves nothing about the
thing you meant to measure.

## Where this sits among its neighbours

[[mem.pattern.tdd.wire-before-guard]] is the same principle one level up: a red
that is a *payload refusal* (the wire does not exist) is weak; a red that is a
*wrong admission* (the mechanism runs and the guard is missing) is strong. This
memory is that distinction for a test whose subject is **data rather than code**,
where the weak red has a characteristic disguise — a missing file — and the
assertions fail soft rather than not compiling.

[[mem.pattern.harness.grep-negative-needs-positive-control]] and
`mem_019fbd70c9247f02a4c25c76fa3ca57e` (*an absence assertion is evidence only if
its subject is reachable from where the prober stands*) are the same error in
other instruments.
