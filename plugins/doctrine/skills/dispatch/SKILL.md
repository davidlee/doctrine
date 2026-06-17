---
name: dispatch
description: Use to drive a slice's phases to completion through sub-agent workers in isolated worktrees — you orchestrate and are the sole writer, the workers execute. A thin router that detects the harness and hands off to `/dispatch-subprocess` (pi / codex) or `/dispatch-agent` (claude); the funnel cadence (import → verify → branch-point guard → one commit → record) is identical on both arms. Default serial (one worker per phase); parallelize file-disjoint phases. Conflicts report-and-halt, never auto-merge.
---

# Dispatch (router)

Drive a slice's phases to completion through sub-agent **workers** — you are the
orchestrator and **sole writer**, they execute. You spawn each worker into an
isolated worktree fork, collect its **source delta** (its fork branch), and funnel
that delta into the coordination branch through one strict, crash-recoverable
cadence. Workers mutate source; **only you** make doctrine-mediated writes.

**This skill is the harness-identical half** — the drive loop, the import funnel,
batching, and reconciliation. The harness-*shaped* spawn (how a worker is created
and identified) lives in the two arm skills; this skill **routes** to the right
one. The funnel cadence below is the same regardless of arm.

**Announce at start:** "Using the dispatch skill to run workers under the
orchestrator funnel."

**Default serial, parallel by opportunity.** The point is to drive the whole slice
unattended without burning your own context — *not* to hunt for parallelism. One
worker per phase is the norm; a slice with no parallelism is still a dispatch slice
(**serial is a batch of one, same funnel**). When two or more phases are
**file-disjoint** you *may* admit them to one concurrent batch — an optimization,
never the entry condition. Bailing to inline serial execution because "nothing
parallelizes" is the failure this skill exists to prevent.

**Sub-skill, not yet a `/route` destination.** It is the unattended, sub-agent
analog of `/execute`. The caller invokes it directly until a routing slot is wired.

## Routing — detect the harness, hand off (D6/§4)

The CLI half is harness-identical; only the spawn line differs. Route on the
agent's **harness self-belief cross-checked against env-marker detection**:

| Detected harness | Hand off to | Spawn mechanism |
|---|---|---|
| **pi** (subprocess-capable, pi-subagents extension) | [`/dispatch-subprocess`](../dispatch-subprocess/SKILL.md) (pi row) | `subagent(agent="dispatch-worker", task="<prompt>", cwd="$D")` — see subprocess skill |
| **codex** (subprocess-capable, env channel) | [`/dispatch-subprocess`](../dispatch-subprocess/SKILL.md) (codex row) | `env -C "$D" DOCTRINE_WORKER=1 codex exec "<prompt>"` — legacy placeholder, untested end-to-end |
| **claude** (`Agent` tool, `isolation: worktree`) | [`/dispatch-agent`](../dispatch-agent/SKILL.md) | `Agent` `subagent_type: dispatch-worker` + SubagentStart stamp |

- **Cross-check, don't trust self-belief alone.** Confirm the agent's stated
  harness against an env marker. Route **only when they agree:**

  | Harness | Env marker | Detection order |
  |---|---|---|
  | pi | `PI_HOME` (set by pi binary at startup) | 1st — `PI_HOME` + self-belief="pi" |
  | claude | `CLAUDECODE` | 2nd — `CLAUDECODE` + self-belief="claude" |
  | codex | Unknown (deferred to codex spike) | 3rd — self-belief="codex", no env marker known |

- **Mismatch ⇒ refuse, NAMING the cause** (e.g. "self-belief=pi but no `PI_HOME` in env"), never a blind spawn.
- **Unknown harness ⇒ refuse** (not pi, not claude, not codex — never guess).

The arms own the spawn template, worker-identity mechanism, and arm-specific
residuals (base-pinning, self-clear, concurrency floor). Everything below is shared.

## Set up once — the coordination worktree (SL-064 / ADR-012)

Before batch 1, create or resume the slice's **dedicated coordination worktree**:

```sh
doctrine worktree coordinate --slice <N> --dir <path>
```

