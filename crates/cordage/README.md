# cordage

`cordage` is a deterministic graph-evaluation core for Rust.

It is designed for systems where the same set of entities participates in multiple typed relationships, and where you need more than adjacency traversal:

* typed directed overlays over one shared node set
* deterministic edge ordering and graph resolution
* per-overlay cycle handling
* reverse reachability
* composed ordering across overlays
* channel propagation over reachable nodes
* structured provenance and explanation data
* zero runtime dependencies

`cordage` is not a replacement for `petgraph`. Use `petgraph` when you want a broad graph library with standard graph algorithms and flexible graph storage. Use `cordage` when your graph is part of an evaluation pipeline and you need repeatable results, explicit conflict handling, and explainable propagation.

## What problem does it solve?

Many systems eventually grow more than one relationship graph over the same objects.

For example, a job, rule, document, workflow step, package, policy item, or build unit might have:

* a parent/child relationship
* dependency relationships
* sequencing relationships
* membership or grouping relationships
* derived state that should propagate through one or more of those relationships

A normal graph library can store those edges. `cordage` is for when you also need to answer questions like:

* Which nodes are reachable along this relationship?
* Which single parent survives when only one parent is allowed?
* What deterministic order should these nodes appear in?
* What cycles were detected, and which edges were evicted?
* Which input nodes contributed to a propagated result?
* Can I explain a node’s order and predecessor cone without rendering prose in the core library?

## Core model

A `Graph` contains opaque `NodeId`s and one or more typed overlays.

Each overlay has:

* a cycle policy:

  * `CyclePolicy::Reject` diagnoses cycles and keeps the graph queryable
  * `CyclePolicy::Evict` removes deterministic loser edges until the overlay is acyclic
* an incoming-edge arity:

  * `Arity::AtMostOne` for spine-like relationships
  * `Arity::Unbounded` for ordinary multi-parent relationships

Edges carry `EdgeAttrs`:

```rust
EdgeAttrs::new(rank, age)
```

The core does not interpret the meaning of `rank` or `age`; it only uses them to make deterministic keep/evict decisions.

## Basic usage

```rust
use cordage::{
    Arity, CyclePolicy, EdgeAttrs, GraphBuilder, OverlayConfig,
};

let mut builder = GraphBuilder::new();

let membership = builder.overlay(OverlayConfig::new(
    CyclePolicy::Reject,
    Arity::Unbounded,
));

let parent = builder.node();
let child = builder.node();

builder.edge(membership, parent, child, EdgeAttrs::new(0, 0));

let graph = builder.build().expect("valid graph");

let outgoing: Vec<_> = graph.out_edges(membership, parent).collect();

assert_eq!(outgoing.len(), 1);
assert_eq!(outgoing[0].0, child);
```

## Reachability

`reachable` walks one overlay in one direction.

```rust
use cordage::Direction;

let reached = graph.reachable(membership, parent, Direction::Along);

assert!(reached.contains(&child));
```

Reachability is strict: the start node is not included in its own reachable set, even if a cycle reaches back to it.

## Spine paths

An overlay with `Arity::AtMostOne` can be used as a spine. After build-time resolution, each node has at most one kept parent on that overlay.

```rust
let spine = builder.overlay(OverlayConfig::new(
    CyclePolicy::Reject,
    Arity::AtMostOne,
));

// add spine edges...

let graph = builder.build().expect("valid graph");
let path = graph.spine_path(spine, some_node);
```

The returned path is ordered from root to node. Non-spine overlays return `None`.

## Channel propagation

A channel folds seeded values over a reachable set.

```rust
use std::collections::BTreeMap;

use cordage::{
    ChannelSpec, ChannelValue, Combinator, Direction,
};

let spec = ChannelSpec::new(
    membership,
    Combinator::Any,
    Direction::Along,
);

let mut seeds = BTreeMap::new();
seeds.insert(child, ChannelValue::Flag(true));

let channel = graph.evaluate(spec, &seeds);

assert_eq!(channel.value(parent), Some(ChannelValue::Flag(true)));
```

Supported combinators:

| Combinator      |     Seed type |   Output type | Meaning                                      |
| --------------- | ------------: | ------------: | -------------------------------------------- |
| `Any`           |  `Flag(bool)` |  `Flag(bool)` | Whether any reachable present seed is true   |
| `All`           |  `Flag(bool)` |  `Flag(bool)` | Whether all reachable present seeds are true |
| `Max`           | `Scalar(i64)` | `Scalar(i64)` | Maximum reachable scalar                     |
| `CountDistinct` |  `Flag(bool)` |  `Count(u32)` | Count of distinct reachable true seeds       |

`Any`, `All`, and `Max` include the node’s own valid seed in the fold. `CountDistinct` uses strict reachability and does not count the node itself.

Invalid seeds are not silently coerced. A seed for an unknown node, or a seed whose value variant does not match the selected combinator, is reported in `Channel::diagnostics`.

## Ordering

`cordage` can compose an ordering from one or more overlays.

```rust
use cordage::{Direction, OrderLayer, OrderSpec};

builder.order_spec(OrderSpec::new(vec![
    OrderLayer::new(dependencies, Direction::Along),
    OrderLayer::new(sequence, Direction::Along),
]));

let graph = builder.build().expect("valid graph");

let ordered_nodes = graph.ordered();
```

Ordering is deterministic. If the requested ordering layers conflict, `cordage` preserves earlier-layer authority and records later-layer evictions in provenance rather than silently producing an arbitrary order.

## Provenance and explanation

Build-time cycle handling and edge eviction are surfaced as structured data.

```rust
let provenance = graph.provenance();

for eviction in provenance.evictions() {
    println!("{:?}", eviction.reason());
}
```

For a single node, `explain` returns a structured account containing:

* the node’s composed order key
* predecessor cones per overlay
* evicted edges involving that node

The core does not render prose. Callers decide how to present explanations in their own domain language.

## Design goals

### Deterministic

`cordage` uses ordered data structures and explicit ordering keys so equivalent inputs produce equivalent outputs.

### Product-neutral

The crate has no application vocabulary. Callers provide the meaning of overlays and channels.

### Build, then query

Graph construction is separated from graph querying. `GraphBuilder` collects input. `build()` validates malformed references, resolves configured conflicts, and produces an immutable `Graph`.

### Zero runtime dependencies

`cordage` has no runtime dependencies by design.

## When to use `cordage`

Use `cordage` when you need a small, deterministic graph-evaluation core for:

* policy engines
* rule evaluation
* workflow ordering
* document or job dependency models
* build or planning systems
* explainable propagation over typed relationships
* systems where cycles should be diagnosed or resolved predictably

Use another graph crate when you primarily need:

* shortest paths
* centrality algorithms
* min/max spanning trees
* graph mutation after construction
* Graphviz export
* large-scale graph performance
* broad ecosystem integration

## Status

`cordage` is young. The public API is intentionally small, but the semantics are specific. Expect the crate to be most useful when you want its model — typed overlays plus deterministic evaluation — rather than a general graph toolkit.
