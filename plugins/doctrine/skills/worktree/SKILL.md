---
name: worktree
description: Use when work needs isolation from the current workspace — when /execute (mode=solo) or the future /dispatch funnel must run a phase on its own branch without touching the main working tree. Detects existing isolation, creates a git worktree fork with smart directory selection and safety verification, provisions it via `doctrine worktree provision` (the sole copier; coordination tier excluded), runs the commit-before-spawn / branch-point / baseline guards, and hands back the fork branch.
---

# Worktree

Drive the lifecycle of an isolated git worktree: detect existing isolation,
create a fork when needed, **provision it through `doctrine worktree provision`
(the sole copier)**, run the spawn guards, and verify a green baseline before
handing off.

**Announce at start:** "Using the worktree skill to set up an isolated workspace."

**This is a sub-skill.** `/execute` (mode=solo) invokes it for opt-in isolation;
the future `/dispatch` funnel will invoke it for workers. It is not a `/route`
destination — reach it through the caller.

## Mode contract

The skill is parameterised so the funnel slice reuses it without inheriting solo
semantics (design §5, the OQ-1 split seam).

**Inputs:**
- `mode = solo | worker`
- `allow_work_in_place: bool` — **true only for `solo`**
- requested branch / path
- `base` — the ref the fork is created from. **`solo` defaults to `HEAD`;
  `worker` REQUIRES an explicit base** (the orchestrator's coordination HEAD `B`),
  because the session HEAD is **not** `B` — the orchestrator drives from the
  coordination branch while the session repo may sit on `main`. Forking from the
  implicit session HEAD imports a divergent base and corrupts the `B..S` delta.

**Behaviour:**
- `solo` — MAY degrade to the work-in-place rung on sandbox denial (no fork; the
  blessed trunk path).
- `worker` — **MUST NOT degrade.** A worker with no real fork is a hard failure;
  the funnel's isolation is mandatory. It forks from the **supplied `base` (`B`)** via
  rung 3, never the implicit session HEAD — for a worker the session HEAD is *not* `B`
  (the orchestrator drives the coordination branch while the session repo may sit on
  `main`), so an implicit-HEAD fork is a divergent base that breaks `S.parent == B`.
  The harness-native creation rung (rung 2) cannot pin the base and is never depended
  on. Beyond creation/provision/guards, the worker half runs a constrained
  edit→verify→**commit-one-`S`-to-fork** loop; see
  [Worker mode](#worker-mode-the-funnel-half) (SL-031 §5.2).

**Outputs:**
```
{ fork_path, branch, head_sha,
  provision_report { copied, withheld },
  baseline_result }
```

`worker` adds two fields once it has committed its delta:
```
{ fork_branch,        # the branch ref carrying the single commit S (the importable delta)
  head_sha_after }    # the fork HEAD after S — what the orchestrator imports B..S from
```

## Detection (adapt, don't re-create — D1)

If the CWD is already inside a linked worktree, adapt to it; do not create another.

```bash
git rev-parse --git-dir          # differs from common-dir ⇒ linked worktree
git rev-parse --git-common-dir
```

`--git-dir` ≠ `--git-common-dir` ⇒ already in a linked worktree → skip creation.

**Submodule guard.** A submodule *also* trips that inequality. Disambiguate:
- the `.git` gitdir resolves under `…/worktrees/` ⇒ worktree; under `…/modules/`
  ⇒ submodule; **or**
- a non-empty `git rev-parse --show-superproject-working-tree` ⇒ submodule.

A submodule is **not** the isolation we want — treat it as "not yet forked".

## Creation backend ladder (degrade in order — D5/F1)

1. **Existing isolation** (detection above) → skip creation, adopt the fork.
2. **Harness native worktree-*creation*** if present and invocable
   **creation-only** (no auto-copy). Opportunistic only — Claude Code's
   `WorktreeCreate` hook (code.claude.com/docs/en/hooks) is the concrete instance:
   it *replaces* git creation (the hook makes the worktree, returns its path). But
   it is **Claude-Code-specific** — a non-Claude agent (codex, pi, …) has none, and
   a project MAY configure the hook to copy. **Design around it; never depend on
   it** — fall through to rung 3, which works under any agent.
3. **`git worktree add <path> <branch> <base>`** — guaranteed-present, the **blessed
   tested default**. Prefer this rung; it is fully controlled. Pass `<base>`
   explicitly (`HEAD` for solo, `B` for a worker) — the explicit base is *why* this
   rung is controlled where the harness backend is not.
4. **Work-in-place** (`solo` only, `allow_work_in_place=true`) on sandbox denial —
   no fork, the trunk path. `worker` aborts instead.

### Worktree directory selection

For rung 3, pick the fork directory before creating, in priority order
(`superpowers:using-git-worktrees`):

1. **Check existing directories** — reuse one if present; if both, `.worktrees`
   wins.
   ```bash
   ls -d .worktrees 2>/dev/null     # Preferred (hidden)
   ls -d worktrees 2>/dev/null      # Alternative
   ```
2. **Check CLAUDE.md** for a stated preference — use it without asking.
   ```bash
   grep -i "worktree.*director" CLAUDE.md 2>/dev/null
   ```
3. **Ask the user** if no directory exists and no preference is stated.

The fork path is `<dir>/<branch>` (the caller supplies the branch).

**Safety verification (project-local dirs).** MUST verify the directory is
gitignored before creating — otherwise the fork contents get tracked and pollute
git status:

```bash
git check-ignore -q .worktrees 2>/dev/null || git check-ignore -q worktrees 2>/dev/null
```

If **not** ignored, fix it immediately: add the line to `.gitignore`, commit, then
proceed. (A global dir outside the project needs no such check.)

## Always-provision (D9 — the invariant copy step)

**For any forked path (rungs 1–3), always** run the sole copier from the **source
(main-tree) root** — never from inside the fork:

```bash
doctrine worktree provision <fork>
```

`provision` reads `.worktreeinclude`, fails closed on a tier-naming pattern,
enumerates the gitignored candidate files, and copies only the allowlisted ones —
**withholding the coordination/runtime tier at the copy seam even under a broad
`**` pattern**. Committed files are already in the fork via HEAD; `provision`
carries only the irreducible *gitignored* files a project declares.

### Honest invariant framing (F7 — do not overstate)

The copy-seam guarantee holds **because `provision` is the only copier.**

- `doctrine worktree check-allowlist` is a **static smell test** — green means no
  pattern *names* the tier; it is **NOT** completeness. `select_copies` (inside
  `provision`) is the actual guarantee.
- **If a harness force-copies on creation** and cannot be run creation-only, doctrine
  cannot prevent that copy — the guarantee there degrades to `check-allowlist` only,
  and the project must keep `.worktreeinclude` precise. This is why rung 3 (fully
  controlled) is the default and rung 2 is never depended on.

## Guards

Run these around the spawn. A guard that fails **aborts** — report, do not
improvise past it.

### Commit-before-spawn (D5 — exact gate)

Before creation, the source tree must be clean so the fork sees only committed
HEAD:

```bash
git status --porcelain -z
```

**Abort** if any tracked file is dirty **or** any untracked, non-ignored file
exists (it would be silently absent from the fork). Ignored files are fine —
`provision` handles the allowlisted subset.

### Branch-point check (D5 — in scope)

Assert the fork's HEAD equals the **intended base** — `HEAD` for solo, the supplied
`base` (`B`) for a worker. **Resolve the intended base to a SHA first**, then compare
the fork against *that*, not against a re-read of session HEAD (which, for a worker,
is the wrong ref):

```bash
git rev-parse <base>               # intended base (HEAD for solo, B for worker) → SHA
git -C <fork> rev-parse HEAD       # fork, post-create → must equal that SHA
```

Mismatch ⇒ abort / recreate — a worker fork that landed on session HEAD instead of
`B` is exactly the divergent-base corruption this guard exists to catch. Cheap; solo
rarely exercises it, but it lands here so the funnel slice only *extends* it to the
concurrent case.

**Concurrency extension (SL-031, D5).** The funnel reuses the same ref-equality at
a *different* boundary — the batch commit. `doctrine worktree branch-point-check
--base <B> [--head <SHA>]` exits 0 iff coordination HEAD still equals the
orchestrator's pre-spawn base `B`, 1 otherwise (→ re-dispatch). It is the
**orchestrator's** guard, run worker-mode-OFF at import time — *not* something the
worker runs. Naming note (C-V): a HEAD-stationarity compare, not a merge-base.

### Baseline-verify (D9 — project-configured, green-gate)

After provisioning, run the project's regenerate-and-verify command **inside the
fork** and gate handoff on green:

```bash
cd <fork> && just check            # this repo: fmt + lint + test + build
```

The command is **project-provided** — doctrine is a framework, so it is never a
hardcoded `cargo …`. An unbuildable fork is fixed in provisioning, **never handed
off**.

## Worker mode (the funnel half)

`mode=worker` is the path `/dispatch` spawns inside an isolated fork. The fork,
provision, and three guards above all run first — **unchanged**; this section is
what happens *after* a green baseline, in place of solo handoff. The worker is a
constrained writer: it produces exactly one importable delta and returns, never
touching the coordination/runtime tier (the fork already withholds it, D9).

**Self-arm first (D2a, fail-open — C-I).** The worker's first act is:

```bash
export DOCTRINE_WORKER=1
```

This arms the doctrine guard so any doctrine-mediated authored write (`slice …`,
`memory record`, `backlog …`, minting) **refuses** — workers return a source
delta; all doctrine-mediated writes funnel through the orchestrator. The harness
`Agent` tool exposes **no env seam**, so nothing *enforces* the line — it is a
self-armed prompt contract that fails **open** if omitted (the orchestrator's
import-time `.doctrine/`-reject belt, not this var, is the real protection). Arm it
regardless.

**The constrained loop:**

1. **Mutate source only.** Edit tracked/untracked source files in the fork. Do
   **not** write `.doctrine/` authored trees, runtime state, or memory — those are
   the orchestrator's, and an import touching them is rejected (report+halt).
2. **Verify.** Run the **orchestrator-supplied** verify command (passed in the
   worker prompt — not assumed to be `just check`; doctrine is a framework). A red
   verify is reported back; the worker does not commit a red delta.
3. **Commit exactly one `S`.** Commit the source change to the fork branch as
   **one non-merge commit `S` descended from the base `B`**:

   ```bash
   git add -A && git commit -m "<task summary>"     # raw git — NOT a doctrine verb
   ```

   This is a plain `git commit`, **not** a doctrine-mediated authored write, so the
   `DOCTRINE_WORKER=1` guard does not refuse it (D2a). `S` is the **importable delta
   unit**: the orchestrator imports the net diff `B..S`. Therefore:
   - **Exactly one** non-merge commit on top of `B`. **No** multi-commit history,
     **no** merge commit, **no** rebase that re-parents off `B` — each is a contract
     violation the orchestrator rejects before import.
   - Stay within your declared file set; straying breaks the file-disjoint batch.

**MUST NOT degrade to work-in-place.** A worker with no real fork is a **hard
abort**, never a silent in-tree edit — isolation is the funnel's whole premise
(contrast `solo` rung 4). If creation failed, report and stop.

**Return** a structured report (held in orchestrator context, never a doctrine
artifact): what changed, the verify result, and any memory-worthy notes — plus the
two output fields `{ fork_branch, head_sha_after }` so the orchestrator can import
`B..head_sha_after`. Knowledge trails the orchestrator's confirmed commit, not the
fork (record-on-trunk, below).

## Squash-orphan caveat (record-on-trunk)

Memory recorded inside a worktree branch is **orphaned by a squash-merge** (the
content survives but the git anchor points at a commit that never lands;
SL-008 staleness fires). When durable memory must outlive the fork, **record it on
trunk**, not inside the worktree branch.

## The `.worktreeinclude` template (F2 — project-owned, not installed)

Doctrine has no secrets / irreducible local files → its own default is **nothing to
copy**, and the installer ships **no** `.worktreeinclude` (a root-file install would
clobber a consuming project's file). `provision` tolerates its absence (copies
nothing, exit 0).

A project that needs to carry gitignored local files into forks may adopt a
repo-root `.worktreeinclude`. Documented subset: blank lines, `#` comments, literal
repo-relative paths, and simple `glob` patterns (`*`, `**`, `?`). **No `!` negation,
no anchoring** in v1 (the parser rejects them). The coordination/runtime tier is
withheld regardless.

```gitignore
# .worktreeinclude — repo-relative gitignored files to carry into a worktree fork.
# Doctrine withholds the coordination/runtime tier (.doctrine/state, phases links,
# handover.md, memory caches) at the copy seam even if a pattern below matches it.
#
# .env.local              # example: a local env file the fork needs
# config/secrets.toml      # example: an untracked secret
#
# Validate statically (smell test, NOT completeness):
#   doctrine worktree check-allowlist
```

## Quick Reference

| Situation | Action |
|---|---|
| `--git-dir` ≠ `--git-common-dir`, not a submodule | Already forked → adopt, skip creation |
| Submodule (`modules/` gitdir / superproject) | Not isolation → treat as not-forked |
| Native creation-only hook present | Opportunistic rung 2; else fall through |
| Default fork | Rung 3: `git worktree add .worktrees/<branch> <branch> <base>` (base = `HEAD` solo, `B` worker) |
| `worker` fork base | Rung-3 from the supplied `B`; pass base explicitly, never inherit current HEAD (the harness-native rung can't pin base) |
| Worktree dir not ignored | Add to `.gitignore` + commit before creating |
| Sandbox denies fork, `mode=solo` | Rung 4: work-in-place (no fork) |
| Sandbox denies fork, `mode=worker` | **Abort** (isolation mandatory) |
| Any fork created | Always `doctrine worktree provision <fork>` |
| Tree dirty / untracked-non-ignored | **Abort** commit-before-spawn |
| Fork HEAD ≠ intended base (`HEAD` solo / `B` worker) | **Abort** / recreate — divergent-base corruption |
| `just check` red in fork | Fix in provisioning; **never hand off** |
| `worker` start | `export DOCTRINE_WORKER=1` (self-arm, fail-open) |
| `worker` verify green | Commit ONE non-merge `S` to fork; return `{fork_branch, head_sha_after}` |
| `worker` >1 commit / merge / rebased fork | **Contract violation** — orchestrator rejects pre-import |
| `worker` verify red | Report; **do not** commit a red delta |

## Red Flags

**Never:**
- Copy gitignored files into a fork by any path other than `doctrine worktree
  provision` (it is the sole copier — the exclusion guarantee depends on it).
- Imply `check-allowlist` green means the allowlist is complete (it is a smell test).
- Run `provision` from inside the fork (run it from the source root).
- Let a `worker` degrade to work-in-place.
- In `worker` mode: write `.doctrine/` authored trees, skip `export
  DOCTRINE_WORKER=1`, or land more than one non-merge commit `S` (multi-commit,
  merge, or rebased forks are rejected — the import unit is the net diff `B..S`).
- Fork from a dirty tree or hand off a red baseline.
- Fork a `worker` from the implicit current/session HEAD instead of the supplied
  `base` (`B`) — for a worker the session HEAD is not `B`, so it is a divergent base
  that breaks `S.parent == B`. Rung-3 from the explicit `B`; the harness-native rung
  can't pin the base.
- Author or edit this skill in `.doctrine/skills/` (the gitignored install copy);
  the source of truth is here under `plugins/`.

**Always:**
- Detect before creating; adapt to existing isolation.
- Prefer rung 3 (`git worktree add`); treat the native hook as opportunistic.
- Provision every fork; run all three guards; verify a green baseline before handoff.
- Record durable memory on trunk, not inside the fork branch.

## Outcome

Report the outputs: `fork_path`, `branch`, `head_sha`, the provision report
(copied / withheld files), and the baseline result. The fork branch is the
deliverable handed back to the caller.

<!-- Attribution: see NOTICE.md (adapted from superpowers:using-git-worktrees, MIT). -->
