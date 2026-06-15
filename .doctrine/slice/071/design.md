# SL-071 — Entity corpus scanner / hydrator substrate: design

Canonical technical design. Scope, non-goals, and risks live in `slice-071.md`;
this file is the *how*.

## Architecture overview

Destination: `catalog` owns the entity scan — `relation_graph` and `priority`
become consumers. The migration is staged to avoid breaking the
behaviour-preservation gate.

```
                    catalog::scan  (single source of truth)
                   /       |        \
    catalog::hydrate   catalog::graph   catalog::diagnostic
         |
    relation_graph    priority
    (inspect,         (survey, next,
     validate)         blockers, explain)
```

ADR-001 layering: `catalog` is engine-tier — depends on leaf tier (`entity`,
`integrity`, `relation`, `projection`, `meta`, `fsutil`) and kind modules
(`slice`, `governance`, `spec`, `backlog`, `review`, `rec`, `revision`),
never on command modules.

---

## §1 — Patch plan

### Patch 1: Mechanical re-home + compatibility re-exports

Create `src/catalog/mod.rs` + `src/catalog/scan.rs`. Move these items from
`relation_graph.rs` into `catalog::scan`:

| Item | Visibility | Notes |
|---|---|---|
| `EntityKey` + `impl` | `pub(crate)` | The corpus-wide identity type |
| `ScannedEntity` | `pub(crate)` | The reusable scan record |
| `scan_entities` | `pub(crate)` | The KINDS-walk entry point |
| `status_and_title_for` | `pub(crate)` — move as private `fn`, visibility for tests only | One parse per entity |
| `title_for` | `pub(crate)` — move as private `fn`, visibility for tests only | Lenient title-only read |
| `outbound_for` | `pub(crate)` | Cross-kind relation dispatch |

Items that stay in `relation_graph.rs`:

| Item | Reason |
|---|---|
| `dep_seq_for` | Priority/blocker substrate — semantically distinct from relation scan. Move reconsidered after catalog lands. |
| `require_minted` | Consumer/query gate over a projection, not scan/hydration. |
| `build_relation_graph_from` | Cordage-graph builder — consumer, not scan |
| `inspect_from` / `render_from` | Keyed read surfaces — consumers |
| `validate_relations` | Edge-validity walk — consumer |
| `RelationGraph`, `OverlayMap` | Internal consumer types |
| `resolve_target` | Graph-builder helper — consumer |

In `relation_graph.rs`, add re-exports so existing imports compile unchanged:

```rust
pub(crate) use crate::catalog::scan::{
    outbound_for, scan_entities, EntityKey, ScannedEntity,
};
```

The private helpers (`status_and_title_for`, `title_for`) are used only by
`scan_entities` — they move too but need no `pub(crate)` re-export.

**Preserved invariants:**
- `scan_entities` walks `integrity::KINDS` in table order, sorts IDs ascending per kind
- `outbound_for` dispatches to each kind's existing `relation_edges` reader — never re-parses TOML generically
- `status_and_title_for` does one parse per entity (SL-050 F1)
- Error behaviour unchanged — `scan_entities` still returns `anyhow::Result`, fail-fast on the first bad entity
- No semantic fork: re-exports are aliases, not wrappers with logic. One `scan_entities` body, living in `catalog::scan`.

### Patch 2: Equivalence tests

Add tests that pin scan behaviour before richer catalog types are introduced:

1. **`scan_order_is_stable`** — asserts that the scan yields entities in
   KINDS-table / id-ascending order. Load-bearing for byte-identical `inspect`
   output.

2. **`catalog_scan_matches_legacy_shape`** — fixture test asserting the tuple
   `(key.canonical(), kind.prefix, status, title, outbound: [(label, target)])`
   for every entity is as expected. Uses a fixture `.doctrine` directory with
   3+ entities spanning ≥2 KINDS entries with id gaps (e.g. SL-001, SL-003,
   ADR-002) — enough to prove both table-order and id-ascending. Pins:
   canonical key sequence, title/status extraction, outbound relation extraction.

