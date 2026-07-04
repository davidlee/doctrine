<!-- Shipped reference (ADR-005 PULL tier). Edit the source in
     `install/dispatch-mechanics.md`; the installed copy at
     `.doctrine/dispatch-mechanics.md` is inert. Explains the fork→land funnel and
     its sharp edges — it never reproduces `doctrine --help`; ask the CLI for exact
     flags. Distilled from project-local dispatch memories (CHR-036). -->

# Dispatch mechanics

How the `/dispatch` funnel actually moves code — the fork→verify→import→land
pipeline, the git-plumbing invariants it rests on, and the failure modes that
have cost real respawns. This is the *explain* tier: read it once to build the
mental model. For the sharp mid-operation traps you hit *while* driving a
dispatch, retrieve `mem.signpost.doctrine.dispatch`; for command shapes ask
`doctrine <command> --help`.

The orchestrator is the sole writer. Workers execute one phase inside an
isolated worktree and hand back a single source-delta; the orchestrator imports
that delta, verifies, and commits. Everything below protects that contract.

## The fork base is explicit, never the session HEAD

A worker's fork **must** be created from the explicit coordination base `B` —
the orchestrator's coordination-branch HEAD captured pre-spawn — never from the
implicit current/session HEAD.

The session repo may sit on a different branch (e.g. `main`) than the
coordination branch, so **session HEAD ≠ `B`**. A fork that inherits the
implicit HEAD lands on a divergent base: `S.parent != B`, and the net diff
`B..S` then smuggles the session↔coordination divergence into the import —
unrelated commits ride into the wrong slice's delta.

Two belts enforce it:
- **Worker baseline guard.** Immediately after `git worktree add <dir> <branch>
  <B>`, assert `git -C <dir> rev-parse HEAD == B` and abort otherwise.
- **Orchestrator import guard (trusted side).** Assert `git rev-parse S^ == B`
  before applying the delta — catches a misbased fork even if the worker skipped
  its own guard.

Harness trap: some spawn backends build the fork from the session HEAD and give
no reliable base control (the claude `Agent` tool at `isolation: worktree` is
one). Where the backend won't honour `B`, spawn a plain agent that self-forks
from `B` explicitly rather than trusting the backend's isolation.

## Verify scope: never run the project-wide gate in the funnel

Funnel and worker verification must be **scoped to the touched files**, not the
whole tree. A project-wide format gate (e.g. `cargo fmt` inside `just gate`)
reformats in place across the crate and pulls unrelated pre-existing drift into
the worker's delta — a base commit that isn't format-clean under the pinned
toolchain will smuggle format churn into every slice.

Scope it: lint + test + a `--check`-only formatter over the touched files. Prove
the phase, don't reformat the world.

## The import severs ancestry — so "did it land?" needs a patch-id oracle

The funnel imports a worker's delta with a non-committing 3-way apply onto `B`,
then the orchestrator commits separately. This **severs git ancestry**: the fork
branch `S` is never an ancestor of the coordination commit. Every naive
landed-oracle is therefore unsound:

- `git branch --merged` — the apply-funnel branch is never merged, always
  reported unmerged (and `git branch -d` always refuses it; deletion needs `-D`).
- **Delta-emptiness** (`git diff B..fork` empty ⇒ landed) — `B..fork` is the
  whole worker delta, never empty for real work, so it refuses every fork. And
  the moment a sibling moves the coordination HEAD, `HEAD..fork` legitimately
  diverges ⇒ non-empty ⇒ also refuses a spent fork. Either way the operator
  learns a `--force` reflex and the safety gate collapses.
- **A runtime-tier "import receipt"** stamped on apply-success — unsound too. It
  certifies the *apply*, not the *commit*: it is born before the separate commit,
  lives in disposable state, and survives a crash-before-commit — reading
  "landed" when no commit ever reached the branch, so a recovery-time cleanup
  reaps the only surviving copy of unmerged work. A flag in disposable state must
  never gate an irreversible `branch -D`.

