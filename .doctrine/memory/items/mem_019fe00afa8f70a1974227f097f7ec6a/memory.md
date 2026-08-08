# A worktree subagent's filesystem view lies in BOTH directions

Observed 2026-08-08 (SL-250 `VH-1` probes). Two failure modes, one cause, and
the cause is **doctrine's own jail working as designed** — established by
differential against a doctrine-free control project (see *Attribution*, below).

**The cause.** `worktree pretooluse` wraps Bash into a bwrap jail whose view is
scoped to the worktree. Anything outside is not merely unwritable — it is *not
there*, as far as the confined process can tell. Both modes below follow, and
both are dangerous because the worker's own instruments report success.

## Direction 1 — phantom absence (loud, and misdiagnosed)

Every `git` command inside the worktree fails identically:

```
$ git rev-parse --show-toplevel
fatal: not a git repository: (null)
[exit 128]
```

`pwd` succeeds; only `git` fails. The `(null)` is the tell.

A linked worktree's `.git` is a **file**, not a directory, holding
`gitdir: /<parent>/.git/worktrees/<name>`. The jailed view cannot follow that
pointer out to the parent, so git resolves it to nothing.

**The parent repository is fine. Nothing has been deleted.**

The probe subagent instead concluded *"the parent repository is genuinely gone …
Confirmed with the sandbox disabled, so it is not a sandbox filesystem-view
artifact."* Both claims false — the operator verified every named file present
throughout.

The hazard is not the git error; it is the **story built on it**. Absence of
evidence inside a confined view reads as evidence of absence, and the resulting
narrative — catastrophic mid-session repo loss — is alarming enough to derail
the work. Expect a worker to escalate this as an emergency.

## Direction 2 — phantom success (silent, and worse)

```
$ echo probe > /<parent>/README.md
(Bash completed with no output)
```

Exit 0. No error envelope, no stderr. **And no file** — not at the target, not
redirected into the worktree, nowhere on disk. Verified from the parent session
by `cat` and a `find` across every worktree.

More dangerous than direction 1 precisely because it is quiet. The git error
announces itself; this looks like the write worked.

**A worker cannot trust `echo >`, `cp`, `mv`, `tee`, or any shell write outside
its worktree to have happened. A zero exit code is not evidence it landed.**

## Attribution — settled by differential

Both directions are **doctrine's wrap**, not the harness. A control project with
no doctrine installed was probed under two permission modes:

| | doctrine project | clean control |
|---|---|---|
| shell write to parent checkout | rc=0, **absorbed** | rc=0, **landed for real** |
| `readlink /proc/self/ns/mnt` | — | `mnt:[4026531832]` (init) |
| `wc -l /proc/self/mountinfo` | — | 31 |
| `bwrap` in shell ancestry | — | **absent** |

The control shows no kernel confinement in either default or auto mode, so the
permission mode cannot explain the absorb. Doctrine's `worktree pretooluse`
(`src/worktree/pretooluse.rs`, `WrapBash` ⇒ `permissionDecision: "allow"` plus a
rewritten command) is the only remaining candidate.

Note the two doctrine outcomes have different signatures and only one is loud:
`Deny` emits a reason prefixed `worktree-jail: `; `WrapBash` emits allow-plus-
rewrite and is silent. **Absence of `worktree-jail:` does not mean the wall was
inactive** — it usually means the wrap path ran.

What the harness contributes is a separate, weaker layer — see
[[mem.fact.claude.native-worktree-isolation-is-tool-layer-only]].

## What to do

- **Verify a worker's writes from the parent session**, never from its exit code.
- **Never conclude repo loss from inside a worktree.** `cat .git` shows the
  `gitdir:` pointer — a readable pointer with an unreachable target is this, not
  deletion.
- A worker needing git facts should be **given** them by the orchestrator, or use
  the broker (`worker_commit` on the claude arm), rather than shelling `git`.
- Sanity-check any claim that a sandbox was "disabled" or "ruled out" — that is
  the assertion that made a wrong diagnosis sound verified rather than guessed.

Adjacent, distinct: on the subprocess (pi) arm a worker cannot self-commit
because linked worktrees get a read-only `.git` (AGENTS.md) — a *permission*
limit on a resolvable path. This is the path not resolving at all.

See [[mem.pattern.harness.grep-negative-needs-positive-control]] — the same
epistemics one layer down. Here the instrument lies positively too.
