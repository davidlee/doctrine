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

**The notification is the only proof of completion. Silence is never proof.**
The four legs below are *alive* detectors: each can say "still working", none can
say "done". So a clock wake that finds every leg quiet has established exactly
one thing — that the worker is quiet *right now* — and a worker between two tool
calls is quiet in precisely the same way a finished one is. Absent a
notification, treat the worker as **live** whatever the tree looks like: do not
spawn, do not commit, do not run a verification gate. The only conclusion silence
ever licenses is *death* (§ the 90-minute floor), and death is resumed from the
transcript — it is never read as completion.

Measured, this loop, 2026-08-10: an ad-hoc guard (`ps`, `git status`, `git log`)
read a clean tree with no live processes at 07:39:05 and was believed. The worker
wrote at 07:39:23 and 07:39:36, committed `9d48adbc8` at 07:42:09, and notified at
07:46 — alive throughout. Two lessons, and the second is the durable one:
**run the four legs as written, not an improvised substitute** (the `ps` leg is
the instantaneous-sample defect this file already warns about two paragraphs
down, re-invented under time pressure); and the legs would in fact have held the
line here — the commit leg read four minutes old, which is *alive* — so the
error was reading "five commits landed and nothing is running" as finished.
Committed-recently means working, not done.

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
| phase `in_progress`, sheet cold > 90 min **and all four legs silent** | it died. **Resume it from its transcript, don't re-spawn** — see below. |
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

### When the worker dies, resume it — do not re-spawn it

`SendMessage` to the dead agent's id **resumes it from its saved transcript**,
with everything it had learned still in context. A fresh spawn starts from the
sheet and re-derives all of it. On a phase that has measured host behaviour the
design got wrong — the kind that fills a findings ledger — that context is the
expensive thing in the tree, not the code.

The revive message states **disk state, not instructions**: the tip sha and what
it contains, which boxes are ticked, exactly what is dirty and how large, and
"nobody has touched it since". Then re-state the contract in one paragraph,
because a resumed agent's oldest context is its briefing and that is what
degrades first. Two things worth saying every time:

- **assess the uncommitted work before building on it** — it was in flight when
  the process died and may not compile;
- **if finishing it costs more than redoing it, redo it** — sunk cost is not
  evidence, and a worker resumed mid-edit is prone to defending its own draft.

**Check `TaskList` before you plan on resuming.** The transcript is only
reachable while the agent registry is — and the registry does not survive the
sandbox being rebuilt (below). An empty `TaskList` where a worker should be
means resume is not available at any price: **re-spawn, and treat the tree as
your only witness.** Read the tip, the sheet's ticked boxes and the findings
shard before writing the brief; a worker that got most of the way through a task
usually left its evidence in the shard even though its context is gone.

### The failure mode above the worker: the harness process itself dies

It takes the worker *and* the loop's own `ScheduleWakeup` with it, so the loop
does not notice — **it stops firing entirely** and only restarts when a human
re-invokes it. Nothing in the guard can catch this, because the guard only runs
when the loop runs. Know the shape so you diagnose it in one read rather than
three: every leg cold *at once*, by hours, with a clean recent commit and a
plausible dirty file. That looks like a stalled worker; it is a stopped loop.

The consequence for briefs: **tell workers to commit each task as it goes green**
rather than at the phase's end. The uncommitted window is the whole exposure,
and it is the one variable the orchestrator controls from outside the worker.

**One level above that again: the sandbox is rebuilt.** Same symptom, worse
blast radius — the agent registry goes too, so the transcript is unreachable and
resume is off the table. The tell is one command: `ps -o pid,etime,comm -p 1`.
A pid 1 younger than the work you are looking at means the whole jail was torn
down and restarted, not merely a process killed inside it. Distinguishing the
two matters because they route differently — a dead harness resumes, a rebuilt
sandbox re-spawns.

**And if this phase's code can kill processes it did not start, suspect it.**
Twice in `SL-248` the operator was dropped back to their shell with no error,
during runs of a suite whose sweep kills by session; the sweep refused only its
own session, and the harness sits in a *different* one — `bwrap` at pid 1 in
session 0, the agent at pid 2 in session 0. Nothing in the guard protected
either. Before spawning a worker back into a suite like that, run it yourself
once: it is the discriminating experiment, it is cheap, and it is much better
to lose the orchestrator's firing than the worker's context on top of it.

