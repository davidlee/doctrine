# A back edge can entangle a module with the core SCC — measure the tangle delta, don't trust a coupling map's "separate SCC" claim

Severing one back edge can drop the command tangle by more than 2; a static
SCC-separation claim and a monotone baseline both drift — measure empirically.

## What happened (SL-203)

RSK-227's coupling map declared `{commands, mcp_server}` a **core-separate**
2-node SCC (`backlog-227.md:94`), so severing its one back edge
(`mcp_server::tools::render_model_band_guidance → commands::prompt::model_keys`,
`tools.rs:1365`) should drop the `command` tangle by exactly **2** (both edges of
a clean 2-cycle leave). Execute-time measurement said otherwise:

- restoring the back edge: `command` tangle 86 → **90** (+4)
- severing it: 90 → **86** (**−4**, not −2)

The back edge did not close an isolated 2-cycle — it **fused `mcp_server` into
the 23-node core SCC**. Once fused, `mcp_server`'s own core-pointing edges
(`→ memory / review / slice / worktree`…) counted as tangle. Severing it ejects
`mcp_server` *and* those edges → −4. The decoupling was *better* than the map
predicted, but the map's topology was wrong.

Second trap, independent: the stored `[tangle_baseline] command = 123` was
**stale**. ADR-001's gate is monotone-upward (fails only on growth), so it never
forces tightening; the live count had drifted to 90 while the baseline sat at
123 — ~33 edges of silent slack. Reading `123` as "the live count" is a mistake.

## Why

- A directed **back edge from a lower/peripheral module up into a hub can merge
  that module into the hub's SCC**, so the edge's removal is worth far more than
  its own weight in tangle terms. A coupling snapshot that names an SCC
  "separate" is a point-in-time inference, not a running invariant.
- A **monotone-non-increasing baseline is an upper bound, not the current
  value.** It only ratchets when someone tightens it; drops accumulate as slack.

## How to apply

- When a slice's success criterion is a tangle-count delta, **defer the number to
  execute-time and measure it** (design VT deferral is correct) — never hard-code
  the predicted drop as a gate keyword. SL-203's plan VT-1 hard-coded `command =
  121`; both the `123` and the `−2` were wrong (corrected to `86`).
- To read the live count, force a gate failure with a deliberately-low baseline
  (`command = 0`) and read `TangleGrew { actual }` — do not trust the stored
  baseline as current.
- Ratchet to the **measured** tight value, folding in any accumulated slack.

Related: [[mem.pattern.lint.back-edge-tangle-inject-fnptr]] (the fn-pointer
inversion applied here), [[mem.pattern.lint.module-split-needs-layering-entry]].
Corrects RSK-227's topology model; the precedent flows to SL-204.
