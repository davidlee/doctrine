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

## Design pivot (2026-06-26 — see `design.md`)

The original framing above ("inject env into the Claude arm") was **inverted by
two findings**:

1. **No Claude hook can inject env into a worker** (Probe 2,
   `mem.fact.dispatch.claude-worker-no-per-worktree-env`). The injection approach
   is impossible on the claude arm.
2. **POL-002.** `project_env_contract`/`coordinate`/`gc` hardcode `CARGO_TARGET_DIR`
   in the *shipped platform* — a host-convention coupling the policy forbids. The
   codex arm's `$fork_env` is that violation; extending it to claude would deepen it.

So the design **flips the mechanism** (B1): **retire** the jail-wide shared
`CARGO_TARGET_DIR` export, letting each worktree default to its own in-tree,
gitignored `target/` — per-worktree isolation correct by construction, both arms,
no env channel. The **platform exits the build-env business** (per-worktree build
target becomes a project concern). Requires an **ADR-008 Revision** (D-B1/D-B5
mechanism change). Full rationale, alternatives, and validation in `design.md`.

## Scope & Objectives

1. **Retire the shared `CARGO_TARGET_DIR` (project layer).** Remove the
   `flake.nix:80` export so every worktree builds into its own `<worktree>/target`.

2. **Make the platform build-tool-agnostic (POL-002).** Remove the cargo coupling:
   `project_env_contract` + its emission in `run_fork`/`coordinate`, and the
   `gc.rs` target-base reaping. Migrate the `dispatch-subprocess` skill off `$fork_env`.
   Order: project-side first → codex-skill migration → platform removal last (R2).

3. **Clean up stale-target mitigations** the shared target made necessary:
   `just rebuild-stale`, the `justfile`/`AGENTS.md` staleness guidance, touch
   rituals, and the superseded shared-target memory cluster.

4. **Verify**: two worktrees build isolated (no cross-thrash); both arms' `just
   check`/`just gate` report honest pass/fail; no `CARGO_TARGET_DIR`/cargo literal
   remains in shipped `src/`.

## Non-Goals

- **D-B3 (per-worker bwrap confinement).** Out of scope — a separate slice if the
  userns probe succeeds.
- **D-B4 (sccache).** Deferred per ADR-008; the warm-fork-cache lever if cold
  builds bite (this slice accepts cold fork builds).
- **B2 (persisted `DOCTRINE_CARGO_TARGET_ROOT` + token + project GC).** Rejected in
  favour of B1 (design.md D1); revisit only via D-B4.
- **Host-side `cargo install` / `~/.cargo/bin/doctrine` staleness** and **MCP binary
  freshness** (`.mcp.json` hardcodes the RO binary). Separate problems.
- **ISS-011 Defects A-C (hook stamp reliability).** Closed/done; not re-litigated.

## Summary

Per-worktree build isolation by **retiring** the shared `CARGO_TARGET_DIR` (in-tree
`target/` per worktree, both arms) and **removing the cargo coupling from the
shipped platform** (POL-002). Mechanism change from ADR-008 D-B1/D-B5 → routed via
a Revision. Clean up the stale-target workarounds the shared target required.

## Follow-Ups

- D-B3 bwrap confinement spike (if userns probe passes)
- D-B4 sccache (if cold builds cause pain)
- MCP binary freshness (`.mcp.json` hardcodes the RO binary)
