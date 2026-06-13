// SPDX-License-Identifier: GPL-3.0-only
//! The relation vocabulary leaf (design §5.2/§5.3 extraction seam).
//!
//! A pure data leaf BELOW every kind module and `relation_graph` (ADR-001): it
//! imports nothing doctrine-internal, so the per-kind `relation_edges` accessors
//! and the engine dispatch can both depend on it without a cycle. It is the SEED of
//! ADR-010's code-authoritative relation vocabulary — SL-048 *extends* this enum
//! and the legal-set table, never forks a parallel one (ADR-010 Decision 2; SL-046
//! design §7 D4).
//!
//! [`RelationLabel`] is the full outbound vocabulary every accessor can emit. Most
//! labels back a graph overlay (design §5.3 overlay table — the resolvable subset);
//! two — [`RelationLabel::Drift`] and [`RelationLabel::DecisionRef`] — are
//! **target-unvalidated** (ADR-010 Decision 2): their targets are free-text with no
//! `DRIFT`/`DEC` kind in `integrity::KINDS`, so they never resolve to a node and
//! surface as danglers, never edges (§5.3). They are vocabulary labels with no
//! overlay, carried so the data is preserved (visibility), not dropped.
//!
//! As of SL-046 PHASE-04 the full vocabulary is LIVE: `relation_graph`'s scan reads
//! `.label`/`.target`, and the `inspect` render reads `name()` — so the PHASE-02
//! `not(test)` `dead_code` expect retired itself, as designed.
//!
//! SL-048 PHASE-02 re-arms that pattern for a NEW leaf: the legal-set table
//! ([`RELATION_RULES`]) and its supporting types ([`TargetSpec`]/[`Tier`]/
//! [`LinkPolicy`]/[`RelationRule`]) plus the two new vocabulary variants
//! (`GovernedBy`/`Consumes`) are built AHEAD of their consumers — the `read_block`
//! parser, the `link`/`unlink` writer, forward validation, and the cordage overlay
//! allocation all land at PHASE-03/04. Until then they are exercised only by this
//! module's `#[cfg(test)]` suite, so they read as dead in the bins/lib build. The
//! module-level `not(test)` `dead_code` expect below self-clears the moment those
//! consumers land; scoping it `not(test)` keeps it fulfilled in the test build,
//! where the symbols ARE used (the cfg(test) round-trip caveat).
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-048 PHASE-02 — RELATION_RULES table + GovernedBy/Consumes built ahead of their PHASE-03/04 consumers; self-clears when wired"
    )
)]

/// The outbound relation vocabulary — one label per authored relation axis across
/// the six edge-authoring kinds. `Copy + Ord` so callers can group/sort labels
/// deterministically (no `HashMap` iteration order — REQ-077).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RelationLabel {
    /// slice → spec, backlog → spec.
    Specs,
    /// slice → requirement.
    Requirements,
    /// slice → slice, governance → governance.
    Supersedes,
    /// spec → product spec (single-valued lineage).
    DescendsFrom,
    /// spec → spec (single decomposition parent).
    Parent,
    /// spec → requirement (`members.toml`).
    Members,
    /// spec → spec (`interactions.toml`; the per-edge free-text `type` is re-read
    /// from the source at render — a single relation class, design §5.3 / C2).
    Interactions,
    /// slice·PRD·SPEC → governance (ADR/POL/STD). One shared label spanning all
    /// three sources, as `supersedes` already spans SL+gov; inbound renders
    /// "governs" via [`RelationRule::inbound_name`] (SL-048 design §5.2 / X5).
    /// Constructed only by the table/tests until PHASE-04 threads the live axes
    /// (covered by the module-level `not(test)` `dead_code` expect, below).
    GovernedBy,
    /// product spec → product spec (consumer → provider, directional): "PRD-011
    /// consumes a seam PRD-009 exposes"; inbound renders `consumed_by` (SL-048
    /// design §5.2 OD-1 / X4). Distinct from the work-item `depends_on` axis.
    /// Constructed only by the table/tests until PHASE-04 threads the live axes.
    Consumes,
    /// backlog → slice.
    Slices,
    /// governance → governance (symmetric).
    Related,
    /// review → any (the `[target].ref` subject).
    Reviews,
    /// rec → slice.
    OwningSlice,
    /// backlog → free-text (no `DRIFT` kind; target-unvalidated, always dangles —
    /// ADR-010 Decision 2 / §5.3). No overlay.
    Drift,
    /// rec → free-text DEC ref (no `DEC` kind; target-unvalidated, always dangles —
    /// ADR-010 Decision 2 / §5.3). No overlay.
    DecisionRef,
}

impl RelationLabel {
    /// The stable wire/render name of a label — re-used by the overlay-identity map
    /// and the inspect render (PHASE-03/04). Single source for the label string.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            RelationLabel::Specs => "specs",
            RelationLabel::Requirements => "requirements",
            RelationLabel::Supersedes => "supersedes",
            RelationLabel::DescendsFrom => "descends_from",
            RelationLabel::Parent => "parent",
            RelationLabel::Members => "members",
            RelationLabel::Interactions => "interactions",
            RelationLabel::GovernedBy => "governed_by",
            RelationLabel::Consumes => "consumes",
            RelationLabel::Slices => "slices",
            RelationLabel::Related => "related",
            RelationLabel::Reviews => "reviews",
            RelationLabel::OwningSlice => "owning_slice",
            RelationLabel::Drift => "drift",
            RelationLabel::DecisionRef => "decision_ref",
        }
    }

    /// Parse an authored `[[relation]]` `label = "…"` spelling back to its variant —
    /// the inverse of [`name`](Self::name). `None` for any string that names no
    /// vocabulary label (e.g. a typo or an INVERSE spelling like `superseded_by`,
    /// which is derived render-text, never an authorable outbound label — ADR-010 D5).
    /// Drives [`read_block`]'s "label not in the table at all" `IllegalRow` arm (X2);
    /// the exhaustive `match` over `name()` keeps it in lock-step with the enum (a new
    /// variant fails to compile until it is added here).
    pub(crate) fn from_name(name: &str) -> Option<RelationLabel> {
        let label = match name {
            "specs" => RelationLabel::Specs,
            "requirements" => RelationLabel::Requirements,
            "supersedes" => RelationLabel::Supersedes,
            "descends_from" => RelationLabel::DescendsFrom,
            "parent" => RelationLabel::Parent,
            "members" => RelationLabel::Members,
            "interactions" => RelationLabel::Interactions,
            "governed_by" => RelationLabel::GovernedBy,
            "consumes" => RelationLabel::Consumes,
            "slices" => RelationLabel::Slices,
            "related" => RelationLabel::Related,
            "reviews" => RelationLabel::Reviews,
            "owning_slice" => RelationLabel::OwningSlice,
            "drift" => RelationLabel::Drift,
            "decision_ref" => RelationLabel::DecisionRef,
            _ => return None,
        };
        // Defence-in-depth: the spelling must round-trip, so `name()` stays the single
        // source of every label string (a future drift between the two maps trips this
        // in the test build, where `from_name` is exercised).
        debug_assert_eq!(label.name(), name);
        Some(label)
    }
}

