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
