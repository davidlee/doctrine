// SPDX-License-Identifier: GPL-3.0-only
//! The KINDS-driven entity corpus scanner — the single source of truth for
//! the all-kind raw scan (SL-071). Re-homed from `relation_graph.rs`; consumed
//! by both `relation_graph` (via re-exports) and the richer `catalog` types.
//!
//! Six items moved here:
//! - `outbound_for` — the outbound relation dispatch over `integrity::KINDS`
//! - `EntityKey` — the corpus-wide identity type
//! - `ScannedEntity` — the reusable scan record
//! - `scan_entities` — the KINDS-walk entry point
//! - `status_and_title_for` — one parse per entity (private helper)
//! - `title_for` — lenient title-only read (private helper)

use std::collections::BTreeMap;
use std::path::Path;

use crate::entity::{self};
use crate::estimate::{self, EstimateFacet};
use crate::integrity;
use crate::listing;
use crate::relation::RelationEdge;
use crate::risk::{self, RiskFacet};
use crate::value::{self, ValueFacet};

use super::diagnostic::{CatalogDiagnostic, Severity};
use super::hydrate::CatalogKey;

/// Every authored outbound relation of one entity, dispatched to the owning kind's
/// `relation_edges` accessor by canonical prefix (design §5.2 — one data-driven match
/// over all 14 `integrity::KINDS` rows; the design's "11" counts overlay LABELS, not
/// kinds). Each accessor reads only its own private relations via that kind's existing
/// show-path reader — the adapter never re-parses TOML (cohesion, §5.3). Kinds that
/// author no outbound edges (`REQUIREMENT` — an edge *target* only) return `Ok(vec![])`.
///
/// Grouping by `kind.prefix` (the corpus-wide discriminant used everywhere, e.g.
/// `integrity::kind_by_prefix`): SLICE→slice; ADR/POL/STD→governance (parameterised
/// by the kind's `GovKind`); PRD/SPEC→spec (by subtype); ISS/IMP/CHR/RSK/IDE→backlog
/// (by `ItemKind`); RV→review; REC→rec; REV→revision (SL-066 G3, empty stub this
/// phase — PHASE-03 reads the `[[change]]` rows).
pub(crate) fn outbound_for(
    root: &Path,
    kind: &entity::Kind,
    id: u32,
) -> anyhow::Result<Vec<RelationEdge>> {
    match kind.prefix {
        "SL" => crate::slice::relation_edges(root, id),
        "ADR" => crate::governance::relation_edges(&crate::adr::ADR_KIND, root, id),
        "POL" => crate::governance::relation_edges(&crate::policy::POLICY_KIND, root, id),
        "STD" => crate::governance::relation_edges(&crate::standard::STANDARD_KIND, root, id),
        "PRD" => crate::spec::relation_edges(crate::spec::SpecSubtype::Product, root, id),
        "SPEC" => crate::spec::relation_edges(crate::spec::SpecSubtype::Tech, root, id),
        // REQUIREMENT authors no outbound relations — it is an edge target only.
        // CM (concept-map, SL-076) likewise authors no outbound relations.
        "REQ" | "CM" => Ok(Vec::new()),
        // Knowledge records (SL-059, L7/F-A1) author no outbound relations in Slice
        // A — routing only, no rules/labels/reader. The empty arm keeps the
        // KINDS-driven dispatch total once a record exists (a KINDS row with no arm
        // panics every debug-build graph scan); Slice B swaps it for the real
        // `knowledge::relation_edges` accessor. Kept a SEPARATE arm from `REQ`
        // (which is empty forever) precisely because its body diverges in Slice B —
        // merging the identical-today bodies would couple two distinct futures.
        "ASM" | "DEC" | "QUE" | "CON" => {
            let record_kind = crate::knowledge::RecordKind::from_prefix(kind.prefix)
                .ok_or_else(|| anyhow::anyhow!("unknown record prefix {}", kind.prefix))?;
            crate::knowledge::relation_edges(root, record_kind, id)
        }
        "RV" => crate::review::relation_edges(root, id),
        "REC" => crate::rec::relation_edges(root, id),
        // REV (SL-066, G3) — the arm MUST land WITH the `KINDS` row or the
        // fallthrough `debug_assert!(false)` panics every debug-build corpus scan the
        // moment a REV is minted. The accessor returns an empty stub this phase
        // (PHASE-03 fills it with the `[[change]]`-row `revises` reader).
        "REV" => crate::revision::relation_edges(root, id),
        // RFC (SL-122) — governance-spine dispatching via governance::relation_edges
        // (reads tier-1 `[[relation]]` rows). PHASE-03 filled.
        "RFC" => crate::governance::relation_edges(&crate::rfc::RFC_KIND, root, id),
        // The five backlog kinds share one accessor, routed by their ItemKind (the
        // prefix↔kind map is backlog's single source — no second copy here).
        other => {
            if let Some(item_kind) = crate::backlog::kind_from_prefix(other) {
                crate::backlog::relation_edges(root, item_kind, id)
            } else {
                // Unreachable for any `integrity::KINDS` row (the explicit arms above
                // plus the five backlog prefixes route every kind). A new KINDS row
                // with no arm here lands here — loud in debug (the invariant), a
                // benign empty in release (dispatch stays total, never a panic).
                debug_assert!(false, "outbound_for: unrouted KINDS prefix `{other}`");
                Ok(Vec::new())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// All-kind scan types and entry point.
// ---------------------------------------------------------------------------

/// The projection key for a numbered entity (design §5.2). Stores the kind's
/// `&'static str` prefix — `Copy + Ord`, unlike `entity::Kind` (which is data, not
/// `Ord`, and carries a fn-ptr `scaffold`) — and the numeric id. The pair is the
/// corpus-wide identity, and renders its canonical ref through the same
/// `listing::canonical_id` source `ItemId` uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub(crate) struct EntityKey {
    pub(crate) prefix: &'static str,
    pub(crate) id: u32,
}

impl EntityKey {
    /// The canonical ref string (`SL-046`) for this key — the single id-form
    /// authority, shared with every other prefixed surface (`listing::canonical_id`).
    pub(crate) fn canonical(self) -> String {
        listing::canonical_id(self.prefix, self.id)
    }
}

/// One scanned entity from the all-kind raw scan (the SL-047 D5 seam): its
/// [`EntityKey`], its AUTHORED status (`None` for the genuinely status-less kinds),
/// and its authored outbound relations verbatim (unresolved — resolution is the
/// consumer's edge pass). This is the REUSABLE half of the old `build_relation_graph`
/// — the KINDS-walk scan with NO reference graph built on top — consumed by BOTH
/// `inspect` (`build_relation_graph`) and `priority::graph::build` (EX-5). No second
/// KINDS-walk lives anywhere else (no parallel implementation).
pub(crate) struct ScannedEntity {
    pub(crate) key: EntityKey,
    /// The kind descriptor (data, not `Ord`) — captured from the `KindRef` in the
    /// scan, so the priority consumer (SL-047) needs no second `kind_by_prefix`
    /// lookup. Now live (the priority adapter reads it), so the PHASE-01 self-clearing
    /// `dead_code` scope has retired itself.
    pub(crate) kind: &'static entity::Kind,
    pub(crate) status: Option<String>,
    /// The entity's authored `title`, captured in the scan so the priority display
    /// surfaces need no second read (SL-047 PHASE-03). Read leniently
    /// ([`title_for`]) so a status-less kind (RV/REC, whose strict
    /// [`crate::meta::Meta`] read fails for lack of a top-level `status`) still yields
    /// its title.
    pub(crate) title: String,
    pub(crate) outbound: Vec<RelationEdge>,
    /// The entity's optional `[estimate]` facet (SL-103 PHASE-01) — read
    /// kind-agnostically in the scan, with per-facet malformed isolation (D4):
    /// a malformed PRESENT estimate drops to `None` and pushes an `Error`
    /// diagnostic, leaving the node and the sibling `value` facet intact. Now
    /// consumed by the hydrate projection (PHASE-02 — `Catalog::from_scanned`
    /// copies it onto each `CatalogEntity`).
    pub(crate) estimate: Option<EstimateFacet>,
    /// The entity's optional `[value]` facet (SL-103 PHASE-01) — read with the
    /// same kind-agnostic, per-facet isolation as [`Self::estimate`]. Now
    /// consumed by the hydrate projection (PHASE-02).
    pub(crate) value: Option<ValueFacet>,
    /// The entity's optional `[facet]` (risk) table (SL-133 PHASE-03) — read
    /// with the same kind-agnostic, per-facet isolation as [`Self::estimate`]
    /// and [`Self::value`]. A malformed `[facet]` drops only `risk` to `None`
    /// and pushes an `Error` diagnostic. Consumed by the priority graph base
    /// pre-pass (PHASE-04).
    pub(crate) risk: Option<RiskFacet>,
    /// The entity's authored `tags`, read from the same parsed TOML as the
    /// facets — already normalized at rest (SL-136).
    pub(crate) tags: Vec<String>,
    /// The entity's body prose, read from its `.md` file.
    /// `None` when `ScanMode::default()` or when the `.md` file is missing.
    pub(crate) body: Option<String>,
}

/// Controls optional body reading during entity scan.
/// Default is bodyless (backwards-compatible).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ScanMode {
    pub include_bodies: bool,
}

impl ScanMode {
    pub(crate) const fn include_bodies() -> Self {
        Self {
            include_bodies: true,
        }
    }
}

/// The all-kind raw scan (design §5.2 — the reusable seam factored out of
/// `build_relation_graph`). Walk `integrity::KINDS` in TABLE order; per kind
/// `scan_ids` (already skips the `NNN-slug` symlink + non-dirs — VT-5 free), **sort
/// ids ascending** (C5 — `scan_ids` is unsorted `read_dir` order; the sort makes the
/// scan order — and thus every consumer's mint/render — permutation-invariant,
/// REQ-077), then per entity read its AUTHORED status and title in one combined read
/// ([`status_and_title_for`]) and its authored outbound edges ([`outbound_for`]).
/// Yields entities in KINDS-table /
/// id-ascending order — the SAME order `build_relation_graph`'s old pass-1 minted in,
/// so `inspect`'s mint order (and therefore its byte-identical output) is preserved.
///
/// Disk touches live here (the thin imperative shell — `scan_ids`/
/// `status_and_title_for`/`outbound_for` read the entity tomls); a consumer's
/// tally/mint/edge policy stays pure over the returned `Vec`.
pub(crate) fn scan_entities(
    root: &Path,
    diagnostics: &mut Vec<CatalogDiagnostic>,
    mode: ScanMode,
) -> anyhow::Result<Vec<ScannedEntity>> {
    let mut out = Vec::new();
    for kref in integrity::KINDS {
        let prefix = kref.kind.prefix;
        let mut ids = entity::scan_ids(&root.join(kref.kind.dir))?;
        ids.sort_unstable();
        for id in ids {
            let (status, title) = match status_and_title_for(root, kref, id) {
                Ok(v) => v,
                Err(e) => {
                    diagnostics.push(CatalogDiagnostic {
                        file: root.join(kref.kind.dir).join(format!("{id:03}")),
                        entity_key: Some(CatalogKey::Numbered(EntityKey { prefix, id })),
                        field: None,
                        message: format!("failed to read {prefix}-{id:03}: {e}"),
                        severity: Severity::Error,
                    });
                    continue;
                }
            };
            let outbound = match outbound_for(root, kref.kind, id) {
                Ok(v) => v,
                Err(e) => {
                    diagnostics.push(CatalogDiagnostic {
                        file: root.join(kref.kind.dir).join(format!("{id:03}")),
                        entity_key: Some(CatalogKey::Numbered(EntityKey { prefix, id })),
                        field: None,
                        message: format!("failed to read relations for {prefix}-{id:03}: {e}"),
                        severity: Severity::Error,
                    });
                    continue;
                }
            };
            let (estimate, value, risk, tags) = read_facets(root, kref, id, diagnostics);
            let body = if mode.include_bodies {
                read_body(root, kref.kind, id, diagnostics)
            } else {
                None
            };
            out.push(ScannedEntity {
                key: EntityKey { prefix, id },
                kind: kref.kind,
                status,
                title,
                outbound,
                estimate,
                value,
                risk,
                tags,
                body,
            });
        }
    }
    Ok(out)
}

/// Read the optional `[estimate]` / `[value]` / `[facet]` (risk) facets and
/// the `tags` array off one entity's toml, kind-agnostically, with PER-FACET
/// malformed isolation (SL-103 PHASE-01 / SL-133 PHASE-03, design §5.1 / D4).
/// Each facet parses independently: a malformed PRESENT facet pushes an `Error`
/// diagnostic and drops THAT facet to `None` — no bound coercion, no silent
/// repair — leaving the node and the sibling facets intact. Tags are lenient:
/// absent or non-array → empty vec; already normalized at rest (SL-136).
///
/// The status read ([`status_and_title_for`]) has already validated this file
/// parses; a vanished/garbled file HERE is a concurrent-edit window — the status
/// read is the authority and re-diagnoses next scan, so v1 returns absent rather
/// than re-report.
fn read_facets(
    root: &Path,
    kref: &integrity::KindRef,
    id: u32,
    diagnostics: &mut Vec<CatalogDiagnostic>,
) -> (
    Option<EstimateFacet>,
    Option<ValueFacet>,
    Option<RiskFacet>,
    Vec<String>,
) {
    let path = entity::id_path(root, kref.kind, id, entity::Ext::Toml);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return (None, None, None, Vec::new());
    };
    let Ok(table) = text.parse::<toml::Table>() else {
        return (None, None, None, Vec::new());
    };
    let tags: Vec<String> = table
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let estimate = parse_facet(
        "estimate",
        table.get("estimate"),
        estimate::parse_optional,
        root,
        kref,
        id,
        diagnostics,
    );
    let value = parse_facet(
        "value",
        table.get("value"),
        value::parse_optional,
        root,
        kref,
        id,
        diagnostics,
    );
    let risk = parse_facet(
        "facet",
        table.get("facet"),
        risk::parse_optional,
        root,
        kref,
        id,
        diagnostics,
    );
    (estimate, value, risk, tags)
}

/// Read the entity's body `.md` file. Missing file → `None`
/// (no diagnostic — many kinds legitimately lack bodies).
/// IO/UTF-8 errors → diagnostic + `None` (scan continues).
fn read_body(
    root: &Path,
    kind: &'static entity::Kind,
    id: u32,
    diagnostics: &mut Vec<CatalogDiagnostic>,
) -> Option<String> {
    let path = entity::id_path(root, kind, id, entity::Ext::Md);
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() { None } else { Some(text) }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            diagnostics.push(CatalogDiagnostic {
                file: path,
                entity_key: None,
                field: None,
                message: format!("failed to read body: {e}"),
                severity: Severity::Error,
            });
            None
        }
    }
}

