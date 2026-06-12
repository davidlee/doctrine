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
//! extends this same file with the all-kind scan ([`build_relation_graph`]), the
//! `Projection<EntityKey>`, the reference overlays, and the [`inspect`] query
//! (design §5.4). `inspect` is still unreachable from any binary until PHASE-04
//! wires the CLI command, so it (and the scan it drives) remains `not(test)`-dead.
//!
//! Self-clearing `not(test)` `dead_code` expect (the `dead-code-self-clearing-leaf`
//! precedent): the scan and `inspect` land ahead of their PHASE-04 `inspect` CLI
//! consumer. Under `cfg(test)` the VTs exercise the full surface, so the expect
//! scopes to `not(test)` where the gate's plain `cargo clippy` (bins/lib, no test
//! cfg) sees the items as genuinely dead; it retires itself when PHASE-04 wires the
//! command.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-046 PHASE-03 all-kind scan + inspect query — built ahead of \
                  their PHASE-04 inspect CLI consumer; live under cfg(test), retires \
                  itself when PHASE-04 wires the command"
    )
)]

use std::collections::BTreeMap;
use std::path::Path;

use cordage::{Arity, CyclePolicy, EdgeAttrs, Graph, GraphBuilder, OverlayConfig, OverlayId};

use crate::entity;
use crate::integrity;
use crate::listing;
use crate::projection::Projection;
use crate::relation::{RelationEdge, RelationLabel};

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
/// (by `ItemKind`); RV→review; REC→rec.
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
        "RV" => crate::review::relation_edges(root, id),
        "REC" => crate::rec::relation_edges(root, id),
        // The five backlog kinds share one accessor, routed by their ItemKind (the
        // prefix↔kind map is backlog's single source — no second copy here).
        other => match crate::backlog::kind_from_prefix(other) {
            Some(item_kind) => crate::backlog::relation_edges(root, item_kind, id),
            // Unreachable for any `integrity::KINDS` row; a defensive empty for an
            // unknown prefix keeps the dispatch total without a panic.
            None => Ok(Vec::new()),
        },
    }
}

// ---------------------------------------------------------------------------
// PHASE-03 — the all-kind scan, the reference overlays, and the inspect query.
// ---------------------------------------------------------------------------

/// The projection key for a numbered entity (design §5.2). Stores the kind's
/// `&'static str` prefix — `Copy + Ord`, unlike `entity::Kind` (which is data, not
/// `Ord`, and carries a fn-ptr `scaffold`) — and the numeric id. The pair is the
/// corpus-wide identity, and renders its canonical ref through the same
/// `listing::canonical_id` source `ItemId` uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct EntityKey {
    prefix: &'static str,
    id: u32,
}

impl EntityKey {
    /// The canonical ref string (`SL-046`) for this key — the single id-form
    /// authority, shared with every other prefixed surface (`listing::canonical_id`).
    fn canonical(self) -> String {
        listing::canonical_id(self.prefix, self.id)
    }
}

/// The overlay-identity map: one cordage overlay per OVERLAY-BACKED relation label
/// (the 11 of design §5.3), keyed both ways. The two target-unvalidated labels —
/// `Drift` and `DecisionRef` (ADR-010 Decision 2) — get NO overlay (their targets
/// never resolve to a node), so `overlay_for` returns `None` for them and their
/// edges always dangle.
///
/// Label is overlay identity (OQ2-B): the same label authored from different source
/// kinds (e.g. `Supersedes` from both slice and governance) shares ONE overlay.
struct OverlayMap {
    by_label: BTreeMap<RelationLabel, OverlayId>,
    by_overlay: BTreeMap<OverlayId, RelationLabel>,
}

