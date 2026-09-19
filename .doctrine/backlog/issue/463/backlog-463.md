# ISS-463: e2e_observation refusal test is EPIPE-flaky under full-suite parallelism

`tests/e2e_observation.rs::record_input_cannot_be_combined_with_the_per_field_flags`
fails nondeterministically under full-suite parallelism and passes 3/3 in
isolation.

## Cause

The test writes to the stdin of a command that is **supposed** to refuse the
flag combination and exit. When the child exits before the parent finishes
writing, the write races into `EPIPE` and the harness reports `BrokenPipe`
rather than the refusal the test is actually asserting. Under load the child
wins that race more often, which is why it only shows up in the full suite.

## Why this is worth fixing beyond the one red

A nondeterministic red in the shared gate attacks the "**any unexpected red
suite is the C1 alarm**" rule directly. That rule is what makes a gate run
informative during phased execution: a red that is not your own new failing test
is signal, and you halt on it. A known-flaky row trains agents to re-run reds
instead of treating them as signal — which is the exact habit the alarm exists to
prevent. It already cost one investigation cycle in SL-246 PHASE-02, where a
worker's gate went red on it while nothing in the phase touched observation at
all.

## The fix

Have the test tolerate `EPIPE` on the stdin write — the **refusal is the
assertion**, and a broken pipe is evidence the child refused promptly, not
evidence of failure. Do not paper over it with a retry: a retry keeps the
nondeterminism and keeps the training signal backwards.

Surfaced during SL-246 PHASE-02 (capsule-driver); reported by a capsule
orchestrator, reproduced as passing in isolation.

## The instrumentation destabilises the suite that tests it

Reported by the SL-246 PHASE-05/06 orchestrator: **recording observations appears
to raise this flake's rate.** `doctrine observation record` writes into
`.doctrine/observations/records/`, and the suite exercising the observation
surface is the one carrying the EPIPE race — so the RFC-011 friction-capture
instrumentation perturbs the tests covering the feature being instrumented.

This strengthens the case for fixing the race rather than tolerating it: the flake
is not a fixed background rate, it rises with exactly the activity the project has
asked every agent to perform continuously.
