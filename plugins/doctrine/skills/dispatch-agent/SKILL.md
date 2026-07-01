---
name: dispatch-agent
description: The claude arm of `/dispatch` — spawn a worker via the `Agent` tool using the dispatch-worker subagent type with worktree isolation. Base is explicit — `dispatch arm-spawn --base B` writes the base file, then cd into the spawn dir before the Agent spawn so the WorktreeCreate hook forks at B. Reached only from the `/dispatch` router on a claude↔env-marker agreement; do not invoke directly.
---

# Dispatch — claude arm

Spawn a worker via the `Agent` tool. The harness-identical funnel and drive loop
live in the [`/dispatch` router](../dispatch/SKILL.md) — this skill is only the
spawn template.

## Pre-spawn — arm the base, cd into the spawn dir

The worker's worktree is created by doctrine's **WorktreeCreate hook**
(`doctrine worktree create-fork`), not natively by the harness. The hook
discriminates a dispatch worker **positionally**: a spawn is a dispatch worker iff
the Agent payload cwd **is** the arming dir
`<coord>/.doctrine/state/dispatch/spawn/`. The base it forks is **explicit** — the
`base` file in that dir — never cwd HEAD. cwd is the *discriminator*, not the base
source.

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

## Post-spawn funnel (claude arm) — symmetric live-import

The worker's ro-`.git` blocks its self-commit, so its delta never rides a fork
commit — it lives in the **worktree**, which **persists** on disk after the Agent
returns (doctrine ships `create-fork` as the `WorktreeCreate` hook and **no**
`WorktreeRemove` hook, so the harness does NOT auto-reap; `docs/claude/hooks.md:2442`).
The orchestrator imports that live tree directly. Five steps, in order:

1. **Footer.** Read the Agent return footer for `worktreePath:`.
   NO footer / no `worktreePath:` ⇒ no isolated tree was created (hook abort or
   fallback-to-main) ⇒ ABORT, do NOT enter the funnel. Re-dispatch, or switch to
   the subprocess arm if the hook is failing.
2. **Identity.** Derive from `worktreePath` (the normative datum, live-proven):
   `name = basename(worktreePath)`, `branch = dispatch/<name>`. Do NOT read the
   footer's `worktreeBranch` field — it is `undefined` for the hook-created tree
   (PHASE-04 VA-1, live 2.1.181).
3. **Verify.** `doctrine worktree verify-worker --base <B> --dir <worktreePath> --branch <derived branch>`
   Abort on any refusal: no-worker-head / not-isolated / unstamped / wrong-base / branch-mismatch.
   (`--branch` binds dir↔branch — both belts verify ONE worker state. The
   `no-worker-head` refusal is ALSO the runtime catch if the tree ever went missing —
   the second boundary behind the install-time no-`WorktreeRemove` assert, RV-205 F-2.)
4. **Import.** `doctrine worktree import --base <B> --from-worktree <worktreePath>`
   Gathers the live tracked+untracked delta, runs the `classify_import` belt
   (`.doctrine/`/`.claude/` reject, HEAD==B, clean coord tree), applies onto `B`
   NON-committing. This realizes the router funnel's arm-neutral **Import** beat on
   this arm (the `B..S` single-commit check reads vacuously — a worktree carries no
   commits). A belt/precond violation exits **nonzero** → the funnel HALTS here.
5. **Reap — GATED on step 4 exit 0 (F-3).** ONLY after `import` succeeded (and the
   batch's commit + `record-boundary` have landed) reap the tree:
   ```
   doctrine worktree import --base "$B" --from-worktree "$WT" && \
     <commit + record-boundary> && \
     git worktree remove --force "$WT"
   ```
   The `--force` is required (the tree is intentionally dirty). **On import failure
   the funnel HALTS and LEAVES the tree on disk** for diagnosis — never `--force`-reap
   the sole copy of an unimported delta. A parallel batch reaps each tree
   independently, each gated on its own import.

## Boundary recording
After the batch's code commit and before the knowledge commit:
`doctrine dispatch record-boundary --slice <N> --phase PHASE-NN --code-start <B> --code-end <B+1>`.
Claude-arm-only (no fork branch); skip on codex/pi. **One call double-writes both
registries** (dispatch.rs): the committed `phase/<N>` ref-cut **and** the
primary-tree conformance registry (F-6 guard, upsert by phase). The committed
ledger is also what `dispatch sync --prepare-review` re-derives the registry from
(auto-heal) before the completeness gate runs — so on this arm registry capture is
**enforced machinery**, not a step the orchestrator can forget. The claude arm
therefore needs **no** funnel `slice record-delta` step; `record-delta` survives
on this arm **only** as the manual escape hatch (correct a range / bootstrap a
pre-binding phase).

## Worker confinement (SL-182)
The claude worker is confined by two installed **PreToolUse** hooks (plugin
`hooks.json` → `doctrine worktree pretooluse`): **Bash** is opaquely rewritten
into a nested bwrap jail (rw the worktree, ro everything else); **Edit|Write**
outside the worktree is denied. The orchestrator (no `agent_id`) passes through.
Confinement is by construction — no cooperative worker flag.

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
spawn from the coord root is a benign pass-through, never a dispatch worker); read
the footer's `worktreeBranch` (undefined for the hook-created tree — derive
`branch = dispatch/basename(worktreePath)` instead); funnel a worker that returned
no `worktreePath:` footer; point `dispatch setup --dir` at an outside-root sibling
on the claude arm (the `cd` silently reverts under a jail; use `.dispatch/SL-<n>`);
spawn with a `subagent_type` other than `dispatch-worker`; run `fork` or bwrap here
(that's `/dispatch-subprocess`); claim parallel landing (v1 lands one per base).
**Always:** `arm-spawn --base B` then cd into the spawn dir before the spawn, cd
back to the coord root after; pin `subagent_type` to `dispatch-worker`; embed the
base-guard block in the distilled worker prompt; derive `branch` from
`worktreePath`; run `verify-worker` before `import --from-worktree`; `&&`-gate the
`git worktree remove --force` reap on a clean import exit (a failed import HALTS and
LEAVES the tree — never `--force`-reap an unimported delta); return to the router for
the funnel cadence.

**Never:** import from a captured patch file or a fork commit on this arm (retired —
the delta is the live worktree); reap the worker tree before `import` returns 0.
