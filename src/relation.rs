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
        reason = "SL-096 PHASE-01/02 — read_record exercises from_name/RELATION_RULES/lookup/read_block/tier1_edges; PHASE-02 wires outbound_for to knowledge::relation_edges (more callers for tier1_edges); remaining dead symbols (validate_link, check_target_kind, append_edge/remove_edge, writable_labels_for, owning_verb_for) self-clear when their command handlers land"
    )
)]

/// The outbound relation vocabulary — one label per authored relation axis across
/// the six edge-authoring kinds. `Copy + Ord` so callers can group/sort labels
/// deterministically (no `HashMap` iteration order — REQ-077).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
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
    /// concept-map → any (concept association).
    Contextualizes,
    /// knowledge record → any artefact. Epistemic influence — the record shapes
    /// the target's design; inbound renders `shaped_by` (SL-096 PHASE-01).
    Shapes,
    /// knowledge record → backlog item. Work creation — the record spawned a
    /// backlog item; inbound renders `spawned_by` (SL-096 PHASE-01).
    Spawns,
    /// slice·PRD·SPEC·CM → governance (ADR/POL/STD). One shared label spanning all
    /// four sources, as `supersedes` already spans SL+gov; inbound renders
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
    /// revision → governance/spec truth (SPEC·PRD·REQ·ADR·POL·STD). The typed
    /// `[[change]]`-row payload IS the edge set (the members.toml precedent, SL-066
    /// design §4.4): a `revision change add` row of `(target, action)` projects to one
    /// `Revises` edge. `LinkPolicy::TypedVerbOnly` — `doctrine link … revises …` is
    /// refused; the rule row exists for target validation + inbound-reciprocity naming
    /// (`inspect ADR-X` lists every REV that revises it), NOT a writable Tier-1 edge.
    Revises,
    /// revision → RFC. A single provenance ref authored AT CREATION TIME via
    /// `revision new --originates-from <RFC-NNN>` — the REV's `[[relation]]` block
    /// carries ONE `originates_from` row. `LinkPolicy::TypedVerbOnly` — `doctrine
    /// link … originates_from …` is refused; the rule row exists for target
    /// validation + inbound-reciprocity naming (`inspect RFC-NNN` lists every REV
    /// that originates from it as `precursor of`).
    OriginatesFrom,
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
            RelationLabel::Contextualizes => "contextualizes",
            RelationLabel::Shapes => "shapes",
            RelationLabel::Spawns => "spawns",
            RelationLabel::GovernedBy => "governed_by",
            RelationLabel::Consumes => "consumes",
            RelationLabel::Slices => "slices",
            RelationLabel::Related => "related",
            RelationLabel::Reviews => "reviews",
            RelationLabel::OwningSlice => "owning_slice",
            RelationLabel::Drift => "drift",
            RelationLabel::DecisionRef => "decision_ref",
            RelationLabel::Revises => "revises",
            RelationLabel::OriginatesFrom => "originates_from",
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
            "contextualizes" => RelationLabel::Contextualizes,
            "shapes" => RelationLabel::Shapes,
            "spawns" => RelationLabel::Spawns,
            "governed_by" => RelationLabel::GovernedBy,
            "consumes" => RelationLabel::Consumes,
            "slices" => RelationLabel::Slices,
            "related" => RelationLabel::Related,
            "reviews" => RelationLabel::Reviews,
            "owning_slice" => RelationLabel::OwningSlice,
            "drift" => RelationLabel::Drift,
            "decision_ref" => RelationLabel::DecisionRef,
            "revises" => RelationLabel::Revises,
            "originates_from" => RelationLabel::OriginatesFrom,
            _ => return None,
        };
        // Defence-in-depth: the spelling must round-trip, so `name()` stays the single
        // source of every label string (a future drift between the two maps trips this
        // in the test build, where `from_name` is exercised).
        debug_assert_eq!(label.name(), name);
        Some(label)
    }
}

use anyhow::Context;

use crate::entity::Kind;
use crate::kinds::{
    ADR, ASM, BACKLOG, CHR, CM, CON, DEC, GOV, IDE, IMP, ISS, POL, PRD, QUE, REC, RECORD, REQ, REV,
    RFC, RSK, RV, SL, SPEC, STD,
};

