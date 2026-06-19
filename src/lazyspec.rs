// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine lazyspec` — the pure projection from the loaded doctrine corpus to a
//! single lazyspec **Brief** (SL-026 PHASE-01).
//!
//! This module owns the locked JSON wire shape and the total, side-effect-free
//! [`project`] that folds a pre-loaded [`Corpus`] into a [`Brief`]. It is a pure
//! leaf: no clock, rng, git, or disk — `now`/`version`/`project` arrive as data
//! and every node the corpus carries is already loaded (the loaders land in a
//! later phase). The status and edge maps are TOTAL: an out-of-vocab input takes a
//! per-kind default, never panics or invents a wire string.
//!
//! The corpus records are purpose-built lightweight rows ([`EntityRecord`],
//! [`SpecRecord`], [`SliceRecord`]) rather than the heavyweight authored entity
//! structs: `project` needs only the projected fields (id/title/status/author/
//! date/tags/body + outbound edges), and a thin record keeps the fold and the
//! unit tests free of fixture/IO machinery. The REAL crate types are reused where
//! they carry logic — [`crate::state::PhaseRollup`] (plan-status rule via
//! `total()`/`anomalies()`) and [`crate::relation::RelationEdge`]/`RelationLabel`
//! (the tier-1 outbound edge vocabulary) — never re-derived.

use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::relation::{self, RelationEdge, RelationLabel};
use crate::state::PhaseRollup;

// ---------------------------------------------------------------------------
// Wire structs (the locked JSON shape — serde renames are load-bearing)
// ---------------------------------------------------------------------------

/// The whole lazyspec export: project meta, the projected entity nodes, and the
/// static type manifest. Serialized verbatim to JSON.
#[derive(Debug, Serialize)]
pub(crate) struct Brief {
    pub(crate) meta: BriefMeta,
    pub(crate) entities: Vec<Entity>,
    pub(crate) types: Vec<TypeDef>,
}

/// Export envelope metadata — all three injected/derived from the projection args.
#[derive(Debug, Serialize)]
pub(crate) struct BriefMeta {
    project: String,
    generated_at: String,
    doctrine_version: String,
}

/// One projected node. `date` is YYYY-MM-DD only (never a datetime), `status` is one
/// of the seven wire strings, `related` is outbound-only. `is_virtual`/`rel_type`
/// carry serde renames for the reserved words `virtual`/`type`.
#[derive(Debug, Serialize)]
pub(crate) struct Entity {
    id: String,
    kind: String,
    title: String,
    status: String,
    author: String,
    date: String,
    tags: Vec<String>,
    related: Vec<Relation>,
    body: String,
    #[serde(rename = "virtual")]
    is_virtual: bool,
    validate_ignore: bool,
}

/// One outbound relation. `rel_type` ∈ the four wire strings; `type` is reserved so
/// the field is renamed on the wire.
#[derive(Debug, Serialize)]
pub(crate) struct Relation {
    #[serde(rename = "type")]
    rel_type: String,
    target: String,
}

/// One entry of the static type manifest. `plural`/`dir`/`icon` are cosmetic.
#[derive(Debug, Serialize)]
pub(crate) struct TypeDef {
    name: String,
    plural: String,
    dir: String,
    prefix: String,
    icon: String,
}

// ---------------------------------------------------------------------------
// Corpus (pre-loaded data — no loaders here)
// ---------------------------------------------------------------------------

/// A pre-loaded entity row carrying exactly what one [`Entity`] node needs: its
/// canonical id, lazyspec kind name, the per-kind doctrine `status` source string,
/// the projected fields, and its tier-1 outbound edges. Built by the (later-phase)
/// loaders or, in tests, by hand with zero IO.
#[derive(Debug, Clone)]
pub(crate) struct EntityRecord {
    /// Canonical id (e.g. `SL-026`, `ADR-005`, `ISS-012`).
    pub(crate) id: String,
    /// lazyspec type name (`slice`, `adr`, `issue`, …) — drives the status map arm.
    pub(crate) kind: String,
    /// Display title.
    pub(crate) title: String,
    /// The per-kind doctrine source status string (slice FSM / spec / adr / backlog
    /// vocabulary), mapped to a wire string by [`map_status`]. Out-of-vocab → default.
    pub(crate) status: String,
    /// Author, or `""` where doctrine has none.
    pub(crate) author: String,
    /// Source date, already YYYY-MM-DD.
    pub(crate) date: String,
    pub(crate) tags: Vec<String>,
    /// Assembled inline body.
    pub(crate) body: String,
    /// Tier-1 outbound edges (`Vec<RelationEdge>`).
    pub(crate) edges: Vec<RelationEdge>,
}

/// A spec row — an [`EntityRecord`] plus the typed spec edges that arrive as data
/// (the `descends_from`/`parent` scalar lineage and the `interactions` list) rather
/// than tier-1 `[[relation]]` rows.
#[derive(Debug, Clone)]
pub(crate) struct SpecRecord {
    pub(crate) base: EntityRecord,
    /// `SPEC → PRD` lineage target (typed edge), if any.
    pub(crate) descends_from: Option<String>,
    /// `SPEC → SPEC` decomposition parent (typed edge), if any.
    pub(crate) parent: Option<String>,
    /// `SPEC → SPEC` interaction targets (typed edges).
    pub(crate) interactions: Vec<String>,
}

/// A slice row — an [`EntityRecord`] plus the optional plan: its body and the
/// [`PhaseRollup`] the synthetic `PLAN-NNN` node derives its status from.
#[derive(Debug, Clone)]
pub(crate) struct SliceRecord {
    pub(crate) base: EntityRecord,
    /// `Some((plan_body, rollup))` when the slice has a plan; `None` otherwise.
    pub(crate) plan: Option<(String, PhaseRollup)>,
}

/// The pre-loaded corpus the projection emits. Plain data, hand-constructible in
/// tests with zero IO. Holds the loaded entity rows split by source shape (slices
/// carry an optional plan; specs carry typed edges; everything else is a flat
/// [`EntityRecord`]), plus the `project` name carried for [`BriefMeta`].
#[derive(Debug, Clone, Default)]
pub(crate) struct Corpus {
    /// The export's project name — set by the caller (the loader / the test).
    pub(crate) project: String,
    pub(crate) slices: Vec<SliceRecord>,
    pub(crate) specs: Vec<SpecRecord>,
    /// adr + backlog (issue/improvement/chore/risk/idea) rows — flat records whose
    /// `kind` selects the status map arm.
    pub(crate) others: Vec<EntityRecord>,
}

// ---------------------------------------------------------------------------
// Total status map — output ∈ the seven wire strings
// ---------------------------------------------------------------------------

/// Map a `(kind, doctrine status)` pair to its wire status string. TOTAL: an
/// out-of-vocab input takes the per-kind `draft` default (INV-6). The `kind` is the
/// lazyspec type name on the record; `plan` derives from a [`PhaseRollup`] instead
/// (see [`plan_status`]) and is not routed here.
fn map_status(kind: &str, status: &str) -> &'static str {
    match kind {
        // slice FSM (SL-028, 9 states). The default ("draft") absorbs the three
        // states that map to draft — proposed/design/plan — plus any drift (INV-6),
        // so they are not re-listed (clippy `match_same_arms`).
        "slice" => match status {
            "ready" => "accepted",
            "started" | "audit" | "reconcile" => "in-progress",
            "done" => "complete",
            "abandoned" => "rejected",
            _ => "draft",
        },
        // spec: draft→draft folds into the default.
        "product-spec" | "tech-spec" => match status {
            "active" => "accepted",
            "deprecated" | "superseded" => "superseded",
            _ => "draft",
        },
        "adr" => match status {
            "proposed" => "review",
            "accepted" => "accepted",
            "rejected" => "rejected",
            "superseded" | "deprecated" => "superseded",
            _ => "draft",
        },
        // backlog: open→draft folds into the default.
        "issue" | "improvement" | "chore" | "risk" | "idea" => match status {
            "triaged" => "review",
            "started" => "in-progress",
            "resolved" | "closed" => "complete",
            _ => "draft",
        },
        _ => "draft",
    }
}