This creates (or reattaches to) `dispatch/<slice>` **in its own isolated worktree**
off the resolved trunk — the funnel's **sole write target**. The session `main`
working tree is **never** the funnel target; driving the funnel on `main` is the
SL-060 contention bug this topology fixes. The verb is **markerless** (the
coordination tree *is* the orchestrator — worker-mode OFF) and **resume-stable**: a
live worktree already on `dispatch/<slice>` is refused (`coordination-live`); a
branch with no live worktree **resumes** (reattach, never a second branch), so a
fresh orchestrator after `/handover` picks up the same branch. Drive the loop below
from that worktree; capture `B` as a ref on `dispatch/<slice>`, not on `main`.

## The drive loop — phase by phase to slice-done

Dispatch drives the *whole* slice. The import funnel is the **inner** loop (land one
unit); this is the **outer** loop (drive to completion):

1. **Plan the next unit.** `/phase-plan` the next phase — or, if several upcoming
   phases are file-disjoint, plan them together as one concurrent batch. A phase
   that cannot be delegated (spec / authoring — see Red Flags) you execute inline
   yourself, then continue.
2. **Spawn worker(s)** via the routed arm — one worker per phase in the unit, each
   in its own fork. A serial unit is a single worker; the common case.
3. **Funnel.** Run the strict per-batch cadence (below) to land the unit as exactly
   one commit on the coordination branch.
4. **Repeat** from the new HEAD until the slice's phases are done.
5. **Conclude — stage-1 only, never integrate (ADR-012 D4/D5).** When the last
   phase lands, the orchestrator's job is to leave **reviewable refs**, not to land
   code:
   - **Project for review:** `doctrine dispatch sync --slice <N> --prepare-review`
     — materialises `review/<slice>` (the impl-bundle diff) + `phase/<slice>-NN`
     (the per-phase code units) + the CAS journal, **without writing trunk**.
   - **Remove the coordination worktree directory**, but **KEEP** the
     `dispatch/<slice>`, `review/<slice>`, and `phase/<slice>-NN` refs — they are
     the deliverables, preserved until integration. (Codex/pi only: `doctrine
     worktree gc --fork <branch>` each spent **worker** fork; worktree removal
     strands test binaries that baked the fork path via `CARGO_MANIFEST_DIR` —
     recompile before trusting a RED. The claude arm has no worker forks.)
   - **Send to audit:** `doctrine slice status <id> audit` (bare number), then
     `/audit` from the **parent/root** tree against the prepared `review/<slice>` +
     `phase/*` refs — never from inside the coordination tree (RV verbs refuse on a
     worktree fork, and the tree is now gone).
   - **`review/*` and `phase/*` are EVIDENCE refs, not branches to edit or land
     (R2).** Audit/repair never rewrites them in place. Instead, publish a
     **candidate interaction branch** with `doctrine dispatch candidate create
     --slice <N> --role review_surface|close_target --base <trunk> [--source
     <review/phase ref>] [--worktree]` — a Doctrine no-ff 3-way merge of the
     evidence onto the trunk base, recorded with immutable source/base/merge OIDs.
     `doctrine dispatch candidate status --slice <N>` lists the evidence refs and
     candidate branches separately and prints the safe next verb — route reviewers
     and humans to the **candidate** branch for review/fix, never the raw evidence
     ref. The create→audit→admit→close path: create a candidate → audit it →
     `doctrine dispatch candidate admit --slice <N> --role close_target --candidate
     <ref> [--review RV-NNN]` pins an immutable `admitted_oid` (validates provenance
     + merge-ancestry, refuses a moved ref) → `/close` integrates that OID.

   **Stage-2 integrate is NOT yours and NOT now.** Conclude stops here. Only after
   audit passes does **`/close`** (not `/dispatch`) run `doctrine dispatch sync
   --slice <N> --integrate [--trunk <ref>] [--edge <ref>]` — the post-audit replay
   that projects the audited units. When a candidate workflow is active, integrate
   targets the **admitted `close_target` OID** (and `--edge` the admitted
   `review_surface` OID) via a fast-forward-only CAS row — never a close-time merge;
   a moved target refuses (re-admit a superseding candidate on the new base), and a
   missing admission refuses rather than falling back to a raw ref. Integrating
   pre-audit is the gate this lifecycle exists to enforce.
