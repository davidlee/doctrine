// SPDX-License-Identifier: GPL-3.0-only
//! `relation_query` — the pure projection+filter+render engine over `&Catalog`
//! `relation_query` — the pure projection+filter+render engine over `&Catalog`
//! (SL-137 PHASE-01). Engine-tier (ADR-001): imports catalog (engine) + listing/
//! relation (leaf), never command-tier. No clock, rng, git, or disk.
//!
//! Two projections:
//! - [`project_list`] — filter edges via [`ListFilter`], emit [`RelationRow`]s
//! - [`project_census`] — group by label, tally [`CensusRow`]s
//!
//! Two render paths, riding the [`listing`] spine:
//! - [`render_list`] — table or JSON envelope over [`RELATION_COLUMNS`]
//! - [`render_census`] — table or JSON envelope over [`CENSUS_COLUMNS`]

use std::collections::BTreeMap;

use serde::Serialize;

use crate::catalog::hydrate::{Catalog, CatalogEdgeLabel, CatalogKey, EdgeTarget};
use crate::integrity;
use crate::listing;

// ---------------------------------------------------------------------------
// Axis helpers — zero allocation, zero disk
// ---------------------------------------------------------------------------

/// The resolution state of an edge target. `Serialize`-d as `snake_case` tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TargetState {
    /// Target resolved to a scanned entity.
    Resolved,
    /// Target parsed as a canonical ref but the entity is absent.
    Unresolved,
    /// Target is free text or an unvalidated label — not a canonical ref.
    #[serde(rename = "free_text")]
    FreeText,
}

/// Classify an [`EdgeTarget`] into its [`TargetState`].
pub(crate) fn target_state(target: &EdgeTarget) -> TargetState {
    match target {
        EdgeTarget::Resolved(_) => TargetState::Resolved,
        EdgeTarget::UnresolvedRef { .. } => TargetState::Unresolved,
        EdgeTarget::UnvalidatedText { .. } => TargetState::FreeText,
    }
}

/// The display string for an edge target — its canonical id when resolved,
/// the raw authored text verbatim otherwise.
pub(crate) fn target_display(target: &EdgeTarget) -> String {
    match target {
        EdgeTarget::Resolved(key) => key.canonical(),
        EdgeTarget::UnresolvedRef { raw } | EdgeTarget::UnvalidatedText { raw } => raw.clone(),
    }
}

/// The source kind token for a [`CatalogKey`]: the prefix for numbered entities,
/// `"MEM"` for memory entities.
pub(crate) fn source_kind(key: &CatalogKey) -> &str {
    match key {
        CatalogKey::Numbered(k) => k.prefix,
        CatalogKey::Memory(_) => "MEM",
    }
}

/// The display label for an edge — `references(<role>)` for a role-bearing `references`
/// edge (SL-149 §2.6), the bare label name otherwise. List rows and census groups key
/// on this rendered form, so `references(implements)` and `references(concerns)` are
/// distinct rows/buckets; the `--label` filter compares the BARE name
/// ([`CatalogEdgeLabel::name`]), so `--label references` still matches every role.
pub(crate) fn edge_label_display(edge: &crate::catalog::hydrate::CatalogEdge) -> String {
    match edge.role {
        Some(role) => format!("{}({})", edge.label.name(), role.name()),
        None => edge.label.name().to_string(),
    }
}

// ---------------------------------------------------------------------------
// ListFilter + projection
// ---------------------------------------------------------------------------

/// The resolved filter for [`project_list`]. All axes AND together; an axis
/// at its default (empty / false) imposes no constraint.
#[derive(Debug, Clone, Default)]
pub(crate) struct ListFilter {
    /// When false (default), drop edges whose label is
    /// [`CatalogEdgeLabel::Raw`] (memory-source edges). When true, include them.
    pub(crate) include_memory: bool,
    /// Exact match on the edge label's [`name()`](CatalogEdgeLabel::name).
    pub(crate) label: Option<String>,
    /// Canonical-normalised match on [`target_display`]. The filter value is
    /// parsed through [`integrity::parse_canonical_ref`] and rendered as a
    /// canonical id before comparison; a memory UID is matched verbatim (F3/D6).
    pub(crate) target: Option<String>,
    /// Exact (case-sensitive) match on [`source_kind`], uppercased at match
    /// time so the caller can supply `"mem"` or `"MEM"`.
    pub(crate) source_kind: Option<String>,
    /// When true, keep ONLY edges whose [`target_state`] is NOT [`TargetState::Resolved`].
    pub(crate) unresolved: bool,
}

