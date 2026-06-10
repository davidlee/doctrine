# cordage

A generic multi-channel evaluation engine over a tree plus typed directed (DAG)
overlays.

`cordage` is product-neutral graph machinery: opaque node ids, typed directed
edges with opaque ordering attributes, per-overlay cycle policy, deterministic
ordering, reverse-edge reachability, and generic channel propagation. It carries
no application vocabulary — consumers map their domain onto overlays and channels.

Build a graph with `GraphBuilder`, then query it:

```rust
use cordage::{Arity, CyclePolicy, EdgeAttrs, GraphBuilder, OverlayConfig};

let mut b = GraphBuilder::new();
let membership = b.overlay(OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded));
let parent = b.node();
let child = b.node();
b.edge(membership, parent, child, EdgeAttrs::new(0, 0));
let graph = b.build().expect("valid build");

for (neighbour, _attrs) in graph.out_edges(membership, parent) {
    assert_eq!(neighbour, child);
}
```

Zero runtime dependencies by construction.
