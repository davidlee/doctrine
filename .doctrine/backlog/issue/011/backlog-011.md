# ISS-011: SL-056 SubagentStart hook merge keys identity on command only — stale matcher never healed on reinstall (fail-open unstamped worker)

Source: RV-016 finding F-13 (reconciliation review of SL-056), severity minor / follow-up.

**Scope (widened 2026-06-20):** this item now covers the whole *hook-stamp
reliability* family — both ways the `dispatch-worker` SubagentStart stamp hook
silently fails to fire, leaving an unstamped worker. **Defect A** (original):
stale **matcher** never healed on reinstall. **Defect B** (folded from ISS-034):
stale **command path** carrying a literal `(deleted)` token. Same outcome
(unstamped worker → `verify-worker-refused: unstamped`, or fail-open writes on the
no-env-leg/no-bwrap harness); distinct root causes; sensible to fix together.

## Defect A — stale matcher never healed on reinstall

`src/boot.rs:658-696` — the SubagentStart hook merge keys ownership on the hook
**command** only (the Current-decision merge compares/owns on the command; `set_command`
rewrites only the command). If a `.claude/settings.local.json` already carries a
SubagentStart hook with the right command but a **stale/wrong matcher** (e.g. an old
agent-type literal), a `doctrine claude install` reinstall does NOT heal the matcher → the
stamp hook silently never fires for the dispatch-worker → **fail-open: an unstamped worker
writes freely** on the one harness with no env leg and no bwrap.

`mem.pattern.distribution.hookspec-merge-core-generalized-event-matcher` notes the merge is
generalized over event+matcher; the ownership key should include the matcher.

### Fix (Defect A)

Key the merge identity on `(event, matcher, command)`, or reconcile the matcher on
reinstall so a stale matcher is healed.

## Defect B — stale `(deleted)` command path (folded from ISS-034)

`.claude/settings.local.json` `SubagentStart` had **three** `dispatch-worker`
stamp hooks; **two** invoked
`"/home/david/.cargo/bin/doctrine (deleted) worktree marker --stamp-subagent"` — a
`/proc/self/exe`-style path captured while the running binary had been
rebuilt/replaced, so the install wrote a path with a literal `(deleted)` token.
Those two fail to exec; only the third (clean
`…/doctrine worktree marker --stamp-subagent`) works. Net: workers frequently come
up **unstamped**, so `doctrine worktree verify-worker` refuses with
`verify-worker-refused: unstamped` until the orchestrator hand-stamps via
`echo '{"cwd":"…","agent_type":"dispatch-worker"}' | doctrine worktree marker --stamp-subagent`.

Independent of Defect A, but it compounds friction on every spawn. Discovered
dogfooding `/dispatch` (claude arm) on SL-121, 2026-06-20.

### Fix (Defect B)

Resolve the install-time command path to a stable on-disk binary location, never a
`/proc/self/exe` reading that can carry `(deleted)`; and/or have the merge prune
duplicate/dead SubagentStart stamp hooks (a `(deleted)` command is provably dead)
on reinstall. A `verify-worker` self-stamp on first use would mask the symptom but
not the bad install — fix the writer.

## Defect C — auto-stamp source resolves to the worker worktree (source==fork)

Proven by the IMP-046 fresh-session probe (2026-06-20). Even with a clean,
single, un-poisoned `dispatch-worker` stamp hook, the auto-stamp **never lands a
marker**. `run_stamp_subagent` (`src/worktree.rs:2099`) resolves the copy SOURCE
via `root::find` on the **hook process cwd**, assuming it is the orchestrator tree
(comment at `src/worktree.rs:2110`). Empirically the Claude harness runs the
`SubagentStart` hook with **process cwd = the worker's own worktree**
(`.claude/worktrees/agent-<id>`) — identical to the payload `cwd`. So source==fork
and `verify_sibling_worktree` bails `fork path is the source tree itself; refusing
to provision` (`src/worktree.rs:417`) → unstamped worker.

This is why operators must hand-stamp (Defect B note above): a hand-stamp is run
from the orchestrator cwd, so source ≠ fork and provision succeeds. The hook path
cannot, as written.

### Fix (Defect C)

Do not derive the provision SOURCE from the hook process cwd. Resolve it to the
repo's **primary worktree** (the main checkout — e.g. via `--git-common-dir`'s
parent, or `git worktree list --porcelain` first entry) and pass it explicitly to
`run_provision`, independent of where the hook fires. Probe evidence + harness
finding: `mem.pattern.dispatch.subagentstart-hook-cwd-is-worker-worktree`.

## Related

- **IMP-046** — the fresh-session probe that proved Defect C end-to-end (hook
  fires, matcher matches, payload cwd correct, provision refuses source==fork).
- **ISS-034** — Defect B was first documented there (claude dispatch arm
  isolation/base defect); the hook-stamp half is folded here, the
  isolation/`baseRef:"head"` half stays in ISS-034.
