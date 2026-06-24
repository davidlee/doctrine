# Claude-arm dispatch worker stamps a worker marker on the coord tree (and commits onto the coord branch — expected)

**SL-147 PHASE-05 (2026-06-24), with SL-064's dedicated coordination worktree
(`.dispatch/SL-147`, branch `dispatch/147`).**

Two things a `/dispatch-agent` worker (`Agent` tool, `isolation: worktree`) does
that surprise an orchestrator expecting the codex/pi fork-then-import model:

1. **The worker's commit lands directly on the coordination branch** — NOT on an
   isolated fork. The `Agent` return has **no `worktreePath:` footer**, no
   registered worktree remains, and `dispatch/<N>@{0}` in the reflog is the
   worker's commit. This is EXPECTED claude-arm behaviour (the worktree is created,
   committed in, then collapsed onto the parent), documented in
   [[mem.pattern.dispatch.claude-agent-worktree-integrates-commit-onto-parent]] —
   it persists even now that SL-064 gives the orchestrator a dedicated coord
   worktree (the worker still rides the coord branch, not a separate fork). Do NOT
   read the missing footer as a fallback failure.

2. **The worker's `SubagentStart` hook stamps a worker marker** into the coord
   tree (`<coord>/.doctrine/state/dispatch/worker`, an empty file — see
   [[mem.pattern.dispatch.worker-identity-via-subagent-start-hook]]). This BRICKS
   the coord tree against doctrine-mediated authored writes: the binary then
   refuses `adr status`, `slice status`, etc. with `worker fork (signal: marker):
   refusing authored write`, and any `cargo test` shelling out to the bin fails on
   those (e.g. `e2e_adr_cli_golden`).

## Cure (orchestrator, claude arm)
- Clear the stray marker — the sanctioned self-brick cure, with the linked-worktree
  accident-fence:
  ```
  doctrine worktree marker --clear --operator
  ```
  (Run with Bash cwd inside the coord tree; removes `.doctrine/state/dispatch/worker`.)

## Verify the LANDED commit (the funnel runs post-landing here)
Because the commit is already on the coord branch, the R-5 belt + verify gate
necessarily run AFTER it lands. Trust these checks, not the worker's self-report:
- `git diff --name-only B..HEAD` ⊆ declared files; **no `.doctrine/`/`.claude/`** (R-5).
- `git rev-parse HEAD^ == B` (single non-merge, `parent == B`).
- `just gate` green on the coord tree — but FIRST defeat the stale-bin footgun
  (shared `CARGO_TARGET_DIR` serves a stale `doctrine` bin; `cargo build` may
  fingerprint-skip across worktrees): `touch src/<changed>.rs && cargo build` to
  force a real recompile before trusting any test that shells out to the bin.

If those hold, ACCEPT the commit in place — a reset/re-dispatch only risks the tree
for an already-correct delta. See [[mem.pattern.dispatch.agent-worktree-forks-bash-cwd-head]]
for placing the worker at B (cd the Bash cwd into the coord tree before spawn).
