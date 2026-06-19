# ISS-031: dispatch claude arm: coordination worktree must live inside the cwd-jail or workers fork main not base B

Discovered during SL-111 dispatch session (2026-06-19), continuing PHASE-03 in a
bubblewrap jail. A **second, independent** precondition for the claude arm, on top
of the cd-into-coord-tree instruction that ISS-029 added.

## Root cause

The claude arm achieves base==B by parking the Bash cwd in the coordination
worktree before spawn — the Agent tool's `isolation: worktree` forks the Bash
cwd's HEAD (`worktree.baseRef='head'`; see ISS-029, `mem_019ec65ecbc7`). That
mechanism **silently fails when the coordination worktree lives outside the
harness's primary working directory.**

In this jail the harness confines the persistent Bash cwd to the primary working
dir (`/workspace/doctrine`). Empirically:

- `cd /workspace/doctrine/<subdir>` (inside) → persists across Bash calls.
- `cd /workspace/doctrine-dispatch-111` (sibling, **outside**) → harness reverts
  cwd to `/workspace/doctrine` on the next call.

`dispatch setup --dir` had been pointed at the outside sibling
`/workspace/doctrine-dispatch-111`. So the cd-into-coord-tree step reverted, the
session cwd stayed on `main`, and an Agent worktree spawn would have forked
`main` (HEAD `9a5bdd7c`) — not base B (`0ff72b6f`). That is exactly ISS-029's
fatal failure mode 2: a worker forked off `main` cannot see prior phases' code
(`kinds.rs` lived only on `dispatch/111`).

## Fix applied (this session)

Relocated the coordination worktree **inside** the project root via
`dispatch setup --slice 111 --dir /workspace/doctrine/.dispatch/SL-111`
(the `.dispatch/SL-<n>` convention already used by SL-093/SL-095). cwd then
parks there persistently; the Agent worker forked base B correctly (verified:
worker commit `S^ == 0ff72b6f`, single commit, 2268 tests green).

## Proposed durable fix

- **`dispatch setup` should default `--dir` to a path under the project root**
  (e.g. `.dispatch/SL-<n>`) and/or **refuse / warn** when `--dir` resolves
  outside the detected primary working directory — fail-closed rather than
  silently produce a wrong-base spawn.
- Document the precondition in the dispatch-agent skill alongside the
  cd instruction.

## Resolved

Fail-closed targeted at the hazard, not blanket. `dispatch setup` now refuses an
outside-root `--dir` **only when a `CLAUDE`-prefixed env signature is present**
(the claude arm); non-Claude arms (codex/pi) keep their enforced outside-root
worktree isolation (ADR-008) untouched — defaulting/forcing inside-root for them
would have discarded it. The harness signal is read in `main.rs` and passed into
`run_setup` as an input (pure/imperative split), so the pure guard
`classify_coord_placement(dir_inside_root, claude_harness)` is unit-testable
independent of the test runner's own (Claude) environment. Precondition documented
in `dispatch-agent/SKILL.md` and the `/dispatch` router. `--dir` left required (no
contract change); the skills carry the `.dispatch/SL-<n>` convention.

## Related

- ISS-029 — the sibling claude-arm base hazard (missing cd instruction). This
  item is the placement precondition that makes that cd actually take effect.
- `mem_019ec65ecbc7` — Agent `isolation: worktree` forks Bash cwd HEAD.
- IMP-072 — WorktreeCreate hook for claude-arm fail-closability (would also stamp
  the marker; see ISS-032 / verify-worker `unstamped`).
