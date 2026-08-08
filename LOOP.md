# LOOP.md — dumb-loop slice driver

A self-paced `/loop` drives one slice to completion. Each firing may be a **cold
context**: it knows nothing except this file, the disk, and the CLI. Nothing
load-bearing is carried in an agent's head between firings, and nothing may be.

Subject slice: `SL-248`. Substitute `<N>` = `248`, `<PP>` = the current phase.

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

A **clone** of the primary repo (`origin` → `/home/david/dev/doctrine`), on
branch **`sl-248`**, merged back by hand at the end. Rationale and the merge-time
conflict list: `.doctrine/slice/248/notes.md` § *Execution posture*.

**The clone mints no entities.** `DEC-`/`ISS-`/`RV-`/`REQ-` ids are allocated by
scanning a corpus frozen at fork time while the parent keeps minting — two trees
mint the same id, and renumbering breaks immutability. Capture decisions and
findings in the phase sheet and the notes shard; mint on the parent at merge.
**Exempt:** `doctrine memory record` and `doctrine observation record` — both
key- or UUID-named, so collision-free. Use them freely.

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

The clock-wake guard is three commands. Run them first, before reading anything
else:

```bash
./target/debug/doctrine slice status <N>
find .doctrine/state/slice/<N>/phases -name 'phase-*.md' -mmin -25 | head
find target/debug/.fingerprint target/debug/deps -maxdepth 0 -mmin -30
git status --porcelain | cut -c4- | tr '\n' '\0' | xargs -0 -r -I{} find {} -maxdepth 0 -mmin -30
git log -1 --format=%cr
```

**A worker is alive if *any* of the four says so.** They watch different
activities and each is blind where another sees:

| leg | catches | blind while |
|---|---|---|
| sheet mtime | ticking a task | mid-task, however long that takes |
| build artifacts | compiling, testing | reading or writing source |
| dirty-path mtime | writing source | building, or just after a commit |
| last commit | landing a task | mid-task |

That last blindness is not hypothetical: a worker that has *just committed*
leaves a clean tree, so the churn leg reads dead at the exact moment the worker
was most productive. Hence the commit leg. Only silence on all four is death.

Cost is a handful of stats. The churn leg rides `git status`, which respects
`.gitignore` — do **not** substitute a `find` over the worktree, which walks
`target/` and its 1,200-odd fingerprint entries. It needs `-I{}` so each path
lands *before* the expression; a bare `xargs … find -maxdepth 0 -mmin -30`
appends them after and `find` exits with "paths must precede expression",
which reads as a silent dead leg. This shipped broken and was caught by
*running* the guard, not by reading it — as the `pgrep -f` defect was.

Prefer artifact mtime to `pgrep`. A process check is an *instantaneous sample* —
a worker between builds shows no `cargo` at all and reads dead while plainly
working (measured, not supposed). Mtimes are cumulative: they answer "did this
happen recently", which is the question actually being asked. Comparing two
`git diff`s a minute apart answers the same question and costs a minute of wall
clock; mtime already has the answer. (If you do reach for `pgrep`, never `-f` —
this jail's own `bwrap` argv carries `--setenv PATH …/.cargo/bin…`, so `-f`
matches pid 1 and reports "building" unconditionally.)

| what you see | what you do |
|---|---|
| phase `in_progress`, sheet touched < 25 min ago | **exit.** Live sub-agent. One line, re-arm the fallback, stop. |
| phase `in_progress`, sheet cold, **but any other leg is fresh** | **exit.** Still alive, just slow. Same as above. |
| phase `in_progress`, sheet cold > 90 min **and all four legs silent** | it died. Re-spawn, resuming at the first unticked task. |
| phase `completed`, a next phase exists | beat 4 — spawn the planner. |
| phase `planned`, sheet > 100 lines (filled) | beat 5 — spawn the worker. |
| phase `planned`, sheet ~27 lines (bare template) | beat 4 — spawn the planner. |
| no phases left | § Stop conditions. |

`wc -l` on the sheet is the empty/filled test — the materialised template is 27
lines, a planned sheet is several hundred. Don't eyeball it.

