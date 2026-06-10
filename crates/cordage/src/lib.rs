//! `cordage` — a generic multi-channel evaluation engine over a tree plus typed
//! directed (DAG) overlays.
//!
//! The crate is product-neutral: it carries no `dep`/`seq`/backlog vocabulary and
//! orders edges by opaque attributes it never interprets. Consumers (doctrine's
//! policy and adapter layers, and other products) build a [`Graph`] from a
//! [`GraphBuilder`], then query ordering, reachability, and channel propagation.
//!
//! This module is the crate's full public surface. Public *types* are declared
//! here at the crate root (a flat, re-export-free API — the repo lint posture
//! forbids `pub use`); implementation logic is split into private modules as the
//! engine grows, so the public path shape stays stable across phases.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

// ── identity ─────────────────────────────────────────────────────────────────
// Opaque, builder-allocated, monotonic; no public constructor and no accessor for
// the inner ordinal — callers treat ids as tokens. The adapter (a later slice)
// maps doctrine ids ↔ these.

/// Opaque node identity. Allocated by [`GraphBuilder::node`]; monotonic, no
/// deletion in v1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(u32);

/// Opaque overlay identity. Allocated by [`GraphBuilder::overlay`]; the `u16` cap
/// is a documented build-input limit ([`BuildError::OverlayCapExceeded`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OverlayId(u16);

// ── overlay config ───────────────────────────────────────────────────────────

/// How an overlay's cycles are handled at build time: `Reject` diagnoses, `Evict`
/// resolves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CyclePolicy {
    Reject,
    Evict,
}

/// Incoming-edge cardinality. `AtMostOne` overlays are spine-capable (single kept
/// parent); `Unbounded` overlays carry multi-parent membership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arity {
    AtMostOne,
    Unbounded,
}

/// Per-overlay configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverlayConfig {
    cycle_policy: CyclePolicy,
    arity: Arity,
}

impl OverlayConfig {
    /// Construct an overlay configuration.
    pub fn new(cycle_policy: CyclePolicy, arity: Arity) -> Self {
        Self {
            cycle_policy,
            arity,
        }
    }

    /// The cycle policy.
    pub fn cycle_policy(self) -> CyclePolicy {
        self.cycle_policy
    }

    /// The incoming-edge arity.
    pub fn arity(self) -> Arity {
        self.arity
    }
}

/// Opaque per-edge ordering attributes. The core orders by these but never
/// interprets them: higher `rank` wins keeps; lower `rank` is evicted first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeAttrs {
    rank: i32,
    age: u64,
}

impl EdgeAttrs {
    /// Construct edge attributes.
    pub fn new(rank: i32, age: u64) -> Self {
        Self { rank, age }
    }

    /// The rank ordinal.
    pub fn rank(&self) -> i32 {
        self.rank
    }

    /// The age ordinal.
    pub fn age(&self) -> u64 {
        self.age
    }
}

// ── channels ─────────────────────────────────────────────────────────────────

/// Commutative-monoid channel combinators; each owns a value domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    Max,
    Any,
    All,
    CountDistinct,
}

/// Traversal direction. `Along` walks out-edges, `Against` walks in-edges, `None`
/// performs no traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Along,
    Against,
    None,
}

/// A channel value. The variant a [`Combinator`] consumes/emits is fixed by its
/// domain (see the propagation contract).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelValue {
    Flag(bool),
    Scalar(i64),
    Count(u32),
}

/// The discriminant of a [`ChannelValue`], used to report seed-variant mismatches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Flag,
    Scalar,
    Count,
}

/// A single channel evaluation request: which overlay, combinator, and direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelSpec {
    overlay: OverlayId,
    combinator: Combinator,
    direction: Direction,
}

impl ChannelSpec {
    /// Construct a channel spec.
    pub fn new(overlay: OverlayId, combinator: Combinator, direction: Direction) -> Self {
        Self {
            overlay,
            combinator,
            direction,
        }
    }

    /// The overlay this channel propagates over.
    pub fn overlay(self) -> OverlayId {
        self.overlay
    }

    /// The combinator.
    pub fn combinator(self) -> Combinator {
        self.combinator
    }

    /// The traversal direction.
    pub fn direction(self) -> Direction {
        self.direction
    }
}

// ── ordering composition ─────────────────────────────────────────────────────

/// One precedence layer of an [`OrderSpec`]: an overlay viewed in a direction.
/// `Direction::None` is malformed here (a layer must traverse).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderLayer {
    overlay: OverlayId,
    direction: Direction,
}

