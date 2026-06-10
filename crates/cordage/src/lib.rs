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

mod query;
mod resolve;

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
///
/// **The REQ-080 extension seam.** A *fresh channel* — a new propagated meaning
/// (blocking, staleness, a priority rollup, …) — is composed by a caller pairing
/// one of these combinators with an overlay and a [`Direction`] in a
/// [`ChannelSpec`], then seeding [`Graph::evaluate`]. No core edit is needed to
/// add a channel; the channel's *meaning* lives entirely in the caller. This enum
/// is the one deliberately **curated** extension point: adding a combinator is the
/// only channel change that touches the core, so the monoid set stays small and
/// each variant's domain/laws stay auditable (it is not an open registry).
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

// ── channel propagation result ───────────────────────────────────────────────
// The structured output of `evaluate` (design §5.4). No `String` prose, no
// channel name, no doctrine noun — the `Combinator` is NOT carried: a `Channel`
// is per-`evaluate`, the spec stays in the caller's hand (F40 partial).

/// Why a seed entry could not contribute, surfaced rather than silently dropped.
/// `UnknownSeedNode` wins over `SeedVariantMismatch` when both could apply to one
/// node (F41); at most one diagnostic per node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelDiagReason {
    /// The seed's [`ChannelValue`] variant is outside the combinator's domain
    /// (`Any`/`All`/`CountDistinct` consume `Flag`, `Max` consumes `Scalar`). The
    /// seed is treated as absent.
    SeedVariantMismatch {
        expected: ValueKind,
        actual: ValueKind,
    },
    /// The seed map keyed a [`NodeId`] this graph never allocated. Ignored.
    UnknownSeedNode,
}

/// A single seed-rejection diagnostic from `evaluate`, naming the node and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelDiagnostic {
    node: NodeId,
    reason: ChannelDiagReason,
}

impl ChannelDiagnostic {
    /// The node whose seed was rejected.
    pub fn node(self) -> NodeId {
        self.node
    }

    /// Why the seed was rejected.
    pub fn reason(self) -> ChannelDiagReason {
        self.reason
    }
}

/// The result of a single `evaluate`: per-node folded values, the contributing
/// node sets, and any seed diagnostics. A node absent from `values` had no
/// present seed in its fold set — no combinator identity ever escapes as output
/// (F16). `contributors` and `diagnostics` follow §5.4: contributors are
/// `Any`→present-true witnesses · `All`→present-false set if false / present-true
/// set if true · `Max`→argmax (min-`NodeId` tie) · `CountDistinct`→the counted
/// set; diagnostics are sorted by `NodeId`, at most one per node.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Channel {
    values: BTreeMap<NodeId, ChannelValue>,
    contributors: BTreeMap<NodeId, BTreeSet<NodeId>>,
    diagnostics: Vec<ChannelDiagnostic>,
}

impl Channel {
    /// The folded value of `node`, or `None` if no present seed reached it.
    pub fn value(&self, node: NodeId) -> Option<ChannelValue> {
        self.values.get(&node).copied()
    }

    /// Every node with a folded value, keyed by [`NodeId`].
    pub fn values(&self) -> &BTreeMap<NodeId, ChannelValue> {
        &self.values
    }

    /// The contributing node set behind `node`'s value (empty slice if none).
    pub fn contributors(&self, node: NodeId) -> &BTreeSet<NodeId> {
        static EMPTY: BTreeSet<NodeId> = BTreeSet::new();
        self.contributors.get(&node).unwrap_or(&EMPTY)
    }

    /// The seed diagnostics, sorted by [`NodeId`], at most one per node.
    pub fn diagnostics(&self) -> &[ChannelDiagnostic] {
        &self.diagnostics
    }
}