6. **Hand over on cadence — a quality gate, not an overflow stop.** Reasoning
   quality decays long before any capacity limit, so hand over *early*, while
   sharp. You cannot read your own token count in-loop, so trip on what you **can**
   count, at the next **committed** boundary, whichever first:
   - **`handover_after` batches** since spawn (default `5`), **or**
   - **`handover_delta` cumulative reviewed-delta lines** since spawn (default
     `2000`) — sum each `B..S`.

   Then stop at the boundary and `/handover`; a fresh orchestrator resumes from the
   branch. **Never carry a half-imported batch across a handover.** Tune via
   `/dispatch handover_after=N handover_delta=L`.

## Context hygiene — keep the orchestrator lean

A bounded per-batch footprint is what makes the handover cadence meaningful:

- **Worker reports enter structured and size-capped** — a verdict plus a short
  summary, never raw build/test logs. Mandate the shape in the spawn prompt.
- **Read deltas `--stat` / `--name-only` first.** Pull a full `B..S` body only when
  a check needs it; the R-5 belt and disjointness checks run on name-only.
- **Verify output enters as pass/fail + a short tail**, never the full log.

## Remit — orchestrator is the sole writer (D6a)

| Party | Branch | Worker-mode | Writes |
|---|---|---|---|
| **Orchestrator (you)** | coordination | **OFF** | every doctrine-mediated write: the import commit, memory, AC evidence, notes, status |
| **Worker** | its fork | **ON** (self-armed / hook-stamped) | **source only**, committed as one non-merge `S` to the fork branch — never doctrine state |

The fork withholds the coordination/runtime tier by construction (provision
exclusion, D9). A worker returns a **source delta + a structured report**; you alone
advance the coordination branch.

## Pre-distilled worker prompt (D6 — self-contained, no governance read)

Workers **do not** read boot/governance or run `/route`/`/boot`. You pre-distill
everything into the spawn prompt (the arm carries it to the worker):

- **policy digest** — the rules of the road that bear on the task (lifted from your
  loaded governance, not re-derived by the worker);
- **design excerpts** — the relevant design/contract slices;
- **pre-fetched memories** — the scope-bound gotchas you already retrieved;
- **task spec + declared file set** — what to change and exactly which files (the
  file set is load-bearing for disjoint batching);
- **mandatory verify command** — the project's green-gate (doctrine is a framework;
  never assume `just check` — pass the project's command explicitly);
- **the self-arm mandate** — `export DOCTRINE_WORKER=1` (fails open — see below);
- **the escalation contract** — on an architectural fork, or a task it cannot
  complete cleanly within its declared files, the worker **stops and reports**. It
  has no governance read, so the decision comes **up to you** — you `/consult` it.

**`DOCTRINE_WORKER=1` fails OPEN (C-I).** The enforceable protection is the
import-time R-5 belt, which you run on the trusted side — not the self-armed var.

## Batching — serial by default, file-disjoint to parallelize (C-III)

The default unit is **one phase, one worker** — a batch of one. **Widen** a batch to
run workers concurrently only when their declared changed-path sets are **pairwise
disjoint.** Dependency-disjoint is **not** enough: two independent tasks routinely
edit the same file. Shared file ⇒ separate serial batches, never one concurrent
batch. The funnel is identical for a batch of one.

## The funnel — strict per-batch order (D7)

The cadence is **the batch, not the worker** (a per-worker commit moves HEAD,
landing the next delta on a moved base). Capture `B = git rev-parse HEAD` pre-spawn
(tree clean). After the batch's workers return, run **in this exact order** — the
git mechanics are the shipped `import` verb (see [worktree skill](../worktree/SKILL.md)):

1. **precond (X-1).** `import` asserts the coordination **worktree AND index are
   clean** and `HEAD == B` (`tree-unclean` / `head-moved` tokens). Not clean ⇒
   **abort**.
2. **delta (X-2).** Each worker's delta is the **net diff `B..S`**, `S` the one
   non-merge commit on the fork branch. `import` asserts `S^ == B` (the immediate
   parent *is* `B`) — the trusted-side belt against a divergent-base fork; and a
   single non-merge commit (multi-commit / merge / rebase ⇒ `multi-commit` reject).
   A net diff, **not** a `cherry-pick`/replay.
