# Design SL-226: CLI graph emitter and ego-view

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

The semantic entity graph has exactly one rendering surface — the web explorer
(`map serve`, PRD-016) — and no CLI emission. RFC-001: graph value is gated on
consumption surfaces. Deliver `doctrine graph`: DOT/JSON emission of the
semantic graph to stdout — whole-corpus or focus+depth ego-view, with
kind/label/memory filters — pipe-composable (`| gvpr … | dot -Tsvg`).

## 2. Current State

- `CatalogGraph` (`src/catalog/graph.rs`, SL-071): pure projection, nodes
  (BTreeMap) + flat edge list + units. `outgoing`/`incoming` exist but are
  `#[expect(dead_code, "future consumer")]`; `neighbours(depth)` deferred
  (SL-071 D10). This slice is the anticipated consumer.
- `doctrine catalog graph`: whole-corpus JSON dump of `CatalogGraph` (debug
  tier); same serde contract as `/api/graph`.
- DOT serialization + focus/depth bounding exist **only client-side**:
  `web/map/src/dot.ts` (style tables, deterministic emission),
  `web/map/src/model.ts::bfsCore/neighbourhood` (undirected BFS, [0,3] clamp,
  edge dedup, depth 0 = focus alone).
- `src/concept_map.rs` has a private `dot_escape` + minimal `render_dot` for
  the concept-map kind — unstyled, separate concern.

## 3. Forces & Constraints

- ADR-001 layering: emitter/projection pure (engine/leaf); command thin.
- STD-001: style tables and defaults as named constants, no magic strings.
- Behaviour-preservation gate: entity-engine and concept-map suites stay green
  unchanged.
- Clippy denies `print_stdout`; output via `writeln!(std::io::stdout(), …)`
  as in `src/commands/relation.rs`.
- PRD-016 §2 demotes *static file interchange for external tools*; this verb
  is an in-workflow consumption surface (RFC-001), distinguishable — but the
  PRD-016 boundary sentence must be revisited at reconcile (slice scope,
  Context).
- CLI vocabulary congruence: `relation list` (`--label`, `--include-memory`,
  `--format`, `-p/--path`), `map serve` (`--depth`), `concept-map export`
  (`--format dot|json|…`).

## 4. Guiding Principles

- Composability is the product: deterministic, silent, pipe-clean stdout.
- Emit the projection asked for — no gates, no chatter (DEC-009).
- Presentation policy per surface: the Rust style tables carry no parity
  contract with `dot.ts` (DEC-008).
- Ride the pre-cut seam (SL-071 D10) — no parallel graph machinery.

## 5. Proposed Design

### 5.1 System Model

```
commands/graph.rs (thin shell)
  └─ scan_catalog → CatalogGraph::from_catalog          (existing)
       └─ project: neighbourhood(focus, depth) | whole  (new, graph.rs)
            └─ filters: kinds / label / memory          (new, graph.rs)
                 ├─ --format json → serde (existing contract shape)
                 └─ --format dot  → catalog/dot.rs emit (new, pure)
```

### 5.2 Interfaces & Contracts

CLI:

```
doctrine graph [FOCUS] [--depth N] [--kind K]... [--label L]
               [--include-memory] [--format dot|json] [-p|--path ROOT]
```

- No FOCUS → whole-corpus projection, silent (DEC-009). FOCUS = canonical
  entity ref or memory ref, resolved against the **filtered universe** (D6):
  unknown id → "not found" error; id that exists but is excluded by
  `--kind`/memory default → error naming the excluding filter. Both nonzero.
- `--depth N`: any N ≥ 0, default 1, no clamp; 0 = focus alone (DEC-009).
- `--kind K` repeatable (prefix, normalized to uppercase); `--label L` edge
  filter; `--include-memory` off by default. Filters define the universe
  **before** bounding (D4).
- `--format dot` (default) | `json` — json serializes the projected subgraph
  through the same `CatalogGraph` contract shape (`nodes`/`edges`/`units`)
  that `catalog graph` and `/api/graph` emit (DEC-007).

Rust (all `pub(crate)`):

