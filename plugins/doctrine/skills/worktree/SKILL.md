---
name: worktree
description: Use when work needs isolation from the current workspace — when /execute (mode=solo) or the /dispatch funnel must run a phase on its own branch without touching the main working tree. Detects existing isolation, creates a fork via `doctrine worktree fork` (the sole creation+provision+mark verb; coordination tier excluded), runs the commit-before-spawn / branch-point / baseline guards, and hands back the fork branch.
---

# Worktree

Drive the lifecycle of an isolated git worktree: detect existing isolation,
create a fork through **`doctrine worktree fork`** (the verb that adds the
worktree, provisions it, and optionally stamps the worker marker — all with
compensating rollback; the fork builds into its own in-tree `target/`), run the
spawn guards, and verify a green baseline before handing off.

**Announce at start:** "Using the worktree skill to set up an isolated workspace."

**This is a sub-skill.** `/execute` (mode=solo) invokes it for opt-in isolation;
`/dispatch-subprocess` invokes it for codex/pi workers; the `/dispatch` router
invokes the **coordination** path (`worktree coordinate`, below) for the
per-run `dispatch/<slice>` tree. (The claude *worker* arm, `/dispatch-agent`, does
**not** use `fork` — Claude default-creates the worktree and a SubagentStart hook
stamps it; see that skill.) It is not a `/route` destination — reach it through the
caller.

## Mode contract

