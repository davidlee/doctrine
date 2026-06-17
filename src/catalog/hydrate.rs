// SPDX-License-Identifier: GPL-3.0-only
//! Richer catalog types — `Catalog`, `CatalogEntity`, `CatalogEdge`,
//! `EdgeTarget`, `EdgeOrigin` — and their hydration from a raw `Vec<ScannedEntity>`
//! (SL-071 PHASE-03). `Catalog::from_scanned` is pure: it classifies edge targets
//! via `integrity::parse_canonical_ref`, derives entity paths, and builds
//! structured diagnostics — but reads no files and performs no second disk walk.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::entity;
use crate::integrity;
use crate::memory::MemoryCatalogRecord;
use crate::relation::RelationLabel;

use super::diagnostic::{CatalogDiagnostic, Severity};
use super::scan::{EntityKey, ScannedEntity};

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
}

// ---------------------------------------------------------------------------
// CatalogEdge — one resolved (or classified) outbound relation
// ---------------------------------------------------------------------------

/// One outbound relation with its target classified and its origin recorded.
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct CatalogEdge {
    pub(crate) source: CatalogKey,
    pub(crate) label: CatalogEdgeLabel,
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
                source: SourceSpan {
                    file: entity_dir.clone(),
                    field: None,
                },
            });

            for edge in &se.outbound {
                let target = classify_target(&edge.target, &key_set);
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
                source: SourceSpan {
                    file: record.path.clone(),
                    field: None,
                },
            });

            for relation in &record.relations {
                if relation.label.is_empty() {
                    diagnostics.push(CatalogDiagnostic {
                        file: record.path.clone(),
                        entity_key: Some(CatalogKey::Memory(record.uid.clone())),
                        field: None,
                        message: "empty relation label".to_string(),
                        severity: Severity::Warning,
                    });
                }
                if relation.target.is_empty() {
                    diagnostics.push(CatalogDiagnostic {
                        file: record.path.clone(),
                        entity_key: Some(CatalogKey::Memory(record.uid.clone())),
                        field: Some(relation.label.clone()),
                        message: "empty relation target".to_string(),
                        severity: Severity::Warning,
                    });
                }

                edges.push(CatalogEdge {
                    source: CatalogKey::Memory(record.uid.clone()),
                    label: CatalogEdgeLabel::Raw(relation.label.clone()),
                    target: classify_target(&relation.target, &key_set),
                    origin: EdgeOrigin {
                        file: record.path.join("memory.toml"),
                        field: Some(relation.label.clone()),
                    },
                });
            }
        }

        Self {
            entities,
            edges,
            diagnostics,
        }
    }
}

/// Classify one edge target string against the set of known entity keys.
///
/// Uses `integrity::parse_canonical_ref` — the same oracle `link` and
/// `validate_relations` use. Four outcomes map to three `EdgeTarget` variants:
/// 1. Parse fails → `UnvalidatedText`
/// 2. Parse succeeds, entity present in `key_set` → `Resolved(key)`
/// 3. Parse succeeds, entity absent from `key_set` → `UnresolvedRef`
fn classify_target(raw: &str, key_set: &BTreeSet<CatalogKey>) -> EdgeTarget {
    match integrity::parse_canonical_ref(raw) {
        Ok((kref, id)) => {
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
        }
        Err(_) => EdgeTarget::UnvalidatedText {
            raw: raw.to_string(),
        },
    }
}

// ---------------------------------------------------------------------------
// scan_catalog — the single entry point
// ---------------------------------------------------------------------------

/// Scan the full entity corpus, hydrate into a `Catalog`.
///
/// Calls `scan_entities` (the fail-fast KINDS walk) and `scan_memory_entities`
/// (the memory walk), then `Catalog::from_scanned` (pure projection).
pub(crate) fn scan_catalog(root: &Path) -> anyhow::Result<Catalog> {
    let scanned = super::scan::scan_entities(root)?;
    let mut diagnostics = Vec::new();
    let memory = super::scan::scan_memory_entities(root, &mut diagnostics)?;
    let mut catalog = Catalog::from_scanned(root, &scanned, &memory);
    catalog.diagnostics.extend(diagnostics);
    Ok(catalog)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use crate::catalog::test_helpers::*;

    /// Seed the PHASE-03 fixture: SL-001 → REQ-005 (resolved), ADR-002 → ADR-001
    /// (resolved), SL-003 (no edges). Plus a backlog issue with drift free-text.
    fn seed_hydrate_fixture(root: &Path) {
        seed_slice(root, 1, &[("requirements", &["REQ-005"])]);
        seed_slice(root, 3, &[]);
        seed_adr(root, 1, &[]);
        seed_adr(root, 2, &["ADR-001"]);
        seed_requirement(root, 5);
    }

    // == VT-1: catalog_hydrates_entities_correctly ==

    #[test]
    fn catalog_hydrates_entities_and_resolved_edges() {
        let dir = tmp();
        let root = dir.path();
        seed_hydrate_fixture(root);

        let catalog = scan_catalog(root).unwrap();

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
        assert_eq!(sl001_edge.label.name(), "requirements");
        assert_eq!(
            sl001_edge.target,
            EdgeTarget::Resolved(CatalogKey::Numbered(EntityKey {
                prefix: "REQ",
                id: 5
            }))
        );
        assert_eq!(sl001_edge.origin.file, root.join(".doctrine/slice/001"));
        assert_eq!(sl001_edge.origin.field.as_deref(), Some("requirements"));

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
        seed_slice(root, 1, &[("requirements", &["REQ-999"])]);

        let catalog = scan_catalog(root).unwrap();

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
        assert_eq!(diag.field.as_deref(), Some("requirements"));
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
            "schema = \"doctrine.backlog\"\nversion = 1\n\
             id = 1\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\ntags = []\n\
             [[relation]]\nlabel = \"drift\"\ntarget = \"loose talk\"\n",
        );
        write(root, ".doctrine/backlog/issue/001/backlog-001.md", "i\n");

        let catalog = scan_catalog(root).unwrap();

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

        let catalog = scan_catalog(root).unwrap();

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

        let catalog = scan_catalog(root).unwrap();

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
        let result = classify_target("ZZ-001", &empty_set);
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
        let result = classify_target("just_text", &empty_set);
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
        let result = classify_target("SL-999", &empty_set);
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
        let result = classify_target("SL-001", &set);
        assert_eq!(result, EdgeTarget::Resolved(CatalogKey::Numbered(key)));
    }
}
