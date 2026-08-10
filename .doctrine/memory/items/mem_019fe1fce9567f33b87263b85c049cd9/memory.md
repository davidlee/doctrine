## The trap

A step is written to be idempotent so that a crashed run can resume. The
obvious test is to *replay the submission* and assert the output is unchanged.

That test can be worse than useless, because the same resume machinery that
makes replay safe is usually a **state guard that skips the step entirely**.
In `commands::design::execute_mint` the guard is literal:

```rust
if intent.state() < IntentState::Applied {
    fault(CheckpointStep::EffectsApply);
    apply_record_effects(..)?;
    intent = intent.reaching(IntentState::Applied);
    journal_intent(root, slice, &intent)?;
}
```

Once step 5 has run, the intent is journalled at `Applied`, so a resubmission
of the same `submission_id` runs **none** of it. A byte-equality assertion
across that replay is not weakly vacuous — it is vacuous in the strongest
sense: it compares a file to itself with no writer between the two reads. It
passes identically on a tree where the step is a wildly non-idempotent
appender, and it passes on a tree where the feature does not exist at all.

The abandoned-write window makes this especially seductive, because it leaves
the run's *revision* unmoved. It looks like nothing happened, so replay looks
like a genuine second run. The revision is not the guard; the journalled
intent state is.

## The fix, two parts

1. **Call the step again directly.** Unit tests can reach a private `fn`, and
   "step 5 runs twice over the same record" is the property idempotence
   actually claims — and exactly what a crash *between* the effect and the
   journal write produces. That is the real resume case, and it is the one the
   guard cannot reproduce.
2. **Pin the guard state in the same test**, so the reasoning survives:

```rust
assert_eq!(
    journal.intents[0].state(),
    IntentState::Applied,
    "step 5 already ran and was journalled, so a resumed submission SKIPS it"
);
```

Without (2) a later reader "simplifies" the test back into the replay shape and
silently loses the coverage.

## Why not the fault hook

`injected_fault` is a hard `std::process::exit(70)` by design — a crash must not
unwind. So it is an e2e instrument only; an in-process test cannot use it to
stop mid-step and resume.

## Generalisation

Whenever idempotence is guarded by *recorded progress* rather than by the
writer's own shape, replay tests the guard, not the writer. Test the writer
where the guard is not.

## Related

- `mem.pattern.doctrine.an-absence-assertion-over-a-never-written-file-passes-vacuously`
- `mem.pattern.doctrine.missing-file-red-is-a-weak-negative-control`
