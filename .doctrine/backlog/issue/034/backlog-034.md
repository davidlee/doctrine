# ISS-034: Claude dispatch arm: isolation:worktree falls back to main under shared-clone lock contention, forking wrong/moving base

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Summary

The `/dispatch` **claude arm** (`/dispatch-agent`) relies on the Claude Code
`Agent` tool's `isolation: worktree` to give each worker a **stable** fork of the
coordination tip (base==B). Under a **shared single clone with multiple concurrent
agents**, that assumption breaks: worker-worktree creation intermittently races on
git's repo-global locks and the subagent **silently falls back to the main
worktree** (`/workspace/<repo>`). Because the worktree config is
`worktree.baseRef: "head"`, the fork then tracks whatever HEAD is live in the main
tree — and `main` is moving under the other agents — so the worker executes on a
**wrong, dirty, moving base** instead of B.

This is the highest-value arm of the parallel-work model (claude workers, not
codex/pi subprocesses), and the failure is **structural**, not a one-off: in a busy
shared repo `main` always moves, so the arm is unreliable whenever isolation races.

Discovered dogfooding `/dispatch` on **SL-121** (claude arm) on 2026-06-20.

## Severity / impact

- **Correctness-critical but currently fails *closed*.** Every wrong-base spawn in
  the SL-121 run was caught by a worker **base-guard** (a prompt preamble that
  greps for the prerequisite seams + checks `git status` clean and STOPs on
  mismatch). All three failed workers wrote **nothing** — no corruption, no commit
  on the wrong base. **The safety depends entirely on that guard being present.**
  Without it, a worker would author a phase atop `main` (or a stale base), commit
  it on a `worktree-agent-*` branch, and the funnel's `verify-worker`/delta checks
  would be the only remaining backstop.
- **Throughput:** the arm is effectively unusable while any other agent is
  committing to `main`. SL-121 PHASE-02 could not be landed on the claude arm at
  all (3 consecutive wrong-base spawns).

## Evidence (SL-121 run, 2026-06-20)

Coordination worktree: `.dispatch/SL-121` on branch `dispatch/121`, base
`587d4403`. Funnel committed PHASE-01 (`b902c2dd`) then PHASE-04 (`bbfc90bb`) onto
`dispatch/121` cleanly.

- **PHASE-01 + PHASE-04 succeeded** — workers returned with a `worktreePath:` /
  `worktreeBranch:` footer; the worktrees were real isolated checkouts on their own
  `worktree-agent-*` branches (verified: `.git/worktrees/agent-a53a8bde…/HEAD` →
  `ref: refs/heads/worktree-agent-a53a8bde…`, HEAD = the worker's own commit).
  **Both ran while `main` sat still at `9861c8d2`.**
- **PHASE-02 failed 3×** — each coincided with `main` advancing under other agents
  (`9861c8d2` → `5be3f6b6` → `63d96f90`):
  - Attempts #1/#2 returned with **no `worktreePath:` footer at all** → no isolated
    tree was created; the worker ran in the main worktree. Reported base
    `5be3f6b6` on branch `main`; #2 additionally saw main's uncommitted
    `src/coverage_store.rs` / `src/coverage_verify.rs` / `src/requirement.rs`.
  - Attempt #3 was the decisive one: its **first** reads showed the *correct*
    isolated base (`bbfc90bb`, tree clean, PHASE-01/04 seams present) — then,
    **moments later in the same worker**, `git rev-parse HEAD` returned
    `63d96f90` ("Revert 'fix(boot): rename Pi harness to Codex…'", a `main` commit),
    the tree was now **dirty with main's `coverage_store.rs`**, and
    `git merge-base --is-ancestor bbfc90bb HEAD` → **false**. The worker's working
    directory had become the main worktree mid-run, and `main` had moved under it.

The clean correlation: **worker tree is clobbered to `main` exactly when `main`
moves during the run.** When `main` is static (PHASE-01/04) the isolated tree
holds; when `main` moves (PHASE-02) the worker lands on it.

## Root cause (two distinct defects)

### Defect A — isolation race → fallback-to-main, amplified by `baseRef: "head"`

Many agents share **one** clone (`git worktree list` showed 80+ live worktrees,
churning). Concurrent `git worktree add` / `prune` / `checkout` across agents
contend on git's repo-global locks (`index.lock`, `HEAD.lock`,
`.git/worktrees`). When a worker's `isolation: worktree` setup loses that race, the
subagent does **not** hard-fail — it proceeds in the **main worktree**
(`/workspace/<repo>`). Combined with `worktree.baseRef: "head"`
(`.claude/settings.local.json`), the worker's notion of base is then "whatever HEAD
is in the main tree right now", which other agents are advancing. Net: wrong base,
moving base, dirty base.