/// What an outbound label's target ref is allowed to resolve to — the forward-edge
/// validation axis (design §5.2, the first of the five axes). The rule-table element
/// type is `&'static str` (a `kinds::*` prefix, the canonical kind identity compared
/// by `==`); diagnostics format the `RelationLabel`, never the `TargetSpec`.
#[derive(Clone, Copy)]
pub(crate) enum TargetSpec {
    /// The target must be one of an explicit set of numbered kinds (e.g.
    /// `governed_by` → ADR·POL·STD).
    Kinds(&'static [&'static str]),
    /// The target kind must equal the source kind — governance `supersedes` and
    /// `related` (each gov kind → its own kind). One rule serves a source-set whose
    /// members each point within their own namespace (R2-M1).
    SameKind,
    /// The target may be any numbered kind — RV `reviews` (the subject of a review
    /// is any entity).
    AnyNumbered,
    /// The target is free-text, not a doctrine entity (`drift`, `decision_ref`):
    /// a `decision_ref` is an *external* 3-part forgettable cite (e.g. `DEC-005-C`),
    /// not the 2-part numbered DEC kind — so it never resolves, always dangles, no
    /// overlay (ADR-010 D2).
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
/// one rule serves multiple source kinds (F2 — never one row per kind). Its elements
/// are `&'static str` kind prefixes (`kinds::*`, compared by `==`); diagnostics format
/// `label` only.
#[derive(Clone, Copy)]
pub(crate) struct RelationRule {
    /// The source kinds that may author this label (a set, not one row per kind).
    pub(crate) sources: &'static [&'static str],
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
        sources: &[SL, ISS, IMP, CHR, RSK, IDE],
        label: RelationLabel::Specs,
        inbound_name: "specs",
        target: TargetSpec::Kinds(&[PRD, SPEC]),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: &[SL],
        label: RelationLabel::Requirements,
        inbound_name: "requirements",
        target: TargetSpec::Kinds(&[REQ]),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    // supersedes — two rules at one slot: SL→SL (writable) and gov→same-gov
    // (lifecycle-only, storage-excluded OD-3).
    RelationRule {
        sources: &[SL],
        label: RelationLabel::Supersedes,
        inbound_name: "superseded by",
        target: TargetSpec::Kinds(&[SL]),
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
        sources: RECORD,
        label: RelationLabel::Supersedes,
        inbound_name: "superseded by",
        target: TargetSpec::Kinds(RECORD),
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
        sources: &[CM],
        label: RelationLabel::Contextualizes,
        inbound_name: "contextualized_by",
        target: TargetSpec::Unvalidated,
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: RECORD,
        label: RelationLabel::Shapes,
        inbound_name: "shaped_by",
        target: TargetSpec::Kinds(&[
            PRD, SPEC, REQ, SL, ISS, IMP, CHR, RSK, IDE, ADR, POL, STD, ASM, DEC, QUE, CON,
        ]),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: RECORD,
        label: RelationLabel::Spawns,
        inbound_name: "spawned_by",
        target: TargetSpec::Kinds(&[ISS, IMP, CHR, RSK, IDE]),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: &[SL, PRD, SPEC, CM, ASM, DEC, QUE, CON],
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
        target: TargetSpec::Kinds(&[SL]),
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
        sources: &[SL, RFC],
        label: RelationLabel::Related,
        inbound_name: "related",
        target: TargetSpec::AnyNumbered,
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
        target: TargetSpec::Kinds(&[SL]),
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
    // revises (SL-066, ADR-013) — REV → governance/spec truth. Tier-2 typed: the
    // `[[change]]`-row payload IS the edge set (members.toml precedent), authored ONLY
    // by `revision change add`, NEVER by `doctrine link` (TypedVerbOnly). The rule row
    // exists for target validation + inbound naming ("revises" on `inspect ADR-X`), not
    // a writable Tier-1 edge. Targets are the six authored-truth kinds (NO SL/work/REC).
    RelationRule {
        sources: &[REV],
        label: RelationLabel::Revises,
        inbound_name: "revises",
        target: TargetSpec::Kinds(&[SPEC, PRD, REQ, ADR, POL, STD]),
        tier: Tier::Typed,
        link: LinkPolicy::TypedVerbOnly,
    },
    // originates_from (SL-122) — REV → RFC: a single provenance ref authored at
    // `revision new --originates-from <RFC-NNN>` creation time (NOT a `[[change]]`
    // row). `LinkPolicy::TypedVerbOnly` — `doctrine link … originates_from …` is
    // refused; the rule row exists for target validation + inbound-reciprocity naming
    // (`inspect RFC-NNN` lists "precursor of: REV-NNN").
    RelationRule {
        sources: &[REV],
        label: RelationLabel::OriginatesFrom,
        inbound_name: "precursor of",
        target: TargetSpec::Kinds(&[RFC]),
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
        .find(|r| r.label == label && r.sources.contains(&source.prefix))
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
        .position(|r| r.label == label && r.sources.contains(&source.prefix))
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

// ---------------------------------------------------------------------------
// PHASE-05 — the tier-1 write seam (link/unlink). Edit-preserving toml_edit over
// a `DocumentMut` (comments / inert tables / unknown keys survive verbatim —
// `mem.pattern.entity.edit-preserving-status-transition`), idempotent, with the
// F1 EOF-append defence (R2-m1, design §5.3/§5.5).
// ---------------------------------------------------------------------------

/// The outcome of [`append_edge`] — idempotent. `Wrote` ⇒ a new `[[relation]]` row
/// was appended; `Noop` ⇒ the `(label, target)` row already existed, file untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppendOutcome {
    Wrote,
    Noop,
}

/// The outcome of [`remove_edge`] — idempotent. `Removed` ⇒ a matching `[[relation]]`
/// row was deleted; `Absent` ⇒ no such row, file untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemoveOutcome {
    Removed,
    Absent,
}

/// Does the top-level document place a typed table or array AFTER the first
/// `[[relation]]` array-of-tables? That is the F1 trap (design §5.1/§5.3): a bare
/// `[relationships]` header sitting *after* the `[[relation]]` array would, on a naive
/// tail-`insert`, bind the new keys INTO the last array element = silent corruption.
/// We refuse rather than corrupt. Walks the document in source order; once a
/// `[[relation]]` array is seen, any later non-`relation` top-level item is the trap.
/// Returns the offending key for the refusal message, or `None` when the layout is safe
/// (every typed table precedes all `[[relation]]` arrays — the migrator's F1 shape).
fn trailing_typed_table_after_relation(doc: &toml_edit::DocumentMut) -> Option<String> {
    let mut seen_relation = false;
    for (key, item) in doc.as_table() {
        if key == "relation" && item.is_array_of_tables() {
            seen_relation = true;
        } else if seen_relation {
            // A non-`relation` top-level item authored AFTER the relation array — the
            // F1 trap. (A second `relation` key cannot occur — toml has one per table.)
            return Some(key.to_string());
        }
    }
    None
}