/// Derive the synthetic plan node's wire status from its owning slice's
/// [`PhaseRollup`], via the canonical `total()`/`anomalies()` API (never a partial
/// sum). A malformed phase (`missing_toml`/`unknown` > 0 ⇒ `anomalies() > 0`)
/// suppresses `complete`.
fn plan_status(rollup: &PhaseRollup) -> &'static str {
    if rollup.anomalies() == 0 && rollup.total() > 0 && rollup.completed == rollup.total() {
        "complete"
    } else if rollup.completed > 0 {
        "in-progress"
    } else {
        "draft"
    }
}

// ---------------------------------------------------------------------------
// Total edge map — output ∈ the four rel_type wire strings
// ---------------------------------------------------------------------------

/// Map a [`RelationLabel`] to one of the four wire `rel_type` strings, or `None`
/// when the edge is DROPPED (`Requirements` — REQ is inlined, never a node, INV-4).
/// TOTAL with a `related-to` default arm (INV-2). `blocks` has no v1 source.
fn map_edge(label: RelationLabel) -> Option<&'static str> {
    match label {
        RelationLabel::Requirements => None,
        RelationLabel::DescendsFrom | RelationLabel::Parent => Some("implements"),
        RelationLabel::Supersedes => Some("supersedes"),
        _ => Some("related-to"),
    }
}