/// Parse ONE optional facet keyed `field` off the entity's parsed toml, isolating
/// its failure (SL-103 D4). A key PRESENT but not a TOML table is fail-loud (an
/// `Error` diagnostic + `None`); a present table that fails `parse`'s validation
/// likewise diagnoses and drops to `None`; absent is a silent `None`. The
/// diagnostic carries the entity dir as `file`, the entity key, `field`, and the
/// leaf error message verbatim. Generic over the two `parse_optional` signatures
/// — the single dedup for both facets.
fn parse_facet<F, T>(
    field: &str,
    raw: Option<&toml::Value>,
    parse: F,
    root: &Path,
    kref: &integrity::KindRef,
    id: u32,
    diagnostics: &mut Vec<CatalogDiagnostic>,
) -> Option<T>
where
    F: FnOnce(Option<&toml::value::Table>) -> anyhow::Result<Option<T>>,
{
    let mut push = |message: String| {
        diagnostics.push(CatalogDiagnostic {
            file: root.join(kref.kind.dir).join(format!("{id:03}")),
            entity_key: Some(CatalogKey::Numbered(EntityKey {
                prefix: kref.kind.prefix,
                id,
            })),
            field: Some(field.to_string()),
            message,
            severity: Severity::Error,
        });
    };
    match raw {
        // Present but not a table → fail-loud, NOT silent-absent.
        Some(v) if v.as_table().is_none() => {
            push(format!("{field} must be a table"));
            None
        }
        other => match parse(other.and_then(toml::Value::as_table)) {
            Ok(facet) => facet,
            Err(e) => {
                push(e.to_string());
                None
            }
        },
    }
}