**Sheet mtime alone is not liveness, and reaping on it is how you kill a healthy
worker.** Workers tick as they go (§ *Sub-agent discipline*), so the mtime is a
*cheap* signal — but it goes quiet exactly when the work is slowest and dearest
to lose. A ten-mutation battery is ten build-and-test cycles with nothing
tickable between them; so is a cold `cargo build` after a manifest edit. The
proxy inverts under load: the more expensive the phase, the deader it looks.
Hence the other three legs and the 90-minute floor — **any sign of life beats a
cold sheet.** When the legs disagree, believe the one saying alive.

Re-spawning is not free and not idempotent: a revived worker re-does everything
since the last tick, and a worker reaped mid-write can leave a half-edited file
the next one reads as finished. **Waiting is cheap, reaping is not** — when in
doubt, exit and re-arm. That asymmetry, not the specific numbers, is the rule.

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

Both roles run in-tree on `sl-248`, **no worktree isolation**, and are told so.
Every brief carries, and carries nothing else:

- the slice and phase id, and the sheet path
  `.doctrine/slice/<N>/phases/phase-<PP>.md` — **the sheet is the brief**;
- "read `LOOP.md` § Sub-agent discipline and § Where this runs — both bind you";
- "in-tree on branch `sl-248`, no isolation; another agent shares this index, so
  `git status --porcelain` first and path-limit every commit";
- "end green: `doctrine check gate`. Do not flip your own phase status."

**Planner** — Opus, always. Runs `/phase-plan` for one phase: reads that phase's
`plan.toml` entry, and of `design.md` **only what that phase's row in `plan.md`
§ *Design-section provenance* grants it**, then writes tasks, carried
constraints, STOP conditions, the `VT`/`VA` mapping, risks and decisions into
the sheet. Writes no source. Hands back ≤10 lines: task count, open decisions,
and the worker model it recommends.

`plan.toml` has **no** Reading-list field — the sheet's § *Reading list* is
something the planner **writes**, derived from the provenance row. Do not brief a
planner to go read one; it will find nothing and improvise a scope for itself,
which is the failure the provenance table exists to prevent.

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
- anything needing a new `DEC-`/`ISS-`/`RV-` id → **do not mint it** (§ Where
  this runs). Write it into the sheet's Findings for the orchestrator to mint at
  merge.

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

- `notes_01-03.md`, `notes_04-06.md`, … — the execution record for those phases:
  what was done, what diverged, what was measured. A worker appends to its own
  shard and touches no other.

  **The sheet names the shard; a brief must not.** Those ranges track the order
  phases *executed*, not their ids — phases do not run in id order, so a shard
  boundary drifts away from the numbering as soon as one is taken out of turn.
  An orchestrator deriving the filename from `PHASE-NN` will eventually name a
  shard the sheet disagrees with, and the worker then has two instructions and no
  way to rank them. The brief says "the shard the sheet names"; if the sheet
  names none, `ls .doctrine/slice/<N>/notes_*.md` and take the newest, or the
  worker opens the next one.
