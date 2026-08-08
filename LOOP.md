# LOOP.md — dumb-loop slice driver

A self-paced `/loop` drives one slice to completion. Each firing may be a **cold
context**: it knows nothing except this file, the disk, and the CLI. Nothing
load-bearing is carried in an agent's head between firings, and nothing may be.

**Subject slice: named by the `/loop` prompt that fires this file.** That prompt
is the only place the slice lives — it is passed verbatim to each firing, so a
cold context learns `<N>` from the instruction that woke it, never from this
file. Substitute `<N>` = that slice's number, `<PP>` = the current phase. If the
prompt did not name a slice, stop and ask; do not guess from `doctrine status`,
which lists every active slice and adjudicates between none of them.

> **This file is read every firing.** Its own length is a recurring cost. Keep
> it under ~200 lines; push anything phase-specific into the phase sheet.

## The contract

1. **Disk is truth, transcript is not.** Verify from `doctrine`, `git`, and the
   phase sheet. Never from a sub-agent's report, a memory of the last firing, or
   a summary. A worker that says "done" and a tree that says otherwise: the tree
   wins, every time.
2. **Any firing may be the last.** Kill the session at any point and the next
   firing must resume with no loss.
3. **Three writers, and they do not overlap.** See § Writer map.
4. **One sub-agent at a time.** Sub-agents run **in-tree, no worktree
   isolation** — two at once would collide in one index and one target dir.
5. **`git status --porcelain` first**; `git add <paths>` then
   `git commit <paths> -F -`. Never a pathless commit, never `git add -A`,
   never stash.

## Where this runs

The **primary worktree**, on branch **`edge`**, with **no worktree isolation** —
orchestrator and sub-agent share one tree, one index, one `target/`. Nothing is
merged back at the end because nothing was forked; commits land on `edge` as
they are made.

Two consequences, and they are the reason this section exists:

- **Never switch the branch.** No `git checkout <ref>`, no worktree fork, no
  stash. Read another ref with `git show <ref>:<path>`; restore files with
  `git restore --source=<ref> -- <explicit paths>`.
- **Minting is normal.** A corpus-scanning id allocator (`DEC-`/`ISS-`/`RV-`/
  `REQ-`) sees the live corpus here, so mint entities as findings arise. (An
  earlier revision of this file ran the loop from a clone and banned minting to
  avoid colliding with the parent's allocations. That constraint is gone with
  the clone; do not reintroduce it.)

## Cadence and the re-entrancy guard

**Self-paced, notification-driven.** Sub-agents are spawned in the background, so
their completion wakes the orchestrator directly. The clock is a *fallback* only:
after each spawn, `ScheduleWakeup` ~20 min to catch a sub-agent that hung or died
without notifying. There is no fixed polling interval — a no-op firing is a bug
to be designed out, not a cost to be budgeted.

**First, establish why you woke.**

- **Woken by a task notification** — that sub-agent is finished. Skip the mtime
  test; it proves nothing about a process that has already exited. Go to beat 3.
- **Woken by the fallback clock** — run the guard below.

The clock-wake guard is exactly two commands. Run them first, before reading
anything else:

```bash
./target/debug/doctrine slice status <N>
find .doctrine/state/slice/<N>/phases -name 'phase-*.md' -mmin -20 | head
```

| what you see | what you do |
|---|---|
| phase `in_progress`, sheet touched < 20 min ago | **exit.** Live sub-agent. One line, re-arm the fallback, stop. |
| phase `in_progress`, sheet cold > 40 min (two fallbacks, no tick) | it died. Re-spawn, resuming at the first unticked task. |
| phase `completed`, a next phase exists | beat 4 — spawn the planner. |
| phase `planned`, sheet > 100 lines (filled) | beat 5 — spawn the worker. |
| phase `planned`, sheet ~27 lines (bare template) | beat 4 — spawn the planner. |
| no phases left | § Stop conditions. |

`wc -l` on the sheet is the empty/filled test — the materialised template is 27
lines, a planned sheet is several hundred. Don't eyeball it.

The heartbeat is free because workers tick tasks as they go (§ Sub-agent
discipline). The sheet's mtime *is* the liveness signal — that is a second reason
for the tick-as-you-go rule, not a coincidence.

