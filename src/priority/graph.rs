// SPDX-License-Identifier: GPL-3.0-only
//! The priority graph adapter (SL-047 §5.2) — the THIRD cordage `Graph`.
//!
//! Consumes `relation_graph`'s `pub(crate)` all-kind scan seam
//! ([`crate::relation_graph::scan_entities`]) to build a cordage `Graph` carrying:
//! - the `needs` **dep overlay** (hard prerequisite, `Reject`) and the `after`
//!   **seq overlay** (soft sequence, `Evict`) — the `backlog_order` template,
//!   emitted KIND-AGNOSTICALLY (DD-2: today only backlog authors `needs`/`after`, so
//!   they connect only backlog nodes; non-backlog nodes carry none and that is
//!   correct);
//! - the SL-046 **reference/lineage overlays** (`ref_overlays`) — the consequence
//!   inputs;
//! - per-node [`NodeAttr`] (kind, RAW authored status, `promoted`);
//! - a **consequence pre-pass** tally of inbound work/lineage references; and
//! - an `OrderSpec` over `[dep Along, seq Along]`.
//!
//! NO partition/channel POLICY yet — `NodeAttr` stores the RAW authored status
//! string; classification (workable/terminal) is PHASE-02. A SEPARATE cordage
//! `Graph` from `backlog_order`'s and `inspect`'s — they share the `Projection`
//! *type*, never a graph instance or a scan (the scan is the shared seam, EX-5).
//!
//! Layering (ADR-001): `priority` → `relation_graph` → `projection` → `cordage`. No
//! cycle. The build is pure over the scanned `Vec` (the disk touch lives in
//! `scan_entities`, the imperative shell).
//!
//! The whole adapter is consumed by the priority CLI surface (SL-047 PHASE-03 —
//! `priority::surface` builds the view rows from `build()`), so the PHASE-01/02
//! self-clearing `not(test)` `dead_code` suppression has retired itself, as designed
//! (`mem.pattern.lint.dead-code-expect-vs-cfg-test`).

use std::collections::BTreeMap;

use cordage::{
    Arity, CyclePolicy, Direction, EdgeAttrs, Graph, GraphBuilder, OrderLayer, OrderSpec,
    OverlayConfig, OverlayId,
};

use crate::projection::Projection;
use crate::relation::RelationLabel;
use crate::relation_graph::{self, EntityKey};
use crate::{backlog, entity, integrity};

/// One node's authored attributes (design §5.2). `kind` is the `&'static entity::Kind`
/// descriptor (data, not `Ord` — carries a fn-ptr `scaffold`; stored by reference like
/// `EntityKey` stores `prefix`). `status` is the RAW authored status string — `None`
/// for the status-less REC kind ONLY; RV carries its DERIVED active/done (authored-tier
/// over its finding ledger). NO classification here (workable/terminal is PHASE-02).
/// `promoted` is the backlog `resolution == Promoted` typed flag — DISTINCT from
/// status-terminal, NOT the free-text `origin`.
pub(crate) struct NodeAttr {
    pub(crate) kind: &'static entity::Kind,
    pub(crate) status: Option<String>,
    pub(crate) promoted: bool,
    /// The entity's authored `title`, captured from the scan (display-only — the pure
    /// channel layer never reads it). Carried here so the impure surface shell needs
    /// no second per-row disk read (one scan, one read per entity).
    pub(crate) title: String,
}

/// An unresolved outbound edge — the authored `(source, label, target)` whose target
/// did not resolve to a minted node (free-text, dangling, or a no-overlay label),
/// mirroring `relation_graph`'s dangler shape (keyed by source key here so a future
/// per-entity query can return only its own). `label` is `None` for the dep/seq axes
/// (`needs`/`after`), which carry no `RelationLabel` vocabulary entry (PRD-009 item→
/// item edges, not the SL-046 reference labels).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "priority danglers are a built artifact (the unresolved-prereq record); \
                  no PHASE-03 surface reads them yet — exercised by the graph tests, kept \
                  for a future per-entity dangler query"
    )
)]
pub(crate) struct Dangling {
    pub(crate) source: EntityKey,
    pub(crate) label: Option<RelationLabel>,
    pub(crate) target: String,
}

