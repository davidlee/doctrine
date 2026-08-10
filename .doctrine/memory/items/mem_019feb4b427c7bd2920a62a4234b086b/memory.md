[[mem.pattern.testing.timing-tests-need-a-load-tally]] establishes that a
timing-dependent suite is proved by a tally under load, never one green run.
This memory carries what `SL-248` learned *past* that point, in three rules —
each learned by a worker's **honest** tally being contradicted afterwards.

## A tally is evidence only if its load matches the hazard's channel

This **corrects** that memory's "press it with spinners" step. `PHASE-10` `T2`
tallied 5/5 alone and 8/8 in-suite under **32 CPU spinners**, then failed 1 in 3
on the orchestrator's gate. The spinners could not have found it: the hazard is a
**descriptor** race, and a spinner saturates CPU while opening no file
descriptors. `cargo`'s own build opens thousands, which is why the gate
reproduced what synthetic load could not.

So tally **through the real gate**, not a hand-rolled contention harness — and
when you brief a tally, name the channel the hazard runs on (fds, pids, mounts,
the clock) and require load on *that*. Measuring the wrong axis cost three
firings.

## Where correctness depends on process-wide state, isolate; do not tally

The fd table, cwd, environment and signal dispositions are shared by every
thread in a test binary, so the interfering agent is *another test in the same
process*, appearing only when the scheduler interleaves the two windows.
`PHASE-10` `T4` isolated the assertions needing a fixture's decoys **present**,
tallied a true 5/5 through the gate, and was still red on the orchestrator's
first run: the mirror leg, needing those decoys **absent**, was still forking in
the shared process and inherited a concurrent fixture's set. Five green runs
were not weak evidence badly gathered — they were the wrong *kind* of evidence.

Require the **structural** argument: name the interfering agent and show it can
no longer reach the fork — the test-side counterpart of
[[mem.pattern.tests.process-wide-state-needs-the-production-guard]]. The tally corroborates a fix already argued; it is
never the argument. Fix the class, not the instance — sweep every site sharing
the hazard and make any deliberate exclusion visible.

## For a race, the instrument is a two-arm causality experiment

The `T4` residual was never reproduced by running it more: **27** unaggravated
runs, 21 filtered and 6 full-suite, were all green. What convicted was
aggravating the hypothesised aggressor and toggling one variable — arm A
un-windowed a single suspect call site and held its descriptor set open for 20 s
beside the victim, **red on the first run** with the reported panic's exact
shape; arm B, identical sleep with the window restored, **green**. That is proof
of mechanism.

Brief it that way: *name your suspected aggressor, make it worse, and show the
failure appears and disappears with it* — and require both arms be recorded
before the fix reverts them.

This qualifies [[mem.pattern.audit.rerun-gate-for-concurrency-invariants]]:
running the gate repeatedly is how you *detect* an intermittent invariant
reporting itself, but repetition can only ever fail to observe. Detection is
repetition; conviction is causality.

Briefing side: [[mem.pattern.dispatch.brief-a-diagnosis-as-a-hypothesis]].
