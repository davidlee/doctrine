// SPDX-License-Identifier: GPL-3.0-only
//! The cross-kind relation graph engine (design §5.1/§5.2).
//!
//! Sits at the engine layer (ADR-001): it imports the relation vocabulary leaf
//! ([`crate::relation`]) and every edge-authoring kind module, dispatching a
//! data-driven [`outbound_for`] over `integrity::KINDS` — kind is *data*, not a
//! trait (`mem.pattern.entity.kind-is-data-not-trait`). No kind module imports
//! back, so there is no cycle (the whole reason the vocabulary lives in the leaf).
//!
//! PHASE-02 landed the outbound extraction dispatch ([`outbound_for`]). PHASE-03
//! extended this file with the all-kind scan ([`build_relation_graph`]), the
//! `Projection<EntityKey>`, the reference overlays, and the [`inspect`] query
//! (design §5.4). PHASE-04 wires the `inspect <ID>` CLI command ([`run`]) — the
//! render + `--json` surface — so the scan and `inspect` are now live (the
//! PHASE-03 `not(test)` `dead_code` expect retired itself here, as designed).

use std::collections::BTreeMap;
use std::path::Path;

use cordage::{
    Arity, CyclePolicy, Direction, EdgeAttrs, Graph, GraphBuilder, OverlayConfig, OverlayId,
};

use crate::catalog::hydrate::{CatalogEdgeLabel, CatalogKey, EdgeTarget};
use crate::dep_seq;
use crate::entity;
use crate::integrity;
use crate::listing::{self, Format};
use crate::projection::Projection;
use crate::relation::{RELATION_RULES, RelationEdge, RelationLabel, Role, TargetSpec};

// Re-exports from catalog::scan — the single source of truth (SL-071 D7).
// Aliases, not wrappers — one body, one source.
use crate::catalog::scan::ScanMode;
pub(crate) use crate::catalog::scan::{EntityKey, ScannedEntity, outbound_for, scan_entities};

/// One entity's `needs`/`after` dep/seq edges plus its `promoted` flag, dispatched to
/// the owning kind's reader by canonical prefix — the kind-agnostic READ gate that lets
/// slice (and any future authoring kind) dep/seq edges reach the priority blocker/next
/// view (design D7/§5.2/§5.4). The shape mirrors [`outbound_for`]: one data-driven match
/// over the corpus-wide `kind.prefix` discriminant, each arm reading only its own kind's
/// dep/seq via that kind's existing path — never a second parse.
///
/// - **backlog** — routed through backlog's single `dep_seq_for` reader (ONE parse: it
///   already reads `resolution` for `promoted`), its `Vec<(String, i32)>` `after`
///   adapted into the leaf [`dep_seq::AfterEdge`] shape (design F3).
/// - **slice** — the leaf [`dep_seq::read`] over the slice's own toml; `promoted` is
///   always `false` (only a backlog item carries the typed promoted projection).
/// - **every non-authoring kind** — SHORT-CIRCUITS to an empty [`dep_seq::DepSeq`] with
///   `false`, BEFORE any path construction or disk touch (design F5). This is
///   load-bearing: the priority read loop now visits ALL kinds, not just the five
///   backlog ones, so a non-authoring kind must contribute zero edges with NO read.
pub(crate) fn dep_seq_for(
    root: &Path,
    kind: &entity::Kind,
    id: u32,
) -> anyhow::Result<(dep_seq::DepSeq, bool)> {
    match kind.prefix {
        // Slice authors dep/seq directly (PHASE-03); the leaf reads its own toml. Stem
        // is `"slice"` (the same id-path shape `integrity::KINDS` carries for SL).
        "SL" => {
            let name = format!("{id:03}");
            let path = root
                .join(kind.dir)
                .join(&name)
                .join(format!("slice-{name}.toml"));
            Ok((dep_seq::read(&path)?, false))
        }
        // REV (SL-066, G2) — mirrors the SL arm: a Revision authors its own
        // `needs`/`after` (the IDE-010 payoff — a REV may `needs` a spike), so the
        // leaf reads its `revision-NNN.toml` directly. Without this arm REV-as-source
        // edges short-circuit to the empty fallthrough and never reach the blocker/
        // `next` view. `promoted` is always `false` (a backlog-only projection).
        "REV" => {
            let name = format!("{id:03}");
            let path = root
                .join(kind.dir)
                .join(&name)
                .join(format!("revision-{name}.toml"));
            Ok((dep_seq::read(&path)?, false))
        }
        // The five backlog kinds route to backlog's own one-parse reader, which carries
        // the `promoted` projection. Adapt its `(to, rank)` pairs to the leaf AfterEdge.
        other => {
            if let Some(item_kind) = crate::backlog::kind_from_prefix(other) {
                let bl = crate::backlog::dep_seq_for(root, item_kind, id)?;
                let after = bl
                    .after
                    .into_iter()
                    .map(|(to, rank)| dep_seq::AfterEdge { to, rank })
                    .collect();
                Ok((
                    dep_seq::DepSeq {
                        needs: bl.needs,
                        after,
                    },
                    bl.promoted,
                ))
            } else {
                // Every non-authoring kind: zero dep/seq, no disk read. The kind is
                // tested HERE — before any path is built or any toml is touched (F5).
                Ok((dep_seq::DepSeq::default(), false))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// PHASE-03 — the all-kind scan, the reference overlays, and the inspect query.
// ---------------------------------------------------------------------------

/// The single existence gate for the keyed read surfaces (SL-050 F6). A well-formed
/// ref to a never-minted id (e.g. `SL-999`) is indistinguishable from a real isolated
/// node at the render layer — this turns it into a clean error instead. The oracle is
/// the `Projection<EntityKey>` each keyed surface already holds: it contains EXACTLY
/// the minted keys, so `resolve(key).is_none()` ⇔ the entity was never minted (no
/// entity dir). One helper, one message, every keyed surface (`inspect`/`render`/
/// `explain`/`blockers`/`actionability_block`) routes through it — no second
/// existence path to drift.
///
/// # Errors
///
/// `"{KIND-NNN}: no such entity"` when `key` is absent from `projection`.
pub(crate) fn require_minted(
    projection: &Projection<EntityKey>,
    key: EntityKey,
) -> anyhow::Result<()> {
    if projection.resolve(key).is_none() {
        anyhow::bail!("{}: no such entity", key.canonical());
    }
    Ok(())
}

/// The overlay-identity map: one cordage overlay per OVERLAY-BACKED relation label,
/// keyed both ways. The overlay-backed set is *derived* from [`RELATION_RULES`]
/// (R2-M4) — every distinct label whose `TargetSpec != Unvalidated`. The two
/// target-unvalidated labels — `Drift` and `DecisionRef` (ADR-010 Decision 2) — get
/// NO overlay (their targets never resolve to a node), so `overlay_for` returns
/// `None` for them and their edges always dangle.
///
/// Label is overlay identity (OQ2-B): the same label authored from different source
/// kinds (e.g. `Supersedes` from both slice and governance, `GovernedBy` from SL·PRD·
/// SPEC) shares ONE overlay — the iteration de-dupes on the label key.
struct OverlayMap {
    by_label: BTreeMap<RelationLabel, OverlayId>,
    by_overlay: BTreeMap<OverlayId, RelationLabel>,
}

impl OverlayMap {
    /// Allocate one `Reject`/`Unbounded` overlay per overlay-backed label (I1:
    /// `Reject` removes no edges, `Unbounded` exempts arity eviction — `in_edges`
    /// then enumerates exactly the authored unique inbound set).
    ///
    /// Table-derived (R2-M4): iterate [`RELATION_RULES`] and allocate one overlay per
    /// DISTINCT label whose `TargetSpec != Unvalidated`. The `by_label` `BTreeMap`
    /// de-dupes a label that appears in several rows (e.g. `Supersedes` from SL and
    /// gov) to one overlay. NO hardcoded label const — the table is the single source,
    /// so a new resolvable label gets an overlay automatically (VT-1 pins the set ==
    /// the resolvable graph labels). Behaviour-preserving: the corpus authors no
    /// `governed_by`/`consumes` edges yet, so the allocated set grows by those two
    /// labels but produces byte-identical `inspect` / `*-show` output (EX-2).
    fn build(builder: &mut GraphBuilder) -> Self {
        let mut by_label = BTreeMap::new();
        let mut by_overlay = BTreeMap::new();
        for rule in RELATION_RULES {
            if matches!(rule.target, TargetSpec::Unvalidated) {
                continue;
            }
            // De-dupe: a label spanning several source rows shares ONE overlay.
            if by_label.contains_key(&rule.label) {
                continue;
            }
            let ov = builder.overlay(OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded));
            by_label.insert(rule.label, ov);
            by_overlay.insert(ov, rule.label);
        }
        Self {
            by_label,
            by_overlay,
        }
    }

    /// The overlay backing `label`, or `None` for the target-unvalidated labels
    /// (`Drift`/`DecisionRef`) that carry no overlay. `label_of` is unneeded —
    /// `inspect` iterates `by_overlay` directly (overlay → label), so the reverse
    /// map is read as a field, not through an accessor.
    fn overlay_for(&self, label: RelationLabel) -> Option<OverlayId> {
        self.by_label.get(&label).copied()
    }
}

/// The assembled relation graph: the cordage `Graph`, the `EntityKey ↔ NodeId`
/// projection, the overlay-identity map, and the per-source danglers collected
/// during the edge pass. `inspect` reads inbound from the graph, outbound fresh
/// from `outbound_for`, and returns only the queried entity's danglers.
struct RelationGraph {
    graph: Graph,
    projection: Projection<EntityKey>,
    overlays: OverlayMap,
    /// Danglers keyed by source entity — the unresolved / free-text / no-overlay
    /// outbound targets, so `inspect` returns only the queried entity's set.
    danglers: BTreeMap<EntityKey, Vec<(RelationLabel, String)>>,
}

/// Build the cross-kind reference-overlay graph from a PRE-SCANNED entity slice (the
/// SL-050 F2 shared-scan seam — a SEPARATE cordage `Graph` from `backlog_order`/
/// `priority`: they share the `Projection` *type*, never a graph instance or a scan).
/// Takes only the slice — it touches no disk beyond the scan it is handed (the F2
/// seam: the single corpus walk lives at the command layer). The mint/edge order is the
/// scan order the caller supplies (KINDS table / id ascending), so the mint order — and
/// thus the byte-identical `inspect` output (VT-4) — is preserved exactly.
///
/// 1. Mint nodes: one `intern` per scanned entity, in scan order.
/// 2. Emit edges: per minted entity, per outbound edge, parse + resolve the target; a
///    resolvable target whose label has an overlay ⇒ `builder.edge`,
///    `EdgeAttrs::new(0, 0)` (C3 — two authored rows with the same `(label,src,dst)`
///    collapse to one in cordage's `BTreeSet<Edge>`); anything else (unresolved,
///    parse-error / free-text, or a no-overlay label like `Drift`/`DecisionRef`,
///    INCLUDING a resolvable target under a no-overlay label) ⇒ a dangler.
/// 3. `builder.build()` — NO `OrderSpec` over reference overlays (I2: direct-only,
///    composition-free; no union-cycle pass touches them).
fn build_relation_graph_from(scanned: &[ScannedEntity]) -> anyhow::Result<RelationGraph> {
    let mut builder = GraphBuilder::new();
    let overlays = OverlayMap::build(&mut builder);
    let mut projection: Projection<EntityKey> = Projection::new();

    // Pass 1 — mint every entity's node (scan order: KINDS table, ids ascending).
    for entity in scanned {
        projection.intern(&mut builder, entity.key);
    }

    // Pass 2 — emit edges (resolve only, never intern) and collect danglers.
    let mut danglers: BTreeMap<EntityKey, Vec<(RelationLabel, String)>> = BTreeMap::new();
    for entity in scanned {
        // Present by construction (pass 1 interned every key from the same scan);
        // loud in debug if that ever desyncs, a benign skip in release (the path
        // stays panic-free).
        let Some(src) = projection.resolve(entity.key) else {
            debug_assert!(
                false,
                "build_relation_graph: pass-2 key not interned in pass 1"
            );
            continue;
        };
        for edge in &entity.outbound {
            if let Some(dst) = resolve_target(&projection, edge)
                && let Some(ov) = overlays.overlay_for(edge.label)
            {
                builder.edge(ov, src, dst, EdgeAttrs::new(0, 0));
            } else {
                danglers
                    .entry(entity.key)
                    .or_default()
                    .push((edge.label, edge.target.clone()));
            }
        }
    }

    let graph = builder.build().map_err(|e| {
        anyhow::anyhow!(
            "relation_graph: cordage rejected well-formed adapter input (internal bug): {e:?}"
        )
    })?;

    Ok(RelationGraph {
        graph,
        projection,
        overlays,
        danglers,
    })
}

/// Build the inbound-role index (SL-149 F1): `(source, label, target) -> role`, the
/// role payload an inbound render recovers from the SOURCE entity's outbound edges. The
/// cordage overlay is label-keyed (R5), so the role is NOT carried on the graph edge —
/// `inspect_from` re-keys inbound by `(label, role)` by reading this index, derived once
/// from the same scan the graph is built from.
///
/// Keyed by the parsed target `EntityKey` (a free-text / unparseable target is skipped —
/// it dangles, never an inbound edge). For a given `(source, label, target)` the role is
/// well-defined: edge identity is the `(label, role, target)` triple, and a label has a
/// single resolved-target reciprocal here; the LAST authored row wins on the (rare,
/// hand-edited) duplicate, which the render then groups deterministically.
fn inbound_role_index(
    scanned: &[ScannedEntity],
) -> BTreeMap<(EntityKey, RelationLabel, EntityKey), Option<Role>> {
    let mut index = BTreeMap::new();
    for entity in scanned {
        for edge in &entity.outbound {
            let Ok((kref, tid)) = integrity::parse_canonical_ref(&edge.target) else {
                continue;
            };
            let target = EntityKey {
                prefix: kref.kind.prefix,
                id: tid,
            };
            index.insert((entity.key, edge.label, target), edge.role);
        }
    }
    index
}

/// Resolve an authored edge's `target` to a minted node, or `None`. A target that
/// fails to parse as a canonical ref (free-text — `Drift`/`DecisionRef`), or parses
/// to an id that was never minted (no entity dir), resolves to `None` → a dangler.
fn resolve_target(
    projection: &Projection<EntityKey>,
    edge: &RelationEdge,
) -> Option<cordage::NodeId> {
    let (kref, tid) = integrity::parse_canonical_ref(&edge.target).ok()?;
    projection.resolve(EntityKey {
        prefix: kref.kind.prefix,
        id: tid,
    })
}

// ---------------------------------------------------------------------------
// PHASE-05 — the corpus-edge `validate` walk + supersession cross-check (design
// §5.5, R2-M5 / R2-m2). Report-only: returns finding strings, NEVER rewrites (the
// reseat precedent). Consumed by `integrity::run_validate`.
// ---------------------------------------------------------------------------

/// The `validate` relation-edge walk (design §5.5, R2-M5): scan every entity's
/// authored `[[relation]]` block and report two finding classes — never rewriting:
///
/// 1. **Danglers** — a validated (`Kinds`/`SameKind`/`AnyNumbered`) target that no
///    longer resolves to an entity (a deleted target). `Unvalidated` labels
///    (`drift`/`decision_ref`) are EXCLUDED: their free-text targets dangle BY DESIGN
///    (ADR-010 D2), so they are not findings.
/// 2. **`IllegalRows`** — hand-edited `[[relation]]` rows whose `(source, label)` is
///    off-table (an unknown label, or a label illegal for that source). `read_block`
///    surfaces these; `outbound_for`/`tier1_edges` drop them, so the raw block is
///    re-read here. A mis-ordered hand-edited typed table is caught by the WRITE
///    seam's F1 defence (`append_edge`), not this read walk — this walk reports the
///    row-legality findings the read seam yields.
///
/// Rides the established seams: `scan_entities` for the outbound edges (already legal,
/// resolved-or-not) and `integrity::ensure_ref_resolves` as the dangler oracle (parse +
/// dir-probe — the same existence check `link` uses forward). The raw block re-read for
/// `IllegalRows` uses `integrity::KINDS` (dir + stem) — no new path authority.
pub(crate) fn validate_relations(root: &Path) -> anyhow::Result<Vec<String>> {
    let mut findings = Vec::new();

    // (1) danglers — consume Catalog.edges for target-resolution failures
    // (SL-071 PHASE-05). Catalog classifies every edge target via
    // `parse_canonical_ref`; UnresolvedRef means the target parsed as a
    // canonical ref but the entity was absent from the scan.
    let catalog = crate::catalog::hydrate::scan_catalog(root, ScanMode::default())?;
    // Index entity keys → Kind for label-validation lookups.
    let entity_kinds: BTreeMap<EntityKey, &'static entity::Kind> = catalog
        .entities
        .iter()
        .filter_map(|e| {
            if let CatalogKey::Numbered(key) = &e.key {
                e.kind.map(|k| (*key, k))
            } else {
                None
            }
        })
        .collect();

    for edge in &catalog.edges {
        // Only report UnresolvedRef targets — UnvalidatedText targets dangle
        // by design (free-text / unknown-prefix targets), and Resolved targets
        // are fine.
        if let EdgeTarget::UnresolvedRef { raw } = &edge.target {
            // Only report danglers for validated labels — Unvalidated labels
            // (TargetSpec::Unvalidated in RELATION_RULES) dangle by design
            // (their targets are free-form by contract).
            // Edge source always exists in entity_kinds — edges are built from
            // entities in the same Catalog. A None here is a bug — guarded by
            // the invariant that every CatalogEdge.source is drawn from the
            // same Catalog whose entities built entity_kinds.
            let CatalogKey::Numbered(source_key) = &edge.source else {
                continue;
            };
            let CatalogEdgeLabel::Validated(label) = &edge.label else {
                // Raw label on a numbered edge is catalog corruption.
                findings.push(format!(
                    "internal: numbered edge {} has Raw label {:?}",
                    source_key.canonical(),
                    edge.label.name()
                ));
                continue;
            };
            let Some(kind) = entity_kinds.get(source_key) else {
                findings.push(format!(
                    "internal: edge source {} not in entity-kind map",
                    source_key.canonical()
                ));
                continue;
            };
            // Role-aware `lookup` (SL-149 PHASE-04): the dangler walk reads the
            // validated-ness axis off the `(source, label, role)` rule. The `CatalogEdge`
            // now carries `role` (PHASE-04 threaded it through hydrate), so a `references`
            // edge resolves its own role-keyed rule rather than missing a label-only
            // lookup. Inert until the P5 migration authors live `references` edges, but
            // closed now so references danglers resolve correctly when they land. The
            // role-class hand-edit finding rides the IllegalRows re-read below.
            let validated = crate::relation::lookup(kind, *label, edge.role)
                .is_some_and(|r| !matches!(r.target, TargetSpec::Unvalidated));
            if validated {
                findings.push(format!(
                    "{}: `{}` target `{}` does not resolve (dangling [[relation]] edge)",
                    edge.source.canonical(),
                    edge.label.name(),
                    raw
                ));
            }
        }
    }

    // (2) IllegalRows — hand-edited off-table `(source, label)` rows. Re-read the raw
    // `[[relation]]` block per entity (scan_entities drops the illegal rows).
    for kref in integrity::KINDS {
        let mut ids = entity::scan_ids(&root.join(kref.kind.dir))?;
        ids.sort_unstable();
        for id in ids {
            let toml_path = crate::entity::id_path(root, kref.kind, id, crate::entity::Ext::Toml);
            let text = std::fs::read_to_string(&toml_path)
                .map_err(|e| anyhow::anyhow!("read {} for validate: {e}", toml_path.display()))?;
            let doc = crate::relation::RelationDoc::parse(&text)?;
            let (_edges, illegal) = crate::relation::read_block(kref.kind, &doc);
            for row in illegal {
                let why = match row.reason {
                    crate::relation::IllegalReason::UnknownLabel => "unknown label",
                    crate::relation::IllegalReason::IllegalForSource => "label illegal for source",
                    // SL-149: a hand-edited `references` row with a missing/illegal/stray
                    // role — the role-class finding. Label-only rows never reach here.
                    crate::relation::IllegalReason::IllegalRole => "missing or illegal role",
                };
                findings.push(format!(
                    "{}: [[relation]] row `{}` -> `{}` is illegal ({why})",
                    listing::canonical_id(kref.kind.prefix, id),
                    row.label,
                    row.target
                ));
            }
        }
    }

    findings.extend(validate_supersession(root)?);
    Ok(findings)
}

/// The supersession cross-check (design §5.5, R2-m2 / OD-3 / ADR-010 D4): report where
/// a governance entity's STORED `superseded_by` disagrees with the reciprocal DERIVED
/// from `supersedes` in-edges. Pure read, report-only — MAY surface pre-existing
/// hand-authored drift, which is the intended point (C3); NEVER rewrites.
///
/// The stored side is read via the typed governance seam
/// (`governance::supersession_pair` → `doc.relationships.superseded_by`) — the generic
/// `read_block`/`outbound_for` path deliberately excludes `superseded_by`. The derived
/// side is built per gov kind: X's derived `superseded_by` = every same-kind Y whose
/// stored `supersedes` lists X. Disagreement EITHER WAY (stored-not-derived /
/// derived-not-stored) is a finding.
fn validate_supersession(root: &Path) -> anyhow::Result<Vec<String>> {
    use std::collections::{BTreeMap, BTreeSet};

    // Per governance kind, drive its own ADR/POL/STD namespace.
    let gov_kinds: &[&crate::governance::GovKind] = &[
        &crate::adr::ADR_KIND,
        &crate::policy::POLICY_KIND,
        &crate::standard::STANDARD_KIND,
    ];

    let mut findings = Vec::new();
    for g in gov_kinds {
        let prefix = g.kind.prefix;
        let mut ids = entity::scan_ids(&root.join(g.kind.dir))?;
        ids.sort_unstable();

        // Read every entity's (supersedes, superseded_by) once.
        let mut stored: BTreeMap<u32, (Vec<String>, Vec<String>)> = BTreeMap::new();
        for id in &ids {
            stored.insert(*id, crate::governance::supersession_pair(g, root, *id)?);
        }

        // Derived reciprocal: for each Y listing X in `supersedes`, X is superseded_by Y.
        let mut derived: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (y, (sup, _)) in &stored {
            let y_ref = listing::canonical_id(prefix, *y);
            for x_ref in sup {
                derived
                    .entry(x_ref.clone())
                    .or_default()
                    .insert(y_ref.clone());
            }
        }

        // Compare stored superseded_by against the derived set, both ways.
        for (x, (_sup, stored_by)) in &stored {
            let x_ref = listing::canonical_id(prefix, *x);
            let stored_set: BTreeSet<String> = stored_by.iter().cloned().collect();
            let derived_set = derived.get(&x_ref).cloned().unwrap_or_default();
            for missing in derived_set.difference(&stored_set) {
                findings.push(format!(
                    "{x_ref}: `{missing}` supersedes it (derived) but `{x_ref}` does not list \
                     it in `superseded_by` (supersession drift)"
                ));
            }
            for extra in stored_set.difference(&derived_set) {
                findings.push(format!(
                    "{x_ref}: lists `{extra}` in `superseded_by` but `{extra}` does not \
                     `supersede` it (supersession drift)"
                ));
            }
        }
    }
    Ok(findings)
}

/// The grouping key of an inspect relation surface (SL-149 §2.6 / F1): the structural
/// `RelationLabel` plus the intent `Role` that refines a `references` edge (`None` on
/// every label-only edge). Both outbound and inbound group on this key, so role-bearing
/// `references` edges render distinct verbs instead of collapsing into one label bucket.
pub(crate) type RelationKey = (RelationLabel, Option<Role>);

/// One `(label, role)` group of an inspect surface: the key and its targets (canonical
/// refs), in render order.
type RelationGroup = (RelationKey, Vec<String>);

/// One entity's direct relation view (design §5.2): its authored outbound relations
/// grouped by `(label, role)`, the derived inbound relations grouped by `(label, role)`,
/// and its unresolved/free-text outbound danglers. Direct-only, one-hop, composition-free
/// (I2). Inbound is recomputed every query from `in_edges` — nothing stores a
/// reverse field (ADR-004 §3 / REQ-074).
#[derive(Debug)]
pub(crate) struct InspectView {
    pub(crate) id: String,
    /// Outbound relations grouped by `(label, role)` (SL-149 §2.6 / F1): a `references`
    /// edge groups under its `Some(role)` and renders `references(<role>)`; every
    /// label-only edge groups under `None` and renders the bare label. The role rides
    /// from the entity's own `outbound_for` edge payload.
    pub(crate) outbound: Vec<RelationGroup>,
    /// Inbound relations grouped by `(label, role)` (SL-149 §2.6 / F1). The role is
    /// recovered from the SOURCE entity's outbound edge PAYLOAD — NOT from the cordage
    /// overlay, which stays label-keyed (R5). Re-keying by `(label, role)` is what keeps
    /// inbound `implements` and `concerns` in distinct buckets with distinct verbs
    /// ("implemented by" / "concerned by") instead of collapsing into one.
    pub(crate) inbound: Vec<RelationGroup>,
    pub(crate) danglers: Vec<(RelationLabel, String)>,
}

/// `inspect <ID>` — the cross-kind relation view of one entity (design §5.2/§5.4).
///
/// Parses `id` via `integrity::parse_canonical_ref` (an unknown prefix / malformed
/// ref → a clean `anyhow` error, never a panic), builds the relation graph once,
/// and returns the entity's direct relations:
/// - **outbound**: the entity's own `outbound_for` edges, grouped by label,
///   targets in authored order within a label.
/// - **inbound**: per overlay, `graph.in_edges(ov, node)` → source `EntityKey` →
///   canonical ref, grouped under `label_of(ov)`. The `Supersedes`-overlay inbound
///   is the derived reciprocal "superseded by" (ADR-004 §3) — carried under the
///   `Supersedes` label here; PHASE-04 render flips the word. NO stored
///   `superseded_by` field is read (C8/R3/VT-4).
/// - **danglers**: the queried entity's unresolved / free-text / no-overlay
///   outbound targets.
///
/// A well-formed ref to a non-existent id (never minted) is an ERROR — `"{KIND-NNN}:
/// no such entity"` (SL-050 F6): an unminted id is indistinguishable from a real
/// isolated node at the render layer, so the existence gate makes it a clean failure.
/// The `require_minted` bail also keeps `outbound_for` (which reads the entity's own
/// toml) off a missing file — the gate subsumes the old missing-file guard. NEVER
/// reads `graph.provenance()` (C7 — a benign symmetric-`related` 2-cycle yields a
/// `Reject` `CycleDiagnostic` that must not leak into the view).
///
/// The own-scan convenience wrapper over [`inspect_from`] for callers that do NOT
/// already hold a corpus scan — the unit suite below. The command layer (`main.rs`)
/// holds the single F2 scan and calls `inspect_from`/`render_from` directly, so in a
/// non-test build this wrapper has no caller.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "own-scan convenience wrapper for the unit suite; the F2 command layer \
                  calls inspect_from with the shared scan, so it is test-only"
    )
)]
pub(crate) fn inspect(root: &Path, id: &str) -> anyhow::Result<InspectView> {
    inspect_from(
        &scan_entities(root, &mut vec![], ScanMode::default())?,
        root,
        id,
    )
}