impl OrderLayer {
    /// Construct an order layer.
    pub fn new(overlay: OverlayId, direction: Direction) -> Self {
        Self { overlay, direction }
    }
}

/// An ordering specification: precedence-ordered layers. Empty = pure-`NodeId`
/// order. An overlay may appear in at most one layer, in either direction.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrderSpec {
    layers: Vec<OrderLayer>,
}

impl OrderSpec {
    /// Construct an order spec from precedence-ordered layers.
    pub fn new(layers: Vec<OrderLayer>) -> Self {
        Self { layers }
    }
}

// ── build-input errors ───────────────────────────────────────────────────────

/// A malformed build input. `build()` errors **only** on malformed input; cycles,
/// evictions, and degradation are data in provenance, never errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    /// An edge referenced a node id this builder never allocated.
    UnknownNode(NodeId),
    /// An edge or order layer referenced an overlay id this builder never allocated.
    UnknownOverlay(OverlayId),
    /// An overlay appeared in more than one `OrderSpec` layer.
    OverlayRepeatedInOrderSpec(OverlayId),
    /// An `OrderSpec` layer used `Direction::None`.
    DirectionNoneLayer(OverlayId),
    /// More overlays were allocated than the `u16` id space allows.
    OverlayCapExceeded,
    /// More nodes were allocated than the `u32` id space allows.
    NodeCapExceeded,
}

// ── internal adjacency edges ─────────────────────────────────────────────────
// Two distinct structs with *explicit* `Ord` over the documented adjacency key
// (F21 — never derive-incidental). `BTreeSet` membership then gives set-dedupe of
// identical edges for free (the key spans every field).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OutEdge {
    dst: NodeId,
    rank: i32,
    age: u64,
}

impl Ord for OutEdge {
    fn cmp(&self, other: &Self) -> Ordering {
        // out-sets order by (dst, rank, age) — traversal determinism only.
        (self.dst, self.rank, self.age).cmp(&(other.dst, other.rank, other.age))
    }
}

impl PartialOrd for OutEdge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InEdge {
    src: NodeId,
    rank: i32,
    age: u64,
}

impl Ord for InEdge {
    fn cmp(&self, other: &Self) -> Ordering {
        // in-sets order by (src, rank, age).
        (self.src, self.rank, self.age).cmp(&(other.src, other.rank, other.age))
    }
}

impl PartialOrd for InEdge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy)]
struct RawEdge {
    overlay: OverlayId,
    src: NodeId,
    dst: NodeId,
    attrs: EdgeAttrs,
}

type OutIndex = BTreeMap<OverlayId, BTreeMap<NodeId, BTreeSet<OutEdge>>>;
type InIndex = BTreeMap<OverlayId, BTreeMap<NodeId, BTreeSet<InEdge>>>;

// ── builder ──────────────────────────────────────────────────────────────────

/// Accumulates nodes, overlays, edges, and an optional [`OrderSpec`], then
/// validates them in [`GraphBuilder::build`]. Mutators return bare ids and never
/// validate; all validation defers to `build()`.
#[derive(Debug, Default)]
pub struct GraphBuilder {
    overlays: Vec<OverlayConfig>,
    node_count: u32,
    edges: Vec<RawEdge>,
    order_spec: OrderSpec,
    overlay_overflow: bool,
    node_overflow: bool,
}

impl GraphBuilder {
    /// A fresh, empty builder.
    pub fn new() -> Self {
        Self {
            overlays: Vec::new(),
            node_count: 0,
            edges: Vec::new(),
            order_spec: OrderSpec { layers: Vec::new() },
            overlay_overflow: false,
            node_overflow: false,
        }
    }

    /// Allocate an overlay. Exceeding the `u16` id space is recorded and surfaces
    /// as [`BuildError::OverlayCapExceeded`] at `build()`.
    pub fn overlay(&mut self, config: OverlayConfig) -> OverlayId {
        let Ok(id) = u16::try_from(self.overlays.len()) else {
            self.overlay_overflow = true;
            return OverlayId(u16::MAX);
        };
        self.overlays.push(config);
        OverlayId(id)
    }

    /// Allocate a node. Exceeding the `u32` id space is recorded and surfaces as
    /// [`BuildError::NodeCapExceeded`] at `build()`.
    pub fn node(&mut self) -> NodeId {
        let id = self.node_count;
        let Some(next) = self.node_count.checked_add(1) else {
            self.node_overflow = true;
            return NodeId(u32::MAX);
        };
        self.node_count = next;
        NodeId(id)
    }