This is the same family as **ISS-031** (jail `cd` silently reverts to project root
→ worker forks `main` not B) — the failure mode is "claude worker silently ends up
on `main`", reached here by lock-race rather than by an outside-root `cd`.

### Defect B — `SubagentStart` stamp hook points at a deleted binary

`.claude/settings.local.json` `SubagentStart` has **three** `dispatch-worker`
hooks; **two** invoke `"/home/david/.cargo/bin/doctrine (deleted) worktree marker
--stamp-subagent"` — a stale `/proc/self/exe`-style path with a literal
`(deleted)` token (the binary was rebuilt/replaced after the path was captured).
Those two fail; only the third (`…/doctrine worktree marker --stamp-subagent`)
works. Result: workers frequently come up **unstamped**, so
`doctrine worktree verify-worker` refuses with `verify-worker-refused: unstamped`
until the orchestrator hand-stamps via
`echo '{"cwd":"…","agent_type":"dispatch-worker"}' | doctrine worktree marker --stamp-subagent`.
Independent of Defect A, but it compounds the friction on every spawn.

## Why it matters to the model

The claude arm is the preferred arm (in-harness workers, no subprocess). Its
correctness rests on `isolation: worktree` delivering a **stable** fork pinned to
the coordination tip for the worker's whole lifetime. Under real multi-agent load
on a shared clone, `baseRef: "head"` makes the fork **track a moving `main`**
whenever isolation races, so "base==B for the worker's lifetime" does not hold. The
arm is only reliable today when nothing else touches `main` — which is not a
condition we can assume in the dispatch use case.

## Candidate remedies (for triage — not yet decided)

1. **Pin the worker base, don't track `head`.** Set the fork base to the
   coordination ref (`dispatch/<slice>`) or an explicit SHA rather than
   `baseRef: "head"`. Serial phases already self-base (each funnel commit advances
   `dispatch/<slice>`), so a pinned `dispatch/<slice>` base carries all prior
   phases and is immune to `main` moving. *Open Q:* does Claude Code's
   `worktree.baseRef` accept a branch/SHA, or only `"head"`? And `settings.local.json`
   is repo-wide — changing it affects every agent, so it can't be flipped
   unilaterally mid-flight.
2. **Make the base-guard a first-class part of the `dispatch-worker` template.**
   The prompt-level guard (grep prerequisite seams + assert clean tree + STOP on
   mismatch) is what kept all 3 SL-121 failures non-destructive. Today it is added
   ad hoc per spawn; it should be standard so the arm always fails closed.
3. **Harden `verify-worker` to detect the fallback explicitly** — e.g. refuse when
   the worker's worktree path is the main worktree, or when
   `merge-base --is-ancestor B HEAD` fails — turning a silent wrong-base into a loud
   funnel halt even absent the prompt guard.
4. **Lock-retry / backoff on worktree creation** (harness-level) so a lost race
   retries instead of falling back to the main tree.
5. **Fix Defect B:** repair the two `SubagentStart` hooks' `doctrine (deleted)`
   paths (point at the live binary), or have `verify-worker` self-stamp on first
   use.
6. **Isolation alternative:** the **subprocess arm** (`/dispatch-subprocess`,
   `doctrine worktree fork --worker`) forks deterministically and binds the
   subprocess cwd to the fork — no fallback-to-main. It shares the same git-lock
   contention but as a single, doctrine-controlled, retriable op. Robust today, at
   the cost of codex/pi workers instead of claude.

## Reproduction

1. In a shared clone with ≥1 other agent actively committing to `main`, run
   `/dispatch` (claude arm) on a slice with ≥2 serial phases.
2. Land phase 1 (funnel commits to `dispatch/<slice>`).
3. Spawn phase 2 while another agent commits to `main`.
4. Observe the worker report a base on `main` (or a base that mutates mid-run),
   often with no `worktreePath:` footer and/or unrelated dirty files.

## Related

- **ISS-031** — same "claude worker silently on `main`, not B" outcome via the
  jail `cd`-revert path; this is the lock-race sibling. The `dispatch-setup`
  fail-closed for outside-root `--dir` does not cover this case (the `--dir` here is
  correctly inside-root `.dispatch/SL-121`).
- **SL-121** — the slice under dispatch when this surfaced; PHASE-01 + PHASE-04
  landed on `dispatch/121` @ `bbfc90bb`, PHASE-02/03 blocked by this issue.
- `mem_019ec65ecbc7` — the controlled probe establishing that `isolation: worktree`
  forks off the Bash cwd HEAD (the assumption this issue shows is violated under
  contention).

## Status of the SL-121 work (context, not part of this issue)

No data lost. PHASE-01 + PHASE-04 are committed and intact on `dispatch/121`
(`bbfc90bb`); the coordination tree is clean. PHASE-02 + PHASE-03 remain to land
(awaiting a decision on arm: subprocess vs. pause-and-retry-claude).
