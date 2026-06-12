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

use cordage::{Arity, CyclePolicy, EdgeAttrs, Graph, GraphBuilder, OverlayConfig, OverlayId};

use crate::entity;
use crate::integrity;
use crate::listing::{self, Format};
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
// PHASE-03 — the all-kind scan, the reference overlays, and the inspect query.
// ---------------------------------------------------------------------------

/// The projection key for a numbered entity (design §5.2). Stores the kind's
/// `&'static str` prefix — `Copy + Ord`, unlike `entity::Kind` (which is data, not
/// `Ord`, and carries a fn-ptr `scaffold`) — and the numeric id. The pair is the
/// corpus-wide identity, and renders its canonical ref through the same
/// `listing::canonical_id` source `ItemId` uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
/// REQ-077), then per entity read its AUTHORED status ([`status_for`]) and its
/// authored outbound edges ([`outbound_for`]). Yields entities in KINDS-table /
/// id-ascending order — the SAME order `build_relation_graph`'s old pass-1 minted in,
/// so `inspect`'s mint order (and therefore its byte-identical output) is preserved.
///
/// Disk touches live here (the thin imperative shell — `scan_ids`/`status_for`/
/// `outbound_for` read the entity tomls); a consumer's tally/mint/edge policy stays
/// pure over the returned `Vec`.
pub(crate) fn scan_entities(root: &Path) -> anyhow::Result<Vec<ScannedEntity>> {
    let mut out = Vec::new();
    for kref in integrity::KINDS {
        let prefix = kref.kind.prefix;
        let mut ids = entity::scan_ids(&root.join(kref.kind.dir))?;
        ids.sort_unstable();
        for id in ids {
            out.push(ScannedEntity {
                key: EntityKey { prefix, id },
                kind: kref.kind,
                status: status_for(root, kref, id)?,
                title: title_for(root, kref, id)?,
                outbound: outbound_for(root, kref.kind, id)?,
            });
        }
    }
    Ok(out)
}