```rust
// src/catalog/graph.rs
pub(crate) struct Subgraph<'g> {
  pub(crate) nodes: BTreeMap<&'g NodeKey, &'g CatalogNode>,
  pub(crate) edges: Vec<&'g CatalogEdge>,
  pub(crate) units: &'g Units,
}
impl CatalogGraph {
  pub(crate) fn whole(&self) -> Subgraph<'_>;
}
impl Subgraph<'_> {
  // D4: filters apply to the universe BEFORE neighbourhood bounding —
  // whole() → filter chain → optional neighbourhood(focus, depth).
  pub(crate) fn retain_kinds(self, prefixes: &[String]) -> Self;  // uppercased
  pub(crate) fn retain_label(self, label: &str) -> Self;
  pub(crate) fn exclude_memory(self) -> Self;
  pub(crate) fn neighbourhood(self, focus: &NodeKey, depth: u32) -> Result<Self, FocusError>;
}

// src/catalog/dot.rs
pub(crate) fn dot_escape(s: &str) -> String;               // lifted from concept_map.rs
pub(crate) fn render(sub: &Subgraph, focus: Option<&NodeKey>) -> String;
```

### 5.3 Data, State & Ownership

- `Subgraph` borrows from `CatalogGraph` — no cloning, no new ownership; it is
  a view, serialized or rendered then dropped. Serialize via a mirroring
  `serde::Serialize` impl producing the established contract keys.
- Style tables in `catalog/dot.rs`: `NODE_STYLES: &[(&str, NodeStyle)]`,
  `EDGE_COLORS: &[(&str, EdgeColor)]` — named constants (STD-001), values
  ported from `dot.ts` (both tables, DEC-008). Unknown kind/label falls back
  to the ported defaults.

### 5.4 Lifecycle, Operations & Dynamics

- One-shot: scan → filter → bound → emit → exit. No daemon, no cache; A1
  (scope) holds — the same hydration already backs `map serve` startup.
- BFS: undirected (out + in), visited-set, edge dedup by identity tuple
  (source, label, role, target), breadth-bounded at `depth`. Port of
  `model.ts::bfsCore` minus the clamp. Adjacency maps (in/out per node) are
  built once over the filtered edge set before traversal — no per-node O(E)
  scans; the existing `outgoing`/`incoming` helpers and their
  `expect(dead_code)` markers are untouched (they serve per-node queries, not
  traversal).
- Edges to `UnresolvedRef`/`UnvalidatedText` targets are **collected** when
  their source is visited (so D5 ghosts appear) but never **enqueued** — a
  dangling target has no node to expand.
- Emission order (byte-determinism): nodes in BTreeMap key order; edges sorted
  by (source, label, role, target). Trailing newline; nothing on stderr on
  success.
- DOT semantics ported from `dot.ts::graphToDot`: `rankdir=LR`,
  `bgcolor="transparent"` (DEC-008), `nodesep`/`ranksep` as in dot.ts, focus
  `penwidth=3.0`, MEM nodes labelled by title (others by id), tooltip
  `id: title · kind · status`.

### 5.5 Invariants, Assumptions & Edge Cases

- Every emitted edge's endpoints exist in the emitted DOT: resolved targets as
  styled nodes; `UnresolvedRef`/`UnvalidatedText` targets as dashed grey
  **ghost nodes** labelled with the raw text (D5), with namespaced ids
  (`"?:<raw>"`) so a free-text target can never collide with a real node key.
  Identical raw texts share one ghost node. JSON is unaffected — edges already
  carry the target variant.
- Depth 0 emits exactly the focus node, zero edges.
- Filters compose conjunctively and define the universe **before** bounding
  (D4): `retain_kinds`/`exclude_memory` restrict nodes (and their incident
  edges), `retain_label` restricts edges — so BFS reach travels only the
  filtered universe (label-chain following works). In whole-corpus mode,
  `retain_label` additionally drops nodes with no surviving incident edge
  (edge-induced node set).
- Empty projection (filters exclude everything) emits a valid empty
  `digraph`/empty-collections JSON — not an error.
- D6: FOCUS resolves against the filtered universe. Error messages
  distinguish the two failure modes: unknown id vs present-but-excluded
  (naming the excluding filter, e.g. "MEM focus requires --include-memory").

## 6. Open Questions & Unknowns

None blocking. Scope OQ-1 resolved by DEC-009; scope OQ-2 by DEC-008.

## 7. Decisions, Rationale & Alternatives

- **D1 = DEC-007** — new top-level `graph` verb, `--format dot|json`; shared
  serde contract; `catalog graph` untouched (debug tier). Alt: extend
  `catalog graph` (wrong altitude), dot-only (loses agent JSON).
