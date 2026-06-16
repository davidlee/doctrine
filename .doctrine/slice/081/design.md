# SL-081 Design — Memory in the catalog graph

## Architecture overview

```
memory.rs                    catalog/scan.rs              catalog/hydrate.rs          catalog/graph.rs
──────────                   ───────────────              ──────────────────          ────────────────
read_catalog_record()   →    scan_memory_entities()   →   Catalog::from_scanned() →   CatalogGraph
  │                              │                           │                            │
  │ RawMemoryToml                │ Vec<MemoryCatalogRecord>   │ CatalogEntity              │ CatalogNode
  │   .memory_uid (validated)    │ per-uid items>shipped      │   .key: CatalogKey         │   .kind_label: "MEM"
  │   .title                     │ real dirs only            │   .kind_label: "MEM"       │   .memory_type: Some(..)
  │   .status                    │ (entity::scan_named)      │   .kind: None              │
  │   .memory_type               │ uid == dir.name check     │ CatalogEdge                │ NodeKey = CatalogKey
  │   .relations (vec)           │ malformed → diagnostic    │   .source: CatalogKey      │
  │                              │                           │   .label: CatalogEdgeLabel │
  │ RawRelation {label, target}  │                           │   .target: EdgeTarget      │
  └──────────────────────────────┘                           └────────────────────────────┘
```

Memory entities enter the catalog through a `memory.rs`-owned read helper
(reusing `RawMemoryToml`), are scanned as real directories only (no symlinks)
via `entity::scan_named`, hydrated into `CatalogEntity`/`CatalogEdge` with
`CatalogKey::Memory(uid)` identity, and projected into `CatalogGraph` with
`kind_label = "MEM"`. The frontend consumes the same `nodes`/`edges` JSON
shape — no frontend changes in SL-081.

---

## Decision table

| # | Decision | Rationale |
|---|---|---|
| D1 | `CatalogKey` enum over `EntityKey`; manual flat `Serialize` | One identity at the catalog boundary; flat JSON matching existing `NodeKey` contract. `EntityKey` stays numbered-only, `Copy`, KINDS-backed — priority/relation graph untouched. |
| D2 | `CatalogEdgeLabel` with `Raw(String)` for memory | Memory has no `entity::Kind`, can't participate in `RELATION_RULES`. Raw preservation is honest; vocabulary extension is a follow-up slice. |
| D3 | `memory.rs`-owned `read_catalog_record`, reuses `RawMemoryToml` | Single TOML parse site. Validates uid shape only (`is_uid`); tolerates unknown status/memory_type values. `Memory::parse` remains strict authority for `memory find`/`show`. |
| D4 | `entity::scan_named` for directory listing — real dirs only, no symlinks | Matches existing named-entity scan contract. Prevents `mem.foo.bar → mem_uid` alias traversal. |
| D5 | uid == dir name check on every scanned record | The uid directory is canonical per memory-spec § Identity. Mismatch → `CatalogDiagnostic::Error`. |
| D6 | Malformed records → `CatalogDiagnostic::Error`, never silent skip | A broken `memory.toml` is evidence; hiding it recreates the original "memory exists but graph says nothing" failure mode. Missing `shipped/` is fine; malformed individual records are not. |
| D7 | `validate_relations` skips memory edges, keeps numbered invariant explicit | `CatalogKey::Numbered` edge source MUST be found in entity_kinds — `None` is a bug, not expected. `CatalogEdgeLabel::Raw` on a numbered edge is also a bug. |
| D8 | Memory markdown gated on graph membership + `is_uid` validation | Matches numbered entity discipline. No `starts_with("mem_")` guesswork. Uses `safe_join` for path construction. |
| D9 | `memory_type` surfaced in `CatalogNode` JSON only — frontend display deferred | Enables future hover-pane enrichment. No `web/map/` changes in SL-081. |
| D10 | Empty relation rows → `CatalogDiagnostic::Warning` | `[[relation]]` with missing `label` or empty `target` is authored intent gone wrong — surface it, don't emit blank graph edges. |

---

## 1. `CatalogKey` — identity at the catalog boundary

Location: `src/catalog/hydrate.rs` (new), replacing scattered uses of `EntityKey`
in catalog entity/edge/target types.

