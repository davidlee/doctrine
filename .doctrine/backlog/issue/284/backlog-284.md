# ISS-284: design apply never rechecks the run revision before persisting

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine design apply` reads the run snapshot **once**
(`src/commands/design.rs:1224`) and checks the payload's declared revision against
that in-memory copy at admission. Between admission and persistence it never
re-reads the snapshot or conditionally replaces it. The one pre-write recheck
covers the **authored watermark** — `design.md`'s bytes — not the run's revision
(`src/commands/design.rs:1305-1313`), and `write_snapshot` is unconditional.

So a second writer that advances the run after this invocation admitted can have
its snapshot overwritten by a candidate derived from the older revision. The
compare-and-swap is real at *admission* and absent at *persistence*.

## Why this is filed rather than fixed

**The window is currently negligible.** Nothing slow sits between admit and write:
the intervening work is pure computation plus a journal write. Two agents would
have to interleave inside microseconds, and the design run is a single-coordinator
surface by construction (delegation is proposal-only — the coordinator is the sole
writer, per `design-prompts/delegation.md`). No live defect is known.

It is filed because the property the code *has* is narrower than the property the
shipped guidance *states*. `install/hymns/stage/design.md` says "Every mutation
compare-and-swaps against the run's revision", which a reader will take to cover
the write, not only the admission.

## Why it surfaced now

An external adversarial review of SL-233's PHASE-16 design found it. The proposed
obligation runbook runner would have executed a **verifier subprocess** between
admission and persistence, turning a microsecond window into an
arbitrary-duration one.

PHASE-16 dissolves that at the design level rather than fixing this: verifiers run
shell-side and their *results* enter as payload, so nothing slow enters the span
(see `.doctrine/slice/233/sketches/runbook-runner.md` §4 and DEC-101). **That
architecture depends on this issue staying dormant** — a future change that puts
anything slow inside admit→persist re-arms it.

## What a fix would look like

Either a genuine compare-and-swap immediately before `write_snapshot` (re-read the
snapshot, refuse if its revision moved since admission), or a stated
serialisation/lock boundary covering the whole admit→persist span. The first is
cheaper and matches the CAS vocabulary the run already uses; the refusal would
join the existing `Refusal` family so callers get the same "re-read the run, do
not retry harder" instruction the stale-revision refusal already gives.

## Notes

- **Do not "fix" this inside PHASE-16.** That phase's scope fence is mechanism for
  the runbook runner; widening it to touch the persistence path would be exactly
  the improvisation the fence exists to prevent.
- A regression test wants a deliberate interleave, which needs a seam to pause at.
  `PreWriteHook` / `FaultHook` already exist on the apply path and are the obvious
  place to drive it from.
