# LOOP.md — dumb-loop slice driver

A self-paced `/loop` drives one slice to completion. Each firing may be a **cold
context**: it knows nothing except this file, the disk, and the CLI. Nothing
load-bearing is carried in an agent's head between firings, and nothing may be.

**This is a template.** Copy it to `.doctrine/slice/<N>/LOOP.md`, bake `<N>` in,
customise § *Where this runs*, and fire the loop at the copy — extra files in a
slice directory are free. Fold anything durable the copy learns back here at
close. Untouched, `<N>` comes from the `/loop` prompt, passed verbatim to each
firing; if it names no slice, stop and ask — never guess from `doctrine status`,
which lists every active slice and adjudicates between none.

> **Read every firing, so its length is a recurring cost.** Keep it near ~200
> lines. Phase detail belongs in the sheet; anything reusable past this slice
> belongs in a memory (§ *Method*).

## The contract

1. **Disk is truth, transcript is not.** Verify from `doctrine`, `git`, and the
   sheet — never a report, a recollection, or a summary. A worker that says
   "done" against a tree that says otherwise: the tree wins.
2. **Any firing may be the last.** Kill the session anywhere; the next firing
   resumes with no loss.
3. **Three writers, and they do not overlap** — § *Writer map*.
4. **One sub-agent at a time**; two collide in one index and one `target/`.
5. **`git status --porcelain` first**, then `git add <paths>` and
   `git commit <paths> -F -`. Never pathless, never `-A`, never stash.

## Where this runs

**Customise per slice — the one part that is not portable.**

Default: the primary worktree on its own branch, **no isolation** — orchestrator
and sub-agents share one tree, one index, one `target/`. Nothing merges back
because nothing forked.

- **Never switch the branch.** Read another ref with `git show <ref>:<path>`;
  restore with `git restore --source=<ref> -- <explicit paths>`.
- **Minting is normal here** — the allocator scans a live corpus, so mint
  `DEC-`/`ISS-`/`RV-`/`REQ-` as findings arise. **Invert on a clone or fork**:
  two trees scanning a corpus frozen at fork time mint the same id and
  renumbering breaks immutability, so capture in the sheet and mint at merge.
  `memory record` and `observation record` are exempt either way.

## Cadence and the re-entrancy guard

**Self-paced, notification-driven.** Sub-agents run in the background, so
completion wakes the orchestrator directly. The clock is a *fallback*: after each
spawn, `ScheduleWakeup` ~20 min to catch one that hung or died without notifying.
No polling interval — a no-op firing is a bug to design out, not a cost to budget.

**Establish why you woke.** A task notification means that sub-agent has exited —
skip the liveness test, which proves nothing about a dead process, and go to beat
3. A clock wake means run the guard.

**The notification is the only proof of completion. Silence is never proof.** The
legs below are *alive* detectors: each can say "still working", none can say
"done", and a worker between two tool calls is quiet exactly as a finished one
is. Absent a notification treat it as **live** whatever the tree looks like — do
not spawn, do not commit, do not run a gate. Silence licenses only *death*, and
death is resumed, never read as completion.

**A worker is alive if *any* leg says so** — each is blind where another sees, so
run all four **as written**. Mtimes beat `pgrep`, which is an *instantaneous
sample*: a worker between builds shows no `cargo` while plainly working.

```bash
./target/debug/doctrine slice status <N>
# sheet mtime — ticking a task; blind mid-task, however long that takes
find .doctrine/state/slice/<N>/phases -name 'phase-*.md' -mmin -25 | head
# build artifacts — compiling, testing; blind while reading or writing source
find target/debug/.fingerprint target/debug/deps -maxdepth 0 -mmin -30
# dirty-path mtime — writing source; blind while building or just after a commit.
# Rides `git status` for .gitignore; needs -I{} or find exits "paths must precede
# expression" and the leg reads as a silent dead.
git status --porcelain | cut -c4- | tr '\n' '\0' | xargs -0 -r -I{} find {} -maxdepth 0 -mmin -30
# last commit — landing a task; blind mid-task. Covers the just-committed worker,
# whose clean tree makes the churn leg read dead when it was most productive.
git log -1 --format=%cr
```

| what you see | what you do |
|---|---|
| phase `in_progress`, sheet < 25 min old | **exit.** Live. One line, re-arm, stop. |
| phase `in_progress`, sheet cold, **any other leg fresh** | **exit.** Alive, just slow. |
| phase `in_progress`, sheet cold > 90 min, **all four silent** | it died — triage, § *Method*. |
| phase `completed`, a next phase exists | beat 4 — spawn the planner. |
| phase `planned`, sheet > 100 lines (filled) | beat 5 — spawn the worker. |
| phase `planned`, sheet ~27 lines (bare template) | beat 4 — spawn the planner. |
| no phases left | § *Stop conditions*. |

`wc -l` is the empty/filled test. Don't eyeball it.