/// `inspect` over a PRE-SCANNED entity slice (the SL-050 F2 shared-scan seam). `inspect`
/// is now the thin `scan_entities(root)?` + delegate wrapper; this carries the body.
/// `root` is RETAINED for the queried entity's OWN per-entity re-reads — its outbound
/// `outbound_for` (its own toml) and `render_human`'s interaction-type read — which are
/// per-entity, not corpus, so they are not part of `scan_entities` and stay as-is.
pub(crate) fn inspect_from(
    scanned: &[ScannedEntity],
    root: &Path,
    id: &str,
) -> anyhow::Result<InspectView> {
    let (kref, qid) = integrity::parse_canonical_ref(id)?;
    let query_key = EntityKey {
        prefix: kref.kind.prefix,
        id: qid,
    };

    let rg = build_relation_graph_from(scanned)?;

    // Existence gate (F6): a well-formed ref to a never-minted id is an error, not an
    // empty-section view — the `Projection` holds exactly the minted keys. This also
    // keeps `outbound_for` (which reads the entity's own toml below) off a missing file.
    require_minted(&rg.projection, query_key)?;
    // Present by construction now the gate has passed.
    let Some(node) = rg.projection.resolve(query_key) else {
        debug_assert!(false, "inspect_from: gate passed but key not resolvable");
        anyhow::bail!("{}: no such entity", query_key.canonical());
    };

    // outbound — the entity's own authored edges, grouped by `(label, role)` (targets in
    // authored order within a group). The role rides on the edge payload (SL-149 F1): a
    // `references` edge groups under `Some(role)`, a label-only edge under `None`.
    let mut outbound_by_key: BTreeMap<(RelationLabel, Option<Role>), Vec<String>> = BTreeMap::new();
    for edge in outbound_for(root, kref.kind, qid)? {
        outbound_by_key
            .entry((edge.label, edge.role))
            .or_default()
            .push(edge.target);
    }
    let outbound: Vec<RelationGroup> = outbound_by_key.into_iter().collect();

    // inbound — derived from in_edges per overlay (no stored reverse field read). The
    // cordage overlay is LABEL-keyed (R5 — one `references` overlay), so the role is NOT
    // in the graph edge; it is recovered from the SOURCE entity's outbound PAYLOAD via
    // [`inbound_role_index`] (F1). Re-keying by `(label, role)` keeps `implements` and
    // `concerns` inbound in distinct buckets, each with its own derived verb.
    let role_index = inbound_role_index(scanned);
    let mut inbound_by_key: BTreeMap<(RelationLabel, Option<Role>), Vec<EntityKey>> =
        BTreeMap::new();
    for (&overlay, &label) in &rg.overlays.by_overlay {
        for (src_node, _attrs) in rg.graph.in_edges(overlay, node) {
            let Some(src_key) = rg.projection.key_of(src_node) else {
                continue;
            };
            // The role of the source's edge to the queried entity under this label —
            // `None` for a label-only edge, `Some(role)` for a `references` edge. A
            // missing index entry (cannot happen for a graph edge built from the same
            // scan) falls back to `None`.
            let role = role_index
                .get(&(src_key, label, query_key))
                .copied()
                .unwrap_or(None);
            inbound_by_key
                .entry((label, role))
                .or_default()
                .push(src_key);
        }
    }
    // in_edges orders by the (src,rank,age) adjacency key, but src NodeId order is mint
    // order, not ref order — sort each group by EntityKey::Ord (prefix lexicographic, id
    // numeric) for a deterministic, permutation-invariant render correct past id 999
    // (RSK-007).
    let inbound: Vec<RelationGroup> = inbound_by_key
        .into_iter()
        .map(|(key, mut srcs)| {
            srcs.sort();
            (key, srcs.into_iter().map(EntityKey::canonical).collect())
        })
        .collect();

    // danglers — only the queried entity's set (empty if none).
    let danglers = rg.danglers.get(&query_key).cloned().unwrap_or_default();

    Ok(InspectView {
        id: query_key.canonical(),
        outbound,
        inbound,
        danglers,
    })
}

// ---------------------------------------------------------------------------
// PHASE-04 — the `inspect <ID>` command: render (human + --json) and the shell.
// ---------------------------------------------------------------------------