Three consecutive revives of the same phase → stop and report. Do not loop on a
sub-agent that cannot finish.

## The orchestrator's turn

Opus, and **thin** — it routes, it does not read source and it does not read
`design.md`. Beats in order, stopping at the first that ends the firing:

1. **Guard** — as above. Exit if a sub-agent is live.
2. **Orient** — `.doctrine/slice/<N>/handover.md`, then `git log --oneline -5`.
   Nothing else.
3. **Verify the last claim** — if a phase reports complete: `doctrine check gate`,
   confirm green, then flip it:
   `doctrine slice phase <N> <PP> --status completed`. **The flip is the
   orchestrator's, never the worker's**, and it happens after the gate, not after
   the report. Then read the boundary warning; if it names foreign commits,
   tighten: `doctrine slice record-delta <N> <PP> --start <first own>^ --end <own tip>`.
4. **Plan** — spawn the **planner** (§ Sub-agent briefs). It fills the runtime
   sheet; you do not.
5. **Spawn** — one **worker**, one phase. Flip to `in_progress` *before*
   spawning, so a clock wake's guard sees it.
6. **Close the firing** — rewrite `handover.md` (§ Handover), `ScheduleWakeup`
   ~20 min, one line to the user: phase, beat, what's next.

Beats 4 and 5 may fall in the same firing — planning and execution are separated
by living in **different sub-agent contexts**, which is the whole point of
delegating them. What must never share a context is planning and execution, not
planning and spawning.

## Sub-agent briefs

Both roles run in-tree on `edge`, **no worktree isolation**, and are told so.
Every brief carries, and carries nothing else:

- the slice and phase id, and the sheet path
  `.doctrine/state/slice/<N>/phases/phase-<PP>.md` — **the sheet is the brief**;
- "read `LOOP.md` § Sub-agent discipline and § Where this runs — both bind you";
- "in-tree on branch `edge`, no isolation; another agent shares this index, so
  `git status --porcelain` first and path-limit every commit";
- "end green: `doctrine check gate`. Do not flip your own phase status."

**Planner** — Opus, always. Runs `/phase-plan` for one phase: reads that phase's
`plan.toml` entry — `objective`, `entrance_criteria`, `exit_criteria`,
`verification`, `specs`, `targets` — and only the `design.md` sections that
entry's prose actually cites. **There is no Reading-list field in `plan.toml`;
the Reading list is something the planner writes.** Where the entry cites no
section, the planner chooses, and records in the sheet which sections it read
and why. It then writes tasks, carried constraints, STOP conditions, the
`VT`/`VA` mapping, risks and decisions into the sheet. Writes no source.

Two things every planner brief should carry, both learned the hard way:

- **Name any task that structurally cannot stage a red**, with the compensating
  positive control that must fail in its place. A test written after the code
  that makes it compile passes on first run and proves nothing.
- **Verify inventories against the tree**, never against a prior sheet or a
  memory — "the five touch sites for a new `Finding` category" were six, and the
  sixth silently dropped data.