    /// Add a directed edge on an overlay. Unknown ids are not validated here —
    /// `build()` does.
    pub fn edge(&mut self, overlay: OverlayId, src: NodeId, dst: NodeId, attrs: EdgeAttrs) {
        self.edges.push(RawEdge {
            overlay,
            src,
            dst,
            attrs,
        });
    }

    /// Set the order spec (replacing any previous).
    pub fn order_spec(&mut self, spec: OrderSpec) {
        self.order_spec = spec;
    }

    /// Validate the accumulated inputs and assemble the [`Graph`].
    ///
    /// # Errors
    ///
    /// Returns a [`BuildError`] for malformed input **only** — an edge or layer
    /// referencing an unallocated node/overlay id, an overlay repeated across
    /// `OrderSpec` layers, a `Direction::None` layer, or an exceeded id cap.
    /// Cycles, evictions, and degradation are never errors.
    pub fn build(self) -> Result<Graph, BuildError> {
        if self.overlay_overflow {
            return Err(BuildError::OverlayCapExceeded);
        }
        if self.node_overflow {
            return Err(BuildError::NodeCapExceeded);
        }

        let overlay_count = self.overlays.len();
        for edge in &self.edges {
            if usize::from(edge.overlay.0) >= overlay_count {
                return Err(BuildError::UnknownOverlay(edge.overlay));
            }
            if edge.src.0 >= self.node_count {
                return Err(BuildError::UnknownNode(edge.src));
            }
            if edge.dst.0 >= self.node_count {
                return Err(BuildError::UnknownNode(edge.dst));
            }
        }

        let mut seen: BTreeSet<OverlayId> = BTreeSet::new();
        for layer in &self.order_spec.layers {
            if usize::from(layer.overlay.0) >= overlay_count {
                return Err(BuildError::UnknownOverlay(layer.overlay));
            }
            if matches!(layer.direction, Direction::None) {
                return Err(BuildError::DirectionNoneLayer(layer.overlay));
            }
            if !seen.insert(layer.overlay) {
                return Err(BuildError::OverlayRepeatedInOrderSpec(layer.overlay));
            }
        }

        let (out, incoming) = build_indices(&self.edges);
        Ok(Graph { out, incoming })
    }
}

fn build_indices(edges: &[RawEdge]) -> (OutIndex, InIndex) {
    let mut out: OutIndex = BTreeMap::new();
    let mut incoming: InIndex = BTreeMap::new();
    for edge in edges {
        out.entry(edge.overlay)
            .or_default()
            .entry(edge.src)
            .or_default()
            .insert(OutEdge {
                dst: edge.dst,
                rank: edge.attrs.rank,
                age: edge.attrs.age,
            });
        incoming
            .entry(edge.overlay)
            .or_default()
            .entry(edge.dst)
            .or_default()
            .insert(InEdge {
                src: edge.src,
                rank: edge.attrs.rank,
                age: edge.attrs.age,
            });
    }
    (out, incoming)
}

// ── graph & queries ──────────────────────────────────────────────────────────

/// An assembled graph: `BTreeMap` adjacency with a derived reverse index. The
/// phase-1 surface is the two adjacency views; resolution passes and richer
/// queries land in later phases.
#[derive(Debug)]
pub struct Graph {
    out: OutIndex,
    incoming: InIndex,
}

impl Graph {
    /// Out-edges of `node` on `overlay`, as `(neighbour, attrs)` ordered by the
    /// `(dst, rank, age)` adjacency key. A foreign/unknown id yields an empty
    /// iterator — defined, non-panicking.
    pub fn out_edges(
        &self,
        overlay: OverlayId,
        node: NodeId,
    ) -> impl Iterator<Item = (NodeId, EdgeAttrs)> + '_ {
        self.out
            .get(&overlay)
            .and_then(|m| m.get(&node))
            .into_iter()
            .flat_map(|set| set.iter().map(|e| (e.dst, EdgeAttrs::new(e.rank, e.age))))
    }

    /// In-edges of `node` on `overlay`, as `(neighbour, attrs)` ordered by the
    /// `(src, rank, age)` adjacency key. A foreign/unknown id yields an empty
    /// iterator — defined, non-panicking.
    pub fn in_edges(
        &self,
        overlay: OverlayId,
        node: NodeId,
    ) -> impl Iterator<Item = (NodeId, EdgeAttrs)> + '_ {
        self.incoming
            .get(&overlay)
            .and_then(|m| m.get(&node))
            .into_iter()
            .flat_map(|set| set.iter().map(|e| (e.src, EdgeAttrs::new(e.rank, e.age))))
    }
}
