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
  entity ref or memory ref; not present in the graph → error, nonzero exit.
- `--depth N`: any N ≥ 0, default 1, no clamp; 0 = focus alone (DEC-009).
- `--kind K` repeatable (prefix, e.g. `SL`); `--label L` edge filter (D4
  edge-induced); `--include-memory` off by default.
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
  pub(crate) fn neighbourhood(&self, focus: &NodeKey, depth: u32) -> Subgraph<'_>;
}
impl Subgraph<'_> {
  pub(crate) fn retain_kinds(self, prefixes: &[String]) -> Self;
  pub(crate) fn retain_label(self, label: &str) -> Self;   // D4: edge-induced
  pub(crate) fn exclude_memory(self) -> Self;
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

- One-shot: scan → project → filter → emit → exit. No daemon, no cache; A1
  (scope) holds — the same hydration already backs `map serve` startup.
- BFS: undirected (out + in), visited-set, edge dedup by identity tuple,
  breadth-bounded at `depth`. Port of `model.ts::bfsCore` minus the clamp.
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
  **ghost nodes** labelled with the raw text (D5). JSON is unaffected — edges
  already carry the target variant.
- Depth 0 emits exactly the focus node, zero edges.
- Filters compose conjunctively, applied after bounding; `retain_label` is
  edge-induced (D4): surviving nodes = endpoints of surviving edges ∪ focus.
- Empty projection (filters exclude everything) emits a valid empty
  `digraph`/empty-collections JSON — not an error.
- A FOCUS that parses but isn't in the graph errors; one that names a MEM node
  works even without `--include-memory`? No — D6: `--include-memory` governs
  the node universe first; a MEM focus without the flag is "not in the graph"
  (consistent, documented in help text).

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
- **D4** — `--label` edge-induced: node set shrinks to surviving endpoints
  (∪ focus). Alt: keep all nodes → orphan clouds defeat the filter.
- **D5** — dangling refs render as dashed ghost nodes (free diagnostic). Alt:
  drop silently (hides corpus damage the graph model deliberately preserves).
- **D6** — memory participation is decided before focus resolution; MEM focus
  requires `--include-memory`. Alt: focus implies inclusion (surprising
  asymmetry with non-focus MEM nodes).
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
  focus-not-found error, empty-projection validity.
- VT-B filters: kind retention, label edge-induction (D4), memory exclusion
  default + `--include-memory`, D6 focus/memory interaction.
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

(adversarial pass pending)
