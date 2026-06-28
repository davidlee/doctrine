---
name: dispatch
description: Use to drive a slice's phases to completion through sub-agent workers in isolated worktrees — you orchestrate and are the sole writer, the workers execute. Routes to `/dispatch-subprocess` (codex/pi) or `/dispatch-agent` (claude); overridable via `[dispatch] claude-force-subprocess-dispatch` in `doctrine.toml`. The funnel cadence (import → verify → branch-point → one commit → record) is identical on both arms. Default serial (one worker per phase); parallelize file-disjoint phases. Conflicts report-and-halt, never auto-merge.
---
# Dispatch (router)
Drive a slice's phases to completion through sub-agent **workers** — you are the
orchestrator and **sole writer**, they execute.

**Announce at start:** "Using the dispatch skill to run workers under the
orchestrator funnel."

## The outer loop
1. `dispatch setup --slice <N> --dir <path>` — create/resume coordination worktree.
   On the claude arm `--dir` MUST resolve inside the project root (convention
   `.dispatch/SL-<n>`); an outside-root dir fails closed (ISS-031 — the pre-spawn
   `cd` silently reverts under a jail, forking `main` not B).
2. **Claude arm only:** `cd` into the coordination directory and park Bash cwd
   there for the full drive loop. The Agent tool's `isolation: worktree` forks off
   the Bash cwd HEAD — base==B holds only if the cwd is parked at B before the
   spawn (a mis-parked cwd forks the wrong base). Step out to the session root
   only for authored writes (slice status, memory, audit).
3. `dispatch plan-next --slice <N>` — find next actionable phase(s); plan parallel batches when file-disjoint
4. Route to the correct arm:
   - Check `doctrine.toml` → `[dispatch]` → `claude-force-subprocess-dispatch`
     (default `false` if the file or key is absent).
   - If `true`, route workers via [`/dispatch-subprocess`](../dispatch-subprocess/SKILL.md)
     (default to `pi` arm until `preferred-subprocess-harness` selection is wired — IMP-101).
   - Otherwise, route per env-marker: `.claude/` present →
     [`/dispatch-agent`](../dispatch-agent/SKILL.md); otherwise →
     [`/dispatch-subprocess`](../dispatch-subprocess/SKILL.md).
   Then spawn worker(s) per the chosen arm's template.
5. Funnel the batch (import → verify → branch-point → one commit → record)
6. Repeat from new HEAD until all phases done
7. Conclude: `slice verify-vt <id>` (VT gate, coord tree) → on green
   `dispatch sync --prepare-review` → remove coord worktree → audit

## The funnel (per batch)

Capture `B = git rev-parse HEAD` pre-spawn, then capture the S1 regression
baseline on the coord tree at `B`:
```
doctrine check regression capture --base "$B"     # suite @ B; no-op on cache hit
```
**INV-1 — normalise filter state before BOTH this capture and the verify diff**:
clear the worker marker (`doctrine worktree marker --clear --operator`) and force a
real rebuild, so capture and diff run an identical suite invocation + test
selection. Same tree alone is insufficient — a leaked `DOCTRINE_WORKER`/marker
changes which tests run and breaks the cancellation property (a fingerprint
mismatch is then a cache miss → honest re-capture, never a poisoned baseline).

After workers return, in exact order:
1. Precond — worktree/index clean, HEAD == B
2. Delta check — net diff `B..S`, single non-merge commit, `S^ == B`
3. R-5 belt — reject any `.doctrine/` or `.claude/` touch
4. Import — apply surviving net-diffs onto `B`, non-committing
5. Verify — `doctrine check regression diff --base "$B"` (suite @ S, SAME
   normalised filter state as the capture). Exits non-zero on `new ∪ changed` (a
   slice regression regardless of which test binary/env it surfaces under) OR an
   unobtainable run (a compile error / panic / format change is a hard halt, never
   a silent green ∅). `persistent` (same key + same signature at B and S) is the
   tolerated env artifact; a non-empty baseline is surfaced as a trunk warning to
   fix, not laundered as "env". On halt, the named `new`/`changed` keys ARE the
   offenders — no by-hand isolation. (Carry-forward of the green current-set as
   `baseline-<B'>` is a deferred cost optimisation; steady state still re-captures.)
