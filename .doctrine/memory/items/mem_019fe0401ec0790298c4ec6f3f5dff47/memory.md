# Claude Code's `isolation: worktree` is not confinement

Probed empirically 2026-08-08 in a **doctrine-free** project on NixOS, under
both default and auto permission modes (SL-250 `VH-1` control run). Every claim
below is a probe result, not a reading of docs.

## No kernel confinement, in either mode

| probe | result |
|---|---|
| `readlink /proc/self/ns/mnt` | `mnt:[4026531832]` — the **init** namespace |
| `wc -l /proc/self/mountinfo` | 31 — the host's full mount table |
| `bwrap` in the shell's ancestry (`ps -o pid,ppid,args`) | **absent**; plain `bash -c` under the harness |

No mount namespace, no bind mounts, no sandbox binary. Whatever `isolation:
worktree` does, it does not confine the process.

## Two guard layers, disagreeing about the same path

Target: `<parent>/CANARY.md` — inside the repo, outside the worktree.

- **`Write` tool → refused**, with harness vocabulary and no doctrine prefix:
  *"This agent is isolated in the worktree <wt>. Edit the worktree copy of this
  file instead of the shared-checkout path."*
- **`echo SHELL-PARENT > <parent>/CANARY.md` → succeeded**, and the file was
  genuinely overwritten (verified from the parent session).

Same path, same session, opposite verdicts.

## The Bash guard is static analysis that fails open on the simple case

Under auto mode the harness **refused** every command containing `$` expansion,
a pipe, or a compound `;`:

> *"…this command is too complex to verify that it stays inside the worktree;
> break it into plain, separate commands. Refusing to run it."*

Then it **permitted** the plain, decomposed form — including a bare redirect to
the parent checkout. So it refuses what it cannot parse and approves what it
can parse while getting the verdict wrong. The refusals read as rigour and are
the opposite: they push the operator toward exactly the simple forms that pass.

## Escapes that walk straight through

All rc=0, all landed for real:

- **symlink** — `ln -s <parent> ./escape` then write via `./escape/CANARY.md`.
  A relative path defeats the tool guard's absolute-path check.
- **`cd`** — `cd <parent> && echo VIA-CD > CANARY.md`.
- **python** — `python3 -c "open('<parent>/CANARY.md','w').write(...)"`.

## The boundary is the parent *checkout*, not "outside the worktree"

A target entirely outside the repo (`/tmp/probe-outside/target.txt`) was
writable by **both** layers — the `Write` tool allowed it and shell allowed it.
The guard only knows about the shared checkout path. `$HOME`, `/tmp`, and any
sibling repo are unguarded.

## Consequence for doctrine

**Doctrine's jail is not redundant on the claude arm; it is the only real
confinement present.** The harness contributes a tool-layer path check on one
path, defeated by a symlink. `worktree pretooluse`'s bwrap wrap is what actually
scopes the filesystem view — see
[[mem.fact.worktree.gitdir-pointer-unresolvable-in-sandboxed-subagent]] for the
differential that established this, and for the two ways a jailed worker's own
instruments then lie to it.

Do not read a harness worktree block as evidence doctrine's wall fired: they
have distinct signatures. Doctrine denies with a `worktree-jail: ` prefix
(`src/worktree/pretooluse.rs:58`); the harness message names the "shared-checkout
path" and carries no prefix.

Also distinguishable by path: the harness creates its worktrees at
`.claude/worktrees/agent-<id>`, doctrine's `create-fork` at `.worktrees/<name>`
(`WORKTREES_SUBDIR`). The location alone attributes a fork.

## Caveats

Linux/NixOS only; macOS Seatbelt not probed here. Build not pinned — re-probe
before relying on this after a harness upgrade, since this is exactly the kind
of gap a vendor closes quietly.
