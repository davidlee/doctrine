# DEC-003: worker gate does not re-run coord's governance self-checks

> Revised 2026-07-24 across three adversarial passes. Rejected in turn: the
> engine-publish of `DOCTRINE_BIN=current_exe()` (redundant-or-harmful); the
> launch-frozen `DOCTRINE_BIN`→coord-build precondition (not temporally achievable,
> RV-291 F-1); and the git-derived self-locating recipe (RV-292 F-1/2/3 — a worker
> fork cannot locate its coord tree via git, the coord binary is never built, and
> `validate` precedes `build`). This decision changes frame: stop trying to give the
> gate a fork-consistent `doctrine`, and instead stop running the checks that need
> one — because they carry no worker-delta signal in a fork. The one residual (a delta
> to the checks' *own logic*) is closed **project-side** by a fresh-binary corpus gate
> at the orchestrator's close (ii), so the `DOCTRINE_BIN`/coord-build rule is retained
> and reframed, not deleted (revised again 2026-07-24 after a codex external read).

## Decision

For SL-225 #1 (ISS-218 — the `worker_commit` commit gate's `just check` →
`just validate` → `doctrine doctor` false-reds because `validate` shells the stale
PATH `~/.cargo/bin/doctrine`):

- **`validate` is the whole surface.** Of the `check` belt
  (`fmt lint lint-js validate test build`, justfile:17), only `validate`
  (`doctrine prompt check` + `doctrine doctor`) shells the installed binary; the rest
  use cargo/node or build a fresh binary.
- **Skip the governance legs in a worker context.** `validate` runs `doctrine prompt
  check` / `doctrine doctor` only off the fork path; in a worker fork it exits 0
  early. The signal is three legs (the same predicate as fix #2's
  `under_worker_marker`): `DOCTRINE_DISPATCH_GATE` (set by `worker_commit` on the gate
  spawn) **or** `DOCTRINE_WORKER = "1"` (subprocess arm — **exact** match, ==
  `env_worker_set()` / marker.rs:127, never a lax `-n`) **or** the marker file
  (`.doctrine/state/dispatch/worker`, claude-arm manual run).
- **One neutral engine line.** `worker_commit::run_commit_gate` spawns the gate with
  `.env("DOCTRINE_DISPATCH_GATE", "1")` — needed because the gate clears the marker
  and removes `DOCTRINE_WORKER` for its own run (SL-199 F2 / fix #2), so neither
  other leg is visible in that window. No change to gate logic, staging, or the guard.
- **Close the residual project-side, at the orchestrator (ii).** Off the fork path,
  `validate` resolves the **fresh coord build** (`${DOCTRINE_BIN:-./target/debug/doctrine}`,
  PATH fallback) instead of bare `doctrine`, so `close`'s `doctrine check gate`
  validates the landed corpus with the source-consistent binary; the project close
  ritual gains a build-before-gate beat. Entirely project-tier (`justfile` /
  governance / ritual) — **zero engine surface, inert for clients**.
- **Retain and reframe the `DOCTRINE_BIN` precondition** in `.doctrine/governance.md`
  and `CLAUDE.md` — it now governs (ii)'s coord-side *close-time* build, not a
  fork-side *launch-time* env contract (RV-291 F-1). Reframed, **not deleted**.

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

## Closing the residual (not conceding it)

The fork-skip leaves **one** coverage question: a phase that changes `doctrine doctor`
/ `prompt check`'s **own logic** (a Rust change to what governance-consistency means)
is not exercised against the real authored corpus *by a fresh binary in the fork* —
and never was (the fork gate always ran the *stale PATH* binary; that is ISS-218).
Traced, no orchestrator beat catches it either: `dispatch_import` runs only the pure
classify belt (prefix + scope), the pi-arm import runs only `prove` (`fmt-check lint`),
`dispatch_conclude_phase` flips the sheet + boundary commit, and `close`'s
`doctrine check gate` → `validate` shells **bare PATH** `doctrine`.

So the residual is **closed**, not conceded — project-side, at the orchestrator's
**close** (ii above): the orchestrator *is* coord (source landed, tree owned, build
tractable — none of the fork-side blockers apply), so `close`'s gate validates the
landed corpus with the freshly-built source-consistent binary. Backstopped by the
phase's own unit tests. The earlier draft's claim that "coord's gate/audit on landing"
already covered it was **false** — that gate is itself stale-PATH (codex external read,
2026-07-24).

## Preconditions / carried assumptions

- **The fork-skip needs no coord-build precondition and no launch-time env contract.**
  It depends only on the worker-context signal, established by `worker_commit` (gate)
  or the existing marker/`DOCTRINE_WORKER` (worker's own run) — all already present.
- **(ii) needs a coord-side *close-time* build.** The fresh-binary corpus gate assumes
  the orchestrator builds the coord/landing tree before `close`'s `check gate`
  (the reframed `DOCTRINE_BIN` precondition + close-ritual beat). This is tractable —
  coord is a full tree the orchestrator owns — unlike the fork-side launch-time
  contract RV-291 F-1 killed.
- **Guard invariant relied upon:** a worker cannot write authored `.doctrine/` state
  (worker-mode guard). If that ever changed, the inertness argument would need
  revisiting.

## Links

- Governs SL-225 #1; fulfils ISS-218 (IMP-270 dup-closed).
- POL-002 (platform independence) forced the neutral-signal framing and killed the
  three binary-resolution designs.
- The `DOCTRINE_BIN`→coord-build rule is **retained and reframed** in
  `.doctrine/governance.md` (§ orchestration) and CLAUDE.md (§ Dispatch precondition) —
  a coord-side close-time build governing (ii)'s fresh-binary corpus gate, no longer a
  fork-side launch contract.
- Sibling #2 (marker-aware e2e skip, CHR-044) is the same principle — governance-
  coupled checks don't belong in the worker's run — and shares the env-or-marker
  predicate; the engine already provides the gate marker-clear (SL-199 F2).
- Prior arraignments: RV-291 (engine-publish + frozen precondition), RV-292
  (git self-locating recipe).