Hands back ≤12 lines: task count, open decisions, which tasks cannot stage a red
and their controls, whether the phase wants one worker or two in sequence (with
the split point and the first's exit state), and the worker model it recommends.

**Worker** — Sonnet when the sheet is fully specified and the work is mechanical:
a known edit shape, no open decisions. Opus when the sheet carries an open
decision, a STOP condition likely to fire, or verification the worker must
design. When in doubt: Opus. A Sonnet worker that improvises past a design gap
costs more than the Opus that would not have.

## Sub-agent discipline

**Tick a task only after its knowledge is durable.** The order is: finish the
work → harvest it (notes shard, memory, observation) → *then* tick the box and
note the evidence in the sheet's Findings. A ticked box with un-harvested
knowledge is a lie the next firing believes. This is also the heartbeat: an
untouched sheet reads as a dead sub-agent.

**Harvest as you go, never at the end.** Context exhaustion is the expected
ending, not the exception.

- durable gotcha / pattern / footgun → `doctrine memory record`;
- friction, confusion, token waste → `doctrine observation record friction …`;
- execution record, divergences, measurements → the notes shard;
- a decision, issue, or finding needing a `DEC-`/`ISS-`/`RV-` id → mint it (§
  Where this runs) and cite the id in the sheet's Findings, so the orchestrator
  reads a reference rather than re-deriving the content.

**Stop, do not improvise.** A STOP condition in the sheet, a design gap, a
criterion that does not compile as written, a decision needing a human ruling:
write it into the sheet's Findings, leave the box unticked, and hand back. A
criterion that does not hold is a stop-and-report, **never** an
adjust-the-criterion. The loop halting on a real question is the cheapest outcome
available. `DEC-181` is what that looks like when it works.

**End green.** `doctrine check gate` — `check`/`gate` build before validating,
which is what gives the corpus check a fresh binary. Commit path-limited, with
the conventional scope `feat(SL-<N>): PHASE-<PP> …`.

## Notes, sharded

`notes.md` is tracked and committed; `handover.md` is **gitignored**
(`.gitignore:51`) — it does not survive `rm -rf` of state. Never put anything
load-bearing only there.

**The sheet names the shard; the brief must not.** The orchestrator got this
wrong twice — naming `notes_03-06.md` and `notes_07-08.md` in briefs whose
sheets said `notes_04-06.md` and `notes_08.md` — and both times the worker had
to adjudicate and leave a note. The orchestrator does not know the shard split,
because phases do not execute in id order (`SL-249` ran `01 02 08 03 …`) and the
ranges follow execution, not numbering. So: the planner writes the shard name
into the sheet, the worker follows the sheet, and the brief says "the shard the
sheet names" and nothing more.

- `notes_01-03.md`, `notes_04-06.md`, … — the execution record for those phases:
  what was done, what diverged, what was measured. A worker appends to its own
  shard and touches no other.
- `notes.md` stays small and holds only what outlives a phase:
  1. an index of the shards;
  2. **Owed to the reconciliation brief** — the ledger `/audit` reads first.
     This never moves into a shard;
  3. cross-phase invariants and decisions;
  4. Open — questions, blockers, deferrals;
  5. Learned — memories minted, pointers only.

Extra files in a slice directory are fine; `doctrine validate` scans entity kinds
and ignores them (`spike-credentials.sh` is the precedent).

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
| phase sheet — tasks, criteria mapping, risks | planner |
| phase **status** flips, `record-delta` | orchestrator only, after the gate |
| `notes_NN-MM.md` shard | the phase's worker |
| `notes.md`, `handover.md` | orchestrator only |
| authored `.doctrine/` entities (plan, design, backlog) | orchestrator only |
| new entity **ids** | whoever needs one — the corpus here is live |
| memories, observations | whoever learns it, at the moment it bites |

## Stop conditions

Call `ScheduleWakeup(stop: true)` and report, on any of:

- **Done.** Last phase `completed` and gate green → route to `/audit`. The audit
  is not loop work; it needs a human in the loop. Stop.
- **Blocked.** A sub-agent hit a STOP condition, or a ruling is owed. Stop and
  ask; do not spawn again.
- **No progress.** Three firings with no phase-status change and no new commit,
  or three revives of one phase.
- **Budget.** Session approaching ~250k: write `handover.md`, stop, and say a
  fresh session should resume from it. A loop that runs into a summarization is a
  loop that starts trusting its own recollection.

## Budget

Delegation is what keeps the orchestrator alive: planning a phase is a
session-sized job, and doing it in-context burns the loop down in three or four
phases. So the planner is a sub-agent, always, however tempting it looks to just
read the design yourself.

Rough orchestrator costs per firing: guard-and-exit ≤ 2k, spawn ≤ 10k,
verify-and-flip ≤ 15k. These are the *orchestrator's* only — a planner or worker
sub-agent spends its own context freely and hands back ten lines. If the
orchestrator is reading source or `design.md`, the split has failed and the loop
is now on a countdown.

Read entities with `doctrine <kind> show <ID>`, not raw files. Use this tree's
`./target/debug/doctrine`, never the PATH binary, or you will read a corpus
through a stale parser.