3. **`inspect_output_is_byte_identical`** — runs `doctrine inspect SL-NNN
   --json` on a fixture and compares to golden. Also exercises entity kinds that
   carry relations (RV, PRD) to cover edge output.

4. **`priority_graph_shape_unchanged`** — builds the priority graph from a
   fixture scan and asserts node count, edge count, and dep/seq overlay set are
   identical to expected values.

5. **`validate_relation_findings_unchanged`** — runs `validate_relations` on a
   fixture with a known dangling edge and asserts the finding strings.

Tests live in `src/catalog/scan.rs` (unit tests with fixture helpers) and/or
`tests/` (integration tests with real `.doctrine/` directory). Prefer fixture
directories over large golden snapshots unless a snapshot is the clearest
assertion.

### Patch 3: Richer catalog types

Add `Catalog`, `CatalogEntity`, `CatalogEdge`, `CatalogDiagnostic` and a
`scan_catalog` entry point **built from `Vec<ScannedEntity>`** — no second
disk walk.

```rust
// src/catalog/hydrate.rs

pub(crate) struct Catalog {
    pub(crate) entities: Vec<CatalogEntity>,
    pub(crate) edges: Vec<CatalogEdge>,
    pub(crate) diagnostics: Vec<CatalogDiagnostic>,
}

pub(crate) struct CatalogEntity {
    pub(crate) key: EntityKey,
    pub(crate) kind: &'static entity::Kind,
    pub(crate) path: PathBuf,
    pub(crate) title: String,
    pub(crate) status: Option<String>,
    pub(crate) source: SourceSpan,
}

pub(crate) struct CatalogEdge {
    pub(crate) source: EntityKey,
    pub(crate) label: RelationLabel,
    pub(crate) target: EdgeTarget,
    pub(crate) origin: EdgeOrigin,
}

pub(crate) enum EdgeTarget {
    /// Target parses as a canonical ref AND the entity exists in the scan.
    Resolved(EntityKey),
    /// Target parses as a canonical ref but no entity exists under that id.
    UnresolvedRef { raw: String },
    /// Target fails to parse as a canonical ref (free text, unvalidated label,
    /// or unknown kind prefix — `parse_canonical_ref` uses one error type).
    UnvalidatedText { raw: String },
}

pub(crate) struct EdgeOrigin {
    pub(crate) file: PathBuf,
    pub(crate) field: Option<String>,
}
```

```rust
// src/catalog/diagnostic.rs

pub(crate) struct CatalogDiagnostic {
    pub(crate) file: PathBuf,
    pub(crate) entity_key: Option<EntityKey>,
    pub(crate) field: Option<String>,
    pub(crate) message: String,
    pub(crate) severity: Severity,
}

pub(crate) enum Severity {
    Error,
    Warning,
    Info,
}
```

```rust
// src/catalog/scan.rs (new function)

pub(crate) fn scan_catalog(root: &Path) -> anyhow::Result<Catalog> {
    let scanned = scan_entities(root)?;
    Ok(Catalog::from_scanned(root, scanned))
}
```

`Catalog::from_scanned` is pure — it classifies targets, derives entity paths
from `EntityKey` + `Kind.dir`, and wraps the existing scan output. Target
classification uses `integrity::parse_canonical_ref` (the same oracle `link` and
`validate_relations` use):

1. Parse the target string as a canonical ref via `integrity::parse_canonical_ref`
2. If parse fails (not a ref, or unknown prefix — one error type) → `UnvalidatedText`
3. If parse succeeds and entity is present in scan → `Resolved(key)`
4. If parse succeeds but entity absent from scan → `UnresolvedRef`

**No new disk reads.** `Catalog` is a projection of `ScannedEntity` data.
Diagnostics at this stage derive only from edge classification (unresolved refs).

**Diagnostic limitation:** `scan_catalog` wraps `scan_entities`, which is
fail-fast — it bails on the first malformed entity. Richer diagnostics
(malformed TOML, duplicate ids, unknown relation labels) require an
error-tolerant walk. That is a follow-up slice; this slice delivers the
catalog shape and edge classification. The `CatalogDiagnostic` type and
`diagnostics` field are plumbed now so the follow-up only needs to fill them.

