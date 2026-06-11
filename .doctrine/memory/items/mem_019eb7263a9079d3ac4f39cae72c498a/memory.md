# Dispatch worker must fork rung-3 from explicit base B, never inherit session HEAD

A dispatch worker's fork **must** be created from the explicit coordination base
`B` (the orchestrator's coordination-branch HEAD), never from the implicit
session/current HEAD.

**Why (harness-agnostic).** The orchestrator drives the coordination branch while
the session repo may sit elsewhere (e.g. `main`), so **session HEAD ≠ `B`**. A fork
that inherits the implicit current HEAD lands on a divergent base: `S.parent != B`,
and the net diff `B..S` then smuggles the session↔coordination divergence into the
import — unrelated commits land in the wrong slice's delta. Proven in SL-042
PHASE-01: `main`'s slice work would be dragged into a sibling slice's delta
(coord diverges from main; e.g. merge-base behind both).

## Do

- Fork via worktree rung 3, base pinned explicitly:
  ```bash
  git worktree add <dir> <branch> <B>            # <B> = coordination HEAD, NOT current HEAD
  git -C <dir> rev-parse HEAD                     # baseline guard: MUST equal <B>, else abort
  ```
- Pass `base=<B>` into `/worktree mode=worker`; the orchestrator captures
  `B = git rev-parse HEAD` on the coordination branch pre-spawn.
- Orchestrator import belt (trusted side): assert `git rev-parse S^ == B` before
  applying the delta — catches a misbased fork even if the worker skipped its
  baseline guard.

## Claude-Code-specific trap

The `Agent` tool at `isolation: worktree` builds the fork from the **session HEAD**
(not `B`) and gives no reliable isolation here — the concrete way the implicit-HEAD
trap bites under Claude Code. Spawn a **plain** `Agent` that self-forks rung-3
instead. Harnesses without such a backend can ignore this; the rung-3 rule above
already protects them.

Related: [[mem.pattern.dispatch.authoring-entities-not-dispatchable]],
[[mem.concept.dispatch.gitignored-tier-partition]]. SL-042 notes carry the
PHASE-01 worktree-fragility gotcha (coordination branch `sl-042-coord`).
