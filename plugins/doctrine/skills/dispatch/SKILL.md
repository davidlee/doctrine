---
name: dispatch
description: Use to drive a slice's phases to completion through sub-agent workers in isolated worktrees — you orchestrate and are the sole writer, the workers execute. Default is serial (one worker per phase) to keep your context clean and run the slice unattended without babysitting; parallelize file-disjoint phases into one concurrent batch when you can. Every worker's source delta funnels back through one strict cadence (import → verify → branch-point guard → one commit → record); conflicts report-and-halt, never auto-merge.
---

# Dispatch

Drive a slice's phases to completion through sub-agent **workers** — you are the
orchestrator and **sole writer**, they execute. You spawn each worker into an
isolated worktree fork, collect its **source delta** (its fork branch), and funnel
that delta into the coordination branch through one strict, crash-recoverable
cadence. Workers mutate source; **only you** make doctrine-mediated writes.

**Default serial, parallel by opportunity.** The point is to drive the whole slice
unattended without burning your own context — *not* to hunt for parallelism. One
worker per phase is the norm: a worker burns the phase's tokens so yours stay clean,
and you run phase after phase without babysitting. When two or more phases are
**file-disjoint** you *may* admit them to one concurrent batch — an optimization,
never the entry condition. A slice with no parallelism is still a dispatch slice;
**serial is a batch of one, same funnel**. Bailing to inline serial execution
because "nothing parallelizes" is the failure this skill exists to prevent.

**Announce at start:** "Using the dispatch skill to run workers under the
orchestrator funnel."

**This is a sub-skill, not yet a `/route` destination.** It is the unattended,
sub-agent analog of `/execute` (which is serial, inline, you-drive-one-phase). Reach
it when you want to drive a slice's phases to completion *through workers* instead of
executing them inline yourself — **whether or not any phase parallelizes**. Until a
routing slot is wired, the caller invokes it directly. To run a single phase inline
in your own context, `/execute` remains the path.

**Composes `/worktree mode=worker base=<B>` — do not restate it.** The rung-3 fork
from the explicit base `B`, provision, spawn guards, baseline (`fork == B`), and the
worker's `self-arm → mutate-source → verify → commit-one-S` loop all live in the
[worktree skill](../worktree/SKILL.md). This skill is the orchestrator half only:
batching, the import funnel, and reconciliation. You pass `B`; the worktree skill
pins the fork to it.

## The drive loop — phase by phase to slice-done

Dispatch drives the *whole* slice, not one batch. The import funnel below is the
**inner** loop (land one unit); this is the **outer** loop (drive to completion):

1. **Plan the next unit.** `/phase-plan` the next phase — or, if several upcoming
   phases are file-disjoint, plan them together as one concurrent batch. A phase that
   cannot be delegated (spec / authoring work — see Red Flags) you execute inline
   yourself, then continue the loop.
2. **Spawn worker(s).** One worker per phase in the unit, each in its own worktree
   fork (below). A serial unit is a single worker; that is the common case.
3. **Funnel.** Run the strict per-batch cadence (below) to land the unit as exactly
   one commit on the coordination branch.
4. **Repeat** from the new HEAD until the slice's phases are done.
5. **Hand over on cadence — a quality gate, not an overflow stop.** Your context is
   disposable and rebuildable from the coordination branch (see Crash / overflow
   recovery), so handover is cheap; the **dumb zone is not**. Reasoning quality decays
   long before any capacity limit, so hand over *early*, while you are still sharp —
   do **not** wait to "run low." You cannot read your own token count in-loop and no
   human is watching the unattended drive, so trip on what you **can** count, at the
   next **committed** batch boundary, whichever comes first:
   - **`handover_after` batches** since your spawn (default `5`), **or**
   - **`handover_delta` cumulative reviewed-delta lines** since your spawn (default
     `2000`) — you already compute each `B..S`; sum them. Big phases (the real context
     fillers) trip this sooner than a raw count would.

   Then stop at the committed boundary and `/handover`; a fresh orchestrator resumes
   from the branch. **Never carry a half-imported batch across a handover.** Defaults
   are deliberately conservative starting points — tune via
   `/dispatch handover_after=N handover_delta=L`.

The slice reaches done unattended: you alternate plan → spawn → funnel without
hand-holding each phase, and hand over **early and cleanly** to stay out of the dumb
zone — many lean orchestrator instances, never one bloated one.

## Context hygiene — keep the orchestrator lean

A bounded, roughly-uniform per-batch footprint is what makes the handover cadence
above meaningful — without it, `handover_after` is noise (a 5-line delta and a
4000-line failure log both count as "one"). Keep your own context small so N batches
≈ a fixed budget:

- **Worker reports enter your context structured and size-capped** — a verdict plus a
  short summary, never raw build/test logs. Mandate the shape in the spawn prompt.
- **Read deltas `--stat` / `--name-only` first.** Pull a full `B..S` diff into context
  only when a check actually needs the body; the R-5 belt and disjointness checks run
  on name-only.
- **Verify output enters as pass/fail + a short tail**, never the full log. On RED you
  keep the tail to name the offender (X-3); you do not need the whole run.

These also shrink the unobservable tail (failure noise) that `handover_delta` can't
see — so the two levers and the hygiene rules reinforce each other.

## Remit — orchestrator is the sole writer (D6a)

| Party | Branch | Worker-mode | Writes |
|---|---|---|---|
| **Orchestrator (you)** | coordination | **OFF** | every doctrine-mediated write: the import commit, memory, AC evidence, notes, status |
| **Worker** | its fork | **ON** (self-armed) | **source only**, committed as one non-merge `S` to the fork branch — never doctrine state |

The fork withholds the coordination/runtime tier by construction (`/worktree`
provision exclusion, D9). A worker returns a **source delta + a structured report**;
it is never a doctrine artifact writer. You alone advance the coordination branch.

## Worker spawn — fork rung-3 from `B`, never the implicit session HEAD