/// Render the relation view of `id` to a string from a PRE-SCANNED entity slice (the
/// command-layer seam, SL-047 §5.4 + SL-050 F2): `main.rs`'s `inspect` handler builds
/// the single corpus scan ONCE, calls this for the relation portion, then APPENDS the
/// priority actionability block BELOW it (the composition lives at the command layer,
/// which alone may depend on both `relation_graph` and `priority`; ADR-001 forbids
/// `relation_graph` from calling up into `priority`). The relation portion stays
/// byte-identical — the appended block is additive (EX-2 / VT-2 behaviour-preserving).
/// No trailing newline on JSON (the golden contract); the human surface ends in `\n`.
///
/// Delegates through [`inspect_from`], so it inherits the F6 existence gate (a
/// never-minted id errors before any render). `root` is retained for the queried
/// entity's own per-entity re-reads (`inspect_from`'s outbound + `render_human`'s
/// interaction types).
pub(crate) fn render_from(
    scanned: &[ScannedEntity],
    root: &Path,
    id: &str,
    format: Format,
) -> anyhow::Result<String> {
    let view = inspect_from(scanned, root, id)?;
    match format {
        // The queried entity's per-edge interaction `type` is re-read from the
        // SOURCE here (C2 / §5.3) — a human-render annotation only; never carried
        // in `InspectView`.
        Format::Table => render_human(root, &view),
        Format::Json => render_json(&view),
    }
}

/// Render one entity's relation view for human reading (default). Fixed
/// deterministic section order — **outbound, then inbound, then danglers** (EX-2);
/// each section omitted when empty (the `show`-surface convention — governance /
/// spec `format_show` omit empty relationship blocks; VT-3). Within a section,
/// labels are already ordered (the `RelationLabel` `Ord`), targets in the view's
/// order. House style: `Vec<String>` parts each carrying their own newline, joined
/// by `concat` (the `governance::format_show` / `backlog::format_show` precedent —
/// avoids the `push_str(&format!)` lint).
///
/// Two presentation flips, both by SECTION (never by reading a stored field):
/// - inbound `Supersedes` renders the word **"superseded by"** (the derived
///   reciprocal — ADR-004 §3); outbound `Supersedes` stays **"supersedes"**.
/// - the queried entity's OUTBOUND `Interactions` targets are annotated with their
///   per-edge free-text `type`, re-read from the source `interactions.toml` via the
///   spec reader (C2 / EX-4) — `SPEC-002 (calls)`.
fn render_human(root: &Path, view: &InspectView) -> anyhow::Result<String> {
    // Re-read the queried entity's interaction types from source (C2) — only a tech
    // spec authors any; every other kind yields an empty map, so the annotation is a
    // no-op there. `parse_canonical_ref` already classified the id in `inspect`; the
    // queried id is `view.id`.
    let interaction_types = match integrity::parse_canonical_ref(&view.id) {
        Ok((kref, qid)) if kref.kind.prefix == "SPEC" => crate::spec::interaction_types(root, qid)?,
        _ => BTreeMap::new(),
    };

    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("{} — relations\n", view.id));

    render_outbound(&mut parts, view, &interaction_types);
    render_inbound(&mut parts, view);
    render_danglers(&mut parts, view);

    // An entity with no relations at all renders the header plus an explicit note,
    // so an empty view is never a bare one-liner (VT-3 — empty sections render
    // cleanly).
    if view.outbound.is_empty() && view.inbound.is_empty() && view.danglers.is_empty() {
        parts.push("\n(no relations)\n".to_string());
    }
    Ok(parts.concat())
}

/// Append the outbound section (omitted when empty). The queried tech spec's
/// `Interactions` targets carry their re-read free-text `type` annotation (C2).
fn render_outbound(
    parts: &mut Vec<String>,
    view: &InspectView,
    interaction_types: &BTreeMap<String, String>,
) {
    if view.outbound.is_empty() {
        return;
    }
    parts.push("\noutbound:\n".to_string());
    for ((label, role), targets) in &view.outbound {
        let rendered: Vec<String> = if *label == RelationLabel::Interactions {
            targets
                .iter()
                .map(|t| match interaction_types.get(t) {
                    Some(ty) => format!("{t} ({ty})"),
                    None => t.clone(),
                })
                .collect()
        } else {
            targets.clone()
        };
        parts.push(format!(
            "  {}: {}\n",
            outbound_label(*label, *role),
            rendered.join(", ")
        ));
    }
}

/// The rendered outbound label for `(label, role)` (SL-149 §2.6): a `references` edge
/// renders `references(<role>)` (e.g. `references(implements)`); every label-only edge
/// (role `None`) renders the bare label, byte-identical to the pre-SL-149 surface
/// (behaviour-preservation for `specs`/`supersedes`/`governed_by`/… goldens).
fn outbound_label(label: RelationLabel, role: Option<Role>) -> String {
    match role {
        Some(role) => format!("{}({})", label.name(), role.name()),
        None => label.name().to_string(),
    }
}

/// Append the inbound section (omitted when empty). The `Supersedes` overlay's
/// inbound is the derived reciprocal — rendered as the word "superseded by"
/// (ADR-004 §3); the flip is by SECTION, not by reading any stored field.
fn render_inbound(parts: &mut Vec<String>, view: &InspectView) {
    if view.inbound.is_empty() {
        return;
    }
    parts.push("\ninbound:\n".to_string());
    for ((label, role), srcs) in &view.inbound {
        // Table-driven inbound render text (design §5.5 X5 / R2-M3, re-keyed SL-149
        // §2.6): `relation::inbound_name(label, role)` renders `supersedes` → "superseded
        // by", `governed_by` → "governs", and the role-keyed `references` verbs
        // (`implements` → "implemented by", `concerns` → "concerned by"). Legacy
        // label-only edges pass role `None` and carry `inbound_name == name()`, so
        // shipped goldens are unchanged. The `--json` inbound keeps the raw label +
        // role (`inspect_value`), per R2-M3.
        let word = crate::relation::inbound_name(*label, *role);
        parts.push(format!("  {word}: {}\n", srcs.join(", ")));
    }
}

/// Append the danglers section (omitted when empty) — the queried entity's
/// unresolved / free-text / no-overlay outbound targets, grouped by label.
fn render_danglers(parts: &mut Vec<String>, view: &InspectView) {
    if view.danglers.is_empty() {
        return;
    }
    parts.push("\ndanglers:\n".to_string());
    for (label, target) in &view.danglers {
        parts.push(format!("  {}: {target}\n", label.name()));
    }
}

/// Render the `--json` view: the serialized `InspectView`, every surface asserted
/// (`id`, `outbound`, `inbound`, `danglers` — VT-2). Built MANUALLY with
/// `serde_json::json!` (the `spec::show_json` precedent — the repo derives no
/// `Serialize` on domain enums; `RelationLabel` renders via `.name()`). Each label
/// group is `{ "label": <name>, "targets": [...] }`; each dangler is
/// `{ "label": <name>, "target": <ref> }`. The interaction `type` is a human-render
/// extra ONLY (design §5.2) — it is NOT in the JSON. The envelope is the 4
/// `InspectView` fields (`id`/`outbound`/`inbound`/`danglers`) under a `"kind":
/// "inspect"` discriminant (the `spec::show_json` envelope precedent), so an agent
/// reads the same shape `InspectView` carries. No trailing newline (the black-box
/// golden contract — `write!`, not `writeln!`).
fn render_json(view: &InspectView) -> anyhow::Result<String> {
    serde_json::to_string_pretty(&inspect_value(view))
        .map_err(|e| anyhow::anyhow!("failed to serialize inspect JSON: {e}"))
}

/// The `inspect` `--json` envelope as a [`serde_json::Value`] (the 4 `InspectView`
/// surfaces under a `"kind": "inspect"` discriminant). Factored out so the command
/// layer can INJECT the priority `actionability` block as an additive key (SL-047
/// §5.4 / SL-046 D1) without `relation_graph` depending on `priority` (ADR-001) — the
/// relation surfaces stay byte-identical; only a new key is added.
pub(crate) fn inspect_value(view: &InspectView) -> serde_json::Value {
    // The JSON carries the STRUCTURAL label faithfully (`references`, not the human
    // `references(implements)`); the role is an additive sibling key, emitted ONLY for a
    // role-bearing `references` group (SL-149 §2.6 / R2-M3). A label-only group omits the
    // `role` key, so every shipped label-only golden stays byte-identical.
    let group = |label: RelationLabel, role: Option<Role>, targets: &[String]| match role {
        Some(role) => {
            serde_json::json!({ "label": label.name(), "role": role.name(), "targets": targets })
        }
        None => serde_json::json!({ "label": label.name(), "targets": targets }),
    };
    let outbound: Vec<serde_json::Value> = view
        .outbound
        .iter()
        .map(|((l, r), t)| group(*l, *r, t))
        .collect();
    let inbound: Vec<serde_json::Value> = view
        .inbound
        .iter()
        .map(|((l, r), t)| group(*l, *r, t))
        .collect();
    let danglers: Vec<serde_json::Value> = view
        .danglers
        .iter()
        .map(|(l, t)| serde_json::json!({ "label": l.name(), "target": t }))
        .collect();
    serde_json::json!({
        "kind": "inspect",
        "id": view.id,
        "outbound": outbound,
        "inbound": inbound,
        "danglers": danglers,
    })
}

// ---------------------------------------------------------------------------
// SL-138 — the relation-transitive walk (design §5). The engine-layer transitive
// query over the cordage depth-bounded primitive (`Graph::reachable_bounded`),
// plus its human + JSON renders. Relation-only — it never calls `priority`
// (ADR-001): no actionability block, no up-call. A SEPARATE entry point from
// `inspect_from` (1-hop), not a flag on it: the two render contracts differ.
// ---------------------------------------------------------------------------

/// Walk direction for [`transitive_from`], defined HERE in the engine layer
/// (ADR-001): the command layer maps its clap flag DOWN to this, so the engine
/// never depends on a command-layer type (the `Format`/`listing.rs` precedent).
/// `Inbound` walks in-edges (`Against` — blast radius: what transitively depends on
/// the entity); `Outbound` walks out-edges (`Along` — derivation / governance
/// ancestry); `Both` emits the two as separate sections (the awareness view, D3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransitiveDir {
    Inbound,
    Outbound,
    Both,
}

impl TransitiveDir {
    fn has_inbound(self) -> bool {
        matches!(self, Self::Inbound | Self::Both)
    }
    fn has_outbound(self) -> bool {
        matches!(self, Self::Outbound | Self::Both)
    }
}

/// One label's transitively-reachable target set in a single direction. `targets`
/// are canonical ids, id-ascending (REQ-077 determinism); `truncated` is the OR of
/// the depth cap biting across that label's walk.
#[derive(Debug)]
pub(crate) struct TransitiveGroup {
    pub label: RelationLabel,
    pub targets: Vec<String>,
    pub truncated: bool,
}

/// The transitive view of one entity (design §5 output contract, C4). `inbound` is
/// `Some` iff the requested direction includes inbound (emitted FIRST — the
/// blast-radius framing), `outbound` likewise. A requested direction with no
/// reachable edges is `Some(vec![])` (renders `(none)` / `[]`); a NON-requested
/// direction is `None` — omitted entirely (the table emits no section, the JSON no
/// key). `max_depth` is `None` when unbounded; view-level `truncated` is the OR
/// across every emitted group (the table's "… some chains truncated" line reads it).
#[derive(Debug)]
pub(crate) struct TransitiveView {
    pub id: String,
    pub max_depth: Option<usize>,
    pub truncated: bool,
    pub inbound: Option<Vec<TransitiveGroup>>,
    pub outbound: Option<Vec<TransitiveGroup>>,
}

/// The overlay-backed labels to walk, sorted by `name()` ascending (the §5 group
/// order). `None` → every label the graph allocates an overlay for, read off the
/// live [`OverlayMap`] (table-derived from [`RELATION_RULES`] via [`OverlayMap::build`],
/// so the no-overlay set `{contextualizes, drift, decision_ref}` is excluded by
/// construction — NO hardcoded list, C2). `Some(ls)` → `ls` after rejecting any label
/// with no overlay: those are 1-hop-only by nature (`TargetSpec::Unvalidated` — their
/// targets never resolve to a node) and not transitively walkable (EX-3).
///
/// # Errors
///
/// A requested label lacking an overlay (`contextualizes` / `drift` / `decision_ref`)
/// yields a "not transitively walkable" error listing the overlay-backed set.
/// The shared "not transitively walkable" error body: the offending names plus the
/// table-derived overlay-backed set (sorted). ONE message for both rejection cases —
/// an unknown name (CLI `from_name` miss) and a known no-overlay label (F4) — so the
/// surface is uniform regardless of which tier caught it.
fn not_walkable_message(bad: &[&str], overlays: &OverlayMap) -> String {
    let mut walkable: Vec<&str> = overlays.by_label.keys().map(|label| label.name()).collect();
    walkable.sort_unstable();
    format!(
        "not transitively walkable: {}; overlay-backed labels are: {}",
        bad.join(", "),
        walkable.join(", ")
    )
}

/// Resolve raw `--labels` names (the command tier) to validated overlay-backed
/// [`RelationLabel`]s. Empty input → `None` (the default = every overlay-backed
/// label). An unknown name OR a no-overlay name (`contextualizes` / `drift` /
/// `decision_ref`) yields one [`not_walkable_message`] error. Table-derived via
/// [`OverlayMap::build`] (no scan, no hardcoded list — C2/F4): the SINGLE
/// name-validation point for `inspect --transitive`. The command layer calls this and
/// hands the result to [`transitive_from`] (ADR-001 — the engine never parses the clap
/// strings; it only ever sees typed `RelationLabel`s).
///
/// # Errors
///
/// A name that is unknown or not overlay-backed (see above).
pub(crate) fn resolve_transitive_label_names(
    names: &[String],
) -> anyhow::Result<Option<Vec<RelationLabel>>> {
    if names.is_empty() {
        return Ok(None);
    }
    let mut builder = GraphBuilder::new();
    let overlays = OverlayMap::build(&mut builder);
    let mut good = Vec::with_capacity(names.len());
    let mut bad = Vec::new();
    for name in names {
        match RelationLabel::from_name(name) {
            Some(label) if overlays.overlay_for(label).is_some() => good.push(label),
            _ => bad.push(name.as_str()),
        }
    }
    if !bad.is_empty() {
        anyhow::bail!("{}", not_walkable_message(&bad, &overlays));
    }
    Ok(Some(good))
}

fn transitive_labels(
    overlays: &OverlayMap,
    labels: Option<&[RelationLabel]>,
) -> anyhow::Result<Vec<RelationLabel>> {
    let mut selected: Vec<RelationLabel> = match labels {
        None => overlays.by_label.keys().copied().collect(),
        Some(requested) => {
            let bad: Vec<&str> = requested
                .iter()
                .filter(|label| overlays.overlay_for(**label).is_none())
                .map(|label| label.name())
                .collect();
            if !bad.is_empty() {
                anyhow::bail!("{}", not_walkable_message(&bad, overlays));
            }
            requested.to_vec()
        }
    };
    selected.sort_by_key(|label| label.name());
    Ok(selected)
}

/// Walk one direction over the selected overlays, one `reachable_bounded` per label,
/// returning the NON-EMPTY groups (a label with zero reachable targets is omitted —
/// design §4/§5: an empty direction renders a single `(none)`, never one `(none)` per
/// label). A truncated walk always has ≥1 target (the node at the cap is in `depths`),
/// so suppressing empty groups never drops a truncation signal. `targets` are mapped
/// `NodeId`→`EntityKey`→canonical and sorted id-ascending (REQ-077).
fn walk_transitive(
    rg: &RelationGraph,
    node: cordage::NodeId,
    direction: Direction,
    labels: &[RelationLabel],
    max_depth: Option<usize>,
) -> Vec<TransitiveGroup> {
    let mut groups = Vec::new();
    for &label in labels {
        // Present by construction: `transitive_labels` rejected every no-overlay label.
        let Some(overlay) = rg.overlays.overlay_for(label) else {
            continue;
        };
        let reach = rg
            .graph
            .reachable_bounded(overlay, node, direction, max_depth);
        let mut targets: Vec<EntityKey> = reach
            .depths
            .keys()
            .filter_map(|reached| rg.projection.key_of(*reached))
            .collect();
        if targets.is_empty() {
            continue;
        }
        targets.sort();
        groups.push(TransitiveGroup {
            label,
            targets: targets.into_iter().map(EntityKey::canonical).collect(),
            truncated: reach.truncated,
        });
    }
    groups
}

