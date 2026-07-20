---
name: dispatch-agent
description: The claude arm of `/dispatch` — spawn a worker via the `Agent` tool using the dispatch-worker subagent type with worktree isolation. Base is explicit — `dispatch arm-spawn --base B` writes the base file, then cd into the spawn dir before the Agent spawn so the WorktreeCreate hook forks at B. Worker self-commits via the gated `worker_commit` MCP tool; the orchestrator lands via `dispatch_import` → `dispatch_conclude_phase` → `dispatch_reap`. Reached only from the `/dispatch` router on a claude↔env-marker agreement; do not invoke directly.
---

# Dispatch — claude arm

Spawn a worker via the `Agent` tool. The harness-identical funnel and drive loop
live in the [`/dispatch` router](../dispatch/SKILL.md) — this skill is the spawn
template plus this arm's landing mechanics.

## Pre-spawn — arm the base, cd into the spawn dir

The worker's worktree is created by doctrine's **WorktreeCreate hook**
(`doctrine worktree create-fork`), not natively by the harness. The hook
discriminates a dispatch worker **positionally**: a spawn is a dispatch worker iff
the Agent payload cwd **is** the arming dir
`<coord>/.doctrine/state/dispatch/spawn/`. The base it forks is **explicit** — the
`base` file in that dir — never cwd HEAD. cwd is the *discriminator*, not the base
source.

Arming does double duty: the hook also **provisions the worker's
`DispatchRecord`** — the registration `worker_commit` resolves the worker's
opaque agent id against. An unarmed spawn is a benign pass-through worktree with
**no record**, so its `worker_commit` refuses `unknown-agent` and it cannot land
a fork commit at all.

**Base-clean beat (before `arm-spawn`).** Assert the base is prove-clean —
`doctrine check prove` (NON-mutating fmt-check + lint) — before arming/spawning,
and on `main` before you branch the fork. A RED base is a BASE defect (operator
format-and-commit / prep worktree), NEVER folded into a worker delta, NEVER
auto-fixed. This is the same pre-spawn beat the funnel documents; run it once per
batch.

**Before every spawn (or parallel batch):**
1. `doctrine dispatch arm-spawn --base <B> [--slice <N>]` — writes
   `<coord>/.doctrine/state/dispatch/spawn/base = <B>` and prints the spawn dir's
   absolute path. Idempotent: re-arming at B′ rewrites `base`.
2. `cd` **into** that spawn dir. This is the arming signal — the hook forks at B
   only when the payload cwd is the spawn dir.
3. Issue the Agent spawn(s) (below). File-disjoint parallel batch: arm once, then
   issue N spawns from the spawn dir — all read the same B; each hook derives its
   own `branch`/`dir` from its own `name`.
4. `cd` **back to the coord root** after the spawn(s) — positional disarm. A
   benign `isolation:worktree` spawn issued from the coord root passes through
   (provisioned, not worker-forked); only a spawn from inside the spawn dir is a
   dispatch worker. A lingering `base` file is inert — the trigger is cwd-position,
   not file-presence.

**Serial drive:** re-arm each phase — arm at B, cd in, spawn, cd back; the funnel
commit advances coord HEAD; arm at B′ for the next phase. Base is explicit, so
coord-HEAD drift between arm and create is irrelevant. Default cwd is the coord
root; step into the spawn dir only to issue worker spawns.

**Placement precondition (ISS-031):** the coordination worktree (and so its arming
dir) MUST live inside the project root — convention `.dispatch/SL-<n>` (`dispatch
setup --dir .dispatch/SL-<n>`). Under a cwd-confining jail (bubblewrap rooted at
`/workspace/<repo>`), a `cd` to an outside sibling silently reverts to the project
root on the next Bash call. `dispatch setup` fails closed on the claude arm when
`--dir` resolves outside the root. Confirm placement with a bare
`cd <spawn-dir> ; pwd` in a separate Bash call before trusting it.

## Spawn

BASE GUARD — run FIRST, before any read/edit/commit. STOP and write nothing if any check fails:
  1. git status --porcelain                         → MUST be empty (clean tree)
  2. git rev-parse --git-dir vs --git-common-dir    → MUST differ (isolated linked worktree, not main tree)
  3. git merge-base --is-ancestor <B> HEAD          → MUST exit 0 (HEAD descends from base <B>)
  4. grep prerequisite seams: <seams>               → MUST be present
