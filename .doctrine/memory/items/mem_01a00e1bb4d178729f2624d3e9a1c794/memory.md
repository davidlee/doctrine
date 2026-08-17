`tests/architecture_layering.rs`'s per-tier tangle ratchet (assertion 4) raises
`Violation::TangleGrew` only when `actual > baseline`:

```rust
let actual = count_tangle_edges(units, &filtered_edges, map, *tier);
if actual > *bl_count { violations.push(Violation::TangleGrew { .. }) }
```

So a **green** suite proves `actual <= baseline` and nothing more. It cannot
distinguish "the count is unchanged" from "the count dropped" — and a criterion
worded *baseline UNCHANGED at N* is asking the stronger question. Passing the suite
is not evidence for it.

**To read the actual number**, with nothing else installed:

1. set the tier's baseline to `0` in `.doctrine/adr/001/layering.toml`;
2. `cargo test --test architecture_layering architecture_layering_gate` — the
   failure prints `TangleGrew { baseline: 0, actual: N }`;
3. revert the file and confirm with `git diff --stat` BEFORE committing anything.

`dump_real_graph` (`#[ignore]`) prints units and edges but no SCC/tangle count, so
it does not answer this.

The asymmetry cuts the useful way for a *ratchet* — a drop should never fail a
build — so this is a limit to work around, not a defect to fix. Where a phase's
deliverable IS the number (a routing repair that must not merge two SCCs), measure
it and record the measurement; see
[[mem.pattern.lint.back-edge-tangle-inject-fnptr]] and
[[mem.pattern.lint.mcp-server-entangled-with-core]] for why the number, not the
verdict, is the thing that carries information there.

Found at SL-238 PHASE-08, whose `VA-3` read *unchanged at 76* and whose whole
purpose was the measurement.
