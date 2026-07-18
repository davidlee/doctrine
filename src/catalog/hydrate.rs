// SPDX-License-Identifier: GPL-3.0-only
//! Richer catalog types — `Catalog`, `CatalogEntity`, `CatalogEdge`,
//! `EdgeTarget`, `EdgeOrigin` — and their hydration from a raw `Vec<ScannedEntity>`
//! (SL-071 PHASE-03). `Catalog::from_scanned` is pure: it classifies edge targets
//! via `integrity::parse_canonical_ref`, derives entity paths, and builds
//! structured diagnostics — but reads no files and performs no second disk walk.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::entity;
use crate::integrity;
use crate::memory::{self, MemoryCatalogRecord};
use crate::relation::{RelationLabel, Role};

use super::diagnostic::{CatalogDiagnostic, Severity};
use super::scan::{EntityKey, ScanMode, ScannedEntity};

/// Corpus-wide identity — numbered AND named (memory) entities.
/// Serializes flat: Numbered → "SL-001", Memory → "`mem_019ecf`..."
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CatalogKey {
    Numbered(EntityKey),
    Memory(String),
}

impl CatalogKey {
    pub(crate) fn canonical(&self) -> String {
        match self {
            CatalogKey::Numbered(key) => key.canonical(),
            CatalogKey::Memory(uid) => uid.clone(),
        }
    }
}

impl serde::Serialize for CatalogKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.canonical().serialize(serializer)
    }
}

/// An edge label — validated for numbered, raw for memory.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) enum CatalogEdgeLabel {
    Validated(RelationLabel),
    Raw(String),
}

impl CatalogEdgeLabel {
    pub(crate) fn name(&self) -> &str {
        match self {
            CatalogEdgeLabel::Validated(label) => label.name(),
            CatalogEdgeLabel::Raw(label) => label.as_str(),
        }
    }
}

// ---------------------------------------------------------------------------
// Catalog — the top-level scan result
// ---------------------------------------------------------------------------

/// The hydrated, presentation-neutral result of a full entity corpus scan.
/// Consumer-neutral: every downstream query (inspect, priority, graph, coverage,
/// agent-context) projects from this one structure.
#[derive(Clone, serde::Serialize)]
pub(crate) struct Catalog {
    pub(crate) entities: Vec<CatalogEntity>,
    pub(crate) edges: Vec<CatalogEdge>,
    pub(crate) diagnostics: Vec<CatalogDiagnostic>,
    /// The project-wide estimation/value display units, resolved ONCE in the
    /// shell (`scan_catalog`) from `doctrine.toml` and carried on the catalog
    /// (SL-103 PHASE-02, design §4.4). Serialized onto the catalog; the graph
    /// projection consumes it directly in PHASE-03.
    pub(crate) units: Units,
}

/// The project-wide display units for the estimation and value facets, resolved
/// once from `doctrine.toml` (SL-103 PHASE-02, design §4.4). Stored on the
/// [`Catalog`] so every downstream consumer shares one resolution.
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct Units {
    pub(crate) estimation: String,
    pub(crate) value: String,
}

// ---------------------------------------------------------------------------
// CatalogEntity — one hydrated entity
// ---------------------------------------------------------------------------

/// One entity hydrated from the raw scan, carrying its identity, derived
/// filesystem path, authored metadata, and a source span.
#[derive(Clone, serde::Serialize)]
pub(crate) struct CatalogEntity {
    pub(crate) key: CatalogKey,
    pub(crate) kind_label: &'static str,
    pub(crate) kind: Option<&'static entity::Kind>,
    /// The entity's directory on disk, derived from `EntityKey` + `Kind.dir` —
    /// the same path authority used by the existing readers.
    pub(crate) path: PathBuf,
    pub(crate) title: String,
    pub(crate) status: Option<String>,
    /// Source location for this entity's identity/metadata.
    pub(crate) source: SourceSpan,
    /// Memory type classification for `CatalogKey::Memory` entities;
    /// `None` for numbered entities.
    pub(crate) memory_type: Option<String>,
    /// NOTE: `[estimate]`/`[value]` facets are no longer carried through the
    /// catalog (deleted at SL-222 PHASE-09). A key-presence tripwire in the
    /// scan layer detects any extant top-level key as unmigrated residue.
    /// The entity's body prose, read from its `.md` file.
    /// `None` when bodies were not requested or the `.md` file is missing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) body: Option<String>,
}

// ---------------------------------------------------------------------------
// CatalogEdge — one resolved (or classified) outbound relation
// ---------------------------------------------------------------------------

/// One outbound relation with its target classified and its origin recorded.
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct CatalogEdge {
    pub(crate) source: CatalogKey,
    pub(crate) label: CatalogEdgeLabel,
    /// The intent [`Role`] that refines a `references` edge (SL-149 §2.6 / F1). The
    /// role rides as edge PAYLOAD through the graph/projection — distinct from cordage
    /// overlay keying, which stays LABEL-keyed (one `references` overlay regardless of
    /// role, R5). `Some` only on a `Validated(References)` edge; `None` on every
    /// label-only edge and every `Raw` (free-text / unvalidated memory) edge — a `Raw`
    /// edge carries no typed role. Serialized so the projection/render layer (inspect
    /// inbound, web graph) recovers the role verb (`references(implements)`); the
    /// overlay count is unchanged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) role: Option<Role>,
    /// The free-text descriptor on a `references:concerns` edge (SL-196 §5.2), copied
    /// from the raw [`RelationEdge`] payload. `None` on every edge without one; guarded
    /// with `skip_serializing_if` so a descriptorless edge serializes byte-identically to
    /// the pre-slice /api/graph contract (no `descriptor: null` key). Inbound never
    /// carries a descriptor (design D3).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) descriptor: Option<String>,
    pub(crate) target: EdgeTarget,
    /// Where this edge was authored (which entity file, which field).
    pub(crate) origin: EdgeOrigin,
}

/// The classification of an outbound edge's `target` string.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) enum EdgeTarget {
    /// Target parsed as a canonical ref and the entity exists in the scan.
    Resolved(CatalogKey),
    /// Target parsed as a canonical ref but no entity exists under that id.
    UnresolvedRef {
        /// The raw authored target string.
        raw: String,
    },
    /// Target failed to parse as a canonical ref: free text, unvalidated label,
    /// or unknown kind prefix.
    UnvalidatedText {
        /// The raw authored target string.
        raw: String,
    },
}

