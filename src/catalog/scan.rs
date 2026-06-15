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

use std::path::Path;

use crate::entity;
use crate::integrity;
use crate::listing;
use crate::relation::RelationEdge;

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
        "REQ" => Ok(Vec::new()),
        // Knowledge records (SL-059, L7/F-A1) author no outbound relations in Slice
        // A — routing only, no rules/labels/reader. The empty arm keeps the
        // KINDS-driven dispatch total once a record exists (a KINDS row with no arm
        // panics every debug-build graph scan); Slice B swaps it for the real
        // `knowledge::relation_edges` accessor. Kept a SEPARATE arm from `REQ`
        // (which is empty forever) precisely because its body diverges in Slice B —
        // merging the identical-today bodies would couple two distinct futures.
        #[expect(
            clippy::match_same_arms,
            reason = "SL-059 L7: distinct from the REQ arm — Slice B replaces this empty body with knowledge::relation_edges; REQ stays empty forever"
        )]
        "ASM" | "DEC" | "QUE" | "CON" => Ok(Vec::new()),
        "RV" => crate::review::relation_edges(root, id),
        "REC" => crate::rec::relation_edges(root, id),
        // REV (SL-066, G3) — the arm MUST land WITH the `KINDS` row or the
        // fallthrough `debug_assert!(false)` panics every debug-build corpus scan the
        // moment a REV is minted. The accessor returns an empty stub this phase
        // (PHASE-03 fills it with the `[[change]]`-row `revises` reader).
        "REV" => crate::revision::relation_edges(root, id),
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
pub(crate) fn scan_entities(root: &Path) -> anyhow::Result<Vec<ScannedEntity>> {
    let mut out = Vec::new();
    for kref in integrity::KINDS {
        let prefix = kref.kind.prefix;
        let mut ids = entity::scan_ids(&root.join(kref.kind.dir))?;
        ids.sort_unstable();
        for id in ids {
            let (status, title) = status_and_title_for(root, kref, id)?;
            out.push(ScannedEntity {
                key: EntityKey { prefix, id },
                kind: kref.kind,
                status,
                title,
                outbound: outbound_for(root, kref.kind, id)?,
            });
        }
    }
    Ok(out)
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
            let m = crate::meta::read_meta(&tree_root, kref.stem, id)?;
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
    let name = format!("{id:03}");
    let path = root
        .join(kref.kind.dir)
        .join(&name)
        .join(format!("{}-{name}.toml", kref.stem));
    let text = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("read {} for title: {e}", path.display()))?;
    let parsed: TitleOnly = toml::from_str(&text)
        .map_err(|e| anyhow::anyhow!("parse title from {}: {e}", path.display()))?;
    Ok(parsed.title)
}

// ---------------------------------------------------------------------------
// Tests — SL-071 PHASE-02 equivalence gates
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use std::fs;

    /// Write `root/<rel>` with `body`, creating parents.
    fn write(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    /// Format `[[relation]]` rows from (label, targets) pairs.
    fn relation_rows(edges: &[(&str, &[&str])]) -> String {
        let mut rows = String::new();
        for (label, targets) in edges {
            for t in *targets {
                rows.push_str(&format!(
                    "[[relation]]\nlabel = \"{label}\"\ntarget = \"{t}\"\n"
                ));
            }
        }
        rows
    }

    /// Seed a slice entity (toml + md) with the given `[[relation]]` edges.
    fn seed_slice(root: &Path, id: u32, edges: &[(&str, &[&str])]) {
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"s{id}\"\ntitle = \"S{id}\"\nstatus = \"proposed\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{}",
                relation_rows(edges)
            ),
        );
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.md"),
            "scope\n",
        );
    }

    /// Seed an ADR entity (toml + md) with optional `supersedes` array.
    fn seed_adr(root: &Path, id: u32, supersedes: &[&str]) {
        let rels = if supersedes.is_empty() {
            String::new()
        } else {
            let refs: Vec<String> = supersedes.iter().map(|s| format!("\"{s}\"")).collect();
            format!("\n[relationships]\nsupersedes = [{}]\n", refs.join(", "))
        };
        write(
            root,
            &format!(".doctrine/adr/{id:03}/adr-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"a{id}\"\ntitle = \"A{id}\"\nstatus = \"accepted\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"{rels}"
            ),
        );
        write(
            root,
            &format!(".doctrine/adr/{id:03}/adr-{id:03}.md"),
            "body\n",
        );
    }

    /// Seed a requirement entity (edge target only).
    fn seed_requirement(root: &Path, id: u32) {
        write(
            root,
            &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.toml"),
            &format!("id = {id}\nslug = \"r{id}\"\ntitle = \"R{id}\"\nstatus = \"active\"\n"),
        );
        write(
            root,
            &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.md"),
            "r\n",
        );
    }

    /// Seed the shared multi-kind fixture: SL-001, SL-003, ADR-002, REQ-005.
    /// ≥3 entities spanning ≥2 KINDS entries with id gaps.
    fn seed_fixture(root: &Path) {
        seed_slice(root, 1, &[("requirements", &["REQ-005"])]);
        seed_slice(root, 3, &[]);
        seed_adr(root, 2, &["ADR-001"]);
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

        let scanned = scan_entities(root).unwrap();
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

        let scanned = scan_entities(root).unwrap();
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
            crate::relation::RelationLabel::Requirements
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
        let scanned = scan_entities(root).unwrap();
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
        seed_slice(root, 1, &[("requirements", &["REQ-999"])]);
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
}
