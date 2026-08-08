# `not a git repository: (null)` in a worktree subagent means the sandbox, not a lost repo

Observed 2026-08-08 (SL-250 `VH-1` probe, `/tmp/install_test`, Claude Code,
manual permission mode).

## The symptom

A subagent spawned with `isolation: worktree` lands in a real linked worktree —
`pwd` returns it, the parent's `git worktree list` registers it — and then
**every** `git` command inside it fails identically:

```
$ git rev-parse --show-toplevel
fatal: not a git repository: (null)
[exit 128]
```

`pwd` succeeds. Only `git` fails. The `(null)` is the tell.

## The cause

A linked worktree's `.git` is a **file**, not a directory, holding a pointer:

```
gitdir: /<parent>/.git/worktrees/<name>
```

The subagent's sandboxed filesystem view is scoped to its own worktree, so it
cannot follow that pointer out to the parent. Git resolves the pointer to
nothing and reports the unresolvable target as `(null)`.

The parent repository is **fine**. Nothing has been deleted.

## Why this is worth a memory: the misdiagnosis is confident and wrong

The probe subagent concluded:

> the parent repository is genuinely gone … Confirmed with the sandbox disabled,
> so it is not a sandbox filesystem-view artifact … this contradicts the
> session-start snapshot, so the parent repo was removed after this session began.

Every part of that was false. `<parent>/.git` was present throughout, along with
every other file the subagent claimed missing. Its "confirmed with the sandbox
disabled" claim did not hold — it either did not disable the sandbox or the view
persisted regardless.

The failure mode is not the git error; it is the **story an agent builds around
it**. Absence-of-evidence inside a confined view reads as evidence-of-absence,
and the resulting narrative (catastrophic repo loss, mid-session deletion) is
alarming enough to derail whatever the worker was doing. Expect a worker to
report this as an emergency.

## What to do

- **Never conclude repo loss from inside a worktree.** The confined view cannot
  see the parent by construction. Confirm from the parent session before acting.
- A worker needing git facts should be **given** them by the orchestrator, or
  use the broker (`worker_commit` on the claude arm) rather than shelling `git`.
- If you must diagnose: `cat .git` in the worktree shows the `gitdir:` pointer.
  A readable pointer plus an unreadable target is this, not deletion.
- Sanity-check any claim that a sandbox was "disabled" — that is the assertion
  that made this diagnosis sound verified rather than guessed.

Adjacent, distinct: on the subprocess (pi) arm a worker cannot self-commit
because linked worktrees get a read-only `.git` (AGENTS.md) — that is a
*permission* limit on a resolvable path. This is the path failing to resolve at
all.

Operator's note from the same session: worktrees behave oddly when the repo sits
outside the usual workspace path (this one was at `/tmp`), so treat that as an
aggravating factor rather than the root cause.

See [[mem.pattern.harness.grep-negative-needs-positive-control]] for the same
epistemics one layer down — a negative result is a claim about your instrument
before it is a claim about the world.