/// A role-agnostic structured account of one node (design §5.4, D11/F13): its
/// composed-order key, its transitive predecessor chains per overlay, and the
/// evictions it is an endpoint of. No `String` prose, no policy role — rendering
/// belongs to the policy layer.
///
/// `paths` keys every overlay; each value is the predecessor chains of `node`,
/// oriented **root → … → node**. A chain ends at a root OR at the first node of a
/// degraded post-arity SCC (F47): SCC members are chain ENDPOINTS only, never
/// walked through, so the account is finite and deterministic on a cyclic `Reject`
/// view. A node inside a degraded SCC gets the singleton chain `[[node]]`; the
/// cycle itself is explained by [`Provenance::cycles`], not a path. `evicted` is
/// the evictions with `node` as src or dst (F26).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explanation {
    node: NodeId,
    order_key: OrderKey,
    paths: BTreeMap<OverlayId, Vec<Vec<NodeId>>>,
    evicted: Vec<EvictedEdge>,
}

impl Explanation {
    /// The explained node.
    pub fn node(&self) -> NodeId {
        self.node
    }

    /// The node's composed-order key.
    pub fn order_key(&self) -> OrderKey {
        self.order_key
    }

    /// The transitive predecessor chains of the node, keyed by overlay; each
    /// chain is oriented root → … → node (F47 termination).
    pub fn paths(&self) -> &BTreeMap<OverlayId, Vec<Vec<NodeId>>> {
        &self.paths
    }

    /// The evictions with the node as an endpoint (src or dst), in provenance
    /// order (F26).
    pub fn evicted(&self) -> &[EvictedEdge] {
        &self.evicted
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

/// A node's composed-order level: its longest-path depth in the order structure
/// `U`, tagged clean (`Finite`) or cycle-degraded (`Degraded`). Every `Degraded`
/// sorts after every `Finite` — taint propagates downstream, so no surviving `U`
/// edge ever runs tainted→clean (F11/F33).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Finite(u32),
    Degraded(u32),
}

/// A node's total-order key: its [`Level`] then its [`NodeId`]. `(level, node)`
/// is total within each variant, so [`Graph::ordered`] is deterministic and
/// respects every surviving `U` edge (I2/I3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrderKey {
    level: Level,
    node: NodeId,
}

impl OrderKey {
    /// The composed-order level.
    pub fn level(self) -> Level {
        self.level
    }

    /// The node this key orders.
    pub fn node(self) -> NodeId {
        self.node
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

// ── provenance ───────────────────────────────────────────────────────────────
// Build-time resolution is surfaced, never silent: cycles, arity-keep losers, and
// order-composition conflicts are reported as data here (D5/D8). No `String` prose,
// no role, no doctrine noun (F13) — rendering is the policy layer's.

/// A reference to a single edge: its endpoints and ordering attributes. Carries
/// `attrs` to disambiguate parallel edges (same endpoints, differing rank/age).
/// Ordered by `(src, dst, rank, age)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeRef {
    src: NodeId,
    dst: NodeId,
    attrs: EdgeAttrs,
}

impl EdgeRef {
    /// The edge source.
    pub fn src(&self) -> NodeId {
        self.src
    }

    /// The edge destination.
    pub fn dst(&self) -> NodeId {
        self.dst
    }

    /// The edge ordering attributes.
    pub fn attrs(&self) -> EdgeAttrs {
        self.attrs
    }
}

impl Ord for EdgeRef {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.src, self.dst, self.attrs.rank(), self.attrs.age()).cmp(&(
            other.src,
            other.dst,
            other.attrs.rank(),
            other.attrs.age(),
        ))
    }
}

impl PartialOrd for EdgeRef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Why an edge was removed during build resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictReason {
    /// Lost the single-parent contest on an `AtMostOne` overlay (pass 1).
    ArityViolation,
    /// Removed to break a cycle on an `Evict` overlay (pass 2).
    IntraOverlayCycle,
    /// Removed from the composed order structure to break a cross-layer cycle
    /// (pass 3 — PHASE-03).
    UnionCycleVsLayer,
}

/// An edge removed by a resolution pass, with the overlay it belonged to and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvictedEdge {
    overlay: OverlayId,
    edge: EdgeRef,
    reason: EvictReason,
}

impl EvictedEdge {
    /// The overlay the evicted edge belonged to.
    pub fn overlay(&self) -> OverlayId {
        self.overlay
    }