/// One entity's AUTHORED status string for the cross-kind scan, dispatched by
/// canonical prefix (the same data-driven shape as [`outbound_for`]). REC is
/// genuinely status-less (one record per act, no lifecycle) ⇒ `None`. RV authors no
/// `status` field either, but carries a status DERIVED at read time from its
/// authored finding ledger (`review::derived_status_string`, D-C8) — authored-tier,
/// not a runtime read. Every other kind stores `status` top-level in its
/// `<stem>-NNN.toml`, read through the shared `meta::read_meta` (one reader, no new
/// parse). The `kref` carries both the tree dir and the toml `stem`.
fn status_for(root: &Path, kref: &integrity::KindRef, id: u32) -> anyhow::Result<Option<String>> {
    match kref.kind.prefix {
        // Status-less by design — no diagnostic, just absent.
        "REC" => Ok(None),
        // Derived (authored-tier) over the finding ledger, never stored.
        "RV" => Ok(Some(crate::review::derived_status_string(root, id)?)),
        // Every other kind stores `status` top-level — the shared status reader.
        _ => {
            let tree_root = root.join(kref.kind.dir);
            Ok(Some(
                crate::meta::read_meta(&tree_root, kref.stem, id)?.status,
            ))
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

/// Build the cross-kind relation graph once (design §5.4 — mirrors
/// `backlog_order::build`). A SEPARATE cordage `Graph` from `backlog_order`: they
/// share the `Projection` *type*, never a graph instance or a scan.
///
/// Re-expressed on the [`scan_entities`] seam (SL-047 D5): the KINDS-walk raw scan is
/// the shared seam; this fn only builds the REFERENCE-overlay graph on top of it. The
/// scan order (KINDS table / id ascending) is unchanged, so the mint order — and thus
/// the byte-identical `inspect` output (VT-4 behaviour-preservation gate) — is
/// preserved exactly.
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
fn build_relation_graph(root: &Path) -> anyhow::Result<RelationGraph> {
    let scanned = scan_entities(root)?;

    let mut builder = GraphBuilder::new();
    let overlays = OverlayMap::build(&mut builder);
    let mut projection: Projection<EntityKey> = Projection::new();

    // Pass 1 — mint every entity's node (scan order: KINDS table, ids ascending).
    for entity in &scanned {
        projection.intern(&mut builder, entity.key);
    }

    // Pass 2 — emit edges (resolve only, never intern) and collect danglers.
    let mut danglers: BTreeMap<EntityKey, Vec<(RelationLabel, String)>> = BTreeMap::new();
    for entity in &scanned {
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
pub(crate) struct InspectView {
    pub(crate) id: String,
    pub(crate) outbound: Vec<(RelationLabel, Vec<String>)>,
    pub(crate) inbound: Vec<(RelationLabel, Vec<String>)>,
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
/// A well-formed ref to a non-existent id (never minted) returns an empty-section
/// view, not an error (VT-5 — mirrors a `show`-like read surface over an empty
/// entity). NEVER reads `graph.provenance()` (C7 — a benign symmetric-`related`
/// 2-cycle yields a `Reject` `CycleDiagnostic` that must not leak into the view).
pub(crate) fn inspect(root: &Path, id: &str) -> anyhow::Result<InspectView> {
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
            // order is mint order, not ref order — sort for a deterministic,
            // permutation-invariant render (REQ-077). NOTE: this is a LEXICAL sort of
            // the canonical-ref strings, which equals numeric order only while every
            // namespace stays below id 1000 (zero-pad is min-3, not fixed-width):
            // "SL-1000" sorts before "SL-999". True numeric order is RSK-007.
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

// ---------------------------------------------------------------------------
// PHASE-04 — the `inspect <ID>` command: render (human + --json) and the shell.
// ---------------------------------------------------------------------------

/// Render the relation view of `id` to a string (build the view once, format per
/// `Format`) WITHOUT printing — the command-layer seam (SL-047 §5.4): `main.rs`'s
/// `inspect` handler calls this for the relation portion, then APPENDS the priority
/// actionability block BELOW it (the composition lives at the command layer, which
/// alone may depend on both `relation_graph` and `priority`; ADR-001 forbids
/// `relation_graph` from calling up into `priority`). The relation portion stays
/// byte-identical — the appended block is additive (EX-2 / VT-2 behaviour-preserving).
/// No trailing newline on JSON (the golden contract); the human surface ends in `\n`.
pub(crate) fn render(root: &Path, id: &str, format: Format) -> anyhow::Result<String> {
    let view = inspect(root, id)?;
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
    for (label, targets) in &view.outbound {
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
        parts.push(format!("  {}: {}\n", label.name(), rendered.join(", ")));
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
    for (label, srcs) in &view.inbound {
        let word = if *label == RelationLabel::Supersedes {
            "superseded by"
        } else {
            label.name()
        };
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
    let group = |label: RelationLabel, targets: &[String]| serde_json::json!({ "label": label.name(), "targets": targets });
    let outbound: Vec<serde_json::Value> =
        view.outbound.iter().map(|(l, t)| group(*l, t)).collect();
    let inbound: Vec<serde_json::Value> = view.inbound.iter().map(|(l, t)| group(*l, t)).collect();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrity::KINDS;
    use crate::relation::RelationLabel;
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
    fn slice_outbound_specs_requirements_supersedes() {
        let dir = tmp();
        let root = dir.path();
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
        let dir = tmp();
        let root = dir.path();
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

    // -- VT-2 exclusion proof (REC decision_ref carried, not dropped) --------

    #[test]
    fn rec_decision_ref_carried_as_free_text_not_dropped() {
        let dir = tmp();
        let root = dir.path();
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
        let dir = tmp();
        let root = dir.path();
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
        let dir = tmp();
        let root = dir.path();
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
        let dir = tmp();
        let root = dir.path();
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
        let dir = tmp();
        let root = dir.path();
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
        let dir = tmp();
        let root = dir.path();
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
        let dir = tmp();
        let root = dir.path();
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
