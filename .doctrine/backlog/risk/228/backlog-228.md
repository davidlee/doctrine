# RSK-228: SL-204 SCC-decomposition premise unverified — RSK-227's separable-SCC map falsified by SL-203

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Risk

SL-204 (IMP-266, the larger integrity/core-SCC refactor) inherits its planning
premise from RSK-227's coupling map, which models the command tier as a set of
separable SCCs. SL-203 falsified that model for its own scope: RSK-227 called
`{commands, mcp_server}` a **core-separate** 2-node SCC, but execute-time
measurement showed the back edge had fused `mcp_server` **into** the 23-node core
SCC (severing it dropped the tangle by **4**, not the predicted 2 — see
`mem.pattern.lint.mcp-server-entangled-with-core` and SL-203 design §1 F-EXEC-1).

If a static coupling snapshot mis-attributed one module's SCC membership, it may
mis-attribute others. SL-204's decomposition plan — which edges to cut, in what
order, and the expected tangle drop per cut — must therefore be **re-derived
empirically against the live graph**, not read off RSK-227's map or its stored
`[tangle_baseline]` figures (which drift stale under the monotone gate).

## Action when SL-204 is designed

- Re-run the real graph (`architecture_layering.rs` `#[ignore] dump_real_graph`)
  and recompute the core SCC membership + per-edge tangle contribution fresh.
- Treat every predicted "−N drop" as an execute-time measurement (force a gate
  failure with `command = 0` to read `TangleGrew { actual }`), never a hard-coded
  plan keyword.
- Reconcile RSK-227's map to whatever the live measurement shows before locking
  SL-204's design.

Links: RSK-227 (falsified map), SL-203 (the warm-up that surfaced this),
`mem.pattern.lint.mcp-server-entangled-with-core`.