```rust
/// Corpus-wide identity — numbered entities AND named (memory) entities.
/// Serializes flat: Numbered → "SL-001", Memory → "mem_019ecf..."
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CatalogKey {
    Numbered(EntityKey),
    Memory(String),
}

impl CatalogKey {
    pub(crate) fn canonical(&self) -> String {
        match self {
            CatalogKey::Numbered(k) => k.canonical(),
            CatalogKey::Memory(uid) => uid.clone(),
        }
    }
}

/// Manual impl — NOT #[derive(Serialize)]. Flattens to string exactly like
/// the pre-existing `NodeKey` serialization (src/catalog/graph.rs:30).
impl serde::Serialize for CatalogKey {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.canonical().serialize(s)
    }
}
```

`EntityKey` is unchanged — `Copy`, numbered-only, KINDS-backed. `CatalogKey`
is `Clone` (not `Copy` — the `Memory(String)` variant prevents it).
`BTreeSet<CatalogKey>` and `BTreeMap<CatalogKey, ...>` work with `Clone + Ord`.

---

## 2. `CatalogEdgeLabel` — validated for numbered, raw for memory

Location: `src/catalog/hydrate.rs` (new)

```rust
/// An edge label — validated against RELATION_RULES for numbered entities,
/// preserved as-authored for memory entities.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) enum CatalogEdgeLabel {
    Validated(RelationLabel),
    Raw(String),
}

impl CatalogEdgeLabel {
    pub(crate) fn name(&self) -> &str {
        match self {
            CatalogEdgeLabel::Validated(l) => l.name(),
            CatalogEdgeLabel::Raw(s) => s.as_str(),
        }
    }
}
```

Serializes as a flat string (the `name()`), matching the existing edge label
JSON shape. The `Raw` variant carries the authored label text from
memory `[[relation]]` rows verbatim — no vocabulary restriction, no silent drop.

---

## 3. `RawRelation` — from fieldless to real

Location: `src/memory.rs:392` (edit)

```rust
// Before:
#[derive(Debug, Default, Deserialize, Serialize)]
struct RawRelation {}

// After:
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct RawRelation {
    #[serde(default)]
    pub(crate) label: String,
    #[serde(default)]
    pub(crate) target: String,
}
```