use crate::entity::Kind;

/// What an outbound label's target ref is allowed to resolve to — the forward-edge
/// validation axis (design §5.2, the first of the five axes). No `Debug`: it holds
/// `&Kind` refs and `entity::Kind` is data without a `Debug` impl (compared by
/// `prefix`); diagnostics format the `RelationLabel`, never the `TargetSpec`.
#[derive(Clone, Copy)]
pub(crate) enum TargetSpec {
    /// The target must be one of an explicit set of numbered kinds (e.g.
    /// `governed_by` → ADR·POL·STD).
    Kinds(&'static [&'static Kind]),
    /// The target kind must equal the source kind — governance `supersedes` and
    /// `related` (each gov kind → its own kind). One rule serves a source-set whose
    /// members each point within their own namespace (R2-M1).
    SameKind,
    /// The target may be any numbered kind — RV `reviews` (the subject of a review
    /// is any entity).
    AnyNumbered,
    /// The target is free-text with no kind in `integrity::KINDS` (`drift`,
    /// `decision_ref`): never resolves, always dangles, no overlay (ADR-010 D2).
    Unvalidated,
}

/// The storage shape of a label's edges (design §5.2, the second axis). `One` →
/// uniform `[[relation]]` rows; `Typed` → a bespoke per-kind structure
/// (`members.toml`, the `descends_from` scalar, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tier {
    /// Tier-1: the uniform `[[relation]]` block.
    One,
    /// Tier-2/3: a bespoke typed structure, not migrated to `[[relation]]`.
    Typed,
}

/// Whether (and how) the `link`/`unlink` verb admits a triple for a label (design
/// §5.2, the third axis).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LinkPolicy {
    /// `link`/`unlink` may author this edge.
    Writable,
    /// Tier-1 by shape but storage-excluded from the migration and never
    /// `link`-writable — governance `supersedes`, whose `superseded_by` carve-out
    /// pair waits on the transactional supersede verb (OD-3 / IMP-006).
    LifecycleOnly,
    /// Authored only through a bespoke typed verb (`spec req add`, `review …`),
    /// never generic `link`.
    TypedVerbOnly,
}

/// One row of the legal-set vocabulary table (design §5.2, the ADR-010 D2 spine):
/// the `(source ∈ sources, label)` key plus the five axes it drives —
/// `target` (forward validation), `tier` (storage shape), `link` (verb admission),
/// and `inbound_name` (derived-reciprocal render text, X5). `sources` is a SET so
/// one rule serves multiple source kinds (F2 — never one row per kind). No `Debug`:
/// it holds `&Kind` refs (no `Debug` impl); diagnostics format `label` only.
#[derive(Clone, Copy)]
pub(crate) struct RelationRule {
    /// The source kinds that may author this label (a set, not one row per kind).
    pub(crate) sources: &'static [&'static Kind],
    /// The outbound label this rule governs.
    pub(crate) label: RelationLabel,
    /// How the derived reciprocal renders on the target (`governed_by` → "governs").
    /// `== label.name()` for every label whose inbound spelling equals its outbound;
    /// only `supersedes`/`governed_by`/`consumes` differ (R2-M3, render-text only).
    pub(crate) inbound_name: &'static str,
    /// What the target ref may resolve to (forward validation).
    pub(crate) target: TargetSpec,
    /// The storage shape of this label's edges.
    pub(crate) tier: Tier,
    /// Whether the generic `link`/`unlink` verb admits this label.
    pub(crate) link: LinkPolicy,
}

// Local kind aliases — leaf-layer references into the per-kind `Kind` descriptors
// (the same statics `integrity::KINDS` indexes; compared by `prefix`, the canonical
// identity, since `Kind` is data without `PartialEq`). No cycle (ADR-001): these are
// `&'static` data refs, not a dependency on those modules' logic.
const SLICE: &Kind = &crate::slice::SLICE_KIND;
const PRD: &Kind = &crate::spec::PRODUCT_SPEC_KIND;
const SPEC: &Kind = &crate::spec::TECH_SPEC_KIND;
const REQ: &Kind = &crate::requirement::REQUIREMENT_KIND;
const ADR: &Kind = &crate::adr::ADR_KIND.kind;
const POL: &Kind = &crate::policy::POLICY_KIND.kind;
const STD: &Kind = &crate::standard::STANDARD_KIND.kind;
const RV: &Kind = &crate::review::REVIEW_KIND;
const REC: &Kind = &crate::rec::REC_KIND;
const ISS: &Kind = &crate::backlog::ISSUE_KIND;
const IMP: &Kind = &crate::backlog::IMPROVEMENT_KIND;
const CHR: &Kind = &crate::backlog::CHORE_KIND;
const RSK: &Kind = &crate::backlog::RISK_KIND;
const IDE: &Kind = &crate::backlog::IDEA_KIND;