/// Where an outbound edge was authored — the entity file and the field/section
/// that contained the `[[relation]]` row.
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct EdgeOrigin {
    /// The entity directory that authored this edge.
    pub(crate) file: PathBuf,
    /// The field or section name (e.g. the `label` value of a `[[relation]]` row).
    pub(crate) field: Option<String>,
}

/// The source location for an authored fact — the entity directory and an
/// optional section/field name. No line/col tracking (deferred to a follow-up
/// slice when a TOML span parser is available).
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct SourceSpan {
    /// The entity directory on disk.
    pub(crate) file: PathBuf,
    /// The TOML section or field that sourced this fact.
    pub(crate) field: Option<String>,
}

// ---------------------------------------------------------------------------
// Hydration — pure projection over scanned entities
// ---------------------------------------------------------------------------

/// Edge label for a memory body `[[mem.…]]` wikilink rendered as a catalog edge
/// (SL-202) — mirrors how an untyped TOML memory relation renders (STD-001).
const MEMORY_WIKILINK_EDGE_LABEL: &str = "related";
/// `EdgeOrigin.field` marker distinguishing a body-wikilink edge from a
/// TOML-relation edge (whose origin field is the relation label).
const MEMORY_WIKILINK_ORIGIN_FIELD: &str = "body";

impl Catalog {
    /// Pure projection of a raw entity scan into a hydrated `Catalog`.
    /// Classifies every edge target via `integrity::parse_canonical_ref`,
    /// derives entity paths from `EntityKey` + `Kind.dir`, and collects
    /// diagnostics for unresolved and unvalidated targets.
    ///
    /// `root` is used only to derive paths — no file reads happen here.
    pub(crate) fn from_scanned(
        root: &Path,
        scanned: &[ScannedEntity],
        memory: &[MemoryCatalogRecord],
        mem_key_map: &BTreeMap<String, String>,
        units: Units,
    ) -> Self {
        // Entity key set for O(log n) target resolution lookups.
        let key_set: BTreeSet<CatalogKey> = scanned
            .iter()
            .map(|entity| CatalogKey::Numbered(entity.key))
            .chain(
                memory
                    .iter()
                    .map(|record| CatalogKey::Memory(record.uid.clone())),
            )
            .collect();

        let mut entities = Vec::with_capacity(scanned.len() + memory.len());
        let mut edges = Vec::new();
        let mut diagnostics = Vec::new();

        for se in scanned {
            let entity_dir = root.join(se.kind.dir).join(format!("{:03}", se.key.id));

            entities.push(CatalogEntity {
                key: CatalogKey::Numbered(se.key),
                kind_label: se.key.prefix,
                kind: Some(se.kind),
                path: entity_dir.clone(),
                title: se.title.clone(),
                status: se.status.clone(),
                memory_type: None,

                body: se.body.clone(),
                source: SourceSpan {
                    file: entity_dir.clone(),
                    field: None,
                },
            });

            for edge in &se.outbound {
                let target = classify_target(&edge.target, &key_set, mem_key_map);
                let origin = EdgeOrigin {
                    file: entity_dir.clone(),
                    field: Some(edge.label.name().to_string()),
                };

                // Generate diagnostics from edge classification.
                match &target {
                    EdgeTarget::UnresolvedRef { raw } => {
                        diagnostics.push(CatalogDiagnostic {
                            file: entity_dir.clone(),
                            entity_key: Some(CatalogKey::Numbered(se.key)),
                            field: Some(edge.label.name().to_string()),
                            message: format!(
                                "dangling reference: `{raw}` does not resolve to any scanned entity"
                            ),
                            severity: Severity::Warning,
                        });
                    }
                    EdgeTarget::UnvalidatedText { raw } => {
                        diagnostics.push(CatalogDiagnostic {
                            file: entity_dir.clone(),
                            entity_key: Some(CatalogKey::Numbered(se.key)),
                            field: Some(edge.label.name().to_string()),
                            message: format!(
                                "unvalidated target: `{raw}` is not a canonical reference"
                            ),
                            severity: Severity::Info,
                        });
                    }
                    EdgeTarget::Resolved(_) => { /* no diagnostic */ }
                }

                edges.push(CatalogEdge {
                    source: CatalogKey::Numbered(se.key),
                    label: CatalogEdgeLabel::Validated(edge.label),
                    // The typed role rides as edge payload (SL-149 F1) — `Some` on a
                    // `references` edge, `None` on every label-only edge.
                    role: edge.role,
                    descriptor: edge.descriptor.clone(),
                    target,
                    origin,
                });
            }
        }

        for record in memory {
            entities.push(CatalogEntity {
                key: CatalogKey::Memory(record.uid.clone()),
                kind_label: "MEM",
                kind: None,
                path: record.path.clone(),
                title: record.title.clone(),
                status: Some(record.status.clone()),
                memory_type: Some(record.memory_type.clone()),

                body: None,
                source: SourceSpan {
                    file: record.path.clone(),
                    field: None,
                },
            });

            // Resolved targets already drawn for this record, so the body pass
            // can dedup against them (INV-1). The TOML pass only *populates*
            // this set — it never checks it, so TOML-vs-TOML multiplicity is
            // unchanged (two TOML relations to one target still draw two edges).
            let mut seen: BTreeSet<CatalogKey> = BTreeSet::new();

            for relation in &record.relations {
                // D10: empty-field rows are surfaced as diagnostics, NOT emitted
                // as blank graph edges. Continue after each empty-field diagnostic
                // so the edges.push below is never reached for degenerate rows.
                if relation.label.is_empty() {
                    diagnostics.push(CatalogDiagnostic {
                        file: record.path.clone(),
                        entity_key: Some(CatalogKey::Memory(record.uid.clone())),
                        field: None,
                        message: "empty relation label".to_string(),
                        severity: Severity::Warning,
                    });
                    continue;
                }
                if relation.target.is_empty() {
                    diagnostics.push(CatalogDiagnostic {
                        file: record.path.clone(),
                        entity_key: Some(CatalogKey::Memory(record.uid.clone())),
                        field: Some(relation.label.clone()),
                        message: "empty relation target".to_string(),
                        severity: Severity::Warning,
                    });
                    continue;
                }

                let target = classify_target(&relation.target, &key_set, mem_key_map);

                // Generate diagnostics for memory-edge target classification,
                // mirroring the numbered-entity pattern (RV-051 F-3).
                if let EdgeTarget::UnresolvedRef { raw } = &target {
                    diagnostics.push(CatalogDiagnostic {
                        file: record.path.join("memory.toml"),
                        entity_key: Some(CatalogKey::Memory(record.uid.clone())),
                        field: Some(relation.label.clone()),
                        message: format!(
                            "dangling reference: memory edge target `{raw}` does not resolve"
                        ),
                        severity: Severity::Warning,
                    });
                }

                // Feed the dedup set (populate-only — see the `seen` comment).
                if let EdgeTarget::Resolved(k) = &target {
                    seen.insert(k.clone());
                }

                edges.push(CatalogEdge {
                    source: CatalogKey::Memory(record.uid.clone()),
                    label: CatalogEdgeLabel::Raw(relation.label.clone()),
                    // A `Raw` (free-text / unvalidated memory) edge carries no typed role
                    // — the role dimension is only on the validated `references` label.
                    role: None,
                    descriptor: None,
                    target,
                    origin: EdgeOrigin {
                        file: record.path.join("memory.toml"),
                        field: Some(relation.label.clone()),
                    },
                });
            }

            // Body wikilink pass (SL-202): body `[[mem.…]]` wikilinks become
            // catalog edges alongside TOML relations. `record.body` feeds edge
            // extraction here; the entity node itself carries no prose
            // (`CatalogEntity.body` stays `None`, set above) — opposite lifecycles.
            if let Some(body) = &record.body {
                for link in crate::links::extract_wikilinks(body) {
                    let target = classify_target(&link.target, &key_set, mem_key_map);
                    match &target {
                        EdgeTarget::Resolved(k) => {
                            // Dedup on the resolved `CatalogKey`: suppress if a TOML
                            // relation OR a prior body wikilink already drew this
                            // edge (one-directional — body defers to TOML; INV-1).
                            if !seen.insert(k.clone()) {
                                continue;
                            }
                        }
                        EdgeTarget::UnvalidatedText { raw } => {
                            // Deliberate divergence from the silent TOML path (F-1):
                            // a well-formed `mem.<key>` wikilink that fails to resolve
                            // is near-certainly a real dangling reference (the
                            // `mem.word.word` extractor shape excludes prose). Scoped
                            // to the body pass — `classify_target` and the TOML
                            // diagnostic are untouched, so TOML `UnvalidatedText`
                            // targets stay silent (behaviour-preservation).
                            diagnostics.push(CatalogDiagnostic {
                                file: record.path.join("memory.md"),
                                entity_key: Some(CatalogKey::Memory(record.uid.clone())),
                                field: Some(MEMORY_WIKILINK_ORIGIN_FIELD.to_string()),
                                message: format!(
                                    "dangling reference: body wikilink `{raw}` does not resolve"
                                ),
                                severity: Severity::Warning,
                            });
                        }
                        EdgeTarget::UnresolvedRef { .. } => {
                            // Unreachable: `extract_wikilinks` yields only `mem.*`/`mem_*`
                            // targets, which `parse_canonical_ref` rejects → memory
                            // branch → never a numbered `UnresolvedRef`. No-op for
                            // match exhaustiveness (design BOUNDARY).
                        }
                    }

                    edges.push(CatalogEdge {
                        source: CatalogKey::Memory(record.uid.clone()),
                        label: CatalogEdgeLabel::Raw(MEMORY_WIKILINK_EDGE_LABEL.to_string()),
                        role: None,
                        descriptor: None,
                        target,
                        origin: EdgeOrigin {
                            file: record.path.join("memory.md"),
                            field: Some(MEMORY_WIKILINK_ORIGIN_FIELD.to_string()),
                        },
                    });
                }
            }
        }

        Self {
            entities,
            edges,
            diagnostics,
            units,
        }
    }
}

