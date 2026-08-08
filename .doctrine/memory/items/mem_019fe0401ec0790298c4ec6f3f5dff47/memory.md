# Claude Code's `isolation: worktree` — what it actually enforces

Probed empirically 2026-08-08 in a **doctrine-free** project on NixOS, across
three permission postures (SL-250 `VH-1` control runs). Every claim is a probe
result, not a reading of docs.

**Headline: confinement depends on the permission posture, not on
`isolation: worktree`.** The isolation flag alone buys a tool-layer path check
that a symlink defeats. `/sandbox` is what buys kernel confinement.

## Default and auto modes — no kernel confinement

| probe | result |
|---|---|
| `readlink /proc/self/ns/mnt` | `mnt:[4026531832]` — the **init** namespace |
| `wc -l /proc/self/mountinfo` | 31 — the host's full mount table |
| `bwrap` in the shell's ancestry | **absent**; plain `bash -c` under the harness |
| shell write to the parent checkout | rc=0, **landed for real** |

## `/sandbox` mode — real confinement, and honest about it

| probe | result |
|---|---|
| `readlink /proc/self/ns/mnt` | `mnt:[4026535075]` — a **distinct** mount namespace |
| `wc -l /proc/self/mountinfo` | 78 — a constructed table |
| `ps -o pid,ppid,args` | the shell is **PID 1**; a PID namespace too |
| shell write to the parent checkout | **exit 1**, `Read-only file system` |
| shell read of the parent checkout | **allowed**, and the content is true |

So `/sandbox` mounts outside-the-worktree **read-only**, passes reads through
truthfully, and fails writes loudly. No phantom in either direction.

## Two guard layers, disagreeing about the same path

Target: `<parent>/CANARY.md` — inside the repo, outside the worktree.

- **`Write` tool → refused**, harness vocabulary, no doctrine prefix: *"This
  agent is isolated in the worktree <wt>. Edit the worktree copy of this file
  instead of the shared-checkout path."*
- **`echo SHELL-PARENT > <parent>/CANARY.md` → succeeded** (outside `/sandbox`),
  and the file was genuinely overwritten, verified from the parent session.

Same path, same session, opposite verdicts.

## The Bash guard refuses on syntax, not on paths

Under auto and `/sandbox` the harness **refused** every command containing a
redirect, a pipe, or a compound `;`:

> *"…this command is too complex to verify that it stays inside the worktree;
> break it into plain, separate commands. Refusing to run it — a worktree-isolated
> agent's git operations must target its own worktree."*

Two things make this worse than it reads:

1. It refused a redirect targeting a path **inside** the worktree
   (`echo x > ./CTRL.md`) — entirely benign. It is not analysing the path.
2. It then **permitted** the decomposed form writing **outside** the worktree.

It rejects the safe command and admits the unsafe one, and the refusal text
asserts a claim about "git operations" for commands containing no git. The
refusals read as rigour and function as the opposite: they steer the operator
toward exactly the simple forms that pass.

## Escapes that walk straight through (outside `/sandbox`)

All rc=0, all landed for real:

- **symlink** — `ln -s <parent> ./escape`, then write via `./escape/CANARY.md`.
  A relative path defeats the tool guard's absolute-path check.
- **`cd`** — `cd <parent> && echo VIA-CD > CANARY.md`.
- **python** — `python3 -c "open('<parent>/CANARY.md','w').write(...)"`.

## The tool guard's boundary is the parent *checkout*, not "outside the worktree"

A target entirely outside the repo (`/tmp/probe-outside/target.txt`) was writable
by **both** layers — the `Write` tool allowed it and shell allowed it. The guard
only knows the shared checkout path. `$HOME`, `/tmp` and any sibling repo are
unguarded.

## Consequence for doctrine

**Doctrine's jail is not made redundant by the harness.** Absent `/sandbox`, the
harness contributes a tool-layer check on one path, defeated by a symlink. A
dispatch posture that assumed `isolation: worktree` confines a worker would be
assuming something only true when the operator happens to be in `/sandbox`.

## Telling the two confinements apart

They have opposite signatures, so a failure attributes itself:

| | doctrine's wrap | Claude `/sandbox` |
|---|---|---|
| parent checkout readable | **no** — `git` fails `not a git repository: (null)` | **yes**, content true |
| write outside | **rc=0, silent, no file** | **exit 1, `Read-only file system`** |
| deny message | prefixed `worktree-jail: ` (`src/worktree/pretooluse.rs:58`) | harness text, "shared-checkout path" |
| fork location | `.worktrees/<name>` (`WORKTREES_SUBDIR`) | `.claude/worktrees/agent-<id>` |

Doctrine hides the outside and absorbs writes; `/sandbox` exposes it read-only
and refuses them. See
[[mem.fact.worktree.gitdir-pointer-unresolvable-in-sandboxed-subagent]] for why
doctrine's posture makes a worker's own instruments lie, in both directions.

## Caveats

Linux/NixOS only; macOS Seatbelt not probed. Build not pinned — re-probe after a
harness upgrade, since this is exactly the kind of gap a vendor closes quietly.
