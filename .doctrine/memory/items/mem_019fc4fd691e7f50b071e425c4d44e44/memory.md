A byte-identical / equality comparison is only as strong as the **baseline** it
compares against. If the baseline is itself a produced run rather than a fixed
expectation, any hole in it is reproduced faithfully on the other side and the
comparison **reports agreement as evidence**.

So the baseline needs a positive control of its own — assertions that it is the
run you think it is — before anything is compared to it.

## The case that named it

SL-241 PHASE-05 guard probe (e): a run against a fixture carrying an in-repo
interpretation-surface declaration must be byte-identical trusted-side to a run
against the fixture without one. The baseline leg first asserted only:

- `pipeline_first_refusal` is empty (it refused nowhere), and
- the canonical-state assertion for a passing run holds.

Both of those hold for a genuine four-stage run **and** for a run in which a
stage never emitted at all. A ladder with a gap in it would then be reproduced
identically by the comparison leg and compare EQUAL — scoring the strongest claim
in the task against a baseline that had skipped a stage.

Fixed by asserting **every stage by name** before the comparison, which the rig's
own happy-path self-test already did for the same stated reason: *asserting only
the final status would score a run that skipped a stage entirely as green*.

Caught by reading, not by a red — the run was green before and after.

## How to apply

When you write `assert_eq(baseline_output, subject_output)`:

1. ask what the baseline would look like if the machinery producing it were
   partly broken; and
2. assert the baseline's own internal structure — every step present, by name —
   not just its summary verdict.

The summary verdict is the field most likely to be identical for the wrong
reason. This is the absence-shaped assertion hazard one level up: not "nothing
crossed", but "both sides agree".

Related: [[mem.pattern.evidence.witness-the-exact-set-not-the-emptiness]],
[[mem.pattern.tests.guard-needs-a-discriminating-difference]],
[[mem.pattern.harness.grep-negative-needs-positive-control]].