/// Every governance kind — the source-set for `supersedes`(gov)/`related`, and the
/// `governed_by` target-set.
const GOV: &[&Kind] = &[ADR, POL, STD];
/// Every backlog item kind — they share one `relation_edges` accessor, so they
/// share `specs`/`slices`/`drift` (the backlog source-set).
const BACKLOG: &[&Kind] = &[ISS, IMP, CHR, RSK, IDE];

/// The legal-set vocabulary table (design §5.2 / ADR-010 D2). **Declared in
/// `RelationLabel` enum-discriminant order** (R2-C1 / §5.3 X1): VT-1 pins the
/// derived distinct-label order against the enum's `Ord`, so `inspect`'s
/// `BTreeMap<RelationLabel>` regroup stays canonical and new variants land at their
/// source kind's axis-run tail where no existing golden pins a successor. The two
/// `supersedes` rows (SL and gov) sit adjacently at the `Supersedes` slot. Lookup is
/// keyed by `(source ∈ sources, label)` — see [`lookup`]. NOT yet wired into any
/// reader/writer/overlay; PHASE-03/04 consume it.
pub(crate) const RELATION_RULES: &[RelationRule] = &[
    RelationRule {
        sources: &[SLICE, ISS, IMP, CHR, RSK, IDE],
        label: RelationLabel::Specs,
        inbound_name: "specs",
        target: TargetSpec::Kinds(&[PRD, SPEC]),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: &[SLICE],
        label: RelationLabel::Requirements,
        inbound_name: "requirements",
        target: TargetSpec::Kinds(&[REQ]),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    // supersedes — two rules at one slot: SL→SL (writable) and gov→same-gov
    // (lifecycle-only, storage-excluded OD-3).
    RelationRule {
        sources: &[SLICE],
        label: RelationLabel::Supersedes,
        inbound_name: "superseded by",
        target: TargetSpec::Kinds(&[SLICE]),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: GOV,
        label: RelationLabel::Supersedes,
        inbound_name: "superseded by",
        target: TargetSpec::SameKind,
        tier: Tier::One,
        link: LinkPolicy::LifecycleOnly,
    },
    RelationRule {
        sources: &[SPEC],
        label: RelationLabel::DescendsFrom,
        inbound_name: "descends_from",
        target: TargetSpec::Kinds(&[PRD]),
        tier: Tier::Typed,
        link: LinkPolicy::TypedVerbOnly,
    },
    RelationRule {
        sources: &[SPEC],
        label: RelationLabel::Parent,
        inbound_name: "parent",
        target: TargetSpec::Kinds(&[SPEC]),
        tier: Tier::Typed,
        link: LinkPolicy::TypedVerbOnly,
    },
    RelationRule {
        sources: &[PRD, SPEC],
        label: RelationLabel::Members,
        inbound_name: "members",
        target: TargetSpec::Kinds(&[REQ]),
        tier: Tier::Typed,
        link: LinkPolicy::TypedVerbOnly,
    },
    RelationRule {
        sources: &[SPEC],
        label: RelationLabel::Interactions,
        inbound_name: "interactions",
        target: TargetSpec::Kinds(&[SPEC]),
        tier: Tier::Typed,
        link: LinkPolicy::TypedVerbOnly,
    },
    RelationRule {
        sources: &[SLICE, PRD, SPEC],
        label: RelationLabel::GovernedBy,
        inbound_name: "governs",
        target: TargetSpec::Kinds(GOV),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: &[PRD],
        label: RelationLabel::Consumes,
        inbound_name: "consumed_by",
        target: TargetSpec::Kinds(&[PRD]),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: BACKLOG,
        label: RelationLabel::Slices,
        inbound_name: "slices",
        target: TargetSpec::Kinds(&[SLICE]),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: GOV,
        label: RelationLabel::Related,
        inbound_name: "related",
        target: TargetSpec::SameKind,
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: &[RV],
        label: RelationLabel::Reviews,
        inbound_name: "reviews",
        target: TargetSpec::AnyNumbered,
        tier: Tier::Typed,
        link: LinkPolicy::TypedVerbOnly,
    },
    RelationRule {
        sources: &[REC],
        label: RelationLabel::OwningSlice,
        inbound_name: "owning_slice",
        target: TargetSpec::Kinds(&[SLICE]),
        tier: Tier::Typed,
        link: LinkPolicy::TypedVerbOnly,
    },
    RelationRule {
        sources: BACKLOG,
        label: RelationLabel::Drift,
        inbound_name: "drift",
        target: TargetSpec::Unvalidated,
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: &[REC],
        label: RelationLabel::DecisionRef,
        inbound_name: "decision_ref",
        target: TargetSpec::Unvalidated,
        tier: Tier::Typed,
        link: LinkPolicy::TypedVerbOnly,
    },
];

/// The rule governing `(source, label)` — the table's lookup key (design §5.2). A
/// source matches when its `prefix` (the canonical `Kind` identity — `Kind` is data
/// without `PartialEq`, compared by prefix everywhere) is in the rule's `sources`.
/// `None` ⇒ illegal for that source (the X2 per-kind legality `read_block`
/// enforces). NOT yet wired into a live reader (PHASE-03).
pub(crate) fn lookup(source: &Kind, label: RelationLabel) -> Option<&'static RelationRule> {
    RELATION_RULES
        .iter()
        .find(|r| r.label == label && r.sources.iter().any(|k| k.prefix == source.prefix))
}

/// One authored outbound relation: its [`RelationLabel`] and the canonical ref
/// string it points at. `target` is the authored ref verbatim and MAY be free-text
/// or dangling — resolution (and dangler classification) happens later, at the graph
/// scan (PHASE-03); the accessor never resolves (design §5.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RelationEdge {
    pub(crate) label: RelationLabel,
    pub(crate) target: String,
}

impl RelationEdge {
    /// Construct an edge from a label and an owned target ref.
    pub(crate) fn new(label: RelationLabel, target: String) -> Self {
        Self { label, target }
    }
}