The skill is parameterised so the funnel reuses it without inheriting solo
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
  the funnel's isolation is mandatory. `fork --worker --base <B>` pins the base
  explicitly — never the implicit session HEAD, which for a worker is *not* `B`
  and breaks `S.parent == B`. Beyond creation, the worker half runs a constrained
  edit→verify→**commit-one-`S`-to-fork** loop; see
  [Worker mode](#worker-mode-the-funnel-half) (SL-031 §5.2).

**Outputs:** `{ fork_path, branch, head_sha, provision_report, baseline_result }`.
`worker` adds `{ fork_branch, head_sha_after }` once it has committed its single
delta `S` — what the orchestrator imports `B..head_sha_after` from.

## Coordination mode — `doctrine worktree coordinate` (SL-064, a THIRD path)

`solo`/`worker` above both go through `fork`. The **dispatch coordination worktree**
is a separate creation verb, **not** a `fork` mode — the `/dispatch` router calls it
once per run, before batch 1:

```sh
doctrine worktree coordinate --slice <N> --dir <path>
```

It creates (or resumes) `dispatch/<slice>` in its own worktree off the **resolved
trunk** — the funnel's sole write target (design §2; ADR-012). It differs from
`fork` on every axis that matters:

- **Markerless.** The coordination tree **is** the orchestrator (worker-mode OFF),
  so — unlike `fork --worker` — it stamps **no** worker marker. Orchestrator-classed,
  refused under worker-mode like the rest of the verb class.
- **Create-or-resume, never a second branch.** A live worktree already on
  `dispatch/<slice>` is **refused** (`coordination-live` — concurrent same-slice
  dispatch is illegitimate). A branch that exists with **no** live worktree
  **resumes** (reattach) — so a fresh orchestrator after `/handover` picks up the
  same branch (resume-stable; a per-run discriminator would *break* resume).
- **Regenerates the runtime phase sheets** from the committed `plan.toml` (the
  SL-056 provision axis), via the sole copier — no coordination-tier copy.

Not a worker delta path: there is no `S`, no import, no marker. Lifecycle is
worktree-life < branch-life — the directory is removed at conclude, but
`dispatch/<slice>` + the projected `review`/`phase` refs are **kept** as deliverables
(see `/dispatch`). Solo/worker creation below is unchanged.

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

## Creation — `doctrine worktree fork` (the controlled path — D5/F1)

Pick the fork directory, run the commit-before-spawn guard, then call the verb:

```bash
doctrine worktree fork --base <B> --branch <name> --dir <path> [--worker]
```

`fork` is the **single create+provision+(mark)+emit verb**, atomic via
compensating rollback (any failure after `git worktree add` triggers a best-effort
`worktree remove --force` / `branch -D` / dir reap; a rollback that itself fails
**names the leftover and exits non-zero** — never a silent half-rollback). It:

1. `git worktree add -b <branch> <dir> <base>` — refuses if `<dir>`/`<branch>`
   exist or `<base>` is not a commit. Pass `<base>` explicitly: `HEAD` for solo,
   `B` for a worker — the explicit base is *why* this path is controlled.
2. **provisions** the fork (the sole copier — coordination/runtime tier withheld
   at the copy seam, D9; see [Provisioning](#provisioning-d9)).
3. with `--worker`, **stamps the worker marker** before any spawn window (solo
   omits `--worker`).
4. reports **human status to stderr**; stdout stays empty (machine-clean). The fork
   builds into its own in-tree `<dir>/target` — no env contract is emitted (SL-156:
   the platform exited the build-env business).

`fork` is **Orchestrator-classed — refused under worker-mode** (a worker never
forks; the orchestrator/solo, worker-mode OFF, calls it).

**Creation backend ladder — degrade in order:**
1. **Existing isolation** (detection above) → skip creation, adopt the fork.
2. **Harness-native creation** if present and invocable **creation-only** (no
   auto-copy) — opportunistic only. (Claude Code's `WorktreeCreate` hook is the
   instance, but its payload lacks base/path/type so doctrine cannot drive it —
   the claude arm instead default-creates + SubagentStart-stamps, see
   `/dispatch-agent`.) **Design around it; never depend on it.**
3. **`doctrine worktree fork`** — the **blessed tested default**. Prefer this.
4. **Work-in-place** (`solo` only, `allow_work_in_place=true`) on sandbox denial —
   no fork, the trunk path. `worker` aborts instead.

### Worktree directory selection

Pick the fork directory before calling `fork`, in priority order
(`superpowers:using-git-worktrees`):

1. **Reuse an existing dir** — `.worktrees` wins over `worktrees` if both exist.
2. **Check CLAUDE.md** for a stated preference — use it without asking.
3. **Ask the user** if no directory exists and no preference is stated.

The fork path is `<dir>/<branch>`. **Safety:** the dir MUST be gitignored
(`git check-ignore -q .worktrees`) before creating, else the fork contents get
tracked — if not ignored, add the line to `.gitignore` and commit first. (A
global dir outside the project needs no such check.)

## Provisioning (D9)

`fork` provisions automatically; **never run a second copier.** `doctrine
worktree provision <fork>` exists as the standalone sole copier (run from the
**source root**, never inside the fork) for the rare adopt-existing path that
skipped `fork`. It reads `.worktreeinclude`, fails closed on a tier-naming
pattern, and copies only allowlisted gitignored files — **withholding the
coordination/runtime tier at the copy seam even under a broad `**` pattern.**

### Honest invariant framing (F7 — do not overstate)

The copy-seam guarantee holds **because provision is the only copier.**

- `doctrine worktree check-allowlist` is a **static smell test** — green means no
  pattern *names* the tier; it is **NOT** completeness. `select_copies` (inside
  provision) is the actual guarantee.
- **If a harness force-copies on creation** and cannot be run creation-only,
  doctrine cannot prevent that copy — the guarantee degrades to `check-allowlist`
  only, and the project must keep `.worktreeinclude` precise. This is why the
  `fork` verb (fully controlled) is the default and the harness-native rung is
  never depended on.

## Guards

Run these around the spawn. A guard that fails **aborts** — report, do not
improvise past it.

- **Commit-before-spawn (D5).** Before `fork`, the source tree must be clean so
  the fork sees only committed HEAD: `git status --porcelain -z`. **Abort** on any
  dirty tracked file **or** any untracked non-ignored file (it would be silently
  absent from the fork). Ignored files are fine — provision handles the
  allowlisted subset.
- **Branch-point check (D5).** `fork` lands the worktree at `<base>` by
  construction; the **concurrency extension** is the batch-commit boundary:
  `doctrine worktree branch-point-check --base <B>` exits 0 iff coordination HEAD
  still equals the orchestrator's pre-spawn `B`, 1 otherwise (→ re-dispatch). It is
  the **orchestrator's** guard at import time, worker-mode OFF — not the worker's.
  A HEAD-stationarity compare, not a merge-base (C-V).
- **Baseline-verify (D9).** After creation, run the project's regenerate-and-verify
  command **inside the fork** and gate handoff on green (this repo: `doctrine
  check gate`). The command is **project-provided** — never a hardcoded `cargo …`. An unbuildable
  fork is fixed in provisioning, **never handed off**.

## Worker mode (the funnel half)

`mode=worker` is the path a codex/pi worker runs inside its `fork --worker`
worktree (claude workers are stamped by the SubagentStart hook instead — see
`/dispatch-agent`). Creation, provision, and the guards above run first
**unchanged**; this is what happens *after* a green baseline, in place of solo
handoff. The worker is a constrained writer: one importable delta, then return,
never touching the coordination/runtime tier (the fork already withholds it, D9).

**Self-arm first (D2a, fail-open — C-I).** The worker's first act is `export
DOCTRINE_WORKER=1`. This arms the doctrine guard so any doctrine-mediated authored
write (`slice`, `memory record`, `backlog`, minting) **refuses**. Nothing
*enforces* the line — the `Agent` tool exposes no env seam, so it is a self-armed
prompt contract that fails **open** if omitted (the orchestrator's import-time
`.doctrine/`/`.claude/`-reject belt, not this var, is the real protection). Arm it
regardless.

**The constrained loop:**

1. **Mutate source only.** Edit source files in the fork. Do **not** write
   `.doctrine/` authored trees, runtime state, or memory — an import touching them
   is rejected (report+halt).
2. **Verify.** Run the **orchestrator-supplied** verify command (passed in the
   worker prompt — not assumed `doctrine check gate`). A red verify is reported back; the
   worker does not commit a red delta.
3. **Commit exactly one `S`.** `git add -A && git commit` — raw git, **not** a
   doctrine verb (so `DOCTRINE_WORKER=1` does not refuse it, D2a). `S` is the
   importable unit (the orchestrator imports `B..S`): **exactly one** non-merge
   commit on top of `B` — no multi-commit, merge, or rebase that re-parents off
   `B` (each is a contract violation rejected before import). Stay within your
   declared file set; straying breaks the file-disjoint batch.

**MUST NOT degrade to work-in-place.** A worker with no real fork is a **hard
abort**, never a silent in-tree edit (contrast `solo` rung 4). If creation failed,
report and stop.

**Return** a structured report (held in orchestrator context, never a doctrine
artifact): what changed, the verify result, memory-worthy notes, plus
`{ fork_branch, head_sha_after }`. Knowledge trails the orchestrator's confirmed
commit, not the fork (record-on-trunk, below).

## Squash-orphan caveat (record-on-trunk)

Memory recorded inside a worktree branch is **orphaned by a squash-merge** (the
content survives but the git anchor points at a commit that never lands; SL-008
staleness fires). When durable memory must outlive the fork, **record it on
trunk**, not inside the worktree branch.

## The `.worktreeinclude` template (F2 — project-owned, not installed)

Doctrine has no secrets / irreducible local files → its own default is **nothing
to copy**, and the installer ships **no** `.worktreeinclude` (a root-file install
would clobber a consuming project's file). Provision tolerates its absence (copies
nothing, exit 0).

A project that needs to carry gitignored local files into forks may adopt a
repo-root `.worktreeinclude`: blank lines, `#` comments, literal repo-relative
paths, simple `glob` patterns (`*`, `**`, `?`). **No `!` negation, no anchoring**
(the parser rejects them). The coordination/runtime tier is withheld regardless.
Validate statically (smell test, NOT completeness): `doctrine worktree
check-allowlist`.

## Quick Reference

| Situation | Action |
|---|---|
| `--git-dir` ≠ `--git-common-dir`, not a submodule | Already forked → adopt, skip creation |
| Submodule (`modules/` gitdir / superproject) | Not isolation → treat as not-forked |
| Dispatch coordination tree (per run) | `doctrine worktree coordinate --slice <N> --dir <path>` — markerless, create-or-resume, off resolved trunk; NOT a `fork` mode |
| Default fork (solo or codex/pi worker) | `doctrine worktree fork --base <B> --branch <name> --dir <path> [--worker]` |
| Worker fork base | `--base <B>` explicit, never the implicit session HEAD (it is not `B`) |
| Worktree dir not ignored | Add to `.gitignore` + commit before `fork` |
| Sandbox denies fork, `mode=solo` | Work-in-place (no fork) |
| Sandbox denies fork, `mode=worker` | **Abort** (isolation mandatory) |
| Adopted a fork that skipped `fork` | `doctrine worktree provision <fork>` from the source root (sole copier) |
| Tree dirty / untracked-non-ignored | **Abort** commit-before-spawn |
| Batch-commit boundary | `doctrine worktree branch-point-check --base <B>` → 1 = re-dispatch |
| Baseline red in fork | Fix in provisioning; **never hand off** |
| `worker` start | `export DOCTRINE_WORKER=1` (self-arm, fail-open) |
| `worker` verify green | Commit ONE non-merge `S`; return `{fork_branch, head_sha_after}` |
| `worker` >1 commit / merge / rebased fork | **Contract violation** — orchestrator rejects pre-import |
| `worker` verify red | Report; **do not** commit a red delta |

## Red Flags

**Never:**
- Copy gitignored files into a fork by any path other than `fork` / `provision`
  (the sole copiers — the exclusion guarantee depends on them).
- Run a second copier after `fork`, or run `provision` from inside the fork.
- Imply `check-allowlist` green means the allowlist is complete (it is a smell test).
- Let a `worker` degrade to work-in-place.
- In `worker` mode: write `.doctrine/` authored trees, skip `export
  DOCTRINE_WORKER=1`, or land more than one non-merge commit `S`.
- Fork from a dirty tree or hand off a red baseline.
- Fork a `worker` from the implicit session HEAD instead of `--base <B>` — for a
  worker the session HEAD is not `B`, a divergent base that breaks `S.parent == B`.
- Author or edit this skill in `.doctrine/skills/` (the gitignored install copy);
  the source of truth is here under `plugins/`.

**Always:**
- Detect before creating; adapt to existing isolation.
- Prefer the `fork` verb; treat the harness-native rung as opportunistic.
- Run all three guards; verify a green baseline before handoff.
- Record durable memory on trunk, not inside the fork branch.

## Outcome

Report the outputs: `fork_path`, `branch`, `head_sha`, the provision report
(copied / withheld files), and the baseline result. The fork branch is the
deliverable handed back to the caller.

<!-- Attribution: see NOTICE.md (adapted from superpowers:using-git-worktrees, MIT). -->