/// The assembled priority graph (design §5.2). The cordage `Graph`, the
/// `EntityKey ↔ NodeId` projection, the per-node attributes, the consequence pre-pass
/// tally, the two dep/seq overlay handles, the reference/lineage overlay set (the
/// consequence inputs), and the edge-pass danglers. Opaque cordage ids never escape a
/// `pub(crate)` signature.
pub(crate) struct PriorityGraph {
    pub(crate) graph: Graph,
    pub(crate) projection: Projection<EntityKey>,
    pub(crate) attrs: BTreeMap<EntityKey, NodeAttr>,
    pub(crate) consequence: BTreeMap<EntityKey, u32>,
    pub(crate) dep_overlay: OverlayId,
    pub(crate) seq_overlay: OverlayId,
    /// The reference/lineage consequence-input overlays. Their inbound contribution is
    /// already tallied into `consequence` during build, so no post-build surface reads
    /// these handles; kept for completeness + the graph tests' overlay-identity
    /// assertions.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "consequence is pre-tallied into `consequence`; the ref overlay handles \
                      have no post-build reader yet (exercised by the graph tests)"
        )
    )]
    pub(crate) ref_overlays: Vec<OverlayId>,
    /// The edge-pass danglers (unresolved outbound targets). No PHASE-03 surface reads
    /// them yet; exercised by the graph tests, kept for a future dangler query.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "no PHASE-03 surface reads the priority danglers yet (exercised by the \
                      graph tests; kept for a future per-entity dangler query)"
        )
    )]
    pub(crate) dangling: Vec<Dangling>,
}