/// Project a record's tier-1 outbound edges to wire [`Relation`]s, dropping the
/// inlined `requirements` axis (INV-4) and mapping every other label total.
fn project_edges(edges: &[RelationEdge]) -> Vec<Relation> {
    edges
        .iter()
        .filter_map(|e| {
            map_edge(e.label).map(|rel_type| Relation {
                rel_type: rel_type.to_string(),
                target: e.target.clone(),
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// The static type manifest
// ---------------------------------------------------------------------------

/// The full lazyspec type manifest — always emitted in entirety, even for an empty
/// corpus. `plan` is hand-authored (no `Kind` const exists for it). `plural`/`dir`/
/// `icon` are cosmetic. The fixed source list keeps the manifest a single authority.
fn type_manifest() -> Vec<TypeDef> {
    // (name, plural, dir, prefix, virtual is implied by the node, icon)
    let rows: &[(&str, &str, &str, &str, &str)] = &[
        ("slice", "slices", "slices", "SL", "🔪"),
        (
            "product-spec",
            "product-specs",
            "specs/product",
            "PRD",
            "📦",
        ),
        ("tech-spec", "tech-specs", "specs/tech", "SPEC", "⚙"),
        ("adr", "adrs", "adrs", "ADR", "🏛"),
        ("issue", "issues", "issues", "ISS", "🐛"),
        ("improvement", "improvements", "improvements", "IMP", "✨"),
        ("chore", "chores", "chores", "CHR", "🧹"),
        ("risk", "risks", "risks", "RSK", "⚠"),
        ("idea", "ideas", "ideas", "IDE", "💡"),
        ("plan", "plans", "plans", "PLAN", "🗺"),
    ];
    let mut types: Vec<TypeDef> = rows
        .iter()
        .map(|(name, plural, dir, prefix, icon)| TypeDef {
            name: (*name).to_string(),
            plural: (*plural).to_string(),
            dir: (*dir).to_string(),
            prefix: (*prefix).to_string(),
            icon: (*icon).to_string(),
        })
        .collect();
    types.sort_by(|a, b| a.name.cmp(&b.name));
    types
}

// ---------------------------------------------------------------------------
// Per-record node projection
// ---------------------------------------------------------------------------

/// Project a flat [`EntityRecord`] (adr/backlog) to an [`Entity`] node. Status is
/// mapped per-kind (total); edges are projected; `is_virtual` is `false` for every
/// real kind here (only specs are virtual, handled separately).
fn project_record(rec: &EntityRecord, is_virtual: bool) -> Entity {
    Entity {
        id: rec.id.clone(),
        kind: rec.kind.clone(),
        title: rec.title.clone(),
        status: map_status(&rec.kind, &rec.status).to_string(),
        author: rec.author.clone(),
        date: rec.date.clone(),
        tags: rec.tags.clone(),
        related: project_edges(&rec.edges),
        body: rec.body.clone(),
        is_virtual,
        validate_ignore: true,
    }
}

/// Project a [`SpecRecord`] — a virtual node — folding the typed spec edges
/// (`descends_from`/`parent` → `implements`, `interactions` → `related-to`) in
/// AFTER the tier-1 edges, all mapped through the same total edge map.
fn project_spec(spec: &SpecRecord) -> Entity {
    let mut node = project_record(&spec.base, true);
    let typed = spec
        .descends_from
        .iter()
        .map(|t| (RelationLabel::DescendsFrom, t))
        .chain(spec.parent.iter().map(|t| (RelationLabel::Parent, t)))
        .chain(
            spec.interactions
                .iter()
                .map(|t| (RelationLabel::Interactions, t)),
        );
    for (label, target) in typed {
        if let Some(rel_type) = map_edge(label) {
            node.related.push(Relation {
                rel_type: rel_type.to_string(),
                target: target.clone(),
            });
        }
    }
    node
}

/// The synthetic plan id for a slice canonical id `SL-NNN` → `PLAN-NNN` (INV-5),
/// reusing the owning slice's zero-padded number verbatim. Returns `None` for a
/// slice id that does not match `SL-<digits>` (defensive; the loader supplies
/// canonical ids).
fn plan_id_for(slice_id: &str) -> Option<String> {
    slice_id
        .strip_prefix("SL-")
        .map(|nnn| format!("PLAN-{nnn}"))
}

/// Project the synthetic plan node for a slice that has a plan. Status derives from
/// the [`PhaseRollup`]; the single outbound edge is `plan → owning slice`
/// (`implements`); `date` is the owning slice's `date` (already YYYY-MM-DD, INV-7).
fn project_plan(slice: &SliceRecord, plan_body: &str, rollup: &PhaseRollup) -> Option<Entity> {
    let id = plan_id_for(&slice.base.id)?;
    Some(Entity {
        id,
        kind: "plan".to_string(),
        title: format!("Plan for {}", slice.base.id),
        status: plan_status(rollup).to_string(),
        author: String::new(),
        date: slice.base.date.clone(),
        tags: Vec::new(),
        related: vec![Relation {
            rel_type: "implements".to_string(),
            target: slice.base.id.clone(),
        }],
        body: plan_body.to_string(),
        is_virtual: false,
        validate_ignore: true,
    })
}

// ---------------------------------------------------------------------------
// The pure projection
// ---------------------------------------------------------------------------

/// Fold a pre-loaded [`Corpus`] into a [`Brief`] — TOTAL and side-effect-free.
/// `now` (RFC3339, emitted verbatim — never date-parsed) and `version` are injected;
/// `project` rides on the corpus. Emits every slice/spec/adr/backlog node plus one
/// synthetic `PLAN-NNN` per slice with a plan, sorts `entities` by canonical id and
/// `types` by name (idempotent against a shuffled corpus), and never emits a REQ
/// node (INV-4).
pub(crate) fn project(corpus: &Corpus, now: &str, version: &str) -> Brief {
    let mut entities: Vec<Entity> = Vec::new();

    for slice in &corpus.slices {
        entities.push(project_record(&slice.base, false));
        if let Some((plan_body, rollup)) = &slice.plan
            && let Some(plan_node) = project_plan(slice, plan_body, rollup)
        {
            entities.push(plan_node);
        }
    }
    for spec in &corpus.specs {
        entities.push(project_spec(spec));
    }
    for rec in &corpus.others {
        entities.push(project_record(rec, false));
    }

    entities.sort_by(|a, b| a.id.cmp(&b.id));

    Brief {
        meta: BriefMeta {
            project: corpus.project.clone(),
            generated_at: now.to_string(),
            doctrine_version: version.to_string(),
        },
        entities,
        types: type_manifest(),
    }
}

// ---------------------------------------------------------------------------
// The impure shell — `load_corpus` (disk/clock live here, never in `project`)
// ---------------------------------------------------------------------------

/// The one optional scalar pair a numbered entity's `<stem>-NNN.toml` may carry that
/// the projected node needs but the identity [`crate::meta::Meta`] reader drops: the
/// authored `created` date and `tags`. Both `#[serde(default)]` — a spec (no dates,
/// §5.4) or an untagged entity parses cleanly to `None`/`[]`. This is a single
/// scalar read over text ALREADY loaded for the tier-1 edge pass (the same shape
/// `state::TrackingStatus` uses) — not a second entity reader.
#[derive(Debug, Default, Deserialize)]
struct AuthoredHead {
    #[serde(default)]
    created: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

impl AuthoredHead {
    /// Parse the optional `created`/`tags` head from a `<stem>-NNN.toml` body,
    /// tolerating absence (a malformed body would already have failed the edge read).
    fn parse(toml_text: &str) -> Self {
        toml::from_str(toml_text).unwrap_or_default()
    }
}

/// Read a `<stem>-NNN.toml`'s raw text under `tree_root/NNN/`.
fn read_entity_toml(tree_root: &Path, stem: &str, id: u32) -> anyhow::Result<String> {
    let name = format!("{id:03}");
    let path = tree_root.join(&name).join(format!("{stem}-{name}.toml"));
    std::fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))
}

/// Read an entity's prose `.md` body under `tree_root/NNN/<file_stem>-NNN.md`,
/// returning `""` for a missing file. The legitimate raw prose-tier read (OQ-4) —
/// the body is emitted verbatim, never structurally parsed.
fn read_prose_body(tree_root: &Path, file_stem: &str, id: u32) -> anyhow::Result<String> {
    let name = format!("{id:03}");
    let path = tree_root.join(&name).join(format!("{file_stem}-{name}.md"));
    match std::fs::read_to_string(&path) {
        Ok(b) => Ok(b),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(e).with_context(|| format!("Failed to read {}", path.display())),
    }
}

/// Assemble the common [`EntityRecord`] fields shared by every projected node:
/// identity (id/title/status from [`crate::meta::Meta`]), the authored date/tags
/// from the same toml, the verbatim prose body, and the tier-1 outbound edges read
/// through the unified [`relation::tier1_edges`] seam (D7 — never a per-kind edge
/// reach-in). `kind` is the lazyspec type name; `engine_kind` selects the edge
/// vocabulary; `stem`/`file_stem` name the toml/md files.
fn load_entity_record(
    tree_root: &Path,
    engine_kind: &crate::entity::Kind,
    stem: &str,
    file_stem: &str,
    lazyspec_kind: &str,
    id: u32,
) -> anyhow::Result<EntityRecord> {
    let meta = crate::meta::read_meta(tree_root, stem, id)?;
    let toml_text = read_entity_toml(tree_root, stem, id)?;
    let head = AuthoredHead::parse(&toml_text);
    let edges = relation::tier1_edges(engine_kind, &toml_text)?;
    Ok(EntityRecord {
        id: listing_canonical(engine_kind.prefix, id),
        kind: lazyspec_kind.to_string(),
        title: meta.title,
        status: meta.status,
        author: String::new(),
        date: head.created.unwrap_or_default(),
        tags: head.tags,
        body: read_prose_body(tree_root, file_stem, id)?,
        edges,
    })
}

/// The canonical id (`SL-026`) for a prefix + numeric id — the single id-form
/// authority (shared with every prefixed surface).
fn listing_canonical(prefix: &str, id: u32) -> String {
    crate::listing::canonical_id(prefix, id)
}

/// Load every slice into a [`SliceRecord`]: the common record, the verbatim scope
/// `.md` body, plus `Some((plan.md body, PhaseRollup))` for a slice that has a plan
/// (so `project` emits its synthetic `PLAN-NNN` node). The plan body is the raw
/// `plan.md`; the rollup comes from the canonical [`crate::state::phase_rollup`].
fn load_slices(root: &Path) -> anyhow::Result<Vec<SliceRecord>> {
    let tree = root.join(crate::slice::SLICE_KIND.dir);
    let mut out = Vec::new();
    for id in crate::entity::scan_ids(&tree)? {
        let base = load_entity_record(
            &tree,
            &crate::slice::SLICE_KIND,
            "slice",
            "slice",
            "slice",
            id,
        )?;
        let plan = load_plan(root, &tree, id)?;
        out.push(SliceRecord { base, plan });
    }
    Ok(out)
}

/// The `(plan.md body, PhaseRollup)` for a slice with a plan, or `None`. The plan's
/// presence is gated on `plan.toml` (the authored facet); the rollup folds the
/// runtime phase tracking. A slice with no plan, or a plan with no phases tracked
/// yet, both carry a rollup the projection tolerates (`phase_rollup` → `None` for
/// untracked, surfaced as a zeroed rollup so the node still reads as a draft plan).
fn load_plan(root: &Path, tree: &Path, id: u32) -> anyhow::Result<Option<(String, PhaseRollup)>> {
    let name = format!("{id:03}");
    let slice_dir = tree.join(&name);
    if !slice_dir.join("plan.toml").exists() {
        return Ok(None);
    }
    // The plan facet's files are `plan.toml`/`plan.md` (no id suffix), unlike the
    // `<stem>-NNN.md` body convention — read `plan.md` directly.
    let body = match std::fs::read_to_string(slice_dir.join("plan.md")) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => {
            return Err(e).with_context(|| {
                format!("Failed to read {}", slice_dir.join("plan.md").display())
            });
        }
    };
    let rollup = crate::state::phase_rollup(root, id)?.unwrap_or_default();
    Ok(Some((body, rollup)))
}

/// The spec node's `date` (INV-7). A spec carries no authored `created` on disk
/// (§5.4), so the impure shell injects the spec toml's filesystem mtime as an honest
/// last-changed `YYYY-MM-DD`. A `created`, if ever added (IMP-105 / a schema change),
/// still wins. mtime is checkout-unstable across clones — acceptable for a read-only
/// viewer, weak as provenance (the lossy-v1 tradeoff, design §5.4). The mtime read is
/// disk I/O — it lives here in the shell, never in pure [`project`]; the field is
/// always a parseable date, never empty (`clock::today` the can't-happen fallback).
fn spec_date(head: &AuthoredHead, spec_toml: &Path) -> String {
    head.created.clone().unwrap_or_else(|| {
        std::fs::metadata(spec_toml)
            .and_then(|m| m.modified())
            .map_or_else(|_| crate::clock::today(), crate::clock::date_of_system_time)
    })
}

/// Load both spec subtypes into [`SpecRecord`]s. The body is the BOTH-tier
/// `spec::render()` (requirements inline, OQ-4); the typed lineage
/// (`descends_from`/`parent`) and `interactions` ride as DATA on the record (mapped
/// to `implements`/`related-to` by `project`); only the *tier-1* edges land in the
/// base record's `edges` (the typed spec edges are NOT re-read into tier-1 — D7).
fn load_specs(root: &Path) -> anyhow::Result<Vec<SpecRecord>> {
    use crate::spec::SpecSubtype;
    let mut out = Vec::new();
    for (subtype, kind, lazyspec_kind) in [
        (
            SpecSubtype::Product,
            &crate::spec::PRODUCT_SPEC_KIND,
            "product-spec",
        ),
        (SpecSubtype::Tech, &crate::spec::TECH_SPEC_KIND, "tech-spec"),
    ] {
        let tree = root.join(kind.dir);
        for id in crate::entity::scan_ids(&tree)? {
            out.push(load_spec(root, &tree, subtype, kind, lazyspec_kind, id)?);
        }
    }
    Ok(out)
}

/// Assemble one [`SpecRecord`] — reuses the widened `spec::read_spec`/`read_members`/
/// `read_interactions`/`render` (no new spec read logic), resolves each member's
/// requirement via `requirement::load_with_prose`, and carries the typed lineage +
/// interaction targets as data.
fn load_spec(
    root: &Path,
    tree: &Path,
    subtype: crate::spec::SpecSubtype,
    kind: &crate::entity::Kind,
    lazyspec_kind: &str,
    id: u32,
) -> anyhow::Result<SpecRecord> {
    let name = format!("{id:03}");
    let dir = tree.join(&name);
    let (spec, spec_text, prose_body) = crate::spec::read_spec(subtype, root, id)?;

    // Resolve members → (member, requirement) + per-member prose, the `run_show`
    // read sequence, so `render` inlines the requirements (OQ-4).
    let members = crate::spec::read_members(&dir.join("members.toml"))?;
    let mut resolved = Vec::with_capacity(members.len());
    let mut req_bodies: Vec<Option<String>> = Vec::with_capacity(members.len());
    for member in members {
        let (req, prose) = crate::requirement::load_with_prose(root, &member.requirement)?;
        req_bodies.push(prose);
        resolved.push((member, req));
    }
    let interactions = crate::spec::read_interactions(&dir.join("interactions.toml"))?;
    let body = crate::spec::render(&spec, &prose_body, &resolved, &req_bodies, &interactions);

    let head = AuthoredHead::parse(&spec_text);
    let base = EntityRecord {
        id: listing_canonical(kind.prefix, id),
        kind: lazyspec_kind.to_string(),
        title: spec.title.clone(),
        status: spec.status.as_str().to_string(),
        author: String::new(),
        date: spec_date(
            &head,
            &dir.join(format!("{}-{name}.toml", crate::spec::SPEC_STEM)),
        ),
        tags: head.tags,
        body,
        // Tier-1 ONLY (the typed lineage/members/interactions edges are carried
        // separately below) — read through the unified seam over the spec's toml.
        edges: relation::tier1_edges(kind, &spec_text)?,
    };
    Ok(SpecRecord {
        base,
        descends_from: spec.descends_from,
        parent: spec.parent,
        interactions: interactions.into_iter().map(|i| i.target).collect(),
    })
}

/// Load the ADR tree into flat [`EntityRecord`]s.
fn load_adrs(root: &Path) -> anyhow::Result<Vec<EntityRecord>> {
    let kind = &crate::adr::ADR_KIND.kind;
    let tree = root.join(kind.dir);
    let mut out = Vec::new();
    for id in crate::entity::scan_ids(&tree)? {
        out.push(load_entity_record(&tree, kind, "adr", "adr", "adr", id)?);
    }
    Ok(out)
}

/// Load all five backlog kinds into flat [`EntityRecord`]s, reusing
/// `backlog::read_all` for identity/status and the unified tier-1 seam for edges.
fn load_backlog(root: &Path) -> anyhow::Result<Vec<EntityRecord>> {
    let mut out = Vec::new();
    for item in crate::backlog::read_all(root)? {
        let kind = item.kind.kind();
        let tree = root.join(kind.dir);
        let toml_text = read_entity_toml(&tree, "backlog", item.id)?;
        let head = AuthoredHead::parse(&toml_text);
        out.push(EntityRecord {
            id: item.kind.canonical_id(item.id),
            kind: item.kind.as_str().to_string(),
            title: item.title.clone(),
            status: item.status.as_str().to_string(),
            author: String::new(),
            date: head.created.unwrap_or_default(),
            tags: head.tags,
            body: read_prose_body(&tree, "backlog", item.id)?,
            edges: relation::tier1_edges(kind, &toml_text)?,
        });
    }
    Ok(out)
}

/// Load the doctrine corpus at `root` into a [`Corpus`] by COMPOSING the existing
/// per-kind readers + the unified [`relation::tier1_edges`] seam — the impure shell
/// half of `doctrine lazyspec`. Every disk read lives here (and its helpers); the
/// fold over the result ([`project`]) stays pure. The injected `now`/`version` are
/// `project`'s args (filled by the PHASE-04 call site); `load_corpus` fills the
/// corpus's `project` name from the root dir basename. Only the lazyspec kinds are
/// loaded — slice / product+tech spec / adr / the five backlog kinds; requirements
/// inline via spec `render()` and are never standalone nodes (INV-4).
pub(crate) fn load_corpus(root: &Path) -> anyhow::Result<Corpus> {
    let project = root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(Corpus {
        project,
        slices: load_slices(root)?,
        specs: load_specs(root)?,
        others: {
            let mut others = load_adrs(root)?;
            others.extend(load_backlog(root)?);
            others
        },
    })
}

/// The testable seam behind `doctrine export lazyspec`: load the corpus at `root`,
/// fold it through the pure [`project`] with the injected `now`/`version`, and
/// serialize the [`Brief`] to pretty JSON. Read-only — no mutation path. The clap
/// arm reads the clock/version/root at the boundary and prints the returned string,
/// staying a one-liner while this stays unit-testable over a seeded temp tree.
pub(crate) fn run_export_lazyspec(root: &Path, now: &str, version: &str) -> anyhow::Result<String> {
    let corpus = load_corpus(root)?;
    let brief = project(&corpus, now, version);
    Ok(serde_json::to_string_pretty(&brief)?)
}

// ---------------------------------------------------------------------------
// Tests (in-memory Corpus only — no IO, no fixtures, no disk)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(id: &str, kind: &str, status: &str) -> EntityRecord {
        EntityRecord {
            id: id.to_string(),
            kind: kind.to_string(),
            title: format!("title {id}"),
            status: status.to_string(),
            author: String::new(),
            date: "2026-06-19".to_string(),
            tags: Vec::new(),
            body: String::new(),
            edges: Vec::new(),
        }
    }

    fn slice(id: &str, status: &str) -> SliceRecord {
        SliceRecord {
            base: rec(id, "slice", status),
            plan: None,
        }
    }

    fn all_completed(n: u32) -> PhaseRollup {
        PhaseRollup {
            completed: n,
            ..PhaseRollup::default()
        }
    }

    fn brief_of(corpus: &Corpus) -> Brief {
        project(corpus, "2026-06-19T10:00:00Z", "9.9.9")
    }

    // --- VT-1: status + edge totality (default arms) ---

    #[test]
    fn unknown_slice_status_takes_the_draft_default() {
        // INV-6: an out-of-vocab / drifted slice status maps to the per-kind default.
        assert_eq!(map_status("slice", "wat-is-this"), "draft");
        assert_eq!(map_status("slice", ""), "draft");
        // and a node carrying it serializes as draft.
        let mut corpus = Corpus::default();
        corpus.slices.push(slice("SL-001", "totally-not-a-state"));
        let brief = brief_of(&corpus);
        assert_eq!(brief.entities[0].status, "draft");
    }

    #[test]
    fn each_kind_status_default_is_draft() {
        for kind in ["product-spec", "tech-spec", "adr", "issue", "idea", "wat"] {
            assert_eq!(map_status(kind, "off-vocab"), "draft", "{kind}");
        }
    }

    #[test]
    fn exotic_edge_label_takes_the_related_to_default() {
        // INV-2: an out-of-vocab-for-lazyspec label still lands in the 4-string vocab.
        assert_eq!(map_edge(RelationLabel::Drift), Some("related-to"));
        assert_eq!(map_edge(RelationLabel::Consumes), Some("related-to"));
        assert_eq!(map_edge(RelationLabel::GovernedBy), Some("related-to"));
        assert_eq!(map_edge(RelationLabel::Slices), Some("related-to"));
    }

    // --- VT-2: INV-1, INV-4, INV-5, INV-7 ---

    #[test]
    fn every_entity_has_validate_ignore_true_and_iso_date() {
        // INV-1 + INV-7 across every node kind incl. the synthetic plan.
        let mut corpus = Corpus::default();
        corpus.slices.push(SliceRecord {
            base: rec("SL-002", "slice", "started"),
            plan: Some(("plan body".to_string(), all_completed(2))),
        });
        corpus.specs.push(SpecRecord {
            base: rec("SPEC-001", "tech-spec", "active"),
            descends_from: Some("PRD-001".to_string()),
            parent: None,
            interactions: Vec::new(),
        });
        corpus.others.push(rec("ADR-001", "adr", "accepted"));
        corpus.others.push(rec("ISS-001", "issue", "open"));
        let brief = brief_of(&corpus);
        assert!(!brief.entities.is_empty());
        for e in &brief.entities {
            assert!(e.validate_ignore, "{} validate_ignore", e.id);
            assert_eq!(e.date.len(), 10, "{} date YYYY-MM-DD", e.id);
            assert_eq!(e.date.matches('-').count(), 2, "{} date dashes", e.id);
            assert!(!e.date.contains('T'), "{} date not a datetime", e.id);
        }
    }

    #[test]
    fn no_req_node_even_when_a_spec_members_requirements() {
        // INV-4: a tier-1 `requirements` edge is dropped, and no REQ entity exists.
        let mut corpus = Corpus::default();
        let mut base = rec("SL-003", "slice", "started");
        base.edges.push(RelationEdge::new(
            RelationLabel::Requirements,
            "REQ-009".to_string(),
        ));
        base.edges.push(RelationEdge::new(
            RelationLabel::Specs,
            "SPEC-001".to_string(),
        ));
        corpus.slices.push(SliceRecord { base, plan: None });
        let brief = brief_of(&corpus);
        // no REQ entity emitted
        assert!(brief.entities.iter().all(|e| !e.id.starts_with("REQ-")));
        // the requirements edge is dropped; the specs edge survives as related-to
        let sl = &brief.entities[0];
        assert_eq!(sl.related.len(), 1);
        assert_eq!(sl.related[0].rel_type, "related-to");
        assert_eq!(sl.related[0].target, "SPEC-001");
    }

    #[test]
    fn synthetic_plan_id_and_only_plans_are_synthetic() {
        // INV-5: PLAN-NNN shape, owning slice number, and only plan nodes are synthetic.
        let mut corpus = Corpus::default();
        corpus.slices.push(SliceRecord {
            base: rec("SL-026", "slice", "started"),
            plan: Some(("body".to_string(), all_completed(1))),
        });
        let brief = brief_of(&corpus);
        let plan = brief
            .entities
            .iter()
            .find(|e| e.kind == "plan")
            .expect("plan node");
        assert_eq!(plan.id, "PLAN-026");
        // its one edge implements the owning slice
        assert_eq!(plan.related.len(), 1);
        assert_eq!(plan.related[0].rel_type, "implements");
        assert_eq!(plan.related[0].target, "SL-026");
        // the plan date is the owning slice's date (YYYY-MM-DD)
        assert_eq!(plan.date, "2026-06-19");
    }

    // --- VT-3: plan-rollup rule ---

    #[test]
    fn malformed_phase_suppresses_complete() {
        // missing_toml > 0 (all others completed) must NOT map to complete.
        let rollup = PhaseRollup {
            completed: 3,
            missing_toml: 1,
            ..PhaseRollup::default()
        };
        assert_eq!(plan_status(&rollup), "in-progress");
        // a clean all-completed rollup DOES map to complete.
        assert_eq!(plan_status(&all_completed(3)), "complete");
        // an unknown-status anomaly also suppresses complete.
        let with_unknown = PhaseRollup {
            completed: 2,
            unknown: 1,
            ..PhaseRollup::default()
        };
        assert_eq!(plan_status(&with_unknown), "in-progress");
        // zero progress is draft.
        assert_eq!(plan_status(&PhaseRollup::default()), "draft");
    }

    // --- VT-4: ordering / idempotence ---

    #[test]
    fn shuffled_corpus_yields_identically_ordered_output() {
        let build = |order: &[&str]| {
            let mut corpus = Corpus::default();
            corpus.project = "doctrine".to_string();
            for id in order {
                if let Some(nnn) = id.strip_prefix("SL-") {
                    corpus.slices.push(SliceRecord {
                        base: rec(id, "slice", "started"),
                        plan: Some((format!("plan {nnn}"), all_completed(1))),
                    });
                } else {
                    corpus.others.push(rec(id, "adr", "accepted"));
                }
            }
            brief_of(&corpus)
        };
        let a = build(&["SL-003", "ADR-001", "SL-001"]);
        let b = build(&["ADR-001", "SL-001", "SL-003"]);
        let ids_a: Vec<&str> = a.entities.iter().map(|e| e.id.as_str()).collect();
        let ids_b: Vec<&str> = b.entities.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids_a, ids_b);
        // ids are canonically sorted (incl. the synthetic plans)
        assert_eq!(
            ids_a,
            vec!["ADR-001", "PLAN-001", "PLAN-003", "SL-001", "SL-003"]
        );
        // types are sorted by name and identical
        let types_a: Vec<&str> = a.types.iter().map(|t| t.name.as_str()).collect();
        let types_b: Vec<&str> = b.types.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(types_a, types_b);
        let mut sorted = types_a.clone();
        sorted.sort_unstable();
        assert_eq!(types_a, sorted);
    }

    #[test]
    fn type_manifest_is_full_even_for_an_empty_corpus() {
        let brief = brief_of(&Corpus::default());
        assert!(brief.entities.is_empty());
        let names: Vec<&str> = brief.types.iter().map(|t| t.name.as_str()).collect();
        for want in [
            "slice",
            "product-spec",
            "tech-spec",
            "adr",
            "issue",
            "improvement",
            "chore",
            "risk",
            "idea",
            "plan",
        ] {
            assert!(names.contains(&want), "manifest missing {want}");
        }
        assert_eq!(brief.types.len(), 10);
    }

    // --- serde renames ---

    #[test]
    fn serde_renames_emit_virtual_and_type() {
        let entity = Entity {
            id: "SPEC-001".to_string(),
            kind: "tech-spec".to_string(),
            title: "t".to_string(),
            status: "draft".to_string(),
            author: String::new(),
            date: "2026-06-19".to_string(),
            tags: Vec::new(),
            related: vec![Relation {
                rel_type: "implements".to_string(),
                target: "PRD-001".to_string(),
            }],
            body: String::new(),
            is_virtual: true,
            validate_ignore: true,
        };
        let json = serde_json::to_string(&entity).expect("serialize entity");
        assert!(json.contains("\"virtual\":true"), "virtual rename: {json}");
        assert!(!json.contains("is_virtual"), "no raw field name: {json}");
        assert!(
            json.contains("\"type\":\"implements\""),
            "type rename: {json}"
        );
        assert!(!json.contains("rel_type"), "no raw rel field name: {json}");
    }

    #[test]
    fn meta_carries_injected_now_and_version_verbatim() {
        let mut corpus = Corpus::default();
        corpus.project = "doctrine".to_string();
        let brief = project(&corpus, "2026-06-19T10:00:00Z", "1.2.3");
        assert_eq!(brief.meta.project, "doctrine");
        // now is emitted verbatim (NOT date-parsed) — a datetime survives in meta.
        assert_eq!(brief.meta.generated_at, "2026-06-19T10:00:00Z");
        assert_eq!(brief.meta.doctrine_version, "1.2.3");
    }

    // -----------------------------------------------------------------------
    // PHASE-04 — the drift canary
    // -----------------------------------------------------------------------

    /// One exotic, drifted, multi-kind corpus that exercises EVERY emitted wire
    /// field and every default arm in a single fold, so the conformance test can
    /// assert the whole surface (R2) against one [`Brief`]. Built by hand (zero IO):
    /// - a slice with a DRIFTED status (→ `draft` default, INV-6) carrying every
    ///   tier-1 edge label that maps (`Supersedes`→supersedes, an exotic
    ///   `Drift`→related-to default, `Requirements`→dropped, INV-4) plus a plan
    ///   (synthetic `PLAN-NNN`, INV-5/7);
    /// - a tech spec (virtual) whose typed lineage/interaction edges cover
    ///   `implements`/`related-to`;
    /// - an adr and a backlog issue with off-vocab statuses (→ `draft`).
    fn conformance_corpus() -> Corpus {
        let mut corpus = Corpus::default();
        corpus.project = "doctrine".to_string();

        let mut slice_base = rec("SL-026", "slice", "totally-drifted-status");
        slice_base.author = "ada".to_string();
        slice_base.tags = vec!["wire".to_string()];
        slice_base.edges.push(RelationEdge::new(
            RelationLabel::Supersedes,
            "SL-001".to_string(),
        ));
        slice_base.edges.push(RelationEdge::new(
            RelationLabel::Drift,
            "SL-099".to_string(),
        ));
        slice_base.edges.push(RelationEdge::new(
            RelationLabel::Requirements,
            "REQ-005".to_string(),
        ));
        corpus.slices.push(SliceRecord {
            base: slice_base,
            plan: Some(("plan body".to_string(), all_completed(1))),
        });

        corpus.specs.push(SpecRecord {
            base: rec("SPEC-002", "tech-spec", "active"),
            descends_from: Some("PRD-001".to_string()),
            parent: None,
            interactions: vec!["SPEC-003".to_string()],
        });

        corpus.others.push(rec("ADR-001", "adr", "off-vocab"));
        corpus.others.push(rec("ISS-007", "issue", "off-vocab"));
        corpus
    }

    /// The closed wire vocabularies — the canary surfaces a vocabulary drift here.
    const WIRE_STATUSES: [&str; 7] = [
        "draft",
        "review",
        "accepted",
        "in-progress",
        "complete",
        "rejected",
        "superseded",
    ];
    const WIRE_REL_TYPES: [&str; 4] = ["implements", "supersedes", "blocks", "related-to"];

    #[test]
    fn conformance_asserts_invariants_across_kinds_and_fields() {
        let brief = brief_of(&conformance_corpus());

        // INV-1 every node validate_ignore == true.
        for e in &brief.entities {
            assert!(e.validate_ignore, "INV-1 {} validate_ignore", e.id);
        }

        // INV-6 every status ∈ the seven wire strings, and the drifted/off-vocab
        // sources took the `draft` default.
        for e in &brief.entities {
            assert!(
                WIRE_STATUSES.contains(&e.status.as_str()),
                "INV-6 {} status {:?} off-vocab",
                e.id,
                e.status
            );
        }
        assert_eq!(
            node(&brief, "SL-026").status,
            "draft",
            "drifted slice → draft"
        );
        assert_eq!(
            node(&brief, "ADR-001").status,
            "draft",
            "off-vocab adr → draft"
        );
        assert_eq!(
            node(&brief, "ISS-007").status,
            "draft",
            "off-vocab issue → draft"
        );

        // INV-7 every date is YYYY-MM-DD (never a datetime), incl. the synthetic plan.
        for e in &brief.entities {
            assert_eq!(e.date.len(), 10, "INV-7 {} date len", e.id);
            assert_eq!(e.date.matches('-').count(), 2, "INV-7 {} date dashes", e.id);
            assert!(!e.date.contains('T'), "INV-7 {} not a datetime", e.id);
        }

        // INV-2 every EMITTED rel_type ∈ the four wire strings, and the set of
        // emitted types covers `supersedes`, `implements`, `related-to` (incl. the
        // exotic-label → related-to DEFAULT arm). `blocks` has no v1 source.
        let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for e in &brief.entities {
            for r in &e.related {
                assert!(
                    WIRE_REL_TYPES.contains(&r.rel_type.as_str()),
                    "INV-2 {} rel_type {:?} off-vocab",
                    e.id,
                    r.rel_type
                );
                seen.insert(r.rel_type.as_str());
            }
        }
        for want in ["supersedes", "implements", "related-to"] {
            assert!(
                seen.contains(want),
                "INV-2 missing emitted rel_type {want}: {seen:?}"
            );
        }
        // the exotic `Drift` label landed on the default arm.
        let sl = node(&brief, "SL-026");
        assert!(
            related(&brief, "SL-026", "related-to", "SL-099"),
            "exotic label → related-to default: {:?}",
            sl.related
        );
        // the `Requirements` edge was DROPPED (INV-4) — no REQ target survives.
        assert!(
            sl.related.iter().all(|r| !r.target.starts_with("REQ-")),
            "INV-4 requirements edge dropped: {:?}",
            sl.related
        );

        // INV-4 no REQ entity node anywhere.
        assert!(
            brief.entities.iter().all(|e| !e.id.starts_with("REQ-")),
            "INV-4 REQ must inline, never a node"
        );

        // INV-5 synthetic ids are exactly `PLAN-NNN`, and ONLY plan nodes are virtual?
        // no — only plan nodes are SYNTHETIC; spec nodes are virtual-but-real.
        let synthetic: Vec<&str> = brief
            .entities
            .iter()
            .filter(|e| e.kind == "plan")
            .map(|e| e.id.as_str())
            .collect();
        assert_eq!(
            synthetic,
            vec!["PLAN-026"],
            "INV-5 only PLAN-NNN are synthetic"
        );
        // every plan id is PLAN-<the owning slice number>.
        assert_eq!(node(&brief, "PLAN-026").related.len(), 1);
        assert!(
            related(&brief, "PLAN-026", "implements", "SL-026"),
            "INV-5 plan implements its slice"
        );

        // is_virtual: specs are virtual; every other kind is not.
        for e in &brief.entities {
            let want = e.kind == "tech-spec" || e.kind == "product-spec";
            assert_eq!(e.is_virtual, want, "{} is_virtual", e.id);
        }

        // entities id-sorted; types name-sorted.
        let ids: Vec<&str> = brief.entities.iter().map(|e| e.id.as_str()).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted, "entities id-sorted");
        let types: Vec<&str> = brief.types.iter().map(|t| t.name.as_str()).collect();
        let mut sorted_types = types.clone();
        sorted_types.sort_unstable();
        assert_eq!(types, sorted_types, "types name-sorted");

        // the serde keyword renames serialize as `"virtual"` / `"type"`.
        let json = serde_json::to_string(&brief).expect("serialize brief");
        assert!(json.contains("\"virtual\":"), "virtual rename present");
        assert!(!json.contains("\"is_virtual\""), "no raw is_virtual");
        assert!(
            json.contains("\"type\":\"supersedes\""),
            "type rename present"
        );
        assert!(!json.contains("\"rel_type\""), "no raw rel_type");
    }

    /// The exact wire key set of one entity — a stray/renamed/removed field flips
    /// this RED (the field-map / DocMeta-home check: every emitted field is agreed,
    /// none stray).
    #[test]
    fn entity_field_map_is_exactly_the_agreed_keys() {
        // A node carrying one edge so the nested relation key set is asserted too.
        let mut base = rec("ADR-001", "adr", "accepted");
        base.edges.push(RelationEdge::new(
            RelationLabel::Supersedes,
            "ADR-000".to_string(),
        ));
        let entity = project_record(&base, false);
        let value = serde_json::to_value(&entity).expect("serialize entity");
        let obj = value.as_object().expect("entity is a JSON object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        // EXACTLY the agreed node fields — a stray/renamed field flips this RED.
        // `virtual` is the serde-renamed `is_virtual`; no node-level `type` key.
        assert_eq!(
            keys,
            vec![
                "author",
                "body",
                "date",
                "id",
                "kind",
                "related",
                "status",
                "tags",
                "title",
                "validate_ignore",
                "virtual",
            ]
        );
        // the nested relation has EXACTLY `type` (renamed) + `target`.
        let rel = obj
            .get("related")
            .and_then(|r| r.as_array())
            .and_then(|a| a.first())
            .and_then(|r| r.as_object())
            .expect("one relation object");
        let mut rel_keys: Vec<&str> = rel.keys().map(String::as_str).collect();
        rel_keys.sort_unstable();
        assert_eq!(rel_keys, vec!["target", "type"]);

        // BriefMeta's key set.
        let mut corpus = Corpus::default();
        corpus.project = "p".to_string();
        let brief = project(&corpus, "2026-06-19T10:00:00Z", "1.2.3");
        let meta_val = serde_json::to_value(&brief.meta).expect("serialize meta");
        let mut meta_keys: Vec<&str> = meta_val
            .as_object()
            .expect("meta object")
            .keys()
            .map(String::as_str)
            .collect();
        meta_keys.sort_unstable();
        assert_eq!(
            meta_keys,
            vec!["doctrine_version", "generated_at", "project"]
        );

        // TypeDef's key set.
        let td = serde_json::to_value(type_manifest().into_iter().next().expect("a typedef"))
            .expect("serialize typedef");
        let mut td_keys: Vec<&str> = td
            .as_object()
            .expect("typedef object")
            .keys()
            .map(String::as_str)
            .collect();
        td_keys.sort_unstable();
        assert_eq!(td_keys, vec!["dir", "icon", "name", "plural", "prefix"]);
    }

    /// Golden-file canary: a MINIMAL hand-built corpus → an EXPECTED Brief JSON,
    /// value-compared (key order independent). Deterministic via the injected
    /// `now`/`version`. A deliberate change to ANY wire field (rename, removal,
    /// status/edge re-map, ordering) flips this RED — the property the canary owns.
    #[test]
    fn golden_brief_value_matches() {
        // The minimal corpus: one accepted ADR with a single `related-to` edge.
        let mut corpus = Corpus::default();
        corpus.project = "demo".to_string();
        let mut adr = rec("ADR-001", "adr", "accepted");
        adr.title = "First decision".to_string();
        adr.date = "2026-01-02".to_string();
        adr.body = "## Context\nbody\n".to_string();
        adr.edges.push(RelationEdge::new(
            RelationLabel::Drift,
            "ADR-002".to_string(),
        ));
        corpus.others.push(adr);

        let brief = project(&corpus, "2026-06-19T10:00:00Z", "1.2.3");
        let produced = serde_json::to_value(&brief).expect("serialize brief");
        assert_eq!(produced, golden_brief(), "golden Brief drift");
    }

    /// The expected Brief for [`golden_brief_value_matches`], inline (stays under
    /// `src/`). The static `types` manifest is asserted in full so a manifest edit
    /// also trips the canary. Built with `serde_json::json!` so the exact key set,
    /// values, ordering and the keyword renames (`virtual`/`type`) are all pinned.
    fn golden_brief() -> serde_json::Value {
        serde_json::json!({
            "meta": {
                "project": "demo",
                "generated_at": "2026-06-19T10:00:00Z",
                "doctrine_version": "1.2.3"
            },
            "entities": [
                {
                    "id": "ADR-001",
                    "kind": "adr",
                    "title": "First decision",
                    "status": "accepted",
                    "author": "",
                    "date": "2026-01-02",
                    "tags": [],
                    "related": [
                        { "type": "related-to", "target": "ADR-002" }
                    ],
                    "body": "## Context\nbody\n",
                    "virtual": false,
                    "validate_ignore": true
                }
            ],
            "types": [
                { "name": "adr", "plural": "adrs", "dir": "adrs", "prefix": "ADR", "icon": "🏛" },
                { "name": "chore", "plural": "chores", "dir": "chores", "prefix": "CHR", "icon": "🧹" },
                { "name": "idea", "plural": "ideas", "dir": "ideas", "prefix": "IDE", "icon": "💡" },
                { "name": "improvement", "plural": "improvements", "dir": "improvements", "prefix": "IMP", "icon": "✨" },
                { "name": "issue", "plural": "issues", "dir": "issues", "prefix": "ISS", "icon": "🐛" },
                { "name": "plan", "plural": "plans", "dir": "plans", "prefix": "PLAN", "icon": "🗺" },
                { "name": "product-spec", "plural": "product-specs", "dir": "specs/product", "prefix": "PRD", "icon": "📦" },
                { "name": "risk", "plural": "risks", "dir": "risks", "prefix": "RSK", "icon": "⚠" },
                { "name": "slice", "plural": "slices", "dir": "slices", "prefix": "SL", "icon": "🔪" },
                { "name": "tech-spec", "plural": "tech-specs", "dir": "specs/tech", "prefix": "SPEC", "icon": "⚙" }
            ]
        })
    }

    /// A `related(brief, id, rel_type, target)` helper mirroring `loader_tests`'
    /// one — keeps the conformance assertions terse.
    fn node<'a>(brief: &'a Brief, id: &str) -> &'a Entity {
        brief
            .entities
            .iter()
            .find(|e| e.id == id)
            .unwrap_or_else(|| panic!("node {id} present"))
    }

    fn related(brief: &Brief, id: &str, rel_type: &str, target: &str) -> bool {
        node(brief, id)
            .related
            .iter()
            .any(|r| r.rel_type == rel_type && r.target == target)
    }
}

