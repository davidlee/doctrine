# Design SL-137: Corpus-level relation query verb — list edges by label, target, source-kind

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

The only relation read surface is `inspect <ID>` — per-entity, one hop. There is
no corpus-wide query: "show all `governed_by` edges", "which entities reference
ADR-001?", "how is the relation vocabulary used across the corpus?". The data
already exists, fully typed, in the hydrated `Catalog` (`catalog::hydrate`) — but
the only corpus-level reader is `catalog graph` (raw developer JSON). RFC-001's
thesis names this exact gap: the graph is rich on the inside, thin on the outside.

This slice adds two read verbs over that existing data — `relation list`
(filterable edge enumeration) and `relation census` (per-label distribution with
a target-resolution health breakdown). Pure consumption: zero new modelling, no
write path, no new disk I/O.

## 2. Current State

- `Catalog.edges: Vec<CatalogEdge>` is the corpus edge set, built once by
  `scan_catalog` (`catalog::hydrate`). Each `CatalogEdge` carries:
  - `source: CatalogKey` — `Numbered(EntityKey)` (has a `prefix`) or `Memory(String)`.
  - `label: CatalogEdgeLabel` — `Validated(RelationLabel)` (closed vocabulary,
    numbered entities) or `Raw(String)` (memory free-text labels).
  - `target: EdgeTarget` — `Resolved(CatalogKey)` | `UnresolvedRef{raw}` |
    `UnvalidatedText{raw}`.
  - `origin: EdgeOrigin`.
- The edge target is classified once, at hydration, against the scanned key set
  (`classify_target`) — this design re-reads that classification, never recomputes it.
- `inspect` is the only relation reader for entities; it prints scan-degradation
  diagnostics to stderr and renders one entity's 1-hop neighbourhood.
- `link`/`unlink` are top-level write verbs in `commands/relation.rs`
  (`run_link`/`run_unlink`).
- The read spine (`listing`): `Column<R>` + `render_columns` + `json_envelope` +
  `Format`. The corpus-projection precedent is `coverage_view` (defines
  `Column<CoverageRow>`, renders via the spine; bespoke projection, shared render).

## 3. Forces & Constraints

- **ADR-001 (layering):** leaf ← engine ← command, no cycles. The command reads an
  engine projection; the projection reads `catalog` (engine) + `listing`/`relation`
  (leaf).
- **ADR-004 (outbound-only, reciprocity derived):** edges are stored outbound only.
  `--target ADR-001` ("who references ADR-001") reads the outbound store and filters
  by target — no reciprocity computation, no inbound index needed.
- **ADR-010 (relation vocabulary):** the closed label set is `RELATION_RULES` /
  `RelationLabel`; memory labels are free-text `Raw` (mem.pattern.link.memory-label-fork).
- **Pure/imperative split:** projection/filter is pure over `&Catalog`; the only
  impurity (root discovery, scan, stdout/stderr) lives in the command shell.
- **Behaviour preservation:** read-only consumer — `catalog`/`relation`/`listing`
  internals are untouched, so their suites stay green unchanged.
- **Scope discipline (slice non-goals):** no transitive walk (SL-138), no graph
  export, no write path.

## 4. Guiding Principles

- Re-project existing data; do not re-model or re-classify it.
- A query returns "empty" rather than erroring on an unmatched filter — an empty
  result is a valid answer (no hard validation of filter values).
- Two orthogonal axes get two orthogonal controls; neither is hard-wired.
- Machine-readable JSON carries the full resolution axis even when the CLI default
  hides it behind a flag.
- Ride the shared render spine; do not fork table/JSON formatting.

## 5. Proposed Design

### 5.1 System Model

```
commands/relation.rs   (command)  RelationCommand{List,Census}; run_relation_list/_census
        │  root::find → scan_catalog → (print Error diags to stderr) → project → render → stdout
        ▼
relation_query.rs      (engine, NEW)  pure projection+filter+render over &Catalog
        ▼
catalog::hydrate (engine)  ·  listing (leaf)  ·  relation (leaf: RelationLabel)
```

`relation_query.rs` is a new engine-tier module (sibling to `relation_graph.rs`,
which owns the per-entity inspect projection; this one owns the corpus-flat query).
It is pure — `&Catalog` in, rows/strings out; no clock, rng, git, or disk.

The clap `RelationCommand { List, Census }` enum is added to `commands/relation.rs`
beside the existing link/unlink verbs; `cli.rs` registers a top-level `Relation`
subcommand (decision D1). `link`/`unlink` remain top-level (asymmetry accepted, D1).

