# Claude-arm subagent write-confinement hooks

## Context

ADR-008 conceded that "a generic harness cannot confine a worker to its
worktree (observed: Claude Code creates the worktree but lets the agent write to
main)" and deferred the raw-tree confinement of ADR-006 D2b to sandbox/harness
work. On the claude `Agent` arm there has been **no real isolation** — worker-mode
is a cooperative flag, trivially defeatable (RSK-014).

RSK-014 probe-h1 **proved** (claude-code 2.1.181, NixOS, bwrap 0.11.2) that a
`PreToolUse` hook can impose hard write-containment on an `isolation: worktree`
subagent. Two walls, both load-bearing:

- **Bash wall** — `PreToolUse(Bash)` rewrites the command via `updatedInput`,
  opaquely (base64) wrapping it in a nested bwrap jail: rw-bind only the
  subagent's worktree, `--ro-bind / /` everything else. Native gives **zero**
  Bash containment (necessity-controlled) — the wrapper is the whole wall.
- **Edit/Write wall** — native confines subagent Edit/Write **only to the shared
  repo checkout**; `/tmp`, `$HOME`, `/etc`, ssh keys… stay writable. A
  `PreToolUse(Edit|Write)` pathcheck (`realpath(file_path) ⊆ cwd`, fail-closed)
  closes the rest. Both layers required; NOT redundant.

Binding is deterministic: PreToolUse stdin carries `agent_id` (present **iff**
subagent) and `cwd` == hook pwd == the worktree, concurrency-stable. Fail-closed
rule: pass through ⟺ `agent_id` absent (orchestrator); `agent_id` present ⇒ `cwd`
must resolve to a recognised worktree or **deny**.

This slice **graduates the proven apparatus** from `.harness/probe/` (working
scripts live at `.doctrine/backlog/risk/014/probe-h1/`) into the installed
doctrine skill hooks, so claude-arm dispatch workers are confined by construction
on the Linux/bwrap arm — closing the ADR-006 D2b / ADR-012 OQ-D impersonation gap
for real (not the cooperative marker).

Recipe + evidence: `mem.pattern.dispatch.claude-worktree-subagent-bwrap-confinement`,
`.doctrine/backlog/risk/014/probe-h1/`.

## Scope & Objectives

1. **Bash wall** — land the `PreToolUse(Bash)` bwrap wrapper as an installed
   doctrine hook. Opaque (base64) command wrap; rw worktree, ro everything else;
   anchor to `cwd`. Fail-closed: unresolved worktree / missing bwrap / parse error
   → deny. Pass through ⟺ `agent_id` absent.
2. **Edit/Write wall** — land the `PreToolUse(Edit|Write)` pathcheck
   (`realpath ⊆ cwd`, fail-closed). Pin the repo-root deny by the pathwall's own
   rule (ancestor-of-cwd), not by leaning on native's race-win.
3. **Per-run config surface + worker correlation (OQ-3, in scope).** A way to
   tune a run's jail (extra rw/ro binds, network, strict/loose) and correlate that
   config with the *specific* worker. The binding key exists (`agent_id` and/or
   `cwd`==worktree). The orchestrator (ADR-006 sole-writer) declares per-worker
   policy at spawn time, written to a path **outside every worktree**; the hook
   looks it up by `cwd`. To resolve in design: where the map lives, its schema,
   authority, and lifecycle/GC alongside the worktree. A fixed default policy
   (rw worktree, ro rest) is the floor; the config surface tunes from there.
4. **Install/reload contract.** Hook *registration* loads at session start only —
   no hot-reload. The install path + restart ritual must be designed in (and the
   orchestrator escape hatch: Edit/Write are not Bash-gated, so a broken Bash
   wrapper can always be disabled and the session restarted).

## Open Questions (resolve in /design)

- **OQ-A (was OQ-2) — altitude.** Rust `doctrine` subcommand (e.g. `doctrine
  worktree pretooluse`) vs installed bash scripts. The hook fires on **every**
  tool call; per-call `bash + jq` spawn cost adds up. DRY pull: hooks already
  shell `doctrine boot --emit` and `worktree create-fork`. Lean Rust subcommand;
  measure startup cost both ways before locking.
- **OQ-B (was OQ-4) — shared-`.git` posture.** The wrapper ro-binds **all** of
  `.git` (battery vectors 5–7 confirmed refs/config/hooks denied). For a linked
  worktree its git metadata lives under `<main>/.git/worktrees/<name>`, also ro →
  the worker **cannot self-commit**, which is exactly the established funnel
  (orchestrator imports the working-tree diff). **Question to resolve:** keep all
  `.git` ro (strongest, simplest invariant — recommended default), or selectively
  loosen (rw `.git/objects` + `.git/worktrees/<name>`, keep refs/packed-refs/
  config/**hooks** ro) if worker self-commit is ever wanted. Recommendation:
  don't loosen without a concrete driver. **Interacts with SL-181** — on the
  bwrap arm the ro-`.git` wall makes SL-181's ref-corruption guard
  belt-and-suspenders; SL-181 stays sole protection on unconfined arms.

## Non-Goals

- **macOS sandbox-exec / Seatbelt** — owned by **IMP-045** (cross-platform jail
  seam). Until built, the hook denies worktree-subagent Bash on non-bwrap
  platforms; never silent pass-through. macOS posture documented (allow-default /
  deny-write-except mirroring bwrap) but not implemented here.
- **OQ-5 — jail the orchestrator too** (fail-open structural close: wrap the
  orchestrator with the widest jail). Hardening follow-up; deferred.
- **OQ-6 — socket-reachable privileged peers** (postgres `COPY … TO PROGRAM`,
  nix-daemon). The honest bwrap residual; per-project net-ns/socket hygiene.
  Deferred; not closable by the jail itself.
- **Reads / network egress** — out of scope; the walls are write-containment only
  (exfil + egress remain separate, unaddressed walls).
- **The cooperative marker / SL-181's anti-accident guard** — separate slice;
  this slice is the *hard* enforcement path, not the cooperative one.

## Summary

Graduate the proven two-wall confinement apparatus (`PreToolUse(Bash)` bwrap
wrapper + `PreToolUse(Edit|Write)` pathcheck) from probe scripts into installed
doctrine skill hooks, plus a per-run jail config surface keyed on the worker
binding (OQ-3). Linux/bwrap arm; fail-closed when bwrap absent. Closes the
claude-arm holes of RSK-014 / ADR-006 D2b on its arm.

## Follow-Ups

- IMP-045 — macOS sandbox-exec backend for the same seam.
- OQ-5 — orchestrator-jail (widest-jail pass-through).
- OQ-6 — socket-reachable privileged-peer residual (per-project hygiene).
- STD-001 follow-up if any magic strings emerge (hook paths, bind flags).