/// The transitive relation query (design §5): walk the cross-kind relation graph
/// from `id`, per selected overlay × per selected direction, via the cordage
/// depth-bounded primitive. Rides the same seams as [`inspect_from`] —
/// [`build_relation_graph_from`] for the graph and [`require_minted`] for the
/// existence gate — so a never-minted id errors identically (EX-4). Relation-only:
/// no `priority` up-call (ADR-001).
///
/// `_root` is retained for call-site symmetry with [`inspect_from`] / [`render_from`]
/// (the PHASE-03 command layer threads the same `(scanned, root, id, …)` tuple), per
/// the §5 signature; the relation-only walk reads nothing per-entity from disk, so it
/// is currently unused here.
///
/// # Errors
///
/// A malformed / unknown-prefix `id` (parse), a never-minted id (the existence gate),
/// or a `labels` entry that is not overlay-backed ([`transitive_labels`]).
pub(crate) fn transitive_from(
    scanned: &[ScannedEntity],
    _root: &Path,
    id: &str,
    dir: TransitiveDir,
    labels: Option<&[RelationLabel]>,
    max_depth: Option<usize>,
) -> anyhow::Result<TransitiveView> {
    let (kref, qid) = integrity::parse_canonical_ref(id)?;
    let query_key = EntityKey {
        prefix: kref.kind.prefix,
        id: qid,
    };

    let rg = build_relation_graph_from(scanned)?;

    // Existence gate (F6, shared with `inspect_from`): a well-formed ref to a
    // never-minted id is an error, not an empty-section view.
    require_minted(&rg.projection, query_key)?;
    let Some(node) = rg.projection.resolve(query_key) else {
        debug_assert!(false, "transitive_from: gate passed but key not resolvable");
        anyhow::bail!("{}: no such entity", query_key.canonical());
    };

    // Validate + order the label set ONCE (rejects no-overlay labels up front), then
    // reuse it for each requested direction.
    let selected = transitive_labels(&rg.overlays, labels)?;

    let inbound = dir
        .has_inbound()
        .then(|| walk_transitive(&rg, node, Direction::Against, &selected, max_depth));
    let outbound = dir
        .has_outbound()
        .then(|| walk_transitive(&rg, node, Direction::Along, &selected, max_depth));

    // View-level truncation: OR across every emitted group in either direction (C4).
    let truncated = inbound
        .iter()
        .chain(outbound.iter())
        .flatten()
        .any(|group| group.truncated);

    Ok(TransitiveView {
        id: query_key.canonical(),
        max_depth,
        truncated,
        inbound,
        outbound,
    })
}

/// Render the transitive view for human reading (design §4/§5). Header carries the
/// depth (`depth 5`, or `depth all` when unbounded); sections are **inbound then
/// outbound** (blast-radius first), each omitted when the direction was not requested
/// (`None`) and rendered as a single `(none)` when requested-but-empty. A trailing
/// truncation line appears iff the view truncated. House style: `Vec<String>` parts
/// joined by `concat` (the `render_human` precedent — avoids the `push_str(&format!)`
/// lint).
pub(crate) fn render_transitive_human(view: &TransitiveView) -> String {
    let depth = view
        .max_depth
        .map_or_else(|| "all".to_string(), |d| d.to_string());
    let mut parts: Vec<String> = vec![format!("{} — transitive (depth {depth})\n", view.id)];

    if let Some(groups) = &view.inbound {
        parts.push("\ndepends on this (inbound):\n".to_string());
        push_transitive_groups(&mut parts, groups);
    }
    if let Some(groups) = &view.outbound {
        parts.push("\nthis depends on (outbound):\n".to_string());
        push_transitive_groups(&mut parts, groups);
    }
    if view.truncated {
        // `truncated` implies a finite cap bit, so `max_depth` is `Some` here; `depth`
        // already holds its string form (`all` is unreachable when truncated).
        parts.push(format!(
            "\n… some chains truncated at depth {depth} — re-run with --max-depth all\n"
        ));
    }
    parts.concat()
}

/// Append one direction's group lines (`  label: a, b, c`), or a single `  (none)`
/// when the requested direction reached nothing.
fn push_transitive_groups(parts: &mut Vec<String>, groups: &[TransitiveGroup]) {
    if groups.is_empty() {
        parts.push("  (none)\n".to_string());
        return;
    }
    for group in groups {
        parts.push(format!(
            "  {}: {}\n",
            group.label.name(),
            group.targets.join(", ")
        ));
    }
}

/// Render the transitive `--json` view (no trailing newline — the golden contract).
///
/// # Errors
///
/// Propagates a `serde_json` serialization failure (not expected for this shape).
pub(crate) fn render_transitive_json(view: &TransitiveView) -> anyhow::Result<String> {
    serde_json::to_string_pretty(&transitive_value(view))
        .map_err(|e| anyhow::anyhow!("failed to serialize transitive JSON: {e}"))
}

