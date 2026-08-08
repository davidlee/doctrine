# LOOP.md — dumb-loop slice driver

A `/loop` firing every ~10 minutes drives one slice to completion. Each firing is
a **cold context**: it knows nothing except this file, the disk, and the CLI.
Nothing is carried in an agent's head between firings, and nothing may be.

Subject slice: `SL-248`. Substitute `<N>` = `248`, `<PP>` = the current phase.

> **This file is read every firing.** Its own length is a recurring cost. Keep
> it under ~200 lines; push anything phase-specific into the phase sheet.

## The contract

1. **Disk is truth, transcript is not.** Verify from `doctrine`, `git`, and the
   phase sheet. Never from a worker's report, a memory of the last firing, or a
   summary. A worker that says "done" and a tree that says otherwise: the tree
   wins, every time.
2. **Any firing may be the last.** Kill the session at any point and the next
   firing must resume with no loss. That property is what keeps a session under
   250k — state lives on disk, not in context.
3. **Two writers, and they do not overlap.** See § Writer map.
4. **Other agents are live in this repo.** `git status --porcelain` first;
   `git add <paths>` then `git commit <paths> -F -`. Never a pathless commit,
   never `git add -A`, never stash, never checkout the primary tree off `edge`.

## Cadence and the re-entrancy guard

**Interval: ~10 min.** The wake fires on the clock whether or not a worker is
running — it does not block on one — and a phase outlasts several intervals. So
most firings land mid-phase and **must no-op**, cheaply: a no-op that reads
files before it checks the guard spends ten times what it needs to learn
nothing.

The no-op path is exactly two commands. Run them first, every firing, before
reading anything else:

```bash
./target/debug/doctrine slice status <N>
find .doctrine/state/slice/<N>/phases -name 'phase-*.md' -mmin -20 | head
```

The windows below are **intervals, not minutes** — at ~10 min, live is two
intervals of silence and dead is three. Change the interval and these move with
it; a worker that ticks less often than one interval would otherwise be declared
dead while it is working.

| what you see | what you do |
|---|---|
| phase `in_progress`, sheet touched < 2 intervals ago (~20 min) | **exit.** A worker is live. Say one line and stop. |
| phase `in_progress`, sheet cold > 3 intervals (~30 min) | the worker died. Re-spawn it, resuming at the first unticked task. |
| phase `completed`, a next phase exists | plan the next phase, spawn its worker. |
| phase `planned` with a filled sheet | spawn its worker. |
| phase `planned` with an empty sheet | `/phase-plan` it yourself, then stop. Planning is a whole firing. |
| no phases left | § Stop conditions. |

The heartbeat is free because workers tick tasks as they go (§ Worker
discipline). The sheet's mtime *is* the liveness signal — that is a second
reason for the tick-as-you-go rule, not a coincidence.

Three consecutive revives of the same phase → stop and report. Do not loop on a
worker that cannot finish.

## The orchestrator's turn

Opus. Beats, in order, stopping at the first that ends the firing:

1. **Guard** — the two commands above. Exit if a worker is live.
2. **Orient** — `.doctrine/slice/<N>/handover.md`, then
   `git log --oneline -5`. Nothing else. Do **not** read `design.md` wholesale;
   the phase's `plan.toml` entry and the design sections its Reading list names
   are the whole of what planning needs.
3. **Verify the last claim** — if the previous phase reports complete: run
   `doctrine check gate`, confirm green, then flip it:
   `doctrine slice phase <N> <PP> --status completed`. **The flip is the
   orchestrator's, never the worker's**, and it happens after the gate, not
   after the report. Tighten the boundary if the warning names foreign commits:
   `doctrine slice record-delta <N> <PP> --commit <sha>` (or `--start`/`--end`).
4. **Plan** — `/phase-plan` for the next phase. Fill the runtime sheet: tasks,
   carried constraints, STOP conditions, the `VT`/`VA` mapping, risks,
   decisions. This is a firing's whole work; end it here.
5. **Spawn** — one worker, one phase (§ Worker brief). Flip to `in_progress`
   *before* spawning, so the next firing's guard sees it.
6. **Close the firing** — rewrite `handover.md` (§ Handover). One line to the
   user: phase, beat, what's next.

Never do 4 and 5 in the same firing. Planning wants the design in context;
execution wants the code. Keeping them apart is what keeps both under budget.

## Worker brief

Sonnet when the sheet is fully specified and the work is mechanical — a known
edit shape, no open decisions. Opus when the sheet carries an open decision, a
STOP condition likely to fire, or verification the worker must design. When in
doubt: opus. A sonnet worker that improvises past a design gap costs more than
the opus that would not have.

Every worker prompt carries, and carries nothing else:

- the slice and phase id, and the sheet path
  `.doctrine/slice/<N>/phases/phase-<PP>.md` — **the sheet is the brief**;
