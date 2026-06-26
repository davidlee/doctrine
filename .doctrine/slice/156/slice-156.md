# Per-worktree CARGO_TARGET_DIR for dispatch workers

## Context

Doctrine's bubblewrap jail shares a single `CARGO_TARGET_DIR`
(`~/.cargo/doctrine-target-jail`, set in `flake.nix`) across ALL worktrees. This
causes a correctness hazard: cargo's incremental fingerprint reuses artifacts
compiled in one worktree when tests run from another, producing false-RED (stale
test binary with old fixtures/deleted tests) and false-GREEN (verify passes
against another branch's artefacts) results.

ADR-008 (accepted) designs the fix: per-worktree `CARGO_TARGET_DIR`
(`.../doctrine-target-jail/wt/<branch>`, D-B1), set at worker spawn. The mechanism
is partially wired:
- `target_dir_for_branch()` (pure mapping, in `src/worktree/shared.rs`)
- `project_env_contract()` (emits `CARGO_TARGET_DIR=.../wt/<branch>`, in
  `src/worktree/fork.rs`)
- `fork_core()` (byte-identical creation core, used by both arms, in
  `src/worktree/fork.rs`)
- `run_fork()` (CLI verb: calls `fork_core` + emits env contract on stdout)
- `create-fork` (Claude `WorktreeCreate` hook: calls `fork_core`, only returns path)

Both arms now share `fork_core` for worktree creation (SL-152 unified the
creation path). The gap is at env injection:

| Arm | Creation | Env contract reaches worker? |
|---|---|---|
| codex/pi (`dispatch-subprocess`) | `run_fork` → `fork_core` | ✅ `$fork_env` captured from stdout, set in subprocess env |
| Claude Agent (`dispatch-agent`) | `create-fork` → `fork_core` | ❌ WorktreeCreate hook only prints path; agent inherits jail-wide `CARGO_TARGET_DIR` |

The dispatched agent inherits the orchestrator's `CARGO_TARGET_DIR` →
shared-target staleness persists for Claude-arm workers.

The dispatch-subprocess skill already passes `$fork_env` to `env`. The
dispatch-agent skill has no corresponding mechanism — the WorktreeCreate hook
can't set env vars for the spawned subagent.

See `research.md` for the full pre-slice evidence dump (memories, code citations,
ADR cross-refs).

## Scope & Objectives

1. **Claude arm: inject per-worktree `CARGO_TARGET_DIR` into the worker's environment.**
   So that Claude Agent dispatch workers compile into their own target dir, not the
   shared jail-wide cache. This is the one gap between the two arms.

2. **Clean up stale-target mitigations** that are no longer needed once per-worktree
   targets are reliable (or document which remain):
   - `just rebuild-stale` (`touch src/main.rs`) — still needed for the orchestrator
     tree? Re-evaluate.
   - Touch-`main.rs` rituals in dispatch verify — retire where per-worktree target
     makes them unnecessary.

3. **Verify end-to-end**: a fresh dispatch worker on either arm builds cleanly
   without cross-worktree artifact thrash, and `just check` / `just gate` in the
   worker tree reports correct pass/fail (no false red/green from stale
   cross-worktree artifacts).

## Non-Goals

- **D-B3 (per-worker bwrap confinement).** The OS-level sandbox is out of scope
  for this slice — a separate slice if the userns probe succeeds.
- **D-B4 (sccache).** Deferred per ADR-008.
- **Host-side `cargo install` / `~/.cargo/bin/doctrine` staleness.** The RO binary
  staleness (`just install` host-side vs jail restarts) is a separate problem
  (MCP config, session-lifetime binary). Not in this slice.
- **codex/pi arm env contract.** Already works — the gap is Claude-arm only.
- **ISS-011 Defects A-C (hook stamp reliability).** Closed/done; not re-litigated
  here.

## Summary

Close the last gap in D-B1: inject per-worktree `CARGO_TARGET_DIR` into Claude
Agent dispatch workers so both arms get correct build isolation. Clean up the
remaining stale-target workarounds that are no longer needed.

## Follow-Ups

- D-B3 bwrap confinement spike (if userns probe passes)
- D-B4 sccache (if cold builds cause pain)
- MCP binary freshness (`.mcp.json` hardcodes the RO binary)