**SourceSpan:** `(file, field)` only — the entity's TOML path and an optional
section/field name. No line/col tracking. That requires parser support not
present in the codebase; deferred to a follow-up.

**`path` derivation:** `CatalogEntity.path` is derived from `EntityKey` +
`Kind.dir` — the same path authority used by the existing readers
(`status_and_title_for`, `outbound_for`). No new path convention.

### Patch 4: Presentation-neutral graph

A pure projection of `Catalog` into a graph, with no cordage dependency.

```rust
// src/catalog/graph.rs

pub(crate) struct CatalogGraph {
    pub(crate) nodes: BTreeMap<NodeKey, CatalogNode>,
    pub(crate) edges: Vec<CatalogEdge>,
}

pub(crate) enum NodeKey {
    Entity(EntityKey),
}

pub(crate) struct CatalogNode {
    pub(crate) title: String,
    pub(crate) status: Option<String>,
    pub(crate) kind_label: &'static str,
}

impl CatalogGraph {
    pub(crate) fn outgoing(&self, node: &NodeKey) -> Vec<&CatalogEdge> { ... }
    pub(crate) fn incoming(&self, node: &NodeKey) -> Vec<&CatalogEdge> { ... }
}
```

Build: `CatalogGraph::from_catalog(&Catalog)` — pure projection. Edges with
`UnresolvedRef`/`UnvalidatedText` targets still appear in the edge
list (they carry a `source` entity) but have no target node.

`neighbours(depth)` is deferred — it involves BFS/DFS traversal and is not
needed for the debug output. Add when the concept mapper or an inspect-like view
needs it.

### Patch 5: Consumer migration

Migration order, least-risk first:

1. **`relation_graph`** already consumes `catalog::scan::scan_entities` via
   re-export — no change needed.

2. **`validate_relations`** — option to consume `Catalog.edges` for dangler
   detection. Not forced; stays on `scan_entities` until Catalog proves itself.

3. **`priority`** — stays on `ScannedEntity`. The `kind`, `status`, `title`
   fields it reads are already on `ScannedEntity`. Migrate only when doing so
   removes adapter code without changing behaviour.

4. **`coverage_scan`** — may consume `Catalog` in a follow-up slice.

5. **Future consumers** (concept mapper, agent-context) — consume `Catalog`,
   never `relation_graph` directly.

### Patch 6: Debug CLI (optional)

Add only if needed for acceptance evidence or debug exploration. Use the
`catalog` noun:

```sh
doctrine catalog scan --json
doctrine catalog graph --json
```

Not a polished user-facing feature — thin JSON dump for developers. No colour,
no pagination, no `--format table`. If the CLI surface belongs behind a debug
gate, use `doctrine debug catalog-scan --json`.

---

## §2 — Design decisions

### D1: `dep_seq_for` stays in relation_graph

`dep_seq_for` is structurally similar to `outbound_for` (KINDS dispatch → per-kind
reader) but semantically it is priority/blocker substrate — the read gate that
lets dep/seq edges reach the priority blocker/next view. Moving it increases the
change surface of Patch 1 without proving the catalog scanner. Reconsider after
`catalog::scan` lands.

### D2: `require_minted` stays in relation_graph

It is a consumer/query gate over a `Projection<EntityKey>`, not a scan or
hydration function. It belongs with the keyed read surfaces (`inspect_from`,
`render_from`, etc.) that use it.

### D3: `CatalogEntity.path` is derived, not scanned

The entity's filesystem path is `root.join(kind.dir).join(format!("{id:03}"))`.
This uses data already on `EntityKey` + `Kind` — the same path authority as the
existing readers. No new disk probe. Stored on `CatalogEntity` for consumers
that need it (coverage scanner, agent-context).

### D4: SourceSpan is minimal