### 5.2 Interfaces & Contracts

CLI:

```bash
doctrine relation list [--labels validated|raw|all] [--label NAME]
                       [--target REF] [--source-kind PREFIX] [--unresolved]
                       [--format table|json] [--json]
doctrine relation census [--labels validated|raw|all] [--format table|json] [--json]
```

Engine (`relation_query.rs`):

```rust
/// Target-resolution axis — shared by the list `state` field and census columns.
#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum TargetState { Resolved, Unresolved, FreeText }

fn target_state(t: &EdgeTarget) -> TargetState; // Resolved→R, UnresolvedRef→U, UnvalidatedText→F
fn target_display(t: &EdgeTarget) -> String;    // Resolved→canonical; else raw verbatim
fn source_kind(k: &CatalogKey) -> &str;         // Numbered→prefix; Memory→"MEM"

/// Label-provenance selector (Axis A). Default Validated.
#[derive(Clone, Copy)] enum LabelScope { Validated, Raw, All }
impl FromStr for LabelScope { /* table|raw|all, clean anyhow error */ }

struct ListFilter {
  labels: LabelScope,
  label: Option<String>,        // exact match on label name()
  target: Option<String>,       // exact match on target_display()
  source_kind: Option<String>,  // uppercased; exact match on source_kind()
  unresolved: bool,             // keep only state != Resolved
}

#[derive(Serialize)] struct RelationRow { source: String, label: String, target: String, state: TargetState }
#[derive(Serialize)] struct CensusRow  { label: String, count: usize, resolved: usize, unresolved: usize, free_text: usize }

fn project_list(cat: &Catalog, f: &ListFilter) -> Vec<RelationRow>;
fn project_census(cat: &Catalog, labels: LabelScope) -> Vec<CensusRow>;
fn render_list(rows: &[RelationRow], fmt: Format, opts: RenderOpts) -> anyhow::Result<String>;
fn render_census(rows: &[CensusRow], fmt: Format, opts: RenderOpts) -> anyhow::Result<String>;
```

### 5.3 Data, State & Ownership

No new persistent state. The engine borrows `&Catalog` and produces owned row
vectors. The command owns the impure resources (root, scan result, stdout/stderr).
Label admission reads `CatalogEdgeLabel`; the target axis reads `EdgeTarget` —
both already populated by hydration. `source_kind` reads `EntityKey.prefix` for
numbered keys and emits the literal `"MEM"` for memory keys (the `CatalogKey`
identity boundary — mem.pattern.catalog.catalogkey-identity-boundary).

### 5.4 Lifecycle, Operations & Dynamics

**`project_list`** — iterate `cat.edges`; keep an edge iff ALL active axes admit it
(AND across axes): `LabelScope` admits its `CatalogEdgeLabel` variant → `--label`
matches `name()` → `--source-kind` matches `source_kind()` → `--target` matches
`target_display()` → `--unresolved` ⇒ `target_state != Resolved`. Map survivors to
`RelationRow`. Sort `(label, source canonical)`. Deterministic — no clock/rng.

**`project_census`** — filter `cat.edges` by `LabelScope` only; group by label
`name()`; per group accumulate `count` and per-`TargetState` tallies. Emit one
`CensusRow` per label. Sort `(count desc, label asc)`. Invariant:
`count == resolved + unresolved + free_text`.

**Render** — `Column<RelationRow>` (`source │ label │ target │ state`) and
`Column<CensusRow>` (`label │ count │ resolved │ unresolved │ free_text`) arrays;
`listing::render_columns` for table, `listing::json_envelope("relation" /
"relation-census", rows)` for JSON. Empty rows → empty string (header suppressed).

**Command shell** — `scan_catalog` → print only `Severity::Error` diagnostics to
stderr (corpus didn't fully parse ⇒ the view is incomplete; Warning/Info are
suppressed — `--unresolved`/the census breakdown surface those on demand) →
project → render → stdout.

### 5.5 Invariants, Assumptions & Edge Cases

- This surface does NOT ride `listing::build`/`retain`/`ListArgs` — its filters are
  not the substr/regex/status/tag set, so the `validate_statuses` opt-in trap
  (mem.pattern.listing.validate-statuses-is-opt-in) does not apply.
- `--labels validated` (default) excludes every memory `Raw`-label edge; `raw`
  shows only them; `all` shows both. `--source-kind MEM` only yields rows once
  `--labels raw|all` admits memory edges.
- `drift` is a `Validated` label with an always-`UnvalidatedText` target — it counts
  under census `free_text` by design, not as a fault.