/// The transitive `--json` envelope as a [`serde_json::Value`] (design §5 C4),
/// discriminated `"kind": "inspect-transitive"`. A non-requested direction key is
/// OMITTED (not null/empty); `max_depth` is JSON `null` when unbounded. Each group is
/// `{ "label", "truncated", "targets" }` — NO `role` key (transitive `references` is
/// role-collapsed, F3). Keys serialize alphabetically (no `preserve_order`), so
/// `inbound` precedes `outbound` and `kind` falls mid-object — the order the C4 golden
/// pins. Built MANUALLY (the `inspect_value` precedent — the repo derives no
/// `Serialize` on domain enums; `RelationLabel` renders via `.name()`).
pub(crate) fn transitive_value(view: &TransitiveView) -> serde_json::Value {
    let group = |group: &TransitiveGroup| {
        serde_json::json!({
            "label": group.label.name(),
            "truncated": group.truncated,
            "targets": group.targets,
        })
    };
    let direction =
        |groups: &[TransitiveGroup]| serde_json::Value::Array(groups.iter().map(&group).collect());

    let mut obj = serde_json::Map::new();
    obj.insert("kind".to_string(), serde_json::json!("inspect-transitive"));
    obj.insert("id".to_string(), serde_json::json!(view.id));
    obj.insert(
        "max_depth".to_string(),
        view.max_depth
            .map_or(serde_json::Value::Null, |d| serde_json::json!(d)),
    );
    obj.insert("truncated".to_string(), serde_json::json!(view.truncated));
    if let Some(groups) = &view.inbound {
        obj.insert("inbound".to_string(), direction(groups));
    }
    if let Some(groups) = &view.outbound {
        obj.insert("outbound".to_string(), direction(groups));
    }
    serde_json::Value::Object(obj)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrity::KINDS;
    use crate::relation::RelationLabel;
    use crate::test_support::{SCHEMA_BACKLOG, SCHEMA_KNOWLEDGE};
    use std::fs;

    /// Write `parent/dir/<name>` with `body`, creating parents.
    fn write(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    /// Find the `KindRef` for a prefix (the dispatch input the scan supplies).
    fn kind_for(prefix: &str) -> &'static entity::Kind {
        KINDS.iter().find(|k| k.kind.prefix == prefix).unwrap().kind
    }

    /// (label, target) pairs for ergonomic assertions.
    fn pairs(edges: &[RelationEdge]) -> Vec<(RelationLabel, &str)> {
        edges.iter().map(|e| (e.label, e.target.as_str())).collect()
    }

    /// A throwaway corpus root under an RAII temp dir (auto-removed on drop), matching
    /// the spec.rs tests' `tempfile::tempdir()` convention — no hand-rolled
    /// pid/nanos uniqueness, no leaked dirs. Callers bind the `TempDir` and read
    /// `.path()` so the dir outlives the test body.
    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    // -- VT-1 outbound correctness per kind + outbound_for dispatch ----------

    #[test]
    fn slice_outbound_references_supersedes() {
        let dir = tmp();
        let root = dir.path();
        write(
            &root,
            ".doctrine/slice/001/slice-001.toml",
            // SL-149 PHASE-05: the old specs/requirements rows are now references(implements).
            "id = 1\nslug = \"a\"\ntitle = \"A\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"PRD-010\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"REQ-001\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"REQ-002\"\n\
             [[relation]]\nlabel = \"supersedes\"\ntarget = \"SL-000\"\n",
        );
        write(&root, ".doctrine/slice/001/slice-001.md", "scope\n");
        let edges = outbound_for(&root, kind_for("SL"), 1).unwrap();
        assert_eq!(
            pairs(&edges),
            vec![
                (RelationLabel::References, "PRD-010"),
                (RelationLabel::References, "REQ-001"),
                (RelationLabel::References, "REQ-002"),
                (RelationLabel::Supersedes, "SL-000"),
            ]
        );
    }

    #[test]
    fn governance_outbound_supersedes_related_only() {
        let dir = tmp();
        let root = dir.path();
        // ADR with every axis populated — only supersedes + related must emit.
        write(
            &root,
            ".doctrine/adr/002/adr-002.toml",
            // SL-095 PHASE-02: `supersedes` is now a `[[relation]]` row.
            "id = 2\nslug = \"a\"\ntitle = \"A\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsuperseded_by = [\"ADR-009\"]\n\
             tags = [\"layering\"]\n\
             [[relation]]\nlabel = \"supersedes\"\ntarget = \"ADR-001\"\n\
             [[relation]]\nlabel = \"related\"\ntarget = \"ADR-004\"\n",
        );
        write(&root, ".doctrine/adr/002/adr-002.md", "body\n");
        let edges = outbound_for(&root, kind_for("ADR"), 2).unwrap();
        assert_eq!(
            pairs(&edges),
            vec![
                (RelationLabel::Supersedes, "ADR-001"),
                (RelationLabel::Related, "ADR-004"),
            ],
            "governance emits supersedes + related ONLY (no superseded_by, no tags)"
        );
    }

    #[test]
    fn spec_outbound_lineage_members_interactions() {
        let dir = tmp();
        let root = dir.path();
        write(
            &root,
            ".doctrine/spec/tech/001/spec-001.toml",
            "id = 1\nslug = \"s\"\ntitle = \"S\"\nstatus = \"draft\"\nkind = \"tech\"\n\
             descends_from = \"PRD-005\"\nparent = \"SPEC-000\"\n",
        );
        write(&root, ".doctrine/spec/tech/001/spec-001.md", "b\n");
        write(
            &root,
            ".doctrine/spec/tech/001/members.toml",
            "[[member]]\nrequirement = \"REQ-009\"\nlabel = \"FR\"\norder = 1\n",
        );
        write(
            &root,
            ".doctrine/spec/tech/001/interactions.toml",
            "[[edge]]\ntarget = \"SPEC-002\"\ntype = \"calls\"\nnotes = \"sync\"\n",
        );
        let edges = outbound_for(&root, kind_for("SPEC"), 1).unwrap();
        assert_eq!(
            pairs(&edges),
            vec![
                (RelationLabel::DescendsFrom, "PRD-005"),
                (RelationLabel::Parent, "SPEC-000"),
                (RelationLabel::Members, "REQ-009"),
                (RelationLabel::Interactions, "SPEC-002"),
            ]
        );
    }

    #[test]
    fn product_spec_lineage_options_absent_emit_nothing() {
        let dir = tmp();
        let root = dir.path();
        // A product spec has no descends_from/parent and no interactions.toml.
        write(
            &root,
            ".doctrine/spec/product/003/spec-003.toml",
            "id = 3\nslug = \"p\"\ntitle = \"P\"\nstatus = \"draft\"\nkind = \"product\"\n",
        );
        write(&root, ".doctrine/spec/product/003/spec-003.md", "b\n");
        write(&root, ".doctrine/spec/product/003/members.toml", "");
        let edges = outbound_for(&root, kind_for("PRD"), 3).unwrap();
        assert!(
            edges.is_empty(),
            "absent Options + empty members emit nothing"
        );
    }

    #[test]
    fn backlog_outbound_slices_specs_drift_only() {
        let dir = tmp();
        let root = dir.path();
        // Every axis populated — only slices/specs/drift must emit (not
        // needs/after/triggers).
        write(
            &root,
            ".doctrine/backlog/issue/001/backlog-001.toml",
            // SL-048 PHASE-04: slices/specs/drift migrated to `[[relation]]`; the typed
            // `needs` axis stays in a `[relationships]` table preceding the arrays (F1).
            "id = 1\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nneeds = [\"ISS-002\"]\n\
             [[relation]]\nlabel = \"slices\"\ntarget = \"SL-020\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"concerns\"\ntarget = \"PRD-009\"\n\
             [[relation]]\nlabel = \"drift\"\ntarget = \"some-free-text\"\n",
        );
        write(&root, ".doctrine/backlog/issue/001/backlog-001.md", "b\n");
        let edges = outbound_for(&root, kind_for("ISS"), 1).unwrap();
        // SL-048 PHASE-04 (X1): `read_block` emits in canonical RELATION_RULES order —
        // specs (pos 0) precedes slices (pos 10) precedes drift (pos 14). The former
        // hardcoded accessor order (slices, specs, drift) is replaced; no render golden
        // depends on the raw accessor order (inspect regroups by enum Ord, which is the
        // same specs<slices<drift; format_show/show_json keep their own literal order).
        assert_eq!(
            pairs(&edges),
            vec![
                (RelationLabel::References, "PRD-009"),
                (RelationLabel::Slices, "SL-020"),
                (RelationLabel::Drift, "some-free-text"),
            ],
            "backlog emits references/slices/drift ONLY (no needs/after/triggers), canonical order"
        );
    }

    #[test]
    fn review_outbound_single_reviews_edge() {
        let dir = tmp();
        let root = dir.path();
        write(
            &root,
            ".doctrine/review/001/review-001.toml",
            "id = 1\nslug = \"r\"\ntitle = \"R\"\n\
             [review]\nfacet = \"reconciliation\"\nraiser = \"a\"\nresponder = \"b\"\n\
             [target]\nref = \"SL-046\"\n",
        );
        let edges = outbound_for(&root, kind_for("RV"), 1).unwrap();
        assert_eq!(pairs(&edges), vec![(RelationLabel::Reviews, "SL-046")]);
    }

    #[test]
    fn rec_outbound_owning_slice_and_decision_ref() {
        let dir = tmp();
        let root = dir.path();
        write(
            &root,
            ".doctrine/rec/001/rec-001.toml",
            "id = 1\nslug = \"r\"\ntitle = \"R\"\n\
             [rec]\nmove = \"accept\"\nowning_slice = \"SL-046\"\ndecision_ref = \"DEC-005-C\"\n",
        );
        let edges = outbound_for(&root, kind_for("REC"), 1).unwrap();
        assert_eq!(
            pairs(&edges),
            vec![
                (RelationLabel::OwningSlice, "SL-046"),
                (RelationLabel::DecisionRef, "DEC-005-C"),
            ]
        );
    }

    #[test]
    fn requirement_authors_no_outbound() {
        let dir = tmp();
        let root = dir.path();
        // REQ is an edge target only; the dispatch returns empty without touching disk.
        let edges = outbound_for(&root, kind_for("REQ"), 1).unwrap();
        assert!(edges.is_empty());
    }

    // -- SL-066 G3/G2: the REV arms land WITH the KINDS row -------------------

    #[test]
    fn revision_outbound_arm_reads_change_rows() {
        // G3: a REV row in KINDS routes to `revision::relation_edges` BEFORE the
        // `debug_assert!(false)` fallthrough. The accessor reads the `[[change]]`
        // payload (PHASE-03), projecting each row to one `Revises` edge — a REV with
        // no rows authors none.
        let dir = tmp();
        let root = dir.path();
        // No `[[change]]` rows → no outbound edges.
        write(
            &root,
            ".doctrine/revision/001/revision-001.toml",
            "id = 1\nslug = \"r\"\ntitle = \"R\"\nstatus = \"proposed\"\napproval = \"none\"\n",
        );
        let empty = outbound_for(&root, kind_for("REV"), 1).unwrap();
        assert!(
            empty.is_empty(),
            "a REV with no change rows authors no outbound"
        );

        // A `[[change]]` row projects to one `revises` edge to its target.
        write(
            &root,
            ".doctrine/revision/002/revision-002.toml",
            "id = 2\nslug = \"r\"\ntitle = \"R\"\nstatus = \"proposed\"\napproval = \"none\"\n\
             [[change]]\ntarget = \"ADR-006\"\naction = \"modify\"\nprimary = true\n",
        );
        let edges = outbound_for(&root, kind_for("REV"), 2).unwrap();
        assert_eq!(edges.len(), 1, "one change row → one revises edge");
        assert_eq!(edges[0].label, RelationLabel::Revises);
        assert_eq!(edges[0].target, "ADR-006");
    }

    #[test]
    fn revision_dep_seq_arm_reads_its_own_toml() {
        // G2: REV-as-source `needs`/`after` route to the leaf `dep_seq::read` over
        // `revision-NNN.toml` (mirrors the SL arm), not the empty short-circuit.
        let dir = tmp();
        let root = dir.path();
        write(
            &root,
            ".doctrine/revision/001/revision-001.toml",
            "id = 1\nslug = \"r\"\ntitle = \"R\"\nstatus = \"proposed\"\napproval = \"none\"\n\
             [relationships]\nneeds = [\"SL-046\"]\nafter = []\n",
        );
        let (ds, promoted) = dep_seq_for(&root, kind_for("REV"), 1).unwrap();
        assert_eq!(ds.needs, vec!["SL-046"], "REV needs reach the blocker view");
        assert!(!promoted, "REV carries no backlog-only promoted projection");
    }

    // -- SL-059 VT-1: outbound_for total-dispatch for the six knowledge kinds --

    #[test]
    fn knowledge_kinds_author_outbound_edges() {
        let dir = tmp();
        let root = dir.path();
        // Seed an assumption record with [[relation]] rows
        write(
            &root,
            ".doctrine/knowledge/assumption/001/record-001.toml",
            &format!(
                "schema = \"{SCHEMA_KNOWLEDGE}\"\nversion = 1\n\n\
             id = 1\nslug = \"a\"\ntitle = \"A\"\n\
             record_kind = \"assumption\"\nstatus = \"held\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             tags = []\n\n\
             [facet]\n\
             claim = \"\"\nconfidence = \"\"\nbasis = \"\"\n\
             validation_plan = \"\"\nvalidated_by = \"\"\nvalidated_on = \"\"\n\
             invalidated_by = \"\"\ninvalidated_on = \"\"\n\n\
             [evidence]\n\
             supports = []\ncontradicts = []\nnotes = []\n\
             [[relation]]\nlabel = \"shapes\"\ntarget = \"SL-001\"\n\
             [[relation]]\nlabel = \"spawns\"\ntarget = \"ISS-001\"\n\
             [[relation]]\nlabel = \"governed_by\"\ntarget = \"ADR-001\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"concerns\"\ntarget = \"SL-001\"\n"
            ),
        );
        write(
            &root,
            ".doctrine/knowledge/assumption/001/record-001.md",
            "body\n",
        );
        let edges = outbound_for(&root, kind_for("ASM"), 1).unwrap();
        assert_eq!(edges.len(), 4);
        // Verify each edge exists with correct target
        assert!(
            edges
                .iter()
                .any(|e| e.label == RelationLabel::Shapes && e.target == "SL-001")
        );
        assert!(
            edges
                .iter()
                .any(|e| e.label == RelationLabel::Spawns && e.target == "ISS-001")
        );
        assert!(
            edges
                .iter()
                .any(|e| e.label == RelationLabel::GovernedBy && e.target == "ADR-001")
        );
        assert!(edges.iter().any(|e| e.label == RelationLabel::References
            && e.role == Some(crate::relation::Role::Concerns)
            && e.target == "SL-001"));
    }

    // -- SL-059 VT-2: scan-side totality (F-A7, the L7 partner) ---------------

    #[test]
    fn knowledge_rows_present_but_no_record_tree_leaves_the_graph_unchanged() {
        let dir = tmp();
        let root = dir.path();
        // A fixture with an ordinary entity but NO knowledge trees. The four KINDS
        // rows are present, so `scan_entities` visits the (absent) record dirs — it
        // is benign only because `entity::scan_ids` returns Ok(vec![]) on a missing
        // dir. The scan returns exactly the pre-existing entity; the four record
        // kinds contribute nothing. Regression tripwire if `scan_ids` is made strict.
        write(
            &root,
            ".doctrine/requirement/001/requirement-001.toml",
            "id = 1\nslug = \"r\"\ntitle = \"R\"\nstatus = \"active\"\n",
        );
        write(&root, ".doctrine/requirement/001/requirement-001.md", "b\n");
        let scanned = scan_entities(&root, &mut vec![], ScanMode::default()).unwrap();
        let keys: Vec<_> = scanned.iter().map(|e| e.key.canonical()).collect();
        assert_eq!(keys, vec!["REQ-001"], "no record kind contributes a node");
    }

    // -- VT-2 exclusion proof (REC decision_ref carried, not dropped) --------

    #[test]
    fn rec_decision_ref_carried_as_free_text_not_dropped() {
        let dir = tmp();
        let root = dir.path();
        write(
            &root,
            ".doctrine/rec/002/rec-002.toml",
            "id = 2\nslug = \"r\"\ntitle = \"R\"\n\
             [rec]\nmove = \"accept\"\ndecision_ref = \"DEC-001-A\"\n",
        );
        let edges = outbound_for(&root, kind_for("REC"), 2).unwrap();
        // decision_ref survives even with no owning_slice — carried, will dangle.
        assert_eq!(
            pairs(&edges),
            vec![(RelationLabel::DecisionRef, "DEC-001-A")]
        );
    }

    // -- SL-060 PHASE-04: dep_seq_for cross-kind dispatch --------------------

    #[test]
    fn dep_seq_for_slice_arm_reads_needs_after_promoted_false() {
        let dir = tmp();
        let root = dir.path();
        // A slice authoring both dep/seq axes — the slice arm reads its own toml via the
        // leaf; `promoted` is always false (only a backlog item carries that projection).
        write(
            &root,
            ".doctrine/slice/001/slice-001.toml",
            "id = 1\nslug = \"a\"\ntitle = \"A\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nneeds = [\"SL-002\"]\n\
             after = [{ to = \"SL-003\", rank = 4 }]\n",
        );
        write(&root, ".doctrine/slice/001/slice-001.md", "scope\n");
        let (ds, promoted) = dep_seq_for(&root, kind_for("SL"), 1).unwrap();
        assert_eq!(ds.needs, vec!["SL-002"]);
        assert_eq!(
            ds.after,
            vec![dep_seq::AfterEdge {
                to: "SL-003".to_string(),
                rank: 4,
            }]
        );
        assert!(!promoted, "a slice is never promoted");
    }

    #[test]
    fn dep_seq_for_backlog_arm_one_parse_carries_promoted() {
        let dir = tmp();
        let root = dir.path();
        // A promoted backlog issue authoring dep/seq — the backlog arm routes to backlog's
        // single `dep_seq_for` (ONE parse), adapting its `(to, rank)` pairs to AfterEdge
        // and carrying `resolution == promoted` through.
        write(
            &root,
            ".doctrine/backlog/issue/001/backlog-001.toml",
            "id = 1\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"resolved\"\n\
             resolution = \"promoted\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nneeds = [\"ISS-002\"]\n\
             after = [{ to = \"RSK-001\", rank = 2 }]\n",
        );
        write(&root, ".doctrine/backlog/issue/001/backlog-001.md", "b\n");
        let (ds, promoted) = dep_seq_for(&root, kind_for("ISS"), 1).unwrap();
        assert_eq!(ds.needs, vec!["ISS-002"]);
        assert_eq!(
            ds.after,
            vec![dep_seq::AfterEdge {
                to: "RSK-001".to_string(),
                rank: 2,
            }]
        );
        assert!(
            promoted,
            "resolution=promoted carried through the backlog arm"
        );
    }

    #[test]
    fn dep_seq_for_non_authoring_kind_short_circuits_before_any_read() {
        let dir = tmp();
        let root = dir.path();
        // VT-4 no-read probe (design F5): a non-authoring kind (ADR) whose on-disk toml is
        // ABSENT. The dispatch must return an empty DepSeq WITHOUT error — proving the kind
        // is tested BEFORE any path is built or any toml is touched. (If the arm read disk
        // first it would fail to open the missing ADR-001 toml.) `promoted` is false.
        let (ds, promoted) = dep_seq_for(&root, kind_for("ADR"), 1).unwrap();
        assert_eq!(
            ds,
            dep_seq::DepSeq::default(),
            "non-authoring kind yields empty dep/seq with no disk read"
        );
        assert!(!promoted);

        // Stronger probe: a GARBAGE toml on disk for a non-authoring kind is never opened —
        // a read arm would choke on the malformed TOML; the short-circuit ignores it.
        write(
            &root,
            ".doctrine/requirement/001/requirement-001.toml",
            "this is not valid toml at all = = =\n",
        );
        let (ds2, _promoted2) = dep_seq_for(&root, kind_for("REQ"), 1).unwrap();
        assert_eq!(
            ds2,
            dep_seq::DepSeq::default(),
            "garbage toml for a non-authoring kind is never read → still empty"
        );
    }

    // -- VT-3 interactions collapse to a single `Interactions` class ---------
    // (The per-edge free-text `type` round-trips from the SOURCE `Interaction`
    //  struct — asserted in spec.rs where the reader + struct are visible.)

    #[test]
    fn interactions_collapse_to_single_class_label() {
        let dir = tmp();
        let root = dir.path();
        write(
            &root,
            ".doctrine/spec/tech/004/spec-004.toml",
            "id = 4\nslug = \"s\"\ntitle = \"S\"\nstatus = \"draft\"\nkind = \"tech\"\n",
        );
        write(&root, ".doctrine/spec/tech/004/spec-004.md", "b\n");
        write(&root, ".doctrine/spec/tech/004/members.toml", "");
        write(
            &root,
            ".doctrine/spec/tech/004/interactions.toml",
            "[[edge]]\ntarget = \"SPEC-009\"\ntype = \"depends-on\"\nnotes = \"n\"\n\
             [[edge]]\ntarget = \"SPEC-010\"\ntype = \"calls\"\n",
        );
        // Two interactions with different free-text types share ONE label class; the
        // type is NOT encoded in the label (re-read at render — C2).
        let edges = outbound_for(&root, kind_for("SPEC"), 4).unwrap();
        assert_eq!(
            pairs(&edges),
            vec![
                (RelationLabel::Interactions, "SPEC-009"),
                (RelationLabel::Interactions, "SPEC-010"),
            ]
        );
    }

    // -- PHASE-03 inspect query ---------------------------------------------

    /// All inbound targets under `label` (any role) in a view (sorted-render order).
    /// SL-149: the view key is `(label, role)`; these label-only helpers match the
    /// label component and flatten across roles, sufficient for the label-only fixtures.
    fn inbound_for(view: &InspectView, label: RelationLabel) -> Vec<&str> {
        view.inbound
            .iter()
            .filter(|((l, _), _)| *l == label)
            .flat_map(|(_, v)| v.iter().map(String::as_str))
            .collect()
    }

    /// All outbound targets under `label` (any role) in a view.
    fn outbound_targets(view: &InspectView, label: RelationLabel) -> Vec<&str> {
        view.outbound
            .iter()
            .filter(|((l, _), _)| *l == label)
            .flat_map(|(_, v)| v.iter().map(String::as_str))
            .collect()
    }

    /// A minimal slice toml with the given relations, in the SL-048 migrated shape
    /// (`axes` → `[relationships]` typed leftovers then `[[relation]]` rows).
    fn slice_toml(id: u32, axes: &[(&str, &[&str])]) -> String {
        format!(
            "id = {id}\nslug = \"s\"\ntitle = \"S\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{}",
            crate::relation::rels_block(kind_for("SL"), axes)
        )
    }

    /// Seed a slice entity (toml + md) under `root`.
    fn seed_slice(root: &Path, id: u32, axes: &[(&str, &[&str])]) {
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
            &slice_toml(id, axes),
        );
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.md"),
            "scope\n",
        );
    }

    /// Seed an ADR governance entity (SL-048 migrated shape: `related` → `[[relation]]`;
    /// `supersedes`/`superseded_by` stay typed; `tags` → root-level (SL-136)).
    fn seed_adr(root: &Path, id: u32, axes: &[(&str, &[&str])]) {
        write(
            root,
            &format!(".doctrine/adr/{id:03}/adr-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"a\"\ntitle = \"A\"\nstatus = \"accepted\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{}",
                crate::relation::rels_block(kind_for("ADR"), axes)
            ),
        );
        write(
            root,
            &format!(".doctrine/adr/{id:03}/adr-{id:03}.md"),
            "body\n",
        );
    }

    // VT-1 — derived inbound correctness over a seeded multi-kind corpus, incl.
    // the supersedes reciprocal. Structural proof: NO stored reverse field is read
    // (the predecessor authors no `superseded_by`; inbound is derived from the
    // successor's outbound `supersedes` via in_edges — ADR-004 §3 / REQ-074).
    #[test]
    fn inbound_derived_from_in_edges_including_supersedes_reciprocal() {
        let dir = tmp();
        let root = dir.path();
        // SL-002 supersedes SL-001 and requires REQ-005; SL-001 authors nothing.
        seed_slice(&root, 1, &[]);
        seed_slice(
            &root,
            2,
            &[
                ("references(implements)", &["REQ-005"]),
                ("supersedes", &["SL-001"]),
            ],
        );
        // REQ-005 is an edge target only (no outbound).
        write(
            &root,
            ".doctrine/requirement/005/requirement-005.toml",
            "id = 5\nslug = \"r\"\ntitle = \"R\"\nstatus = \"active\"\n",
        );
        write(&root, ".doctrine/requirement/005/requirement-005.md", "r\n");

        // SL-001's only inbound is the derived "superseded by" from SL-002.
        let pred = inspect(&root, "SL-001").unwrap();
        assert_eq!(pred.id, "SL-001");
        assert!(pred.outbound.is_empty(), "predecessor authors no outbound");
        assert_eq!(
            inbound_for(&pred, RelationLabel::Supersedes),
            vec!["SL-002"],
            "supersedes-overlay inbound is the derived reciprocal (renders 'superseded by')"
        );

        // REQ-005's only inbound is the references(implements) edge from SL-002.
        let req = inspect(&root, "REQ-005").unwrap();
        assert_eq!(inbound_for(&req, RelationLabel::References), vec!["SL-002"]);

        // SL-002 owns the outbound; it has no inbound.
        let succ = inspect(&root, "SL-002").unwrap();
        assert_eq!(
            outbound_targets(&succ, RelationLabel::Supersedes),
            vec!["SL-001"]
        );
        assert!(succ.inbound.is_empty(), "successor has no inbound");
    }

    // VT-2 / C3 — two authored rows sharing (label, src, dst) surface as ONE
    // inbound edge, no panic. Asserted at the projection boundary: the duplicate
    // collapses in cordage's BTreeSet<Edge> (EdgeAttrs(0,0)).
    #[test]
    fn duplicate_authored_ref_collapses_to_single_inbound_no_panic() {
        let dir = tmp();
        let root = dir.path();
        // SL-002 lists SL-001 twice under supersedes (an authoring duplicate).
        seed_slice(&root, 1, &[]);
        seed_slice(&root, 2, &[("supersedes", &["SL-001", "SL-001"])]);
        let view = inspect(&root, "SL-001").unwrap();
        assert_eq!(
            inbound_for(&view, RelationLabel::Supersedes),
            vec!["SL-002"],
            "two identical (label,src,dst) rows collapse to one inbound edge"
        );
    }

    // VT-3 / C5 — out-of-order planted entity dirs yield identical output: the
    // ascending sort after scan_ids makes mint + render permutation-invariant
    // (REQ-077). We seed the same corpus and assert the view is stable regardless
    // of how many supersedors target SL-001 (their canonical-ref render order is
    // independent of NodeId mint order).
    #[test]
    fn inbound_render_is_permutation_invariant() {
        let dir = tmp();
        let root = dir.path();
        // Three supersedors of SL-001, planted out of id order on disk; scan_ids is
        // read_dir order (unsorted), so the only thing making the render stable is
        // the ascending sort + the EntityKey sort in inspect.
        seed_slice(&root, 1, &[]);
        seed_slice(&root, 4, &[("supersedes", &["SL-001"])]);
        seed_slice(&root, 2, &[("supersedes", &["SL-001"])]);
        seed_slice(&root, 3, &[("supersedes", &["SL-001"])]);
        let view = inspect(&root, "SL-001").unwrap();
        assert_eq!(
            inbound_for(&view, RelationLabel::Supersedes),
            vec!["SL-002", "SL-003", "SL-004"],
            "inbound renders in ascending canonical-ref order, not filesystem order"
        );

        // RSK-007: same-prefix ids ≥ 1000 must sort numerically, not lexically.
        // Plant SL-998, SL-999, SL-1000, SL-1001 out-of-order as supersedors of
        // SL-001. Lexical sort would give ["SL-1000","SL-1001","SL-0998","SL-0999"].
        let dir2 = tmp();
        let root2 = dir2.path();
        seed_slice(&root2, 1, &[]);
        seed_slice(&root2, 1001, &[("supersedes", &["SL-001"])]);
        seed_slice(&root2, 998, &[("supersedes", &["SL-001"])]);
        seed_slice(&root2, 1000, &[("supersedes", &["SL-001"])]);
        seed_slice(&root2, 999, &[("supersedes", &["SL-001"])]);
        let view2 = inspect(&root2, "SL-001").unwrap();
        assert_eq!(
            inbound_for(&view2, RelationLabel::Supersedes),
            vec!["SL-998", "SL-999", "SL-1000", "SL-1001"],
            "inbound sort is numeric-within-prefix, not lexical (RSK-007)"
        );
    }

    // VT-4 / C8/R3 — a stored `superseded_by` with NO reciprocal `supersedes`
    // produces NO inbound. The reader projects only the outbound `supersedes`; the
    // stored reverse field is never read (ADR-004 §5 carve-out, but §3 derivation).
    #[test]
    fn stored_superseded_by_without_reciprocal_yields_no_inbound() {
        let dir = tmp();
        let root = dir.path();
        // ADR-002 carries a stored superseded_by = ADR-009 but NO entity authors
        // `supersedes = [ADR-002]`. ADR-009 exists but supersedes nothing.
        seed_adr(&root, 2, &[("superseded_by", &["ADR-009"])]);
        seed_adr(&root, 9, &[]);
        let view = inspect(&root, "ADR-002").unwrap();
        assert!(
            view.inbound.is_empty(),
            "a lone stored superseded_by produces no derived inbound"
        );
        // And ADR-009 has no inbound from ADR-002 either (no reciprocal supersedes).
        let nine = inspect(&root, "ADR-009").unwrap();
        assert!(nine.inbound.is_empty());
    }

    // VT-5 / R4 — free-text / dangling targets surface as danglers, never panic;
    // the NNN-slug symlink is skipped (scan_ids ignores non-dirs); an entity with
    // no relations yields empty sections, not an error.
    #[test]
    fn dangling_and_free_text_targets_surface_as_danglers() {
        let dir = tmp();
        let root = dir.path();
        // A backlog issue with a free-text drift, an unresolved slice ref, and a
        // resolvable slice ref. drift → dangler (no DRIFT kind); SL-099 → dangler
        // (no such entity); SL-001 → a real edge.
        seed_slice(&root, 1, &[]);
        write(
            &root,
            ".doctrine/backlog/issue/001/backlog-001.toml",
            // SL-048 PHASE-04: slices/drift migrated to `[[relation]]` rows.
            "id = 1\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [[relation]]\nlabel = \"slices\"\ntarget = \"SL-001\"\n\
             [[relation]]\nlabel = \"slices\"\ntarget = \"SL-099\"\n\
             [[relation]]\nlabel = \"drift\"\ntarget = \"some-free-text\"\n",
        );
        write(&root, ".doctrine/backlog/issue/001/backlog-001.md", "b\n");
        let view = inspect(&root, "ISS-001").unwrap();
        // The resolvable slice edge is NOT a dangler.
        assert_eq!(
            outbound_targets(&view, RelationLabel::Slices),
            vec!["SL-001", "SL-099"],
            "outbound lists every authored target regardless of resolution"
        );
        // Danglers: the unresolved SL-099 and the free-text drift.
        assert!(
            view.danglers
                .contains(&(RelationLabel::Slices, "SL-099".to_string())),
            "an unresolved canonical ref dangles"
        );
        assert!(
            view.danglers
                .contains(&(RelationLabel::Drift, "some-free-text".to_string())),
            "a free-text drift target dangles (no DRIFT kind / overlay)"
        );

        // VT-5 — NNN-slug symlink is skipped: plant one beside SL-001 and confirm
        // it neither mints a node nor breaks the scan.
        std::os::unix::fs::symlink("001", root.join(".doctrine/slice/a-slug")).unwrap();
        let still = inspect(&root, "ISS-001").unwrap();
        assert_eq!(
            outbound_targets(&still, RelationLabel::Slices),
            vec!["SL-001", "SL-099"]
        );

        // VT-5 — an entity with no relations: empty sections, not an error.
        let empty = inspect(&root, "SL-001").unwrap();
        // SL-001 is referenced by ISS-001's slices edge → it DOES have inbound;
        // a freshly-isolated no-relation entity proves the empty path instead.
        seed_slice(&root, 50, &[]);
        let lone = inspect(&root, "SL-050").unwrap();
        assert!(lone.outbound.is_empty());
        assert!(lone.inbound.is_empty());
        assert!(lone.danglers.is_empty());
        // (SL-001 has the inbound slices edge — sanity that inspect saw it.)
        assert_eq!(inbound_for(&empty, RelationLabel::Slices), vec!["ISS-001"]);
    }

    // SL-050 F6 — a well-formed ref to a never-minted id is now an ERROR (flips the old
    // empty-view half); the exact message is `KIND-NNN: no such entity`.
    #[test]
    fn nonexistent_id_is_no_such_entity_error() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(&root, 1, &[]);
        // Well-formed ref, no such entity → the existence gate errors (not an empty view).
        let err = inspect(&root, "SL-999").unwrap_err();
        assert_eq!(
            err.to_string(),
            "SL-999: no such entity",
            "the exact existence-gate message"
        );
    }

    // An unknown prefix is a clean error (not a panic) — the parse-classification path,
    // unchanged by the F6 existence gate (it fails before the scan).
    #[test]
    fn unknown_prefix_clean_error() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(&root, 1, &[]);
        let err = inspect(&root, "ZZZ-001").unwrap_err();
        assert!(
            err.to_string().contains("ZZZ"),
            "unknown prefix surfaces a clean error mentioning the prefix"
        );
    }

    // -- PHASE-03 VT-1: table-driven overlay coverage (R2-M4) ---------------

    /// VT-1 (R2-M4): the overlay-backed label set, the resolvable-graph label set, and
    /// the table's distinct non-`Unvalidated` labels are the SAME set. Asserted by the
    /// PROPERTY — both expectations are derived from `RELATION_RULES` (the single
    /// source), NOT from a deleted parallel const, so it cannot be a tautology against
    /// the implementation. A real `GraphBuilder` is driven so the assertion is over the
    /// actually-allocated overlays (`by_label` keys), not a re-derivation.
    #[test]
    fn overlay_set_equals_resolvable_graph_labels_table_driven() {
        use crate::relation::{RELATION_RULES, TargetSpec};
        use std::collections::BTreeSet;

        // Side A — every distinct label the table marks resolvable (TargetSpec !=
        // Unvalidated). Derived from the table, NOT a hardcoded list.
        let resolvable_from_table: BTreeSet<RelationLabel> = RELATION_RULES
            .iter()
            .filter(|r| !matches!(r.target, TargetSpec::Unvalidated))
            .map(|r| r.label)
            .collect();

        // Side B — the labels OverlayMap::build actually allocates an overlay for,
        // read off a real builder (the live allocation, not a re-derivation).
        let mut builder = GraphBuilder::new();
        let overlays = OverlayMap::build(&mut builder);
        let overlay_backed: BTreeSet<RelationLabel> = overlays.by_label.keys().copied().collect();

        assert_eq!(
            overlay_backed, resolvable_from_table,
            "the allocated overlay set must equal the table's resolvable (non-Unvalidated) labels"
        );

        // And the complement is EXACTLY the Unvalidated no-overlay pair — overlay_for
        // returns None for those and only those.
        let unvalidated: BTreeSet<RelationLabel> = RELATION_RULES
            .iter()
            .filter(|r| matches!(r.target, TargetSpec::Unvalidated))
            .map(|r| r.label)
            .collect();
        assert_eq!(
            unvalidated,
            BTreeSet::from([
                RelationLabel::Contextualizes,
                RelationLabel::Drift,
                RelationLabel::DecisionRef,
            ]),
            "the no-overlay set is exactly contextualizes + drift + decision_ref"
        );
        for label in [
            RelationLabel::Contextualizes,
            RelationLabel::Drift,
            RelationLabel::DecisionRef,
        ] {
            assert!(
                overlays.overlay_for(label).is_none(),
                "{label:?} (Unvalidated) must have no overlay"
            );
        }
        // The 18 = 21 distinct labels minus the 3 Unvalidated (SL-149 PHASE-05 retired
        // specs/requirements, collapsing them into the single resolvable `references`
        // label → one overlay, label-keyed per R5; SL-159 PHASE-02 added supports/disputes).
        // The set, not just the count, is the real assertion above; the count is a sanity tag.
        assert_eq!(overlay_backed.len(), 18, "overlay-backed label count is 18");
    }

    // -- PHASE-04 VT-4 / X3 arm (a): exact reader coverage (read_block live) ---

    /// The distinct `(label, role)` keys `RELATION_RULES` legalises for a given source
    /// prefix (SL-149 PHASE-03: role-aware). A label-only row contributes `(label, None)`;
    /// a `references` row contributes one key per legal role (`(References, Some(role))`).
    /// The exact-coverage invariant now spans the role dimension — the P2 `references`
    /// placeholder filter is gone.
    fn table_labels_for(
        prefix: &str,
    ) -> std::collections::BTreeSet<(RelationLabel, Option<crate::relation::Role>)> {
        use crate::relation::RELATION_RULES;
        RELATION_RULES
            .iter()
            .filter(|r| r.sources.iter().any(|k| *k == prefix))
            .map(|r| (r.label, r.role))
            .collect()
    }

    /// The distinct `(label, role)` keys a kind's live `outbound_for` accessor ACTUALLY
    /// emits over a corpus where every legal axis is authored — read off `RelationEdge`'s
    /// `(label, role)` (SL-149: the reader now threads role through the storage seam).
    fn emitted_labels(
        root: &Path,
        prefix: &str,
        id: u32,
    ) -> std::collections::BTreeSet<(RelationLabel, Option<crate::relation::Role>)> {
        outbound_for(root, kind_for(prefix), id)
            .unwrap()
            .iter()
            .map(|e| (e.label, e.role))
            .collect()
    }

    /// VT-1 (SL-149: exact-coverage, now per-`(label, role)`): per source kind, the
    /// `(label, role)` set the shipped `relation_edges` accessor EMITS == the `(label,
    /// role)` set `RELATION_RULES` legalises for that source — no off-table emission, no
    /// table rule without a reader path. The role dimension is in scope: the fully-populated
    /// fixture authors one edge of every legal `(label, role)` (`references` once per role),
    /// and the reader threads the role off the `[[relation]] role` cell. The exact set (not
    /// ⊆) is the assertion: a fully-populated fixture authors one edge of every legal axis
    /// (tier-1 via `[[relation]]`, tier-2/3 via its typed
    /// structure), and the emitted distinct-label set must equal the table's.
    #[test]
    fn reader_emitted_labels_equal_table_labels_per_source() {
        let dir = tmp();
        let root = dir.path();

        // --- SL: specs, requirements, supersedes, governed_by, related (all tier-1),
        //     plus references in all three roles (SL-149: implements/scoped_from/concerns
        //     — the SL source authors every references role) ---
        write(
            &root,
            ".doctrine/slice/001/slice-001.toml",
            "id = 1\nslug = \"s\"\ntitle = \"S\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [[relation]]\nlabel = \"specs\"\ntarget = \"PRD-010\"\n\
             [[relation]]\nlabel = \"requirements\"\ntarget = \"REQ-001\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"SPEC-018\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"scoped_from\"\ntarget = \"IMP-001\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"concerns\"\ntarget = \"RFC-003\"\n\
             [[relation]]\nlabel = \"supersedes\"\ntarget = \"SL-002\"\n\
             [[relation]]\nlabel = \"governed_by\"\ntarget = \"ADR-001\"\n\
             [[relation]]\nlabel = \"related\"\ntarget = \"ADR-010\"\n",
        );
        write(&root, ".doctrine/slice/001/slice-001.md", "s\n");
        assert_eq!(
            emitted_labels(root, "SL", 1),
            table_labels_for("SL"),
            "slice reader emits exactly its table labels"
        );

        // --- ADR (governance): supersedes + related (both tier-1 after SL-095) ---
        write(
            &root,
            ".doctrine/adr/001/adr-001.toml",
            "id = 1\nslug = \"a\"\ntitle = \"A\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\ntags = []\n\
             [relationships]\nsuperseded_by = []\n\
             [[relation]]\nlabel = \"supersedes\"\ntarget = \"ADR-002\"\n\
             [[relation]]\nlabel = \"related\"\ntarget = \"ADR-003\"\n",
        );
        write(&root, ".doctrine/adr/001/adr-001.md", "a\n");
        assert_eq!(
            emitted_labels(root, "ADR", 1),
            table_labels_for("ADR"),
            "governance reader emits exactly supersedes + related"
        );

        // --- ISS (backlog): specs + slices + related + drift + governed_by (all tier-1;
        //     governed_by + related widened in for backlog by SL-145) + references(concerns)
        //     (SL-149: a backlog item authors ONLY the concerns role — implements/scoped_from
        //     are SL-only) ---
        write(
            &root,
            ".doctrine/backlog/issue/001/backlog-001.toml",
            "id = 1\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [[relation]]\nlabel = \"specs\"\ntarget = \"PRD-010\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"concerns\"\ntarget = \"SL-001\"\n\
             [[relation]]\nlabel = \"slices\"\ntarget = \"SL-001\"\n\
             [[relation]]\nlabel = \"related\"\ntarget = \"ADR-010\"\n\
             [[relation]]\nlabel = \"governed_by\"\ntarget = \"ADR-001\"\n\
             [[relation]]\nlabel = \"drift\"\ntarget = \"free-text\"\n",
        );
        write(&root, ".doctrine/backlog/issue/001/backlog-001.md", "i\n");
        assert_eq!(
            emitted_labels(root, "ISS", 1),
            table_labels_for("ISS"),
            "backlog reader emits exactly specs + slices + related + governed_by + drift"
        );

        // --- SPEC (tech): governed_by (tier-1) + descends_from/parent (typed) +
        //     members (members.toml) + interactions (interactions.toml) ---
        write(
            &root,
            ".doctrine/spec/tech/001/spec-001.toml",
            "id = 1\nslug = \"sp\"\ntitle = \"SP\"\nstatus = \"draft\"\nkind = \"tech\"\n\
             descends_from = \"PRD-010\"\nparent = \"SPEC-002\"\n\
             [[relation]]\nlabel = \"governed_by\"\ntarget = \"ADR-001\"\n",
        );
        write(&root, ".doctrine/spec/tech/001/spec-001.md", "sp\n");
        write(
            &root,
            ".doctrine/spec/tech/001/members.toml",
            "[[member]]\nlabel = \"M\"\norder = 0\nrequirement = \"REQ-001\"\n",
        );
        write(
            &root,
            ".doctrine/spec/tech/001/interactions.toml",
            "[[edge]]\ntarget = \"SPEC-003\"\ntype = \"calls\"\nnotes = \"\"\n",
        );
        assert_eq!(
            emitted_labels(root, "SPEC", 1),
            table_labels_for("SPEC"),
            "tech spec reader emits governed_by + descends_from + parent + members + interactions"
        );

        // --- PRD (product): governed_by + consumes (tier-1) + members (members.toml) ---
        write(
            &root,
            ".doctrine/spec/product/001/spec-001.toml",
            "id = 1\nslug = \"pr\"\ntitle = \"PR\"\nstatus = \"draft\"\nkind = \"product\"\n\
             parent = \"PRD-002\"\n\
             [[relation]]\nlabel = \"governed_by\"\ntarget = \"ADR-001\"\n\
             [[relation]]\nlabel = \"consumes\"\ntarget = \"PRD-002\"\n",
        );
        write(&root, ".doctrine/spec/product/001/spec-001.md", "pr\n");
        write(
            &root,
            ".doctrine/spec/product/001/members.toml",
            "[[member]]\nlabel = \"M\"\norder = 0\nrequirement = \"REQ-001\"\n",
        );
        assert_eq!(
            emitted_labels(root, "PRD", 1),
            table_labels_for("PRD"),
            "product spec reader emits governed_by + consumes + members"
        );

        // --- RV: reviews (the [target].ref) ---
        write(
            &root,
            ".doctrine/review/001/review-001.toml",
            "id = 1\nslug = \"r\"\ntitle = \"R\"\n\
             [review]\nfacet = \"reconciliation\"\nraiser = \"a\"\nresponder = \"b\"\n\
             [target]\nref = \"SL-001\"\n",
        );
        assert_eq!(
            emitted_labels(root, "RV", 1),
            table_labels_for("RV"),
            "review reader emits exactly reviews"
        );

        // --- REC: owning_slice + decision_ref ---
        write(
            &root,
            ".doctrine/rec/001/rec-001.toml",
            "id = 1\nslug = \"r\"\ntitle = \"R\"\n\
             [rec]\nmove = \"accept\"\nowning_slice = \"SL-001\"\ndecision_ref = \"DEC-001-A\"\n",
        );
        assert_eq!(
            emitted_labels(root, "REC", 1),
            table_labels_for("REC"),
            "rec reader emits exactly owning_slice + decision_ref"
        );

        // --- ASM (knowledge): shapes + spawns + governed_by ---
        write(
            &root,
            ".doctrine/knowledge/assumption/001/record-001.toml",
            &format!(
                "schema = \"{SCHEMA_KNOWLEDGE}\"\nversion = 1\n\n\
             id = 1\nslug = \"a\"\ntitle = \"A\"\n\
             record_kind = \"assumption\"\nstatus = \"held\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             tags = []\n\n\
             [facet]\n\
             claim = \"\"\nconfidence = \"\"\nbasis = \"\"\n\
             validation_plan = \"\"\nvalidated_by = \"\"\nvalidated_on = \"\"\n\
             invalidated_by = \"\"\ninvalidated_on = \"\"\n\n\
             [evidence]\n\
             supports = []\ncontradicts = []\nnotes = []\n\
             [[relation]]\nlabel = \"shapes\"\ntarget = \"SL-001\"\n\
             [[relation]]\nlabel = \"spawns\"\ntarget = \"ISS-001\"\n\
             [[relation]]\nlabel = \"governed_by\"\ntarget = \"ADR-001\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"concerns\"\ntarget = \"SL-001\"\n"
            ),
        );
        write(
            &root,
            ".doctrine/knowledge/assumption/001/record-001.md",
            "body\n",
        );
        // RECORD kinds emit shapes + spawns + governed_by + references(concerns) via
        // Supersedes is LifecycleOnly (verb-writes to typed [relationships], not
        // authored in [[relation]]) — the typed parse lands in PHASE-03.
        // table_labels_for now includes Supersedes from RELATION_RULES, but
        // outbound_for won't emit it until the typed [relationships] block is
        // parsed, so we compare against the Writable subset.
        {
            let mut expected = table_labels_for("ASM");
            expected.remove(&(RelationLabel::Supersedes, None));
            assert_eq!(
                emitted_labels(root, "ASM", 1),
                expected,
                "ASM: shapes + spawns + governed_by + references(concerns) (supersedes is LifecycleOnly — typed parse in PHASE-03)"
            );
        }

        // --- DEC (knowledge): shapes + spawns + governed_by ---
        write(
            &root,
            ".doctrine/knowledge/decision/001/record-001.toml",
            &format!(
                "schema = \"{SCHEMA_KNOWLEDGE}\"\nversion = 1\n\n\
             id = 1\nslug = \"d\"\ntitle = \"D\"\n\
             record_kind = \"decision\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             tags = []\n\n\
             [facet]\n\
             context = \"\"\nchoice = \"\"\nalternatives = []\n\
             rationale = \"\"\nconsequences = []\n\
             decided_by = \"\"\ndecided_on = \"\"\n\n\
             [evidence]\n\
             supports = []\ncontradicts = []\nnotes = []\n\
             [[relation]]\nlabel = \"shapes\"\ntarget = \"SL-001\"\n\
             [[relation]]\nlabel = \"spawns\"\ntarget = \"ISS-001\"\n\
             [[relation]]\nlabel = \"governed_by\"\ntarget = \"ADR-001\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"concerns\"\ntarget = \"SL-001\"\n"
            ),
        );
        write(
            &root,
            ".doctrine/knowledge/decision/001/record-001.md",
            "body\n",
        );
        {
            let mut expected = table_labels_for("DEC");
            expected.remove(&(RelationLabel::Supersedes, None));
            assert_eq!(
                emitted_labels(root, "DEC", 1),
                expected,
                "DEC: shapes + spawns + governed_by + references(concerns) (supersedes is LifecycleOnly — typed parse in PHASE-03)"
            );
        }

        // --- QUE (knowledge): shapes + spawns + governed_by ---
        write(
            &root,
            ".doctrine/knowledge/question/001/record-001.toml",
            &format!(
                "schema = \"{SCHEMA_KNOWLEDGE}\"\nversion = 1\n\n\
             id = 1\nslug = \"q\"\ntitle = \"Q\"\n\
             record_kind = \"question\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             tags = []\n\n\
             [facet]\n\
             question = \"\"\nwhy_matters = \"\"\nanswer = \"\"\n\
             answered_by = \"\"\nanswered_on = \"\"\n\n\
             [evidence]\n\
             supports = []\ncontradicts = []\nnotes = []\n\
             [[relation]]\nlabel = \"shapes\"\ntarget = \"SL-001\"\n\
             [[relation]]\nlabel = \"spawns\"\ntarget = \"ISS-001\"\n\
             [[relation]]\nlabel = \"governed_by\"\ntarget = \"ADR-001\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"concerns\"\ntarget = \"SL-001\"\n"
            ),
        );
        write(
            &root,
            ".doctrine/knowledge/question/001/record-001.md",
            "body\n",
        );
        {
            let mut expected = table_labels_for("QUE");
            expected.remove(&(RelationLabel::Supersedes, None));
            assert_eq!(
                emitted_labels(root, "QUE", 1),
                expected,
                "QUE: shapes + spawns + governed_by + references(concerns) (supersedes is LifecycleOnly — typed parse in PHASE-03)"
            );
        }

        // --- CON (knowledge): shapes + spawns + governed_by ---
        write(
            &root,
            ".doctrine/knowledge/constraint/001/record-001.toml",
            &format!(
                "schema = \"{SCHEMA_KNOWLEDGE}\"\nversion = 1\n\n\
             id = 1\nslug = \"c\"\ntitle = \"C\"\n\
             record_kind = \"constraint\"\nstatus = \"active\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             tags = []\n\n\
             [facet]\n\
             statement = \"\"\nsource = \"\"\napplies_to = []\n\
             waiver_reason = \"\"\nwaived_by = \"\"\nwaived_on = \"\"\n\n\
             [evidence]\n\
             supports = []\ncontradicts = []\nnotes = []\n\
             [[relation]]\nlabel = \"shapes\"\ntarget = \"SL-001\"\n\
             [[relation]]\nlabel = \"spawns\"\ntarget = \"ISS-001\"\n\
             [[relation]]\nlabel = \"governed_by\"\ntarget = \"ADR-001\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"concerns\"\ntarget = \"SL-001\"\n"
            ),
        );
        write(
            &root,
            ".doctrine/knowledge/constraint/001/record-001.md",
            "body\n",
        );
        {
            let mut expected = table_labels_for("CON");
            expected.remove(&(RelationLabel::Supersedes, None));
            assert_eq!(
                emitted_labels(root, "CON", 1),
                expected,
                "CON: shapes + spawns + governed_by + references(concerns) (supersedes is LifecycleOnly — typed parse in PHASE-03)"
            );
        }

        // --- EVD (knowledge): shapes + spawns + governed_by + supports + disputes + references(concerns) ---
        write(
            &root,
            ".doctrine/knowledge/evidence/001/record-001.toml",
            &format!(
                "schema = \"{SCHEMA_KNOWLEDGE}\"\nversion = 1\n\n\
             id = 1\nslug = \"e\"\ntitle = \"E\"\n\
             record_kind = \"evidence\"\nstatus = \"captured\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             tags = []\n\n\
             [facet]\n\
             datum = \"\"\nprovenance = \"\"\nconfidence = \"\"\n\n\
             [evidence]\n\
             supports = []\ncontradicts = []\nnotes = []\n\
             [[relation]]\nlabel = \"shapes\"\ntarget = \"SL-001\"\n\
             [[relation]]\nlabel = \"spawns\"\ntarget = \"ISS-001\"\n\
             [[relation]]\nlabel = \"governed_by\"\ntarget = \"ADR-001\"\n\
             [[relation]]\nlabel = \"supports\"\ntarget = \"ASM-001\"\n\
             [[relation]]\nlabel = \"disputes\"\ntarget = \"HYP-001\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"concerns\"\ntarget = \"SL-001\"\n"
            ),
        );
        write(
            &root,
            ".doctrine/knowledge/evidence/001/record-001.md",
            "body\n",
        );
        {
            let mut expected = table_labels_for("EVD");
            expected.remove(&(RelationLabel::Supersedes, None));
            assert_eq!(
                emitted_labels(root, "EVD", 1),
                expected,
                "EVD: shapes + spawns + governed_by + supports + disputes + references(concerns) (supersedes is LifecycleOnly)"
            );
        }

        // --- HYP (knowledge): shapes + spawns + governed_by + references(concerns) ---
        write(
            &root,
            ".doctrine/knowledge/hypothesis/001/record-001.toml",
            &format!(
                "schema = \"{SCHEMA_KNOWLEDGE}\"\nversion = 1\n\n\
             id = 1\nslug = \"h\"\ntitle = \"H\"\n\
             record_kind = \"hypothesis\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             tags = []\n\n\
             [facet]\n\
             proposition = \"\"\npredicts = \"\"\n\n\
             [evidence]\n\
             supports = []\ncontradicts = []\nnotes = []\n\
             [[relation]]\nlabel = \"shapes\"\ntarget = \"SL-001\"\n\
             [[relation]]\nlabel = \"spawns\"\ntarget = \"ISS-001\"\n\
             [[relation]]\nlabel = \"governed_by\"\ntarget = \"ADR-001\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"concerns\"\ntarget = \"SL-001\"\n"
            ),
        );
        write(
            &root,
            ".doctrine/knowledge/hypothesis/001/record-001.md",
            "body\n",
        );
        {
            let mut expected = table_labels_for("HYP");
            expected.remove(&(RelationLabel::Supersedes, None));
            assert_eq!(
                emitted_labels(root, "HYP", 1),
                expected,
                "HYP: shapes + spawns + governed_by + references(concerns) (supersedes is LifecycleOnly)"
            );
        }
    }

    // -- PHASE-05: corpus-edge validate + supersession cross-check ------------

    /// VT-3 (R2-M5/X2): a deleted target leaves a `[[relation]]` dangler that
    /// `validate_relations` reports; a hand-edited illegal `(source, label)` row is
    /// reported as an `IllegalRow`; a free-text `Unvalidated` target is NOT a finding
    /// (it dangles by design). Report-only — the corpus is never rewritten.
    #[test]
    fn validate_relations_reports_danglers_and_illegal_rows() {
        let dir = tmp();
        let root = dir.path();
        // SL-001 links requirements to REQ-005, which we DO seed (resolves), and to
        // REQ-999, which we do NOT (a dangler). The free-text `drift` case rides a
        // backlog issue below (`drift` is a backlog label, not a slice one).
        seed_slice(
            root,
            1,
            &[("references(implements)", &["REQ-005", "REQ-999"])],
        );
        write(
            root,
            ".doctrine/requirement/005/requirement-005.toml",
            "id = 5\nslug = \"r\"\ntitle = \"R\"\nstatus = \"active\"\n",
        );
        write(root, ".doctrine/requirement/005/requirement-005.md", "r\n");
        // A backlog issue with a free-text `drift` (Unvalidated) target — must NOT be a
        // finding (it dangles by design), plus a resolvable `slices` edge to SL-001.
        write(
            root,
            ".doctrine/backlog/issue/001/backlog-001.toml",
            &format!(
                "schema = \"{SCHEMA_BACKLOG}\"\nversion = 1\n\
             id = 1\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\ntags = []\n\
             [[relation]]\nlabel = \"drift\"\ntarget = \"loose talk\"\n\
             [[relation]]\nlabel = \"slices\"\ntarget = \"SL-001\"\n"
            ),
        );
        write(root, ".doctrine/backlog/issue/001/backlog-001.md", "i\n");
        // SL-002 carries a HAND-EDITED illegal row: a slice cannot author `descends_from`
        // (a spec-only label; `related` is now legal for slices since SL-095).
        write(
            root,
            ".doctrine/slice/002/slice-002.toml",
            "id = 2\nslug = \"s\"\ntitle = \"S\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [[relation]]\nlabel = \"descends_from\"\ntarget = \"PRD-001\"\n",
        );
        write(root, ".doctrine/slice/002/slice-002.md", "s\n");

        let findings = validate_relations(root).unwrap();
        let joined = findings.join("\n");
        assert!(
            joined.contains("SL-001") && joined.contains("REQ-999") && joined.contains("dangling"),
            "the deleted REQ-999 target is reported as a dangler: {joined}"
        );
        assert!(
            !joined.contains("REQ-005"),
            "the resolvable REQ-005 target is NOT a finding: {joined}"
        );
        assert!(
            !joined.contains("loose talk"),
            "the Unvalidated drift target dangles by design — not a finding: {joined}"
        );
        assert!(
            joined.contains("SL-002") && joined.contains("illegal"),
            "the hand-edited illegal `descends_from` row is reported: {joined}"
        );
        // Report-only: the corpus file is byte-unchanged.
        let after =
            std::fs::read_to_string(root.join(".doctrine/slice/002/slice-002.toml")).unwrap();
        assert!(
            after.contains("label = \"descends_from\""),
            "validate never rewrites the corpus"
        );
    }

    /// VT-3 (SL-149): `validate_relations` reports a hand-edited `references` row with a
    /// MISSING role and one with an ILLEGAL-for-source role as role-class `IllegalRow`s,
    /// while a well-formed `references(implements)` row and a label-only `governed_by` row
    /// produce NO finding (no false positive). Report-only — the corpus is never rewritten.
    #[test]
    fn validate_relations_flags_bad_references_role() {
        let dir = tmp();
        let root = dir.path();
        // SL-001: a well-formed references(implements) + a label-only governed_by — both
        // legal, neither a finding; plus a hand-edited references row with NO role.
        write(
            root,
            ".doctrine/slice/001/slice-001.toml",
            "id = 1\nslug = \"s\"\ntitle = \"S\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"SPEC-018\"\n\
             [[relation]]\nlabel = \"governed_by\"\ntarget = \"ADR-001\"\n\
             [[relation]]\nlabel = \"references\"\ntarget = \"PRD-010\"\n",
        );
        write(root, ".doctrine/slice/001/slice-001.md", "s\n");
        // A backlog issue with a references row whose role is illegal-for-source
        // (`implements` is SL-only) — a role-class finding.
        write(
            root,
            ".doctrine/backlog/issue/001/backlog-001.toml",
            &format!(
                "schema = \"{SCHEMA_BACKLOG}\"\nversion = 1\n\
             id = 1\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\ntags = []\n\
             [[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"SPEC-018\"\n"
            ),
        );
        write(root, ".doctrine/backlog/issue/001/backlog-001.md", "i\n");
        // Seed the well-formed references(implements) target and the governed_by target so
        // neither is ALSO a dangler — isolating the role-class findings under test.
        write(
            root,
            ".doctrine/spec/tech/018/spec-018.toml",
            "id = 18\nslug = \"x\"\ntitle = \"X\"\nstatus = \"draft\"\nkind = \"tech\"\n",
        );
        write(root, ".doctrine/spec/tech/018/spec-018.md", "x\n");
        write(
            root,
            ".doctrine/adr/001/adr-001.toml",
            "id = 1\nslug = \"a\"\ntitle = \"A\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\ntags = []\n",
        );
        write(root, ".doctrine/adr/001/adr-001.md", "a\n");

        let findings = validate_relations(root).unwrap();
        let joined = findings.join("\n");
        // The missing-role SL row and the illegal-role ISS row are both role-class findings.
        assert!(
            joined.contains("SL-001: [[relation]] row `references` -> `PRD-010`")
                && joined.contains("missing or illegal role"),
            "the missing-role references row is reported as a role-class IllegalRow: {joined}"
        );
        assert!(
            joined.contains("ISS-001: [[relation]] row `references` -> `SPEC-018`")
                && joined.contains("missing or illegal role"),
            "the illegal-for-source role is reported: {joined}"
        );
        // No false positive: NEITHER the well-formed references(implements) SPEC-018 row nor
        // the label-only governed_by ADR-001 row appears in ANY finding (both resolve and
        // are role-legal). The only findings are the two role-class ones above.
        assert_eq!(
            findings.len(),
            2,
            "exactly the two role-class findings, nothing else: {joined}"
        );
    }

    /// VT-4 (R2-m2/OD-3): the supersession cross-check reads the STORED `superseded_by`
    /// via the typed governance seam and reports disagreement with the `supersedes`
    /// in-edge reciprocal — BOTH ways (a derived edge missing from the stored field, and
    /// a stored entry with no derived backing). A consistent pair yields NO finding.
    #[test]
    fn validate_supersession_reports_drift_both_ways() {
        let dir = tmp();
        let root = dir.path();
        // ADR-002 supersedes ADR-001 (derived: ADR-001 superseded_by ADR-002). ADR-001
        // stores NO superseded_by ⇒ a "derived-not-stored" finding. ADR-003 stores a
        // bogus superseded_by ADR-009 with no backing ⇒ a "stored-not-derived" finding.
        seed_adr(root, 1, &[]);
        seed_adr(root, 2, &[("supersedes", &["ADR-001"])]);
        seed_adr(root, 3, &[("superseded_by", &["ADR-009"])]);

        let findings = validate_supersession(root).unwrap();
        let joined = findings.join("\n");
        assert!(
            joined.contains("ADR-001") && joined.contains("ADR-002"),
            "ADR-002 supersedes ADR-001 but ADR-001 omits it from superseded_by: {joined}"
        );
        assert!(
            joined.contains("ADR-003") && joined.contains("ADR-009"),
            "ADR-003 lists ADR-009 in superseded_by with no backing supersedes: {joined}"
        );
    }

    /// A consistent supersession pair (the successor's `supersedes` AND the
    /// predecessor's `superseded_by` agree) produces no cross-check finding.
    #[test]
    fn validate_supersession_clean_on_consistent_pair() {
        let dir = tmp();
        let root = dir.path();
        seed_adr(root, 1, &[("superseded_by", &["ADR-002"])]);
        seed_adr(root, 2, &[("supersedes", &["ADR-001"])]);
        assert!(
            validate_supersession(root).unwrap().is_empty(),
            "a consistent supersedes/superseded_by pair is clean"
        );
    }

    // -- SL-138 PHASE-02: the relation-transitive walk (design §5) ------------

    /// Scan a seeded corpus the way the command layer does (the F2 single scan).
    fn scan(root: &Path) -> Vec<ScannedEntity> {
        scan_entities(root, &mut vec![], ScanMode::default()).unwrap()
    }

    /// The (label, targets) of a direction's groups, for ergonomic assertions.
    fn groups(dir: &Option<Vec<TransitiveGroup>>) -> Vec<(RelationLabel, Vec<String>)> {
        dir.as_ref()
            .map(|gs| gs.iter().map(|g| (g.label, g.targets.clone())).collect())
            .unwrap_or_default()
    }

    // VT-1 — inbound vs outbound differ correctly on a directed fixture.
    // SL-001 --governed_by--> ADR-005: inbound from ADR reaches SL (blast radius);
    // outbound from ADR on that label is empty; outbound from SL reaches ADR.
    #[test]
    fn transitive_inbound_outbound_directional() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[("governed_by", &["ADR-005"])]);
        seed_adr(root, 5, &[]);
        let scanned = scan(root);

        // Inbound from ADR-005: SL-001 governs-points at it → reachable Against.
        let view = transitive_from(
            &scanned,
            root,
            "ADR-005",
            TransitiveDir::Inbound,
            None,
            Some(5),
        )
        .unwrap();
        assert_eq!(
            groups(&view.inbound),
            vec![(RelationLabel::GovernedBy, vec!["SL-001".to_string()])],
            "inbound governed_by from ADR-005 reaches SL-001"
        );
        assert!(view.outbound.is_none(), "outbound not requested → omitted");

        // Outbound from ADR-005: ADR authors no governed_by → empty (but Some, requested).
        let view = transitive_from(
            &scanned,
            root,
            "ADR-005",
            TransitiveDir::Outbound,
            None,
            Some(5),
        )
        .unwrap();
        assert!(view.inbound.is_none(), "inbound not requested → omitted");
        assert_eq!(
            groups(&view.outbound),
            vec![],
            "outbound from ADR-005 on governed_by is empty"
        );

        // Outbound from SL-001 reaches its governor ADR-005.
        let view = transitive_from(
            &scanned,
            root,
            "SL-001",
            TransitiveDir::Outbound,
            None,
            Some(5),
        )
        .unwrap();
        assert_eq!(
            groups(&view.outbound),
            vec![(RelationLabel::GovernedBy, vec!["ADR-005".to_string()])],
            "outbound governed_by from SL-001 reaches ADR-005"
        );
    }

    // VT-2 — per-label sectioning, `labels` narrowing, and the F3 role collapse:
    // a slice with mixed-role `references` outbound renders ONE `references` group.
    #[test]
    fn transitive_per_label_sections_narrowing_and_role_collapse() {
        let dir = tmp();
        let root = dir.path();
        // SL-001 outbound across two DISTINCT references roles (implements→REQ-001,
        // concerns→ADR-005) plus governed_by→ADR-005. The two references roles ride
        // ONE label-keyed overlay (R5), so a transitive walk collapses them into a
        // single `references` section (F3). All targets must MINT to form graph edges.
        seed_slice(
            root,
            1,
            &[
                ("references(implements)", &["REQ-001"]),
                ("references(concerns)", &["ADR-005"]),
                ("governed_by", &["ADR-005"]),
            ],
        );
        seed_adr(root, 5, &[]);
        write(
            root,
            ".doctrine/requirement/001/requirement-001.toml",
            "id = 1\nslug = \"r\"\ntitle = \"R\"\nstatus = \"active\"\n",
        );
        write(root, ".doctrine/requirement/001/requirement-001.md", "b\n");
        let scanned = scan(root);

        // Default labels (None): both overlay-backed sections present, sorted by name.
        let view = transitive_from(
            &scanned,
            root,
            "SL-001",
            TransitiveDir::Outbound,
            None,
            Some(5),
        )
        .unwrap();
        assert_eq!(
            groups(&view.outbound),
            vec![
                (RelationLabel::GovernedBy, vec!["ADR-005".to_string()]),
                // ONE references group, two roles collapsed (F3), targets id-ascending.
                (
                    RelationLabel::References,
                    vec!["ADR-005".to_string(), "REQ-001".to_string()]
                ),
            ],
            "per-label sections, references roles collapsed to one section"
        );

        // Narrow to `references` only — the governed_by section drops out.
        let view = transitive_from(
            &scanned,
            root,
            "SL-001",
            TransitiveDir::Outbound,
            Some(&[RelationLabel::References]),
            Some(5),
        )
        .unwrap();
        assert_eq!(
            groups(&view.outbound),
            vec![(
                RelationLabel::References,
                vec!["ADR-005".to_string(), "REQ-001".to_string()]
            )],
            "labels narrowing to a subset"
        );
    }

    // VT-3 — no-overlay labels are rejected from an explicit `labels` arg, absent
    // from the default set, and the overlay-backed predicate is table-derived.
    #[test]
    fn transitive_rejects_no_overlay_labels_and_predicate_is_table_derived() {
        use crate::relation::{RELATION_RULES, TargetSpec};
        use std::collections::BTreeSet;

        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[("governed_by", &["ADR-005"])]);
        seed_adr(root, 5, &[]);
        let scanned = scan(root);

        // Each no-overlay (TargetSpec::Unvalidated) label → "not transitively walkable".
        for label in [
            RelationLabel::Contextualizes,
            RelationLabel::Drift,
            RelationLabel::DecisionRef,
        ] {
            let err = transitive_from(
                &scanned,
                root,
                "SL-001",
                TransitiveDir::Both,
                Some(&[label]),
                Some(5),
            )
            .unwrap_err();
            assert!(
                err.to_string().contains("not transitively walkable")
                    && err.to_string().contains(label.name()),
                "{label:?} rejected with a clear message: {err}"
            );
        }

        // The default (None) section set excludes every no-overlay label — proven by
        // building the full corpus reachable set and asserting none appear. Here SL-001
        // only authors governed_by, so we assert the absence structurally via the label
        // selector the engine uses: the default set == the table's resolvable labels.
        let rg = build_relation_graph_from(&scanned).unwrap();
        let default_set: BTreeSet<RelationLabel> = transitive_labels(&rg.overlays, None)
            .unwrap()
            .into_iter()
            .collect();
        let resolvable_from_table: BTreeSet<RelationLabel> = RELATION_RULES
            .iter()
            .filter(|r| !matches!(r.target, TargetSpec::Unvalidated))
            .map(|r| r.label)
            .collect();
        assert_eq!(
            default_set, resolvable_from_table,
            "default transitive label set is table-derived (== resolvable labels), no hardcoded list"
        );
        for label in [
            RelationLabel::Contextualizes,
            RelationLabel::Drift,
            RelationLabel::DecisionRef,
        ] {
            assert!(
                !default_set.contains(&label),
                "{label:?} (no overlay) absent from the default transitive set"
            );
        }
    }

    // VT-4 — depth cap truncates + sets group & view `truncated`; unbounded reaches
    // the deepest leaf; a never-minted id errors via the existence gate.
    #[test]
    fn transitive_depth_cap_truncation_and_existence_gate() {
        let dir = tmp();
        let root = dir.path();
        // A supersedes chain SL-004 -> SL-003 -> SL-002 -> SL-001 (outbound Along).
        seed_slice(root, 1, &[]);
        seed_slice(root, 2, &[("supersedes", &["SL-001"])]);
        seed_slice(root, 3, &[("supersedes", &["SL-002"])]);
        seed_slice(root, 4, &[("supersedes", &["SL-003"])]);
        let scanned = scan(root);

        // Unbounded: outbound from SL-004 reaches all three, no truncation.
        let view = transitive_from(
            &scanned,
            root,
            "SL-004",
            TransitiveDir::Outbound,
            None,
            None,
        )
        .unwrap();
        assert_eq!(
            groups(&view.outbound),
            vec![(
                RelationLabel::Supersedes,
                vec![
                    "SL-001".to_string(),
                    "SL-002".to_string(),
                    "SL-003".to_string()
                ]
            )],
            "unbounded supersedes walk reaches the deepest leaf"
        );
        assert!(!view.truncated, "no cap → not truncated");

        // Depth 2: reaches SL-003 (1) + SL-002 (2); SL-001 (3) is cut → truncated.
        let view = transitive_from(
            &scanned,
            root,
            "SL-004",
            TransitiveDir::Outbound,
            None,
            Some(2),
        )
        .unwrap();
        let outbound = view.outbound.as_ref().unwrap();
        assert_eq!(
            outbound[0].targets,
            vec!["SL-002".to_string(), "SL-003".to_string()],
            "depth 2 excludes the depth-3 node"
        );
        assert!(outbound[0].truncated, "group truncated at the cap");
        assert!(view.truncated, "view-level truncated ORs the group flag");

        // Never-minted id → the existence gate errors (not an empty view).
        let err = transitive_from(&scanned, root, "SL-999", TransitiveDir::Both, None, Some(5))
            .unwrap_err();
        assert_eq!(err.to_string(), "SL-999: no such entity");
    }

    // VT-5 — JSON golden: kind=inspect-transitive, inbound before outbound,
    // max_depth null when unbounded, a non-requested direction key absent.
    #[test]
    fn transitive_json_envelope_golden() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[("governed_by", &["ADR-005"])]);
        seed_adr(root, 5, &[]);
        let scanned = scan(root);

        // Both directions, unbounded → max_depth null, both keys present, inbound first.
        let view =
            transitive_from(&scanned, root, "ADR-005", TransitiveDir::Both, None, None).unwrap();
        let json = render_transitive_json(&view).unwrap();
        let expected = "\
{
  \"id\": \"ADR-005\",
  \"inbound\": [
    {
      \"label\": \"governed_by\",
      \"targets\": [
        \"SL-001\"
      ],
      \"truncated\": false
    }
  ],
  \"kind\": \"inspect-transitive\",
  \"max_depth\": null,
  \"outbound\": [],
  \"truncated\": false
}";
        assert_eq!(json, expected, "the C4 JSON envelope golden");

        // A single requested direction omits the other key entirely.
        let view = transitive_from(
            &scanned,
            root,
            "ADR-005",
            TransitiveDir::Inbound,
            None,
            Some(5),
        )
        .unwrap();
        let value = transitive_value(&view);
        assert!(
            value.get("inbound").is_some(),
            "requested direction present"
        );
        assert!(
            value.get("outbound").is_none(),
            "non-requested direction key absent (not null/empty)"
        );
        assert_eq!(value.get("max_depth").and_then(|v| v.as_u64()), Some(5));
    }

    // Human render: header depth, inbound-before-outbound, (none) for empty, and the
    // truncation line gated on the view flag.
    #[test]
    fn transitive_human_render_shape() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[("governed_by", &["ADR-005"])]);
        seed_adr(root, 5, &[]);
        let scanned = scan(root);

        let view = transitive_from(
            &scanned,
            root,
            "ADR-005",
            TransitiveDir::Both,
            None,
            Some(5),
        )
        .unwrap();
        let text = render_transitive_human(&view);
        assert_eq!(
            text,
            "ADR-005 — transitive (depth 5)\n\
             \ndepends on this (inbound):\n  governed_by: SL-001\n\
             \nthis depends on (outbound):\n  (none)\n"
        );

        // Unbounded → header reads "depth all".
        let view = transitive_from(
            &scanned,
            root,
            "ADR-005",
            TransitiveDir::Inbound,
            None,
            None,
        )
        .unwrap();
        assert!(
            render_transitive_human(&view).starts_with("ADR-005 — transitive (depth all)\n"),
            "unbounded header reads depth all"
        );
    }
}