The rule that generalises out of it, and it belongs in the **planner** brief:
**a destructive test instrument is floored before it is aimed, never after.**
The floor in that phase was scheduled as the last task so as not to change code
under test mid-battery — defensible in itself, and it put the guard behind the
twenty-one rows that exercise the very thing it makes safe. The battery passed;
the teardown happened during the task carrying the guard, before it was
committed. If a phase builds something that signals, deletes, or unmounts, its
first commit carries the refusal that bounds it.

**A tally is only evidence if its load model matches the hazard's channel, and
the gate is the load that counts.** The rule from the earlier flake — for a test
that reads a live process, a `/proc` entry, a window or a clock, the unit of
evidence is a tally under load, not an exit code — is necessary and it is not
sufficient. `PHASE-10` `T2` was tallied 5/5 alone and 8/8 in-suite under **32 CPU
spinners** and still failed 1 in 3 on the orchestrator's gate. The spinners could
not have found it: the hazard is a **descriptor** race, and a spinner saturates
CPU while opening no file descriptors. `cargo`'s own build opens thousands, which
is why the gate reproduces what the synthetic load cannot. So: tally **through
`doctrine check gate`**, not through `cargo test` under hand-rolled contention,
and when you brief a tally, name the channel the hazard runs on — fds, pids,
mounts, the clock — and require load on *that*. This has now cost three
firings, each time as a worker's honest tally that measured the wrong axis.

**Where a test's correctness depends on process-wide state, the answer is
isolation, and a tally is not the instrument.** The fd table, cwd, environment
and signal dispositions are shared by every thread in the test binary, so the
interfering agent is *another test in the same process* — and it appears only
when the scheduler interleaves the two windows. `PHASE-10` `T4` isolated the
assertions needing row 10's decoys **present**, tallied a true 5/5 through the
gate, and still red on the orchestrator's first run: the mirror case, the leg
needing those decoys **absent**, was still forking in the shared process and
inherited a concurrent fixture's set. Five green runs were not weak evidence
badly gathered; they were the wrong *kind* of evidence. A window this narrow is
closed by removing the interfering agent, never by failing to observe it.

So when a brief touches process-wide state: require the **structural**
argument — name the interfering agent, show it can no longer reach the fork —
and treat the tally as corroboration of a fix already argued, never as the
argument. Fix the class, not the instance: sweep every site sharing the hazard,
and make any deliberate exclusion visible, as `UNWALKED` does.

**The orchestrator's own verification run is what convicts, not the worker's
self-tally.** Both times a worker's honest tally has been contradicted, it was
contradicted by the first run taken outside the worker's own process and
sitting. Never skip it because the hand-back tally looks strong; a strong tally
is exactly when it is worth taking.

**For a race, the instrument is a two-arm causality experiment, not repetition.**
The `T4` residual was never reproduced by running it more: **27** unaggravated
runs — 21 filtered, 6 full-suite — were all green. What convicted was aggravating
the hypothesised aggressor and toggling one variable: arm A un-windowed a single
suspect call site and held its descriptor set open for 20 s beside the victim →
**red on the first run**, with the reported panic's exact shape; arm B, identical
sleep with the window restored → **green**. That is proof of mechanism. A tally
can only ever fail to observe. So brief it this way: *name your suspected
aggressor, make it worse, and show the failure appears and disappears with it* —
and require both arms be recorded before the fix reverts them.

**Brief the evidence and the constraints; label the diagnosis a hypothesis.**
The same firing's brief carried an orchestrator diagnosis that was wrong in its
specifics and prescribed a repair route that measurement rejected. Both were
caught only because the brief said *verify it, don't take it on trust* and the
worker did. Keep that phrasing on every diagnosis you hand down, and never
prescribe a route as settled — give the constraints it must satisfy (here: don't
weaken an assertion, don't reopen a settled decision, don't launder a flake into
a smaller flake) and let the worker choose against them. A worker that can
overturn the brief on evidence is the point of the split.

## The orchestrator's turn

Opus, and **thin** — it routes, it does not read source and it does not read
`design.md`. Beats in order, stopping at the first that ends the firing:

1. **Guard** — as above. Exit if a sub-agent is live.
2. **Orient** — `.doctrine/slice/<N>/handover.md`, then `git log --oneline -5`.
   Nothing else.