/// Append one tier-1 `[[relation]]` row to an entity's authored TOML `text`, edit-
/// preserving and idempotent (design §5.3). PURE: text in, text out — the impure
/// read/write shell is [`append_edge`]. Order of operations is load-bearing:
///
/// 1. **Idempotent no-op guard FIRST** — if a `[[relation]]` row already carries this
///    `(label, target)`, return `Noop` with the text byte-unchanged (before any
///    structural assert, so a re-link of an already-linked edge never even inspects
///    the layout — `mem.pattern.entity.edit-preserving-status-transition`).
/// 2. **F1 EOF-append defence** ([`trailing_typed_table_after_relation`]) — refuse a
///    hand-edited file whose typed table trails the `[[relation]]` array, rather than
///    tail-inserting into the last array element (R2-m1). The refusal is an
///    `IllegalRow`-class hard error, never a silent corruption.
/// 3. Append via `toml_edit::value(target)` — escapes the target automatically, so a
///    target with a quote/backslash can never break out of the string literal
///    (`mem.pattern.render.toml-splice-escape-user-values`); never `.replace()`-splice.
fn append_relation_row(
    text: &str,
    label: RelationLabel,
    target: &str,
) -> anyhow::Result<(String, AppendOutcome)> {
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| anyhow::anyhow!("parse TOML for relation append: {e}"))?;

    // (1) idempotent no-op guard — before any structural inspection.
    if relation_row_present(&doc, label, target) {
        return Ok((text.to_string(), AppendOutcome::Noop));
    }

    // (2) F1 defence — refuse a trailing typed table rather than corrupt it.
    if let Some(offending) = trailing_typed_table_after_relation(&doc) {
        anyhow::bail!(
            "refusing to append [[relation]]: typed table `[{offending}]` is authored AFTER \
             the [[relation]] array (F1 — appending would corrupt it by tail-inserting into \
             the last array element). Re-home `[{offending}]` above the [[relation]] block."
        );
    }

    // (3) append a new array-of-tables element with escaped values.
    let array = doc
        .as_table_mut()
        .entry("relation")
        .or_insert_with(|| toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new()))
        .as_array_of_tables_mut()
        .ok_or_else(|| {
            anyhow::anyhow!("`relation` is present but is not an array-of-tables (corrupt file)")
        })?;
    let mut row = toml_edit::Table::new();
    row.insert("label", toml_edit::value(label.name()));
    row.insert("target", toml_edit::value(target));
    array.push(row);

    Ok((doc.to_string(), AppendOutcome::Wrote))
}

/// Remove one tier-1 `[[relation]]` row from `text`, edit-preserving and idempotent.
/// PURE — the impure shell is [`remove_edge`]. Removes EVERY array element matching
/// `(label, target)` (a hand-duplicated pair collapses to one logical edge, so both
/// rows go); `Absent` when none match (idempotent double-unlink). Comments and every
/// other table survive verbatim (the `DocumentMut` round-trip).
fn remove_relation_row(
    text: &str,
    label: RelationLabel,
    target: &str,
) -> anyhow::Result<(String, RemoveOutcome)> {
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| anyhow::anyhow!("parse TOML for relation remove: {e}"))?;
    let Some(array) = doc
        .as_table_mut()
        .get_mut("relation")
        .and_then(toml_edit::Item::as_array_of_tables_mut)
    else {
        return Ok((text.to_string(), RemoveOutcome::Absent));
    };
    let before = array.len();
    array.retain(|row| !row_matches(row, label, target));
    if array.len() == before {
        return Ok((text.to_string(), RemoveOutcome::Absent));
    }
    Ok((doc.to_string(), RemoveOutcome::Removed))
}

/// Is there a `[[relation]]` row carrying exactly this `(label, target)` in `doc`?
/// The idempotency oracle for both verbs — reads the live array-of-tables, comparing
/// the authored `label`/`target` strings verbatim.
fn relation_row_present(doc: &toml_edit::DocumentMut, label: RelationLabel, target: &str) -> bool {
    doc.as_table()
        .get("relation")
        .and_then(toml_edit::Item::as_array_of_tables)
        .is_some_and(|array| array.iter().any(|row| row_matches(row, label, target)))
}

/// One `[[relation]]` array element matches `(label, target)` iff both string cells
/// equal the queried values verbatim.
fn row_matches(row: &toml_edit::Table, label: RelationLabel, target: &str) -> bool {
    row.get("label").and_then(toml_edit::Item::as_str) == Some(label.name())
        && row.get("target").and_then(toml_edit::Item::as_str) == Some(target)
}

/// Append a tier-1 `[[relation]]` edge to the entity TOML at `toml_path` (design §5.3).
/// The impure shell over the pure [`append_relation_row`]: read the file, apply the
/// edit-preserving append, write it back ONLY when a row was actually added (`Wrote`)
/// — a `Noop` never rewrites the file (no spurious mtime churn). The caller resolves
/// `toml_path` from `(source_kind, id)` via `integrity::KINDS` (the command shell).
pub(crate) fn append_edge(
    toml_path: &std::path::Path,
    label: RelationLabel,
    target: &str,
) -> anyhow::Result<AppendOutcome> {
    let text = std::fs::read_to_string(toml_path)
        .map_err(|e| anyhow::anyhow!("read {} for relation append: {e}", toml_path.display()))?;
    let (next, outcome) = append_relation_row(&text, label, target)?;
    if outcome == AppendOutcome::Wrote {
        crate::fsutil::write_atomic(toml_path, next.as_bytes())
            .with_context(|| format!("write {} after relation append", toml_path.display()))?;
    }
    Ok(outcome)
}

