# A worktree subagent's filesystem view lies in BOTH directions

Observed 2026-08-08 (SL-250 `VH-1` probes, `/tmp/install_test`, Claude Code,
`isolation: worktree` subagents). Three probes, two failure modes, one cause.

**The cause.** The subagent's filesystem view is scoped to its own worktree.
Anything outside is not merely unwritable — it is *not there*, as far as the
confined process can tell. Both failure modes below follow from that, and both
are dangerous because the subagent's own instruments report success.

## Direction 1 — phantom absence (loud, and misdiagnosed)

Every `git` command inside the worktree fails identically:

```
$ git rev-parse --show-toplevel
fatal: not a git repository: (null)
[exit 128]
```

`pwd` succeeds; only `git` fails. The `(null)` is the tell.

A linked worktree's `.git` is a **file**, not a directory, holding
`gitdir: /<parent>/.git/worktrees/<name>`. The confined view cannot follow that
pointer out to the parent, so git resolves it to nothing.

**The parent repository is fine. Nothing has been deleted.**

The probe subagent instead concluded *"the parent repository is genuinely gone …
Confirmed with the sandbox disabled, so it is not a sandbox filesystem-view
artifact … the parent repo was removed after this session began."* Every part
was false — the operator verified `<parent>/.git` and every other named file
present throughout. Its "confirmed with the sandbox disabled" step did not
happen or did not take.

The hazard is not the git error; it is the **story built on it**. Absence of
evidence inside a confined view reads as evidence of absence, and the resulting
narrative — catastrophic mid-session repo loss — is alarming enough to derail
the work. Expect a worker to escalate this as an emergency.

## Direction 2 — phantom success (silent, and worse)

```
$ echo probe > /<parent>/README.md
(Bash completed with no output)
```

Exit 0. No error envelope, no stderr. **And no file** — not at the target path,
not redirected into the worktree, nowhere on disk. Verified from the parent
session afterward by `cat` and a `find` across every worktree.

This is the more dangerous direction precisely because it is quiet. Direction 1
announces itself; direction 2 looks like the write worked.

**A worktree subagent cannot trust `echo >`, `cp`, `mv`, `tee`, or any shell
write outside its worktree to have happened. A zero exit code is not evidence
the write landed.**

Note the asymmetry with the tool layer: the `Write`/`Edit` *tools* blocked
loudly with an isolation error naming the worktree, while `Bash` was absorbed
silently. The guard sits at the tool layer; shell escapes fall through to the
filesystem view instead.

## What to do

- **Verify writes outside a worktree from the parent session**, never from the
  worker's exit code.
- **Never conclude repo loss from inside a worktree.** Confirm from the parent
  before acting. `cat .git` in the worktree shows the `gitdir:` pointer — a
  readable pointer with an unreachable target is this, not deletion.
- A worker needing git facts should be **given** them by the orchestrator, or
  use the broker (`worker_commit` on the claude arm), rather than shelling `git`.
- Sanity-check any claim that a sandbox was "disabled" or "ruled out" — that is
  the assertion that made a wrong diagnosis sound verified rather than guessed.

## Attribution is open

Which mechanism confines the view was **not** established. Claude Code ships
worktree-aware isolation at the tool layer (the `Write` block named the worktree
and the shared-checkout path, in harness vocabulary, with no
`worktree-jail: ` prefix — so that one was the harness, not doctrine). Doctrine's
own `worktree pretooluse` wrap (`src/worktree/pretooluse.rs`, `WrapBash` ⇒
`permissionDecision: "allow"` + rewritten command) produces an
observationally identical silent-absorb for direction 2. Both were live during
these probes.

The behaviour above holds either way, which is why it is recorded as a fact
about worktree subagents rather than about doctrine's jail. To settle
attribution, A/B it: remove doctrine's two `Bash` `PreToolUse` entries from the
project settings file and re-run — if the absorb persists, it is the harness.

Adjacent, distinct: on the subprocess (pi) arm a worker cannot self-commit
because linked worktrees get a read-only `.git` (AGENTS.md). That is a
*permission* limit on a resolvable path. This is the path not resolving at all.

See [[mem.pattern.harness.grep-negative-needs-positive-control]] — the same
epistemics one layer down: a negative result is a claim about your instrument
before it is a claim about the world. Here the instrument lies positively too.
