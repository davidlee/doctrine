# DEC-003: worker_commit gate binary: layered recipe + engine publishes DOCTRINE_BIN

## Decision

For SL-225 #1 (ISS-218 — the `worker_commit` gate's `just validate` → `doctrine
doctor` false-reds because it shells the stale PATH `~/.cargo/bin/doctrine`):

- **Project tier** (this repo's `justfile:28` `validate` recipe): resolve the
  `doctrine` it shells in **layered** order — `${DOCTRINE_BIN}` → the local
  `./target/debug/doctrine` (if executable) → bare `doctrine` (PATH). The PATH
  tail is *correct* for a generic host (their installed binary is not stale).
- **Platform tier** (engine, one thin generic affordance): `worker_commit` exports
  `DOCTRINE_BIN=<the running MCP server's `current_exe()`>` into the gate
  subprocess environment, so the layered recipe's first rung is reliably set.

## Why (POL-002 — do not conflate doctrine-as-project with doctrine-as-platform)

The stale-binary false-red is a **dogfooding artifact**: only an in-flight slice
that changes doctrine's *own* binary makes the installed binary stale relative to
the fork. A generic host never hits it (workers can't write `.doctrine/`; their
slice doesn't rebuild the doctrine binary). So the fix is **project-tier**; the
engine must not learn cargo's `./target/debug` layout (POL-002).

The engine's *only* contribution is publishing "the doctrine I am running as"
(`current_exe()`) — an agnostic value, not a host convention. A generic host's
gate simply ignores `$DOCTRINE_BIN`; harmless.

## Why not the alternatives

- **Bake `./target/debug/doctrine` or `current_exe()` resolution *into* the engine
  gate** — POL-002 violation: cargo layout / a resolution policy in platform code.
- **Recipe existence-check alone (`[ -x ./target/debug/doctrine ] || doctrine`),
  zero engine** — holes on a **non-Rust phase**: the gate's `validate` leg runs
  *before* its `build` leg, and a pure-JS / docs phase never builds the fork's
  binary, so it silently falls back to the stale PATH binary — re-masking exactly
  ISS-218's SL-206-P14 repro. `$DOCTRINE_BIN` (the coord/orchestrator build, which
  carries the earlier phase's rules and is guaranteed present) closes that hole
  without a local build.

## Preconditions / carried assumptions

- `current_exe()` is the *right* binary only when the **coord-built** doctrine is
  what serves MCP — already the AGENTS.md convention ("run corpus verbs from the
  coord tree's `./target/debug/doctrine`"). If the operator serves MCP from a
  stale PATH binary, `current_exe()` inherits that staleness → captured as [[ASM]].

## Links

- Governs SL-225 #1; fulfils ISS-218 (IMP-270 dup-closed).
- POL-002 (platform independence) is the gate that forced this split.
- Sibling #2 (marker-aware e2e skip, CHR-044) is likewise project-tier — the
  engine already provides the gate marker-clear (SL-199 F2).
