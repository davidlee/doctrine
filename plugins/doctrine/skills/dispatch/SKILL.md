---
name: dispatch
description: Use to drive a slice's phases to completion through sub-agent workers in isolated worktrees — you orchestrate and are the sole writer, the workers execute. Routes to `/dispatch-subprocess` (codex/pi) or `/dispatch-agent` (claude); the funnel cadence (import → verify → branch-point → one commit → record) is identical on both arms. Default serial (one worker per phase); parallelize file-disjoint phases. Conflicts report-and-halt, never auto-merge.
---
# Dispatch (router)
Drive a slice's phases to completion through sub-agent **workers** — you are the
orchestrator and **sole writer**, they execute.

**Announce at start:** "Using the dispatch skill to run workers under the
orchestrator funnel."

## The outer loop
1. `dispatch setup --slice <N> --dir <path>` — create/resume coordination worktree
2. `dispatch plan-next --slice <N>` — find next actionable phase(s); plan parallel batches when file-disjoint
3. Route to the correct arm, spawn worker(s) — [`/dispatch-subprocess`](../dispatch-subprocess/SKILL.md) for codex/pi, [`/dispatch-agent`](../dispatch-agent/SKILL.md) for claude
4. Funnel the batch (import → verify → branch-point → one commit → record)
5. Repeat from new HEAD until all phases done
6. Conclude: `dispatch sync --prepare-review` → remove coord worktree → audit

## The funnel (per batch)

Capture `B = git rev-parse HEAD` pre-spawn. After workers return, in exact order:
1. Precond — worktree/index clean, HEAD == B
2. Delta check — net diff `B..S`, single non-merge commit, `S^ == B`
3. R-5 belt — reject any `.doctrine/` or `.claude/` touch
4. Import — apply surviving net-diffs onto `B`, non-committing
5. Verify — run project verify; if RED, isolate offender per delta
6. Branch-point guard — coordination HEAD still `B`?
7. Commit — ONE commit on coordination branch
8. Record — knowledge trails the confirmed commit
**Report-and-halt** on conflict, moved HEAD, or authored-tree touch — never auto-resolve.

## Handover cadence
Hand over at a committed boundary: after `handover_after` batches (default 5) or
`handover_delta` cumulative reviewed-delta lines (default 2000), whichever first.

## Conclude
When all phases land: `dispatch sync --prepare-review` → remove coordination worktree
directory (KEEP the refs) → `slice status <id> audit` → `/audit` from parent/root.
Stage-2 integrate is `/close`'s job, post-audit — never land code pre-audit.

## Red Flags

IMPORTANT: READ VERY CLOSELY

**Never:** spawn without routing; let a worker write `.doctrine/`/`.claude/`;
commit per worker; replay fork history; auto-merge conflicts; auto-adapt plan/
design (`/consult` forks); drive on session `main`; integrate at conclude; delete
deliverable refs; bail to inline execution.

NEVER bail to inline execution - if you are about to `/execute`, STOP.
NEVER use git like a drunk with a chainsaw - if you are about to do something potentially risky, STOP.

**Always:** route to correct arm on confirmed harness; keep context lean
(capped reports, stat-first diffs); hold strict funnel order; pre-distill
self-contained worker prompts; trail knowledge after the confirmed commit.