3. **R-5 belt — reject authored-tree touches (C-II).** `import` rejects any path
   under `.doctrine/` **or `.claude/`** (`doctrine-touch` / `claude-touch`). This
   protects the trunk-minting guarantee from an unarmed worker; sound because **you**
   run it (worker-mode OFF, mechanically checkable) where the env contract is not.
4. **import — non-committing.** `import` applies every surviving net-diff onto `B`,
   NON-committing. An apply **conflict** on a file-disjoint batch means the
   changed-path analysis was wrong (or a worker strayed) ⇒ **report + halt**, human
   re-plans. Never auto-resolve.
5. **verify — combined tree.** Run the project verify command on the combined tree.
   On **RED**, re-run verify against **each delta alone** to **name the offending
   worker** (X-3 — file-disjoint removes git conflicts, not semantic coupling) ⇒
   report + halt.
6. **branch-point guard (D5).** `doctrine worktree branch-point-check --base B` —
   coordination HEAD still `B`? A mismatch means an **external** mover ⇒
   **re-dispatch from the new HEAD**, never commit against a moved base.
7. **commit — one batch commit.** ONE commit on the coordination branch ⇒ `B+1`.
7a. **record the code boundary (claude arm).** Capture the code tip *before* the
   knowledge commit and run `doctrine dispatch record-boundary --slice <N> --phase
   PHASE-NN --code-start <B> --code-end <B+1>`. This is the input stage-1
   `prepare-review` tree-reads to **cut `phase/<slice>-NN`** on the fork-less claude
   arm (design §4.3). On pi and codex the worker fork branch **is** the native phase
   deliverable — skip this step (see [`/dispatch-subprocess`](../dispatch-subprocess/SKILL.md)).
8. **record — knowledge trails the commit.** Memory / AC evidence / notes, *after*
   the confirmed commit (and after 7a, so `boundaries.toml` rides the knowledge
   commit onto `dispatch/<slice>`).

The next batch forks from `B+1`. **Report-and-halt, never auto-merge** — conflict,
moved HEAD, or a `.doctrine/`/`.claude/`-touch all stop the funnel and surface to a
human (ADR-006: policy is report, never auto-resolve).

## Crash / overflow recovery

No orchestrator state is load-bearing. Rebuild from the **coordination branch** +
`git worktree list`: committed batches are durable on the branch; in-flight forks
are re-imported (their `B..S` still applies) or re-dispatched. Context overflow is
just a crash — recover the same way.

## Out of scope (v1)

- **Remote / non-shared-store workers (C-VI).** The no-transport import assumes the
  rung-3 fork is a local `git worktree` (shared `.git`). A remote agent would hand
  back a `git format-patch` series applied through the **same** cadence — noted, not
  specified. v1 assumes the shared object store.
- **A routing slot.** `/dispatch` is not yet a `/route` destination.
- **Parallel landing (υ).** v1 funnels **one landing per base** — concurrent
  EXECUTION is first-class, but the orchestrator's sequential imports bump HEAD, so
  siblings re-dispatch onto the bumped base. Per-batch re-anchor (IMP-043) is
  **demoted, not on the hot path**: branch-point movement is now handled once, at
  **sync time**, as target-movement under the CAS journal (moved target ⇒ report,
  never auto-resolve) — not re-anchored per batch.

## Quick Reference