**Sheet mtime alone is not liveness; reaping on it kills healthy workers.** It
goes quiet exactly when the work is slowest and dearest to lose — a ten-mutation
battery is ten build-and-test cycles with nothing tickable between them — so the
proxy inverts under load: the more expensive the phase, the deader it looks.
**Any sign of life beats a cold sheet**; when legs disagree, believe the one
saying alive. Re-spawning is not idempotent — a revived worker redoes everything
since the last tick, and one reaped mid-write leaves a half-edited file the next
reads as finished. **Waiting is cheap, reaping is not**: that asymmetry, not the
numbers, is the rule. Three consecutive revives of one phase → stop and report.

Silence has three explanations that present identically and route differently:
dead worker (resume from its transcript), dead harness (only a human restarts
it), rebuilt sandbox (re-spawn; resume is gone). Triage before acting —
`doctrine memory show mem.pattern.dispatch.loop-death-triage-resume-vs-respawn`.

## The orchestrator's turn

Opus, and **thin** — it routes; it reads neither source nor `design.md`. Beats in
order, stopping at the first that ends the firing:

1. **Guard** — above. Exit if a sub-agent is live.
2. **Orient** — `handover.md`, then `git log --oneline -5`. Nothing else.
3. **Verify the last claim**, every firing, not only at close. *The gate is
   evidence, the report is a claim*: run `doctrine check gate` yourself and
   confirm the test-count delta matches what the worker claims. *Findings reached
   § Owed*: one grep — per task, because at close an omission stays invisible for
   as many tasks as the phase has left. If a phase reports complete, flip it
   *after* the gate is green (`doctrine slice phase <N> <PP> --status
   completed`); **the flip is the orchestrator's, never the worker's**. Then read
   the boundary warning and, if it names foreign commits, tighten with
   `doctrine slice record-delta`.
4. **Plan** — spawn the **planner**; it fills the sheet, you do not. Spawn a
   general agent, **never a read-only `Plan` type**: that cannot write the sheet
   and returns its whole body as text, which you pay for twice.
5. **Spawn** — one **worker**, one phase. Flip to `in_progress` *before* spawning
   so a clock wake's guard sees it.
6. **Close the firing** — rewrite `handover.md`, `ScheduleWakeup` ~20 min, one
   line to the user: phase, beat, what's next.

Beats 4 and 5 may share a firing. What must never share a *context* is planning
and execution; planning and spawning may.

**Hold your own commits while a worker is live.** A phase's delta is one
contiguous range, so a driver or notes commit landing between the worker's first
and last cannot be excluded by any `--start`/`--end` — it rides into the phase
and shows up in `slice conformance`'s undeclared cell. Make the edit when it
bites, hold the commit until after the worker's code tip, and if you do commit
mid-flight, say so at close rather than leaving the auditor to find it.

## Sub-agent briefs

Both roles run in-tree, no isolation, and are told so. How to write the brief
itself — hypothesis not route, unstageable reds, inventories against the tree —
is `doctrine memory show mem.pattern.dispatch.brief-a-diagnosis-as-a-hypothesis`.
Every brief carries, and carries nothing else:

- the slice and phase id and the sheet path — **the sheet is the brief**;
- "read `LOOP.md` § *Sub-agent discipline* and § *Where this runs* — both bind you";
- "another agent shares this index: `git status --porcelain` first, path-limit
  every commit";
- "commit per task and harvest at that boundary, not at the end";
- "end green: `doctrine check gate`. Do not flip your own phase status";
- "**`slice verify-vt` is not a worker self-check** — it attributes against the
  delta boundary the orchestrator records *after* you exit, so your own phase
  reads `UNATTRIBUTABLE` however good your tests are."

**Planner** — Opus, always. Runs `/phase-plan` for one phase: reads that phase's
`plan.toml` entry, and of `design.md` only what its row in `plan.md`
§ *Design-section provenance* grants. Writes tasks, carried constraints, STOP
conditions, the `VT`/`VA` mapping, risks, decisions and the notes shard's name
into the sheet; writes no source. Hands back ≤12 lines: task count, open
decisions, which tasks cannot stage a red and their controls, whether the phase
wants one worker or two in sequence, and the worker model it recommends.
`plan.toml` has **no** Reading-list field — § *Reading list* is something the
planner **writes**; brief one to go read a field that does not exist and it will
improvise a scope for itself, which is what the provenance table prevents.

**Worker** — Sonnet when the sheet is fully specified and the work mechanical;
Opus when it carries an open decision, a STOP condition likely to fire, or
verification the worker must design. In doubt, Opus — a Sonnet that improvises
past a design gap costs more than the Opus that would not have.

## Sub-agent discipline

**Tick a task only after its knowledge is durable.** Finish → harvest → *then*
tick and note the evidence in Findings. A ticked box with un-harvested knowledge
is a lie the next firing believes, and the tick is also the heartbeat.

**Harvest as you go, never at the end** — context exhaustion is the expected
ending, not the exception. Harvest at each task's commit boundary:

- durable gotcha / pattern / footgun → `doctrine memory record`;
- friction, confusion, token waste → `doctrine observation record friction …`;
- execution record, divergences, measurements → the notes shard;
- anything needing a `DEC-`/`ISS-`/`RV-` id → mint it (§ *Where this runs*) and
  cite the id in Findings, so the orchestrator reads a reference.

