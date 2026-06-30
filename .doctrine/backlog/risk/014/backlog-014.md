# RSK-014: Claude subagent confinement is unsolved — worker-mode is cooperative-only off the bwrap arm

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Tracks the standing reality that doctrine's dispatch **worker-mode marker is a
cooperative flag, not an enforced boundary**, on every spawn arm except one.
Surfaced while scoping SL-181 / IMP-065 (the positive coordination marker).
ADR-006 D2a already concedes it ("the fence … is the funnel + the jail, **not** a
fail-closed CLI floor"); this RSK names the residual explicitly so it is tracked
until closed conclusively rather than re-rediscovered per slice.

## The state of play

Worker identity = a presence-only file `.doctrine/state/dispatch/worker` (primary)
+ optional `DOCTRINE_WORKER=1` env (codex/pi). **Both are trivially defeatable by
the agent they are meant to constrain** — env is set/unset at will; an unprotected
file is written / deleted / `cd`-ed around at will. They stop an *honest* worker
that respects the CLI; they stop a *capable* one not at all.

Confinement altitude, by arm:

| arm | real isolation? | notes |
|---|---|---|
| **claude `Agent` (`isolation: worktree`)** | **none** | cooperative only; worker can `cd` into the coord/main tree, write anywhere, forge any marker. No env seam to the subagent (so even the env leg is absent). |
| **vanilla pi subagent (in-pi)** | **none** | cooperative only; same as claude. |
| **pi subprocess via `scripts/pi-spawn-confined.sh`** | **yes (bwrap)** | `--ro-bind / /` + rw worktree + ro marker + no push. The *only* arm where "outside the worktree is read-only" is a fact, not a request. Linux-only. |

## Why it isn't trivially closable

- **ENV** — worthless as a hard stop (agent controls its own env).
- **Unprotected file** — worthless without an accompanying read-only filesystem
  outside the worktree (i.e. bwrap).
- **bwrap** — works, but **linux-only**; no vetted seatbelt/`sandbox-exec` (macOS)
  equivalent identified yet.
- **`claude -p` jailed** — works technically, but Anthropic bills `-p` at Console
  API rates, not the interactive subscription, making fan-out economically
  untenable (forces API pricing). Recorded:
  [[mem.pattern.dispatch.spawn-backend-harness-agnostic-no-free-env-seam]].

## The candidate enforcement paths for claude subagents (unsolved / unexplored)

1. A **PreToolUse hook** keyed on a *reliable hard environmental cue* to constrain
   writes — **AND** disallowing the `Bash` tool (which trivially circumvents any
   tool-level gate). Blocker: no hook/cue hard enough to serve as the constraint
   has been found.
   - **Active probe (H1):** rather than *disallow* Bash, opaquely *rewrite* it via
     `PreToolUse(Bash).updatedInput` to wrap the command in a nested `bwrap` jail —
     rw-bound to only the spawning subagent's worktree, everything else ro. Closes
     the Bash hole the tool-level gate leaves open, if the harness exposes a
     deterministic worktree binding at hook time (`cwd` or identity↔`WorktreeCreate`
     correlation) and honours `updatedInput`. Falsification-first brief +
     experiments: [`probe-brief-h1-pretooluse-bwrap.md`](./probe-brief-h1-pretooluse-bwrap.md).
     Two-wall caveat: validates the **Bash** wall only; Edit/Write need a separate
     `PreToolUse(Edit|Write)` path-check; reads/egress out of scope.
   - **PROVEN (2026-06-30, claude-code 2.1.181, NixOS bwrap 0.11.2).** H1a/b/c/d
     all held. Binding is deterministic: PreToolUse stdin carries `agent_id` (iff
     subagent) and `cwd` == hook pwd == the worktree (concurrency-stable). The
     live harness honours `updatedInput`; the full 11-vector escape battery
     (absolute / traversal / symlink / hardlink / shared-`.git` / `/proc` alias /
     child-proc / detached job / `$HOME`) was contained by `Read-only file system`.
     Apparatus + raw logs + verdict: [`probe-h1/`](./probe-h1/) (`results.md`).
   - **Edit/Write wall — PARTIALLY native, needs our hook for the rest.** This
     harness natively confines subagent Edit/Write **only to the shared repo
     checkout** (a Write to a repo-root path is denied before any hook runs —
     contradicts the brief's "native covers Bash only" claim). But a second probe
     proved coverage is partial: Writes to `/tmp` and `$HOME` were NOT caught by
     native and were denied by **our** `PreToolUse(Edit|Write)` pathcheck (deny
     branch proven to enforce in-situ). So full Edit/Write containment needs BOTH
     layers: native (repo) + our pathcheck (entire non-repo filesystem). NOT
     redundant. Net: this path is a **viable real close** of the claude-arm holes
     — NEW machinery = the bwrap Bash wrapper + the Edit/Write pathcheck; graduate
     `.harness/probe/` into the doctrine skill hooks under a slice to land it.
     Linux/bwrap only (macOS still open — see IMP-045 seatbelt seam).
2. A **gatekeeper-agent** enforcement model (auto-mode or a custom variant) that
   adjudicates the worker's actions.
3. Manually-started sessions inside a worktree with reduced permissions, coordinated
   via the Agent Teams / channels API. Blocker: unless session context is fully
   controllable along that path (`/new`, `/compact`, …), it degrades to a weak
   agent-babysitting DX.

## Mitigating reality (why anti-accident checks still earn their keep)

Anthropic models are among the best-aligned against *malicious compliance*, so
**anti-accident** checks (e.g. SL-181's cheap ref-corruption guard) buy more real
safety than the cooperative-flag critique alone would suggest — an honest worker
that respects the CLI is the common case, not the adversarial one. This does **not**
make the marker a boundary; it means the residual is "capable-but-misbehaving
worker," which is rarer with well-aligned models. Worth tracking until a genuine
enforcement path lands.

## Don't-lose

- The genuine close of the dispatch impersonation gap (ADR-011 D6/M2, ADR-012
  OQ-D) is **confinement**, not a marker. SL-181 only upgrades the *cooperative*
  path; the REV against ADR-012 retracts the "positive marker is the real close"
  framing and points here.

## Considerations for the landing slice (H1 proven → graduate `probe-h1/`)

Captured for the eventual slice that lifts the wrapper + pathcheck from
`.harness/probe/` into the doctrine skill hooks. Design OQs, not yet decided:

- **OQ-1 bwrap detection & platform fallback.** bwrap is a separate dependency
  and Linux-only. The hook must *detect* bwrap and **fail-closed (deny)** if
  absent — never degrade to unwrapped exec. Open: the non-Linux posture. macOS
  has no vetted seatbelt/`sandbox-exec` equivalent yet (IMP-045 owns that seam).
  Until one lands, the honest behaviour on macOS is "deny worktree-subagent Bash"
  or "refuse to spawn confined workers," not silent pass-through. Decide the
  degraded-mode contract explicitly.
- **OQ-2 Rust subcommand vs installed bash scripts.** The hook fires on EVERY
  tool call (Bash + Edit/Write). Per-call cost of `bash + jq` spawns adds up.
  Weigh a Rust `doctrine` subcommand handler (e.g. `doctrine worktree pretooluse`)
  — single-source with the existing `worktree create-fork` hook, faster parse,
  testable — against bash-script simplicity + binary-startup cost per call.
  Likely route through doctrine (DRY: hooks already shell `doctrine boot --emit`
  and `worktree create-fork`). Measure both before deciding.
- **OQ-3 per-run config surface + worker correlation.** Need a way to tune a
  run's jail (extra rw/ro binds, network, strict/loose) and correlate that config
  with the *specific* worker. The binding key exists (Exp 1): `agent_id` and/or
  `cwd`==worktree, both in PreToolUse stdin. Candidate: the orchestrator declares
  per-worker policy at spawn time, writing a `{worktree|agent_id → policy}` map to
  a path OUTSIDE every worktree (the brief's `.harness/` map idea); the hook looks
  it up by `cwd`. Open: where the map lives, its schema, who's authoritative
  (orchestrator-sole-writer per ADR-006), and lifecycle/GC alongside the worktree.

Recipe + evidence for the above: [[mem.pattern.dispatch.claude-worktree-subagent-bwrap-confinement]], `probe-h1/`.

Refs: SL-181, IMP-065, ADR-006 D2a/D2b, ADR-012 OQ-D, ADR-008 (jail isolation),
ADR-011 D6/M2.