/// The reference/lineage relation labels that back a consequence-input overlay — the
/// SL-046 overlay-backed labels MINUS the two target-unvalidated ones (`Drift`/
/// `DecisionRef`, which never resolve). One `Reject`/`Unbounded` overlay each; these
/// are the `ref_overlays`. Label is overlay identity (the same label from different
/// source kinds shares ONE overlay).
const REF_LABELS: &[RelationLabel] = &[
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

/// The WORK/LINEAGE label subset whose inbound references count toward consequence
/// (design §5.2, EX-3). `reviews`/`owning_slice` are bookkeeping and EXCLUDED; the
/// two target-unvalidated labels never resolve and so cannot contribute anyway.
const CONSEQUENCE_LABELS: &[RelationLabel] = &[
    RelationLabel::Specs,
    RelationLabel::Requirements,
    RelationLabel::Slices,
    RelationLabel::DescendsFrom,
    RelationLabel::Parent,
    RelationLabel::Members,
];

/// Whether a label counts toward the consequence tally (the work/lineage subset).
fn counts_toward_consequence(label: RelationLabel) -> bool {
    CONSEQUENCE_LABELS.contains(&label)
}

/// Build the priority graph once (design §5.2). The exact build order breaks the
/// mint-order ↔ consequence ↔ graph cycle:
///
/// 1. **Scan** via the `relation_graph` seam → entity set + each entity's outbound
///    edges + RAW authored status (RV derived, REC `None`).
/// 2. **Consequence pre-pass** — tally inbound targets of the WORK/LINEAGE label
///    subset ONLY ([`CONSEQUENCE_LABELS`]) into a `BTreeMap<EntityKey, u32>`, directly
///    from the scanned outbound edges (no graph needed yet). `reviews`/`owning_slice`
///    excluded (EX-3).
/// 3. **Mint** every node into the projection in `(consequence desc, canonical-id
///    asc)` order — the monotonic `NodeId` is the order key's tier-3 fallback. A
///    dedicated pre-intern pass (the `backlog_order` C4 discipline): mint EVERY node
///    first, distinct keys asserted, THEN resolve+emit edges (resolve is get-only,
///    never intern inside the edge pass).
/// 4. **Edges** — reference/lineage onto `ref_overlays` (resolve-only; unresolved →
///    dangler). `needs` → `dep_overlay` (`Reject`, oriented prereq→src i.e. B→A flip,
///    `EdgeAttrs::new(0, 0)`). `after` → `seq_overlay` (`Evict`, `EdgeAttrs::new(rank,
///    age)`). The dep/seq edges read kind-agnostically (DD-2) — today only backlog
///    authors them.
/// 5. `OrderSpec::new([dep Along, seq Along])`, then `builder.build()`.
///
/// # Errors
///
/// Propagates a scan/read error, or an internal cordage rejection of well-formed
/// adapter input (an adapter bug, not a recoverable condition).
pub(crate) fn build(root: &std::path::Path) -> anyhow::Result<PriorityGraph> {
    // 1. Scan — the shared seam (KINDS table / id-ascending, permutation-invariant).
    let scanned = relation_graph::scan_entities(root)?;

    // 2. Consequence pre-pass — tally inbound work/lineage references directly from
    //    the scanned outbound edges (in-memory, derived; ADR-004 stores no reverse).
    //    A consequence edge counts only when its target resolves to a SCANNED entity
    //    (a real node); free-text / dangling targets contribute nothing.
    let keys: std::collections::BTreeSet<EntityKey> = scanned.iter().map(|e| e.key).collect();
    let mut consequence: BTreeMap<EntityKey, u32> = BTreeMap::new();
    for entity in &scanned {
        for edge in &entity.outbound {
            if !counts_toward_consequence(edge.label) {
                continue;
            }
            if let Ok((kref, tid)) = integrity::parse_canonical_ref(&edge.target) {
                let target = EntityKey {
                    prefix: kref.kind.prefix,
                    id: tid,
                };
                if keys.contains(&target) {
                    *consequence.entry(target).or_insert(0) += 1;
                }
            }
        }
    }

    // 3. Mint — (consequence desc, canonical-id asc). The monotonic NodeId is the
    //    tier-3 fallback (the within-level allocation key). Pre-intern EVERY node in
    //    this order BEFORE any edge resolves (C4), asserting distinct keys.
    let mut order: Vec<EntityKey> = scanned.iter().map(|e| e.key).collect();
    order.sort_by(|a, b| {
        let ca = consequence.get(a).copied().unwrap_or(0);
        let cb = consequence.get(b).copied().unwrap_or(0);
        cb.cmp(&ca).then_with(|| a.cmp(b))
    });

    let mut builder = GraphBuilder::new();
    // Reference/lineage overlays (the consequence inputs) + the two dep/seq overlays.
    // Capture every OverlayId from the builder — never fabricate an id.
    let mut ref_by_label: BTreeMap<RelationLabel, OverlayId> = BTreeMap::new();
    let mut ref_overlays: Vec<OverlayId> = Vec::new();
    for &label in REF_LABELS {
        let ov = builder.overlay(OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded));
        ref_by_label.insert(label, ov);
        ref_overlays.push(ov);
    }
    let dep_overlay = builder.overlay(OverlayConfig::new(CyclePolicy::Reject, Arity::Unbounded));
    let seq_overlay = builder.overlay(OverlayConfig::new(CyclePolicy::Evict, Arity::Unbounded));

    let mut projection: Projection<EntityKey> = Projection::new();
    for &key in &order {
        assert!(
            projection.resolve(key).is_none(),
            "priority::graph: duplicate EntityKey {} (canonical ids unique by prefix)",
            key.canonical()
        );
        projection.intern(&mut builder, key);
    }

    // 3b. Read each backlog entity's dep/seq + promoted ONCE (kind-agnostically,
    //     DD-2 — the per-kind dispatch finds nothing for non-backlog kinds), so the
    //     attrs pass and the edge pass share one read per item (no double parse).
    let mut dep_seq: BTreeMap<EntityKey, backlog::DepSeq> = BTreeMap::new();
    for entity in &scanned {
        if let Some(item_kind) = backlog::kind_from_prefix(entity.key.prefix) {
            dep_seq.insert(
                entity.key,
                backlog::dep_seq_for(root, item_kind, entity.key.id)?,
            );
        }
    }

    // 3c. Per-node attributes — RAW authored status verbatim, kind, promoted. Only a
    //     backlog item can be `promoted`; every other kind is never promoted.
    let mut attrs: BTreeMap<EntityKey, NodeAttr> = BTreeMap::new();
    for entity in &scanned {
        attrs.insert(
            entity.key,
            NodeAttr {
                kind: entity.kind,
                status: entity.status.clone(),
                promoted: dep_seq.get(&entity.key).is_some_and(|ds| ds.promoted),
                title: entity.title.clone(),
            },
        );
    }

    // 4. Edges — resolve-only (never intern inside the edge pass).
    let mut dangling: Vec<Dangling> = Vec::new();
    for entity in &scanned {
        let Some(src) = projection.resolve(entity.key) else {
            debug_assert!(false, "priority::graph: edge-pass key not interned");
            continue;
        };

        // Reference/lineage edges onto ref_overlays (consequence inputs). Unresolved
        // or no-overlay (target-unvalidated) targets → dangler.
        for edge in &entity.outbound {
            if let Some(dst) = resolve(&projection, &edge.target)
                && let Some(&ov) = ref_by_label.get(&edge.label)
            {
                builder.edge(ov, src, dst, EdgeAttrs::new(0, 0));
            } else {
                dangling.push(Dangling {
                    source: entity.key,
                    label: Some(edge.label),
                    target: edge.target.clone(),
                });
            }
        }

        // dep/seq edges — kind-agnostic (DD-2): present only for backlog entities.
        if let Some(ds) = dep_seq.get(&entity.key) {
            // `A.needs = [B]` ⇒ B must precede A: edge B→A (the flip), hard, never
            // evicts. An unresolved prereq is a dangler (no node to edge from).
            for prereq_ref in &ds.needs {
                match resolve(&projection, prereq_ref) {
                    Some(prereq) => builder.edge(dep_overlay, prereq, src, EdgeAttrs::new(0, 0)),
                    None => dangling.push(Dangling {
                        source: entity.key,
                        label: None, // dep (needs) is item→item, no RelationLabel class
                        target: prereq_ref.clone(),
                    }),
                }
            }
            // `A.after = [{to=B, rank}]` ⇒ B before A: edge B→A carrying the genuine
            // `(rank, age)` eviction key; `age` is the entry's index in this item's
            // `after` array (the `backlog_order` discipline).
            for (idx, (to_ref, rank)) in ds.after.iter().enumerate() {
                match resolve(&projection, to_ref) {
                    Some(prereq) => {
                        let age = u64::try_from(idx).map_err(|e| {
                            anyhow::anyhow!("priority::graph: after-edge index overflows u64: {e}")
                        })?;
                        builder.edge(seq_overlay, prereq, src, EdgeAttrs::new(*rank, age));
                    }
                    None => dangling.push(Dangling {
                        source: entity.key,
                        label: None, // seq (after) is item→item, no RelationLabel class
                        target: to_ref.clone(),
                    }),
                }
            }
        }
    }

    // 5. OrderSpec over [dep Along, seq Along], then build.
    builder.order_spec(OrderSpec::new(vec![
        OrderLayer::new(dep_overlay, Direction::Along),
        OrderLayer::new(seq_overlay, Direction::Along),
    ]));

    let graph = builder.build().map_err(|e| {
        anyhow::anyhow!(
            "priority::graph: cordage rejected well-formed adapter input (internal bug): {e:?}"
        )
    })?;

    Ok(PriorityGraph {
        graph,
        projection,
        attrs,
        consequence,
        dep_overlay,
        seq_overlay,
        ref_overlays,
        dangling,
    })
}