/// One entity's AUTHORED `(status, title)` for the cross-kind scan, dispatched by
/// canonical prefix (the same data-driven shape as [`outbound_for`]). For the COMMON
/// (non-RV/REC) path this is ONE parse: the shared `meta::read_meta` deserializes the
/// full [`crate::meta::Meta`], which already carries BOTH `status` and `title`, so the
/// status and title come from a single toml read (SL-050 F1 — collapsing the former
/// `status_for` + `title_for` double-parse).
///
/// REC is genuinely status-less (one record per act, no lifecycle) ⇒ `None` status,
/// and its title comes from the lenient [`title_for`] (its toml authors no top-level
/// `status`, so strict `read_meta` would fail). RV authors no `status` field either,
/// but carries a status DERIVED at read time from its authored finding ledger
/// (`review::derived_status_string`, D-C8) — authored-tier, not a runtime read — with
/// its title likewise read leniently. RV/REC therefore still take two reads each
/// (derived/ledger status + lenient title); that residual is scope-sanctioned (F1).
/// The `kref` carries both the tree dir and the toml `stem`.
fn status_and_title_for(
    root: &Path,
    kref: &integrity::KindRef,
    id: u32,
) -> anyhow::Result<(Option<String>, String)> {
    match kref.kind.prefix {
        // Status-less by design — no diagnostic, just absent; lenient title.
        "REC" => Ok((None, title_for(root, kref, id)?)),
        // Derived (authored-tier) status over the finding ledger; lenient title.
        "RV" => Ok((
            Some(crate::review::derived_status_string(root, id)?),
            title_for(root, kref, id)?,
        )),
        // Every other kind stores both `status` and `title` top-level — ONE parse.
        _ => {
            let tree_root = root.join(kref.kind.dir);
            let m = crate::meta::read_meta(&tree_root, kref.kind.stem, id, kref.kind.prefix)?;
            Ok((Some(m.status), m.title))
        }
    }
}