    /// The evicted edge.
    pub fn edge(&self) -> EdgeRef {
        self.edge
    }

    /// Why it was evicted.
    pub fn reason(&self) -> EvictReason {
        self.reason
    }
}

/// A diagnosed cyclic component on a `Reject` overlay (REQ-076): the nodes and
/// participating edges of one strongly-connected component. A self-loop is a
/// single-node cycle (F20). Reported, never linearized — `build()` still returns
/// `Ok` (a cycle is data, an authoring error to surface).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleDiagnostic {
    overlay: OverlayId,
    nodes: BTreeSet<NodeId>,
    edges: Vec<EdgeRef>,
}

impl CycleDiagnostic {
    /// The overlay the cycle was found on.
    pub fn overlay(&self) -> OverlayId {
        self.overlay
    }

    /// The nodes of the cyclic component.
    pub fn nodes(&self) -> &BTreeSet<NodeId> {
        &self.nodes
    }

    /// The edges participating in the cyclic component, sorted.
    pub fn edges(&self) -> &[EdgeRef] {
        &self.edges
    }
}

/// Build-time resolution provenance: the cycles diagnosed and the edges evicted.
/// Both are sorted by `(overlay, edge)` for deterministic reporting (F21) — a
/// reporting order distinct from the F17 eviction *selection* key (F37).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Provenance {
    cycles: Vec<CycleDiagnostic>,
    evictions: Vec<EvictedEdge>,
}

impl Provenance {
    /// The diagnosed cyclic components, sorted by `(overlay, nodes)`.
    pub fn cycles(&self) -> &[CycleDiagnostic] {
        &self.cycles
    }

    /// The evicted edges, sorted by `(overlay, edge)`.
    pub fn evictions(&self) -> &[EvictedEdge] {
        &self.evictions
    }
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

        let resolution = resolve::resolve(&self.edges, &self.overlays);
        let (out, incoming) = build_indices(&resolution.edges);
        let mut graph = Graph {
            out,
            incoming,
            provenance: resolution.provenance,
            degraded_sccs: resolution.degraded_sccs,
            order_spec: self.order_spec,
            overlays: self.overlays,
            node_count: self.node_count,
            order_keys: BTreeMap::new(),
        };
        graph.compose_order();
        Ok(graph)
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
    provenance: Provenance,
    /// Cyclic post-arity SCCs of `Reject` overlays (F46) — the taint seeds, read
    /// by `compose_order` (pass 4) and retained for later explain (F47).
    degraded_sccs: BTreeMap<OverlayId, Vec<BTreeSet<NodeId>>>,
    /// The validated order spec, re-stored from the builder; read by
    /// `compose_order` and available to later phases.
    order_spec: OrderSpec,
    /// Per-overlay configs, re-stored from the builder and indexed by
    /// `OverlayId` ordinal. Read by `spine_path` (the `AtMostOne` check) and
    /// `evaluate`.
    overlays: Vec<OverlayConfig>,
    /// Total node count — the level recurrence is total over `0..node_count`.
    node_count: u32,
    /// Per-node composed-order key (pass 4), total over all nodes.
    order_keys: BTreeMap<NodeId, OrderKey>,
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

    /// Build-time resolution provenance: the cycles diagnosed and edges evicted
    /// while assembling this graph. Empty when nothing was resolved.
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// The strict reachable set of `node` on `overlay` in `direction` — the
    /// transitive successors, **excluding `node` itself** even when cyclically
    /// reachable (I6/F8). `Along` walks out-edges, `Against` in-edges, `None`
    /// yields the empty set (F25). A foreign overlay or node yields the empty set
    /// (F14). Total and terminating over a degraded `Reject` view (F12) — it
    /// reads the resolved per-overlay adjacency, never the composed order (I7).
    pub fn reachable(
        &self,
        overlay: OverlayId,
        node: NodeId,
        direction: Direction,
    ) -> BTreeSet<NodeId> {
        query::reachable(&self.out, &self.incoming, overlay, node, direction)
    }

