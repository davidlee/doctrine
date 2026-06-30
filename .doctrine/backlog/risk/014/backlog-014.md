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

Refs: SL-181, IMP-065, ADR-006 D2a/D2b, ADR-012 OQ-D, ADR-008 (jail isolation),
ADR-011 D6/M2.
