# The second source tree gets a ZERO tangle baseline, and it is synthesised, not authored

`tests/architecture_layering.rs` runs the `ADR-001` gate once per source tree
(`gate_tree`, `:1273`). For the root tree it loads the tier map, the accepted
violations and the tangle baseline from `.doctrine/adr/001/layering.toml`. For a
**second** tree — `crates/doctrine-control/src`, section
`[doctrine_control_tiers]` — only the tier map is authored. `load_tree_layering`
(`:506-527`, SL-248 PHASE-01 `D1`) synthesises the other two:

```rust
let baseline = TangleBaseline(BTreeMap::from([
    (Tier::Leaf, 0), (Tier::Engine, 0), (Tier::Command, 0),
]));
Ok((map, Accepted(BTreeSet::new()), baseline))
```

Rationale: a brand-new crate has earned neither a tangle allowance nor an
accepted violation, and inheriting the root's baseline would be a loosening by
accident.

## The consequence, which is sharper than the rule

**Any import cycle between two units of the same tier in `doctrine-control` is a
hard gate failure with no local escape.** The tangle ratchet (`:783-791`) counts
edges whose endpoints share a non-trivial SCC within one tier and reds
`TangleGrew` when the count exceeds the baseline — which is 0, for every tier.
There is no row you can add to `layering.toml` to permit it: the baseline is not
read from that file. Raising it means editing `tests/architecture_layering.rs`,
which SL-248's plan assigns to PHASE-01/02 and to no later phase.

So in this crate, a cycle is a **design** problem, always. It cannot be
baselined away the way the root package's 120-edge command tangle was.

Found while planning SL-248 PHASE-03: `sec-5`'s `HostFacts::available_bytes`
sketch returns `Result<ByteCount, CapacityUnknown>` with `ByteCount` in
`config.rs`, while `config.rs` must import `HostFacts` for root resolution —
a two-node `leaf` SCC that `sec-6`'s own unit table (`host | leaf | none`)
contradicts. The table was right and the sketch was wrong.

**Corollary for edge extraction:** `extract_edges` skips `#[cfg(test)]` items, so
a test-only fixture shared across units (`crate::host::FixtureHost` used from
`config.rs`'s test module) produces no edge and cannot contribute to a tangle.
Sharing test fixtures across units in this crate is free; sharing production
types is not.
