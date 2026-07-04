# IMP-265: Dissolve commands↔mcp_server dependency cycle

Tier-1 (cheap, isolated) mitigation surfaced by RSK-227. The command tier's
smallest tangle is a self-contained 2-edge SCC: `{commands, mcp_server}`.

## The cycle

- `commands → mcp_server` — the cli exposes an `mcp serve` subcommand.
- `mcp_server → commands` — the MCP handler needs the `Command` enum to
  dispatch tool calls.

These two edges form their own non-trivial SCC (separate from the 23-node core),
so both count toward the command tangle baseline.

## Fix

Break the back-edge, SL-016 style (extract the shared type to a leaf so the
cycle can't close):

- extract the `Command` dispatch entry / enum that `mcp_server` imports into a
  leaf module, **or**
- invert the `serve` wiring so `commands` no longer imports `mcp_server`.

## Payoff / effort

Dissolves an entire SCC (−2 tangle edges, ratchet the baseline down). Small,
self-contained — likely no slice needed, a quick design sketch + edit. Good
warm-up that proves the cycle-breaking pattern before the larger IMP-266.

Behaviour-preservation gate: existing suites are the proof; they must stay green
unchanged. **Precedent:** SL-016 (extract plan types to break `slice↔state`).

Surfaced by RSK-227 (see its `graph/` coupling map). Complements — not replaces —
RSK-227's capture-and-watch arm: this shrinks the tangle rather than gating it.
