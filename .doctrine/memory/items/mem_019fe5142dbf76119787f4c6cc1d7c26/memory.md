## The trap

A suite that asserts on *when* — live processes, `/proc` entries, reaps, spawn
windows, wall clocks — has a hidden input: machine load. A single green run of
such a suite says only that the race was won once. Reporting that run as
"gate exit 0" is true and worthless.

`SL-248` `PHASE-09` shipped exactly this. The phase's own close-time evidence
was one green `doctrine check gate`; the orchestrator then measured **2 of 3**
gate runs red against **3 of 3** green on bare `cargo test`. The discriminator
is that `doctrine check gate` *builds immediately before it tests*, so the
suite runs on a loaded machine and the bare `cargo test` does not.

## The tell

Green bare, red under the gate — or red intermittently at a rate that tracks
how busy the box is — is a **race**, not a flake. Do not re-run until green.

## The practice

1. **Tally, don't sample.** Run the gate at least 3× before claiming a phase
   green; more if any assertion mentions a pid, a window, a timeout or a sleep.
2. **Press it.** Spin N background loops (`for i in $(seq 1 24); do (while :;
   do :; done) & done`) and run the suite under contention. This surfaces in
   minutes what a quiet box hides for weeks. Kill them afterwards.
3. **Control the repair.** Restore the old assertion beside the new one and run
   the pair under load: the fix is credible when the old form reds and the new
   form is green *in the same process*. A repair never seen to matter is not
   known to work — the same rule `SL-241` applies to guards.

## The repair shape

The unsound pattern is reading a fact about a process **after** the code that
owned that process has finished with it. In `SL-248` the test resolved a pid
against `/proc` after the harness had already `wait`ed on the subject; the
entry was gone, and the assertion passed only while the reap lagged it.

Fix it by **recording the observation at a point the code already certifies is
live**, and asserting on the recording afterwards. Prefer a seam whose result
is load-bearing for a verdict you already assert: in `SL-248` the `Arm.live`
closure decides `subject_live_when_observer_ran`, and `classify_concurrent`
makes `ArmResult::Held` unreachable unless it returned true — so an existing
`assert_eq!(result, Held)` becomes the proof that the recorded read happened in
the live window. Liveness stops being an argument in a comment and becomes an
assertion.

Deleting the late read is the cheap alternative and is usually wrong: it drops
a claim (this pid is *real*, not merely *different from the decoy*) that the
assertion existed to make.

## Correction — the load model must match the hazard's channel

Point 2 above ("spin N background CPU loops") is **necessary and not
sufficient, and taken alone it is misleading.** `SL-248` `PHASE-10` `T2` was
tallied 5/5 alone and 8/8 in-suite under **32 spinners on 32 cores**, reported
green in good faith, and then failed **1 in 3** on the orchestrator's
`doctrine check gate`.

The spinners were structurally incapable of finding it. The hazard was a
**descriptor** race — a decoy fd set swept by another thread's capsule run
between a guard window's release and the arm's fork — and a busy-loop
saturates CPU while opening **no file descriptors**. `cargo`'s own build opens
thousands, which is precisely why the gate reproduces what the synthetic load
cannot.

So, before tallying, **name the channel the hazard runs on** — descriptors,
pids, mounts, signals, the clock — and load *that*. A CPU spinner is the right
instrument for a scheduler race and the wrong one for every other kind.

**And prefer the real gate to hand-rolled contention.** `doctrine check gate`
builds before it tests, so it loads CPU, memory, fds and the page cache at
once, in proportions no hand-written loop reproduces. If you can only afford
one form of evidence, make it three gate runs.

In this slice the failure mode has now cost three separate firings, each time
arriving as a worker's *honest* tally that measured the wrong axis. The
practice that survives: the tally is the claim, the channel is what makes it
evidence.