| Situation | Action |
|---|---|
| No phase parallelizes | **Serial — one worker per phase, batch of one, same funnel.** Never bail to inline |
| Set up the run | `doctrine worktree coordinate --slice <N> --dir <path>` — funnel on the dedicated `dispatch/<slice>` worktree, never session `main` |
| Drive the slice | Loop: `/phase-plan` → route+spawn → funnel → repeat from new HEAD until done |
| Pick the arm | pi → `/dispatch-subprocess` (pi row); codex → `/dispatch-subprocess` (codex row); claude → `/dispatch-agent`; route only on self-belief↔env agreement |
| Harness mismatch / unknown | **Refuse, NAMING the cause** — never a blind spawn |
| Handover cadence | Early, at a **committed** boundary, on `handover_after` (5) **or** `handover_delta` `B..S` lines (2000), whichever first |
| Worker reports a fork / can't finish clean | It halted by contract → **you `/consult`**; never auto-adapt plan or design |
| Phase can't be delegated (spec / authoring) | Execute inline yourself, then resume the loop |
| Two tasks share a file | **Separate serial batches** — file-disjoint required to parallelize |
| Batch returned | `import` (precond → `S^==B` → R-5 reject → apply non-committing) → verify → branch-point → one commit → **record-boundary (claude arm)** → record knowledge |
| Delta touches `.doctrine/` / `.claude/` | **Report + halt** (R-5 belt — the real protection; `DOCTRINE_WORKER=1` fails open) |
| Worker fork `>1` / merge / rebased commit | **Reject** before import (the unit is net diff `B..S`) |
| Combined verify RED | Re-verify each delta alone to NAME the offender → report + halt |
| `branch-point-check` exits 1 | External HEAD move → **re-dispatch**, never commit on a moved base |
| Crash / context overflow | Rebuild from coordination branch + `git worktree list`; no load-bearing state |
| All phases landed (conclude) | `dispatch sync --prepare-review` → remove coordination worktree (KEEP `dispatch`/`review`/`phase` refs) → `slice status <id> audit` → `/audit` from parent/root. **Never** `--integrate` — that is `/close`, post-audit |
| Codex/pi worker forks spent | `doctrine worktree gc --fork <branch>` each (claude arm has none) |
| Review/repair the evidence | **Never edit `review/*`/`phase/*` in place (R2)** — `dispatch candidate create` a candidate interaction branch; `candidate status` routes to it; `candidate admit` pins the immutable OID `/close` integrates |

## Red Flags

**Never:**
- Spawn without routing — a blind spawn on an unconfirmed harness. Route on
  agreement (three-way: pi/codex/claude); refuse naming the cause on mismatch.
- Restate an arm's spawn template here — link to `/dispatch-subprocess` /
  `/dispatch-agent`.
- Let a worker write `.doctrine/`/`.claude/` authored trees, or import a delta that
  touches them (the R-5 belt is non-droppable — it, not `DOCTRINE_WORKER=1`, is the
  real protection).
- Commit per worker (HEAD moves) — commit **per batch**, once.
- Replay fork history (`cherry-pick`) instead of the **net diff `B..S`**.
- Auto-merge or auto-resolve a conflict / moved HEAD / authored-tree touch —
  **report and halt**.
- Auto-adapt the plan or design to keep the drive moving — an emergent architectural
  decision is the **semantic** report-and-halt: `/consult` it, never decide solo.
- Drive the funnel on the session `main` tree — always the dedicated
  `dispatch/<slice>` coordination worktree (`worktree coordinate`).
- Integrate at conclude. Conclude is **stage-1 `--prepare-review` only**; stage-2
  `--integrate` is `/close`'s job, **post-audit**. Never land code pre-audit.
- Delete the `dispatch`/`review`/`phase` refs at conclude — only the worktree
  *directory* is removed; the refs are the preserved deliverables.
- Record knowledge before the confirmed commit.
- Bail to serial **inline** execution because no phase parallelizes — serial still
  means spawn a worker in a worktree (batch of one).
- Author or edit this skill in `.doctrine/skills/` (the gitignored install copy);
  the source is here under `plugins/`.

**Always:**
- Route to the correct arm on a confirmed harness; default to one worker per phase,
  parallelizing only file-disjoint phases; `/consult` emergent architectural forks.
- Keep your own context lean (capped reports, stat-first diffs, verify tails).
- Run as worker-mode OFF, the sole doctrine-mediated writer, on the coordination
  branch.
- Pre-distill a self-contained prompt; workers never read boot/governance.
- Hold the strict funnel order; the R-5 belt and the branch-point guard are
  mandatory; make knowledge trail the confirmed commit.

## Outcome

Driven phase by phase, the slice reaches completion unattended. Each batch — usually
a single serial worker, occasionally a file-disjoint concurrent set — lands as
exactly one commit on the dedicated `dispatch/<slice>` coordination worktree, every
imported delta policy-checked and verified before it lands, with conflicts surfaced
to a human rather than merged. Conclude projects the reviewable refs
(`review/<slice>` + `phase/<slice>-NN`) and stops for audit; integration to trunk is
`/close`'s post-audit act. The reviewable refs — not the session `main` tree — are
the deliverable.