- "read the sheet's Reading list before writing code; it cites file:line";
- the § Worker discipline rules below, by reference to this file;
- whether it works in-tree or in a fork, and if a fork, the base;
- "end green: `doctrine check gate`. Do not flip your own phase status."

## Worker discipline

**Tick a task only after its knowledge is durable.** The order is: finish the
work → harvest it (notes shard, memory, observation) → *then* tick the box and
note the evidence in the sheet's Findings. A ticked box with un-harvested
knowledge is a lie the next firing believes. This is also the heartbeat: an
untouched sheet reads as a dead worker.

**Harvest as you go, never at the end.** Context exhaustion is the expected
ending, not the exception.

- durable gotcha / pattern / footgun → `doctrine memory record` (or the
  `memory_record` MCP tool in a confined worker);
- friction, confusion, token waste → `doctrine observation record friction …`;
  in a confined worker with the doctrine MCP server, `observation_record`; in a
  fork with **neither**, do not capture — report it in the hand-back and let the
  orchestrator record it. A fork-local record dies with the tree.
- execution record, divergences, measurements → the notes shard.

**Stop, do not improvise.** A STOP condition in the sheet, a design gap, a
decision that needs a human ruling: write it into the sheet's Findings, leave
the box unticked, and hand back. The loop halting on a real question is the
cheapest outcome available. `DEC-181` is what that looks like when it works.

**End green.** `doctrine check gate` — `check`/`gate` build before validating,
which is what gives the corpus check a fresh binary. Commit path-limited, with
the conventional scope `feat(SL-<N>): PHASE-<PP> …`.

## Notes, sharded

`notes.md` is tracked and committed; `handover.md` is **gitignored**
(`.gitignore:51` `.doctrine/**/handover.md`) — it does not travel to a fork and
does not survive `rm -rf` of state. Never put anything load-bearing only there.

- `notes_01-03.md`, `notes_04-06.md`, … — the execution record for those
  phases: what was done, what diverged, what was measured. The worker appends to
  its own shard and touches no other.
- `notes.md` stays small and holds only what outlives a phase:
  1. an index of the shards;
  2. **Owed to the reconciliation brief** — the ledger `/audit` reads first.
     This never moves into a shard;
  3. cross-phase invariants and decisions (`DEC-…`);
  4. Open — questions, blockers, deferrals;
  5. Learned — memories minted, pointers only.

Extra files in a slice directory are fine; `doctrine validate` scans entity
kinds and ignores them (`spike-credentials.sh` is the precedent).

## Handover

Rewritten by the orchestrator at the end of every firing. Fixed sections, ~50
lines, no narrative history — history is the shard.

```
## Where we are      one line: slice, phase, status, beat
## Last firing       what changed, and the sha
## Next              the single next action, verbatim enough to execute
## Live              open decisions, blockers, awaiting-human
## Traps             what already bit this slice — do not rediscover
```

If it exceeds ~50 lines, something belongs in `notes.md` instead.

## Writer map

| surface | writer |
|---|---|
| source, tests | worker |
| phase sheet — task ticks, Findings | worker |
| phase sheet — tasks, criteria mapping, risks | orchestrator (`/phase-plan`) |
| phase **status** flips, `record-delta` | orchestrator only, after the gate |
| `notes_NN-MM.md` shard | the phase's worker |
| `notes.md`, `handover.md` | orchestrator only |
| authored `.doctrine/` entities (plan, design, backlog, REV) | orchestrator only |
| memories, observations | whoever learns it, at the moment it bites |

A **forked** worker writes source only: authored `.doctrine/` state does not
survive its tree. Its notes and observations come back in the hand-back and the
orchestrator lands them.

## Stop conditions

Call `ScheduleWakeup(stop: true)` — or stop the loop — and report, on any of:

- **Done.** Last phase `completed` and gate green → route to `/audit`. The audit
  is not loop work; it needs a human in the loop. Stop.
- **Blocked.** A worker hit a STOP condition, or a ruling is owed. Stop and ask;
  do not spawn again.
- **No progress.** Three firings with no phase-status change and no new commit,
  or three revives of one phase.
- **Budget.** Session approaching ~250k: write `handover.md`, stop, and say that
  a fresh session should resume from it. A loop that runs into a summarization
  is a loop that starts trusting its own recollection.

## Budget

Target per firing: no-op ≤ 2k, spawn ≤ 15k, plan ≤ 60k. The orchestrator's
context is spent on *routing*; the worker's is spent on code. If the
orchestrator is reading source files, the split has failed.

At ~10 min the binding cost is the **plan** firings, not the no-ops: roughly
three or four phases of planning fills a 250k session, and a day of no-ops
barely dents it. So the discipline that matters is beat 2 — orient from
`handover.md` and the phase's `plan.toml` entry, never from `design.md`
wholesale.

Read entities with `doctrine <kind> show <ID>`, not raw files. Use the coord
tree's `./target/debug/doctrine`, never the PATH binary, or you will read a
corpus through a stale parser.