**Stop, do not improvise.** A STOP condition, a design gap, a criterion that does
not compile as written, a decision needing a human ruling: write it into
Findings, leave the box unticked, hand back. A criterion that does not hold is a
stop-and-report, **never** an adjust-the-criterion — the loop halting on a real
question is the cheapest outcome available.

**Commit per task, path-limited**, scoped `feat(SL-<N>): PHASE-<PP> T<k> — …`.
The commit is not the point, the *boundary* is: it is where you still remember
why, so the message, the Findings line and the harvest all get written cheaply.
What survives a dead worker is code and commits; what dies with it is every
insight still only in its head. A task is done when all three exist, not before.

**End green.** `doctrine check gate`, which builds before validating and so gives
the corpus check a fresh binary.

## Notes, sharded

`notes.md` is tracked; `handover.md` is **gitignored** and does not survive an
`rm -rf` of state. Never put anything load-bearing only there.

- `notes_01-03.md`, `notes_04-06.md`, … — the execution record: what was done,
  what diverged, what was measured. A worker appends to its own shard only.
  **The sheet names the shard; a brief must not** — ranges follow the order
  phases *executed*, not their ids, so an orchestrator deriving the name from
  `PHASE-NN` eventually names a shard the sheet disagrees with, leaving the
  worker two instructions and no way to rank them.
- `notes.md` stays small and holds only what outlives a phase: an index of the
  shards; **Owed to the reconciliation brief**, the ledger `/audit` reads first
  and which never moves into a shard; cross-phase invariants and decisions; Open;
  Learned (pointers to memories); and **Traps**, what already bit this slice —
  the one section that grows monotonically, hence tracked here.

## Handover

Rewritten by the orchestrator at the end of every firing. ~50 lines, no history —
history is the shard.

```
## Where we are      one line: slice, phase, status, beat
## Last firing       what changed, and the sha
## Next              the single next action, verbatim enough to execute
## Live              open decisions, blockers, awaiting-human
## Traps             a pointer to `notes.md` § Traps, not a second copy
```

**The diagnostic: a section describing itself as the one thing with no copy
elsewhere.** That is a bug report, not a boast — the file is gitignored, so *no
copy elsewhere* means *lost on `rm -rf`*. Move it to a tracked file and leave a
pointer. The size rule is downstream: a fat handover is the symptom, misfiled
durable state the cause, and trimming prose fixes neither.

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
| authored `.doctrine/` entities | orchestrator only |
| new entity **ids** | whoever needs one — but see § *Where this runs* |
| memories, observations | whoever learns it, at the moment it bites |

**The worker owns § *Owed*** because it already holds the finding. Two
constraints keep that safe: **append at the end, never edit an existing item**
(numbering is immutable, and the orchestrator may be writing another section in
the same window), and **the orchestrator still verifies at beat 3, every
firing** — "the worker wrote it" is a claim like any other.

## Stop conditions

`ScheduleWakeup(stop: true)` and report, on any of:

- **Done.** Last phase `completed`, gate green → route to `/audit`, which needs a
  human in the loop and is not loop work.
- **Blocked.** A sub-agent hit a STOP condition, or a ruling is owed. Ask; do not
  spawn again.
- **No progress.** Three firings with no status change and no new commit, or
  three revives of one phase.

**Budget is NOT a stop condition.** It is a hand-over condition, and it is the
one beat where getting the distinction wrong turns an autonomous loop back into a
babysat one — compaction is survivable by construction, so keep going and just
make sure `handover.md` reflects disk before you get close. Why, and what a
firing costs: `mem.pattern.dispatch.budget-is-a-handover-condition`.

Delegation is what keeps the orchestrator alive — planning a phase in-context
burns the loop down in three or four phases. If it is reading source or
`design.md`, the split has failed. Read entities with `doctrine <kind> show
<ID>`, and use this tree's `./target/debug/doctrine`, never the PATH binary, or
you will read a corpus through a stale parser.

## Method

Reusable past one slice, so it lives in the corpus rather than in a file read
every firing — `doctrine memory show <key>`:

| key | what it settles |
|---|---|
| `mem.pattern.dispatch.loop-death-triage-resume-vs-respawn` | dead worker vs dead harness vs rebuilt sandbox; how to word a revive |
| `mem.pattern.dispatch.brief-a-diagnosis-as-a-hypothesis` | constraints not routes; unstageable reds; inventories against the tree; a diagnostic is liveness before it is a defect |
| `mem.pattern.dispatch.budget-is-a-handover-condition` | why low context is never a stop; what an orchestrator firing costs |
| `mem.pattern.testing.convict-a-race-by-causality-not-repetition` | load the hazard's own channel; isolate process-wide state; two-arm experiment |
| `mem.pattern.testing.floor-a-destructive-instrument-before-aiming-it` | a phase that signals, deletes or unmounts bounds itself in its *first* commit |