// ---------------------------------------------------------------------------
// Loader tests (the impure shell — over a PHASE-02-seeded on-disk tree)
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod loader_tests {
    use super::*;
    use crate::catalog::test_helpers::{seed_adr, seed_requirement, seed_slice, seed_spec};
    use crate::spec::SpecSubtype;

    /// SL-026 PHASE-03 exit: `load_corpus` over a tree built ENTIRELY by the
    /// PHASE-02 seed helpers yields a `Corpus` whose entities + outbound edges match
    /// the seeded fixtures across slice / spec / adr / backlog / plan. Composition
    /// only — no fixture format of its own, the REAL loaders read the REAL scaffolds.
    #[test]
    fn load_corpus_assembles_every_kind_from_a_seeded_tree() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // A requirement (edge target only — must NEVER be a standalone node, INV-4),
        // membered by both spec subtypes.
        seed_requirement(root, 5);
        // Product spec PRD-001 with one member.
        seed_spec(root, SpecSubtype::Product, 1, &["REQ-005"], &[], &[]);
        // Tech spec SPEC-002: one member, one interaction, and the tech lineage.
        seed_spec(
            root,
            SpecSubtype::Tech,
            2,
            &["REQ-005"],
            &["SPEC-003"],
            &[("descends_from", "PRD-001"), ("parent", "SPEC-004")],
        );
        // ADR-001 with a NON-`supersedes` (`related`) edge.
        seed_adr(root, 1, &[("related", &["ADR-002"])]);
        // A backlog issue.
        crate::backlog::test_support::write_fixture(
            root,
            crate::backlog::test_support::Fixture {
                kind: crate::backlog::ItemKind::Issue,
                id: 7,
                slug: "round-trip",
                title: "Round trip",
                status: "open",
                resolution: "",
                tags: &[],
                facet: None,
                rels: None,
            },
        );
        // A slice WITH a plan + phase tracking, and a plain slice.
        seed_slice(root, 9, &[("specs", &["SPEC-002"])]);
        seed_slice(root, 3, &[]);
        write_plan(root, 9);
        write_phase(root, 9, "phase-01", "completed");

        let corpus = load_corpus(root).unwrap();
        let brief = project(&corpus, "2026-06-19T10:00:00Z", "9.9.9");

        // Project name comes from the root dir basename (data on the Corpus).
        assert_eq!(brief.meta.project, corpus.project);
        assert!(!corpus.project.is_empty());

        // --- entity set: every seeded kind present, REQ never a node (INV-4) ---
        let ids: Vec<&str> = brief.entities.iter().map(|e| e.id.as_str()).collect();
        for want in [
            "SL-003", "SL-009", "PRD-001", "SPEC-002", "ADR-001", "ISS-007",
        ] {
            assert!(ids.contains(&want), "missing {want}: {ids:?}");
        }
        assert!(
            brief.entities.iter().all(|e| !e.id.starts_with("REQ-")),
            "requirement must inline, never a node: {ids:?}"
        );

        // --- spec: members + interactions made it into the rendered body / edges ---
        let spec = node(&brief, "SPEC-002");
        assert_eq!(spec.kind, "tech-spec");
        assert!(spec.is_virtual, "specs are virtual nodes");
        // render() inlines the membered requirement (REQ-005) into the body.
        assert!(
            spec.body.contains("REQ-005"),
            "spec body must inline its member: {}",
            spec.body
        );
        // typed lineage → implements; interaction → related-to.
        assert!(
            related(spec, "implements", "PRD-001"),
            "descends_from → implements: {:?}",
            spec.related
        );
        assert!(
            related(spec, "implements", "SPEC-004"),
            "parent → implements: {:?}",
            spec.related
        );
        assert!(
            related(spec, "related-to", "SPEC-003"),
            "interaction → related-to: {:?}",
            spec.related
        );

        // --- adr: the non-supersedes (`related`) edge is present (→ related-to) ---
        let adr = node(&brief, "ADR-001");
        assert_eq!(adr.kind, "adr");
        assert!(
            related(adr, "related-to", "ADR-002"),
            "ADR related edge: {:?}",
            adr.related
        );

        // --- backlog: identity + status mapped ---
        let iss = node(&brief, "ISS-007");
        assert_eq!(iss.kind, "issue");
        assert_eq!(iss.title, "Round trip");

        // --- slice with a plan: carries (plan_body, rollup) → synthetic PLAN node ---
        let plan = node(&brief, "PLAN-009");
        assert_eq!(plan.kind, "plan");
        // all phases completed → the rollup maps the plan to complete.
        assert_eq!(plan.status, "complete");
        assert!(plan.body.contains("plan body"), "plan body: {}", plan.body);
        assert!(
            related(plan, "implements", "SL-009"),
            "plan implements its slice: {:?}",
            plan.related
        );
        // the plain slice (SL-003) has NO plan node.
        assert!(!ids.contains(&"PLAN-003"), "SL-003 has no plan: {ids:?}");

        // the slice's own tier-1 `specs` edge survives as related-to.
        let sl9 = node(&brief, "SL-009");
        assert!(
            related(sl9, "related-to", "SPEC-002"),
            "slice specs edge: {:?}",
            sl9.related
        );
    }

    /// INV-7 regression (R2 surface-parity): a real-shaped spec carries NO authored
    /// `created` (§5.4), yet its loaded node date must be a parseable `YYYY-MM-DD` —
    /// lazyspec's `DocMeta.date` is mandatory `%Y-%m-%d`, so empty is a hard break.
    /// The in-memory fixtures all set a date, hiding this against the real dateless
    /// corpus; this loads the REAL scaffold so the suite catches the class (the impure
    /// shell injects the spec toml mtime, never an empty string).
    #[test]
    fn dateless_spec_loads_a_parseable_date_from_toml_mtime() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_requirement(root, 5);
        seed_spec(root, SpecSubtype::Product, 1, &["REQ-005"], &[], &[]);

        let corpus = load_corpus(root).unwrap();
        let brief = project(&corpus, "2026-06-19T10:00:00Z", "9.9.9");
        let spec = node(&brief, "PRD-001");

        // The seeded spec toml has no `created` — the date must still be a real ISO
        // date, never the empty string that broke INV-7 against the live corpus.
        assert!(!spec.date.is_empty(), "INV-7 spec date not empty");
        assert_eq!(spec.date.len(), 10, "INV-7 date len: {:?}", spec.date);
        assert_eq!(
            spec.date.matches('-').count(),
            2,
            "INV-7 dashes: {:?}",
            spec.date
        );
        assert!(
            !spec.date.contains('T'),
            "INV-7 not a datetime: {:?}",
            spec.date
        );
    }

    /// PHASE-04 — the `doctrine export lazyspec` command path: drive the testable
    /// seam (`run_export_lazyspec`, the one-liner the clap arm calls) over a seeded
    /// temp tree and assert it returns parseable, conformant Brief JSON (Ok / exit
    /// 0). Deterministic via the injected `now`/`version`.
    #[test]
    fn export_lazyspec_command_emits_parseable_brief_json() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_adr(root, 1, &[("related", &["ADR-002"])]);
        seed_slice(root, 9, &[]);

        let json = run_export_lazyspec(root, "2026-06-19T10:00:00Z", "9.9.9")
            .expect("export lazyspec Ok (exit 0)");

        let value: serde_json::Value = serde_json::from_str(&json).expect("parseable Brief JSON");
        // envelope meta carries the injected now/version verbatim.
        assert_eq!(value["meta"]["generated_at"], "2026-06-19T10:00:00Z");
        assert_eq!(value["meta"]["doctrine_version"], "9.9.9");
        // the seeded entities are present, with the wire `virtual`/`validate_ignore`
        // keys (a proof the real wire shape — not a raw struct — reached stdout).
        let ids: Vec<&str> = value["entities"]
            .as_array()
            .expect("entities array")
            .iter()
            .map(|e| e["id"].as_str().expect("id string"))
            .collect();
        assert!(ids.contains(&"ADR-001"), "ADR-001 present: {ids:?}");
        assert!(ids.contains(&"SL-009"), "SL-009 present: {ids:?}");
        let adr = &value["entities"][0];
        assert_eq!(adr["validate_ignore"], true);
        assert!(adr.get("virtual").is_some(), "wire `virtual` key present");
    }

    /// Write an authored `plan.toml` (one phase) + `plan.md` body under a slice.
    fn write_plan(root: &std::path::Path, slice_id: u32) {
        let name = format!("{slice_id:03}");
        let dir = root.join(".doctrine/slice").join(&name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("plan.toml"),
            "schema = \"doctrine.plan\"\nversion = 1\n\
             [[phase]]\nid = \"PHASE-01\"\nname = \"p\"\nobjective = \"o\"\n",
        )
        .unwrap();
        std::fs::write(dir.join("plan.md"), "plan body\n").unwrap();
    }

    /// Write a runtime phase-tracking toml under the gitignored state tree.
    fn write_phase(root: &std::path::Path, slice_id: u32, stem: &str, status: &str) {
        let name = format!("{slice_id:03}");
        let dir = root
            .join(".doctrine/state/slice")
            .join(&name)
            .join("phases");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{stem}.toml")),
            format!("status = \"{status}\"\n"),
        )
        .unwrap();
    }

    fn node<'a>(brief: &'a Brief, id: &str) -> &'a Entity {
        brief
            .entities
            .iter()
            .find(|e| e.id == id)
            .unwrap_or_else(|| panic!("node {id} present"))
    }

    fn related(entity: &Entity, rel_type: &str, target: &str) -> bool {
        entity
            .related
            .iter()
            .any(|r| r.rel_type == rel_type && r.target == target)
    }
}