On any failure: STOP, author/commit nothing, report "base-guard-failed: <check>".

Check #2 is the in-worker mirror of the orchestrator's `not-isolated` belt —
if the worker's git-dir equals git-common-dir, it is not an isolated worktree.

```
subagent_type: dispatch-worker
isolation: worktree
prompt: <pre-distilled worker prompt, including the base-guard block above>
```

**The prompt MUST instruct the self-commit:** finish by calling the
`worker_commit` MCP tool with the worker's own **worktree NAME** (never a path)
plus the commit message, and report the returned oid (`fork_tip`) in the
structured hand-back. Raw `git commit` fails in the jail (ro `.git`) —
`worker_commit` is the worker's only commit path. On a `Refused` outcome the
worker reports the reason verbatim and stops; it never retries around a
`forbidden-zone`/`commit-gate-red` refusal.

## Post-spawn funnel — worker self-commit, orchestrator lands

**Primary (B): the worker's delta arrives as ONE committed non-merge commit `C`
(`C^ == B`) on its own `dispatch/<agent>` branch**, landed server-side by the
gated `worker_commit` tool (belts are the security boundary: non-empty delta →
forbidden-zone scope → `HEAD == B` → the `check commit` gate). The delta is
therefore **fork-durable** — it survives the worktree, and a revive/fixup resumes
from the committed tip. In order:

1. **Footer.** Read the Agent return footer for `worktreePath:`.
   NO footer / no `worktreePath:` ⇒ no isolated tree was created (hook abort or
   fallback-to-main) ⇒ ABORT, do NOT enter the funnel. Re-dispatch, or switch to
   the subprocess arm if the hook is failing.
2. **Identity.** Derive from `worktreePath` (the normative datum, live-proven):
   `name = basename(worktreePath)`, `branch = dispatch/<name>`. Do NOT read the
   footer's `worktreeBranch` field — it is `undefined` for the hook-created tree.
   Cross-check the worker's reported `fork_tip` against `git rev-parse <branch>`.
3. **Verify.** `doctrine worktree verify-worker --base <B> --dir <worktreePath> --branch <branch>`
   Abort on any refusal: no-worker-head / not-isolated / unstamped / wrong-base /
   branch-mismatch. It accepts the post-commit HEAD (tests
   `merge-base --is-ancestor B HEAD`, a descendant — not `HEAD == B`); `--branch`
   binds dir↔branch so both belts verify ONE worker state.
4. **Land.** `dispatch_import{slice, name: <branch>}` (MCP) — resolves the coord
   tree server-side by slice, runs the `classify_import` scope belt as a HARD
   pre-compose gate (`.doctrine/`/`.claude/` reject, undeclared-scope reject —
   nothing lands on a refusal), composes coord-tip ⊕ worker-tip via `merge-tree`
   (object-db only, working-tree-free), and lands **ONE non-merge commit**
   preserving the worker AUTHOR. Import and commit are folded — no separate
   orchestrator commit step. Any `Refused{reason}` ⇒ report-and-halt
   (`merge-conflict`, `head-moved`, `undeclared-scope`, … are never
   auto-resolved). The delta is already commit-gate-green (worker_commit ran the
   gate), so no separate post-import prove run on this path.
