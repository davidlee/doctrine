# Dissolve commands↔mcp_server dependency cycle

## Context

RSK-227 mapped the command-tier coupling and found `{commands, mcp_server}` as
its own 2-edge SCC, separate from the 23-node core. Tier-1 mitigation IMP-265.
This is the cheap warm-up that proves the cycle-breaking pattern before the
larger integrity refactor (SL-204 / IMP-266).

The cycle is two concrete edges:

- **forward** `commands → mcp_server` — `src/commands/serve.rs:28` calls
  `crate::mcp_server::serve(McpConfig{…})` to start the MCP stdio server.
- **back** `mcp_server → commands` — `src/mcp_server/tools.rs:1365` calls
  `crate::commands::prompt::model_keys(root, None)`.

The forward edge is legitimate (a `serve` command *should* start its server).
The back edge is incidental: the MCP tool layer reaches into a command handler
for one prompt helper. Sever the back edge and the SCC dissolves.

## Scope & Objectives

Approach locked in `/design` as **D-B: fn-pointer injection** (not extraction —
see design §7 / OQ-1 below).

- Invert the back edge: inject `model_keys` as a `ModelKeysFn` fn-pointer through
  `McpConfig`, supplied by `commands::serve` (which already depends downward on
  `mcp_server` via the forward edge) and threaded to the `doctrine_onboard` tool
  arm. `model_keys` stays in `commands::prompt`; nothing moves.
- Result: `mcp_server` production code imports neither `crate::commands` nor
  `crate::install`; it becomes corpus-agnostic and the `{commands, mcp_server}`
  SCC becomes trivial. **No new same-tier edge** (avoids the concentration
  RSK-227 warns against).
- Ratchet the `command` tangle baseline in `.doctrine/adr/001/layering.toml`
  down by the removed edges — expected `123 → 121` (the 2-cycle is core-separate,
  so both its edges leave the count). The gate is monotone; a lower count must be
  recorded.

## Non-Goals

- The 23-node core SCC / `integrity` knot — that is SL-204 (IMP-266).
- Touching the forward `serve → mcp_server::serve` edge (it is correct).
- Any behaviour change to `model_keys` output or the MCP tool surface — pure
  re-wiring; `doctrine_onboard` output byte-identical.
- Moving `model_keys` or introducing a leaf module (extraction rejected, D-A).
- Building a fan-out/degree gate (RSK-227 capture-and-watch, explicitly deferred).

## Affected surface

Design-target touch-set (recorded as `design-target` selectors):

- `src/mcp_server/mod.rs` — `ModelKeysFn` type + `model_keys` field on
  `McpConfig`; `serve` threads it into `dispatch`.
- `src/mcp_server/tools.rs` — `+ModelKeysFn` param on `dispatch` /
  `handle_tools_call` / `call_tool` / `render_onboard` /
  `render_model_band_guidance`; drop the `crate::commands::prompt::model_keys`
  static call; wiring-guard test.
- `src/commands/serve.rs` — supply `model_keys` in the `McpConfig` literal.
- `.doctrine/adr/001/layering.toml` — `[tangle_baseline] command` ratchet 123→121.
- `tests/architecture_layering.rs` — the gate; fix stale header comment 120→121.
- `src/commands/prompt.rs` — source of `model_keys`; **unchanged** (stays put).

## Risks / assumptions / open questions

Resolved in `/design`:

- **OQ-1** (where does `model_keys` belong?) → **resolved**: stays in
  `commands::prompt`. The "move to a leaf" premise is unsound — its corpus
  gather is anchored to the `install`/RustEmbed command tier, so a moved
  `model_keys` would only relocate the upward reach. Injection dissolves the
  edge cleanly instead.
- **ASM-1** (`model_keys` cleanly liftable to a leaf) → **falsified**: the filter
  is pure but the gather binds `Assets` (command tier). Superseded by D-B.
- **R1** (extraction sprawl) → **avoided** by choosing D-B (zero module moves).

## Verification / closure intent

- `tests/architecture_layering.rs` green with the `{commands, mcp_server}` SCC
  gone and the `command` tangle baseline lowered (VT).
- Full suite green, unchanged — behaviour-preservation gate (VT).
- `doctrine slice conformance` clean against the seeded selectors (VA).

## Summary

## Follow-Ups