**Sound oracle: a durable patch-id check.** Run `git cherry <coordination-HEAD>
<fork-branch>` and treat the fork as landed **only when every commit in its
`B..fork` range is marked `-`** (its patch is already present in coordination's
history by patch-id). Any `+` ⇒ not fully landed ⇒ refuse. This is keyed on
durable git state *after* the commit, so it is crash-proof (a crash before the
commit leaves no landed patch ⇒ `+` ⇒ refuse), robust to a sibling moving HEAD
(patch-id matches the commit's patch, not a whole-tree diff), and robust to the
apply severing ancestry (patch-id ≠ ancestry). Ranging over *all* commits (not a
single tip) lets one oracle serve both the single-commit dispatch fork and the
multi-commit solo fork.

### The squash blind spot

The patch-id oracle **cannot** distinguish a squash-merged fork from one that
never landed. A multi-commit `git merge --squash` produces one squash commit
whose patch-id matches none of the fork's individual commits, so `git cherry`
lists every commit `+` and the tip is not an ancestor — byte-for-byte the signal
of a fork that never landed. Do not build a squash detector; it cannot exist.
Collapse squash + never-landed into one `not-landed` refusal whose message names
both remedies. This is the load-bearing reason a solo fork must land via a
non-squash (`--no-ff`) verb: squash destroys the oracle.

## Landing on a shared trunk races — report and halt, never auto-merge

Integrating onto a live shared trunk is fast-forward-only plus an expected-tip
compare-and-swap. On a trunk where other agents commit concurrently, two
failures bite — both report-and-halt by design:

1. **Trunk moved mid-command.** The admitted target's base went stale between
   "create candidate" and "integrate". Fix: re-create the candidate superseding
   the prior on the new base, re-admit, re-integrate. To shrink the race window,
   chain create→admit→integrate in one shell invocation (the candidate ref name
   is deterministic, so nothing needs threading). Expect retries under churn.
2. **Dirty-worktree refusal.** Integrate resyncs the live checkout and refuses a
   blanket-dirty tree — even when the dirty file is another slice's authored WIP
   that cannot conflict with the projected code. You may not stash or discard
   another agent's uncommitted work. Resolution is the work's owner committing
   it, then re-superseding (their commit advanced trunk) and integrating. The
   driver cannot self-unblock — surface it and wait.

The trunk ladder defaults to the remote's `origin/HEAD`, which lags a local
trunk; point dispatch verbs at the intended local trunk ref explicitly.

## Worker-spawn identity is accident-fenced, not fail-closed

A `SubagentStart`-style spawn hook that stamps a worker-identity marker runs
synchronously (the marker is present before the worker's first command *when the
hook succeeds*) but is **read-only** — it cannot abort the subagent on failure.
On a stamp failure the worker proceeds unstamped and un-gateable by the hook. So
worker identity must be fenced by the **import belt + a worker-mode env guard +
the pre-distilled prompt**, never by the hook's exit status. The only
fail-closed-capable creation seam is a worktree-creation hook (non-zero exit
aborts creation), which is preferable where the harness exposes it with enough
payload to act on.

## Workers can silently discard their own work

A worker may build a phase correctly (tests green) and then `git reset` /
`checkout -- ` / `stash` / `clean` the entire delta away — hallucinating a
"pre-existing WIP", reverting its own edits, and never committing. The fork comes
back clean (HEAD == B, empty diff). Fence it at the prompt:
- State the fork is a **clean** checkout with **no** WIP and that the target
  files do not yet exist.
- Forbid every work-discarding git verb; the only git the worker runs is the
  final `git add <paths>` + `commit`.
- For a red-proof reversion (TDD), instruct it to *edit* the scratch out, never
  to git-discard it.

And never trust the worker's self-reported success — the fork's committed git
state is the ground truth. Re-read the commit, re-run the suite; distrust the
handover's own green/failure labels.

## Subprocess-arm RPC hygiene (codex/pi)

When the worker is a subprocess speaking a line-oriented RPC:
- **One compact JSON object per line.** A pretty-printed prompt message emits
  multi-line JSON and every line fails to parse — the prompt never lands and the
  worker sits idle. Build RPC lines compact (single-line).
- **The process may park on success, not self-exit.** It emits a completion
  event but blocks on stdin, so a timeout-bound spawn burns the full timeout even
  though work finished in minutes. Run a watcher that polls the log for the
  completion event and kills the process on match, so the spawn returns at
  completion rather than at timeout.

## See also

- `mem.signpost.doctrine.dispatch` — the retrieval index for the sharp
  mid-operation traps (retrieve *during* a dispatch, not up front).
- ADR-006 (worktree posture), ADR-008 (jail isolation), ADR-011 (harness-agnostic
  spawn), ADR-012 (integration topology) — the decisions behind this machinery.
- `doctrine dispatch --help`, `doctrine worktree --help` — exact command shapes.