/// Why a `[[relation]]` row was rejected by [`read_block`] (X2) — the validation
/// finding's reason. A finding is NEVER a live edge; PHASE-05's `validate` reports
/// these (the only consumer), so until then they read as dead in the bins/lib build
/// (covered by the module-level `not(test)` `dead_code` expect).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IllegalReason {
    /// The `label` string names no vocabulary label at all (a typo, or an inverse
    /// spelling like `superseded_by` — derived render text, never authorable).
    UnknownLabel,
    /// The label is a real vocabulary label, but this `source_kind` may not author it
    /// (e.g. a slice carrying `related`, a backlog item carrying `governed_by`). The
    /// per-kind legality the hardcoded readers enforced for free.
    IllegalForSource,
}

/// One `[[relation]]` row [`read_block`] refused (X2): the offending label spelling
/// **verbatim** (so a typo is reported as authored, even when it maps to no variant),
/// the target ref, and the [`IllegalReason`]. A validation finding, not a live edge —
/// see [`read_block`]. Carries the authored spelling rather than a `RelationLabel`
/// because the `UnknownLabel` case has no variant to name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IllegalRow {
    /// The authored `label` string, verbatim (may name no vocabulary label).
    pub(crate) label: String,
    /// The authored `target` ref, verbatim.
    pub(crate) target: String,
    /// Why the row was rejected.
    pub(crate) reason: IllegalReason,
}

/// One generic tier-1 `[[relation]]` row as authored on disk — `label = "…", target =
/// "…"`. The uniform storage shape the PHASE-04 migration writes and `read_block`
/// parses. A serde row struct mirrors the established kind-module `toml::from_str`
/// idiom (`slice::Relationships`, `spec` members/interactions) — no parallel parser.
#[derive(Debug, Clone, serde::Deserialize)]
struct RelationRow {
    label: String,
    target: String,
}

/// The document shape `read_block` deserializes: the array of generic `[[relation]]`
/// rows. `#[serde(default)]` so a file with no `[[relation]]` block (or a hand-trimmed
/// one) parses to an empty block, matching the read-tolerant convention every kind
/// module's relationship reader follows.
#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct RelationDoc {
    #[serde(default)]
    relation: Vec<RelationRow>,
}

impl RelationDoc {
    /// Parse the `[[relation]]` block out of a kind's authored TOML text. Rides the
    /// `toml::from_str` idiom the show-path readers use; `#[serde(default)]` ignores
    /// every other key (the kind's own metadata/typed tables), so one parse over the
    /// whole file yields just the generic rows.
    pub(crate) fn parse(text: &str) -> anyhow::Result<RelationDoc> {
        toml::from_str(text).map_err(|e| anyhow::anyhow!("parse [[relation]] block: {e}"))
    }
}

/// Parse the generic `[[relation]]` rows of one entity and split them into the legal
/// outbound edges and the illegal validation findings (design §5.3, X2).
///
/// Generic storage must NOT mean a generic parser that emits anything: a slice cannot
/// author `related`, a backlog item cannot author `governed_by`. That per-kind
/// legality lived in the hardcoded readers' code shape; `read_block` reproduces it by
/// checking each row's `(source_kind, label)` against [`RELATION_RULES`] via
/// [`lookup`]:
/// - **legal** (`label` resolves to a variant AND `source_kind ∈ rule.sources`) ⇒ a
///   [`RelationEdge`].
/// - **illegal** (`label` names no variant, OR the variant is not authorable by
///   `source_kind`) ⇒ an [`IllegalRow`] finding — NEVER a live edge.
///
/// Legal edges are emitted in **`RELATION_RULES` declaration order** for the source
/// kind (X1 canonical); within one label, authored row order is preserved. This pins
/// the per-kind tier-1 sequence the accessor return value / JSON / `format_show` paths
/// consume before any `BTreeMap` regroup. `IllegalRow`s follow authored row order.
///
/// NOT yet wired into a live reader (PHASE-04); PHASE-05's `validate` consumes the
/// findings. Until then both this fn and [`IllegalRow`] read as dead in the bins/lib
/// build (the module-level `not(test)` `dead_code` expect), exercised by the suite.
pub(crate) fn read_block(
    source_kind: &Kind,
    doc: &RelationDoc,
) -> (Vec<RelationEdge>, Vec<IllegalRow>) {
    let mut illegal: Vec<IllegalRow> = Vec::new();
    // Bucket the legal edges by their canonical label position in RELATION_RULES, so
    // the emitted order is the table's declaration order for this source (X1) while
    // same-label rows keep their authored sequence. `Vec<(pos, edge)>` then a stable
    // sort by pos — stable so within a label the authored order survives.
    let mut legal: Vec<(usize, RelationEdge)> = Vec::new();
    for row in &doc.relation {
        match RelationLabel::from_name(&row.label) {
            None => illegal.push(IllegalRow {
                label: row.label.clone(),
                target: row.target.clone(),
                reason: IllegalReason::UnknownLabel,
            }),
            Some(label) => match canonical_position(source_kind, label) {
                Some(pos) => {
                    legal.push((pos, RelationEdge::new(label, row.target.clone())));
                }
                None => illegal.push(IllegalRow {
                    label: row.label.clone(),
                    target: row.target.clone(),
                    reason: IllegalReason::IllegalForSource,
                }),
            },
        }
    }
    // Stable sort by canonical position: same-label rows keep authored order (X1).
    legal.sort_by_key(|(pos, _)| *pos);
    let edges = legal.into_iter().map(|(_, e)| e).collect();
    (edges, illegal)
}

/// The index of the FIRST `RELATION_RULES` row that legalises `(source, label)`, or
/// `None` if the pair is illegal. The index is the canonical-order key `read_block`
/// sorts by (X1) — and because the table is declared in `RelationLabel` enum-`Ord`
/// order (VT-1), distinct labels sort into the same order `inspect`'s `BTreeMap`
/// regroup produces, keeping every render surface canonical.
fn canonical_position(source: &Kind, label: RelationLabel) -> Option<usize> {
    RELATION_RULES
        .iter()
        .position(|r| r.label == label && r.sources.iter().any(|k| k.prefix == source.prefix))
}

