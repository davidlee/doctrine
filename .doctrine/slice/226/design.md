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
  (SL-071 D10). This slice is the anticipated consumer of that seam.
- `doctrine catalog graph`: whole-corpus JSON dump of `CatalogGraph` (debug
  tier); same serde contract as `/api/graph`. Edges carry externally-tagged
  `label`/`target` variants, optional `role`, optional `descriptor`
  (skip-if-none), and `origin` — hydration deliberately preserves authored
  edge multiplicity.
- DOT serialization + focus/depth bounding exist **only client-side**:
  `web/map/src/dot.ts` (style tables, deterministic emission — with a latent
  defect: `shape="box,rounded"` is invalid Graphviz, see D10),
  `web/map/src/model.ts::bfsCore/neighbourhood` (undirected BFS, [0,3] clamp,
  edge collection from expanded nodes only, depth 0 = focus alone).
  `normalizeGraph` composes roled references edges into `references(<role>)`
  display labels — its comment says "for CLI parity".
- Memory focus resolution precedent: `src/commands/map.rs:59-68` resolves
  `mem_<uid>` / `mem.<key>` refs across items + shipped memories via
  `memory::collect_all` + `resolve_memory_from_all`.
- `src/concept_map.rs` has a private `dot_escape` + minimal `render_dot` for
  the concept-map kind — unstyled, separate concern.

## 3. Forces & Constraints

- ADR-001 layering: emitter/projection pure (engine/leaf); command thin. New
  modules must be classified in `.doctrine/adr/001/layering.toml`.
- New verbs need read/write classification in `src/commands/guard.rs`
  (ADR-006 D2a); `graph` classifies as `Read`.
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
- Emit the projection asked for — no gates, no chatter (DEC-009). Silence
  never extends to swallowing user errors (unknown focus/kind are hard
  errors).
- Presentation policy per surface: the Rust style tables carry no parity
  contract with `dot.ts` (DEC-008).
- Contract parity by construction, not by mirroring: JSON output reuses
  `CatalogGraph`'s own serialization (D11).

## 5. Proposed Design

### 5.1 System Model

```
commands/graph.rs (thin shell; wired via commands/cli.rs + commands/mod.rs,
                   Read-classified in commands/guard.rs)
  └─ scan_catalog → CatalogGraph::from_catalog            (existing)
       └─ focus classification (full graph → filtered)    (new, command layer)
       └─ projection pipeline (new, catalog/graph.rs):
            filter_kinds → filter_memory → filter_label
              ├─ ego:   neighbourhood(focus, depth)
              └─ whole: drop_isolated()  [only if --label]
                 ├─ --format json → CatalogGraph serde     (existing impl)
                 └─ --format dot  → catalog/dot.rs render  (new, pure;
                                     registered in catalog/mod.rs)
```

### 5.2 Interfaces & Contracts

CLI:

```
doctrine graph [FOCUS] [--depth N] [--kind K]... [--label L]
               [--include-memory] [--format dot|json] [-p|--path ROOT]
```

- No FOCUS → whole-corpus projection, silent (DEC-009).
- FOCUS forms: canonical entity ref (`SL-226`) or memory ref (`mem_<uid>` /
  `mem.<key>`), memory refs resolved via the `memory::collect_all` +
  `resolve_memory_from_all` seam exactly as `map.rs` does (one extra corpus
  read, same as `map serve --focus`).
