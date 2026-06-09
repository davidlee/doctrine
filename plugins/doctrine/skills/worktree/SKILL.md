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

**Behaviour:**
- `solo` — MAY degrade to the work-in-place rung on sandbox denial (no fork; the
  blessed trunk path).
- `worker` — **MUST NOT degrade.** A worker with no real fork is a hard failure;
  the funnel's isolation is mandatory.

**This slice (SL-029) implements `solo`. `worker` is DECLARED here, implemented by
the funnel slice — do not add worker behaviour beyond this contract.**

**Outputs:**
```
{ fork_path, branch, head_sha,
  provision_report { copied, withheld },
  baseline_result }
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
3. **`git worktree add <path> <branch>`** — guaranteed-present, the **blessed
   tested default**. Prefer this rung; it is fully controlled.
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

Capture the source HEAD *before* creating, assert the fork's HEAD matches it
*after*:

```bash
git rev-parse HEAD                 # source, pre-create  → SHA
git -C <fork> rev-parse HEAD       # fork,   post-create → must equal SHA
```

Mismatch ⇒ abort / recreate. Cheap; solo rarely exercises it, but it lands here so
the funnel slice only *extends* it to the concurrent case.

### Baseline-verify (D9 — project-configured, green-gate)

After provisioning, run the project's regenerate-and-verify command **inside the
fork** and gate handoff on green:

```bash
cd <fork> && just check            # this repo: fmt + lint + test + build
```

The command is **project-provided** — doctrine is a framework, so it is never a
hardcoded `cargo …`. An unbuildable fork is fixed in provisioning, **never handed
off**.

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
| Default fork | Rung 3: `git worktree add .worktrees/<branch> <branch>` |
| Worktree dir not ignored | Add to `.gitignore` + commit before creating |
| Sandbox denies fork, `mode=solo` | Rung 4: work-in-place (no fork) |
| Sandbox denies fork, `mode=worker` | **Abort** (isolation mandatory) |
| Any fork created | Always `doctrine worktree provision <fork>` |
| Tree dirty / untracked-non-ignored | **Abort** commit-before-spawn |
| Fork HEAD ≠ captured source SHA | **Abort** / recreate |
| `just check` red in fork | Fix in provisioning; **never hand off** |

## Red Flags

**Never:**
- Copy gitignored files into a fork by any path other than `doctrine worktree
  provision` (it is the sole copier — the exclusion guarantee depends on it).
- Imply `check-allowlist` green means the allowlist is complete (it is a smell test).
- Run `provision` from inside the fork (run it from the source root).
- Let a `worker` degrade to work-in-place.
- Fork from a dirty tree or hand off a red baseline.
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