/// Classify one edge target string against the set of known entity keys.
///
/// Uses `integrity::parse_canonical_ref` — the same oracle `link` and
/// `validate_relations` use. Four outcomes map to three `EdgeTarget` variants:
/// 1. Parse fails → try memory resolution → `UnvalidatedText` if unresolvable
/// 2. Parse succeeds, entity present in `key_set` → `Resolved(key)`
/// 3. Parse succeeds, entity absent from `key_set` → `UnresolvedRef`
///
/// Memory target resolution (applied when `parse_canonical_ref` fails):
/// - If `raw` is a memory UID (`mem_<hex>`) → look up `CatalogKey::Memory(raw)` in `key_set`
/// - If `raw` is a known memory key → resolve to UID via `mem_key_map` → look up in `key_set`
fn classify_target(
    raw: &str,
    key_set: &BTreeSet<CatalogKey>,
    mem_key_map: &BTreeMap<String, String>,
) -> EdgeTarget {
    if let Ok((kref, id)) = integrity::parse_canonical_ref(raw) {
        let key = CatalogKey::Numbered(EntityKey {
            prefix: kref.kind.prefix,
            id,
        });
        if key_set.contains(&key) {
            EdgeTarget::Resolved(key)
        } else {
            EdgeTarget::UnresolvedRef {
                raw: raw.to_string(),
            }
        }
    } else {
        // Memory target resolution: try UID first, then key→UID lookup.
        let uid = if memory::is_uid(raw) {
            Some(raw.to_string())
        } else {
            mem_key_map.get(raw).cloned()
        };
        if let Some(uid) = uid {
            let mem_key = CatalogKey::Memory(uid);
            if key_set.contains(&mem_key) {
                return EdgeTarget::Resolved(mem_key);
            }
        }
        EdgeTarget::UnvalidatedText {
            raw: raw.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// scan_catalog — the single entry point
// ---------------------------------------------------------------------------

/// Scan the full entity corpus, hydrate into a `Catalog`.
///
/// Calls `scan_entities` (the fail-fast KINDS walk) and `scan_memory_entities`
/// (the memory walk), then `Catalog::from_scanned` (pure projection). The memory
/// walk also yields a key→UID map spanning both corpora (shipped + items) so that
/// memory→memory edge targets authored by readable key resolve to their canonical
/// UID nodes regardless of corpus (ISS-213).
pub(crate) fn scan_catalog(root: &Path, mode: ScanMode) -> anyhow::Result<Catalog> {
    let mut diagnostics = Vec::new();
    let scanned = super::scan::scan_entities(root, &mut diagnostics, mode)?;
    let (memory, mem_key_map) = super::scan::scan_memory_entities(root, &mut diagnostics)?;
    let units = resolve_units(root)?;
    let mut catalog = Catalog::from_scanned(root, &scanned, &memory, &mem_key_map, units);
    catalog.diagnostics.extend(diagnostics);
    Ok(catalog)
}

/// Resolve the project-wide [`Units`] from `<root>/doctrine.toml` (the IMPURE
/// shell — `from_scanned` stays pure). NotFound-TOLERANT ONLY (RV-094 F-4,
/// mirroring `coverage_store::load_config`): an ABSENT file falls to the
/// sub-config defaults (`espresso_shots` / `magic_beans`); a parse error
/// propagates (`?`) and EVERY other read error (permission/I-O) is returned as
/// `Err` — never silently defaulted.
fn resolve_units(root: &Path) -> anyhow::Result<Units> {
    let cfg = match std::fs::read_to_string(root.join(crate::dtoml::DOCTRINE_TOML)) {
        Ok(text) => crate::dtoml::parse(&text)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => crate::dtoml::DoctrineToml::default(),
        Err(e) => return Err(e.into()),
    };
    Ok(Units {
        estimation: crate::estimate::resolve_unit(&cfg.estimation),
        value: crate::value::resolve_unit(&cfg.value),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use crate::catalog::test_helpers::*;
    use crate::test_support::{SCHEMA_BACKLOG, SCHEMA_MEMORY};

    /// Seed the PHASE-03 fixture: SL-001 → REQ-005 (resolved), ADR-002 → ADR-001
    /// (resolved), SL-003 (no edges). Plus a backlog issue with drift free-text.
    fn seed_hydrate_fixture(root: &Path) {
        seed_slice(root, 1, &[("references(implements)", &["REQ-005"])]);
        seed_slice(root, 3, &[]);
        seed_adr(root, 1, &[]);
        seed_adr(root, 2, &[("supersedes", &["ADR-001"])]);
        seed_requirement(root, 5);
    }

    // == VT-1: catalog_hydrates_entities_correctly ==

    #[test]
    fn catalog_hydrates_entities_and_resolved_edges() {
        let dir = tmp();
        let root = dir.path();
        seed_hydrate_fixture(root);

        let catalog = scan_catalog(root, ScanMode::default()).unwrap();

        // Entity count: SL-001, SL-003, ADR-001, ADR-002, REQ-005 = 5
        assert_eq!(
            catalog.entities.len(),
            5,
            "expected 5 entities (SL-001, SL-003, ADR-001, ADR-002, REQ-005)"
        );

        // Verify entity paths
        let sl001 = catalog
            .entities
            .iter()
            .find(|e| e.key.canonical() == "SL-001")
            .unwrap();
        assert_eq!(sl001.path, root.join(".doctrine/slice/001"));
        assert_eq!(sl001.title, "S1");
        assert_eq!(sl001.status.as_deref(), Some("proposed"));
        assert_eq!(sl001.kind.unwrap().prefix, "SL");

        let req005 = catalog
            .entities
            .iter()
            .find(|e| e.key.canonical() == "REQ-005")
            .unwrap();
        assert_eq!(req005.path, root.join(".doctrine/requirement/005"));

        // Edge count: SL-001→REQ-005 (1), ADR-002→ADR-001 (1) = 2
        assert_eq!(catalog.edges.len(), 2);

        // SL-001 → REQ-005: resolved
        let sl001_edge = catalog
            .edges
            .iter()
            .find(|e| e.source.canonical() == "SL-001")
            .unwrap();
        assert_eq!(sl001_edge.label.name(), "references");
        assert_eq!(
            sl001_edge.target,
            EdgeTarget::Resolved(CatalogKey::Numbered(EntityKey {
                prefix: "REQ",
                id: 5
            }))
        );
        assert_eq!(sl001_edge.origin.file, root.join(".doctrine/slice/001"));
        assert_eq!(sl001_edge.origin.field.as_deref(), Some("references"));

        // ADR-002 → ADR-001: resolved (supersedes)
        let adr002_edge = catalog
            .edges
            .iter()
            .find(|e| e.source.canonical() == "ADR-002")
            .unwrap();
        assert_eq!(adr002_edge.label.name(), "supersedes");
        assert_eq!(
            adr002_edge.target,
            EdgeTarget::Resolved(CatalogKey::Numbered(EntityKey {
                prefix: "ADR",
                id: 1
            }))
        );
    }

    // == VT-2: unresolved ref generates Warning diagnostic ==

    #[test]
    fn edge_classification_unresolved_ref_produces_warning() {
        let dir = tmp();
        let root = dir.path();
        // SL-001 → REQ-999 (dangling — parses as canonical ref but not seeded).
        seed_slice(root, 1, &[("references(implements)", &["REQ-999"])]);

        let catalog = scan_catalog(root, ScanMode::default()).unwrap();

        assert_eq!(catalog.entities.len(), 1);
        assert_eq!(catalog.edges.len(), 1);

        // Edge is classified as UnresolvedRef
        let edge = &catalog.edges[0];
        assert_eq!(
            edge.target,
            EdgeTarget::UnresolvedRef {
                raw: "REQ-999".to_string()
            }
        );

        // Diagnostic: one Warning
        let diags: Vec<&CatalogDiagnostic> = catalog
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .collect();
        assert_eq!(diags.len(), 1, "expected one Warning diagnostic");
        let diag = diags[0];
        assert!(diag.message.contains("REQ-999"));
        assert!(diag.message.contains("dangling"));
        assert_eq!(
            diag.entity_key.as_ref().map(|k| k.canonical()),
            Some("SL-001".to_string())
        );
        assert_eq!(diag.field.as_deref(), Some("references"));
    }

    // == VT-3: unvalidated text produces Info diagnostic ==

    #[test]
    fn edge_classification_unvalidated_text_produces_info() {
        let dir = tmp();
        let root = dir.path();
        // Backlog issue with drift → free text.
        write(
            root,
            ".doctrine/backlog/issue/001/backlog-001.toml",
            &format!(
                "schema = \"{SCHEMA_BACKLOG}\"\nversion = 1\n\
             id = 1\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\ntags = []\n\
             [[relation]]\nlabel = \"drift\"\ntarget = \"loose talk\"\n"
            ),
        );
        write(root, ".doctrine/backlog/issue/001/backlog-001.md", "i\n");

        let catalog = scan_catalog(root, ScanMode::default()).unwrap();

        assert_eq!(catalog.entities.len(), 1);
        assert_eq!(catalog.edges.len(), 1);

        // Edge is classified as UnvalidatedText
        let edge = &catalog.edges[0];
        assert_eq!(
            edge.target,
            EdgeTarget::UnvalidatedText {
                raw: "loose talk".to_string()
            }
        );

        // Diagnostic: one Info
        let diags: Vec<&CatalogDiagnostic> = catalog
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Info)
            .collect();
        assert_eq!(diags.len(), 1, "expected one Info diagnostic");
        let diag = diags[0];
        assert!(diag.message.contains("loose talk"));
        assert!(diag.message.contains("not a canonical reference"));
        assert_eq!(diag.field.as_deref(), Some("drift"));
    }

    // == VT-4: entity path derivation ==

    #[test]
    fn entity_path_derivation_matches_expected() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[]);
        seed_adr(root, 2, &[]);
        seed_requirement(root, 5);

        let catalog = scan_catalog(root, ScanMode::default()).unwrap();

        for entity in &catalog.entities {
            let CatalogKey::Numbered(key) = &entity.key else {
                panic!("fixture should only produce numbered entities");
            };
            let kind = entity.kind.unwrap();
            let expected = root.join(kind.dir).join(format!("{:03}", key.id));
            assert_eq!(
                entity.path,
                expected,
                "path mismatch for {}",
                entity.key.canonical()
            );
            assert_eq!(
                entity.source.file,
                expected,
                "source.file mismatch for {}",
                entity.key.canonical()
            );
            // SourceSpan.field is None for entity-level spans (no section/field authored).
            assert!(
                entity.source.field.is_none(),
                "source.field should be None for {}",
                entity.key.canonical()
            );
        }
    }

    // == VT-5: pre-existing equivalence tests still green ==
    // (Verified by running the full test suite. This test is a canary.)

    #[test]
    fn scan_catalog_integration_on_full_fixture() {
        let dir = tmp();
        let root = dir.path();
        seed_hydrate_fixture(root);

        let catalog = scan_catalog(root, ScanMode::default()).unwrap();

        // All edges are resolved (no diagnostics) — fixture has no dangling refs.
        assert_eq!(catalog.diagnostics.len(), 0);

        // 5 entities, 2 edges (SL-001→REQ-005, ADR-002→ADR-001).
        assert_eq!(catalog.entities.len(), 5);
        assert_eq!(catalog.edges.len(), 2);

        // Edge origins point to the source entity directories.
        for edge in &catalog.edges {
            let source_entity = catalog
                .entities
                .iter()
                .find(|e| e.key == edge.source)
                .unwrap();
            assert_eq!(edge.origin.file, source_entity.path);
        }
    }

    // == Additional: classify_target edge cases ==

    #[test]
    fn classify_target_unknown_kind_prefix_is_unvalidated() {
        // ZZ-001 parses as a ref pattern but ZZ is not a known KINDS prefix.
        let empty_set: BTreeSet<CatalogKey> = BTreeSet::new();
        let empty_map: BTreeMap<String, String> = BTreeMap::new();
        let result = classify_target("ZZ-001", &empty_set, &empty_map);
        assert_eq!(
            result,
            EdgeTarget::UnvalidatedText {
                raw: "ZZ-001".to_string()
            }
        );
    }

    #[test]
    fn classify_target_no_dash_is_unvalidated() {
        let empty_set: BTreeSet<CatalogKey> = BTreeSet::new();
        let empty_map: BTreeMap<String, String> = BTreeMap::new();
        let result = classify_target("just_text", &empty_set, &empty_map);
        assert_eq!(
            result,
            EdgeTarget::UnvalidatedText {
                raw: "just_text".to_string()
            }
        );
    }

    #[test]
    fn classify_target_parses_but_absent_is_unresolved() {
        let empty_set: BTreeSet<CatalogKey> = BTreeSet::new();
        let empty_map: BTreeMap<String, String> = BTreeMap::new();
        let result = classify_target("SL-999", &empty_set, &empty_map);
        assert_eq!(
            result,
            EdgeTarget::UnresolvedRef {
                raw: "SL-999".to_string()
            }
        );
    }

    #[test]
    fn classify_target_parses_and_present_is_resolved() {
        let key = EntityKey {
            prefix: "SL",
            id: 1,
        };
        let mut set = BTreeSet::new();
        set.insert(CatalogKey::Numbered(key));
        let empty_map: BTreeMap<String, String> = BTreeMap::new();
        let result = classify_target("SL-001", &set, &empty_map);
        assert_eq!(result, EdgeTarget::Resolved(CatalogKey::Numbered(key)));
    }

    // == Memory integration helpers and tests ==

    fn seed_memory(root: &Path, uid: &str, title: &str, relations: &[(&str, &str)]) {
        use crate::memory::MEMORY_ITEMS_DIR;
        let items_dir = root.join(MEMORY_ITEMS_DIR).join(uid);
        std::fs::create_dir_all(&items_dir).unwrap();
        let rels: Vec<String> = relations
            .iter()
            .map(|(l, t)| format!("[[relation]]\nlabel = \"{l}\"\ntarget = \"{t}\"\n"))
            .collect();
        std::fs::write(
            items_dir.join("memory.toml"),
            format!(
                "schema = \"{SCHEMA_MEMORY}\"\nversion = 1\nmemory_uid = \"{uid}\"\ntitle = \"{title}\"\nstatus = \"active\"\nmemory_type = \"pattern\"\n{}",
                rels.concat()
            ),
        )
        .unwrap();
    }

    /// Seed a SHIPPED memory (corpus dir carries no key symlinks — the whole
    /// point of ISS-213) with an authored `memory_key` and optional relations.
    fn seed_shipped_memory(
        root: &Path,
        uid: &str,
        key: &str,
        title: &str,
        relations: &[(&str, &str)],
    ) {
        use crate::memory::MEMORY_SHIPPED_DIR;
        let ship_dir = root.join(MEMORY_SHIPPED_DIR).join(uid);
        std::fs::create_dir_all(&ship_dir).unwrap();
        let rels: Vec<String> = relations
            .iter()
            .map(|(l, t)| format!("[[relation]]\nlabel = \"{l}\"\ntarget = \"{t}\"\n"))
            .collect();
        std::fs::write(
            ship_dir.join("memory.toml"),
            format!(
                "schema = \"{SCHEMA_MEMORY}\"\nversion = 1\nmemory_uid = \"{uid}\"\nmemory_key = \"{key}\"\ntitle = \"{title}\"\nstatus = \"active\"\nmemory_type = \"signpost\"\n{}",
                rels.concat()
            ),
        )
        .unwrap();
    }

    /// ISS-213: a shipped memory whose relation targets another shipped memory
    /// **by readable key** resolves to a drawn edge (previously dangled as
    /// `UnvalidatedText` because the key→uid map ignored the shipped corpus).
    #[test]
    fn shipped_to_shipped_key_relation_resolves() {
        let dir = tmp();
        let root = dir.path();

        let src_uid = "mem_11111111112222222222333333333344";
        let dst_uid = "mem_aaaaaaaaaabbbbbbbbbbcccccccccccc";
        let dst_key = "mem.signpost.doctrine.lifecycle-start";
        seed_shipped_memory(root, dst_uid, dst_key, "Lifecycle Start", &[]);
        seed_shipped_memory(
            root,
            src_uid,
            "mem.signpost.doctrine.overview",
            "Overview",
            &[("see-also", dst_key)],
        );

        let catalog = scan_catalog(root, ScanMode::default()).unwrap();

        let edge = catalog
            .edges
            .iter()
            .find(|e| e.source == CatalogKey::Memory(src_uid.to_string()))
            .expect("source memory edge present");
        assert_eq!(
            edge.target,
            EdgeTarget::Resolved(CatalogKey::Memory(dst_uid.to_string())),
            "key-form shipped→shipped target must resolve to the dst UID node"
        );
    }

    /// RV-051 F-7: integration test for memory-edge target resolution pipeline.
    #[test]
    fn memory_edge_pipeline_resolves_and_diagnoses() {
        let dir = tmp();
        let root = dir.path();

        seed_slice(root, 1, &[]);

        seed_memory(
            root,
            "mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "Test Memory",
            &[
                ("references", "SL-001"),
                ("references", "SL-999"),
                ("see-also", "some free text"),
            ],
        );

        let catalog = scan_catalog(root, ScanMode::default()).unwrap();

        assert_eq!(catalog.entities.len(), 2);
        assert_eq!(catalog.edges.len(), 3);

        let mem_entity = catalog
            .entities
            .iter()
            .find(|e| matches!(e.key, CatalogKey::Memory(_)))
            .unwrap();
        assert_eq!(mem_entity.kind_label, "MEM");
        assert_eq!(mem_entity.title, "Test Memory");
        assert_eq!(mem_entity.memory_type.as_deref(), Some("pattern"));
        assert!(mem_entity.kind.is_none());

        let resolved_edge = catalog
            .edges
            .iter()
            .find(|e| matches!(&e.target, EdgeTarget::Resolved(k) if k.canonical() == "SL-001"))
            .unwrap();
        assert_eq!(resolved_edge.label.name(), "references");

        let unresolved_edge = catalog
            .edges
            .iter()
            .find(|e| matches!(&e.target, EdgeTarget::UnresolvedRef { raw } if raw == "SL-999"))
            .unwrap();
        assert_eq!(unresolved_edge.label.name(), "references");

        let free_text_edge = catalog
            .edges
            .iter()
            .find(|e| {
                matches!(&e.target, EdgeTarget::UnvalidatedText { raw } if raw == "some free text")
            })
            .unwrap();
        assert_eq!(free_text_edge.label.name(), "see-also");

        let warnings: Vec<&CatalogDiagnostic> = catalog
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .collect();
        assert_eq!(warnings.len(), 1, "expected one dangling-ref Warning");
        assert!(warnings[0].message.contains("SL-999"));
        assert!(warnings[0].message.contains("does not resolve"));
    }

    /// RV-051 F-1: D10 — empty-field memory relations surface diagnostics
    /// but must NOT emit blank graph edges.
    #[test]
    fn memory_empty_relation_fields_surface_diagnostics_not_edges() {
        let dir = tmp();
        let root = dir.path();

        seed_memory(
            root,
            "mem_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "Empty Relations",
            &[("", "SL-001"), ("refs", ""), ("", "")],
        );

        let catalog = scan_catalog(root, ScanMode::default()).unwrap();

        assert_eq!(catalog.entities.len(), 1);
        assert_eq!(
            catalog.edges.len(),
            0,
            "empty-field relations must not produce blank graph edges"
        );

        let warnings: Vec<&CatalogDiagnostic> = catalog
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .collect();
        assert_eq!(warnings.len(), 3);
        assert!(
            warnings
                .iter()
                .any(|d| d.message.contains("empty relation label"))
        );
        assert!(
            warnings
                .iter()
                .any(|d| d.message.contains("empty relation target"))
        );
    }

    /// SL-092 VT-5: scan_catalog propagates entity-scan diagnostics through
    /// Catalog.diagnostics; severity, entity_key, and file survive round-trip.
    #[test]
    fn scan_catalog_propagates_entity_scan_diagnostics() {
        let dir = tmp();
        let root = dir.path();

        // Well-formed entity.
        seed_slice(root, 1, &[]);
        // Malformed entity (meta parse failure).
        write(
            root,
            ".doctrine/slice/002/slice-002.toml",
            "id = notanumber\n",
        );
        write(root, ".doctrine/slice/002/slice-002.md", "scope\n");

        let catalog = scan_catalog(root, ScanMode::default()).unwrap();

        // Only SL-001 in entities.
        assert_eq!(catalog.entities.len(), 1);
        assert!(catalog.entities[0].key.canonical().contains("SL-001"));

        // One Error diagnostic for SL-002.
        assert!(!catalog.diagnostics.is_empty());
        let entity_diags: Vec<_> = catalog
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect();
        assert_eq!(entity_diags.len(), 1);
        assert_eq!(
            entity_diags[0].entity_key.as_ref().map(|k| k.canonical()),
            Some("SL-002".to_string())
        );
        assert!(entity_diags[0].file.to_string_lossy().contains("002"));
        assert!(entity_diags[0].message.contains("SL-002"));
    }

    // == SL-103 PHASE-02: facet carry-through + units resolution ==

    fn test_units() -> Units {
        Units {
            estimation: "espresso_shots".to_string(),
            value: "magic_beans".to_string(),
        }
    }

    /// A numbered `ScannedEntity` with the given risk facet — bypasses the disk scan
    /// so `from_scanned`'s carry-through is exercised in isolation (it is pure).
    /// NOTE: `estimate`/`value` were deleted at SL-222 PHASE-09 and are no longer
    /// carried through the catalog.
    fn scanned_numbered(
        prefix: &'static str,
        id: u32,
        risk: Option<crate::risk::RiskFacet>,
    ) -> ScannedEntity {
        ScannedEntity {
            key: EntityKey { prefix, id },
            kind: crate::integrity::kind_by_prefix(prefix).unwrap().kind,
            status: Some("proposed".to_string()),
            title: format!("{prefix}-{id}"),
            outbound: Vec::new(),
            risk,
            tags: vec![],
            body: None,
        }
    }

    /// PHASE-09: `from_scanned` no longer carries estimate/value facets through
    /// the catalog (deleted). Only risk/tags are present.
    #[test]
    fn from_scanned_carries_no_facets_after_deletion() {
        let dir = tmp();
        let root = dir.path();

        let scanned = vec![
            scanned_numbered("SL", 1, None),
            scanned_numbered("SL", 2, None),
        ];

        let record = MemoryCatalogRecord {
            uid: "mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            key: None,
            title: "M".to_string(),
            status: "active".to_string(),
            memory_type: "pattern".to_string(),
            path: root.join("mem"),
            relations: Vec::new(),
            body: None,
        };

        let catalog =
            Catalog::from_scanned(root, &scanned, &[record], &BTreeMap::new(), test_units());

        // The scan succeeds and entities are present; estimate/value no longer
        // appear on CatalogEntity.
        let sl001 = catalog
            .entities
            .iter()
            .find(|e| e.key.canonical() == "SL-001")
            .unwrap();
        assert_eq!(sl001.key.canonical(), "SL-001");

        // The injected units are stored verbatim on the catalog.
        assert_eq!(catalog.units.estimation, "espresso_shots");
        assert_eq!(catalog.units.value, "magic_beans");
    }

    // == SL-202 PHASE-02: body wikilinks as deduped catalog edges ==

    /// A `MemoryCatalogRecord` for the body-wikilink edge tests — pure
    /// `from_scanned` input, no disk. `path` is synthetic; body edges cite
    /// `path.join("memory.md")` for their origin.
    fn mem_record(
        uid: &str,
        body: Option<&str>,
        relations: Vec<memory::RawRelation>,
    ) -> MemoryCatalogRecord {
        MemoryCatalogRecord {
            uid: uid.to_string(),
            key: None,
            title: "M".to_string(),
            status: "active".to_string(),
            memory_type: "pattern".to_string(),
            path: PathBuf::from("/mem").join(uid),
            relations,
            body: body.map(str::to_string),
        }
    }

    fn raw_relation(label: &str, target: &str) -> memory::RawRelation {
        memory::RawRelation {
            label: label.to_string(),
            target: target.to_string(),
        }
    }

    /// Body edges emitted by this pass, in emission order.
    fn body_edges(catalog: &Catalog) -> Vec<&CatalogEdge> {
        catalog
            .edges
            .iter()
            .filter(|e| e.origin.field.as_deref() == Some("body"))
            .collect()
    }

    const A_UID: &str = "mem_0000000000000000000000000000aaaa";
    const B_UID: &str = "mem_0000000000000000000000000000bbbb";

    /// VT-1: a memory whose only cross-reference is a body `[[mem.<key>]]`
    /// wikilink draws exactly one resolved edge, label `"related"`, origin
    /// field `"body"`, origin file `…/memory.md`.
    #[test]
    fn body_wikilink_draws_one_related_edge() {
        let root = Path::new("/root");
        let a = mem_record(A_UID, Some("See [[mem.target.key]] for more."), vec![]);
        let b = mem_record(B_UID, None, vec![]);
        let mut map = BTreeMap::new();
        map.insert("mem.target.key".to_string(), B_UID.to_string());

        let catalog = Catalog::from_scanned(root, &[], &[a, b], &map, test_units());

        let body = body_edges(&catalog);
        assert_eq!(body.len(), 1, "one body edge");
        let edge = body[0];
        assert_eq!(edge.source, CatalogKey::Memory(A_UID.to_string()));
        assert_eq!(
            edge.target,
            EdgeTarget::Resolved(CatalogKey::Memory(B_UID.to_string()))
        );
        assert_eq!(edge.label, CatalogEdgeLabel::Raw("related".to_string()));
        assert_eq!(edge.role, None);
        assert_eq!(edge.origin.field.as_deref(), Some("body"));
        assert!(edge.origin.file.to_string_lossy().ends_with("memory.md"));
    }

    /// VT-2: a body wikilink AND a TOML relation to the SAME target draw one
    /// edge, not two — the body wikilink defers to the TOML relation (INV-1).
    #[test]
    fn body_wikilink_dedups_against_toml_relation() {
        let root = Path::new("/root");
        let a = mem_record(
            A_UID,
            Some("See [[mem.target.key]]."),
            vec![raw_relation("supports", B_UID)],
        );
        let b = mem_record(B_UID, None, vec![]);
        let mut map = BTreeMap::new();
        map.insert("mem.target.key".to_string(), B_UID.to_string());

        let catalog = Catalog::from_scanned(root, &[], &[a, b], &map, test_units());

        assert!(
            body_edges(&catalog).is_empty(),
            "body edge deduped by TOML relation"
        );
        let to_b = catalog
            .edges
            .iter()
            .filter(|e| e.target == EdgeTarget::Resolved(CatalogKey::Memory(B_UID.to_string())))
            .count();
        assert_eq!(to_b, 1, "exactly one edge to the shared target");
    }

    /// VT-3: two identical body wikilinks to the same target draw one edge
    /// (body-vs-body dedup via the shared `seen` set — INV-1).
    #[test]
    fn duplicate_body_wikilinks_draw_one_edge() {
        let root = Path::new("/root");
        let a = mem_record(
            A_UID,
            Some("First [[mem.target.key]], then again [[mem.target.key]]."),
            vec![],
        );
        let b = mem_record(B_UID, None, vec![]);
        let mut map = BTreeMap::new();
        map.insert("mem.target.key".to_string(), B_UID.to_string());

        let catalog = Catalog::from_scanned(root, &[], &[a, b], &map, test_units());

        assert_eq!(
            body_edges(&catalog).len(),
            1,
            "duplicate body wikilinks collapse to one edge"
        );
    }

    /// VT-4: a body wikilink resolving to `UnvalidatedText` emits one edge plus
    /// one `Warning` (body-pass divergence, INV-2); a TOML relation whose target
    /// is also `UnvalidatedText` stays silent (behaviour-preservation, VA-1).
    #[test]
    fn unresolved_body_wikilink_warns_while_toml_stays_silent() {
        let root = Path::new("/root");
        let a = mem_record(
            A_UID,
            Some("Dangling [[mem.missing.key]]."),
            vec![raw_relation("supports", "mem.also.missing")],
        );
        let mut map = BTreeMap::new();
        // Neither target is mapped → both classify as UnvalidatedText.
        map.insert("mem.unrelated".to_string(), B_UID.to_string());

        let catalog = Catalog::from_scanned(root, &[], &[a], &map, test_units());

        // Body pass: one edge to the UnvalidatedText target.
        let body = body_edges(&catalog);
        assert_eq!(body.len(), 1, "one body edge even when unresolved");
        assert_eq!(
            body[0].target,
            EdgeTarget::UnvalidatedText {
                raw: "mem.missing.key".to_string()
            }
        );

        // Exactly one diagnostic — the body warning; the TOML UnvalidatedText
        // target produces none (the divergence is body-pass-scoped).
        assert_eq!(catalog.diagnostics.len(), 1, "only the body wikilink warns");
        let diag = &catalog.diagnostics[0];
        assert_eq!(diag.severity, Severity::Warning);
        assert_eq!(diag.field.as_deref(), Some("body"));
        assert!(diag.file.to_string_lossy().ends_with("memory.md"));
        assert!(
            !catalog
                .diagnostics
                .iter()
                .any(|d| d.file.to_string_lossy().ends_with("memory.toml")),
            "TOML UnvalidatedText target stays silent"
        );
    }

    /// VT-2: unit resolution — configured `[estimation].unit` / `[value].unit`
    /// surface on `Units`; an absent config falls to the defaults.
    #[test]
    fn resolve_units_reads_config_and_defaults() {
        // Configured units.
        let dir = tmp();
        let root = dir.path();
        write(
            root,
            crate::dtoml::DOCTRINE_TOML,
            "[estimation]\nunit = \"story_points\"\n[value]\nunit = \"gold\"\n",
        );
        let units = resolve_units(root).unwrap();
        assert_eq!(units.estimation, "story_points");
        assert_eq!(units.value, "gold");

        // Present file but no unit keys → defaults.
        let dir2 = tmp();
        let root2 = dir2.path();
        write(root2, crate::dtoml::DOCTRINE_TOML, "[conduct]\n");
        let units2 = resolve_units(root2).unwrap();
        assert_eq!(units2.estimation, "espresso_shots");
        assert_eq!(units2.value, "magic_beans");
    }

    /// VT-3: NotFound is tolerated (default `Units`); a NON-NotFound read error
    /// AND a parse error each propagate as `Err` — never silently defaulted
    /// (RV-094 F-4).
    #[test]
    fn resolve_units_notfound_defaults_but_other_errors_propagate() {
        // Absent doctrine.toml (NotFound) → default units.
        let dir = tmp();
        let root = dir.path();
        let units = resolve_units(root).unwrap();
        assert_eq!(units.estimation, "espresso_shots");
        assert_eq!(units.value, "magic_beans");

        // Parse error → Err, NOT a silent default.
        let dir2 = tmp();
        let root2 = dir2.path();
        write(
            root2,
            crate::dtoml::DOCTRINE_TOML,
            "this is not [valid toml\n",
        );
        assert!(
            resolve_units(root2).is_err(),
            "a parse error must propagate, not default"
        );

        // A non-NotFound read error → Err. `doctrine.toml` is a DIRECTORY here,
        // so the read fails with a kind other than NotFound.
        let dir3 = tmp();
        let root3 = dir3.path();
        let p = root3.join(crate::dtoml::DOCTRINE_TOML);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::create_dir(&p).unwrap();
        assert!(
            resolve_units(root3).is_err(),
            "a non-NotFound read error must propagate, not default"
        );
    }

    // == SL-141 PHASE-02: body forwarding tests ==

    #[test]
    fn scan_catalog_with_bodies_forwards_body_to_catalog_entity() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[]);
        let catalog = scan_catalog(root, ScanMode::include_bodies()).unwrap();
        let e = catalog
            .entities
            .iter()
            .find(|ce| matches!(&ce.key, CatalogKey::Numbered(k) if k.id == 1))
            .unwrap();
        assert!(
            e.body.is_some(),
            "body should be Some when include_bodies is true"
        );
    }

    #[test]
    fn catalog_entity_body_skipped_in_json_when_none() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[]);
        let catalog = scan_catalog(root, ScanMode::default()).unwrap();
        let e = catalog
            .entities
            .iter()
            .find(|ce| matches!(&ce.key, CatalogKey::Numbered(k) if k.id == 1))
            .unwrap();
        let json = serde_json::to_string(e).unwrap();
        assert!(
            !json.contains("\"body\""),
            "JSON should omit body key when None"
        );
    }

    // == SL-196 VT-1: hydrate copies the authored descriptor onto CatalogEdge ==

    /// SL-196 PHASE-03: a `references:concerns` edge authored with a free-text
    /// `descriptor` hydrates that descriptor onto the resulting `CatalogEdge.descriptor`
    /// (the /api/graph payload). Mirrors the `role` payload path.
    #[test]
    fn hydrate_copies_concerns_descriptor_onto_catalog_edge() {
        let dir = tmp();
        let root = dir.path();
        // SL-001 references(concerns) REQ-005, bearing a descriptor.
        crate::catalog::test_helpers::write(
            root,
            ".doctrine/slice/001/slice-001.toml",
            "id = 1\nslug = \"s1\"\ntitle = \"S1\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"concerns\"\n\
             descriptor = \"attention burden\"\ntarget = \"REQ-005\"\n",
        );
        crate::catalog::test_helpers::write(root, ".doctrine/slice/001/slice-001.md", "scope\n");
        seed_requirement(root, 5);

        let catalog = scan_catalog(root, ScanMode::default()).unwrap();
        let edge = catalog
            .edges
            .iter()
            .find(|e| e.source.canonical() == "SL-001")
            .expect("SL-001 edge present");
        assert_eq!(
            edge.descriptor.as_deref(),
            Some("attention burden"),
            "hydrate copies the authored descriptor onto CatalogEdge"
        );
    }

    // == SL-196 VT-2: descriptorless CatalogEdge omits the key (skip_serializing_if) ==

    /// SL-196 PHASE-03: a `CatalogEdge` with `descriptor: None` serializes with NO
    /// `descriptor` key — the `skip_serializing_if` guard keeps the /api/graph edge
    /// contract byte-identical to the pre-slice shape (no `descriptor: null`).
    #[test]
    fn catalog_edge_descriptor_skipped_in_json_when_none() {
        let dir = tmp();
        let root = dir.path();
        // A plain references(implements) edge — no descriptor authored.
        seed_slice(root, 1, &[("references(implements)", &["REQ-005"])]);
        seed_requirement(root, 5);

        let catalog = scan_catalog(root, ScanMode::default()).unwrap();
        let edge = catalog
            .edges
            .iter()
            .find(|e| e.source.canonical() == "SL-001")
            .expect("SL-001 edge present");
        assert!(edge.descriptor.is_none(), "no descriptor authored");
        let json = serde_json::to_string(edge).unwrap();
        assert!(
            !json.contains("\"descriptor\""),
            "JSON omits descriptor key when None: {json}"
        );
    }
}
