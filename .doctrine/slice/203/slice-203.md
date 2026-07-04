# Dissolve commands↔mcp_server dependency cycle

## Context

RSK-227 mapped the command-tier coupling and found `{commands, mcp_server}` as
its own 2-edge SCC, separate from the 23-node core. Tier-1 mitigation IMP-265.
This is the cheap warm-up that proves the cycle-breaking pattern before the
larger integrity refactor (SL-204 / IMP-266).

The cycle is two concrete edges:

- **forward** `commands → mcp_server` — `src/commands/serve.rs:28` calls
  `crate::mcp_server::serve(McpConfig{…})` to start the MCP stdio server.
- **back** `mcp_server → commands` — `src/mcp_server/tools.rs:1272` calls
  `crate::commands::prompt::model_keys(root, None)`.

The forward edge is legitimate (a `serve` command *should* start its server).
The back edge is incidental: the MCP tool layer reaches into a command handler
for one prompt helper. Sever the back edge and the SCC dissolves.

## Scope & Objectives

- Lift `model_keys` (and any tightly-coupled prompt-key helpers it needs) out of
  `commands::prompt` into a **leaf** (or engine) module that both `mcp_server`
  and `commands::prompt` can depend on downward — SL-016 pattern (extract the
  shared item so the cycle can't close).
- Result: `mcp_server` no longer imports `crate::commands::*`; the
  `{commands, mcp_server}` SCC becomes trivial.
- Ratchet the `command` tangle baseline in `.doctrine/adr/001/layering.toml`
  down by the removed edges (the gate is monotone; a lower count must be
  recorded).

## Non-Goals

- The 23-node core SCC / `integrity` knot — that is SL-204 (IMP-266).
- Touching the forward `serve → mcp_server::serve` edge (it is correct).
- Any behaviour change to `model_keys` output or the MCP tool surface — pure
  move / re-home.
- Building a fan-out/degree gate (RSK-227 capture-and-watch, explicitly deferred).

## Affected surface

- `src/commands/prompt.rs` — source of `model_keys`.
- `src/mcp_server/tools.rs` — the back-edge caller (`:1272`).
- `src/commands/serve.rs` — forward edge (read-only reference; unchanged).
- new/target leaf module for the extracted helper (name TBD in `/design`).
- `.doctrine/adr/001/layering.toml` — `[tangle_baseline] command` ratchet.
- `tests/architecture_layering.rs` — the gate that verifies the dissolution.

## Risks / assumptions / open questions

- **OQ-1** — where does `model_keys` belong? A pure prompt-keys leaf, or an
  existing leaf (`prompt` has a leaf/engine split already?). `/design` decides.
- **ASM-1** — `model_keys` has no command-tier dependencies of its own, so it is
  cleanly liftable to a leaf. Verify during design.
- **R1** — if `model_keys` transitively pulls other `commands::prompt` items,
  the extraction surface grows; keep it minimal or the slice sprawls.

## Verification / closure intent

- `tests/architecture_layering.rs` green with the `{commands, mcp_server}` SCC
  gone and the `command` tangle baseline lowered (VT).
- Full suite green, unchanged — behaviour-preservation gate (VT).
- `doctrine slice conformance` clean against the seeded selectors (VA).

## Summary

## Follow-Ups