impl OverlayMap {
    /// Allocate one `Reject`/`Unbounded` overlay per overlay-backed label (I1:
    /// `Reject` removes no edges, `Unbounded` exempts arity eviction — `in_edges`
    /// then enumerates exactly the authored unique inbound set).
    fn build(builder: &mut GraphBuilder) -> Self {
        const OVERLAY_LABELS: &[RelationLabel] = &[
            RelationLabel::Specs,
            RelationLabel::Requirements,
            RelationLabel::Supersedes,
            RelationLabel::DescendsFrom,
            RelationLabel::Parent,
            RelationLabel::Members,
            RelationLabel::Interactions,
            RelationLabel::Slices,
            RelationLabel::Related,
            RelationLabel::Reviews,
            RelationLabel::OwningSlice,
        ];
        let mut by_label = BTreeMap::new();
        let mut by_overlay = BTreeMap::new();
        for &label in OVERLAY_LABELS {
            let ov = builder.overlay(OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded));
            by_label.insert(label, ov);
            by_overlay.insert(ov, label);
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

/// Build the cross-kind relation graph once (design §5.4 — mirrors
/// `backlog_order::build`). A SEPARATE cordage `Graph` from `backlog_order`: they
/// share the `Projection` *type*, never a graph instance or a scan.
///
/// 1. Mint nodes: walk `integrity::KINDS` in TABLE order; per kind `scan_ids`
///    (already skips the `NNN-slug` symlink + non-dirs — VT-5 free), **sort ids
///    ascending** (C5 — `scan_ids` is unsorted `read_dir` order; the sort makes
///    mint + render permutation-invariant, REQ-077), and `intern` each in that
///    order.
/// 2. Emit edges: per minted entity, `outbound_for` → per edge, parse + resolve the
///    target; a resolvable target whose label has an overlay ⇒ `builder.edge`,
///    `EdgeAttrs::new(0, 0)` (C3 — two authored rows with the same `(label,src,dst)`
///    collapse to one in cordage's `BTreeSet<Edge>`); anything else (unresolved,
///    parse-error / free-text, or a no-overlay label like `Drift`/`DecisionRef`,
///    INCLUDING a resolvable target under a no-overlay label) ⇒ a dangler.
/// 3. `builder.build()` — NO `OrderSpec` over reference overlays (I2: direct-only,
///    composition-free; no union-cycle pass touches them).
///
/// Factored out of `inspect` (a `pub(crate)`-shaped named fn) per the SL-047
/// forward-coupling request — SL-047's transitive walk reuses this same scan.
fn build_relation_graph(root: &Path) -> anyhow::Result<RelationGraph> {
    let mut builder = GraphBuilder::new();
    let overlays = OverlayMap::build(&mut builder);
    let mut projection: Projection<EntityKey> = Projection::new();

    // Pass 1 — mint every entity's node (KINDS table order, ids ascending).
    for kref in integrity::KINDS {
        let prefix = kref.kind.prefix;
        let mut ids = entity::scan_ids(&root.join(kref.kind.dir))?;
        ids.sort_unstable();
        for id in ids {
            projection.intern(&mut builder, EntityKey { prefix, id });
        }
    }

    // Pass 2 — emit edges (resolve only, never intern) and collect danglers.
    let mut danglers: BTreeMap<EntityKey, Vec<(RelationLabel, String)>> = BTreeMap::new();
    for kref in integrity::KINDS {
        let prefix = kref.kind.prefix;
        let mut ids = entity::scan_ids(&root.join(kref.kind.dir))?;
        ids.sort_unstable();
        for id in ids {
            let src_key = EntityKey { prefix, id };
            // Present by construction (just interned in pass 1); the `else` is
            // defensive only, keeping the path panic-free.
            let Some(src) = projection.resolve(src_key) else {
                continue;
            };
            for edge in outbound_for(root, kref.kind, id)? {
                if let Some(dst) = resolve_target(&projection, &edge)
                    && let Some(ov) = overlays.overlay_for(edge.label)
                {
                    builder.edge(ov, src, dst, EdgeAttrs::new(0, 0));
                } else {
                    danglers
                        .entry(src_key)
                        .or_default()
                        .push((edge.label, edge.target.clone()));
                }
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

/// One entity's direct relation view (design §5.2): its authored outbound relations
/// grouped by label, the derived inbound relations grouped by label, and its
/// unresolved/free-text outbound danglers. Direct-only, one-hop, composition-free
/// (I2). Inbound is recomputed every query from `in_edges` — nothing stores a
/// reverse field (ADR-004 §3 / REQ-074).
#[derive(Debug)]
struct InspectView {
    id: String,
    outbound: Vec<(RelationLabel, Vec<String>)>,
    inbound: Vec<(RelationLabel, Vec<String>)>,
    danglers: Vec<(RelationLabel, String)>,
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
/// A well-formed ref to a non-existent id (never minted) returns an empty-section
/// view, not an error (VT-5 — mirrors a `show`-like read surface over an empty
/// entity). NEVER reads `graph.provenance()` (C7 — a benign symmetric-`related`
/// 2-cycle yields a `Reject` `CycleDiagnostic` that must not leak into the view).
fn inspect(root: &Path, id: &str) -> anyhow::Result<InspectView> {
    let (kref, qid) = integrity::parse_canonical_ref(id)?;
    let query_key = EntityKey {
        prefix: kref.kind.prefix,
        id: qid,
    };

    let rg = build_relation_graph(root)?;

    // A well-formed ref to a non-existent id (never minted — no entity dir) is an
    // empty-section view, not an error (VT-5). The node-existence gate also keeps
    // `outbound_for` (which reads the entity's own toml) off a missing file.
    let Some(node) = rg.projection.resolve(query_key) else {
        return Ok(InspectView {
            id: query_key.canonical(),
            outbound: Vec::new(),
            inbound: Vec::new(),
            danglers: Vec::new(),
        });
    };

    // outbound — the entity's own authored edges, grouped by label (targets in
    // authored order within a label).
    let mut outbound_by_label: BTreeMap<RelationLabel, Vec<String>> = BTreeMap::new();
    for edge in outbound_for(root, kref.kind, qid)? {
        outbound_by_label
            .entry(edge.label)
            .or_default()
            .push(edge.target);
    }
    let outbound: Vec<(RelationLabel, Vec<String>)> = outbound_by_label.into_iter().collect();

    // inbound — derived from in_edges per overlay (no stored reverse field read).
    let mut inbound_by_label: BTreeMap<RelationLabel, Vec<String>> = BTreeMap::new();
    for (&overlay, &label) in &rg.overlays.by_overlay {
        let mut srcs: Vec<String> = rg
            .graph
            .in_edges(overlay, node)
            .filter_map(|(src_node, _attrs)| rg.projection.key_of(src_node))
            .map(EntityKey::canonical)
            .collect();
        if !srcs.is_empty() {
            // in_edges orders by the (src,rank,age) adjacency key, but src NodeId
            // order is mint order, not ref order — sort to canonical-ref order for a
            // deterministic, permutation-invariant render (REQ-077).
            srcs.sort();
            inbound_by_label.entry(label).or_default().extend(srcs);
        }
    }
    let inbound: Vec<(RelationLabel, Vec<String>)> = inbound_by_label.into_iter().collect();

    // danglers — only the queried entity's set (empty if none).
    let danglers = rg.danglers.get(&query_key).cloned().unwrap_or_default();

    Ok(InspectView {
        id: query_key.canonical(),
        outbound,
        inbound,
        danglers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrity::KINDS;
    use crate::relation::RelationLabel;
    use std::fs;
    use std::path::PathBuf;

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

    fn tmp() -> PathBuf {
        let base = std::env::temp_dir().join(format!("sl046_relgraph_{}", std::process::id()));
        let dir = base.join(format!(
            "{:?}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    // -- VT-1 outbound correctness per kind + outbound_for dispatch ----------

    #[test]
    fn slice_outbound_specs_requirements_supersedes() {
        let root = tmp();
        write(
            &root,
            ".doctrine/slice/001/slice-001.toml",
            "id = 1\nslug = \"a\"\ntitle = \"A\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nspecs = [\"PRD-010\"]\nrequirements = [\"REQ-001\", \"REQ-002\"]\n\
             supersedes = [\"SL-000\"]\n",
        );
        write(&root, ".doctrine/slice/001/slice-001.md", "scope\n");
        let edges = outbound_for(&root, kind_for("SL"), 1).unwrap();
        assert_eq!(
            pairs(&edges),
            vec![
                (RelationLabel::Specs, "PRD-010"),
                (RelationLabel::Requirements, "REQ-001"),
                (RelationLabel::Requirements, "REQ-002"),
                (RelationLabel::Supersedes, "SL-000"),
            ]
        );
    }

    #[test]
    fn governance_outbound_supersedes_related_only() {
        let root = tmp();
        // ADR with every axis populated — only supersedes + related must emit.
        write(
            &root,
            ".doctrine/adr/002/adr-002.toml",
            "id = 2\nslug = \"a\"\ntitle = \"A\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = [\"ADR-001\"]\nsuperseded_by = [\"ADR-009\"]\n\
             related = [\"ADR-004\"]\ntags = [\"layering\"]\n",
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
        let root = tmp();
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
        let root = tmp();
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
        let root = tmp();
        // Every axis populated — only slices/specs/drift must emit (not
        // needs/after/triggers).
        write(
            &root,
            ".doctrine/backlog/issue/001/backlog-001.toml",
            "id = 1\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nslices = [\"SL-020\"]\nspecs = [\"PRD-009\"]\n\
             drift = [\"some-free-text\"]\nneeds = [\"ISS-002\"]\n",
        );
        write(&root, ".doctrine/backlog/issue/001/backlog-001.md", "b\n");
        let edges = outbound_for(&root, kind_for("ISS"), 1).unwrap();
        assert_eq!(
            pairs(&edges),
            vec![
                (RelationLabel::Slices, "SL-020"),
                (RelationLabel::Specs, "PRD-009"),
                (RelationLabel::Drift, "some-free-text"),
            ],
            "backlog emits slices/specs/drift ONLY (no needs/after/triggers)"
        );
    }

    #[test]
    fn review_outbound_single_reviews_edge() {
        let root = tmp();
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
        let root = tmp();
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
        let root = tmp();
        // REQ is an edge target only; the dispatch returns empty without touching disk.
        let edges = outbound_for(&root, kind_for("REQ"), 1).unwrap();
        assert!(edges.is_empty());
    }

    // -- VT-2 exclusion proof (REC decision_ref carried, not dropped) --------

    #[test]
    fn rec_decision_ref_carried_as_free_text_not_dropped() {
        let root = tmp();
        write(
            &root,
            ".doctrine/rec/002/rec-002.toml",
            "id = 2\nslug = \"r\"\ntitle = \"R\"\n\
             [rec]\nmove = \"accept\"\ndecision_ref = \"DEC-001\"\n",
        );
        let edges = outbound_for(&root, kind_for("REC"), 2).unwrap();
        // decision_ref survives even with no owning_slice — carried, will dangle.
        assert_eq!(pairs(&edges), vec![(RelationLabel::DecisionRef, "DEC-001")]);
    }

    // -- VT-3 interactions collapse to a single `Interactions` class ---------
    // (The per-edge free-text `type` round-trips from the SOURCE `Interaction`
    //  struct — asserted in spec.rs where the reader + struct are visible.)

    #[test]
    fn interactions_collapse_to_single_class_label() {
        let root = tmp();
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

    /// All inbound targets under `label` in a view (sorted-render order).
    fn inbound_for(view: &InspectView, label: RelationLabel) -> Vec<&str> {
        view.inbound
            .iter()
            .find(|(l, _)| *l == label)
            .map(|(_, v)| v.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    /// All outbound targets under `label` in a view.
    fn outbound_targets(view: &InspectView, label: RelationLabel) -> Vec<&str> {
        view.outbound
            .iter()
            .find(|(l, _)| *l == label)
            .map(|(_, v)| v.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    /// A minimal slice toml with the given relationships block body.
    fn slice_toml(id: u32, rels: &str) -> String {
        format!(
            "id = {id}\nslug = \"s\"\ntitle = \"S\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n[relationships]\n{rels}"
        )
    }

    /// Seed a slice entity (toml + md) under `root`.
    fn seed_slice(root: &Path, id: u32, rels: &str) {
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
            &slice_toml(id, rels),
        );
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.md"),
            "scope\n",
        );
    }

    /// Seed an ADR governance entity.
    fn seed_adr(root: &Path, id: u32, rels: &str) {
        write(
            root,
            &format!(".doctrine/adr/{id:03}/adr-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"a\"\ntitle = \"A\"\nstatus = \"accepted\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n[relationships]\n{rels}"
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
        let root = tmp();
        // SL-002 supersedes SL-001 and requires REQ-005; SL-001 authors nothing.
        seed_slice(&root, 1, "");
        seed_slice(
            &root,
            2,
            "requirements = [\"REQ-005\"]\nsupersedes = [\"SL-001\"]\n",
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

        // REQ-005's only inbound is the requirements edge from SL-002.
        let req = inspect(&root, "REQ-005").unwrap();
        assert_eq!(
            inbound_for(&req, RelationLabel::Requirements),
            vec!["SL-002"]
        );

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
        let root = tmp();
        // SL-002 lists SL-001 twice under supersedes (an authoring duplicate).
        seed_slice(&root, 1, "");
        seed_slice(&root, 2, "supersedes = [\"SL-001\", \"SL-001\"]\n");
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
        let root = tmp();
        // Three supersedors of SL-001, planted out of id order on disk; scan_ids is
        // read_dir order (unsorted), so the only thing making the render stable is
        // the ascending sort + the canonical-ref sort in inspect.
        seed_slice(&root, 1, "");
        seed_slice(&root, 4, "supersedes = [\"SL-001\"]\n");
        seed_slice(&root, 2, "supersedes = [\"SL-001\"]\n");
        seed_slice(&root, 3, "supersedes = [\"SL-001\"]\n");
        let view = inspect(&root, "SL-001").unwrap();
        assert_eq!(
            inbound_for(&view, RelationLabel::Supersedes),
            vec!["SL-002", "SL-003", "SL-004"],
            "inbound renders in ascending canonical-ref order, not filesystem order"
        );
    }

    // VT-4 / C8/R3 — a stored `superseded_by` with NO reciprocal `supersedes`
    // produces NO inbound. The reader projects only the outbound `supersedes`; the
    // stored reverse field is never read (ADR-004 §5 carve-out, but §3 derivation).
    #[test]
    fn stored_superseded_by_without_reciprocal_yields_no_inbound() {
        let root = tmp();
        // ADR-002 carries a stored superseded_by = ADR-009 but NO entity authors
        // `supersedes = [ADR-002]`. ADR-009 exists but supersedes nothing.
        seed_adr(&root, 2, "superseded_by = [\"ADR-009\"]\n");
        seed_adr(&root, 9, "");
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
        let root = tmp();
        // A backlog issue with a free-text drift, an unresolved slice ref, and a
        // resolvable slice ref. drift → dangler (no DRIFT kind); SL-099 → dangler
        // (no such entity); SL-001 → a real edge.
        seed_slice(&root, 1, "");
        write(
            &root,
            ".doctrine/backlog/issue/001/backlog-001.toml",
            "id = 1\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nslices = [\"SL-001\", \"SL-099\"]\ndrift = [\"some-free-text\"]\n",
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
        seed_slice(&root, 50, "");
        let lone = inspect(&root, "SL-050").unwrap();
        assert!(lone.outbound.is_empty());
        assert!(lone.inbound.is_empty());
        assert!(lone.danglers.is_empty());
        // (SL-001 has the inbound slices edge — sanity that inspect saw it.)
        assert_eq!(inbound_for(&empty, RelationLabel::Slices), vec!["ISS-001"]);
    }

    // VT-5 — a well-formed ref to a non-existent id returns an empty view, not an
    // error; an unknown prefix is a clean error (not a panic).
    #[test]
    fn nonexistent_id_empty_view_unknown_prefix_clean_error() {
        let root = tmp();
        seed_slice(&root, 1, "");
        // Well-formed ref, no such entity → empty sections.
        let ghost = inspect(&root, "SL-999").unwrap();
        assert_eq!(ghost.id, "SL-999");
        assert!(ghost.outbound.is_empty());
        assert!(ghost.inbound.is_empty());
        assert!(ghost.danglers.is_empty());
        // Unknown prefix → clean error.
        let err = inspect(&root, "ZZZ-001").unwrap_err();
        assert!(
            err.to_string().contains("ZZZ"),
            "unknown prefix surfaces a clean error mentioning the prefix"
        );
    }
}
