# DEC-003: worker gate does not re-run coord's governance self-checks

> Revised 2026-07-24 across three adversarial passes. Rejected in turn: the
> engine-publish of `DOCTRINE_BIN=current_exe()` (redundant-or-harmful); the
> launch-frozen `DOCTRINE_BIN`→coord-build precondition (not temporally achievable,
> RV-291 F-1); and the git-derived self-locating recipe (RV-292 F-1/2/3 — a worker
> fork cannot locate its coord tree via git, the coord binary is never built, and
> `validate` precedes `build`). This decision changes frame: stop trying to give the
> gate a fork-consistent `doctrine`, and instead stop running the checks that need
> one — because they carry no worker-delta signal in a fork.

## Decision

For SL-225 #1 (ISS-218 — the `worker_commit` commit gate's `just check` →
`just validate` → `doctrine doctor` false-reds because `validate` shells the stale
PATH `~/.cargo/bin/doctrine`):

- **`validate` is the whole surface.** Of the `check` belt
  (`fmt lint lint-js validate test build`, justfile:17), only `validate`
  (`doctrine prompt check` + `doctrine doctor`) shells the installed binary; the rest
  use cargo/node or build a fresh binary.
- **Skip the governance legs in a worker context.** `validate` runs `doctrine prompt
  check` / `doctrine doctor` only on a generic host; in a worker fork it exits 0
  early. The signal is three legs (the same predicate as fix #2's
  `under_worker_marker`): `DOCTRINE_DISPATCH_GATE` (set by `worker_commit` on the gate
  spawn) **or** `DOCTRINE_WORKER` (subprocess arm) **or** the marker file
  (`.doctrine/state/dispatch/worker`, claude-arm manual run).
- **One neutral engine line.** `worker_commit::run_commit_gate` spawns the gate with
  `.env("DOCTRINE_DISPATCH_GATE", "1")` — needed because the gate clears the marker
  and removes `DOCTRINE_WORKER` for its own run (SL-199 F2 / fix #2), so neither
  other leg is visible in that window. No change to gate logic, staging, or the guard.
- **Delete the obsolete `DOCTRINE_BIN` precondition** from `.doctrine/governance.md`
  and `CLAUDE.md`; it governs nothing now.

## Why the governance legs are inert in a fork (the load-bearing claim)

`doctrine doctor` and `doctrine prompt check` validate the repo's **authored
`.doctrine/` state** (entity/relation/boot consistency, snapshot freshness). A
worker **cannot write `.doctrine/`** — the worker-mode guard refuses authored writes
under the marker. So in any worker fork their *input is byte-identical to coord's*,
and coord already ran them green. The only variable that can flip their verdict in a
fork is the **binary version** — which is exactly, and only, the stale-binary
false-red. They carry **zero worker-delta signal**; the gate exists to validate the
worker's *code delta*, not to re-run coord's governance self-audit. Removing them
removes the false-red and nothing else — see the one concession below.

## Why not chase a fork-consistent binary (the three rejected designs)

- **Engine-publish `DOCTRINE_BIN=current_exe()`** — `.mcp.json` already launches the
  server via `${DOCTRINE_BIN:-doctrine}` and the gate inherits it, so publish is
  redundant when set and *harmful* when unset (pins the stale server binary). RV-291.
- **Launch-frozen `DOCTRINE_BIN`→coord precondition** — not temporally achievable:
  the client resolves `${DOCTRINE_BIN}` at server launch (boot.rs:544-549), freezing
  the server env, but the coord build is created by `dispatch setup` *after* the
  session runs (RV-291 F-1). A later `export` cannot reach the running server.
- **Git-derived self-locating recipe** — git's worktree model is **flat**: a worker
  fork and its coord tree share the *primary/edge* common `.git`, so
  `dirname $(git rev-parse --git-common-dir)` returns the edge root, not coord
  (`git.rs:557` documents this). And even located, the coord binary is **never built**
  by dispatch (RV-292 F-3), while `validate` precedes `build` so no binary reflecting
  the current phase's own delta exists at validate time (RV-292 F-2).

All three try to make the *engine* or a *git inference* resolve *which binary* — a
project/cargo concern that either violates POL-002 or founders on git topology. The
skip needs none of it.

## POL-002 — the platform/project boundary holds

`DOCTRINE_DISPATCH_GATE` is a **neutral context signal** — "a `worker_commit` gate
run is underway" — with no cargo layout, path, or resolution/governance policy. The
engine announces *where*; the **project recipe** decides *what* to skip. A generic
host's justfile never reads it. This is the clean inversion of the rejected designs,
which pushed cargo/binary-resolution knowledge toward the platform.

## The one concession

A phase that changes `doctrine doctor` / `prompt check`'s **own logic** (a Rust
change to what governance-consistency means) will not have that new rule exercised
against coord's authored state *in the fork gate*. It is covered by the phase's own
unit tests and by coord's gate/audit when the delta lands — which is where a
governance-rule change belongs (the fork gate validates the worker's delta, not
coord's pre-existing authored state). This is the sole behaviour the skip removes.

## Preconditions / carried assumptions

- **No coord-build precondition, no launch-time env contract.** The fix depends only
  on the worker-context signal, which is established by `worker_commit` (gate) or the
  existing marker/`DOCTRINE_WORKER` (worker's own run) — all already present.
- **Guard invariant relied upon:** a worker cannot write authored `.doctrine/` state
  (worker-mode guard). If that ever changed, the inertness argument would need
  revisiting.

## Links

- Governs SL-225 #1; fulfils ISS-218 (IMP-270 dup-closed).
- POL-002 (platform independence) forced the neutral-signal framing and killed the
  three binary-resolution designs.
- The obsolete `DOCTRINE_BIN`→coord-build rule is **deleted** from
  `.doctrine/governance.md` (§ orchestration) and CLAUDE.md (§ Dispatch precondition).
- Sibling #2 (marker-aware e2e skip, CHR-044) is the same principle — governance-
  coupled checks don't belong in the worker's run — and shares the env-or-marker
  predicate; the engine already provides the gate marker-clear (SL-199 F2).
- Prior arraignments: RV-291 (engine-publish + frozen precondition), RV-292
  (git self-locating recipe).