File path + optional field name. No line/col tracking — that requires TOML span
parser support not present in the codebase. Deferred to a follow-up slice when a
consumer needs it.

### D5: Edge target classification uses the existing oracle

`integrity::parse_canonical_ref` is the single source for canonical-ref parsing
— the same function `link` and `validate_relations` use. Four outcomes:

- Parse fails → `UnvalidatedText`
- Parse succeeds, prefix not in KINDS → parse fails (one error type) → `UnvalidatedText`
- Parse succeeds, entity present in scan → `Resolved(key)`
- Parse succeeds, entity absent → `UnresolvedRef`

No new parsing path. No duplication of the canonical-ref grammar.

### D6: No generic TOML relation reader

`outbound_for` stays the single route from TOML to edges. The catalog does not
invent a generic `[[relation]]` parser — it consumes `ScannedEntity.outbound`
which was already produced by `outbound_for`'s per-kind dispatch. This respects
the existing per-kind reader contracts.

### D7: Re-exports are aliases, not wrappers

`relation_graph.rs` uses `pub(crate) use crate::catalog::scan::...` — no
wrapper functions that could drift from the catalog implementation. One body,
one source of truth.

### D8: Test placement

Existing relation_graph tests may stay in place for the mechanical re-home patch
— they exercise the moved functions through the re-exports and prove the
behaviour-preservation gate. New ownership and equivalence tests for the scanner
live in `catalog::scan`. A later cleanup may relocate the old tests once
behaviour is pinned and the adapter boundary is stable.

### D9: `Catalog::from_scanned` is pure over inputs

`root: &Path` is a configuration parameter used to derive `CatalogEntity.path`
from `EntityKey` + `Kind.dir`. It is not a disk source — `from_scanned` reads
no files. The path derivation uses the same authority as the existing readers.

### D10: `CatalogGraph` method semantics

`incoming(node)` returns only edges whose target is `Resolved(node)`. A
caller that passes an `EntityKey` for the node gets back edges that point *to*
that entity.

`outgoing(node)` returns all edges whose source is `node`, including those with
`UnresolvedRef` or `UnvalidatedText` targets. Callers must handle the case
where an edge has no target node in the graph.

### D11: `status_and_title_for` visibility

`status_and_title_for` and `title_for` move as private functions in
`catalog::scan`. Only `EntityKey`, `ScannedEntity`, `scan_entities`, and
`outbound_for` are `pub(crate)`. The private helpers need no re-export.

### D12: CLI is optional

The `doctrine catalog scan --json` / `doctrine catalog graph --json` commands
are debug scaffolding — useful for development and demo evidence, but not
gating for acceptance. The slice is complete without them; add only if needed.

---

## §3 — Verification alignment

| What | How |
|---|---|
| Behaviour-preservation gate | Existing inspect/priority/validate suites pass unchanged |
| Scan order stable | `scan_order_is_stable` fixture test (Patch 2) |
| Entity shape unchanged | `catalog_scan_matches_legacy_shape` (Patch 2) |
| Inspect output byte-identical | Golden test on `doctrine inspect --json` (Patch 2) |
| Priority graph shape unchanged | Node/edge/overlay count assertion (Patch 2) |
| Validate findings unchanged | Known-dangling-edge fixture (Patch 2) |
| Catalog hydrates correctly | Fixture tests per entity kind + target classification (Patch 3) |
| Edge target classification | Resolved/UnresolvedRef/UnvalidatedText coverage (Patch 3) |
| No second KINDS walk | Code-review convention: `rg "for kref in integrity::KINDS"` outside `catalog::scan` is a defect |
| `just gate` zero warnings | `cargo clippy` workspace-wide |

---

## §4 — Open questions (resolved)

1. **SourceSpan detail** → File+field only. Line/col deferred.
2. **`dep_seq_for` home** → Stays in `relation_graph.rs`. Reconsider after catalog lands.
3. **`require_minted` home** → Stays in `relation_graph.rs`. Consumer gate, not scan.
4. **CLI subcommand** → `doctrine catalog` or `doctrine debug catalog-*`. Not `corpus`.