- `notes.md` stays small and holds only what outlives a phase:
  1. an index of the shards;
  2. **Owed to the reconciliation brief** — the ledger `/audit` reads first.
     This never moves into a shard;
  3. cross-phase invariants and decisions;
  4. Open — questions, blockers, deferrals;
  5. Learned — memories minted, pointers only;
  6. **Traps** — what already bit this slice. This is the one section that grows
     monotonically, so it is also the one that blows the handover's ~50-line
     budget if kept there. It lives here, tracked; the handover points at it.

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
## Traps             a pointer to `notes.md` § Traps, not a second copy
```

If it exceeds ~50 lines, something belongs in `notes.md` instead. The trap list
is the usual culprit and has already been moved there once — it only grows, and
a gitignored file is the wrong home for the one section nobody can afford to
rediscover.

## Writer map

| surface | writer |
|---|---|
| source, tests | worker |
| phase sheet — task ticks, Findings | worker |
| phase sheet — tasks, criteria mapping, risks | planner |
| phase **status** flips, `record-delta` | orchestrator only, after the gate |
| `notes_NN-MM.md` shard | the phase's worker |
| `notes.md` § *Owed* — **append only, at the end** | the phase's worker |
| `notes.md` every other section, `handover.md` | orchestrator only |
| authored `.doctrine/` entities (plan, design, backlog) | orchestrator only |
| new entity **ids** | nobody, in this clone — mint on the parent at merge |
| memories, observations | whoever learns it, at the moment it bites |

**Why the worker owns § *Owed*.** It was orchestrator-only, and § *Budget* named
that transcription the largest per-phase orchestrator cost — then suggested
delegating it to a harvest sub-agent. The `PHASE-06` worker simply did it,
correctly, without being asked; delegating to the agent that already holds the
finding is strictly cheaper than delegating to a third one that must re-read the
sheet to reconstruct it. So the rule follows the practice.

Two constraints keep it safe. **Append at the end, never edit an existing item** —
numbering is immutable, and the orchestrator may be writing another section in
the same window. **The orchestrator still verifies at close**: read the sheet's
Findings against § *Owed* and confirm every `F-` reached it. "The worker wrote
it" is a claim like any other, and disk is truth (§ *The contract*).

## Stop conditions

Call `ScheduleWakeup(stop: true)` and report, on any of:

- **Done.** Last phase `completed` and gate green → route to `/audit`. The audit
  is not loop work; it needs a human in the loop. Stop.
- **Blocked.** A sub-agent hit a STOP condition, or a ruling is owed. Stop and
  ask; do not spawn again.
- **No progress.** Three firings with no phase-status change and no new commit,
  or three revives of one phase.
**Budget is NOT a stop condition.** It is a hand-over condition, and it is the
one beat where getting the distinction wrong converts an autonomous loop back
into a babysat one. It has its own section below.

## Budget — never a reason to stop

Running low on context is **not** in § Stop conditions and must never be treated
as if it were. Halting there is not caution; it is the exact failure this file
exists to prevent — the human set a loop up precisely so they would not be the
thing that resumes it.

**Compaction is survivable by construction.** Disk is truth and `handover.md` is
rewritten every firing specifically so a firing can begin knowing nothing. That
is the whole mechanism; there is nothing else to do. Autocompact is armed at
200k, so the fresh context arrives on its own and the loop continues through it.

Two things that are *not* available, recorded so no future firing re-derives
them: an orchestrator **cannot** self-terminate and be respawned fresh —
`ScheduleWakeup` and `CronCreate` both fire back into the *same* context — and a
sub-agent is a child, not a successor: its hand-back lands in the parent's
context, so spawning one does not reset anything. (A thin-supervisor shape, where
the loop-holder owns only the timer and rotates a fresh full orchestrator per
window, *is* mechanically possible — sub-agent nesting is verified working — but
it has not been adopted here and is not this file's contract.)

The only thing budget changes is **hygiene before a compaction**: finish the
firing you are in, never leave an unverified claim outstanding across the
boundary, and make sure `handover.md` reflects disk before you get close.

Delegation is what keeps the orchestrator alive: planning a phase is a
session-sized job, and doing it in-context burns the loop down in three or four
phases. So the planner is a sub-agent, always, however tempting it looks to just
read the design yourself.

Rough orchestrator costs per firing: guard-and-exit ≤ 2k, spawn ≤ 10k,
verify-and-flip ≤ 15k. These are the *orchestrator's* only — a planner or worker
sub-agent spends its own context freely and hands back ten lines. If the
orchestrator is reading source or `design.md`, the split has failed.

Measured over `PHASE-03`/`PHASE-04`: ~20–25k per phase closed, of which the
largest share is **carrying the sub-agent's findings into `notes.md`** — the
sheet is gitignored, so that transcription is real work and cannot be skipped. At
that rate a ten-phase slice costs an orchestrator ~250k, which fits a 1M window
several times over. If a future run needs it cheaper, delegate the transcription
to a short harvest sub-agent rather than dropping it; do **not** economise by
trusting a worker's summary over its sheet.

Read entities with `doctrine <kind> show <ID>`, not raw files. Use this tree's
`./target/debug/doctrine`, never the PATH binary, or you will read a corpus
through a stale parser.