- **D2 = DEC-008** — port both style tables; transparent background;
  independent presentation policy, no parity contract. Alt: minimal DOT
  (illegible), node-styles-only (loses edge scannability).
- **D3 = DEC-009** — bare = whole corpus, silent; depth unclamped, default 1.
  Alt: focus-or-filter gate + `--all` (breaks the pipeline gesture), stderr
  size warnings (pipe chatter).
- **D4** (revised in adversarial pass) — filters define the universe before
  bounding: BFS travels only filtered nodes/edges, so
  `graph ADR-010 --label governed_by --depth 3` follows governed_by chains.
  In whole-corpus mode `--label` is additionally edge-induced (orphan nodes
  dropped). Alt (rejected): filter-after-bounding — reach leaks through
  non-matching labels, then their edges vanish, leaving inexplicable nodes.
- **D5** — dangling refs render as dashed ghost nodes (free diagnostic), ids
  namespaced `?:<raw>`; identical raws collapse. Alt: drop silently (hides
  corpus damage the graph model deliberately preserves).
- **D6** (revised in adversarial pass) — FOCUS resolves against the filtered
  universe; MEM focus requires `--include-memory`. Errors name the excluding
  filter rather than claiming "not found". Alt: focus implies inclusion
  (surprising asymmetry with non-focus MEM nodes).
- **D7** — `-p/--path` (congruent with `relation list`); `catalog graph`'s
  `--root` is the debug-tier oddball, left alone.
- **D8** — `dot_escape` lifts from `concept_map.rs` into `catalog/dot.rs`;
  concept_map re-imports. Sole genuine DRY seam; emitters stay separate
  (different styling concerns).

## 8. Risks & Mitigations

- R1 (scope): TS/Rust emitter divergence → dissolved by D2's
  no-parity-contract declaration; both are presentation policy.
- R2: style-table drift from `dot.ts` values at port time → VT compares a
  known kind/label's emitted attrs against the documented constant, and the
  port is a literal value copy in one commit.
- R3: `Subgraph` serialization accidentally diverging from the `catalog
  graph` contract → VT asserts key-shape parity on a fixture.

## 9. Quality Engineering & Validation

TDD red/green/refactor per phase; fixtures via `catalog::test_helpers` seeds.

- VT-A projection: undirected reach, depth 0/1/2 bounding, edge dedup,
  dangling edges collected-not-expanded, focus-not-found error,
  empty-projection validity.
- VT-B filters: kind retention (case-normalized), label filter-before-bound
  chain-following (D4) + whole-corpus edge-induction, memory exclusion
  default + `--include-memory`, D6 error split (unknown vs excluded, both
  nonzero, excluding filter named).
- VT-C DOT: structurally valid digraph on a seeded fixture; per-kind node
  attrs and per-label edge colours match constants (R2); ghost nodes for
  dangling refs (D5); byte-determinism (two runs identical).
- VT-D JSON: projected subgraph emits `nodes`/`edges`/`units` contract keys
  (R3); role payload present on `references` edges, absent otherwise.
- VT-E preservation: concept-map suite green unchanged after D8 lift.
- VA-1: real corpus — `doctrine graph SL-226 --depth 2 --format dot | dot
  -Tsvg` renders without graphviz errors (host with `dot`).
- Gate: `just gate` zero warnings; `cargo fmt`.

## 10. Review Notes

Internal adversarial pass (design session, 2026-07-24) — findings integrated:

1. D4 inverted: filter-after-bounding leaked reach through non-matching
   labels; revised to filter-before-bound (universe semantics). §5.2, §5.5.
2. D6 error honesty: "not found" for a filter-excluded focus was misleading;
   split into unknown-id vs present-but-excluded errors. §5.2, §5.5.
3. BFS expansion through dangling targets was unspecified; now
   collected-not-enqueued. §5.4.
4. §2's claim that `outgoing`/`incoming` `dead_code` expects come off was
   wrong — traversal uses once-built adjacency maps (frontend port); the
   per-node helpers stay as-is. §5.4.
5. Ghost-node ids namespaced (`?:<raw>`) against collision with real keys.
   §5.5, D5.
6. `--kind` case-normalization (uppercase) — a silently-empty graph from
   `--kind sl` is a trap DEC-009's silence would worsen. §5.2.

Governance re-check: ADR-001 (pure emitter/projection, thin command) holds;
STD-001 named constants specified for style tables and defaults; PRD-016
boundary revisit flagged for reconcile (§3). No conflicts requiring /consult.