/// One entity's authored `title` for the cross-kind scan, read leniently. Every
/// kind authors a top-level `title` in its `<stem>-NNN.toml` (slice/governance/spec/
/// requirement/backlog) or beside its `[review]`/`[rec]` table (RV/REC) — but the
/// strict [`crate::meta::read_meta`] also demands `status`, which RV/REC do NOT
/// author top-level. So a `title`-only deserialize (ignoring every other key) is the
/// one reader that works across ALL kinds. The `kref` carries the tree dir + stem.
fn title_for(root: &Path, kref: &integrity::KindRef, id: u32) -> anyhow::Result<String> {
    #[derive(serde::Deserialize)]
    struct TitleOnly {
        title: String,
    }
    let path = entity::id_path(root, kref.kind, id, entity::Ext::Toml);
    let text = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("read {} for title: {e}", path.display()))?;
    let parsed: TitleOnly = toml::from_str(&text)
        .map_err(|e| anyhow::anyhow!("parse title from {}: {e}", path.display()))?;
    Ok(parsed.title)
}

// ---------------------------------------------------------------------------
// Memory entities scan (SL-081 PHASE-03)
// ---------------------------------------------------------------------------

/// Scan memory entities from `MEMORY_ITEMS_DIR` and `MEMORY_SHIPPED_DIR`.
pub(crate) fn scan_memory_entities(
    root: &Path,
    diagnostics: &mut Vec<CatalogDiagnostic>,
) -> anyhow::Result<Vec<crate::memory::MemoryCatalogRecord>> {
    use crate::memory::{MEMORY_ITEMS_DIR, MEMORY_SHIPPED_DIR};
    let mut records: BTreeMap<String, crate::memory::MemoryCatalogRecord> = BTreeMap::new();
    for (dir, fail_on_error) in [(MEMORY_SHIPPED_DIR, false), (MEMORY_ITEMS_DIR, true)] {
        let base = root.join(dir);
        let names = match entity::scan_named(&base) {
            Ok(n) => n,
            Err(_) if !fail_on_error => continue,
            Err(e) => return Err(e),
        };
        for name in &names {
            let toml_path = base.join(name).join("memory.toml");
            match crate::memory::read_catalog_record(&toml_path) {
                Ok(rec) => {
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

// ---------------------------------------------------------------------------
// Tests — SL-071 PHASE-02 equivalence gates
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use crate::catalog::test_helpers::*;

    /// Seed the shared multi-kind fixture: SL-001, SL-003, ADR-002, REQ-005.
    /// ≥3 entities spanning ≥2 KINDS entries with id gaps.
    fn seed_fixture(root: &Path) {
        seed_slice(root, 1, &[("references(implements)", &["REQ-005"])]);
        seed_slice(root, 3, &[]);
        seed_adr(root, 2, &[("supersedes", &["ADR-001"])]);
        seed_requirement(root, 5);
    }

    /// Helper: canonical keys from a scan.
    fn canonical_keys(entities: &[ScannedEntity]) -> Vec<String> {
        entities.iter().map(|e| e.key.canonical()).collect()
    }

    // == T2: scan_order_is_stable ==

    #[test]
    fn scan_order_follows_kinds_table_then_id_ascending() {
        let dir = tmp();
        let root = dir.path();
        // Seed out of order: SL-003 before SL-001 on disk (proves sort, not
        // readdir order, determines output).
        seed_slice(root, 3, &[]);
        seed_slice(root, 1, &[]);
        seed_adr(root, 2, &[]);

        let scanned = scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        let keys = canonical_keys(&scanned);

        // KINDS-table order: SL before ADR. Within SL: id ascending.
        assert_eq!(
            keys,
            vec!["SL-001", "SL-003", "ADR-002"],
            "scan order must be KINDS-table order, ids ascending per kind"
        );
    }

    // == T3: catalog_scan_matches_legacy_shape ==

    #[test]
    fn scan_entity_shape_matches_expected() {
        let dir = tmp();
        let root = dir.path();
        seed_fixture(root);

        let scanned = scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        // Order is proven by T2; here we assert shape on the first entity.
        let sl001 = &scanned[0];
        // Shape: (key.canonical(), kind.prefix, status, title, outbound tuples).
        assert_eq!(sl001.key.canonical(), "SL-001");
        assert_eq!(sl001.key.prefix, "SL");
        assert_eq!(sl001.kind.prefix, "SL");
        assert_eq!(sl001.status.as_deref(), Some("proposed"));
        assert_eq!(sl001.title, "S1");
        assert_eq!(sl001.outbound.len(), 1);
        assert_eq!(
            sl001.outbound[0].label,
            crate::relation::RelationLabel::References
        );
        assert_eq!(
            sl001.outbound[0].role,
            Some(crate::relation::Role::Implements)
        );
        assert_eq!(sl001.outbound[0].target, "REQ-005");

        // SL-003 — no outbound edges.
        let sl003 = &scanned[1];
        assert_eq!(sl003.key.canonical(), "SL-003");
        assert_eq!(sl003.kind.prefix, "SL");
        assert_eq!(sl003.status.as_deref(), Some("proposed"));
        assert_eq!(sl003.title, "S3");
        assert!(sl003.outbound.is_empty());

        // ADR-002 — governance kind with supersedes edge.
        let adr002 = &scanned[2];
        assert_eq!(adr002.key.canonical(), "ADR-002");
        assert_eq!(adr002.kind.prefix, "ADR");
        assert_eq!(adr002.status.as_deref(), Some("accepted"));
        assert_eq!(adr002.title, "A2");
        assert_eq!(adr002.outbound.len(), 1);
        assert_eq!(
            adr002.outbound[0].label,
            crate::relation::RelationLabel::Supersedes
        );
        assert_eq!(adr002.outbound[0].target, "ADR-001");

        // REQ-005 — target-only kind, no outbound relations.
        let req005 = &scanned[3];
        assert_eq!(req005.key.canonical(), "REQ-005");
        assert_eq!(req005.kind.prefix, "REQ");
        assert_eq!(req005.status.as_deref(), Some("active"));
        assert_eq!(req005.title, "R5");
        assert!(req005.outbound.is_empty());
    }

    // == T5: priority_graph_shape_unchanged ==

    #[test]
    fn priority_graph_node_set_matches_scanned() {
        let dir = tmp();
        let root = dir.path();
        seed_fixture(root);

        let pg = crate::priority::graph::build(root).unwrap();
        // Node count equals scanned entity count.
        let scanned = scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        assert_eq!(pg.attrs.len(), scanned.len());
        // Every scanned key resolves in the projection.
        let scanned_keys: std::collections::BTreeSet<EntityKey> =
            scanned.iter().map(|e| e.key).collect();
        for k in &scanned_keys {
            assert!(
                pg.projection.resolve(*k).is_some(),
                "{} must resolve in the priority graph",
                k.canonical()
            );
        }
        // Overlay handles are present (the build doesn't panic and they exist).
        let _ = pg.dep_overlay;
        let _ = pg.seq_overlay;
        // The resolved edge S-001 → REQ-005 exists: count out-edges of SL-001
        // across all overlays (Members overlay carries the requirements edge).
        let sl001_node = pg.projection.resolve(scanned[0].key).unwrap();
        let sl001_out: usize = [pg.dep_overlay, pg.seq_overlay]
            .iter()
            .map(|&ov| pg.graph.out_edges(ov, sl001_node).count())
            .sum();
        // No dep/seq edges in this fixture — but the ref overlay edge is not
        // accessible from the public API (overlay IDs are private). The test
        // asserts the building doesn't panic and node cardinality holds.
        let _ = sl001_out;
    }

    // == T6: validate_relation_findings_unchanged ==

    #[test]
    fn validate_reports_dangling_edge_and_ignores_free_text() {
        let dir = tmp();
        let root = dir.path();
        // SL-001: requirements edge to REQ-999 (dangling — not seeded).
        seed_slice(root, 1, &[("references(implements)", &["REQ-999"])]);
        // A backlog issue with a free-text `drift` target (Unvalidated) —
        // must NOT be a finding.
        write(
            root,
            ".doctrine/backlog/issue/001/backlog-001.toml",
            "schema = \"doctrine.backlog\"\nversion = 1\n\
             id = 1\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\ntags = []\n\
             [[relation]]\nlabel = \"drift\"\ntarget = \"loose talk\"\n",
        );
        write(root, ".doctrine/backlog/issue/001/backlog-001.md", "i\n");

        let findings = crate::relation_graph::validate_relations(root).unwrap();
        let joined = findings.join("\n");

        // Dangling REQ-999 is reported.
        assert!(
            joined.contains("SL-001") && joined.contains("REQ-999") && joined.contains("dangling"),
            "dangling REQ-999 must be reported: {joined}"
        );
        // Free-text `drift` target is NOT a finding.
        assert!(
            !joined.contains("loose talk"),
            "Unvalidated drift target must not be reported: {joined}"
        );
    }

    // -----------------------------------------------------------------------
    // Memory scan tests — SL-081 PHASE-03
    // -----------------------------------------------------------------------

    /// Write a memory.toml under `root/.doctrine/memory/<tree>/<dir>/memory.toml`.
    fn seed_memory(root: &Path, tree: &str, dir: &str, body: &str) -> std::path::PathBuf {
        let dir_path = root.join(".doctrine/memory").join(tree).join(dir);
        std::fs::create_dir_all(&dir_path).unwrap();
        let toml_path = dir_path.join("memory.toml");
        std::fs::write(&toml_path, body).unwrap();
        toml_path
    }

    // == VT-1: valid memory.toml in items/ → record returned ==

    #[test]
    fn scan_memory_entities_valid_item_record_returned() {
        let dir = tmp();
        let root = dir.path();

        seed_memory(
            root,
            "items",
            "mem_11111111112222222222333333333344",
            "memory_uid = \"mem_11111111112222222222333333333344\"\n\
             memory_type = \"concept\"\n\
             status = \"active\"\n\
             title = \"Test Memory\"\n",
        );

        let mut diags = Vec::new();
        let records = scan_memory_entities(root, &mut diags).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].uid, "mem_11111111112222222222333333333344");
        assert_eq!(records[0].title, "Test Memory");
        assert_eq!(records[0].status, "active");
        assert_eq!(records[0].memory_type, "concept");
        assert!(diags.is_empty());
    }

    // == VT-2: items/ overrides shipped/ with same uid ==

    #[test]
    fn scan_memory_entities_items_overrides_shipped() {
        let dir = tmp();
        let root = dir.path();

        let uid = "mem_11111111112222222222333333333344";
        seed_memory(
            root,
            "shipped",
            uid,
            &format!(
                "memory_uid = \"{uid}\"\n\
                 memory_type = \"concept\"\n\
                 status = \"draft\"\n\
                 title = \"Shipped Version\"\n"
            ),
        );
        seed_memory(
            root,
            "items",
            uid,
            &format!(
                "memory_uid = \"{uid}\"\n\
                 memory_type = \"concept\"\n\
                 status = \"active\"\n\
                 title = \"Items Version\"\n"
            ),
        );

        let mut diags = Vec::new();
        let records = scan_memory_entities(root, &mut diags).unwrap();

        assert_eq!(records.len(), 1, "items should override shipped");
        assert_eq!(records[0].title, "Items Version");
        assert!(diags.is_empty());
    }

    // == VT-3: uid != dirname → Error diagnostic, excluded ==

    #[test]
    fn scan_memory_entities_uid_dirname_mismatch_diagnostic() {
        let dir = tmp();
        let root = dir.path();

        let dirname = "mem_11111111112222222222333333333344";
        let wrong_uid = "mem_aaaaaaaaaabbbbbbbbbbcccccccccccc";
        seed_memory(
            root,
            "items",
            dirname,
            &format!(
                "memory_uid = \"{wrong_uid}\"\n\
                 memory_type = \"concept\"\n\
                 status = \"active\"\n\
                 title = \"Mismatched\"\n"
            ),
        );

        let mut diags = Vec::new();
        let records = scan_memory_entities(root, &mut diags).unwrap();

        // The record is excluded (uid mismatch).
        assert!(records.is_empty());

        // One Error diagnostic.
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Severity::Error);
        assert!(diags[0].message.contains("does not match directory name"));
        assert!(diags[0].message.contains(wrong_uid));
        assert!(diags[0].message.contains(dirname));
        assert_eq!(
            diags[0].entity_key.as_ref().map(|k| k.canonical()),
            Some(dirname.to_string())
        );
    }

    // == VT-4: malformed toml → Error diagnostic ==

    #[test]
    fn scan_memory_entities_malformed_toml_diagnostic() {
        let dir = tmp();
        let root = dir.path();

        let uid = "mem_11111111112222222222333333333344";
        seed_memory(root, "items", uid, "this is not valid toml at all[[[\n");

        let mut diags = Vec::new();
        let records = scan_memory_entities(root, &mut diags).unwrap();

        assert!(records.is_empty());
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Severity::Error);
        assert!(diags[0].message.contains("failed to read memory record"));
        assert_eq!(
            diags[0].entity_key.as_ref().map(|k| k.canonical()),
            Some(uid.to_string())
        );
    }

    // == VT-5: missing shipped/ dir → Ok(vec![]) ==

    #[test]
    fn scan_memory_entities_missing_shipped_ok_empty() {
        let dir = tmp();
        let root = dir.path();

        let mut diags = Vec::new();
        let records = scan_memory_entities(root, &mut diags).unwrap();

        assert!(records.is_empty());
        assert!(diags.is_empty());
    }

    // == VT-6: empty both dirs → Ok(vec![]) ==

    #[test]
    fn scan_memory_entities_empty_both_dirs_ok_empty() {
        let dir = tmp();
        let root = dir.path();

        // Create the dirs but leave them empty.
        std::fs::create_dir_all(root.join(".doctrine/memory/shipped")).unwrap();
        std::fs::create_dir_all(root.join(".doctrine/memory/items")).unwrap();

        let mut diags = Vec::new();
        let records = scan_memory_entities(root, &mut diags).unwrap();

        assert!(records.is_empty());
        assert!(diags.is_empty());
    }

    // == SL-092 PHASE-02: graceful scan degradation tests ==

    /// VT-1: malformed sibling TOML (status_and_title_for failure) →
    /// remaining entities + one Error diagnostic with correct entity_key.
    #[test]
    fn scan_entities_skips_malformed_meta_and_emits_diagnostic() {
        let dir = tmp();
        let root = dir.path();

        // SL-001: well-formed
        seed_slice(root, 1, &[]);
        // SL-002: malformed TOML — meta parse will fail
        write(
            root,
            ".doctrine/slice/002/slice-002.toml",
            "id = notanumber\n",
        );
        write(root, ".doctrine/slice/002/slice-002.md", "scope\n");

        let mut diags = Vec::new();
        let scanned = scan_entities(root, &mut diags, ScanMode::default()).unwrap();

        // Only SL-001 returned.
        assert_eq!(scanned.len(), 1);
        assert_eq!(scanned[0].key.canonical(), "SL-001");
        assert_eq!(scanned[0].title, "S1");

        // One diagnostic for SL-002.
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Severity::Error);
        assert_eq!(
            diags[0].entity_key.as_ref().map(|k| k.canonical()),
            Some("SL-002".to_string())
        );
        assert!(diags[0].file.to_string_lossy().contains("002"));
        assert!(diags[0].message.contains("SL-002"));
    }

    /// VT-2: malformed [[relation]] block (outbound_for failure) →
    /// entity skipped + Error diagnostic; message differs from VT-1 meta case.
    #[test]
    fn scan_entities_skips_malformed_relations_and_emits_diagnostic() {
        let dir = tmp();
        let root = dir.path();

        // SL-001: well-formed
        seed_slice(root, 1, &[]);
        // SL-002: valid meta but malformed [[relation]] — missing target field
        write(
            root,
            ".doctrine/slice/002/slice-002.toml",
            "id = 2\nslug = \"s2\"\ntitle = \"S2\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [[relation]]\nlabel = \"supersedes\"\n",
        );
        write(root, ".doctrine/slice/002/slice-002.md", "scope\n");

        let mut diags = Vec::new();
        let scanned = scan_entities(root, &mut diags, ScanMode::default()).unwrap();

        // Only SL-001 returned.
        assert_eq!(scanned.len(), 1);
        assert_eq!(scanned[0].key.canonical(), "SL-001");

        // One Error diagnostic for SL-002.
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Severity::Error);
        assert_eq!(
            diags[0].entity_key.as_ref().map(|k| k.canonical()),
            Some("SL-002".to_string())
        );
        // Message must mention "relations" (outbound_for failure), not just "read" (meta failure).
        assert!(diags[0].message.contains("relations"));
    }

    /// VT-3: all-malformed siblings → empty Vec + N diagnostics; no panic.
    #[test]
    fn scan_entities_all_malformed_returns_empty_no_panic() {
        let dir = tmp();
        let root = dir.path();

        // Three malformed SL entities, none parse properly.
        for id in 1..=3u32 {
            write(
                root,
                &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
                "id = garbage\n",
            );
            write(
                root,
                &format!(".doctrine/slice/{id:03}/slice-{id:03}.md"),
                "scope\n",
            );
        }

        let mut diags = Vec::new();
        let scanned = scan_entities(root, &mut diags, ScanMode::default()).unwrap();

        assert!(scanned.is_empty(), "no entities should be returned");
        assert_eq!(diags.len(), 3, "one diagnostic per malformed entity");
        for d in &diags {
            assert_eq!(d.severity, Severity::Error);
        }
    }

    /// VT-4: mixed-validity (two good, one bad) → two entities, one diagnostic.
    #[test]
    fn scan_entities_mixed_validity_returns_good_and_skips_bad() {
        let dir = tmp();
        let root = dir.path();

        // SL-001: well-formed
        seed_slice(root, 1, &[]);
        // SL-002: malformed
        write(root, ".doctrine/slice/002/slice-002.toml", "id = bogus\n");
        write(root, ".doctrine/slice/002/slice-002.md", "scope\n");
        // SL-003: well-formed
        seed_slice(root, 3, &[]);

        let mut diags = Vec::new();
        let scanned = scan_entities(root, &mut diags, ScanMode::default()).unwrap();

        assert_eq!(scanned.len(), 2);
        let keys: Vec<String> = scanned.iter().map(|e| e.key.canonical()).collect();
        assert_eq!(keys, vec!["SL-001", "SL-003"]);
        assert_eq!(diags.len(), 1);
        assert_eq!(
            diags[0].entity_key.as_ref().map(|k| k.canonical()),
            Some("SL-002".to_string())
        );
    }

    // == SL-103 PHASE-01: scan-side estimate/value facet read ==

    /// Seed a slice with explicit `[estimate]` / `[value]` table bodies appended
    /// (the standard `seed_slice` writes no facets). `facets` is appended verbatim
    /// after the meta keys, so a caller controls the exact (mal)formed shape.
    /// NOTE: a slice's own typed read ([`crate::slice::read_slice`]) validates
    /// `[estimate]`, so MALFORMED-facet isolation is exercised on an ADR
    /// ([`seed_adr_with_facets`]) — a kind whose scan reads only raw relation text,
    /// leaving `read_facets` the sole facet authority (the per-facet isolation seam).
    fn seed_slice_with_facets(root: &Path, id: u32, facets: &str) {
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"s{id}\"\ntitle = \"S{id}\"\nstatus = \"proposed\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{facets}"
            ),
        );
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.md"),
            "scope\n",
        );
    }

    /// Seed an ADR with `facets` appended verbatim. Unlike a slice, the ADR scan
    /// path (`governance::relation_edges` → raw `tier1_edges`; `meta::read_meta`
    /// ignoring unknown keys) never type-checks `[estimate]`/`[value]`, so a
    /// malformed or non-table facet survives to `read_facets` — the isolation unit.
    fn seed_adr_with_facets(root: &Path, id: u32, facets: &str) {
        write(
            root,
            &format!(".doctrine/adr/{id:03}/adr-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"a{id}\"\ntitle = \"A{id}\"\nstatus = \"accepted\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{facets}"
            ),
        );
        write(
            root,
            &format!(".doctrine/adr/{id:03}/adr-{id:03}.md"),
            "body\n",
        );
    }

    /// VT-1: a faceted entity yields `(Some, Some)`; a non-faceted entity yields
    /// `(None, None)` with NO diagnostic.
    #[test]
    fn read_facets_reads_present_and_absent() {
        let dir = tmp();
        let root = dir.path();
        // SL-001: both facets present and valid.
        seed_slice_with_facets(
            root,
            1,
            "[estimate]\nlower = 2\nupper = 8\n\n[value]\nvalue = 5\n",
        );
        // SL-002: no facets at all.
        seed_slice(root, 2, &[]);

        let mut diags = Vec::new();
        let scanned = scan_entities(root, &mut diags, ScanMode::default()).unwrap();

        let sl001 = &scanned[0];
        assert_eq!(sl001.key.canonical(), "SL-001");
        assert_eq!(
            sl001.estimate,
            Some(EstimateFacet {
                lower: 2.0,
                upper: 8.0
            })
        );
        assert_eq!(sl001.value, Some(ValueFacet { value: 5.0 }));

        let sl002 = &scanned[1];
        assert_eq!(sl002.key.canonical(), "SL-002");
        assert!(sl002.estimate.is_none());
        assert!(sl002.value.is_none());

        // A non-faceted entity raises no facet diagnostic.
        assert!(diags.is_empty(), "no diagnostics expected: {diags:?}");
    }

    /// VT-2: per-facet isolation — a malformed `[estimate]` (upper < lower) next
    /// to a VALID `[value]` on the SAME entity pushes one `Error`, drops estimate
    /// to `None`, and leaves `value == Some`. Seeded on an ADR so the malformed
    /// facet reaches `read_facets` (a slice would reject it in its own typed read).
    #[test]
    fn read_facets_isolates_malformed_estimate_from_valid_value() {
        let dir = tmp();
        let root = dir.path();
        seed_adr_with_facets(
            root,
            1,
            "[estimate]\nlower = 5\nupper = 2\n\n[value]\nvalue = 7\n",
        );

        let mut diags = Vec::new();
        let scanned = scan_entities(root, &mut diags, ScanMode::default()).unwrap();

        // The node survives, with the sibling facet intact.
        assert_eq!(scanned.len(), 1);
        let adr = &scanned[0];
        assert_eq!(adr.key.canonical(), "ADR-001");
        assert!(adr.estimate.is_none(), "malformed estimate drops to None");
        assert_eq!(
            adr.value,
            Some(ValueFacet { value: 7.0 }),
            "sibling value facet stays intact"
        );

        // Exactly one Error diagnostic, for the estimate field, verbatim message.
        assert_eq!(diags.len(), 1);
        let d = &diags[0];
        assert_eq!(d.severity, Severity::Error);
        assert_eq!(d.field.as_deref(), Some("estimate"));
        assert_eq!(
            d.entity_key.as_ref().map(|k| k.canonical()),
            Some("ADR-001".to_string())
        );
        assert!(
            d.message.contains("upper must be >= lower"),
            "leaf message verbatim: {}",
            d.message
        );
    }

    /// VT-3: a non-table facet value (`estimate = 7`) is fail-loud, NOT
    /// silent-absent — an `Error` diagnostic + `None`. Seeded on an ADR (a slice
    /// would reject the non-table `estimate` in its own typed read first).
    #[test]
    fn read_facets_present_non_table_is_fail_loud() {
        let dir = tmp();
        let root = dir.path();
        seed_adr_with_facets(root, 1, "estimate = 7\n");

        let mut diags = Vec::new();
        let scanned = scan_entities(root, &mut diags, ScanMode::default()).unwrap();

        assert_eq!(scanned.len(), 1);
        assert!(scanned[0].estimate.is_none());
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].field.as_deref(), Some("estimate"));
        assert!(
            diags[0].message.contains("must be a table"),
            "{}",
            diags[0].message
        );
    }

    /// VT-4: kind-agnostic read — an `[estimate]` authored on an ADR (a NON-slice
    /// toml) is read, proving the read is corpus-wide, not slice-only.
    #[test]
    fn read_facets_is_kind_agnostic_reads_adr_estimate() {
        let dir = tmp();
        let root = dir.path();
        // An ADR with an appended [estimate] facet.
        write(
            root,
            ".doctrine/adr/001/adr-001.toml",
            "id = 1\nslug = \"a1\"\ntitle = \"A1\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [estimate]\nlower = 1\nupper = 3\n",
        );
        write(root, ".doctrine/adr/001/adr-001.md", "body\n");

        let mut diags = Vec::new();
        let scanned = scan_entities(root, &mut diags, ScanMode::default()).unwrap();

        let adr = scanned
            .iter()
            .find(|e| e.key.canonical() == "ADR-001")
            .expect("ADR-001 scanned");
        assert_eq!(
            adr.estimate,
            Some(EstimateFacet {
                lower: 1.0,
                upper: 3.0
            }),
            "estimate read off a non-slice kind"
        );
        assert!(diags.is_empty(), "no diagnostics: {diags:?}");
    }

    /// VT-1b (SL-133 PHASE-03): per-facet isolation — a malformed `[facet]`
    /// (bad likelihood token) next to VALID `[estimate]` and `[value]` on the
    /// SAME entity pushes one `Error`, drops `risk` to `None`, and leaves
    /// estimate + value intact. Seeded on an ADR so the malformed facet reaches
    /// `read_facets` (a slice would reject it in its own typed read).
    #[test]
    fn read_facets_isolates_malformed_risk_facet_from_valid_estimate_and_value() {
        let dir = tmp();
        let root = dir.path();
        seed_adr_with_facets(
            root,
            1,
            "[estimate]\nlower = 2\nupper = 8\n\n[value]\nvalue = 5\n\n[facet]\nlikelihood = \"bogus\"\nimpact = \"high\"\n",
        );

        let mut diags = Vec::new();
        let scanned = scan_entities(root, &mut diags, ScanMode::default()).unwrap();

        // The node survives, with sibling facets intact.
        assert_eq!(scanned.len(), 1);
        let adr = &scanned[0];
        assert_eq!(adr.key.canonical(), "ADR-001");
        assert!(adr.risk.is_none(), "malformed risk facet drops to None");
        assert_eq!(
            adr.estimate,
            Some(EstimateFacet {
                lower: 2.0,
                upper: 8.0
            }),
            "sibling estimate facet stays intact"
        );
        assert_eq!(
            adr.value,
            Some(ValueFacet { value: 5.0 }),
            "sibling value facet stays intact"
        );

        // Exactly one Error diagnostic, for the facet field, verbatim message.
        assert_eq!(diags.len(), 1);
        let d = &diags[0];
        assert_eq!(d.severity, Severity::Error);
        assert_eq!(d.field.as_deref(), Some("facet"));
        assert_eq!(
            d.entity_key.as_ref().map(|k| k.canonical()),
            Some("ADR-001".to_string())
        );
        assert!(
            d.message.contains("invalid likelihood"),
            "leaf message verbatim: {}",
            d.message
        );
    }

    // == SL-141 PHASE-02: body reading tests ==

    #[test]
    fn scan_mode_default_produces_no_body() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[]);
        let scanned = scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        let e = scanned.iter().find(|s| s.key.id == 1).unwrap();
        assert!(e.body.is_none(), "default mode should not read bodies");
    }

    #[test]
    fn scan_mode_include_bodies_reads_body() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[]);
        let scanned = scan_entities(root, &mut vec![], ScanMode::include_bodies()).unwrap();
        let e = scanned.iter().find(|s| s.key.id == 1).unwrap();
        assert!(e.body.is_some(), "include_bodies mode should read body");
        // The body should contain the seeded markdown.
        assert!(
            e.body.as_deref().unwrap().contains("scope"),
            "body should contain seeded markdown"
        );
    }

    // == SL-142 PHASE-01: tags parsing tests ==

    /// Seed a slice with explicit `tags` array in its TOML.
    fn seed_slice_with_tags(root: &Path, id: u32, tags_toml: &str) {
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"s{id}\"\ntitle = \"S{id}\"\nstatus = \"proposed\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{tags_toml}"
            ),
        );
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.md"),
            "scope\n",
        );
    }

    /// VT-2: tags key present with string array → parsed.
    #[test]
    fn read_facets_tags_present_string_array() {
        let dir = tmp();
        let root = dir.path();
        seed_slice_with_tags(root, 1, "tags = [\"foo\", \"bar\"]\n");

        let scanned = scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        assert_eq!(scanned.len(), 1);
        assert_eq!(scanned[0].tags, vec!["foo", "bar"]);
    }

    /// VT-3: absent tags key → empty vec.
    #[test]
    fn read_facets_tags_absent_is_empty() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[]);

        let scanned = scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        assert_eq!(scanned.len(), 1);
        assert!(scanned[0].tags.is_empty());
    }

    /// VT-4: non-array tags value → empty vec (graceful). Seeded on a REC
    /// entity — REC's `status_and_title_for` reads title leniently via
    /// [`title_for`], so the non-array `tags` key survives to `read_facets`
    /// (a slice/ADR's `Meta` deserialization would reject it first).
    #[test]
    fn read_facets_tags_non_array_is_empty() {
        let dir = tmp();
        let root = dir.path();
        // REC-001: a reconciliation record whose title_for only reads `title`.
        write(
            root,
            ".doctrine/rec/001/rec-001.toml",
            "id = 1\nslug = \"r\"\ntitle = \"R\"\ntags = \"not-an-array\"\n\
             [rec]\nmove = \"accept\"\nowning_slice = \"SL-001\"\n",
        );

        let scanned = scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        assert_eq!(scanned.len(), 1);
        assert!(scanned[0].tags.is_empty());
    }

    /// VT-5: non-normalized tag passes through byte-identical (proves no
    /// normalize_tag in the read path).
    #[test]
    fn read_facets_tags_un_normalized_passes_through() {
        let dir = tmp();
        let root = dir.path();
        seed_slice_with_tags(root, 1, "tags = [\"  unpadded  \", \"\"]\n");

        let scanned = scan_entities(root, &mut vec![], ScanMode::default()).unwrap();
        assert_eq!(scanned.len(), 1);
        assert_eq!(
            scanned[0].tags,
            vec!["  unpadded  ".to_string(), String::new()],
            "tags pass through byte-identical — no normalize_tag in read path"
        );
    }
}