3. **Verify the last claim** — every firing, not only at phase close. Two legs,
   both cheap:
   - **The gate is the evidence, the report is a claim.** Run `doctrine check
     gate` yourself and read the tally by seeking forward to the
     `doctrine_control-` binary header — the line is neither first nor last.
     Confirm the delta over the previous task equals the tests the worker says
     it added.
   - **Findings reached § *Owed*.** One grep: every `F-` the sheet's § *Findings*
     gained this task must appear in `notes.md` § *Owed*. This is a one-line
     check and it belongs **here**, per task, not at phase close — scheduling it
     at close means a per-task omission stays invisible for as many tasks as the
     phase has left. `T10`'s `F-51`–`F-54` went four tasks unnoticed for exactly
     that reason, the second such drift (`notes.md` items 142, 143).

   If a phase reports complete: after the gate is green, flip it:
   `doctrine slice phase <N> <PP> --status completed`. **The flip is the
   orchestrator's, never the worker's**, and it happens after the gate, not after
   the report. Then read the boundary warning; if it names foreign commits,
   tighten: `doctrine slice record-delta <N> <PP> --start <first own>^ --end <own tip>`.
4. **Plan** — spawn the **planner** (§ Sub-agent briefs). It fills the runtime
   sheet; you do not. **Spawn it as `claude`, never as the `Plan` agent type** —
   `Plan` is read-only, so it cannot write the sheet and instead returns its
   whole body as text, which you then pay to receive *and* to write yourself.
   `PHASE-10`'s sheet came back as a 69 KB round-trip for this reason.
5. **Spawn** — one **worker**, one phase. Flip to `in_progress` *before*
   spawning, so a clock wake's guard sees it.
6. **Close the firing** — rewrite `handover.md` (§ Handover), `ScheduleWakeup`
   ~20 min, one line to the user: phase, beat, what's next.

**Hold your own commits while a worker is live.** A phase's delta is *one
contiguous range*, so a driver or notes commit landing between the worker's first
and last cannot be excluded by any `--start`/`--end` — it rides into the phase and
surfaces in `slice conformance`'s undeclared cell. Every phase so far has needed
`record-delta` for exactly this, and `PHASE-06` had one that could not be
tightened out at all. Make the edit when it bites; **hold the commit** until after
the worker's code tip. If you do commit mid-flight — a broken guard leg earns
it — say so at close rather than leaving the auditor to find it.

**The diagnostic feed shows the worker's uncommitted intermediate states — read
it as liveness, not as a defect.** A worker mid-refactor has helpers landed and
call sites not yet moved, so `dead_code` and unresolved-name diagnostics are the
*expected* shape of work in progress, not a finding. `T6` and `T7` both produced
them and both were clean at commit; the `T6` advisory fired on one and was wrong.
Before advising a live worker on a diagnostic, check whether the condition
survives into a commit — `git show <tip>:<path>` — because only a committed state
is a claim about anything. An advisory that turns out to be noise costs the
worker a cycle and costs you the credibility of the next one, which may be real.

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
- "**`slice verify-vt` is not a worker self-check.** It attributes against the
  phase's recorded delta boundary, which the orchestrator records *after* you
  exit — so your own phase reads `UNATTRIBUTABLE` however good your tests are.
  The gate is your evidence; `verify-vt` is the orchestrator's." Omitting this
  cost `PHASE-07` a finding, an observation and a chase (`notes.md` item 57).

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

**Commit each task as it goes green, not at the phase's end.** The uncommitted
window is the exposure: a harness restart takes everything not yet committed,
and it gives no warning. `PHASE-08` lost six hours of position that way. This is
the same argument as harvest-as-you-go, applied to the tree instead of the
knowledge.

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

**The diagnostic that catches this early: a section describing itself as the one
thing with no copy elsewhere.** That sentence is a bug report, not a boast —
this file is gitignored, so *no copy elsewhere* means *lost on `rm -rf`*. It was
written verbatim above `SL-248`'s verification ledger (the orchestrator's own
per-task gate runs, which `/audit` reads as evidence) and stood for several
firings before the slice owner caught it. Durable state does not belong here at
**any** length: move it to a tracked file in the slice directory and leave a
pointer. Extra files there are free — `doctrine validate` scans entity kinds and
ignores the rest. So the size rule is downstream of the real one: **the handover
holds pointers and the current firing's position, nothing a later reader would
have to rediscover.** A fat handover is the symptom; misfiled durable state is
the cause, and trimming prose while leaving the state in place fixes neither.

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
the same window. **The orchestrator still verifies — every firing, at beat 3**:
read the sheet's Findings against § *Owed* and confirm every `F-` reached it.
"The worker wrote it" is a claim like any other, and disk is truth (§ *The
contract*). Per firing rather than per phase because the check costs one grep
and the omission is otherwise invisible until close; when a late repair does
land, append it at the end with its out-of-order arrival stated, never
interleaved into the existing numbering.

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