/// The live-reader convenience seam (PHASE-04): parse the `[[relation]]` block out of
/// one entity's authored TOML `text` and return only the **legal** tier-1 edges, in
/// canonical [`RELATION_RULES`] order (X1). The illegal findings are dropped here —
/// the show / `relation_edges` paths surface only live edges; `validate` (PHASE-05) is
/// the sole consumer of [`IllegalRow`]s. The per-kind `relation_edges`/`format_show`/
/// `show_json` consumers call this for their tier-1 edges, then concatenate their own
/// typed tier-2/3 edges (the X1 merge order, §5.3 point 3).
pub(crate) fn tier1_edges(source_kind: &Kind, text: &str) -> anyhow::Result<Vec<RelationEdge>> {
    let doc = RelationDoc::parse(text)?;
    let (edges, _illegal) = read_block(source_kind, &doc);
    Ok(edges)
}

/// The targets of one tier-1 `label` among `edges`, in their canonical-then-authored
/// order — the projection the `format_show` / `show_json` consumers splice per axis
/// (e.g. slice's `specs` line, the reconstructed JSON `relationships.specs` array).
/// An axis with no edges yields an empty `Vec`, matching the read-tolerant empty-axis
/// convention every kind's relationship renderer already follows.
pub(crate) fn targets_for(edges: &[RelationEdge], label: RelationLabel) -> Vec<String> {
    edges
        .iter()
        .filter(|e| e.label == label)
        .map(|e| e.target.clone())
        .collect()
}