6. Branch-point guard — coordination HEAD still `B`?
7. Commit — ONE commit on coordination branch
8. Record — knowledge trails the confirmed commit, **and the per-phase `B→B+1`
   boundary lands in the primary-tree conformance registry** (F-5 resolves it from
   the coord tree; F-6 guard; upsert by phase) — by arm:
   - **claude** — `dispatch record-boundary` already double-writes it (+ the
     `phase/<N>` ref-cut); no separate call (`/dispatch-agent`).
   - **codex/pi** — `doctrine slice record-delta <SL> PHASE-NN --start <B> --end
     <B+1>` — the arm's registry write (symmetric derive deferred, D6/IMP-171; no
     `record-boundary` on this arm; `/dispatch-subprocess`).
   Neither is a "remember to also record" hand-step any more: the Conclude beat's
   completeness gate halts if a landed phase is missing its row (below).
**Report-and-halt** on conflict, moved HEAD, or authored-tree touch — never auto-resolve.

## Handover cadence
Hand over at a committed boundary: after `handover_after` batches (default 5) or
`handover_delta` cumulative reviewed-delta lines (default 2000), whichever first.

## Base freshness (mid-drive)
A long drive lets trunk advance under the coordination branch; the drift stays
invisible until `dispatch sync`/candidate-create conflicts on a merge-base
divergence — the most disruptive place to discover it. `dispatch status` surfaces
it (`trunk: moved (N commit(s) ahead of fork-point)`). When status shows movement,
run `dispatch refresh-base --slice <N>` — it merges current trunk into
`dispatch/<N>` in the live coordination worktree, advancing the base early and in
context so each conflict is one phase's delta. Conflicts there report-and-halt for
manual resolve in the coord tree — never auto-merged.

## Conclude
When all phases land, run the conclude cadence **in the coord tree, before it is
removed** (SL-170 S3/S6): `slice verify-vt <id>` → on green
`dispatch sync --prepare-review` → remove coordination worktree directory (KEEP
the refs) → `slice status <id> audit` → `/audit` from parent/root. Stage-2
integrate is `/close`'s job, post-audit — never land code pre-audit.

`slice verify-vt <id>` is the **VT existence/shape gate** (S3): it reads the coord
tree's `plan.toml` and checks every `VT`-mode criterion's mandated `test_file` +
`keywords`. A `Fail` exits non-zero and **HALTS handover** — do not prepare-review
past it; `/consult` → revise-or-waive the authored plan (never self-relax a
mandate), then re-run. `Uncheckable` / `Waived` are visible but non-halting. The
fs reader suffices here because the orchestrator (sole writer) has committed any
mid-dispatch waiver onto `dispatch/<slice>`, so the coord working tree == the
committed graph `prepare-review` projects (INV-6).

**Embed at handover (S6):** carry the `verify-vt` VT summary block **and** a
one-line S1 regression status (lifted from the verify beat's `check regression
diff`) into the conclude output and the `/handover` packet — so a gap (incl.
`UNCHECKABLE` / `WAIVED`, rendered distinctly) is visible at handover, not at audit.

`prepare-review` is the **enforced** conformance beat (ISS-052): before projecting
refs it commits the boundaries ledger, **derives** the registry from that committed
ledger on the claude arm (auto-heals a lost funnel row), then runs a completeness
**gate** that `bail!`s if any completed phase lacks a registry row — both arms. So
the registry is guaranteed complete by audit; a gap halts here (no refs created),
the operator commits the ledger / `record-delta`s the gap, and re-runs.

## Red Flags
IMPORTANT: READ VERY CLOSELY

**Never:** spawn without routing; let a worker write `.doctrine/`/`.claude/`;
commit per worker; replay fork history; auto-merge conflicts; auto-adapt plan/
design (`/consult` forks); drive on session `main`; integrate at conclude; delete
deliverable refs; bail to inline execution.
! NEVER bail to inline execution - if you are about to `/execute`, STOP.
! NEVER use git like a drunk with a chainsaw - if you are about to do something potentially risky, STOP.
**Always:** route to correct arm on confirmed harness; keep context lean
(capped reports, stat-first diffs); hold strict funnel order; pre-distill
self-contained worker prompts; trail knowledge after the confirmed commit.
