A test that plants a bypass around a guard and then watches the outcome proves
nothing unless **the guarded path and the bypassed path would give different
answers**. If both return the same value, the observation is hollow: it is green
whether the guard holds or has been removed entirely.

## The case that named it

SL-241 PHASE-05 guard probe (d) — I4a, "no script the capsule can write is ever
the process whose exit status is the verdict". The runner is ro-bound outside the
capsule's writable root. The probe plants a capsule-authored `verify.sh` that
exits 0 and asserts the verdict is the real runner's.

But the fixture's honest `verify:` command **passes**. Real runner → 0. Planted
runner → 0. The leg was green whether or not the ro-bind held.

The fix is not a better assertion — it is a **different scenario**: the capsule
also genuinely breaks the suite the verify command runs. Now the two runners
disagree (`verify/suite-failed` vs 0), and observing the refusal is observing the
guard.

## Why the usual controls do not catch it

The mutant that removes the broken suite (`m38`) reds the verdict clause — and
**both of the leg's most convincing controls stay green**:

- the static audit still refuses the mutated capsule
- the planted runner still demonstrably exits 0

Two green controls bracketing an observation that never happened. Plant-landed
and audit-refuses are checks on the *setup*, not on the *discrimination*, and
neither can see that the outcome was never contingent on the guard.

## How to apply

Before trusting a guard test, ask: **what would this have printed if the guard
were deleted?** If the answer is "the same thing", the scenario is wrong, not the
assertion. Fix it by making the guarded and unguarded outcomes differ, then assert
which one you got.

Related: [[mem.pattern.harness.grep-negative-needs-positive-control]] (a negative
result needs a positive control), [[mem.pattern.tests.baseline-needs-its-own-positive-control]]
(the same hollowness one level up, in a comparison's baseline).