/// Remove a tier-1 `[[relation]]` edge from the entity TOML at `toml_path` (design
/// §5.3). The impure shell over the pure [`remove_relation_row`]: write back only on
/// `Removed`; `Absent` leaves the file untouched (idempotent double-unlink).
pub(crate) fn remove_edge(
    toml_path: &std::path::Path,
    label: RelationLabel,
    target: &str,
) -> anyhow::Result<RemoveOutcome> {
    let text = std::fs::read_to_string(toml_path)
        .map_err(|e| anyhow::anyhow!("read {} for relation remove: {e}", toml_path.display()))?;
    let (next, outcome) = remove_relation_row(&text, label, target)?;
    if outcome == RemoveOutcome::Removed {
        crate::fsutil::write_atomic(toml_path, next.as_bytes())
            .with_context(|| format!("write {} after relation remove", toml_path.display()))?;
    }
    Ok(outcome)
}

/// The derived-inbound render text for `label` (design §5.5 X5 / R2-M3): the
/// `inbound_name` the [`RELATION_RULES`] rows pin for that label. Every rule carrying a
/// given label declares the SAME `inbound_name` (VT-3 pins this), so the FIRST match is
/// authoritative: `governed_by` renders governs, `consumes` renders consumed-by,
/// `supersedes` renders superseded-by; every other label renders its own `name()`.
/// Table-driven: the human inbound render reads this so the `supersedes` special-case
/// collapses into one path; the `--json` inbound keeps the raw label regardless (R2-M3).
/// Falls back to `name()` for a label with no rule (defensive — every variant is in the
/// table by `every_variant_appears_in_the_table`).
pub(crate) fn inbound_name(label: RelationLabel) -> &'static str {
    RELATION_RULES
        .iter()
        .find(|r| r.label == label)
        .map_or(label.name(), |r| r.inbound_name)
}

/// The `link`-writable labels a `source_kind` may author, as their authored spellings
/// — for the refusal message that lists the legal labels (design §5.4 step 2). Only
/// `LinkPolicy::Writable` rules are offered; `LifecycleOnly`/`TypedVerbOnly` labels are
/// authored through their own verbs, not generic `link`.
fn writable_labels_for(source: &Kind) -> Vec<&'static str> {
    RELATION_RULES
        .iter()
        .filter(|r| r.link == LinkPolicy::Writable && r.sources.contains(&source.prefix))
        .map(|r| r.label.name())
        .collect()
}

/// The verb that DOES author a non-`link`-writable label, named in the refusal so the
/// user is pointed at the right tool (design §5.4 step 2):
/// - `LifecycleOnly` (governance `supersedes`) ⇒ the transactional supersede verb
///   (IMP-006, unbuilt) — never plain `link`.
/// - `TypedVerbOnly` ⇒ the kind's bespoke verb (`spec req add`, `spec parent`,
///   `review …`).
fn owning_verb_for(rule: &RelationRule) -> &'static str {
    match rule.link {
        LinkPolicy::Writable => "link",
        LinkPolicy::LifecycleOnly => "the transactional supersede verb (IMP-006)",
        LinkPolicy::TypedVerbOnly => "the kind's typed verb (e.g. `spec req add`, `review …`)",
    }
}

/// Validate that `source_kind` may author `label_str` via the generic `link` verb,
/// returning the governing [`RelationRule`] (design §5.4 step 2). PURE — no disk, no
/// target resolution. Two refusals, both naming the remedy:
/// - the `(source, label)` pair is off-table (an unknown label, or a real label this
///   source may not author) ⇒ error listing the source's legal `link` labels;
/// - the label is real but `link ≠ Writable` ⇒ error naming the owning verb.
pub(crate) fn validate_link(
    source_kind: &Kind,
    label_str: &str,
) -> anyhow::Result<&'static RelationRule> {
    let legal = || writable_labels_for(source_kind).join(", ");
    let label = RelationLabel::from_name(label_str).ok_or_else(|| {
        anyhow::anyhow!(
            "`{label_str}` is not a relation label authorable by {} via `link`. Legal labels: {}",
            source_kind.prefix,
            legal()
        )
    })?;
    let rule = lookup(source_kind, label).ok_or_else(|| {
        anyhow::anyhow!(
            "{} may not author `{label_str}` (illegal for this source). Legal `link` labels: {}",
            source_kind.prefix,
            legal()
        )
    })?;
    anyhow::ensure!(
        rule.link == LinkPolicy::Writable,
        "`{label_str}` is not `link`-writable — author it through {}, not generic `link`",
        owning_verb_for(rule)
    );
    Ok(rule)
}

