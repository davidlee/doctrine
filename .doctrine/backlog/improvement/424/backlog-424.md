# IMP-424: Fold sl-248's stranded LOOP.md orchestration lessons into the loop template

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What happened

`LOOP.md` diverged in two directions from the `edge`/`sl-248` merge base and was
resolved to `edge`'s copy at landing (merge `3953b74c3`).

- **`edge`** — `SL-249` generalised the file: slice named by the firing prompt
  rather than baked in, primary-worktree posture, minting re-permitted, 294
  lines. This is the copy a live loop reads every firing.
- **`sl-248`** — deepened instead, to 557 lines, and was declared dead by
  `SL-248`'s handover once execution finished.

Taking `edge`'s copy was the right call for landing — it is the live file, and
grafting into it is a deliberate change, not merge cleanup. But three durable
orchestration lessons exist **only** on the discarded side.

## The three stranded lessons

Each is measured, not theorised, and none is `SL-248`-specific.

- **`f730eef0f` — the notification is the only proof of completion.** Silence is
  never proof. The liveness legs are *alive* detectors: each can say "still
  working", none can say "done". A worker between two tool calls is quiet in
  exactly the way a finished one is. Measured 2026-08-10: an improvised guard
  read a clean tree with no live processes at 07:39:05 and was believed; the
  worker wrote at 07:39:23, committed at 07:42:09, and notified at 07:46. The
  durable half is that the four legs *as written* would have held the line —
  the commit leg read four minutes old, which is alive — so the error was
  reading "five commits landed and nothing is running" as finished.
- **`dcdcff6ce` — the diagnostic feed shows uncommitted intermediate states.**
  Read it as liveness, not as a defect: a worker mid-refactor has helpers landed
  and call sites not yet moved, so `dead_code` and unresolved-name diagnostics
  are the expected shape of work in progress. Check whether the condition
  survives into a commit (`git show <tip>:<path>`) before advising on it; an
  advisory that turns out to be noise costs the worker a cycle and costs you the
  credibility of the next one.
- **`1e56205ff` — what convicts a race, and how to brief a diagnosis.** Where a
  test's correctness depends on process-wide state, the answer is isolation and
  a tally is not the instrument — 27 unaggravated runs were all green while the
  defect was real. What convicted was a two-arm causality experiment: aggravate
  the hypothesised aggressor and toggle one variable, red on arm A's first run,
  green on arm B. Also: the orchestrator's own verification run is what convicts,
  never the worker's self-tally; and a handed-down diagnosis should be labelled a
  hypothesis with the constraints a fix must satisfy, not a prescribed route.

`edge`'s copy also carries a **weaker two-leg liveness guard** where `sl-248`'s
carried four (sheet mtime, build artifacts, dirty-path mtime, last commit) with
the blind-spot table that motivates each. That regression-relative-to-`sl-248`
is the most operationally load-bearing of the gap.

## What to do

Not a straight revert — `edge`'s slice-parametric shape is the one to keep. Fold
the lessons in, sized against the file's own stated ~200-line budget. Two sinks
are legitimate and the split is the real question:

- the loop template itself, for anything a cold firing must act on (the liveness
  legs belong here); and
- the memory corpus, for the reusable diagnostic method (the two-arm causality
  experiment, briefing a diagnosis as a hypothesis) — cross-slice knowledge that
  does not need re-reading every firing.

Sequence after `IMP-423` (extract a slice-agnostic loop template from `LOOP.md`)
if that lands first — the template is where this content wants to live.