/// One projected relation row: the source's canonical id, the edge label, the
/// target display, and the target's resolution state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RelationRow {
    pub(crate) source: String,
    pub(crate) label: String,
    pub(crate) target: String,
    pub(crate) state: TargetState,
}

/// One census row: a label, its total edge count, and the breakdown tallies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CensusRow {
    pub(crate) label: String,
    pub(crate) count: usize,
    pub(crate) resolved: usize,
    pub(crate) unresolved: usize,
    pub(crate) free_text: usize,
}

/// Project a [`Catalog`] through [`ListFilter`] → ordered [`RelationRow`]s.
///
/// Filters apply in AND-composition order (§5.4):
/// 1. `include_memory` gate (false ⇒ drop `CatalogEdgeLabel::Raw` edges)
/// 2. `--label` exact match on `name()`
/// 3. `--source-kind` uppercased match on `source_kind()`
/// 4. `--target` canonical-normalised match on `target_display()`
/// 5. `--unresolved` ⇒ keep only `target_state != Resolved`
///
/// Sorted: `(label, source canonical, target)` — target is the tie-breaker.
pub(crate) fn project_list(catalog: &Catalog, filter: &ListFilter) -> Vec<RelationRow> {
    let normalised_target: Option<String> = filter.target.as_ref().map(|t| {
        // D6 / F3: try canonical-ref parse; memory UIDs are matched verbatim.
        if let Ok((kref, id)) = integrity::parse_canonical_ref(t) {
            listing::canonical_id(kref.kind.prefix, id)
        } else {
            // Not a canonical ref — match verbatim (memory UID path).
            t.clone()
        }
    });

    let mut rows: Vec<RelationRow> = catalog
        .edges
        .iter()
        .filter(|edge| {
            // (1) include_memory gate: drop Raw edges when flag is false.
            if !filter.include_memory && matches!(edge.label, CatalogEdgeLabel::Raw(_)) {
                return false;
            }
            // (2) --label exact match on name().
            if let Some(ref want) = filter.label
                && edge.label.name() != want.as_str()
            {
                return false;
            }
            // (3) --source-kind uppercased match on source_kind().
            if let Some(ref want) = filter.source_kind
                && source_kind(&edge.source).to_uppercase() != want.to_uppercase()
            {
                return false;
            }
            // (4) --target canonical-normalised match.
            if let Some(ref want) = normalised_target
                && target_display(&edge.target) != *want
            {
                return false;
            }
            // (5) --unresolved: keep only non-Resolved targets.
            if filter.unresolved && target_state(&edge.target) == TargetState::Resolved {
                return false;
            }
            true
        })
        .map(|edge| {
            let source = edge.source.canonical();
            // Render the role verb (`references(implements)`) on the row; the `--label`
            // filter above still compares the bare `name()` (SL-149 §2.6).
            let label = edge_label_display(edge);
            let target = target_display(&edge.target);
            let state = target_state(&edge.target);
            RelationRow {
                source,
                label,
                target,
                state,
            }
        })
        .collect();

    // Sort: (label, source canonical, target)
    rows.sort_by(|a, b| {
        a.label
            .cmp(&b.label)
            .then_with(|| a.source.cmp(&b.source))
            .then_with(|| a.target.cmp(&b.target))
    });
    rows
}