Spawn each worker (via your harness's sub-agent mechanism) so it runs
`/worktree mode=worker base=<B>`, where `B` is the coordination HEAD you captured
pre-spawn. The worktree skill rung-3 forks `git worktree add <dir> <branch> <B>`,
provisions, guards (**baseline asserts `fork == B`**), verifies a green baseline, then
runs its constrained edit→verify→commit-`S` loop and returns
`{ fork_branch, head_sha_after }`.

The hazard is **harness-agnostic**: the orchestrator drives the coordination branch
while the session repo may sit elsewhere (e.g. `main`), so the **current/session HEAD
is not `B`**. A fork that inherits the implicit current HEAD instead of the explicit
`B` drags the session↔coordination divergence into `B..S` — a wrong-base corruption
(proven in PHASE-01: `main`'s slice work pulled into a sibling slice's delta). Always
pass `B`; never let the fork default to current HEAD.

> **Claude Code note.** Spawn with the `Agent` tool as a **plain agent** that self-forks
> rung-3. Do **not** use `Agent` at `isolation: worktree`: that backend builds the fork
> from the *session* HEAD (not `B`) and gives no reliable isolation here — it is the
> concrete way the implicit-HEAD trap bites under Claude Code. Harnesses without such a
> backend can ignore this note; the rung-3 rule above already protects them.

Isolation remains **mandatory** — a real sibling-dir fork. A worker that cannot get
one, or whose baseline `fork == B` fails, is a **hard abort** (worktree skill,
`worker` contract) — never an in-tree edit. Isolation *and* the correct base are the
funnel's whole premise.

### Pre-distilled worker prompt (D6 — self-contained, no governance read)

Workers **do not** read boot/governance or run `/route`/`/boot`. You pre-distill
everything a worker needs into its spawn prompt:

- **policy digest** — the rules of the road that bear on the task (lifted from your
  own loaded governance, not re-derived by the worker);
- **design excerpts** — the relevant design/contract slices;
- **pre-fetched memories** — the scope-bound gotchas you already retrieved;
- **task spec + declared file set** — what to change and exactly which files (the
  file set is load-bearing for disjoint batching, below);
- **mandatory verify command** — the project's green-gate (doctrine is a framework;
  never assume `just check` — pass the project's command explicitly);
- **the self-arm mandate** — the worker's first act is `export DOCTRINE_WORKER=1`.
- **the escalation contract** — on an architectural fork, or a task it cannot complete
  cleanly within its declared files, the worker **stops and reports** rather than
  improvising design or straying outside its file set. It has no governance read, so
  the decision comes **up to you** — you `/consult` it, never the worker.

**`DOCTRINE_WORKER=1` is a self-armed prompt contract that fails OPEN (C-I).** The
`Agent` tool exposes **no env seam**, so you cannot set the var in the worker's spawn
environment — only the prompt can mandate it, and nothing enforces it. A worker that
omits the line runs with the doctrine CLI fully open (the D2a guard inert). Mandate
the line regardless, but **do not rely on it** — the enforceable protection is the
import-time R-5 belt below, which you run on the trusted side.

## Batching — serial by default, file-disjoint to parallelize (C-III)

The default unit is **one phase, one worker** — a batch of one. You only *widen* a
batch to run workers concurrently when their declared changed-path sets are
**pairwise disjoint.** A task that would share a file with another in the same batch
stays in its **own** serial batch — not a degraded outcome, just the normal one. The
funnel is identical for a batch of one; you simply run the batches one after another.

**Dependency-disjoint is not enough** to parallelize. Two independent tasks routinely
edit the same file (e.g. two unrelated subcommands both touching `main.rs` — exactly
this slice's own verb-wiring + minting-wiring). File-disjoint is the stronger contract
that makes the deltas co-apply onto the captured base `B` cleanly. Shared file ⇒
separate serial batches, never one concurrent batch.

## The funnel — strict per-batch order (D7)

The cadence is **the batch, not the worker** (§5.1: a per-worker commit moves HEAD,
landing the next delta on a moved base). Capture `B = git rev-parse HEAD` pre-spawn
(tree clean). After the batch's workers return, run **in this exact order**:

1. **precond (X-1).** Assert the coordination **worktree AND index are clean** and
   `HEAD == B`. A dirty tree would be swept into the batch commit while a bare sha
   guard still passes — so check both. Not clean ⇒ **abort**.
2. **delta (X-2).** Each worker's delta is the **net diff `B..S`**, where `S` is the
   one non-merge commit on the fork branch. **Assert `git rev-parse S^ == B`** — the
   immediate parent *is* `B`, not merely an ancestor. This is the trusted-side belt
   against a divergent-base fork (a worker spawned at session HEAD instead of `B`):
   if `S^ != B`, the net diff would smuggle the session↔coordination divergence into
   the import ⇒ **reject before import**. Also validate single non-merge commit;
   multi-commit / merge / rebased fork ⇒ contract violation, reject. A net diff,
   **not** a `cherry-pick`/replay — so the belt-check and the import-effect are the
   same object.
3. **R-5 belt — reject authored-tree touches (C-II).** For each delta,
   `git diff --name-only B..S`; if any path is under `.doctrine/` authored trees ⇒
   **report + halt**. This belt protects PHASE-01's trunk-minting guarantee from an
   unarmed worker that minted authored ids in its fork (A and B are **not**
   failure-independent — do not drop it). It is sound where the env contract is not,
   because **you** run it: worker-mode OFF, the trusted sole writer, mechanically
   checkable.
4. **import — non-committing.** `git apply` **every surviving net-diff onto `B`**,
   NON-committing. A genuine apply **conflict** on a file-disjoint batch means the
   changed-path analysis was wrong (or a worker strayed outside its declared paths)
   ⇒ **report + halt**, human re-plans. Never auto-resolve.
5. **verify — combined tree.** Run the project verify command on the combined working
   tree. On **RED**, re-run verify against **each delta alone** (the forks are already
   isolated) to **name the offending worker** (X-3 — file-disjoint removes git
   conflicts, not semantic coupling) ⇒ report + halt. "report+halt" is never blind.
6. **branch-point guard (D5 under concurrency).** `doctrine worktree
   branch-point-check --base B` — coordination HEAD still `B`? Because the whole
   disjoint batch imports onto the single `B` and commits once, HEAD only moves at
   **your** batch commit; a mismatch means an **external** mover ⇒ **re-dispatch the
   batch from the new HEAD**, never commit against a moved base. (Naming note C-V: a
   HEAD-stationarity compare, not a merge-base computation.)
7. **commit — one batch commit.** ONE commit on the coordination branch ⇒ `HEAD = B+1`.
8. **record — knowledge trails the commit.** Memory / AC evidence / notes, *after*
   the confirmed commit. Knowledge always trails confirmed code; the coordination
   branch is the durable store, your context is disposable.

The next batch forks from `B+1`.

**Report-and-halt, never auto-merge.** Conflict, moved HEAD, or a `.doctrine/`-touch
all stop the funnel and surface to a human (ADR-006: policy is report, never
auto-resolve).

## Crash / overflow recovery

No orchestrator state is load-bearing. Rebuild from the **coordination branch** +
`git worktree list`: committed batches are durable on the branch; in-flight forks are
re-imported (their `B..S` still applies) or re-dispatched. Context overflow is just a
crash — recover the same way.

## Out of scope (v1)

- **Remote / non-shared-store workers (C-VI).** The no-transport import assumes the
  rung-3 fork is a local `git worktree` (shared `.git`, as Claude Code does), so you
  read the fork branch directly. A remote agent would hand back a `git format-patch`
  series applied `git am`-style through the **same** import→reject→verify→guard→commit
  cadence — noted, not specified here. v1 assumes the shared object store.
- **A routing slot.** `/dispatch` is not yet a `/route` destination; wiring fan-out
  into `/route` is deferred until this path is exercised.

## Quick Reference

| Situation | Action |
|---|---|
| No phase parallelizes | **Serial — one worker per phase, batch of one, same funnel.** The norm, not a fallback; never bail to inline |
| Drive the slice | Loop: `/phase-plan` next unit → spawn worker(s) → funnel → repeat from new HEAD until done |
| Handover cadence (quality gate) | Hand over **early**, at a **committed** boundary, on `handover_after` batches (def 5) **or** `handover_delta` cumulative `B..S` lines (def 2000), whichever first — stay out of the dumb zone |
| Worker reports a fork / can't finish clean | It halted by contract → **you `/consult`** the decision; never auto-adapt plan or design to push the drive forward |
| Phase can't be delegated (spec / authoring) | Execute it inline yourself, then resume the loop; dispatch the delegable phases |
| Spawn a worker | Sub-agent runs `/worktree mode=worker base=<B>`; rung-3 fork from `B`, baseline `fork==B`; pass `B`, never inherit current HEAD. *(Claude Code: plain `Agent`, never `isolation: worktree`)* |
| Worker prompt | Pre-distilled: policy digest, design excerpts, memories, task spec + file set, verify cmd, `export DOCTRINE_WORKER=1` mandate |
| Two tasks share a file | **Separate serial batches** — file-disjoint is required to parallelize (dependency-disjoint is unsound) |
| Batch returned | precond clean+`HEAD==B` → net diff `B..S` → R-5 reject → apply non-committing → verify → branch-point → one commit → record |
| Delta touches `.doctrine/` | **Report + halt** (R-5 belt — the real protection; `DOCTRINE_WORKER=1` fails open) |
| Worker fork `>1` / merge / rebased commit | **Reject** before import (the unit is net diff `B..S`, `S` one non-merge commit) |
| Combined verify RED | Re-verify each delta alone to NAME the offender → report + halt (X-3) |
| Apply conflict on disjoint batch | Changed-path analysis was wrong → **report + halt**, human re-plans |
| `branch-point-check` exits 1 | External HEAD move → **re-dispatch** the batch, never commit on a moved base |
| Crash / context overflow | Rebuild from coordination branch + `git worktree list`; no load-bearing state |

## Red Flags

**Never:**
- Let a worker write `.doctrine/` authored trees, or import a delta that touches them
  (the R-5 belt is non-droppable — it, not `DOCTRINE_WORKER=1`, is the real protection).
- Imply `DOCTRINE_WORKER=1` *enforces* anything — it is self-armed and fails open (C-I).
- Commit per worker (HEAD moves; the next delta lands on a moved base) — commit **per
  batch**, once.
- Replay fork history (`cherry-pick`) instead of applying the **net diff `B..S`**.
- Auto-merge or auto-resolve a conflict / moved HEAD / authored-tree touch — **report
  and halt**.
- Auto-adapt the plan or design to keep the drive moving. An emergent
  architecturally-significant decision is the **semantic** report-and-halt: `/consult`
  it, never decide solo mid-drive. Unattended raises the stakes — nobody is watching,
  and a wrong call lands as a commit and compounds across the next phases.
- Record knowledge before the confirmed commit (it must trail the code).
- Bail to serial **inline** execution because no phase parallelizes — serial still
  means spawn a worker in a worktree (batch of one). Inline is only for non-delegable
  authoring / spec phases.
- Let a worker fork from the implicit current/session HEAD instead of the explicit
  `B` — session HEAD ≠ `B`, so it drags unrelated commits into `B..S`. Pass `B`;
  rung-3 fork; baseline `fork == B` or abort. *(Claude Code: this is exactly why you
  never spawn at `isolation: worktree`.)*
- Let a worker degrade to an in-tree edit, or batch two tasks that share a file.
- Restate the worker loop or the worktree guards here — link to the worktree skill.
- Author or edit this skill in `.doctrine/skills/` (the gitignored install copy); the
  source of truth is here under `plugins/`.

**Always:**
- Default to one worker per phase in its own worktree; parallelize only file-disjoint
  phases. Drive the whole slice phase by phase, handing over **early on cadence** to
  stay out of the dumb zone, and `/consult` emergent architectural forks.
- Keep your own context lean (capped worker reports, stat-first diffs, verify tails)
  so the handover cadence tracks a real budget.
- Run as worker-mode OFF, the sole doctrine-mediated writer, on the coordination branch.
- Pre-distill a self-contained prompt; workers never read boot/governance.
- Keep concurrent batches file-disjoint; run shared-file tasks as separate serial batches.
- Hold the strict funnel order; the R-5 belt and the branch-point guard are mandatory.
- Make knowledge trail the confirmed commit; keep all durable state on the branch.

## Outcome

Driven phase by phase, the slice reaches completion unattended. Each batch — usually
a single serial worker, occasionally a file-disjoint concurrent set — lands as exactly
one commit on the coordination branch, every imported delta policy-checked and verified
before it lands, with conflicts surfaced to a human rather than merged. The coordination
branch is the deliverable.