5. **Conclude.** `dispatch_conclude_phase{slice, phase, code_start: <B>,
   code_end: <coord_tip from step 4>}` — one call, two tiers: flips the
   gitignored phase sheet to `completed` AND lands the committed `(B, coord_tip)`
   boundary row (UPSERT-by-phase, working-tree-free). The flip reaches BOTH the
   coord sheet (next-ready authority) and the PRIMARY sheet (the tree
   `prepare-review`'s completeness gate reads its completed-set from) — no manual
   `slice phase --status completed -p <primary>` re-flip is needed at conclude
   (IMP-272). Idempotent on retry; the only fault outcome is a flipped sheet with
   no committed boundary, which a retry re-composes.
6. **Reap.** `dispatch_reap{slice, name: <branch>}` — runs the patch-id
   landed-oracle (`git cherry`): it REFUSES a fork whose patch is not in coord
   history, then removes the landed fork's worktree + branch. Reap ONLY through
   this oracle — never a raw `git worktree remove --force` against an unimported
   tree (that deletes the sole copy of an unlanded delta; the tool's refusal is
   the guard a raw remove lacks).

## Fallback (A) — live-worktree import

The in-a-pinch path when the primary cannot run: the MCP server is down, or the
worker was never provisioned (`worker_commit` → `unknown-agent`) so no fork
commit exists. The worker's uncommitted delta still lives in the **worktree**,
which **persists** after the Agent returns (no `WorktreeRemove` hook — the
harness does not auto-reap). NOT a bypass: a `forbidden-zone` /
`commit-gate-red` / `undeclared-scope` refusal is a worker-delta defect — fix up
or halt, never import around it.

1. `verify-worker` as above (step 3; HEAD == B in this mode — no commit).
2. `doctrine worktree import --base <B> --from-worktree <worktreePath> --slice <N>`
   — gathers the live tracked+untracked delta, runs the same `classify_import`
   belt, applies onto `B` **NON-committing**, then runs the reject-and-halt prove
   gate in-process (an unformatted/lint-red delta HALTS staged, never auto-fixed).
3. Commit ONE on the coordination branch yourself, then record the boundary:
   `doctrine dispatch record-boundary --slice <N> --phase PHASE-NN --code-start <B> --code-end <B+1>`.
4. Reap gated on the landed delta — on this path the tree is the sole copy until
   the coord commit lands; `&&`-chain the removal behind it.

This is also the **pi/subprocess arm's only mode** — a bwrap-confined worker
cannot commit at all (`/dispatch-subprocess`). Never instruct or expect
self-commit off the claude arm.

## Worker confinement (SL-182)
The claude worker is confined by two installed **PreToolUse** hooks (plugin
`hooks.json` → `doctrine worktree pretooluse`): **Bash** is opaquely rewritten
into a nested bwrap jail (rw the worktree, ro everything else); **Edit|Write**
outside the worktree is denied. The orchestrator (no `agent_id`) passes through.
Confinement is by construction — no cooperative worker flag. `worker_commit` is
the deliberate, single-purpose bypass of that wall: the *unconfined server*
commits on the worker's behalf, and the tool's belts — not the wall — are the
security boundary there.

- **No hot-reload.** Plugin `hooks/` changes load at session start only
  (`docs/claude/plugins-reference.md:394`). After `doctrine install` (or any
  hooks edit), pick them up with **`/reload-plugins`** (lighter) or a session
  restart — otherwise the OLD hooks (or none) are live.
- **Fail-closed install.** The installed command is the resolved **absolute**
  exec with a `|| exit 2` guard, so a missing/stale binary **denies** rather than
  running the tool unconfined (PreToolUse otherwise fails open — only exit 2
  blocks).
- **Escape hatch.** A broken Bash wrapper can lock you out of Bash, but Edit/Write
  are not Bash-gated: disable the offending hook via Edit, then `/reload-plugins`.

## Red Flags
**Never:** spawn without first `arm-spawn`-ing and cd'ing INTO the spawn dir (a
spawn from the coord root is a benign pass-through, never a dispatch worker —
and its `worker_commit` refuses `unknown-agent`); read the footer's
`worktreeBranch` (undefined for the hook-created tree — derive
`branch = dispatch/basename(worktreePath)` instead); funnel a worker that
returned no `worktreePath:` footer; import the live tree when a fork commit
exists (the delta is the commit — land it via `dispatch_import`); use fallback
(A) to route around a `worker_commit`/`dispatch_import` refusal (a refusal is a
defect or a halt, never a detour); raw `git worktree remove --force` an
unlanded tree (reap through `dispatch_reap`'s patch-id oracle); claim
self-commit on the pi arm; point `dispatch setup --dir` at an outside-root
sibling (ISS-031); spawn with a `subagent_type` other than `dispatch-worker`;
run `fork` or bwrap here (that's `/dispatch-subprocess`); claim parallel landing
(v1 lands one per base).
**Always:** `arm-spawn --base B` then cd into the spawn dir before the spawn, cd
back to the coord root after; pin `subagent_type` to `dispatch-worker`; embed
the base-guard block AND the `worker_commit` self-commit instruction in the
distilled worker prompt; derive `branch` from `worktreePath`; run `verify-worker`
before landing; land `dispatch_import` → conclude `dispatch_conclude_phase`
(`code_start = B`, `code_end = coord_tip`) → reap `dispatch_reap`, in that
order; return to the router for the drive cadence.