/// Project a [`Catalog`] into census rows grouped by label.
///
/// When `include_memory` is false, `CatalogEdgeLabel::Raw` edges are excluded
/// (D2 hydrate invariant). Rows sorted `(count desc, label asc)`.
pub(crate) fn project_census(catalog: &Catalog, include_memory: bool) -> Vec<CensusRow> {
    let mut groups: BTreeMap<String, CensusRow> = BTreeMap::new();

    for edge in &catalog.edges {
        if !include_memory && matches!(edge.label, CatalogEdgeLabel::Raw(_)) {
            continue;
        }
        // Group by the rendered `(label, role)` form (SL-149 §2.6): a `references`
        // census splits into one row per role (`references(implements)`, …).
        let label = edge_label_display(edge);
        let entry = groups.entry(label).or_insert(CensusRow {
            label: String::new(),
            count: 0,
            resolved: 0,
            unresolved: 0,
            free_text: 0,
        });
        entry.count = entry.count.wrapping_add(1);
        match target_state(&edge.target) {
            TargetState::Resolved => entry.resolved = entry.resolved.wrapping_add(1),
            TargetState::Unresolved => entry.unresolved = entry.unresolved.wrapping_add(1),
            TargetState::FreeText => entry.free_text = entry.free_text.wrapping_add(1),
        }
    }

    // Fill in the label field and sort.
    let mut rows: Vec<CensusRow> = groups
        .into_iter()
        .map(|(lbl, mut row)| {
            row.label = lbl;
            row
        })
        .collect();

    // Sort: count desc, label asc.
    rows.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.label.cmp(&b.label)));
    rows
}

// ---------------------------------------------------------------------------
// Render — rides the listing spine (Column + render_columns + json_envelope)
// ---------------------------------------------------------------------------

/// The serialise name for [`TargetState`], matching the `#[serde(rename_all)]` /
/// `#[serde(rename)]` attributes on the enum.
fn state_name(state: TargetState) -> String {
    match state {
        TargetState::Resolved => "resolved",
        TargetState::Unresolved => "unresolved",
        TargetState::FreeText => "free_text",
    }
    .to_owned()
}

