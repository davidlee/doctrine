## The rule

Running low on context is **not** a stop condition and must never be treated as
one. Halting there is not caution — it is the exact failure a self-paced loop
exists to prevent. The human set the loop up precisely so that they would not be
the thing that resumes it.

## Why it is safe to keep going

**Compaction is survivable by construction.** Disk is truth, and `handover.md`
is rewritten every firing specifically so a firing can begin knowing nothing.
That is the whole mechanism; there is nothing else to do. The fresh context
arrives on its own and the loop continues through it.

Two things are *not* available, recorded so no future firing re-derives them:

- an orchestrator **cannot** self-terminate and be respawned fresh —
  `ScheduleWakeup` and `CronCreate` both fire back into the *same* context;
- a sub-agent is a **child, not a successor** — its hand-back lands in the
  parent's context, so spawning one resets nothing.

(A thin-supervisor shape, where the loop-holder owns only the timer and rotates
a fresh full orchestrator per window, *is* mechanically possible — sub-agent
nesting is verified working — but it has not been adopted.)

## What budget does change

Hygiene before a compaction: finish the firing you are in, never leave an
unverified claim outstanding across the boundary, and make sure `handover.md`
reflects disk before you get close.

## And what keeps the orchestrator cheap

Delegation. Planning a phase is a session-sized job, and doing it in-context
burns the loop down in three or four phases — so the planner is a sub-agent,
always, however tempting it is to read the design yourself. Measured over
`SL-248`: ~20–25k per phase closed, the largest share being carrying the
sub-agent's findings into `notes.md` (the sheet is gitignored, so that
transcription is real work and cannot be skipped). At that rate a ten-phase
slice costs an orchestrator ~250k. If a run needs it cheaper, delegate the
transcription to a short harvest sub-agent rather than dropping it; do **not**
economise by trusting a worker's summary over its sheet.

If the orchestrator is reading source or `design.md`, the split has failed.

Related: [[mem.pattern.dispatch.loop-death-triage-resume-vs-respawn]].
