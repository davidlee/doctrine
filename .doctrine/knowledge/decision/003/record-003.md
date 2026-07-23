# DEC-003: worker_commit gate binary — zero-engine layered recipe + DOCTRINE_BIN precondition

> Revised 2026-07-24 after the SL-225 adversarial pass: the engine-publish first
> proposed here was **rejected** (redundant-or-harmful). The engine is untouched.

## Decision

For SL-225 #1 (ISS-218 — the `worker_commit` commit gate's `just check` →
`just validate` → `doctrine doctor` false-reds because `validate` shells the stale
PATH `~/.cargo/bin/doctrine`):

- **Project tier only.** This repo's `justfile` `validate` recipe resolves the
  `doctrine` it shells in **layered** order — `${DOCTRINE_BIN}` → the local
  `./target/debug/doctrine` (if executable) → bare `doctrine` (PATH). The PATH
  tail is *correct* for a generic host (their installed binary is not stale).
- **Precondition, not code.** `DOCTRINE_BIN` must point at the **coord build** for
  a dispatch session — recorded in `.doctrine/governance.md` (§ orchestration), not
  a skill. The forwarding mechanism already exists: `flake.nix` `try-fwd-env
  DOCTRINE_BIN` + `.mcp.json` `command = "${DOCTRINE_BIN:-doctrine}"`.
- **Engine: no change.** `worker_commit` / the gate are not touched.

## Why zero-engine (the adversarial reversal)

The engine-publish (`worker_commit` exports `DOCTRINE_BIN=current_exe()`) was
rejected on evidence:

- **`.mcp.json` already launches the server via `${DOCTRINE_BIN:-doctrine}`**, and
  the gate is a child of the server process, so it **already inherits** whatever
  `DOCTRINE_BIN` the server has. When `DOCTRINE_BIN` is set → publish is redundant.
- When `DOCTRINE_BIN` is **unset**, `current_exe()` is the *stale* server binary;
  publishing it **pins the recipe's first rung to the stale binary**, pre-empting
  the fall-through to the fork's fresh local `./target/debug/doctrine`. On a Rust
  phase that is strictly *worse* than doing nothing.

So the publish never helps and sometimes hurts. `current_exe()` and an inherited
`DOCTRINE_BIN` are the *same value* whenever the server was launched via
`DOCTRINE_BIN` — which is the only way to get a non-stale server anyway.

## Why (POL-002 — do not conflate doctrine-as-project with doctrine-as-platform)

The stale-binary false-red is a **dogfooding artifact**: only an in-flight slice
that changes doctrine's *own* binary makes the installed binary stale relative to
the fork. A generic host never hits it (workers can't write `.doctrine/`; their
slice doesn't rebuild the doctrine binary). So the fix is **project-tier**; the
engine must not learn cargo's `./target/debug` layout, and must not resolve the
coord path either (both POL-002 violations).

## Why not the alternatives

- **Bake `./target/debug` or a `current_exe()`/coord-path resolution into the
  engine gate** — POL-002 violation (cargo layout / resolution policy in platform
  code); and `current_exe()` is redundant-or-harmful (above).
- **Recipe existence-check alone, no `DOCTRINE_BIN` rung** — holes on a **non-Rust
  phase**: `validate` runs *before* the gate's `build` leg (`DEFAULT_COMMIT =
  ["just","check"]`, verify.rs:149), and a pure-JS / docs phase never builds the
  fork binary, so it falls back to the stale PATH binary — re-masking ISS-218's
  SL-206-P14 repro. The `${DOCTRINE_BIN}` first rung (coord build) closes that hole
  **without** a local build; hence the governance precondition.

## Preconditions / carried assumptions

- **ASM:** the operator/dispatch setup sets `DOCTRINE_BIN` to the coord build. If
  MCP is served from a stale PATH binary *and* `DOCTRINE_BIN` is unset, a non-Rust
  phase still false-reds — the residual corner this design accepts rather than
  solve in engine code. Mitigation is the governance rule, not more mechanism.

## Links

- Governs SL-225 #1; fulfils ISS-218 (IMP-270 dup-closed).
- POL-002 (platform independence) is the gate that forced project/platform split
  *and* killed the engine-publish.
- Precondition lives in `.doctrine/governance.md` (§ orchestration).
- Sibling #2 (marker-aware e2e skip, CHR-044) is likewise project-tier — the
  engine already provides the gate marker-clear (SL-199 F2).