    /// The spine chain of `node` on `overlay`, **root → … → node** — or `None`
    /// unless `overlay` is `AtMostOne` (F23): only spine-capable overlays have a
    /// single kept parent per node (pass-1 arity). The DD1 ergonomic accessor —
    /// the core privileges no overlay as "the" spine; a policy labels one
    /// `AtMostOne` overlay's path itself. A foreign overlay yields `None`.
    pub fn spine_path(&self, overlay: OverlayId, node: NodeId) -> Option<Vec<NodeId>> {
        let config = self.overlays.get(usize::from(overlay.0))?;
        if !matches!(config.arity(), Arity::AtMostOne) {
            return None;
        }
        Some(query::spine_path(&self.incoming, overlay, node))
    }

    /// Propagate a channel: fold `seeds` over `spec`'s overlay/combinator/
    /// direction and return the per-node [`Channel`] (design §5.2). Idempotent
    /// combinators (`Any`/`All`/`Max`) fold present seeds over `{n} ∪ reachable`;
    /// `CountDistinct` counts `Flag(true)` seeds over strict `reachable` (F34).
    /// The seed contract holds (F16): a node whose fold set has no present seed is
    /// absent from `values` — no combinator identity escapes; foreign-node and
    /// variant-mismatched seeds surface as [`ChannelDiagnostic`]s, never silently
    /// coerced (F41). Pure; reads the resolved adjacency, invariant under the
    /// `OrderSpec` (I7/F18).
    pub fn evaluate(&self, spec: ChannelSpec, seeds: &BTreeMap<NodeId, ChannelValue>) -> Channel {
        query::evaluate(&self.out, &self.incoming, self.node_count, spec, seeds)
    }

    /// The composed-order key of `node`, or `None` for a foreign/unknown id
    /// (defined, non-panicking — F14).
    pub fn order_key(&self, node: NodeId) -> Option<OrderKey> {
        self.order_keys.get(&node).copied()
    }

    /// A role-agnostic structured account of `node` (design §5.4): its
    /// composed-order key, its transitive predecessor chains per overlay (each
    /// oriented root → … → node, terminating at a root or a degraded post-arity
    /// SCC entry — F47), and the evictions it is an endpoint of (F26). Finite and
    /// deterministic on cyclic `Reject` views; infallible (a foreign id yields the
    /// `Finite(0)` key and empty paths/evictions, non-panicking — F14).
    pub fn explain(&self, node: NodeId) -> Explanation {
        let order_key = self.order_keys.get(&node).copied().unwrap_or(OrderKey {
            level: Level::Finite(0),
            node,
        });
        let paths = query::predecessor_paths(&self.incoming, &self.degraded_sccs, node);
        let evicted = self
            .provenance
            .evictions()
            .iter()
            .filter(|e| e.edge().src() == node || e.edge().dst() == node)
            .copied()
            .collect();
        Explanation {
            node,
            order_key,
            paths,
            evicted,
        }
    }

    /// Every node in composed total order (by `OrderKey`). An empty `OrderSpec`
    /// yields pure-`NodeId` order (all `Finite(0)`).
    pub fn ordered(&self) -> Vec<NodeId> {
        let mut keys: Vec<OrderKey> = self.order_keys.values().copied().collect();
        keys.sort();
        keys.into_iter().map(|k| k.node).collect()
    }

    /// Passes 3–4 (design §5.4): compose the order structure `U` from the order
    /// spec and resolved adjacency, then materialise per-node [`OrderKey`]s.
    /// Cross-layer evictions are merged into provenance; they touch `U` alone
    /// (I7/F18).
    fn compose_order(&mut self) {
        let outcome = resolve::compose_order(
            &self.out,
            &self.order_spec,
            &self.degraded_sccs,
            self.node_count,
        );
        self.order_keys = outcome.order_keys;
        if !outcome.evictions.is_empty() {
            self.provenance.evictions.extend(outcome.evictions);
            resolve::sort_provenance(&mut self.provenance);
        }
    }
}