- No filter-value validation: an unknown `--source-kind`/`--label`/`--target`
  yields zero rows, never an error.
- `--source-kind` input is upper-cased before comparison (prefixes are upper-case).
- JSON always carries `state` (list) / the full breakdown (census), independent of
  any CLI default.
- Catalog symlink double-count is already handled at the scan walk
  (mem.pattern.entity.corpus-walk-skip-slug-symlink) — this consumer inherits the
  deduped edge set.

## 6. Open Questions & Unknowns

None blocking. Resolved during design:
- OQ-1 (namespace) → D1.
- OQ-2 (census "active labels" semantics) → two orthogonal axes, D2.
- OQ-3 (target-state granularity) → 2-way `--unresolved` bool, 3-way deferred (D3).

## 7. Decisions, Rationale & Alternatives

- **D1 — `relation { list, census }` namespace; link/unlink stay top-level.**
  Matches the slice's stated shape, minimal scope. Alternative B (group link/unlink
  under `relation`, deprecate top-level) churns existing verbs/tests/ADR-cited
  command shapes — its own cohesion slice if ever wanted. Alternative C (flat
  top-level `relations`) reads worse. Asymmetry accepted; reversible.
- **D2 — two orthogonal axes, each its own control.** Axis A label-provenance =
  `--labels validated|raw|all` (default validated); Axis B target-resolution =
  `--unresolved` (list) / always-on breakdown (census). Rejected: hard-excluding
  raw/unvalidated (the slice's first phrasing) — users will want raw labels and the
  validation lens; a default that can't be opened is wrong.
- **D3 — `--unresolved` is a 2-way bool, list-only.** The census breakdown
  (`resolved/unresolved/free_text` columns) subsumes it for census, so census takes
  no `--unresolved`. A 3-way `--target-state` (split dangling vs free-text) is
  deferred — the split is already in catalog diagnostics + `doctrine validate`, and
  the raw target string is visible per row.
- **D4 — census always shows the resolution breakdown.** It is free (every edge is
  already classified) and turns census into a per-label health profile. `Error`
  severity is entity-level (no edge/label to attribute) ⇒ stays on stderr, never a
  column.
- **D5 — new engine module `relation_query.rs`.** Cohesion: corpus-flat query is a
  distinct responsibility from `relation_graph.rs` (per-entity inspect) and from
  `relation.rs` (the leaf vocabulary). Pure, mirroring `coverage_view`.

## 8. Risks & Mitigations

- **R1 — name confusion across `relation.rs` (leaf), `relation_graph.rs`
  (per-entity engine), `relation_query.rs` (corpus engine).** Mitigation: module
  doc-comments state each one's responsibility; `_query` vs `_graph` distinguishes
  flat-corpus vs per-entity.
- **R2 — `--target` exact-match brittleness** (e.g. case, padding). Mitigation:
  match against `target_display()` (canonical for resolved, raw otherwise);
  document exact-match semantics; finer matching is a follow-up if needed.
- **R3 — scope creep toward transitive / inbound indexing.** Mitigation: explicit
  non-goal; `--target` is a flat filter over the outbound store, not a reciprocity
  index.

## 9. Quality Engineering & Validation

Pure projection ⇒ tested over a seeded `Catalog` (`catalog::test_helpers` /
`from_scanned`), no disk. VT:

- **VT-1** `target_state`/`target_display` over all three `EdgeTarget` variants.
- **VT-2** `source_kind`: Numbered→prefix, Memory→`"MEM"`.
- **VT-3** `project_list` filters: `--label`, `--target`, `--source-kind` (incl.
  `MEM`), AND-composition.
- **VT-4** `LabelScope` admission: validated excludes Raw; raw excludes Validated;
  all includes both.
- **VT-5** `--unresolved` keeps only state ≠ Resolved.
- **VT-6** list sort `(label, source)`; empty result → empty string.
- **VT-7** `project_census` breakdown: `count == resolved+unresolved+free_text`;
  `drift` all-free_text; sort `(count desc, label asc)`.
- **VT-8** census `LabelScope` honored (raw labels only under raw/all).
- **VT-9** JSON shapes: list `{source,label,target,state}`; census
  `{label,count,resolved,unresolved,free_text}` under `json_envelope`.
- **VT-10** e2e black-box CLI golden: `relation list`/`census` wire end-to-end on a
  fixture corpus (clap registration + command shell smoke).

Behaviour preservation: no edits to `catalog`/`relation`/`listing` internals ⇒
existing suites green unchanged.

## 10. Review Notes

(internal adversarial pass to follow)