/// The forward-edge legal-KIND check (design §5.5, R2-M1 — NEW code). Given a `rule`
/// already known `link`-writable, the `source_kind`, and the target's parsed kind
/// `target_prefix`, assert the target kind is admissible for the rule's [`TargetSpec`]:
/// - `Kinds(set)` ⇒ `target_prefix` must be in `set` (e.g. `governed_by` → ADR·POL·STD,
///   so `link SL-048 governed_by SL-003` is REFUSED even though `SL-003` resolves);
/// - `SameKind` ⇒ `target_prefix == source_kind.prefix` (governance `related` → same
///   gov kind; a cross-gov target is refused);
/// - `AnyNumbered` ⇒ any numbered kind is fine;
/// - `Unvalidated` ⇒ unreachable here (free-text targets skip the kind check entirely).
///
/// `ensure_ref_resolves` (existence) is the caller's complementary gate — it does NOT
/// check kind, which is exactly why this assertion is needed.
pub(crate) fn check_target_kind(
    rule: &RelationRule,
    source_kind: &Kind,
    target_prefix: &str,
) -> anyhow::Result<()> {
    match rule.target {
        TargetSpec::Kinds(set) => anyhow::ensure!(
            set.contains(&target_prefix),
            "`{}` target must be one of [{}], got a {target_prefix}",
            rule.label.name(),
            set.to_vec().join(", ")
        ),
        TargetSpec::SameKind => anyhow::ensure!(
            target_prefix == source_kind.prefix,
            "`{}` target must be the same kind as the source ({}), got a {target_prefix}",
            rule.label.name(),
            source_kind.prefix
        ),
        TargetSpec::AnyNumbered | TargetSpec::Unvalidated => {}
    }
    Ok(())
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
        // Tier-1 — SL-095 migrated governance supersedes from typed to
        // [[relation]], so the LifecycleOnly exclusion is dropped.
        lookup(source, label)
            .map(|r| r.tier == Tier::One)
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
    use crate::knowledge::ASSUMPTION_KIND;
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
            (
                RelationLabel::Supersedes,
                &["SL", "ADR", "POL", "STD", "ASM", "DEC", "QUE", "CON"],
            ),
            (RelationLabel::DescendsFrom, &["SPEC"]),
            (RelationLabel::Parent, &["SPEC"]),
            (RelationLabel::Members, &["PRD", "SPEC"]),
            (RelationLabel::Interactions, &["SPEC"]),
            (RelationLabel::Slices, &["ISS", "IMP", "CHR", "RSK", "IDE"]),
            (RelationLabel::Related, &["ADR", "POL", "RFC", "SL", "STD"]),
            (RelationLabel::Reviews, &["RV"]),
            (RelationLabel::OwningSlice, &["REC"]),
            (RelationLabel::Drift, &["ISS", "IMP", "CHR", "RSK", "IDE"]),
            (RelationLabel::DecisionRef, &["REC"]),
            (RelationLabel::Shapes, &["ASM", "DEC", "QUE", "CON"]),
            (RelationLabel::Spawns, &["ASM", "DEC", "QUE", "CON"]),
            (RelationLabel::OriginatesFrom, &["REV"]),
        ];
        for (label, want_prefixes) in expected {
            let mut got: Vec<&str> = RELATION_RULES
                .iter()
                .filter(|r| r.label == *label)
                .flat_map(|r| r.sources.iter().copied())
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
                RelationLabel::Supersedes
                    | RelationLabel::GovernedBy
                    | RelationLabel::Consumes
                    | RelationLabel::Contextualizes
                    | RelationLabel::Shapes
                    | RelationLabel::Spawns
                    | RelationLabel::OriginatesFrom
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
        const INVERSE_SPELLINGS: &[&str] = &[
            "superseded_by",
            "governs",
            "consumed_by",
            "contextualized_by",
        ];
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
            RelationLabel::Contextualizes,
            RelationLabel::Shapes,
            RelationLabel::Spawns,
            RelationLabel::GovernedBy,
            RelationLabel::Consumes,
            RelationLabel::Slices,
            RelationLabel::Related,
            RelationLabel::Reviews,
            RelationLabel::OwningSlice,
            RelationLabel::Drift,
            RelationLabel::DecisionRef,
            RelationLabel::Revises,
            RelationLabel::OriginatesFrom,
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
            RelationLabel::Contextualizes,
            RelationLabel::Shapes,
            RelationLabel::Spawns,
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
                // gov related → SameKind; slice related → AnyNumbered.
                (RelationLabel::Related, s) => {
                    if s.iter().any(|k| *k == "ADR") {
                        assert!(
                            matches!(r.target, TargetSpec::SameKind),
                            "gov related → SameKind"
                        );
                    } else {
                        assert!(
                            matches!(r.target, TargetSpec::AnyNumbered),
                            "slice related → AnyNumbered"
                        );
                    }
                }
                (RelationLabel::Supersedes, s) if !s.iter().any(|k| *k == "SL") => {
                    // GOV supersedes is SameKind; RECORD supersedes is Kinds(RECORD).
                    if s.iter().any(|k| *k == "ADR") {
                        assert!(
                            matches!(r.target, TargetSpec::SameKind),
                            "gov supersedes → SameKind"
                        );
                    } else {
                        // RECORD supersedes → Kinds(RECORD). The pattern match on a
                        // constant ref (`Kinds(RECORD)`) is unstable; fall back to a
                        // contents check.
                        match r.target {
                            TargetSpec::Kinds(ks) => {
                                let got: Vec<&str> = ks.iter().copied().collect();
                                let want: Vec<&str> = RECORD.iter().copied().collect();
                                assert_eq!(got, want, "record supersedes → Kinds(RECORD)");
                            }
                            other => panic!(
                                "record supersedes → Kinds(RECORD), got {:?}",
                                std::mem::discriminant(&other)
                            ),
                        }
                    }
                }
                (
                    RelationLabel::Drift
                    | RelationLabel::DecisionRef
                    | RelationLabel::Contextualizes,
                    _,
                ) => {
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
            let mut got: Vec<&str> = ks.iter().copied().collect();
            got.sort_unstable();
            assert_eq!(got, ["ADR", "POL", "STD"]);
        } else {
            panic!("governed_by → Kinds([ADR,POL,STD])");
        }
        // shapes target is the explicit 16-kind set (D2).
        if let TargetSpec::Kinds(ks) = lookup(&ASSUMPTION_KIND, RelationLabel::Shapes)
            .unwrap()
            .target
        {
            let mut got: Vec<&str> = ks.iter().copied().collect();
            got.sort_unstable();
            assert_eq!(
                got,
                [
                    "ADR", "ASM", "CHR", "CON", "DEC", "IDE", "IMP", "ISS", "POL", "PRD", "QUE",
                    "REQ", "RSK", "SL", "SPEC", "STD"
                ]
            );
        } else {
            panic!(
                "shapes → Kinds([PRD, SPEC, REQ, SLICE, ISS, IMP, CHR, RSK, IDE, ADR, POL, STD, ASM, DEC, QUE, CON])"
            );
        }
        // spawns target is the 5 backlog-item kinds.
        if let TargetSpec::Kinds(ks) = lookup(&ASSUMPTION_KIND, RelationLabel::Spawns)
            .unwrap()
            .target
        {
            let mut got: Vec<&str> = ks.iter().copied().collect();
            got.sort_unstable();
            assert_eq!(got, ["CHR", "IDE", "IMP", "ISS", "RSK"]);
        } else {
            panic!("spawns → Kinds([ISS, IMP, CHR, RSK, IDE])");
        }
        // RECORD Supersedes target is Kinds(RECORD), NOT SameKind (D4 — records
        // admit cross-kind supersession; the §6 matrix enforces it at the verb).
        let r = lookup(&ASSUMPTION_KIND, RelationLabel::Supersedes)
            .unwrap_or_else(|| panic!("RECORD Supersedes row not found for ASM"));
        assert_eq!(
            r.link,
            LinkPolicy::LifecycleOnly,
            "record supersedes → LifecycleOnly"
        );
        if let TargetSpec::Kinds(ks) = r.target {
            let mut got: Vec<&str> = ks.iter().copied().collect();
            got.sort_unstable();
            assert_eq!(got, ["ASM", "CON", "DEC", "QUE"]);
        } else {
            panic!("record supersedes → Kinds(RECORD)");
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
        // A slice authoring `related` plus a legal `specs`.
        let slice_doc = RelationDoc::parse(
            "[[relation]]\nlabel = \"related\"\ntarget = \"SL-002\"\n\
             [[relation]]\nlabel = \"specs\"\ntarget = \"PRD-010\"\n",
        )
        .unwrap();
        let (edges, illegal) = read_block(&SLICE_KIND, &slice_doc);
        assert_eq!(
            edge_pairs(&edges),
            vec![
                (RelationLabel::Specs, "PRD-010"),
                (RelationLabel::Related, "SL-002"),
            ],
            "the legal specs and related rows emit edges"
        );
        assert!(illegal.is_empty(), "related is legal for a slice source");

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

    // -- PHASE-05: the tier-1 write seam (append_edge / remove_edge) ----------

    /// Append onto a clean file writes a new `[[relation]]` row with both cells, and
    /// the pre-existing keys/comments survive (edit-preserving). The new row parses
    /// back as a legal edge for the source.
    #[test]
    fn append_relation_row_appends_and_preserves() {
        let text = "# a comment\nid = 1\ntitle = \"x\"\n";
        let (next, outcome) =
            append_relation_row(text, RelationLabel::GovernedBy, "ADR-010").unwrap();
        assert_eq!(outcome, AppendOutcome::Wrote);
        assert!(next.contains("# a comment"), "comment preserved");
        assert!(next.contains("[[relation]]"));
        assert!(next.contains("label = \"governed_by\""));
        assert!(next.contains("target = \"ADR-010\""));
        // Round-trips as a legal slice edge.
        let edges = tier1_edges(&SLICE_KIND, &next).unwrap();
        assert_eq!(
            edge_pairs(&edges),
            vec![(RelationLabel::GovernedBy, "ADR-010")]
        );
    }

    /// Appending the SAME `(label, target)` twice is a `Noop` the second time — the
    /// text is byte-unchanged and no duplicate row is written (idempotent, VT-6).
    #[test]
    fn append_relation_row_is_idempotent() {
        let text = "id = 1\n";
        let (once, o1) = append_relation_row(text, RelationLabel::GovernedBy, "ADR-010").unwrap();
        assert_eq!(o1, AppendOutcome::Wrote);
        let (twice, o2) = append_relation_row(&once, RelationLabel::GovernedBy, "ADR-010").unwrap();
        assert_eq!(o2, AppendOutcome::Noop);
        assert_eq!(once, twice, "a no-op append leaves the text byte-identical");
    }

    /// VT-1 (F1 / R2-m1 — the EOF-append defence): a hand-edited file with a typed
    /// `[relationships]` table placed AFTER a `[[relation]]` array is REFUSED rather
    /// than tail-inserting bare keys into the last array element (silent corruption).
    /// The idempotent no-op guard runs first, so the refusal only fires for a genuinely
    /// new edge.
    #[test]
    fn append_relation_row_refuses_trailing_typed_table() {
        // The F1 trap: a [relationships] header AFTER the [[relation]] array.
        let trap = "id = 1\n\
                    [[relation]]\nlabel = \"specs\"\ntarget = \"PRD-010\"\n\
                    [relationships]\ntags = [\"x\"]\n";
        let err = append_relation_row(trap, RelationLabel::GovernedBy, "ADR-010").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("relationships") && msg.contains("AFTER"),
            "refusal must name the offending trailing table: {msg}"
        );
        // But an ALREADY-present edge is a Noop even on the trap layout (guard-first):
        // re-linking the existing specs row must not trip the structural refusal.
        let (out, outcome) = append_relation_row(trap, RelationLabel::Specs, "PRD-010").unwrap();
        assert_eq!(outcome, AppendOutcome::Noop);
        assert_eq!(out, trap);
    }

    /// `append_relation_row` escapes the target via `toml_edit::value` — a target
    /// carrying a quote cannot break out of the string literal (the migrator never
    /// authors such a target, but the write seam must be splice-safe regardless).
    #[test]
    fn append_relation_row_escapes_target() {
        let text = "id = 1\n";
        let (next, _) = append_relation_row(text, RelationLabel::Drift, "a\"b").unwrap();
        // Parses cleanly (no broken literal) and the target round-trips verbatim.
        let doc = RelationDoc::parse(&next).unwrap();
        let (edges, _illegal) = read_block(&ISSUE_KIND, &doc);
        assert_eq!(edge_pairs(&edges), vec![(RelationLabel::Drift, "a\"b")]);
    }

    /// `remove_relation_row` deletes a matching row (edit-preserving) and is idempotent
    /// — a second remove is `Absent`, the text byte-unchanged (VT-6 double-unlink).
    #[test]
    fn remove_relation_row_round_trips_and_is_idempotent() {
        let (with, _) =
            append_relation_row("id = 1\n", RelationLabel::GovernedBy, "ADR-010").unwrap();
        let (without, o1) =
            remove_relation_row(&with, RelationLabel::GovernedBy, "ADR-010").unwrap();
        assert_eq!(o1, RemoveOutcome::Removed);
        assert!(
            tier1_edges(&SLICE_KIND, &without).unwrap().is_empty(),
            "the edge is gone after remove"
        );
        let (again, o2) =
            remove_relation_row(&without, RelationLabel::GovernedBy, "ADR-010").unwrap();
        assert_eq!(o2, RemoveOutcome::Absent);
        assert_eq!(without, again, "a second remove is a byte-identical no-op");
    }

    /// `inbound_name` is the table-driven derived-inbound render text (X5/R2-M3): the
    /// three inverted labels carry their pinned spelling; every other label renders its
    /// own `name()` so shipped inbound goldens are unchanged.
    #[test]
    fn inbound_name_is_table_driven() {
        assert_eq!(inbound_name(RelationLabel::GovernedBy), "governs");
        assert_eq!(inbound_name(RelationLabel::Consumes), "consumed_by");
        assert_eq!(inbound_name(RelationLabel::Supersedes), "superseded by");
        assert_eq!(inbound_name(RelationLabel::OriginatesFrom), "precursor of");
        // Every non-inverted label renders its own name().
        for label in distinct_labels_in_decl_order() {
            let inverted = matches!(
                label,
                RelationLabel::GovernedBy
                    | RelationLabel::Consumes
                    | RelationLabel::Supersedes
                    | RelationLabel::Contextualizes
                    | RelationLabel::Shapes
                    | RelationLabel::Spawns
                    | RelationLabel::OriginatesFrom
            );
            if !inverted {
                assert_eq!(
                    inbound_name(label),
                    label.name(),
                    "{label:?} inbound render must equal its name()"
                );
            }
        }
    }

    // -- PHASE-05: link validation (validate_link + check_target_kind) --------

    /// `validate_link` accepts a writable `(source, label)` and returns its rule; it
    /// refuses an off-table label (listing the legal ones), an illegal-for-source label,
    /// and a non-`Writable` label (naming the owning verb).
    #[test]
    fn validate_link_gates_source_label_and_policy() {
        // Writable: SL governed_by → ok, returns the GovernedBy rule. (`RelationRule`
        // has no Debug — it holds `&Kind` — so match rather than `.unwrap()`.)
        match validate_link(&SLICE_KIND, "governed_by") {
            Ok(rule) => assert_eq!(rule.label, RelationLabel::GovernedBy),
            Err(e) => panic!("governed_by should be writable for a slice: {e}"),
        }

        // `RelationRule` has no Debug, so `.unwrap_err()` (which Debug-formats Ok) won't
        // compile — extract the refusal message by hand.
        let refusal = |src: &Kind, label: &str| -> String {
            match validate_link(src, label) {
                Ok(_) => panic!("expected `{label}` to be refused for {}", src.prefix),
                Err(e) => e.to_string(),
            }
        };

        // Unknown label spelling — refused, message lists legal labels.
        let e = refusal(&SLICE_KIND, "nonsense");
        assert!(e.contains("governed_by"), "lists legal labels: {e}");

        // A slice CAN author `related` (SL-095) — returns the Related rule.
        match validate_link(&SLICE_KIND, "related") {
            Ok(rule) => assert_eq!(rule.label, RelationLabel::Related),
            Err(e) => panic!("related should be writable for a slice (SL-095): {e}"),
        }

        // Governance `supersedes` is LifecycleOnly — refused, names the supersede verb.
        let e = refusal(&ADR_KIND.kind, "supersedes");
        assert!(e.contains("supersede verb"), "names the owning verb: {e}");

        // A TypedVerbOnly label (spec `members`) — refused, names the typed verb.
        let e = refusal(&PRODUCT_SPEC_KIND, "members");
        assert!(e.contains("typed verb"), "names the typed verb: {e}");
    }

    /// VT-2 (R2-M1): the forward legal-KIND check. `governed_by` (→ ADR·POL·STD) refuses
    /// a slice target even though it would resolve; `SameKind` (gov `related`) refuses a
    /// cross-gov target; the legal kinds pass.
    /// SL-095: a slice `related` now resolves (new BACKLOG/SLICE row); verify
    /// `AnyNumbered` accepts any target kind.
    #[test]
    fn check_target_kind_enforces_target_kind() {
        // `RelationRule` has no Debug — unwrap the rule by hand.
        let unwrap_rule = |r: anyhow::Result<&'static RelationRule>| -> &'static RelationRule {
            match r {
                Ok(rule) => rule,
                Err(e) => panic!("expected a writable rule: {e}"),
            }
        };
        let gov_by = unwrap_rule(validate_link(&SLICE_KIND, "governed_by"));
        // SL-003 (a slice) is NOT a legal governed_by target — refused.
        assert!(check_target_kind(gov_by, &SLICE_KIND, "SL").is_err());
        // ADR/POL/STD all pass.
        for p in ["ADR", "POL", "STD"] {
            assert!(check_target_kind(gov_by, &SLICE_KIND, p).is_ok());
        }

        // SameKind: gov `related` from an ADR accepts an ADR target, refuses a POL.
        let related = unwrap_rule(validate_link(&ADR_KIND.kind, "related"));
        assert!(check_target_kind(related, &ADR_KIND.kind, "ADR").is_ok());
        assert!(
            check_target_kind(related, &ADR_KIND.kind, "POL").is_err(),
            "SameKind refuses a cross-gov target"
        );

        // SL-095: slice `related` targets AnyNumbered — any kind accepted.
        let sl_related = unwrap_rule(validate_link(&SLICE_KIND, "related"));
        assert!(check_target_kind(sl_related, &SLICE_KIND, "ADR").is_ok());
        assert!(check_target_kind(sl_related, &SLICE_KIND, "SPEC").is_ok());
        assert!(check_target_kind(sl_related, &SLICE_KIND, "RV").is_ok());
    }

    /// SL-066 VT-2: the `revises` rule. Source REV, targets the six authored-truth
    /// kinds (off-target — e.g. `revises SL` — refused), `TypedVerbOnly` so generic
    /// `link` is refused (naming the typed verb). The rule row exists for target
    /// validation + inbound naming ("revises"), never as a writable Tier-1 edge.
    #[test]
    fn revises_rule_is_typed_verb_only_with_authored_truth_targets() {
        use crate::revision::REV_KIND;
        // The rule resolves for REV and carries the typed-verb policy.
        let rule = lookup(&REV_KIND, RelationLabel::Revises).expect("revises rule for REV");
        assert_eq!(rule.link, LinkPolicy::TypedVerbOnly);
        assert_eq!(rule.tier, Tier::Typed);
        assert_eq!(rule.inbound_name, "revises");

        // `doctrine link … revises …` is refused (TypedVerbOnly), naming the typed verb.
        match validate_link(&REV_KIND, "revises") {
            Ok(_) => panic!("`link … revises …` must be refused (TypedVerbOnly)"),
            Err(e) => assert!(e.to_string().contains("typed verb"), "names the verb: {e}"),
        }

        // Target validation: the six authored-truth kinds pass; off-target refused.
        for p in ["SPEC", "PRD", "REQ", "ADR", "POL", "STD"] {
            assert!(
                check_target_kind(rule, &REV_KIND, p).is_ok(),
                "{p} is a legal revises target"
            );
        }
        for p in ["SL", "ISS", "REC", "REV", "RV"] {
            assert!(
                check_target_kind(rule, &REV_KIND, p).is_err(),
                "{p} is NOT a legal revises target (off-target)"
            );
        }
    }

    /// `lookup` keys on `(source ∈ sources, label)`: an illegal pairing returns
    /// `None` (the X2 legality `read_block` will enforce), a legal one the rule.
    #[test]
    fn lookup_keys_on_source_and_label() {
        // A backlog item cannot author `governed_by`; a slice can author `related`.
        assert!(lookup(&ISSUE_KIND, RelationLabel::GovernedBy).is_none());
        let sl_related = lookup(&SLICE_KIND, RelationLabel::Related);
        assert!(sl_related.is_some());
        assert!(matches!(
            sl_related.unwrap().target,
            TargetSpec::AnyNumbered
        ));
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
        // Shapes is legal for record kinds, illegal for SL.
        assert!(lookup(&ASSUMPTION_KIND, RelationLabel::Shapes).is_some());
        assert!(lookup(&SLICE_KIND, RelationLabel::Shapes).is_none());
        // Spawns is legal for record kinds, illegal for SL.
        assert!(lookup(&ASSUMPTION_KIND, RelationLabel::Spawns).is_some());
        assert!(lookup(&SLICE_KIND, RelationLabel::Spawns).is_none());
        let _: &Kind = &SLICE_KIND;
    }
}