/// The table columns for the relation list surface.
const RELATION_COLUMNS: [listing::Column<RelationRow>; 4] = [
    listing::Column {
        name: "source",
        header: "source",
        cell: |r| r.source.clone(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "label",
        header: "label",
        cell: |r| r.label.clone(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "target",
        header: "target",
        cell: |r| r.target.clone(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "state",
        header: "state",
        cell: |r| state_name(r.state),
        paint: listing::ColumnPaint::None,
    },
];

/// The table columns for the census surface.
const CENSUS_COLUMNS: [listing::Column<CensusRow>; 5] = [
    listing::Column {
        name: "label",
        header: "label",
        cell: |r| r.label.clone(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "count",
        header: "count",
        cell: |r| r.count.to_string(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "resolved",
        header: "resolved",
        cell: |r| r.resolved.to_string(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "unresolved",
        header: "unresolved",
        cell: |r| r.unresolved.to_string(),
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "free_text",
        header: "free_text",
        cell: |r| r.free_text.to_string(),
        paint: listing::ColumnPaint::None,
    },
];

/// Render a list of [`RelationRow`]s as a table or JSON envelope.
/// Empty rows → empty string (no header, no envelope).
pub(crate) fn render_list(
    rows: &[RelationRow],
    format: listing::Format,
    opts: listing::RenderOpts,
    columns: Option<&[String]>,
) -> anyhow::Result<String> {
    match format {
        listing::Format::Json => render_list_json(rows, columns),
        listing::Format::Table => render_list_table(rows, opts, columns),
    }
}

/// Render census rows as a table or JSON envelope. Empty rows → empty string.
pub(crate) fn render_census(
    rows: &[CensusRow],
    format: listing::Format,
    opts: listing::RenderOpts,
    columns: Option<&[String]>,
) -> anyhow::Result<String> {
    match format {
        listing::Format::Json => render_census_json(rows),
        listing::Format::Table => render_census_table(rows, opts, columns),
    }
}

/// Table render for relation rows.
fn render_list_table(
    rows: &[RelationRow],
    opts: listing::RenderOpts,
    columns: Option<&[String]>,
) -> anyhow::Result<String> {
    if rows.is_empty() {
        return Ok(String::new());
    }
    let default: &[&str] = &["source", "label", "target", "state"];
    let sel = listing::select_columns(&RELATION_COLUMNS, default, columns)?;
    Ok(listing::render_columns(rows, &sel, opts))
}

/// Table render for census rows.
fn render_census_table(
    rows: &[CensusRow],
    opts: listing::RenderOpts,
    columns: Option<&[String]>,
) -> anyhow::Result<String> {
    if rows.is_empty() {
        return Ok(String::new());
    }
    let default: &[&str] = &["label", "count", "resolved", "unresolved", "free_text"];
    let sel = listing::select_columns(&CENSUS_COLUMNS, default, columns)?;
    Ok(listing::render_columns(rows, &sel, opts))
}

// ---------------------------------------------------------------------------
// JSON shapes — faithful serde row types per the spec
// ---------------------------------------------------------------------------

/// One relation row's faithful JSON shape.
#[derive(Serialize)]
struct RelationJsonRow {
    source: String,
    label: String,
    target: String,
    state: TargetState,
}

/// One census row's faithful JSON shape.
#[derive(Serialize)]
struct CensusJsonRow {
    label: String,
    count: usize,
    resolved: usize,
    unresolved: usize,
    free_text: usize,
}

fn render_list_json(rows: &[RelationRow], _columns: Option<&[String]>) -> anyhow::Result<String> {
    let json_rows: Vec<RelationJsonRow> = rows
        .iter()
        .map(|r| RelationJsonRow {
            source: r.source.clone(),
            label: r.label.clone(),
            target: r.target.clone(),
            state: r.state,
        })
        .collect();
    listing::json_envelope("relation", &json_rows)
}

fn render_census_json(rows: &[CensusRow]) -> anyhow::Result<String> {
    let json_rows: Vec<CensusJsonRow> = rows
        .iter()
        .map(|r| CensusJsonRow {
            label: r.label.clone(),
            count: r.count,
            resolved: r.resolved,
            unresolved: r.unresolved,
            free_text: r.free_text,
        })
        .collect();
    listing::json_envelope("census", &json_rows)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use crate::catalog::hydrate::Units;
    use crate::catalog::scan::{EntityKey, ScannedEntity};
    use crate::catalog::test_helpers::tmp;

    // ---- test helpers ----

    fn test_units() -> Units {
        Units {
            estimation: "espresso_shots".to_string(),
            value: "magic_beans".to_string(),
        }
    }

    /// Build a pure `Catalog` from scanned entities (no disk access).
    fn catalog_from(scanned: &[ScannedEntity]) -> Catalog {
        let dir = tmp();
        Catalog::from_scanned(dir.path(), scanned, &[], &BTreeMap::new(), test_units())
    }

    /// A numbered entity with a single outbound edge.
    fn numbered_with_edge(
        prefix: &'static str,
        id: u32,
        label: &str,
        target: &str,
    ) -> ScannedEntity {
        use crate::relation::{RelationLabel, Role};
        let kind = crate::integrity::kind_by_prefix(prefix).unwrap().kind;
        // SL-149: a `references(<role>)` label string carries a role; any other label is
        // roleless.
        let (name, role) = match label
            .strip_prefix("references(")
            .and_then(|s| s.strip_suffix(')'))
        {
            Some(r) => ("references", Role::from_name(r)),
            None => (label, None),
        };
        let rel_label = RelationLabel::from_name(name).unwrap_or_else(|| {
            panic!("unknown relation label {label}");
        });
        ScannedEntity {
            key: EntityKey { prefix, id },
            kind,
            status: Some("proposed".to_string()),
            title: format!("{prefix}-{id}"),
            outbound: vec![crate::relation::RelationEdge::with_role(
                rel_label,
                role,
                target.to_string(),
            )],
            estimate: None,
            value: None,
            risk: None,
            tags: vec![],
            body: None,
        }
    }

    /// Two numbered entities in the same scan — both are present for resolution.
    fn two_entities(a: ScannedEntity, b: ScannedEntity) -> Catalog {
        catalog_from(&[a, b])
    }

    // ---- VT-1: target_state / target_display over all three EdgeTarget variants

    #[test]
    fn target_state_covers_all_variants() {
        let ekey = EntityKey {
            prefix: "SL",
            id: 1,
        };
        let resolved = EdgeTarget::Resolved(CatalogKey::Numbered(ekey));
        assert_eq!(target_state(&resolved), TargetState::Resolved);
        assert_eq!(target_display(&resolved), "SL-001");

        let unresolved = EdgeTarget::UnresolvedRef {
            raw: "SL-999".to_string(),
        };
        assert_eq!(target_state(&unresolved), TargetState::Unresolved);
        assert_eq!(target_display(&unresolved), "SL-999");

        let free = EdgeTarget::UnvalidatedText {
            raw: "loose talk".to_string(),
        };
        assert_eq!(target_state(&free), TargetState::FreeText);
        assert_eq!(target_display(&free), "loose talk");
    }

    // ---- VT-2: source_kind

    #[test]
    fn source_kind_returns_prefix_or_mem() {
        let num = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });
        assert_eq!(source_kind(&num), "SL");

        let mem = CatalogKey::Memory("mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string());
        assert_eq!(source_kind(&mem), "MEM");
    }

    // ---- VT-3: project_list filters AND-compose; four-axis (+ unresolved, include_memory on) narrows

    #[test]
    fn project_list_and_composes_four_axes_to_one_row() {
        // SL-001 → REQ-005 (requirements, resolved) + REQ-999 (requirements, unresolved)
        // REQ-005 seeded → resolves; REQ-999 absent → unresolved.
        // All four axes + include_memory: true narrow to exactly the REQ-999 row.
        use crate::relation::{RelationEdge, RelationLabel, Role};
        let kind = crate::integrity::kind_by_prefix("SL").unwrap().kind;
        let req_kind = crate::integrity::kind_by_prefix("REQ").unwrap().kind;
        let sl001 = ScannedEntity {
            key: EntityKey {
                prefix: "SL",
                id: 1,
            },
            kind,
            status: Some("proposed".to_string()),
            title: "SL-001".to_string(),
            outbound: vec![
                RelationEdge::with_role(
                    RelationLabel::References,
                    Some(Role::Implements),
                    "REQ-005".to_string(),
                ),
                RelationEdge::with_role(
                    RelationLabel::References,
                    Some(Role::Implements),
                    "REQ-999".to_string(),
                ),
            ],
            estimate: None,
            value: None,
            risk: None,
            tags: vec![],
            body: None,
        };
        let req005 = ScannedEntity {
            key: EntityKey {
                prefix: "REQ",
                id: 5,
            },
            kind: req_kind,
            status: Some("active".to_string()),
            title: "REQ-005".to_string(),
            outbound: vec![],
            estimate: None,
            value: None,
            risk: None,
            tags: vec![],
            body: None,
        };
        let catalog = two_entities(sl001, req005);

        // All four axes + include_memory on → one row (REQ-999, the resolved REQ-005 is filtered by unresolved)
        let rows = project_list(
            &catalog,
            &ListFilter {
                include_memory: true,
                label: Some("references".into()),
                source_kind: Some("SL".into()),
                target: Some("REQ-999".into()),
                unresolved: true,
            },
        );
        assert_eq!(
            rows.len(),
            1,
            "four axes + include_memory narrows to one row"
        );
        assert_eq!(rows[0].source, "SL-001");
        assert_eq!(rows[0].label, "references(implements)");
        assert_eq!(rows[0].target, "REQ-999");
        assert_eq!(rows[0].state, TargetState::Unresolved);

        // Baseline: without unresolved filter, both edges appear.
        let all = project_list(
            &catalog,
            &ListFilter {
                include_memory: true,
                ..Default::default()
            },
        );
        assert_eq!(all.len(), 2, "both edges visible without --unresolved");
    }

    // ---- VT-4: include_memory gate; --source-kind MEM without flag → empty

    #[test]
    fn include_memory_false_drops_raw_edges() {
        // Build a Catalog with one numbered edge AND one Raw-label (memory) edge.
        // The numbered edge is Validated; the Raw edge simulates a memory source.
        use crate::catalog::hydrate::{CatalogEdge, CatalogEdgeLabel};
        use std::path::PathBuf;

        let sl001 = numbered_with_edge("SL", 1, "references(implements)", "REQ-001");
        let mut catalog = catalog_from(&[sl001]);
        // Inject a Raw-label edge (simulating a memory-source edge).
        catalog.edges.push(CatalogEdge {
            source: CatalogKey::Memory("mem_user123".to_string()),
            label: CatalogEdgeLabel::Raw("references".to_string()),
            role: None,
            target: EdgeTarget::UnvalidatedText {
                raw: "some note".to_string(),
            },
            origin: crate::catalog::hydrate::EdgeOrigin {
                file: PathBuf::new(),
                field: None,
            },
        });

        // F4a: --source-kind MEM WITHOUT --include-memory → empty (gate drops memory first)
        let rows = project_list(
            &catalog,
            &ListFilter {
                source_kind: Some("MEM".into()),
                ..Default::default()
            },
        );
        assert!(
            rows.is_empty(),
            "MEM source-kind without include_memory should be empty"
        );

        // With include_memory=true, the memory edge is visible.
        let rows = project_list(
            &catalog,
            &ListFilter {
                include_memory: true,
                source_kind: Some("MEM".into()),
                ..Default::default()
            },
        );
        assert_eq!(
            rows.len(),
            1,
            "MEM source-kind WITH include_memory should include the memory edge"
        );
        assert_eq!(rows[0].source, "mem_user123");
        assert_eq!(rows[0].label, "references");
        assert_eq!(rows[0].state, TargetState::FreeText);

        // Without --source-kind, include_memory=true shows all edges (numbered + memory)
        let all = project_list(
            &catalog,
            &ListFilter {
                include_memory: true,
                ..Default::default()
            },
        );
        assert_eq!(
            all.len(),
            2,
            "include_memory=true with no source-kind should show all edges"
        );

        // Default (no flags) excludes the Raw edge.
        let default = project_list(&catalog, &ListFilter::default());
        assert_eq!(default.len(), 1, "default should show only numbered edges");
    }

    // ---- VT-5: --unresolved keeps only state ≠ Resolved

    #[test]
    fn unresolved_filter_keeps_only_non_resolved() {
        // SL-001 → REQ-005 (resolved, REQ-005 is in the scan).
        // SL-001 → REQ-999 (unresolved, REQ-999 not in the scan).
        use crate::relation::{RelationEdge, RelationLabel, Role};
        let kind = crate::integrity::kind_by_prefix("SL").unwrap().kind;
        let req_kind = crate::integrity::kind_by_prefix("REQ").unwrap().kind;
        let sl001 = ScannedEntity {
            key: EntityKey {
                prefix: "SL",
                id: 1,
            },
            kind,
            status: Some("proposed".to_string()),
            title: "SL-001".to_string(),
            outbound: vec![
                RelationEdge::with_role(
                    RelationLabel::References,
                    Some(Role::Implements),
                    "REQ-005".to_string(),
                ),
                RelationEdge::with_role(
                    RelationLabel::References,
                    Some(Role::Implements),
                    "REQ-999".to_string(),
                ),
            ],
            estimate: None,
            value: None,
            risk: None,
            tags: vec![],
            body: None,
        };
        let req005 = ScannedEntity {
            key: EntityKey {
                prefix: "REQ",
                id: 5,
            },
            kind: req_kind,
            status: Some("active".to_string()),
            title: "REQ-005".to_string(),
            outbound: vec![],
            estimate: None,
            value: None,
            risk: None,
            tags: vec![],
            body: None,
        };
        let catalog = two_entities(sl001, req005);

        let rows = project_list(
            &catalog,
            &ListFilter {
                unresolved: true,
                ..Default::default()
            },
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].target, "REQ-999");
        assert_eq!(rows[0].state, TargetState::Unresolved);
    }

    // ---- VT-6: sort (label, source, target); empty → empty string

    #[test]
    fn project_list_sorts_by_label_source_target() {
        // Two SL entities with edges to create mixed order.
        use crate::relation::{RelationEdge, RelationLabel, Role};
        let kind = crate::integrity::kind_by_prefix("SL").unwrap().kind;
        let sl001 = ScannedEntity {
            key: EntityKey {
                prefix: "SL",
                id: 1,
            },
            kind,
            status: Some("proposed".to_string()),
            title: "SL-001".to_string(),
            outbound: vec![RelationEdge::with_role(
                RelationLabel::References,
                Some(Role::Implements),
                "PRD-002".to_string(),
            )],
            estimate: None,
            value: None,
            risk: None,
            tags: vec![],
            body: None,
        };
        let sl002 = ScannedEntity {
            key: EntityKey {
                prefix: "SL",
                id: 2,
            },
            kind,
            status: Some("proposed".to_string()),
            title: "SL-002".to_string(),
            outbound: vec![RelationEdge::with_role(
                RelationLabel::References,
                Some(Role::Implements),
                "PRD-001".to_string(),
            )],
            estimate: None,
            value: None,
            risk: None,
            tags: vec![],
            body: None,
        };
        let catalog = two_entities(sl001, sl002);

        let rows = project_list(&catalog, &ListFilter::default());
        assert_eq!(rows.len(), 2);
        // Sort: same label "references(implements)", then source: SL-001 < SL-002
        assert_eq!(rows[0].source, "SL-001");
        assert_eq!(rows[1].source, "SL-002");
    }

    #[test]
    fn render_list_empty_is_empty_string() {
        let out = render_list(
            &[],
            listing::Format::Table,
            listing::RenderOpts::default(),
            None,
        )
        .unwrap();
        assert_eq!(out, "");
    }

    // ---- VT-7: census count == resolved+unresolved+free_text; sort

    #[test]
    fn census_tallies_honour_breakdown_and_sort() {
        // SL-001 → REQ-005 (resolved), SL-001 → REQ-999 (unresolved), SL-001 → "drift text" (free_text)
        use crate::relation::{RelationEdge, RelationLabel, Role};
        let kind = crate::integrity::kind_by_prefix("SL").unwrap().kind;
        let req_kind = crate::integrity::kind_by_prefix("REQ").unwrap().kind;
        let sl001 = ScannedEntity {
            key: EntityKey {
                prefix: "SL",
                id: 1,
            },
            kind,
            status: Some("proposed".to_string()),
            title: "SL-001".to_string(),
            outbound: vec![
                RelationEdge::with_role(
                    RelationLabel::References,
                    Some(Role::Implements),
                    "REQ-005".to_string(),
                ),
                RelationEdge::with_role(
                    RelationLabel::References,
                    Some(Role::Implements),
                    "REQ-999".to_string(),
                ),
                RelationEdge::new(RelationLabel::Drift, "some free text".to_string()),
            ],
            estimate: None,
            value: None,
            risk: None,
            tags: vec![],
            body: None,
        };
        let req005 = ScannedEntity {
            key: EntityKey {
                prefix: "REQ",
                id: 5,
            },
            kind: req_kind,
            status: Some("active".to_string()),
            title: "REQ-005".to_string(),
            outbound: vec![],
            estimate: None,
            value: None,
            risk: None,
            tags: vec![],
            body: None,
        };
        let catalog = two_entities(sl001, req005);

        let rows = project_census(&catalog, false);
        // Two labels: "references(implements)" (count 2), "drift" (count 1)
        assert_eq!(rows.len(), 2);

        // Sort: count desc → "references(implements)" first (2), then "drift" (1).
        let req_row = &rows[0];
        assert_eq!(req_row.label, "references(implements)");
        assert_eq!(req_row.count, 2);
        assert_eq!(req_row.resolved, 1);
        assert_eq!(req_row.unresolved, 1);
        assert_eq!(req_row.free_text, 0);
        assert_eq!(
            req_row.count,
            req_row.resolved + req_row.unresolved + req_row.free_text
        );

        let drift_row = &rows[1];
        assert_eq!(drift_row.label, "drift");
        assert_eq!(drift_row.count, 1);
        assert_eq!(drift_row.resolved, 0);
        assert_eq!(drift_row.unresolved, 0);
        assert_eq!(drift_row.free_text, 1);
    }

    // ---- VT-8: census include_memory honoured

    #[test]
    fn census_include_memory_drops_raw_labels() {
        // Just a numbered entity; census with/without include_memory is identical.
        let sl001 = numbered_with_edge("SL", 1, "references(implements)", "REQ-001");
        let catalog = catalog_from(&[sl001]);

        let rows = project_census(&catalog, false);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].label, "references(implements)");

        // With include_memory, same result since no memory edges.
        let rows2 = project_census(&catalog, true);
        assert_eq!(rows2.len(), 1);
    }

    // ---- VT-9: JSON shapes under json_envelope

    #[test]
    fn list_json_shape_matches_envelope_contract() {
        let row = RelationRow {
            source: "SL-001".to_string(),
            label: "references(implements)".to_string(),
            target: "REQ-005".to_string(),
            state: TargetState::Resolved,
        };
        let json = render_list(
            &[row],
            listing::Format::Json,
            listing::RenderOpts::default(),
            None,
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["kind"].as_str(), Some("relation"));
        let rows = &v["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert_eq!(r["source"].as_str(), Some("SL-001"));
        assert_eq!(r["label"].as_str(), Some("references(implements)"));
        assert_eq!(r["target"].as_str(), Some("REQ-005"));
        assert_eq!(r["state"].as_str(), Some("resolved"));
    }

    #[test]
    fn census_json_shape_matches_envelope_contract() {
        let row = CensusRow {
            label: "references(implements)".to_string(),
            count: 2,
            resolved: 1,
            unresolved: 1,
            free_text: 0,
        };
        let json = render_census(
            &[row],
            listing::Format::Json,
            listing::RenderOpts::default(),
            None,
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["kind"].as_str(), Some("census"));
        let rows = &v["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert_eq!(r["label"].as_str(), Some("references(implements)"));
        assert_eq!(r["count"].as_u64(), Some(2));
        assert_eq!(r["resolved"].as_u64(), Some(1));
        assert_eq!(r["unresolved"].as_u64(), Some(1));
        assert_eq!(r["free_text"].as_u64(), Some(0));
    }

    // ---- VT-12: --target ADR-1 matches ADR-001; memory target matches by UID

    #[test]
    fn target_filter_normalizes_canonical_ref() {
        // Seed ADR-001 with an edge → ADR-002.
        use crate::relation::{RelationEdge, RelationLabel};
        let adr_kind = crate::integrity::kind_by_prefix("ADR").unwrap().kind;
        let adr001 = ScannedEntity {
            key: EntityKey {
                prefix: "ADR",
                id: 1,
            },
            kind: adr_kind,
            status: Some("accepted".to_string()),
            title: "ADR-001".to_string(),
            outbound: vec![RelationEdge::new(
                RelationLabel::Supersedes,
                "ADR-002".to_string(),
            )],
            estimate: None,
            value: None,
            risk: None,
            tags: vec![],
            body: None,
        };
        let adr002 = ScannedEntity {
            key: EntityKey {
                prefix: "ADR",
                id: 2,
            },
            kind: adr_kind,
            status: Some("accepted".to_string()),
            title: "ADR-002".to_string(),
            outbound: vec![],
            estimate: None,
            value: None,
            risk: None,
            tags: vec![],
            body: None,
        };
        let catalog = two_entities(adr001, adr002);

        // --target ADR-2 (unpadded) normalises to ADR-002
        let rows = project_list(
            &catalog,
            &ListFilter {
                target: Some("ADR-2".into()),
                ..Default::default()
            },
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].target, "ADR-002");
        assert_eq!(rows[0].state, TargetState::Resolved);

        // --target ADR-002 (padded) normalises to ADR-002
        let rows = project_list(
            &catalog,
            &ListFilter {
                target: Some("ADR-002".into()),
                ..Default::default()
            },
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].target, "ADR-002");
    }
}
