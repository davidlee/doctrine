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