The existing comment ("not read by any v1 verb yet: relation/source resolution
is the SL-008 registry") is replaced with a catalog-oriented note: relations
are read by `read_catalog_record` for graph display; vocabulary governance is
deferred.

---

## 4. Memory scan — `memory.rs`-owned, `entity::scan_named` for dirs

### 4a. `MemoryCatalogRecord` and `read_catalog_record`

Location: `src/memory.rs` (new, near `Memory::parse`)

```rust
/// Lightweight catalog projection of one memory entity. Reuses
/// `RawMemoryToml` for deserialization; validates uid shape only.
/// Strict validation (status, memory_type, scope, anchor) lives in
/// `Memory::parse` — the `memory find`/`show` authority.
pub(crate) struct MemoryCatalogRecord {
    pub(crate) uid: String,
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) memory_type: String,
    pub(crate) relations: Vec<RawRelation>,
    pub(crate) path: PathBuf,
}

/// Read a memory entity's catalog-facing metadata. Returns an error on
/// parse failure or invalid uid — the caller emits a
/// `CatalogDiagnostic::Error`.
pub(crate) fn read_catalog_record(toml_path: &Path) -> anyhow::Result<MemoryCatalogRecord> {
    let text = std::fs::read_to_string(toml_path)?;
    let raw: RawMemoryToml = toml::from_str(&text)
        .map_err(|e| anyhow::anyhow!("parse {}: {e}", toml_path.display()))?;
    if !is_uid(&raw.memory_uid) {
        bail!(
            "invalid memory_uid {:?} in {}",
            raw.memory_uid,
            toml_path.display()
        );
    }
    let title = if raw.title.is_empty() {
        raw.memory_uid.clone()
    } else {
        raw.title
    };
    Ok(MemoryCatalogRecord {
        uid: raw.memory_uid,
        title,
        status: raw.status,
        memory_type: raw.memory_type,
        relations: raw.relations,
        path: toml_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf(),
    })
}
```

### 4b. `scan_memory_entities` in `src/catalog/scan.rs`

```rust
/// Scan memory entities from `MEMORY_ITEMS_DIR` and `MEMORY_SHIPPED_DIR`.
/// Uses `entity::scan_named` — real directories only, no symlinks.
/// Items override shipped on uid collision. Malformed records produce
/// diagnostics via an out-param; the caller decides fail-fast vs collect.
/// Missing `shipped/` is silently tolerated.
pub(crate) fn scan_memory_entities(
    root: &Path,
    diagnostics: &mut Vec<CatalogDiagnostic>,
) -> anyhow::Result<Vec<MemoryCatalogRecord>> {
    use crate::memory::{MEMORY_ITEMS_DIR, MEMORY_SHIPPED_DIR, MemoryCatalogRecord};

    let mut records: BTreeMap<String, MemoryCatalogRecord> = BTreeMap::new();

    // shipped/ first (lower priority), then items/ (overrides).
    // shipped/ is regenerable — any error (missing or unreadable) is silently
    // skipped. items/ is the user's authored memory — a read error propagates.
    for (dir, fail_on_error) in [
        (MEMORY_SHIPPED_DIR, false),
        (MEMORY_ITEMS_DIR, true),
    ] {
        let base = root.join(dir);
        let names = match entity::scan_named(&base) {
            Ok(n) => n,
            Err(e) if !fail_on_error => continue,
            Err(e) => return Err(e.into()),
        };
        for name in &names {
            let toml_path = base.join(name).join("memory.toml");
            match crate::memory::read_catalog_record(&toml_path) {
                Ok(rec) => {
                    // D5: uid must match directory name (canonical per memory-spec)
                    if rec.uid != *name {
                        diagnostics.push(CatalogDiagnostic {
                            file: toml_path,
                            entity_key: Some(CatalogKey::Memory(name.clone())),
                            field: None,
                            message: format!(
                                "memory_uid {} does not match directory name {}",
                                rec.uid, name
                            ),
                            severity: Severity::Error,
                        });
                        continue;
                    }
                    records.insert(rec.uid.clone(), rec);
                }
                Err(e) => {
                    // D6: malformed → diagnostic with identity
                    diagnostics.push(CatalogDiagnostic {
                        file: toml_path,
                        entity_key: Some(CatalogKey::Memory(name.clone())),
                        field: None,
                        message: format!("failed to read memory record: {e}"),
                        severity: Severity::Error,
                    });
                }
            }
        }
    }

    Ok(records.into_values().collect())
}
```

---

## 5. Catalog hydration — revised types

Location: `src/catalog/hydrate.rs` (edits to existing types)

### 5a. `CatalogEntity`

```rust
pub(crate) struct CatalogEntity {
    pub(crate) key: CatalogKey,
    pub(crate) kind_label: &'static str,         // "SL", "ADR", "MEM", ...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) kind: Option<&'static entity::Kind>, // None for memory
    pub(crate) path: PathBuf,
    pub(crate) title: String,
    pub(crate) status: Option<String>,
    pub(crate) source: SourceSpan,
}
```

`kind: None` is skipped in JSON serialization — memory entities don't leak a
`null` field.

### 5b. `CatalogEdge`

```rust
pub(crate) struct CatalogEdge {
    pub(crate) source: CatalogKey,
    pub(crate) label: CatalogEdgeLabel,
    pub(crate) target: EdgeTarget,
    pub(crate) origin: EdgeOrigin,
}
```

### 5c. `EdgeTarget`

```rust
pub(crate) enum EdgeTarget {
    Resolved(CatalogKey),
    UnresolvedRef { raw: String },
    UnvalidatedText { raw: String },
}
```

### 5d. `CatalogDiagnostic.entity_key`

Changes from `Option<EntityKey>` to `Option<CatalogKey>` so memory-sourced
diagnostics carry a `CatalogKey::Memory(uid)` identity.

### 5e. `Catalog::from_scanned` — revised signature

```rust
impl Catalog {
    pub(crate) fn from_scanned(
        root: &Path,
        scanned: &[ScannedEntity],
        memory: &[MemoryCatalogRecord],
    ) -> Self { ... }
}
```

Numbered entities map to `CatalogKey::Numbered(key)`, `kind_label = key.prefix`,
`kind = Some(kind)`. Memory entities map to `CatalogKey::Memory(uid)`,
`kind_label = "MEM"`, `kind = None`. Edges from numbered entities use
`CatalogEdgeLabel::Validated(label)`, with origin file `{entity_dir}/` per the
existing convention. Edges from memory entities use `CatalogEdgeLabel::Raw(label)`,
with origin file `{record.path}/memory.toml` (the memory entity directory joined
with the TOML filename — `MemoryCatalogRecord.path` carries the entity dir).

All existing `CatalogDiagnostic` constructors using `Some(se.key)` (where `se.key`
is `EntityKey`) are wrapped: `Some(CatalogKey::Numbered(se.key))`.
`CatalogDiagnostic.entity_key` changes from `Option<EntityKey>` to
`Option<CatalogKey>` (see §5d).

### 5f. Empty relation row diagnostics (D10)

During `from_scanned`, for each memory relation:
- `label.is_empty()` → `CatalogDiagnostic::Warning`: "empty relation label in memory {uid}"
- `target.is_empty()` → `CatalogDiagnostic::Warning`: "empty relation target in memory {uid}"
- Both non-empty → emit as `CatalogEdge` with `Raw(label)`.

### 5g. `classify_target` — updated key set

```rust
fn classify_target(raw: &str, key_set: &BTreeSet<CatalogKey>) -> EdgeTarget { ... }
```

Memory entities are never targets in SL-081 (memory → numbered only), but
`CatalogKey` is the right type for the future.

---

## 6. Graph projection

Location: `src/catalog/graph.rs` (edits)

```rust
// NodeKey becomes a re-export of CatalogKey
pub(crate) use super::hydrate::CatalogKey as NodeKey;

pub(crate) struct CatalogNode {
    pub(crate) title: String,
    pub(crate) status: Option<String>,
    pub(crate) kind_label: &'static str,
    /// The memory type (e.g. "pattern", "fact") — `None` for numbered entities.
    /// Surfaced in JSON for future frontend enrichment; no web/map/ changes in SL-081.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) memory_type: Option<String>,
}
```

`CatalogGraph::from_catalog` builds nodes from `catalog.entities` and edges
from `catalog.edges` — the loop logic is unchanged; only the key/label types
differ. `from_catalog` copies `memory_type` from the scan record.

Existing `NodeKey::Entity(key)` pattern matches in `outgoing`/`incoming` become
`CatalogKey::Numbered(key)`. The `CatalogKey::Memory(_)` arm returns an empty
`Vec` in both directions — memory entities have no incoming edges in the graph
(they are never edge targets in SL-081), and their outgoing edges are already in
the edge list.

---

## 7. Markdown route — validated memory fallback

Location: `src/map_server/routes.rs` (edit to `entity_markdown` handler)

```rust
async fn entity_markdown(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, MapServerError> {
    // Path 1: canonical ref (numbered entities)
    if let Ok((kind_ref, num)) = crate::integrity::parse_canonical_ref(&id) {
        let entity_key = crate::catalog::scan::EntityKey {
            prefix: kind_ref.kind.prefix,
            id: num,
        };
        let graph_key = crate::catalog::hydrate::CatalogKey::Numbered(entity_key);
        let graph = state.graph.read().await;
        let node_exists = graph.nodes.contains_key(&graph_key);
        drop(graph);
        if !node_exists {
            return Err(MapServerError::EntityNotFound(id));
        }
        let body = markdown::read_entity_markdown(&state.root, &entity_key).await?;
        return Ok((
            [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
            body,
        ));
    }

    // Path 2: memory uid (D8 — validated shape + graph membership)
    if crate::memory::is_uid(&id) {
        let graph_key = crate::catalog::hydrate::CatalogKey::Memory(id.clone());
        let graph = state.graph.read().await;
        let node_exists = graph.nodes.contains_key(&graph_key);
        drop(graph);
        if !node_exists {
            return Err(MapServerError::EntityNotFound(id));
        }
        let body = markdown::read_memory_markdown(&state.root, &id).await?;
        return Ok((
            [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
            body,
        ));
    }

    Err(MapServerError::BadEntityId(id))
}
```

Location: `src/map_server/markdown.rs` (new function)

```rust
/// Read a memory entity's markdown body. Tries `MEMORY_ITEMS_DIR` first
/// (local overrides shipped), then `MEMORY_SHIPPED_DIR`. Uses
/// `fsutil::safe_join` for path containment; `NotFound` is the expected
/// miss for shipped-only records — other IO errors propagate.
pub(crate) async fn read_memory_markdown(
    root: &Path,
    uid: &str,
) -> Result<String, MapServerError> {
    for dir in [MEMORY_ITEMS_DIR, MEMORY_SHIPPED_DIR] {
        let dir_path = crate::fsutil::safe_join(root, Path::new(dir))
            .map_err(|e| MapServerError::Other(e))?;
        let md_path = crate::fsutil::safe_join(&dir_path, Path::new(uid))
            .map_err(|e| MapServerError::Other(e))?
            .join("memory.md");
        match tokio::fs::read_to_string(&md_path).await {
            Ok(body) => return Ok(body),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(MapServerError::Other(e.into())),
        }
    }
    Err(MapServerError::EntityNotFound(uid.to_string()))
}
```

No `path.exists()` — uses async read with `NotFound` fallthrough, no
sync/async race.

---

## 8. `validate_relations` — explicit consumption of `CatalogEdgeLabel`

Location: `src/relation_graph.rs` (edit to `validate_relations`)

```rust
for edge in &catalog.edges {
    // Memory edges: no relation vocabulary yet — skip validation (D7).
    let CatalogKey::Numbered(source_key) = &edge.source else {
        continue;
    };
    let CatalogEdgeLabel::Validated(label) = &edge.label else {
        // D7: Raw label on a numbered edge is catalog corruption.
        findings.push(format!(
            "internal: numbered edge {} has Raw label {:?}",
            source_key.canonical(),
            edge.label.name()
        ));
        continue;
    };
    // Original invariant restored — MUST exist (was `if let`):
    let Some(kind) = entity_kinds.get(source_key) else {
        findings.push(format!(
            "internal: edge source {} not in entity-kind map",
            source_key.canonical()
        ));
        continue;
    };
    if let EdgeTarget::UnresolvedRef { raw } = &edge.target {
        let validated = crate::relation::lookup(kind, *label)
            .is_some_and(|r| !matches!(r.target, TargetSpec::Unvalidated));
        if validated {
            findings.push(format!(
                "{}: `{}` target `{}` does not resolve (dangling [[relation]] edge)",
                source_key.canonical(),
                label.name(),
                raw
            ));
        }
    }
}
```

Also updates `require_minted` / `dep_seq_for` / `inspect_from`: these consume
`ScannedEntity` (not `CatalogEntity`), so their `EntityKey` + `RelationEdge`
types are unchanged — no ripple.

---

## 9. `priority/graph.rs` — untouched

`EntityKey` stays numbered-only. `scan_entities()` stays KINDS-only. Priority
consumes `ScannedEntity` directly (not `Catalog`), so `CatalogKey` never
reaches this module.

---

## 10. `scan_catalog` entry point

Location: `src/catalog/hydrate.rs` (edit)

```rust
pub(crate) fn scan_catalog(root: &Path) -> anyhow::Result<Catalog> {
    let scanned = super::scan::scan_entities(root)?;
    let mut diagnostics = Vec::new();
    let memory = super::scan::scan_memory_entities(root, &mut diagnostics)?;
    let mut catalog = Catalog::from_scanned(root, &scanned, &memory);
    catalog.diagnostics.extend(diagnostics);
    Ok(catalog)
}
```

---

## Affected surface (final)

| Module | Change | Blast radius |
|---|---|---|
| `src/memory.rs` | `RawRelation` gains `label`/`target`; new `MemoryCatalogRecord` + `read_catalog_record()` | Memory TOML parse only; `Memory::parse` unchanged |
| `src/catalog/hydrate.rs` | `CatalogKey`, `CatalogEdgeLabel`, revised `CatalogEntity`/`CatalogEdge`/`EdgeTarget`; `from_scanned` accepts memory; `classify_target` uses `CatalogKey`; `CatalogDiagnostic.entity_key` → `CatalogKey`; `scan_catalog` calls memory scan | ~15 type changes, all mechanical |
| `src/catalog/scan.rs` | New `scan_memory_entities()` | Additive; no existing fn signatures change |
| `src/catalog/graph.rs` | `NodeKey` → `CatalogKey` re-export; `CatalogNode.memory_type` | 2 lines + test updates |
| `src/map_server/routes.rs` | `entity_markdown` handler gains memory uid fallback | ~15 lines added |
| `src/map_server/markdown.rs` | New `read_memory_markdown()` | Additive |
| `src/map_server/error.rs` | No changes (existing variants suffice) | — |
| `src/relation_graph.rs` | `validate_relations` handles `CatalogEdgeLabel`, restores invariant | ~10 lines changed |
| `src/priority/graph.rs` | Untouched | — |
| `web/map/` | No changes | — |

---

## Test strategy

### Unit tests (new)

- `memory::read_catalog_record` — valid uid, invalid uid, empty title fallback,
  relations present/absent
- `catalog::scan::scan_memory_entities` — items override shipped, uid ≠ dirname
  → diagnostic, malformed toml → diagnostic, missing shipped/ ok, empty dirs ok
- `catalog::hydrate::CatalogKey::canonical` — Numbered and Memory variants
- `catalog::hydrate::CatalogKey` serialization — flat string for both variants
- `catalog::hydrate::CatalogEdgeLabel` serialization — Validated and Raw
- `catalog::hydrate::from_scanned` — memory nodes have kind_label="MEM",
  kind=null, memory edges have Raw labels
- `catalog::hydrate` empty relation row diagnostics — empty label, empty target

### Integration tests (new)

- `scan_catalog` includes memory nodes and edges in its entity/edge counts
- `CatalogGraph::from_catalog` produces memory nodes in the node map
- `entity_markdown` route — memory uid returns 200 with body, bogus mem_xxx
  returns 400, valid uid not in graph returns 404

### Existing tests (must stay green unchanged)

- All `catalog/*` tests — key type changes are mechanical; existing assertions
  on entity/edge counts and shapes must pass with the new types
- All `map_server/*` tests — markdown route tests for SL/ADR/REQ must pass
- All `relation_graph` tests — `validate_relations` dangler/illegal-row tests
  must pass
- All `priority/*` tests — untouched by design

---

## Risks & mitigations

| Risk | Mitigation |
|---|---|
| `CatalogEdgeLabel` deserialization breaks frontend | Serialization is a flat string (`name()`) — identical to the existing `RelationLabel` JSON shape. Frontend sees no change. |
| Memory TOML schema drifts from `RawMemoryToml` | `RawMemoryToml` is the same struct `Memory::parse` uses — schema drift breaks `memory find` too. Single source of truth. |
| Memory entities without `memory.md` (shipped masters often prose-only) | `read_memory_markdown` returns `EntityNotFound` on missing file — same as numbered entities without a `.md`. Frontend handles this gracefully. |
| `CatalogKey` is `Clone` not `Copy` — perf impact on BTreeSet lookups | Memory is ~60 entities. The BTreeSet in `classify_target` has ~600 entries. `Clone` on a 24-byte String is negligible. |

---

## Remaining open questions

None. All findings from both review passes are resolved:

1. ✅ Flat serialization — manual `impl Serialize` for `CatalogKey`
2. ✅ Memory labels preserved as `Raw(String)` — no silent drops
3. ✅ `memory.rs`-owned scan helper reusing `RawMemoryToml`
4. ✅ `scan_named` for real directories only (no symlinks)
5. ✅ uid == dir name check with diagnostic on mismatch
6. ✅ Malformed records → `CatalogDiagnostic::Error` with populated `entity_key`
7. ✅ Markdown: `is_uid` + graph membership + `safe_join`, no `exists()`
8. ✅ `validate_relations` explicit `CatalogEdgeLabel` handling, invariant restored
9. ✅ `kind: None` serialized-away for memory entities
10. ✅ Empty relation row diagnostics
11. ✅ `memory_type` surfaced in `CatalogNode` JSON, frontend deferred
12. ✅ `read_entity_markdown` still takes `&EntityKey` (unchanged signature), only the route handler unwraps
13. ✅ Memory edge origin file: `record.path.join("memory.toml")`
14. ✅ Error handling: items/ errors propagate, shipped/ errors silently continue
15. ✅ `NodeKey::Entity(key)` → `CatalogKey::Numbered(key)` in `outgoing`/`incoming`
16. ✅ All diagnostic constructors wrap `se.key` in `CatalogKey::Numbered`