- Focus classification (D6) happens in the command layer **before**
  destructive filtering: (1) resolve against the full graph — unknown id →
  "not found" error; (2) check against the filtered universe — present but
  excluded → error naming the excluding filter (e.g. "MEM focus requires
  --include-memory"; "SL-226 excluded by --kind ADR"). Both nonzero exit.
- `--depth N`: any N ≥ 0, default 1, no clamp; 0 = focus alone (DEC-009).
- `--kind K` repeatable: OR within the kind dimension; normalized to
  uppercase; validated against the static entity-kind registry (MEM legal) —
  unknown prefix is a hard error listing legal prefixes (D12).
- `--label L`: matches the bare snake_case label name only (`references`
  matches all roles; per-role filtering out of scope). Dimensions compose
  conjunctively: kind AND label AND memory.
- `--include-memory` off by default.
- `--format dot` (default) | `json` — json is the projected graph serialized
  by `CatalogGraph`'s existing `Serialize` impl: identical contract
  (`nodes`/`edges`/`units`, tagged label/target variants, role/descriptor
  omission rules) by construction (DEC-007, D11).

Rust (all `pub(crate)`, in `src/catalog/graph.rs`):

```rust
impl CatalogGraph {
  // Owned-subset projection pipeline (D11). Each step retains in place.
  pub(crate) fn filter_kinds(self, prefixes: &BTreeSet<String>) -> Self; // uppercased, pre-validated
  pub(crate) fn filter_label(self, label: &str) -> Self;                 // edges only
  pub(crate) fn exclude_memory(self) -> Self;
  pub(crate) fn drop_isolated(self) -> Self;      // whole mode, --label only (D4)
  pub(crate) fn neighbourhood(self, focus: &NodeKey, depth: u32) -> Self;
      // precondition: focus present (command layer classified it, D6)
  pub(crate) fn contains(&self, key: &NodeKey) -> bool;
}

// src/catalog/dot.rs (registered in catalog/mod.rs)
pub(crate) fn dot_escape(s: &str) -> String;      // lifted from concept_map.rs (D8)
pub(crate) fn render(graph: &CatalogGraph, focus: Option<&NodeKey>) -> String;
```

### 5.3 Data, State & Ownership

- Projection operates on an **owned** `CatalogGraph` (D11): the command
  builds it once from the catalog (one O(V+E) copy of a corpus measured in
  hundreds of nodes — noise against A1's hydration cost), then each filter
  retains in place; no borrowed-view type, no mirror `Serialize` impl, no
  lifetime plumbing. JSON drift from the `catalog graph` contract is
  impossible by construction.
- Style tables in `catalog/dot.rs`: `NODE_STYLES: &[(&str, NodeStyle)]`,
  `EDGE_COLORS: &[(&str, EdgeColor)]` — named constants (STD-001), values
  ported from `dot.ts` (both tables, DEC-008) **except** shape/style are
  modelled as separate fields (D10): `shape: "box"`,
  `style: "filled,rounded"` where dot.ts wrongly writes
  `shape: "box,rounded"`. Unknown kind/label falls back to the ported
  defaults.

### 5.4 Lifecycle, Operations & Dynamics

- One-shot: scan → classify focus → filter → bound → emit → exit. No daemon,
  no cache; A1 (scope) holds — the same hydration already backs `map serve`
  startup.
- BFS (`neighbourhood`): undirected (out + in), visited-set, breadth-bounded
  at `depth`; port of `model.ts::bfsCore` minus the clamp. Adjacency maps
  (in/out per node) are built once over the filtered edge set before
  traversal — no per-node O(E) scans; the existing `outgoing`/`incoming`
  helpers and their `expect(dead_code)` markers are untouched (per-node
  query surface, not traversal).
- **Edge identity is the edge's index** in the edge list — dedup by index,
  never by field tuple, so authored multiplicity (same endpoints, different
  descriptor/origin) survives into JSON and DOT verbatim (D13).
- **Edge collection boundary** (D9): edges are collected only from
  **expanded** nodes (`dist < depth`), exactly as `bfsCore` — boundary nodes
  (`dist == depth`) contribute no edges. Consequences, stated: a depth-1
  triangle focused at A emits A–B and A–C but not B–C (traversal-collected,
  not induced-subgraph); a boundary node's dangling edges do not appear.
  Dangling targets (`UnresolvedRef`/`UnvalidatedText`) are collected when
  their **source is expanded** but never enqueued — no node to expand.
- Display label (D14): a `references` edge with a role renders as
  `references(<role>)` (snake_case role), the `normalizeGraph` composition;
  all other edges render the bare label name.
- Emission order (byte-determinism): real nodes in `BTreeMap` key order;
  ghost nodes after all real nodes, sorted by raw text; edges sorted by
  (source canonical, display label, target scalar — canonical id or
  `?:<raw>` — then **original edge index** as total tie-breaker). Hydration
  order is deterministic (BTreeMap entity scan + authored TOML order) — a
  stated assumption (A2). Trailing newline; nothing on stderr on success.
- DOT semantics ported from `dot.ts::graphToDot`: `rankdir=LR`,
  `bgcolor="transparent"` (DEC-008), `nodesep=0.45`/`ranksep=0.8`, focus
  `penwidth=3.0`, MEM nodes labelled by title (others by id), tooltip
  `id: title · kind · status` — where a `None` status omits the trailing
  ` · status` segment entirely (D15).

### 5.5 Invariants, Assumptions & Edge Cases

- Every emitted edge's endpoints exist in the emitted DOT: resolved targets
  as styled nodes; dangling targets as dashed grey **ghost nodes** labelled
  with the raw text (D5), ids namespaced `?:<raw>` so free text can never
  collide with a real node key; identical raws share one ghost. JSON is
  unaffected — edges already carry the target variant.
- Depth 0 emits exactly the focus node, zero edges.
- Fixed pipeline order (D4): node filters (`--kind`, memory) → edge filter
  (`--label`) → bounding (`neighbourhood`) or, in whole mode when `--label`
  was given, `drop_isolated()`. `filter_label` itself touches only edges;
  orphan-dropping is the separate terminal `drop_isolated` op — so an
  ego-mode isolated focus and depth-0 semantics survive a label filter.
- BFS reach travels only the filtered universe: label-chain following
  (`graph ADR-010 --label governed_by --depth 3`) works because
  non-matching edges are gone before traversal.
- Empty projection (filters exclude everything) emits a valid empty
  `digraph`/empty-collections JSON — not an error.
- A2: hydration edge order is deterministic across runs on an unchanged
  corpus (BTreeMap scan + TOML authoring order). Determinism VTs would catch
  a violation.

## 6. Open Questions & Unknowns

None blocking. Scope OQ-1 resolved by DEC-009; scope OQ-2 by DEC-008.

## 7. Decisions, Rationale & Alternatives

- **D1 = DEC-007** — new top-level `graph` verb, `--format dot|json`; shared
  serialization; `catalog graph` untouched (debug tier). Alt: extend
  `catalog graph` (wrong altitude), dot-only (loses agent JSON).
- **D2 = DEC-008** — port both style tables; transparent background;
  independent presentation policy, no parity contract. Alt: minimal DOT
  (illegible), node-styles-only (loses edge scannability).
- **D3 = DEC-009** — bare = whole corpus, silent; depth unclamped, default 1.
  Alt: focus-or-filter gate + `--all` (breaks the pipeline gesture), stderr
  size warnings (pipe chatter).
- **D4** (revised twice: internal pass, RV-298 F-3) — fixed filter-then-bound
  pipeline with a mode-explicit terminal: ego = `neighbourhood`, whole with
  `--label` = `drop_isolated`. `filter_label` never drops nodes itself. Alt
  (rejected): filter-after-bounding (reach leaks); edge-induction folded into
  `filter_label` (breaks ego isolated-focus and depth-0).
- **D5** — dangling refs render as dashed ghost nodes, ids `?:<raw>`,
  identical raws collapse. Alt: drop silently (hides corpus damage the graph
  model deliberately preserves).
- **D6** (revised twice: internal pass, RV-298 F-1) — focus classification in
  the command layer against the **full** graph first (unknown id), then the
  filtered universe (excluded — error names the filter). Projection methods
  assume a validated focus. Alt (rejected): resolution inside the filtered
  projection (the borrowed view had already destroyed the universe needed to
  tell the cases apart).
- **D7** — `-p/--path` (congruent with `relation list`); `catalog graph`'s
  `--root` is the debug-tier oddball, left alone.
- **D8** — `dot_escape` lifts from `concept_map.rs` into `catalog/dot.rs`;
  concept_map re-imports. Sole genuine DRY seam; emitters stay separate.
- **D9** (RV-298 F-6/F-14) — edge set is traversal-collected from expanded
  nodes only (`bfsCore` port): boundary cross-edges (triangle B–C) and
  boundary-node dangling edges are excluded. Alt (rejected): visited-induced
  subgraph — a different, plausible reading; the port's semantics win and
  are now named.
- **D10** (RV-298 F-10) — shape and style are separate fields; the Rust
  tables correct dot.ts's invalid `shape="box,rounded"` (Graphviz warns and
  falls back to box). The web emitter's identical defect is raised as a
  separate backlog issue, not silently diverged from.
- **D11** (RV-298 F-1/F-4/F-11) — projection produces an **owned**
  `CatalogGraph` subset; JSON reuses the existing `Serialize` impl. Kills
  the mirror-drift class entirely for one small clone. Alt (rejected):
  borrowed `Subgraph` + mirror serde — lifetime plumbing plus an
  independently-driftable contract copy.
- **D12** (RV-298 F-12) — `--kind` is OR-within-dimension, AND-across;
  unknown prefix is a hard error against the static kind registry. Alt:
  silent empty graph (typo trap).
- **D13** (RV-298 F-5) — edge identity = edge-list index; authored
  multiplicity preserved verbatim. Alt (rejected): field-tuple identity
  (silently collapses descriptor/origin-distinct edges in the semantic
  projection).
- **D14** (RV-298 F-7) — display label composes `references(<role>)`;
  `--label` matches bare label names only.
- **D15** (RV-298 F-13) — `None` status omits the tooltip segment; no
  placeholder text.

## 8. Risks & Mitigations

- R1 (scope): TS/Rust emitter divergence → dissolved by D2's
  no-parity-contract declaration; both are presentation policy.
- R2: style-table drift from `dot.ts` values at port time → VT compares a
  known kind/label's emitted attrs against the documented constant; the port
  is a literal value copy in one commit (modulo the deliberate D10 fix).
- R3: JSON contract drift → dissolved structurally by D11 (same `Serialize`
  impl); VT-D keeps an exact-equality regression net anyway.
- R4 (new, RV-298 F-8): nondeterminism via sort-key collision → total order
  guaranteed by the edge-index tie-breaker + A2; determinism VTs on
  deliberately colliding fixtures.

## 9. Quality Engineering & Validation

TDD red/green/refactor per phase; fixtures via `catalog::test_helpers` seeds.

- VT-A projection: undirected reach; depth 0/1/2 bounding; depth-1 triangle
  boundary fixture (B–C excluded, D9); boundary-node dangling-edge exclusion
  (D9); duplicate-edge multiplicity preserved (D13); empty-projection
  validity.
- VT-B filters + focus: kind retention (case-normalized, OR semantics);
  unknown-kind hard error (D12); label filter-before-bound chain-following
  (D4); whole-mode `drop_isolated` only with `--label`; ego-mode isolated
  focus survives a label filter; memory exclusion default +
  `--include-memory`; memory-ref focus forms resolve (F-2); D6 error split —
  unknown vs excluded, both nonzero, excluding filter named.
- VT-C DOT: structurally valid digraph on a seeded fixture; per-kind node
  attrs and per-label edge colours match constants (R2); shape/style split
  emits `style="filled,rounded"` not `shape="box,rounded"` (D10); roled
  references edge renders `references(implements)` (D14); ghost nodes for
  dangling refs with `?:` ids (D5); status-less node omits tooltip segment
  (D15); byte-determinism — two runs identical, on a fixture with colliding
  edge sort keys and multiple ghosts (R4).
- VT-D JSON: exact equality — `to_value(projected whole, no filters)` ==
  `to_value(&graph)` on a fixture exercising every `EdgeTarget` variant,
  role, descriptor, and origin (R3); filtered projections keep the contract
  keys and omission rules.
- VT-E preservation: concept-map suite green unchanged after D8 lift.
- VA-1: real corpus — `doctrine graph SL-226 --depth 2 --format dot | dot
  -Tsvg` renders with **empty stderr** (no unknown-shape or other warnings,
  D10), on a host with `dot`.
- Gate: `just gate` zero warnings; `cargo fmt`.

## 10. Review Notes

Internal adversarial pass (design session, 2026-07-24) — integrated: D4
filter-before-bound; D6 error split; dangling collected-not-enqueued;
adjacency maps (dead_code expects stay); ghost-id namespacing; `--kind`
case-normalization.

External adversarial pass — **RV-298** (codex, 14 findings: 1 blocker, 10
major, 3 minor; all accepted, F-9 partially). Structural outcome: the
borrowed `Subgraph` mirror was replaced by the owned-`CatalogGraph`
projection (D11), which dissolved the blocker (F-1) and the contract-parity
risk (F-4) by construction. Full dispositions on the RV-298 ledger; design
deltas: D9–D15, R4, §2/§3/§5 rewrites, VT suite expanded. Wiring surface
(guard.rs, layering.toml, catalog/mod.rs) added to §5.1 and the
design-target selectors (F-9).

Governance re-check: ADR-001 holds (pure projection/emitter; thin command;
layering.toml registration named); ADR-006 D2a guard classification named;
STD-001 named constants; PRD-016 boundary revisit flagged for reconcile
(§3). No conflicts requiring /consult.
