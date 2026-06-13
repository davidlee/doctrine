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
use crate::relation::{RELATION_RULES, RelationEdge, RelationLabel, TargetSpec};

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
    let scanned = scan_entities(root)?;

    // (1) danglers — a validated target that fails to resolve.
    for entity in &scanned {
        for edge in &entity.outbound {
            // Skip `Unvalidated` labels — their targets dangle by design.
            let validated = crate::relation::lookup(entity.kind, edge.label)
                .is_some_and(|r| !matches!(r.target, crate::relation::TargetSpec::Unvalidated));
            if validated && integrity::ensure_ref_resolves(root, &edge.target).is_err() {
                findings.push(format!(
                    "{}: `{}` target `{}` does not resolve (dangling [[relation]] edge)",
                    entity.key.canonical(),
                    edge.label.name(),
                    edge.target
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
            let name = format!("{id:03}");
            let toml_path = root
                .join(kref.kind.dir)
                .join(&name)
                .join(format!("{}-{name}.toml", kref.stem));
            let text = std::fs::read_to_string(&toml_path)
                .map_err(|e| anyhow::anyhow!("read {} for validate: {e}", toml_path.display()))?;
            let doc = crate::relation::RelationDoc::parse(&text)?;
            let (_edges, illegal) = crate::relation::read_block(kref.kind, &doc);
            for row in illegal {
                let why = match row.reason {
                    crate::relation::IllegalReason::UnknownLabel => "unknown label",
                    crate::relation::IllegalReason::IllegalForSource => "label illegal for source",
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
    inspect_from(&scan_entities(root)?, root, id)
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
        // Table-driven inbound render text (design §5.5 X5 / R2-M3): the `supersedes` →
        // "superseded by" special-case collapses into `relation::inbound_name`, which
        // also renders `governed_by` → "governs", `consumes` → "consumed_by". Legacy
        // labels carry `inbound_name == name()`, so shipped goldens are unchanged. The
        // `--json` inbound keeps the raw label (`render_json`), per R2-M3.
        let word = crate::relation::inbound_name(*label);
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
            // SL-048 PHASE-04: tier-1 axes migrated to `[[relation]]` rows.
            "id = 1\nslug = \"a\"\ntitle = \"A\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [[relation]]\nlabel = \"specs\"\ntarget = \"PRD-010\"\n\
             [[relation]]\nlabel = \"requirements\"\ntarget = \"REQ-001\"\n\
             [[relation]]\nlabel = \"requirements\"\ntarget = \"REQ-002\"\n\
             [[relation]]\nlabel = \"supersedes\"\ntarget = \"SL-000\"\n",
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
            // SL-048 PHASE-04 (OD-3): supersedes/superseded_by/tags stay TYPED in a
            // `[relationships]` table preceding the arrays; only `related` migrates.
            "id = 2\nslug = \"a\"\ntitle = \"A\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = [\"ADR-001\"]\nsuperseded_by = [\"ADR-009\"]\n\
             tags = [\"layering\"]\n\
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
             [[relation]]\nlabel = \"specs\"\ntarget = \"PRD-009\"\n\
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
                (RelationLabel::Specs, "PRD-009"),
                (RelationLabel::Slices, "SL-020"),
                (RelationLabel::Drift, "some-free-text"),
            ],
            "backlog emits slices/specs/drift ONLY (no needs/after/triggers), canonical order"
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

    // -- SL-059 VT-1: outbound_for total-dispatch for the four knowledge kinds --

    #[test]
    fn knowledge_kinds_author_no_outbound_never_panic() {
        let dir = tmp();
        let root = dir.path();
        // F-A1: the four-prefix empty arm returns Ok(vec![]) for each ASM/DEC/QUE/CON
        // and never falls through to the debug_assert!(false) unrouted-prefix panic
        // (the empty-arm regression guard — a KINDS row with no arm would panic here).
        for prefix in ["ASM", "DEC", "QUE", "CON"] {
            let edges = outbound_for(&root, kind_for(prefix), 1).unwrap();
            assert!(edges.is_empty(), "{prefix} authors no outbound");
        }
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
        let scanned = scan_entities(&root).unwrap();
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

    /// Seed an ADR governance entity (SL-048 migrated shape — only `related` moves to
    /// `[[relation]]`; supersedes/superseded_by/tags stay typed, OD-3).
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
            &[("requirements", &["REQ-005"]), ("supersedes", &["SL-001"])],
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
        // the ascending sort + the canonical-ref sort in inspect.
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
            BTreeSet::from([RelationLabel::Drift, RelationLabel::DecisionRef]),
            "the no-overlay pair is exactly drift + decision_ref"
        );
        for label in [RelationLabel::Drift, RelationLabel::DecisionRef] {
            assert!(
                overlays.overlay_for(label).is_none(),
                "{label:?} (Unvalidated) must have no overlay"
            );
        }
        // The 13 = 15 distinct labels minus the 2 Unvalidated. The set, not just the
        // count, is the real assertion above; the count is a human-readable sanity tag.
        assert_eq!(overlay_backed.len(), 13, "overlay-backed label count is 13");
    }

    // -- PHASE-04 VT-4 / X3 arm (a): exact reader coverage (read_block live) ---

    /// The distinct labels `RELATION_RULES` legalises for a given source prefix.
    fn table_labels_for(prefix: &str) -> std::collections::BTreeSet<RelationLabel> {
        use crate::relation::RELATION_RULES;
        RELATION_RULES
            .iter()
            .filter(|r| r.sources.iter().any(|k| k.prefix == prefix))
            .map(|r| r.label)
            .collect()
    }

    /// The distinct labels a kind's live `outbound_for` accessor ACTUALLY emits over a
    /// corpus where every legal axis is authored.
    fn emitted_labels(
        root: &Path,
        prefix: &str,
        id: u32,
    ) -> std::collections::BTreeSet<RelationLabel> {
        outbound_for(root, kind_for(prefix), id)
            .unwrap()
            .iter()
            .map(|e| e.label)
            .collect()
    }

    /// VT-4 (X3 arm (a), now `read_block` is LIVE): per source kind, the label set the
    /// shipped `relation_edges` accessor EMITS == the label set `RELATION_RULES`
    /// legalises for that source — no off-table emission, no table rule without a reader
    /// path. The exact set (not ⊆) is the assertion: a fully-populated fixture authors
    /// one edge of every legal axis (tier-1 via `[[relation]]`, tier-2/3 via its typed
    /// structure), and the emitted distinct-label set must equal the table's.
    #[test]
    fn reader_emitted_labels_equal_table_labels_per_source() {
        let dir = tmp();
        let root = dir.path();

        // --- SL: specs, requirements, supersedes, governed_by (all tier-1) ---
        write(
            &root,
            ".doctrine/slice/001/slice-001.toml",
            "id = 1\nslug = \"s\"\ntitle = \"S\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [[relation]]\nlabel = \"specs\"\ntarget = \"PRD-010\"\n\
             [[relation]]\nlabel = \"requirements\"\ntarget = \"REQ-001\"\n\
             [[relation]]\nlabel = \"supersedes\"\ntarget = \"SL-002\"\n\
             [[relation]]\nlabel = \"governed_by\"\ntarget = \"ADR-001\"\n",
        );
        write(&root, ".doctrine/slice/001/slice-001.md", "s\n");
        assert_eq!(
            emitted_labels(root, "SL", 1),
            table_labels_for("SL"),
            "slice reader emits exactly its table labels"
        );

        // --- ADR (governance): supersedes (typed) + related (tier-1) ---
        write(
            &root,
            ".doctrine/adr/001/adr-001.toml",
            "id = 1\nslug = \"a\"\ntitle = \"A\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = [\"ADR-002\"]\nsuperseded_by = []\ntags = []\n\
             [[relation]]\nlabel = \"related\"\ntarget = \"ADR-003\"\n",
        );
        write(&root, ".doctrine/adr/001/adr-001.md", "a\n");
        assert_eq!(
            emitted_labels(root, "ADR", 1),
            table_labels_for("ADR"),
            "governance reader emits exactly supersedes + related"
        );

        // --- ISS (backlog): specs + slices + drift (all tier-1) ---
        write(
            &root,
            ".doctrine/backlog/issue/001/backlog-001.toml",
            "id = 1\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [[relation]]\nlabel = \"specs\"\ntarget = \"PRD-010\"\n\
             [[relation]]\nlabel = \"slices\"\ntarget = \"SL-001\"\n\
             [[relation]]\nlabel = \"drift\"\ntarget = \"free-text\"\n",
        );
        write(&root, ".doctrine/backlog/issue/001/backlog-001.md", "i\n");
        assert_eq!(
            emitted_labels(root, "ISS", 1),
            table_labels_for("ISS"),
            "backlog reader emits exactly specs + slices + drift"
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
             [rec]\nmove = \"accept\"\nowning_slice = \"SL-001\"\ndecision_ref = \"DEC-001\"\n",
        );
        assert_eq!(
            emitted_labels(root, "REC", 1),
            table_labels_for("REC"),
            "rec reader emits exactly owning_slice + decision_ref"
        );
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
        seed_slice(root, 1, &[("requirements", &["REQ-005", "REQ-999"])]);
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
            "schema = \"doctrine.backlog\"\nversion = 1\n\
             id = 1\nslug = \"i\"\ntitle = \"I\"\nkind = \"issue\"\nstatus = \"open\"\n\
             resolution = \"\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\ntags = []\n\
             [[relation]]\nlabel = \"drift\"\ntarget = \"loose talk\"\n\
             [[relation]]\nlabel = \"slices\"\ntarget = \"SL-001\"\n",
        );
        write(root, ".doctrine/backlog/issue/001/backlog-001.md", "i\n");
        // SL-002 carries a HAND-EDITED illegal row: a slice cannot author `related`.
        write(
            root,
            ".doctrine/slice/002/slice-002.toml",
            "id = 2\nslug = \"s\"\ntitle = \"S\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [[relation]]\nlabel = \"related\"\ntarget = \"SL-001\"\n",
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
            "the hand-edited illegal `related` row is reported: {joined}"
        );
        // Report-only: the corpus file is byte-unchanged.
        let after =
            std::fs::read_to_string(root.join(".doctrine/slice/002/slice-002.toml")).unwrap();
        assert!(
            after.contains("label = \"related\""),
            "validate never rewrites the corpus"
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
}