/// Test-only fixture helper (SL-048 PHASE-04): render an entity's relations in the
/// MIGRATED on-disk shape from structured `axes` — each `(label, targets)`. An axis
/// whose `(source, label)` is a tier-1 migrated rule (`Tier::One` AND NOT the
/// storage-excluded gov `supersedes`, OD-3) becomes `[[relation]]` rows; every other
/// axis (typed tier-2/3, gov `supersedes`, or a non-relation key like `tags`/
/// `superseded_by`) stays in a `[relationships]` table emitted FIRST (F1 — typed
/// tables precede all arrays-of-tables). Mirrors what the one-shot corpus migrator
/// produces, so unit fixtures exercise the post-cut shape the live readers expect.
#[cfg(test)]
pub(crate) fn rels_block(source: &Kind, axes: &[(&str, &[&str])]) -> String {
    let migrated = |label: RelationLabel| -> bool {
        // Tier-1 AND link-writable-or-not-lifecycle: the migration moves a label iff
        // it is `Tier::One` and not the OD-3-excluded gov supersedes (LifecycleOnly).
        lookup(source, label)
            .map(|r| r.tier == Tier::One && r.link != LinkPolicy::LifecycleOnly)
            .unwrap_or(false)
    };
    let mut typed = String::new();
    let mut rows = String::new();
    for (label, targets) in axes {
        let is_migrated = RelationLabel::from_name(label)
            .map(migrated)
            .unwrap_or(false);
        if is_migrated {
            for t in *targets {
                rows.push_str(&format!(
                    "[[relation]]\nlabel = \"{label}\"\ntarget = \"{t}\"\n"
                ));
            }
        } else {
            let list = targets
                .iter()
                .map(|t| format!("\"{t}\""))
                .collect::<Vec<_>>()
                .join(", ");
            typed.push_str(&format!("{label} = [{list}]\n"));
        }
    }
    let typed_table = if typed.is_empty() {
        String::new()
    } else {
        format!("[relationships]\n{typed}")
    };
    format!("{typed_table}{rows}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_name_is_stable() {
        assert_eq!(RelationLabel::Supersedes.name(), "supersedes");
        assert_eq!(RelationLabel::OwningSlice.name(), "owning_slice");
        assert_eq!(RelationLabel::Drift.name(), "drift");
        assert_eq!(RelationLabel::DecisionRef.name(), "decision_ref");
    }

    #[test]
    fn edge_carries_label_and_target() {
        let e = RelationEdge::new(RelationLabel::Specs, "PRD-010".to_string());
        assert_eq!(e.label, RelationLabel::Specs);
        assert_eq!(e.target, "PRD-010");
    }

    use crate::adr::ADR_KIND;
    use crate::backlog::{CHORE_KIND, IDEA_KIND, IMPROVEMENT_KIND, ISSUE_KIND, RISK_KIND};
    use crate::entity::Kind;
    use crate::policy::POLICY_KIND;
    use crate::rec::REC_KIND;
    use crate::requirement::REQUIREMENT_KIND;
    use crate::review::REVIEW_KIND;
    use crate::slice::SLICE_KIND;
    use crate::spec::{PRODUCT_SPEC_KIND, TECH_SPEC_KIND};
    use crate::standard::STANDARD_KIND;

    /// The distinct labels of `RELATION_RULES`, in declaration order.
    fn distinct_labels_in_decl_order() -> Vec<RelationLabel> {
        let mut seen: Vec<RelationLabel> = Vec::new();
        for r in RELATION_RULES {
            if !seen.contains(&r.label) {
                seen.push(r.label);
            }
        }
        seen
    }

    /// VT-1 (R2-C1 / X1): the `RelationLabel` enum `Ord` (declaration order) MUST
    /// equal the `RELATION_RULES` distinct-label declaration order. Asserted by the
    /// PROPERTY — the distinct-label sequence derived from the table is already in
    /// strictly-ascending enum `Ord`, and sorting it leaves it unchanged. This keeps
    /// `inspect`'s `BTreeMap<RelationLabel>` regroup canonical against the table.
    #[test]
    fn enum_ord_matches_relation_rules_label_order() {
        let from_table = distinct_labels_in_decl_order();
        let mut sorted = from_table.clone();
        sorted.sort();
        assert_eq!(
            from_table, sorted,
            "RELATION_RULES distinct-label declaration order diverged from RelationLabel enum Ord"
        );
    }

    /// VT-2 (R2-M2): for every PRE-EXISTING tier-1 label, the table's `sources` set
    /// matches what the SHIPPED `relation_edges` accessor emits. `members` is pinned
    /// PRD·SPEC (`spec::relation_edges` is subtype-blind). Compared by `prefix` (the
    /// canonical `Kind` identity). Asserts the property — the legal source-set —
    /// against the shipped accessors' emit behaviour, not a restatement of the table.
    #[test]
    fn sources_match_shipped_accessors() {
        // (label, the source prefixes the shipped accessor proves can emit it).
        // slice::relation_edges emits specs/requirements/supersedes for SL.
        // backlog::relation_edges emits slices/specs/drift for every backlog kind.
        // governance::relation_edges emits supersedes/related for ADR·POL·STD.
        // spec::relation_edges (subtype-blind) emits descends_from/parent/members/
        //   interactions; members is the one design-corrected PRD·SPEC cell.
        // review::relation_edges emits reviews for RV; rec::relation_edges emits
        //   owning_slice/decision_ref for REC.
        let expected: &[(RelationLabel, &[&str])] = &[
            (
                RelationLabel::Specs,
                &["SL", "ISS", "IMP", "CHR", "RSK", "IDE"],
            ),
            (RelationLabel::Requirements, &["SL"]),
            (RelationLabel::Supersedes, &["SL", "ADR", "POL", "STD"]),
            (RelationLabel::DescendsFrom, &["SPEC"]),
            (RelationLabel::Parent, &["SPEC"]),
            (RelationLabel::Members, &["PRD", "SPEC"]),
            (RelationLabel::Interactions, &["SPEC"]),
            (RelationLabel::Slices, &["ISS", "IMP", "CHR", "RSK", "IDE"]),
            (RelationLabel::Related, &["ADR", "POL", "STD"]),
            (RelationLabel::Reviews, &["RV"]),
            (RelationLabel::OwningSlice, &["REC"]),
            (RelationLabel::Drift, &["ISS", "IMP", "CHR", "RSK", "IDE"]),
            (RelationLabel::DecisionRef, &["REC"]),
        ];
        for (label, want_prefixes) in expected {
            let mut got: Vec<&str> = RELATION_RULES
                .iter()
                .filter(|r| r.label == *label)
                .flat_map(|r| r.sources.iter().map(|k| k.prefix))
                .collect();
            got.sort_unstable();
            got.dedup();
            let mut want: Vec<&str> = want_prefixes.to_vec();
            want.sort_unstable();
            want.dedup();
            assert_eq!(
                got, want,
                "RELATION_RULES source set for {label:?} diverged from the shipped accessor"
            );
        }
    }

    /// VT-3 (R2-M3): `inbound_name == name()` for EVERY pre-existing label; the ONLY
    /// labels whose inbound spelling differs from their outbound `name()` are
    /// `supersedes` ("superseded by"), `governed_by` ("governs"), `consumes`
    /// ("consumed_by"). Render-text only — behaviour-preservation mandate.
    #[test]
    fn inbound_name_equals_name_except_the_three_inverted() {
        for r in RELATION_RULES {
            let differs = r.inbound_name != r.label.name();
            let allowed_to_differ = matches!(
                r.label,
                RelationLabel::Supersedes | RelationLabel::GovernedBy | RelationLabel::Consumes
            );
            if differs {
                assert!(
                    allowed_to_differ,
                    "{:?} inbound_name {:?} differs from name() {:?} but is not an allowed inverted label",
                    r.label,
                    r.inbound_name,
                    r.label.name()
                );
            }
        }
        // And the three inverted labels carry exactly their pinned inbound spelling.
        assert_eq!(
            lookup(&SLICE_KIND, RelationLabel::Supersedes)
                .unwrap()
                .inbound_name,
            "superseded by"
        );
        assert_eq!(
            lookup(&SLICE_KIND, RelationLabel::GovernedBy)
                .unwrap()
                .inbound_name,
            "governs"
        );
        assert_eq!(
            lookup(&PRODUCT_SPEC_KIND, RelationLabel::Consumes)
                .unwrap()
                .inbound_name,
            "consumed_by"
        );
    }

    /// VT-4 (ADR-010 D4/D5): `RELATION_RULES` admits OUTBOUND labels only — no
    /// inverse/derived spelling (`superseded_by`, `governs`, `consumed_by`) is
    /// expressible as a rule's `label`. The derived reciprocal lives ONLY in
    /// `inbound_name` (render text); there is no `RelationLabel` variant for it and
    /// thus no row whose `label.name()` is an inverse spelling — structurally
    /// un-authorable in `[[relation]]`.
    #[test]
    fn no_rule_label_is_an_inverse_spelling() {
        const INVERSE_SPELLINGS: &[&str] = &["superseded_by", "governs", "consumed_by"];
        for r in RELATION_RULES {
            assert!(
                !INVERSE_SPELLINGS.contains(&r.label.name()),
                "{:?} round-trips to an inverse outbound spelling {:?} — inverses are derived, not authorable",
                r.label,
                r.label.name()
            );
        }
        // The inverse spellings only ever appear as inbound render text, never as a
        // label name — confirms the outbound/inbound split is structural.
        assert!(
            RELATION_RULES
                .iter()
                .any(|r| r.inbound_name == "superseded by"),
            "expected the supersedes rule to carry the inverted inbound text"
        );
    }

    /// A self-check that the VT-1 / VT-2 enumerations stay exhaustive: the full
    /// variant list used by the property tests must match the table's distinct
    /// labels (so a future variant cannot silently escape the order/source audits).
    #[test]
    fn every_variant_appears_in_the_table() {
        const ALL: &[RelationLabel] = &[
            RelationLabel::Specs,
            RelationLabel::Requirements,
            RelationLabel::Supersedes,
            RelationLabel::DescendsFrom,
            RelationLabel::Parent,
            RelationLabel::Members,
            RelationLabel::Interactions,
            RelationLabel::GovernedBy,
            RelationLabel::Consumes,
            RelationLabel::Slices,
            RelationLabel::Related,
            RelationLabel::Reviews,
            RelationLabel::OwningSlice,
            RelationLabel::Drift,
            RelationLabel::DecisionRef,
        ];
        // ALL is declared in enum order; assert it is sorted (catches a mis-ordered
        // literal) and that it equals the table's distinct-label sequence.
        let mut sorted = ALL.to_vec();
        sorted.sort();
        assert_eq!(ALL, sorted.as_slice(), "ALL is not in enum Ord order");
        assert_eq!(
            distinct_labels_in_decl_order(),
            ALL.to_vec(),
            "RELATION_RULES does not cover exactly the RelationLabel variants in order"
        );
    }

    /// The `tier` axis (design §5.2 storage-shape column): tier-1 = the uniform
    /// `[[relation]]` block; tier-2/3 = bespoke typed structures. Pins the partition
    /// the PHASE-04 migration acts on. Governance `supersedes` is tier-1 BY SHAPE
    /// even though storage-excluded (the `link` axis, not `tier`, carries that).
    #[test]
    fn tier_partition_matches_design() {
        let tier_one = [
            RelationLabel::Specs,
            RelationLabel::Requirements,
            RelationLabel::Supersedes,
            RelationLabel::GovernedBy,
            RelationLabel::Consumes,
            RelationLabel::Slices,
            RelationLabel::Related,
            RelationLabel::Drift,
        ];
        for r in RELATION_RULES {
            let want = if tier_one.contains(&r.label) {
                Tier::One
            } else {
                Tier::Typed
            };
            assert_eq!(
                r.tier, want,
                "{:?} tier diverged from the design storage-shape column",
                r.label
            );
        }
    }

    /// The `target` axis (design §5.2 forward-validation column): the `TargetSpec`
    /// variant per label. `SameKind` for governance `supersedes`/`related`,
    /// `Unvalidated` for `drift`/`decision_ref`, `AnyNumbered` for `reviews`,
    /// `Kinds` for everything else. Reads `r.target` (the forward-validation axis
    /// PHASE-05 consumes) and pins it now.
    #[test]
    fn target_spec_matches_design() {
        for r in RELATION_RULES {
            match (r.label, r.sources) {
                // gov supersedes + gov related → SameKind.
                (RelationLabel::Related, _) => {
                    assert!(
                        matches!(r.target, TargetSpec::SameKind),
                        "related → SameKind"
                    );
                }
                (RelationLabel::Supersedes, s) if !s.iter().any(|k| k.prefix == "SL") => {
                    assert!(
                        matches!(r.target, TargetSpec::SameKind),
                        "gov supersedes → SameKind"
                    );
                }
                (RelationLabel::Drift | RelationLabel::DecisionRef, _) => {
                    assert!(
                        matches!(r.target, TargetSpec::Unvalidated),
                        "{:?} → Unvalidated",
                        r.label
                    );
                }
                (RelationLabel::Reviews, _) => {
                    assert!(
                        matches!(r.target, TargetSpec::AnyNumbered),
                        "reviews → AnyNumbered"
                    );
                }
                // Everything else points at an explicit kind set; reading the inner
                // slice exercises the `Kinds` payload (forward-validation target).
                (_, _) => match r.target {
                    TargetSpec::Kinds(ks) => {
                        assert!(!ks.is_empty(), "{:?} → non-empty Kinds set", r.label)
                    }
                    other => panic!(
                        "{:?} expected an explicit Kinds target, got {}",
                        r.label,
                        match other {
                            TargetSpec::SameKind => "SameKind",
                            TargetSpec::AnyNumbered => "AnyNumbered",
                            TargetSpec::Unvalidated => "Unvalidated",
                            TargetSpec::Kinds(_) => unreachable!(),
                        }
                    ),
                },
            }
        }
        // governed_by points at the three governance kinds specifically.
        if let TargetSpec::Kinds(ks) = lookup(&SLICE_KIND, RelationLabel::GovernedBy)
            .unwrap()
            .target
        {
            let mut got: Vec<&str> = ks.iter().map(|k| k.prefix).collect();
            got.sort_unstable();
            assert_eq!(got, ["ADR", "POL", "STD"]);
        } else {
            panic!("governed_by → Kinds([ADR,POL,STD])");
        }
    }

    // -- PHASE-03: read_block legality (VT-2) + canonical order (VT-3) -------

    /// (label, target) pairs for ergonomic edge assertions.
    fn edge_pairs(edges: &[RelationEdge]) -> Vec<(RelationLabel, &str)> {
        edges.iter().map(|e| (e.label, e.target.as_str())).collect()
    }

    /// VT-2 (X2): the generic parser preserves the per-kind legality the hardcoded
    /// readers had for free. A slice row carrying `related` and a backlog row carrying
    /// `governed_by` ⇒ `IllegalRow` (IllegalForSource), NEVER a live edge; a legal row
    /// ⇒ a `RelationEdge`. An unknown label spelling ⇒ `IllegalRow` (UnknownLabel).
    #[test]
    fn read_block_rejects_illegal_source_label_pairs() {
        // A slice authoring `related` (a governance-only label) plus a legal `specs`.
        let slice_doc = RelationDoc::parse(
            "[[relation]]\nlabel = \"related\"\ntarget = \"SL-002\"\n\
             [[relation]]\nlabel = \"specs\"\ntarget = \"PRD-010\"\n",
        )
        .unwrap();
        let (edges, illegal) = read_block(&SLICE_KIND, &slice_doc);
        assert_eq!(
            edge_pairs(&edges),
            vec![(RelationLabel::Specs, "PRD-010")],
            "the legal specs row emits an edge"
        );
        assert_eq!(
            illegal,
            vec![IllegalRow {
                label: "related".to_string(),
                target: "SL-002".to_string(),
                reason: IllegalReason::IllegalForSource,
            }],
            "related is illegal for a slice source — a finding, not a live edge"
        );

        // A backlog item authoring `governed_by` (SL·PRD·SPEC-only) plus a legal `slices`.
        let backlog_doc = RelationDoc::parse(
            "[[relation]]\nlabel = \"governed_by\"\ntarget = \"ADR-010\"\n\
             [[relation]]\nlabel = \"slices\"\ntarget = \"SL-020\"\n",
        )
        .unwrap();
        let (edges, illegal) = read_block(&ISSUE_KIND, &backlog_doc);
        assert_eq!(edge_pairs(&edges), vec![(RelationLabel::Slices, "SL-020")]);
        assert_eq!(
            illegal,
            vec![IllegalRow {
                label: "governed_by".to_string(),
                target: "ADR-010".to_string(),
                reason: IllegalReason::IllegalForSource,
            }],
            "governed_by is illegal for a backlog source"
        );

        // An unknown label spelling (a typo / an inverse spelling) ⇒ UnknownLabel.
        let bad_doc = RelationDoc::parse(
            "[[relation]]\nlabel = \"superseded_by\"\ntarget = \"SL-001\"\n\
             [[relation]]\nlabel = \"nonsense\"\ntarget = \"X\"\n",
        )
        .unwrap();
        let (edges, illegal) = read_block(&SLICE_KIND, &bad_doc);
        assert!(edges.is_empty(), "no legal edges from unknown labels");
        assert_eq!(
            illegal,
            vec![
                IllegalRow {
                    label: "superseded_by".to_string(),
                    target: "SL-001".to_string(),
                    reason: IllegalReason::UnknownLabel,
                },
                IllegalRow {
                    label: "nonsense".to_string(),
                    target: "X".to_string(),
                    reason: IllegalReason::UnknownLabel,
                },
            ],
            "an inverse spelling and a typo are both UnknownLabel findings, verbatim"
        );
    }

    /// VT-3 (X1): rows authored OUT of canonical order emit edges in `RELATION_RULES`
    /// declaration order for the source kind; within one label, authored row order is
    /// preserved. The slice canonical run is specs → requirements → supersedes; author
    /// them reversed and interleave a duplicate-label pair to prove the stable
    /// same-label order.
    #[test]
    fn read_block_emits_in_canonical_order_stable_within_label() {
        let doc = RelationDoc::parse(
            // Authored order: supersedes, requirements (R-002 then R-001), specs.
            "[[relation]]\nlabel = \"supersedes\"\ntarget = \"SL-000\"\n\
             [[relation]]\nlabel = \"requirements\"\ntarget = \"REQ-002\"\n\
             [[relation]]\nlabel = \"requirements\"\ntarget = \"REQ-001\"\n\
             [[relation]]\nlabel = \"specs\"\ntarget = \"PRD-010\"\n",
        )
        .unwrap();
        let (edges, illegal) = read_block(&SLICE_KIND, &doc);
        assert!(illegal.is_empty(), "all rows are legal for a slice");
        assert_eq!(
            edge_pairs(&edges),
            vec![
                // Canonical RELATION_RULES order — specs, requirements, supersedes …
                (RelationLabel::Specs, "PRD-010"),
                // … with the two requirements rows in their AUTHORED order (002, 001).
                (RelationLabel::Requirements, "REQ-002"),
                (RelationLabel::Requirements, "REQ-001"),
                (RelationLabel::Supersedes, "SL-000"),
            ],
            "edges land in canonical table order; same-label rows keep authored order"
        );
    }

    /// An empty / absent `[[relation]]` block parses to no edges and no findings — the
    /// read-tolerant convention (a hand-trimmed file is valid input).
    #[test]
    fn read_block_empty_block_is_no_edges_no_findings() {
        let doc = RelationDoc::parse("id = 1\ntitle = \"x\"\n").unwrap();
        let (edges, illegal) = read_block(&SLICE_KIND, &doc);
        assert!(edges.is_empty());
        assert!(illegal.is_empty());
    }

    /// `lookup` keys on `(source ∈ sources, label)`: an illegal pairing returns
    /// `None` (the X2 legality `read_block` will enforce), a legal one the rule.
    #[test]
    fn lookup_keys_on_source_and_label() {
        // A backlog item cannot author `governed_by`; a slice cannot author `related`.
        assert!(lookup(&ISSUE_KIND, RelationLabel::GovernedBy).is_none());
        assert!(lookup(&SLICE_KIND, RelationLabel::Related).is_none());
        // governed_by is legal for SL, PRD, SPEC.
        for k in [&SLICE_KIND, &PRODUCT_SPEC_KIND, &TECH_SPEC_KIND] {
            assert!(lookup(k, RelationLabel::GovernedBy).is_some());
        }
        // consumes is legal for PRD only, not the tech spec.
        assert!(lookup(&PRODUCT_SPEC_KIND, RelationLabel::Consumes).is_some());
        assert!(lookup(&TECH_SPEC_KIND, RelationLabel::Consumes).is_none());
        // supersedes resolves to the SL→SL rule for a slice, the gov rule for ADR.
        let sl_sup = lookup(&SLICE_KIND, RelationLabel::Supersedes).unwrap();
        assert_eq!(sl_sup.link, LinkPolicy::Writable);
        let adr_sup = lookup(&ADR_KIND.kind, RelationLabel::Supersedes).unwrap();
        assert_eq!(adr_sup.link, LinkPolicy::LifecycleOnly);
        // Touch the remaining kind statics so the imports are all exercised.
        let _ = (
            &REQUIREMENT_KIND,
            &REVIEW_KIND,
            &REC_KIND,
            &STANDARD_KIND,
            &POLICY_KIND,
            &IMPROVEMENT_KIND,
            &CHORE_KIND,
            &RISK_KIND,
            &IDEA_KIND,
        );
        let _: &Kind = &SLICE_KIND;
    }
}