/// Get-only resolve of an authored ref string to a minted node, or `None`. A ref
/// that fails to parse as a canonical ref (free-text), or parses to an id never
/// minted (no entity dir), is `None` → a dangler. NEVER interns.
fn resolve(projection: &Projection<EntityKey>, reference: &str) -> Option<cordage::NodeId> {
    let (kref, id) = integrity::parse_canonical_ref(reference).ok()?;
    projection.resolve(EntityKey {
        prefix: kref.kind.prefix,
        id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Write `root/<rel>` with `body`, creating parents.
    fn write(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    /// Seed a slice (toml + md) with a `[relationships]` body.
    fn seed_slice(root: &Path, id: u32, rels: &str) {
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"s\"\ntitle = \"S\"\nstatus = \"proposed\"\n\
                 created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n[relationships]\n{rels}"
            ),
        );
        write(
            root,
            &format!(".doctrine/slice/{id:03}/slice-{id:03}.md"),
            "scope\n",
        );
    }

    /// Seed a requirement (an edge target only — has a top-level status).
    fn seed_requirement(root: &Path, id: u32) {
        write(
            root,
            &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.toml"),
            &format!("id = {id}\nslug = \"r\"\ntitle = \"R\"\nstatus = \"active\"\n"),
        );
        write(
            root,
            &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.md"),
            "r\n",
        );
    }

    /// Seed a backlog issue with a `[relationships]` body and a `resolution`.
    fn seed_issue(root: &Path, id: u32, status: &str, resolution: &str, rels: &str) {
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"{status}\"\n\
                 resolution = \"{resolution}\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 [relationships]\n{rels}"
            ),
        );
        write(
            root,
            &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.md"),
            "b\n",
        );
    }

    /// Seed a risk backlog item (so a second backlog kind exists for dep/seq).
    fn seed_risk(root: &Path, id: u32, status: &str, rels: &str) {
        write(
            root,
            &format!(".doctrine/backlog/risk/{id:03}/backlog-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"k\"\ntitle = \"K\"\nkind = \"risk\"\nstatus = \"{status}\"\n\
                 resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 [relationships]\n{rels}"
            ),
        );
        write(
            root,
            &format!(".doctrine/backlog/risk/{id:03}/backlog-{id:03}.md"),
            "k\n",
        );
    }

    /// Seed a reconciliation record (status-LESS by design).
    fn seed_rec(root: &Path, id: u32, owning_slice: &str) {
        write(
            root,
            &format!(".doctrine/rec/{id:03}/rec-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"r\"\ntitle = \"R\"\n\
                 [rec]\nmove = \"accept\"\nowning_slice = \"{owning_slice}\"\n"
            ),
        );
    }

    /// Seed a review (status-LESS authored; status derived from findings).
    fn seed_review(root: &Path, id: u32, target: &str, findings: &str) {
        write(
            root,
            &format!(".doctrine/review/{id:03}/review-{id:03}.toml"),
            &format!(
                "id = {id}\nslug = \"r\"\ntitle = \"R\"\n\
                 [review]\nfacet = \"reconciliation\"\nraiser = \"a\"\nresponder = \"b\"\n\
                 [target]\nref = \"{target}\"\n{findings}"
            ),
        );
    }

    fn key(prefix: &'static str, id: u32) -> EntityKey {
        EntityKey { prefix, id }
    }

    // -- VT-1: builds; node set equals the scanned set; distinct keys ----------

    #[test]
    fn builds_over_multi_kind_corpus_node_set_equals_scanned() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, "requirements = [\"REQ-005\"]\n");
        seed_requirement(root, 5);
        seed_issue(root, 1, "open", "", "slices = [\"SL-001\"]\n");
        seed_rec(root, 1, "SL-001");
        seed_review(root, 1, "SL-001", "");

        let pg = build(root).unwrap();
        // Node set equals the scanned entity set (one NodeAttr per scanned entity).
        let scanned: std::collections::BTreeSet<EntityKey> = relation_graph::scan_entities(root)
            .unwrap()
            .iter()
            .map(|e| e.key)
            .collect();
        let minted: std::collections::BTreeSet<EntityKey> = pg.attrs.keys().copied().collect();
        assert_eq!(minted, scanned, "every scanned entity is a node");
        // Each key resolves (distinct keys, all interned).
        for k in &scanned {
            assert!(
                pg.projection.resolve(*k).is_some(),
                "{} minted",
                k.canonical()
            );
        }
        assert_eq!(pg.attrs.len(), scanned.len());
        // NodeAttr.kind carries the kind descriptor (its prefix matches the key).
        for (k, attr) in &pg.attrs {
            assert_eq!(
                attr.kind.prefix, k.prefix,
                "NodeAttr.kind matches the key prefix"
            );
        }
        // ref_overlays is the SL-046 reference/lineage set (one per REF_LABELS entry),
        // distinct from the dep/seq overlay handles.
        assert_eq!(
            pg.ref_overlays.len(),
            REF_LABELS.len(),
            "one ref overlay per reference/lineage label"
        );
        assert!(!pg.ref_overlays.contains(&pg.dep_overlay));
        assert!(!pg.ref_overlays.contains(&pg.seq_overlay));
    }

    // -- VT-1 + EX-2: NodeAttr status/promoted reads -------------------------

    #[test]
    fn node_attr_status_promoted_per_kind() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, "");
        seed_requirement(root, 5);
        // A promoted issue (resolution == promoted) vs a plain open one.
        seed_issue(root, 1, "resolved", "promoted", "");
        seed_issue(root, 2, "open", "", "");
        seed_rec(root, 1, "SL-001");
        // A review with one OPEN finding ⇒ derived status "active".
        seed_review(
            root,
            1,
            "SL-001",
            "[[finding]]\nid = \"F-1\"\nstatus = \"open\"\nseverity = \"minor\"\n\
             title = \"t\"\ndetail = \"d\"\n",
        );
        // A review with all VERIFIED ⇒ derived status "done".
        seed_review(
            root,
            2,
            "SL-001",
            "[[finding]]\nid = \"F-1\"\nstatus = \"verified\"\nseverity = \"minor\"\n\
             title = \"t\"\ndetail = \"d\"\n",
        );

        let pg = build(root).unwrap();
        // Slice carries its raw authored status.
        assert_eq!(pg.attrs[&key("SL", 1)].status.as_deref(), Some("proposed"));
        assert!(!pg.attrs[&key("SL", 1)].promoted);
        // Requirement carries its top-level status.
        assert_eq!(pg.attrs[&key("REQ", 5)].status.as_deref(), Some("active"));
        // REC is status-less.
        assert_eq!(pg.attrs[&key("REC", 1)].status, None);
        // Promoted issue: flag set, status raw "resolved".
        assert_eq!(pg.attrs[&key("ISS", 1)].status.as_deref(), Some("resolved"));
        assert!(
            pg.attrs[&key("ISS", 1)].promoted,
            "resolution=promoted ⇒ promoted"
        );
        // Plain issue: not promoted.
        assert!(!pg.attrs[&key("ISS", 2)].promoted);
        // RV status is DERIVED, not stored.
        assert_eq!(pg.attrs[&key("RV", 1)].status.as_deref(), Some("active"));
        assert_eq!(pg.attrs[&key("RV", 2)].status.as_deref(), Some("done"));
    }

    // -- VT-2: consequence counts ONLY work/lineage labels --------------------

    #[test]
    fn consequence_counts_only_work_lineage_labels() {
        let dir = tmp();
        let root = dir.path();
        // SL-001 is the consequence target. A `requirements` edge (work/lineage) →
        // counts. A `reviews` edge (bookkeeping) and an `owning_slice` edge → do NOT.
        seed_slice(root, 1, "");
        // SL-002 requires SL-001? requirements target a REQ, not a slice; use a real
        // lineage label: a spec descends_from / parent target a spec. Simpler: an
        // issue's `slices` edge (work/lineage) onto SL-001 counts.
        seed_issue(root, 1, "open", "", "slices = [\"SL-001\"]\n");
        // A review targeting SL-001 (reviews edge) — bookkeeping, must NOT count.
        seed_review(root, 1, "SL-001", "");
        // A rec owning SL-001 (owning_slice edge) — bookkeeping, must NOT count.
        seed_rec(root, 1, "SL-001");

        let pg = build(root).unwrap();
        // Exactly ONE work/lineage inbound (the issue's slices edge); the reviews and
        // owning_slice edges are excluded.
        assert_eq!(
            pg.consequence.get(&key("SL", 1)).copied().unwrap_or(0),
            1,
            "only the slices edge counts; reviews + owning_slice excluded"
        );
    }

    #[test]
    fn lineage_edge_raises_consequence() {
        let dir = tmp();
        let root = dir.path();
        // A requirements edge (work/lineage) raises the target REQ's consequence.
        seed_slice(root, 1, "requirements = [\"REQ-005\", \"REQ-006\"]\n");
        seed_slice(root, 2, "requirements = [\"REQ-005\"]\n");
        seed_requirement(root, 5);
        seed_requirement(root, 6);
        let pg = build(root).unwrap();
        assert_eq!(pg.consequence.get(&key("REQ", 5)).copied(), Some(2));
        assert_eq!(pg.consequence.get(&key("REQ", 6)).copied(), Some(1));
        // An entity nobody references has no consequence entry.
        assert_eq!(pg.consequence.get(&key("SL", 1)).copied().unwrap_or(0), 0);
    }

    // -- VT-3: mint order is (consequence desc, canonical asc); permutation-inv -

    #[test]
    fn mint_order_consequence_desc_then_canonical_asc_and_permutation_invariant() {
        let dir = tmp();
        let root = dir.path();
        // REQ-006 referenced twice, REQ-005 once, REQ-007 zero. Mint order should be
        // REQ-006 (2), REQ-005 (1), then REQ-007 (0) — consequence desc, ties by
        // canonical asc. Slices that author the edges have consequence 0, ordered after
        // by canonical id.
        seed_slice(root, 1, "requirements = [\"REQ-006\"]\n");
        seed_slice(root, 2, "requirements = [\"REQ-005\", \"REQ-006\"]\n");
        seed_requirement(root, 5);
        seed_requirement(root, 6);
        seed_requirement(root, 7);

        let pg = build(root).unwrap();
        // NodeId reflects mint order: lower NodeId = minted earlier. Compare the three
        // requirements: REQ-006 < REQ-005 < REQ-007 by NodeId (consequence desc).
        let n6 = pg.projection.resolve(key("REQ", 6)).unwrap();
        let n5 = pg.projection.resolve(key("REQ", 5)).unwrap();
        let n7 = pg.projection.resolve(key("REQ", 7)).unwrap();
        assert!(n6 < n5, "REQ-006 (consequence 2) mints before REQ-005 (1)");
        assert!(n5 < n7, "REQ-005 (1) mints before REQ-007 (0)");

        // Permutation invariance: the consequence map and the mint order are identical
        // regardless of on-disk scan order (BTree, no clock/RNG). Re-seed the same
        // corpus in a fresh dir in a DIFFERENT authoring order and compare.
        let dir2 = tmp();
        let root2 = dir2.path();
        seed_requirement(root2, 7);
        seed_requirement(root2, 6);
        seed_requirement(root2, 5);
        seed_slice(root2, 2, "requirements = [\"REQ-006\", \"REQ-005\"]\n");
        seed_slice(root2, 1, "requirements = [\"REQ-006\"]\n");
        let pg2 = build(root2).unwrap();
        assert_eq!(
            pg.consequence, pg2.consequence,
            "consequence is permutation-invariant"
        );
        // Same relative NodeId order for the three requirements.
        let m6 = pg2.projection.resolve(key("REQ", 6)).unwrap();
        let m5 = pg2.projection.resolve(key("REQ", 5)).unwrap();
        let m7 = pg2.projection.resolve(key("REQ", 7)).unwrap();
        assert!(m6 < m5 && m5 < m7, "mint order is permutation-invariant");
    }

    // -- EX-4: dep/seq edges + danglers (kind-agnostic emission) ---------------

    #[test]
    fn dep_seq_edges_emitted_for_backlog_unresolved_dangles() {
        let dir = tmp();
        let root = dir.path();
        // ISS-001 needs RSK-001 (resolvable) and ISS-099 (dangling); after ISS-002.
        seed_issue(
            root,
            1,
            "open",
            "",
            "needs = [\"RSK-001\", \"ISS-099\"]\nafter = [{ to = \"ISS-002\", rank = 0 }]\n",
        );
        seed_issue(root, 2, "open", "", "");
        seed_risk(root, 1, "open", "");

        let pg = build(root).unwrap();
        // The dep overlay carries the resolvable needs edge (RSK-001 → ISS-001, the
        // B→A flip): RSK-001 is a predecessor of ISS-001 in `dep`.
        let iss1 = pg.projection.resolve(key("ISS", 1)).unwrap();
        let rsk1 = pg.projection.resolve(key("RSK", 1)).unwrap();
        let dep_preds: Vec<_> = pg
            .graph
            .in_edges(pg.dep_overlay, iss1)
            .map(|(s, _)| s)
            .collect();
        assert!(
            dep_preds.contains(&rsk1),
            "needs edge oriented prereq→src (B→A)"
        );
        // The unresolved ISS-099 needs ref dangles.
        assert!(
            pg.dangling
                .iter()
                .any(|d| d.source == key("ISS", 1) && d.target == "ISS-099"),
            "unresolved needs prereq dangles"
        );
        // The after edge (ISS-002 → ISS-001) lands on the seq overlay.
        let iss2 = pg.projection.resolve(key("ISS", 2)).unwrap();
        let seq_preds: Vec<_> = pg
            .graph
            .in_edges(pg.seq_overlay, iss1)
            .map(|(s, _)| s)
            .collect();
        assert!(
            seq_preds.contains(&iss2),
            "after edge oriented predecessor→src"
        );
    }

    #[test]
    fn non_backlog_nodes_carry_no_dep_seq_edges() {
        let dir = tmp();
        let root = dir.path();
        // Slices author no needs/after — the kind-agnostic read finds nothing; no
        // panic, no edge. (DD-2: dormant for non-backlog.)
        seed_slice(root, 1, "");
        seed_slice(root, 2, "supersedes = [\"SL-001\"]\n");
        let pg = build(root).unwrap();
        let sl1 = pg.projection.resolve(key("SL", 1)).unwrap();
        assert_eq!(pg.graph.in_edges(pg.dep_overlay, sl1).count(), 0);
        assert_eq!(pg.graph.in_edges(pg.seq_overlay, sl1).count(), 0);
        // But the reference edge (supersedes) DID land on a ref overlay → consequence
        // is unaffected (supersedes is not a consequence label) but the edge exists.
        assert!(
            pg.dangling.iter().all(|d| d.target != "SL-001"),
            "SL-001 ref resolves, no dangle"
        );
    }

    // -- A free-text / no-overlay outbound target dangles ----------------------

    #[test]
    fn free_text_outbound_target_dangles() {
        let dir = tmp();
        let root = dir.path();
        // A backlog drift edge is target-unvalidated (no overlay) → always dangles.
        seed_issue(root, 1, "open", "", "drift = [\"some-free-text\"]\n");
        let pg = build(root).unwrap();
        assert!(
            pg.dangling.iter().any(|d| d.source == key("ISS", 1)
                && d.label == Some(RelationLabel::Drift)
                && d.target == "some-free-text"),
            "free-text drift dangles"
        );
    }
}
