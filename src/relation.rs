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
    /// work → canon/backlog/any, refined by a closed [`Role`] (SL-149 / ADR-016): the
    /// structure/intent split — one structural label whose `(source, label, role)` key
    /// drives target validation. `implements` (SL → SPEC·PRD·REQ), `scoped_from` (SL →
    /// backlog), `concerns` (work → any numbered). Inbound is role-derived, so
    /// [`inbound_name`] is keyed `(label, role)` for this label. The migration target of
    /// the retired `specs`/`requirements`/mismapped-`related` edges (PHASE-05).
    References,
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
            RelationLabel::References => "references",
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
            "references" => RelationLabel::References,
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

/// The closed intent dimension that refines the `references` label (SL-149 / ADR-016 —
/// the structure/intent split). A relation's durable *structure* is the label; its
/// *contextual intent* is the role. Only `references` rows carry a role
/// ([`RelationRule::role`] is `Some` there, `None` everywhere else). `Copy + Ord` so the
/// canonical role order is the declaration order (mirrors `RelationLabel`): no `HashMap`
/// iteration on the relation path (REQ-077).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub(crate) enum Role {
    /// SL → SPEC·PRD·REQ: the slice implements that canonical truth. Inbound
    /// "implemented by". The migration target of the retired `specs`/`requirements`
    /// SL→canon edges (PHASE-05).
    Implements,
    /// SL → backlog: the slice was scoped from that idea/improvement/issue/chore/risk.
    /// Inbound "scoped into". Strictly the *origin* edge — kept separate from the Axis D
    /// `part_of` containment edge (design §F7).
    ScopedFrom,
    /// work → any numbered entity: aboutness / relevance ("this work concerns that
    /// artefact"). Inbound "concerned by". The widest role; absorbs the RFC `bears_on`
    /// edges (the `reviews`-in-prose cases SL-145 deferred — ADR-016 §1 corollary).
    Concerns,
}

impl Role {
    /// The stable wire/render spelling of a role — the inverse of [`from_name`](Self::from_name).
    /// Single source for the role string (the `role = "…"` cell PHASE-03 serialises).
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Role::Implements => "implements",
            Role::ScopedFrom => "scoped_from",
            Role::Concerns => "concerns",
        }
    }

    /// Parse an authored `role = "…"` spelling back to its variant — the inverse of
    /// [`name`](Self::name). `None` for any string that names no role. The exhaustive
    /// `match` keeps it in lock-step with the enum (a new variant fails to compile until
    /// it is added here); the `debug_assert` pins the round-trip so `name()` stays the
    /// single source of every role string.
    pub(crate) fn from_name(name: &str) -> Option<Role> {
        let role = match name {
            "implements" => Role::Implements,
            "scoped_from" => Role::ScopedFrom,
            "concerns" => Role::Concerns,
            _ => return None,
        };
        debug_assert_eq!(role.name(), name);
        Some(role)
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
    /// a `decision_ref` is an *external* 3-part the external decision register cite (e.g. `DEC-005-C`),
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
    /// The intent dimension that refines a `references` row (SL-149) — `Some` ONLY on
    /// `references` rows, `None` on every other label. The lookup key extends from
    /// `(source, label)` to `(source, label, role)`: each `(source, label)` is wholly
    /// roleful (every row `Some`) or wholly roleless (every row `None`) — VT-4 pins it.
    pub(crate) role: Option<Role>,
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
    // references (SL-149 / ADR-016) — the work→canon class collapsed onto one
    // structural label refined by a closed Role. One row per (source-set, role); the
    // lookup key is (source, label, role). These coexist with the retained
    // specs/requirements rows through PHASE-04; PHASE-05's migration rewrites the old
    // edges onto these and drops Specs/Requirements. Source sets are PINNED from a live
    // census (design §2.4): implements/scoped_from are SL-only; concerns rides one wide
    // source-set row.
    RelationRule {
        // implements — SL → canonical truth (the migration target of specs/requirements
        // SL→{SPEC,PRD,REQ}). SL-only: backlog items spawn slices that implement; they
        // do not implement canon directly.
        sources: &[SL],
        label: RelationLabel::References,
        role: Some(Role::Implements),
        inbound_name: "implemented by",
        target: TargetSpec::Kinds(&[SPEC, PRD, REQ]),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        // scoped_from — SL → backlog: this slice was scoped from that idea/improvement.
        // SL-only; target the backlog kinds (BACKLOG = ISS/IMP/CHR/RSK/IDE). Kept
        // strictly separate from `part_of` (Axis D containment, design §F7).
        sources: &[SL],
        label: RelationLabel::References,
        role: Some(Role::ScopedFrom),
        inbound_name: "scoped into",
        target: TargetSpec::Kinds(BACKLOG),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        // concerns — work → any numbered entity (aboutness/relevance). One wide
        // source-set row pinned from the live census: SL + RFC + the backlog kinds
        // + RECORD (ASM/DEC/QUE/CON — D6). Target AnyNumbered.
        sources: &[SL, RFC, ISS, IMP, CHR, RSK, IDE, ASM, DEC, QUE, CON],
        label: RelationLabel::References,
        role: Some(Role::Concerns),
        inbound_name: "concerned by",
        target: TargetSpec::AnyNumbered,
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    // supersedes — two rules at one slot: SL→SL (writable) and gov→same-gov
    // (lifecycle-only, storage-excluded OD-3).
    RelationRule {
        sources: &[SL],
        label: RelationLabel::Supersedes,
        role: None,
        inbound_name: "superseded by",
        target: TargetSpec::Kinds(&[SL]),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: GOV,
        label: RelationLabel::Supersedes,
        role: None,
        inbound_name: "superseded by",
        target: TargetSpec::SameKind,
        tier: Tier::One,
        link: LinkPolicy::LifecycleOnly,
    },
    RelationRule {
        sources: RECORD,
        label: RelationLabel::Supersedes,
        role: None,
        inbound_name: "superseded by",
        target: TargetSpec::Kinds(RECORD),
        tier: Tier::One,
        link: LinkPolicy::LifecycleOnly,
    },
    RelationRule {
        sources: &[SPEC],
        label: RelationLabel::DescendsFrom,
        role: None,
        inbound_name: "descends_from",
        target: TargetSpec::Kinds(&[PRD]),
        tier: Tier::Typed,
        link: LinkPolicy::TypedVerbOnly,
    },
    RelationRule {
        sources: &[SPEC, PRD],
        label: RelationLabel::Parent,
        role: None,
        inbound_name: "parent",
        target: TargetSpec::Kinds(&[SPEC, PRD]),
        tier: Tier::Typed,
        link: LinkPolicy::TypedVerbOnly,
    },
    RelationRule {
        sources: &[PRD, SPEC],
        label: RelationLabel::Members,
        role: None,
        inbound_name: "members",
        target: TargetSpec::Kinds(&[REQ]),
        tier: Tier::Typed,
        link: LinkPolicy::TypedVerbOnly,
    },
    RelationRule {
        sources: &[SPEC],
        label: RelationLabel::Interactions,
        role: None,
        inbound_name: "interactions",
        target: TargetSpec::Kinds(&[SPEC]),
        tier: Tier::Typed,
        link: LinkPolicy::TypedVerbOnly,
    },
    RelationRule {
        sources: &[CM],
        label: RelationLabel::Contextualizes,
        role: None,
        inbound_name: "contextualized_by",
        target: TargetSpec::Unvalidated,
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: RECORD,
        label: RelationLabel::Shapes,
        role: None,
        inbound_name: "shaped_by",
        target: TargetSpec::Kinds(&[
            PRD, SPEC, REQ, SL, ISS, IMP, CHR, RSK, IDE, ADR, POL, STD, RFC, ASM, DEC, QUE, CON,
        ]),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: RECORD,
        label: RelationLabel::Spawns,
        role: None,
        inbound_name: "spawned_by",
        target: TargetSpec::Kinds(&[ISS, IMP, CHR, RSK, IDE]),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        // SL-145: BACKLOG (ISS/IMP/CHR/RSK/IDE) widened in so a backlog item may be
        // governed by an ADR/POL/STD. Target gate (Kinds(GOV)) unchanged.
        sources: &[
            SL, PRD, SPEC, CM, ASM, DEC, QUE, CON, ISS, IMP, CHR, RSK, IDE,
        ],
        label: RelationLabel::GovernedBy,
        role: None,
        inbound_name: "governs",
        target: TargetSpec::Kinds(GOV),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: &[PRD],
        label: RelationLabel::Consumes,
        role: None,
        inbound_name: "consumed_by",
        target: TargetSpec::Kinds(&[PRD]),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: BACKLOG,
        label: RelationLabel::Slices,
        role: None,
        inbound_name: "slices",
        target: TargetSpec::Kinds(&[SL]),
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: GOV,
        label: RelationLabel::Related,
        role: None,
        inbound_name: "related",
        target: TargetSpec::SameKind,
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        // SL-145 (D1): extend this AnyNumbered row — not a new row — so a backlog item may
        // author a peer `related` edge to any numbered entity. Target/tier/inbound unchanged.
        sources: &[SL, RFC, ISS, IMP, CHR, RSK, IDE],
        label: RelationLabel::Related,
        role: None,
        inbound_name: "related",
        target: TargetSpec::AnyNumbered,
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: &[RV],
        label: RelationLabel::Reviews,
        role: None,
        inbound_name: "reviews",
        target: TargetSpec::AnyNumbered,
        tier: Tier::Typed,
        link: LinkPolicy::TypedVerbOnly,
    },
    RelationRule {
        sources: &[REC],
        label: RelationLabel::OwningSlice,
        role: None,
        inbound_name: "owning_slice",
        target: TargetSpec::Kinds(&[SL]),
        tier: Tier::Typed,
        link: LinkPolicy::TypedVerbOnly,
    },
    RelationRule {
        sources: BACKLOG,
        label: RelationLabel::Drift,
        role: None,
        inbound_name: "drift",
        target: TargetSpec::Unvalidated,
        tier: Tier::One,
        link: LinkPolicy::Writable,
    },
    RelationRule {
        sources: &[REC],
        label: RelationLabel::DecisionRef,
        role: None,
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
        role: None,
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
        role: None,
        inbound_name: "precursor of",
        target: TargetSpec::Kinds(&[RFC]),
        tier: Tier::Typed,
        link: LinkPolicy::TypedVerbOnly,
    },
];

/// The rule governing `(source, label, role)` — the table's lookup key (design §5.2 /
/// SL-149 §2.6). A source matches when its `prefix` (the canonical `Kind` identity —
/// `Kind` is data without `PartialEq`, compared by prefix everywhere) is in the rule's
/// `sources`, the label matches, AND the row's `role` equals the queried `role`. A
/// label-only edge passes `role = None` and matches the `role = None` row; a `references`
/// edge is reachable only with the right `Some(role)`. `None` ⇒ illegal for that
/// `(source, label, role)` (the X2 per-kind legality `read_block` enforces).
pub(crate) fn lookup(
    source: &Kind,
    label: RelationLabel,
    role: Option<Role>,
) -> Option<&'static RelationRule> {
    RELATION_RULES
        .iter()
        .find(|r| r.label == label && r.role == role && r.sources.contains(&source.prefix))
}

/// The roles reachable for `(source, label)` — the rows whose `(label, sources)` admit
/// this source, projected to their `Some(role)` (SL-149 §2.6). Empty for a label-only
/// label (every matching row is roleless) or an illegal `(source, label)` pair; drives
/// the `MissingRole`/`IllegalRole` gate in [`validate_link`] and the CLI error message.
/// Yields roles in `RELATION_RULES` declaration order (canonical = `Role` `Ord`).
pub(crate) fn legal_roles(source: &Kind, label: RelationLabel) -> impl Iterator<Item = Role> + '_ {
    RELATION_RULES
        .iter()
        .filter(move |r| r.label == label && r.sources.contains(&source.prefix))
        .filter_map(|r| r.role)
}

/// Does `(source, label)` admit ANY row at all (roleful or roleless)? The legality the
/// X2 gate checks before deciding `MissingRole`/`RoleNotApplicable` (so an off-table
/// `(source, label)` is refused as illegal, not mis-reported as a role problem).
fn source_label_admitted(source: &Kind, label: RelationLabel) -> bool {
    RELATION_RULES
        .iter()
        .any(|r| r.label == label && r.sources.contains(&source.prefix))
}

/// One authored outbound relation: its [`RelationLabel`], the intent [`Role`] that
/// refines it (`Some` only on `references` edges — SL-149 §2.5), and the canonical ref
/// string it points at. Edge identity is the `(label, role, target)` triple. `target`
/// is the authored ref verbatim and MAY be free-text or dangling — resolution (and
/// dangler classification) happens later, at the graph scan (PHASE-03); the accessor
/// never resolves (design §5.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RelationEdge {
    pub(crate) label: RelationLabel,
    /// The intent dimension (SL-149): `Some` on a `references` edge, `None` on every
    /// label-only edge. Threaded through the storage seam by [`read_block`]; the typed
    /// tier-2/3 accessors and the label-only `link` path construct roleless edges via
    /// [`new`](Self::new).
    pub(crate) role: Option<Role>,
    pub(crate) target: String,
}

impl RelationEdge {
    /// Construct a label-only (roleless) edge — the common case for every label-only
    /// axis (typed tier-2/3 readers, `governed_by`/`related`/…). `references` edges are
    /// built via [`with_role`](Self::with_role).
    pub(crate) fn new(label: RelationLabel, target: String) -> Self {
        Self {
            label,
            role: None,
            target,
        }
    }

    /// Construct a roled edge — `read_block` uses this for a `references` row whose
    /// `role` cell parsed to a legal [`Role`]. The `(label, role, target)` triple is the
    /// edge's identity (idempotency, render, validation).
    pub(crate) fn with_role(label: RelationLabel, role: Option<Role>, target: String) -> Self {
        Self {
            label,
            role,
            target,
        }
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
    /// The `(source, label)` pair is legal, but the row's `role` cell is wrong for it
    /// (SL-149): a `references` row with NO `role` key, a `role` naming no [`Role`], a
    /// role illegal for this source, OR a label-only row carrying a stray `role` key.
    /// The role-class finding `validate` reports for a hand-edited `references` row.
    IllegalRole,
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct RelationRow {
    label: String,
    /// The intent role of a `references` row, authored verbatim (SL-149 §2.5). Present
    /// ONLY on a `references` `[[relation]]` row (`role = "implements"`); a label-only
    /// row carries no `role` key — `skip_serializing_if` keeps the serialised shape
    /// label-only, which is load-bearing for diff stability. `#[serde(default)]` so a
    /// label-only row parses with `role = None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    role: Option<String>,
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
        let illegal_row = |reason: IllegalReason| IllegalRow {
            label: row.label.clone(),
            target: row.target.clone(),
            reason,
        };
        let Some(label) = RelationLabel::from_name(&row.label) else {
            illegal.push(illegal_row(IllegalReason::UnknownLabel));
            continue;
        };
        // The `(source, label)` pair must be admitted by SOME row first; an off-table
        // pair is the plain illegal-for-source finding (NOT mis-reported as a role
        // problem), preserving the X2 legality the hardcoded readers had for free.
        if !source_label_admitted(source_kind, label) {
            illegal.push(illegal_row(IllegalReason::IllegalForSource));
            continue;
        }
        // Resolve the row's authored `role` cell to an `Option<Role>` (SL-149 §2.5). A
        // `role` string that names no `Role` is itself a role-class finding.
        let role = match &row.role {
            None => None,
            Some(spelling) => {
                let Some(parsed) = Role::from_name(spelling) else {
                    illegal.push(illegal_row(IllegalReason::IllegalRole));
                    continue;
                };
                Some(parsed)
            }
        };
        // Role legality: a `references` (roleful) pair demands a legal role; a label-only
        // pair refuses a stray `role`. `lookup` is role-keyed, so the canonical position
        // is taken for the SAME `(source, label, role)` key — a role mismatch misses and
        // is reported, never silently emitted.
        let Some(pos) = canonical_position(source_kind, label, role) else {
            illegal.push(illegal_row(IllegalReason::IllegalRole));
            continue;
        };
        legal.push((
            pos,
            RelationEdge::with_role(label, role, row.target.clone()),
        ));
    }
    // Stable sort by canonical position: same-label rows keep authored order (X1).
    legal.sort_by_key(|(pos, _)| *pos);
    let edges = legal.into_iter().map(|(_, e)| e).collect();
    (edges, illegal)
}

/// The index of the FIRST `RELATION_RULES` row that legalises `(source, label, role)`,
/// or `None` if no such row exists — a label-only edge keys on `role = None`, a
/// `references` edge on its `Some(role)`, so a missing/illegal/stray role misses
/// (SL-149). The index is the canonical-order key `read_block` sorts by (X1) — and
/// because the table is declared in `RelationLabel` enum-`Ord` order (VT-1), distinct
/// labels sort into the same order `inspect`'s `BTreeMap` regroup produces, keeping
/// every render surface canonical.
fn canonical_position(source: &Kind, label: RelationLabel, role: Option<Role>) -> Option<usize> {
    RELATION_RULES
        .iter()
        .position(|r| r.label == label && r.role == role && r.sources.contains(&source.prefix))
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

/// The targets of one `(label, role)` pair among `edges`, in their authored order —
/// the role-keyed sibling of [`targets_for`] (SL-149 PHASE-04b). The per-kind
/// `show`/`show --json` consumers splice the `references` axis by role
/// (`{implements, scoped_from, concerns}`), each role a separate array. An axis with no
/// edges yields an empty `Vec` (the read-tolerant empty-axis convention). Pure — no IO,
/// no resolution.
pub(crate) fn targets_for_role(
    edges: &[RelationEdge],
    label: RelationLabel,
    role: Role,
) -> Vec<String> {
    edges
        .iter()
        .filter(|e| e.label == label && e.role == Some(role))
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
    role: Option<Role>,
    target: &str,
) -> anyhow::Result<(String, AppendOutcome)> {
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| anyhow::anyhow!("parse TOML for relation append: {e}"))?;

    // (1) idempotent no-op guard — before any structural inspection. Keys on the FULL
    // `(label, role, target)` triple (SL-149): `references(implements) X` is distinct
    // from `references(concerns) X`, so re-linking one never masks the other.
    if relation_row_present(&doc, label, role, target) {
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
    // The `role` cell rides ONLY when the edge carries a role (SL-149 §2.5): a
    // `references` row serialises `role = "implements"`; a label-only row carries NO
    // `role` key — load-bearing for diff stability. The key sits between `label` and
    // `target` so the on-disk shape reads `label / role / target`.
    if let Some(r) = role {
        row.insert("role", toml_edit::value(r.name()));
    }
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
    role: Option<Role>,
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
    array.retain(|row| !row_matches(row, label, role, target));
    if array.len() == before {
        return Ok((text.to_string(), RemoveOutcome::Absent));
    }
    Ok((doc.to_string(), RemoveOutcome::Removed))
}

/// Is there a `[[relation]]` row carrying exactly this `(label, role, target)` triple in
/// `doc`? The idempotency oracle for both verbs — reads the live array-of-tables,
/// comparing the authored `label`/`role`/`target` cells verbatim.
fn relation_row_present(
    doc: &toml_edit::DocumentMut,
    label: RelationLabel,
    role: Option<Role>,
    target: &str,
) -> bool {
    doc.as_table()
        .get("relation")
        .and_then(toml_edit::Item::as_array_of_tables)
        .is_some_and(|array| {
            array
                .iter()
                .any(|row| row_matches(row, label, role, target))
        })
}

/// One `[[relation]]` array element matches `(label, role, target)` iff all three cells
/// equal the queried values verbatim (SL-149 — identity is the full triple). The `role`
/// cell matches `None` iff the row carries NO `role` key, and `Some(r)` iff the row's
/// `role` string equals `r.name()`.
fn row_matches(
    row: &toml_edit::Table,
    label: RelationLabel,
    role: Option<Role>,
    target: &str,
) -> bool {
    let row_role = row.get("role").and_then(toml_edit::Item::as_str);
    row.get("label").and_then(toml_edit::Item::as_str) == Some(label.name())
        && row_role == role.map(Role::name)
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
    role: Option<Role>,
    target: &str,
) -> anyhow::Result<AppendOutcome> {
    let text = std::fs::read_to_string(toml_path)
        .map_err(|e| anyhow::anyhow!("read {} for relation append: {e}", toml_path.display()))?;
    // SL-149 PHASE-04c: the public `append_edge` shell threads the caller's `role`
    // straight to the role-aware pure seam. `link --role` passes `Some(role)` for a
    // `references` edge; the supersede verbs and the label-only `link` path pass `None`.
    // Triple-keyed `(label, role, target)` idempotency is proven at the pure layer.
    let (next, outcome) = append_relation_row(&text, label, role, target)?;
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
    role: Option<Role>,
    target: &str,
) -> anyhow::Result<RemoveOutcome> {
    let text = std::fs::read_to_string(toml_path)
        .map_err(|e| anyhow::anyhow!("read {} for relation remove: {e}", toml_path.display()))?;
    // SL-149 PHASE-04c: threads the caller's `role` to the role-aware pure seam — the
    // `(label, role, target)` triple is the removal identity (see `append_edge`).
    let (next, outcome) = remove_relation_row(&text, label, role, target)?;
    if outcome == RemoveOutcome::Removed {
        crate::fsutil::write_atomic(toml_path, next.as_bytes())
            .with_context(|| format!("write {} after relation remove", toml_path.display()))?;
    }
    Ok(outcome)
}

/// The derived-inbound render text for `(label, role)` (design §5.5 X5 / R2-M3, re-keyed
/// SL-149 §2.6/D5): the `inbound_name` the [`RELATION_RULES`] rows pin for that
/// `(label, role)`. Every rule carrying a given `(label, role)` declares the SAME
/// `inbound_name` (VT-3 pins this), so the FIRST match is authoritative. Label-only edges
/// pass `role = None`: `governed_by` renders governs, `consumes` consumed-by, `supersedes`
/// superseded-by; every other label-only label renders its own `name()`. `references`
/// rows are role-keyed: `implements` → "implemented by", `scoped_from` → "scoped into",
/// `concerns` → "concerned by". Table-driven so the `supersedes` special-case collapses
/// into one path; the `--json` inbound keeps the raw label regardless (R2-M3). Falls back
/// to `name()` for a `(label, role)` with no rule (defensive).
pub(crate) fn inbound_name(label: RelationLabel, role: Option<Role>) -> &'static str {
    RELATION_RULES
        .iter()
        .find(|r| r.label == label && r.role == role)
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

/// Validate that `source_kind` may author `label_str` (refined by `role`) via the generic
/// `link` verb, returning the governing [`RelationRule`] (design §5.4 step 2, re-keyed
/// SL-149 §2.6). PURE — no disk, no target resolution. Refusals, each naming the remedy:
/// - the `(source, label)` pair is off-table (an unknown label, or a real label this
///   source may not author) ⇒ error listing the source's legal `link` labels;
/// - `MissingRole` — `label` is roleful (`references`) but `role` is `None` (the CLI
///   omitted `--role`); the message lists the legal roles;
/// - `RoleNotApplicable` — `role` is `Some` but `label` is label-only (e.g. `governed_by`);
/// - `IllegalRole` — `role` is `Some` but not in `legal_roles(source, label)`;
/// - the row is real but `link ≠ Writable` ⇒ error naming the owning verb.
pub(crate) fn validate_link(
    source_kind: &Kind,
    label_str: &str,
    role: Option<Role>,
) -> anyhow::Result<&'static RelationRule> {
    let legal = || writable_labels_for(source_kind).join(", ");
    let label = RelationLabel::from_name(label_str).ok_or_else(|| {
        anyhow::anyhow!(
            "`{label_str}` is not a relation label authorable by {} via `link`. Legal labels: {}",
            source_kind.prefix,
            legal()
        )
    })?;
    // The `(source, label)` pair must be admitted by SOME row before we can classify a
    // role problem; otherwise it is the plain illegal-for-source refusal.
    anyhow::ensure!(
        source_label_admitted(source_kind, label),
        "{} may not author `{label_str}` (illegal for this source). Legal `link` labels: {}",
        source_kind.prefix,
        legal()
    );
    // Role gate (SL-149): a roleful label (`references`) demands a role; a label-only
    // label refuses one. `roles_here` is empty exactly for a label-only `(source, label)`.
    let roles_here: Vec<Role> = legal_roles(source_kind, label).collect();
    let roleful = !roles_here.is_empty();
    match (roleful, role) {
        (true, None) => anyhow::bail!(
            "`{label_str}` requires a role — author it with `--role <{}>`",
            roles_here
                .iter()
                .map(|r| r.name())
                .collect::<Vec<_>>()
                .join("|")
        ),
        (false, Some(r)) => anyhow::bail!(
            "`{label_str}` does not take a role; remove `--role {}`",
            r.name()
        ),
        (true, Some(r)) => anyhow::ensure!(
            roles_here.contains(&r),
            "`{}` is not a legal role for {} `{label_str}` — legal roles: {}",
            r.name(),
            source_kind.prefix,
            roles_here
                .iter()
                .map(|lr| lr.name())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        (false, None) => {}
    }
    let rule = lookup(source_kind, label, role).ok_or_else(|| {
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
        // [[relation]], so the LifecycleOnly exclusion is dropped. Label-only lookup
        // (role None) — this helper authors the legacy label-keyed shape; references
        // (roleful) is exercised by the role-aware suites, not this fixture.
        lookup(source, label, None)
            .map(|r| r.tier == Tier::One)
            .unwrap_or(false)
    };
    let mut typed = String::new();
    let mut rows = String::new();
    for (label, targets) in axes {
        // SL-149: a `references(<role>)` label string authors a roled `[[relation]]` row
        // (the migration target of the old specs/requirements/related edges).
        if let Some(role) = label
            .strip_prefix("references(")
            .and_then(|s| s.strip_suffix(')'))
        {
            for t in *targets {
                rows.push_str(&format!(
                    "[[relation]]\nlabel = \"references\"\nrole = \"{role}\"\ntarget = \"{t}\"\n"
                ));
            }
            continue;
        }
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
        let e = RelationEdge::with_role(
            RelationLabel::References,
            Some(Role::Implements),
            "PRD-010".to_string(),
        );
        assert_eq!(e.label, RelationLabel::References);
        assert_eq!(e.target, "PRD-010");
    }

    #[test]
    fn targets_for_role_filters_by_label_and_role() {
        // SL-149 PHASE-04b: the role-keyed sibling of `targets_for`. Splits a mixed
        // `references` axis into its three role buckets; a label-only edge and a
        // different-label edge are both excluded.
        let edges = vec![
            RelationEdge::with_role(
                RelationLabel::References,
                Some(Role::Implements),
                "SPEC-018".into(),
            ),
            RelationEdge::with_role(
                RelationLabel::References,
                Some(Role::Concerns),
                "RFC-003".into(),
            ),
            RelationEdge::with_role(
                RelationLabel::References,
                Some(Role::Implements),
                "PRD-010".into(),
            ),
            // a different label sharing no role bucket
            RelationEdge::new(RelationLabel::Supersedes, "SL-009".into()),
        ];
        assert_eq!(
            targets_for_role(&edges, RelationLabel::References, Role::Implements),
            vec!["SPEC-018".to_string(), "PRD-010".to_string()],
        );
        assert_eq!(
            targets_for_role(&edges, RelationLabel::References, Role::Concerns),
            vec!["RFC-003".to_string()],
        );
        assert!(
            targets_for_role(&edges, RelationLabel::References, Role::ScopedFrom).is_empty(),
            "an empty role bucket yields an empty Vec",
        );
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
        // slice::relation_edges emits references/supersedes for SL.
        // backlog::relation_edges emits slices/references/drift plus (post-SL-145)
        // governed_by/related for every backlog kind.
        // governance::relation_edges emits supersedes/related for ADR·POL·STD.
        // spec::relation_edges (subtype-blind) emits descends_from/parent/members/
        //   interactions; members is the one design-corrected PRD·SPEC cell.
        // review::relation_edges emits reviews for RV; rec::relation_edges emits
        //   owning_slice/decision_ref for REC.
        let expected: &[(RelationLabel, &[&str])] = &[
            // SL-149 PHASE-05: specs/requirements collapsed into references; the union of
            // its three rows' source-sets is the pinned census set.
            (
                RelationLabel::References,
                &[
                    "SL", "RFC", "ISS", "IMP", "CHR", "RSK", "IDE", "ASM", "DEC", "QUE", "CON",
                ],
            ),
            (
                RelationLabel::Supersedes,
                &["SL", "ADR", "POL", "STD", "ASM", "DEC", "QUE", "CON"],
            ),
            (RelationLabel::DescendsFrom, &["SPEC"]),
            (RelationLabel::Parent, &["PRD", "SPEC"]),
            (RelationLabel::Members, &["PRD", "SPEC"]),
            (RelationLabel::Interactions, &["SPEC"]),
            (RelationLabel::Slices, &["ISS", "IMP", "CHR", "RSK", "IDE"]),
            (
                RelationLabel::Related,
                &[
                    "ADR", "POL", "RFC", "SL", "STD", "ISS", "IMP", "CHR", "RSK", "IDE",
                ],
            ),
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
                    // SL-149: references is role-derived inbound — every references row's
                    // inbound differs from name() ("implemented by"/"scoped into"/"concerned by").
                    | RelationLabel::References
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
            lookup(&SLICE_KIND, RelationLabel::Supersedes, None)
                .unwrap()
                .inbound_name,
            "superseded by"
        );
        assert_eq!(
            lookup(&SLICE_KIND, RelationLabel::GovernedBy, None)
                .unwrap()
                .inbound_name,
            "governs"
        );
        assert_eq!(
            lookup(&PRODUCT_SPEC_KIND, RelationLabel::Consumes, None)
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
            RelationLabel::References,
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
            RelationLabel::References,
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
                // SL-149: references is role-keyed — the (label, role) → TargetSpec gate
                // golden. implements → Kinds(SPEC,PRD,REQ); scoped_from → Kinds(BACKLOG);
                // concerns → AnyNumbered.
                (RelationLabel::References, _) => match r.role {
                    Some(Role::Implements) => match r.target {
                        TargetSpec::Kinds(ks) => {
                            let mut got: Vec<&str> = ks.iter().copied().collect();
                            got.sort_unstable();
                            assert_eq!(got, ["PRD", "REQ", "SPEC"], "implements → SPEC·PRD·REQ");
                        }
                        _ => panic!("references(implements) → Kinds(SPEC,PRD,REQ)"),
                    },
                    Some(Role::ScopedFrom) => match r.target {
                        TargetSpec::Kinds(ks) => {
                            let got: Vec<&str> = ks.iter().copied().collect();
                            let want: Vec<&str> = BACKLOG.iter().copied().collect();
                            assert_eq!(got, want, "scoped_from → Kinds(BACKLOG)");
                        }
                        _ => panic!("references(scoped_from) → Kinds(BACKLOG)"),
                    },
                    Some(Role::Concerns) => assert!(
                        matches!(r.target, TargetSpec::AnyNumbered),
                        "concerns → AnyNumbered"
                    ),
                    None => panic!("a references row must carry a role"),
                },
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
        if let TargetSpec::Kinds(ks) = lookup(&SLICE_KIND, RelationLabel::GovernedBy, None)
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
        if let TargetSpec::Kinds(ks) = lookup(&ASSUMPTION_KIND, RelationLabel::Shapes, None)
            .unwrap()
            .target
        {
            let mut got: Vec<&str> = ks.iter().copied().collect();
            got.sort_unstable();
            assert_eq!(
                got,
                [
                    "ADR", "ASM", "CHR", "CON", "DEC", "IDE", "IMP", "ISS", "POL", "PRD", "QUE",
                    "REQ", "RFC", "RSK", "SL", "SPEC", "STD"
                ]
            );
        } else {
            panic!(
                "shapes → Kinds([PRD, SPEC, REQ, SLICE, ISS, IMP, CHR, RSK, IDE, ADR, POL, STD, ASM, DEC, QUE, CON])"
            );
        }
        // spawns target is the 5 backlog-item kinds.
        if let TargetSpec::Kinds(ks) = lookup(&ASSUMPTION_KIND, RelationLabel::Spawns, None)
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
        let r = lookup(&ASSUMPTION_KIND, RelationLabel::Supersedes, None)
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
    /// readers had for free. A slice row carrying `related` (SL-095) and a backlog row
    /// carrying `governed_by`/`related` (SL-145) are legal and emit edges; a backlog row
    /// carrying `requirements` (SL-only) ⇒ `IllegalRow` (IllegalForSource), NEVER a live
    /// edge. An unknown label spelling ⇒ `IllegalRow` (UnknownLabel).
    #[test]
    fn read_block_rejects_illegal_source_label_pairs() {
        // A slice authoring `related` plus a legal `references(implements)`.
        let slice_doc = RelationDoc::parse(
            "[[relation]]\nlabel = \"related\"\ntarget = \"SL-002\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"PRD-010\"\n",
        )
        .unwrap();
        let (edges, illegal) = read_block(&SLICE_KIND, &slice_doc);
        assert_eq!(
            edge_pairs(&edges),
            vec![
                (RelationLabel::References, "PRD-010"),
                (RelationLabel::Related, "SL-002"),
            ],
            "the legal references and related rows emit edges"
        );
        assert!(illegal.is_empty(), "related is legal for a slice source");

        // A backlog item authoring `governed_by ADR-010` and `related IMP-005` (both legal
        // for a backlog source post-SL-145) plus a legal `slices`, and a
        // `references(implements)` row that is IllegalRole (implements is SL-only — a
        // backlog item may author references(concerns), not implements). Legal edges emit in
        // table-declaration order: governed_by, slices, related.
        let backlog_doc = RelationDoc::parse(
            "[[relation]]\nlabel = \"governed_by\"\ntarget = \"ADR-010\"\n\
             [[relation]]\nlabel = \"slices\"\ntarget = \"SL-020\"\n\
             [[relation]]\nlabel = \"related\"\ntarget = \"IMP-005\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"REQ-001\"\n",
        )
        .unwrap();
        let (edges, illegal) = read_block(&ISSUE_KIND, &backlog_doc);
        assert_eq!(
            edge_pairs(&edges),
            vec![
                (RelationLabel::GovernedBy, "ADR-010"),
                (RelationLabel::Slices, "SL-020"),
                (RelationLabel::Related, "IMP-005"),
            ],
            "governed_by, slices, and related all emit edges for a backlog source (SL-145)"
        );
        assert_eq!(
            illegal,
            vec![IllegalRow {
                label: "references".to_string(),
                target: "REQ-001".to_string(),
                reason: IllegalReason::IllegalRole,
            }],
            "references(implements) is illegal for a backlog source (implements is SL-only)"
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
    /// declaration order for the source kind; within one `(label, role)`, authored row
    /// order is preserved. The slice canonical run is references(implements) →
    /// supersedes; author them reversed with three references rows to prove the stable
    /// same-key order (SL-149 PHASE-05: the old specs/requirements collapsed into
    /// references(implements)).
    #[test]
    fn read_block_emits_in_canonical_order_stable_within_label() {
        let doc = RelationDoc::parse(
            // Authored order: supersedes, then three references(implements) (R-002, R-001, PRD-010).
            "[[relation]]\nlabel = \"supersedes\"\ntarget = \"SL-000\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"REQ-002\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"REQ-001\"\n\
             [[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"PRD-010\"\n",
        )
        .unwrap();
        let (edges, illegal) = read_block(&SLICE_KIND, &doc);
        assert!(illegal.is_empty(), "all rows are legal for a slice");
        assert_eq!(
            edge_pairs(&edges),
            vec![
                // Canonical RELATION_RULES order — references before supersedes …
                // … with the three references(implements) rows in AUTHORED order.
                (RelationLabel::References, "REQ-002"),
                (RelationLabel::References, "REQ-001"),
                (RelationLabel::References, "PRD-010"),
                (RelationLabel::Supersedes, "SL-000"),
            ],
            "edges land in canonical table order; same-(label,role) rows keep authored order"
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
            append_relation_row(text, RelationLabel::GovernedBy, None, "ADR-010").unwrap();
        assert_eq!(outcome, AppendOutcome::Wrote);
        assert!(next.contains("# a comment"), "comment preserved");
        assert!(next.contains("[[relation]]"));
        assert!(next.contains("label = \"governed_by\""));
        assert!(next.contains("target = \"ADR-010\""));
        // A label-only edge serialises with NO `role` key (SL-149 — diff stability).
        assert!(
            !next.contains("role ="),
            "label-only row carries no role key"
        );
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
        let (once, o1) =
            append_relation_row(text, RelationLabel::GovernedBy, None, "ADR-010").unwrap();
        assert_eq!(o1, AppendOutcome::Wrote);
        let (twice, o2) =
            append_relation_row(&once, RelationLabel::GovernedBy, None, "ADR-010").unwrap();
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
                    [[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"PRD-010\"\n\
                    [relationships]\ntags = [\"x\"]\n";
        let err =
            append_relation_row(trap, RelationLabel::GovernedBy, None, "ADR-010").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("relationships") && msg.contains("AFTER"),
            "refusal must name the offending trailing table: {msg}"
        );
        // But an ALREADY-present edge is a Noop even on the trap layout (guard-first):
        // re-linking the existing references row must not trip the structural refusal.
        let (out, outcome) = append_relation_row(
            trap,
            RelationLabel::References,
            Some(Role::Implements),
            "PRD-010",
        )
        .unwrap();
        assert_eq!(outcome, AppendOutcome::Noop);
        assert_eq!(out, trap);
    }

    /// `append_relation_row` escapes the target via `toml_edit::value` — a target
    /// carrying a quote cannot break out of the string literal (the migrator never
    /// authors such a target, but the write seam must be splice-safe regardless).
    #[test]
    fn append_relation_row_escapes_target() {
        let text = "id = 1\n";
        let (next, _) = append_relation_row(text, RelationLabel::Drift, None, "a\"b").unwrap();
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
            append_relation_row("id = 1\n", RelationLabel::GovernedBy, None, "ADR-010").unwrap();
        let (without, o1) =
            remove_relation_row(&with, RelationLabel::GovernedBy, None, "ADR-010").unwrap();
        assert_eq!(o1, RemoveOutcome::Removed);
        assert!(
            tier1_edges(&SLICE_KIND, &without).unwrap().is_empty(),
            "the edge is gone after remove"
        );
        let (again, o2) =
            remove_relation_row(&without, RelationLabel::GovernedBy, None, "ADR-010").unwrap();
        assert_eq!(o2, RemoveOutcome::Absent);
        assert_eq!(without, again, "a second remove is a byte-identical no-op");
    }

    // -- SL-149 PHASE-03: role storage round-trip + role-class IllegalRow -------

    /// VT-2 (SL-149 storage round-trip): authoring `references(implements)` writes a
    /// `[[relation]] label / role / target` row (role cell present, between label and
    /// target); `read_block` reads it back as a roled edge with identity `(label, role,
    /// target)`; `remove_relation_row` matches the FULL triple (a wrong role misses); and
    /// a label-only edge serialises with NO `role` key.
    #[test]
    fn references_role_round_trips_through_storage() {
        // Author references(implements) SPEC-018 onto a clean slice file.
        let (with, outcome) = append_relation_row(
            "id = 1\n",
            RelationLabel::References,
            Some(Role::Implements),
            "SPEC-018",
        )
        .unwrap();
        assert_eq!(outcome, AppendOutcome::Wrote);
        // The on-disk shape carries the role cell, ordered label / role / target.
        assert!(with.contains("label = \"references\""));
        assert!(with.contains("role = \"implements\""));
        assert!(with.contains("target = \"SPEC-018\""));
        let label_at = with.find("label =").unwrap();
        let role_at = with.find("role =").unwrap();
        let target_at = with.find("target =").unwrap();
        assert!(
            label_at < role_at && role_at < target_at,
            "the row reads label / role / target on disk: {with}"
        );

        // Reads back as a roled edge (identity = the triple).
        let edges = tier1_edges(&SLICE_KIND, &with).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].label, RelationLabel::References);
        assert_eq!(edges[0].role, Some(Role::Implements));
        assert_eq!(edges[0].target, "SPEC-018");

        // remove matches the FULL triple: a wrong role is a no-op; the right one removes.
        let (miss, o_miss) = remove_relation_row(
            &with,
            RelationLabel::References,
            Some(Role::Concerns),
            "SPEC-018",
        )
        .unwrap();
        assert_eq!(o_miss, RemoveOutcome::Absent, "a wrong role does not match");
        assert_eq!(miss, with, "a no-op remove is byte-identical");
        let (without, o_hit) = remove_relation_row(
            &with,
            RelationLabel::References,
            Some(Role::Implements),
            "SPEC-018",
        )
        .unwrap();
        assert_eq!(o_hit, RemoveOutcome::Removed);
        assert!(tier1_edges(&SLICE_KIND, &without).unwrap().is_empty());

        // A label-only edge (`governed_by`) serialises with NO role key.
        let (gb, _) =
            append_relation_row("id = 1\n", RelationLabel::GovernedBy, None, "ADR-010").unwrap();
        assert!(
            !gb.contains("role ="),
            "a label-only row carries no role key: {gb}"
        );
        let gb_edges = tier1_edges(&SLICE_KIND, &gb).unwrap();
        assert_eq!(
            gb_edges[0].role, None,
            "a label-only edge reads back role None"
        );
    }

    /// VT-2 (idempotency on the triple): the same `(label, role, target)` re-appends as a
    /// `Noop`, but the SAME `(label, target)` with a DIFFERENT role is a distinct edge —
    /// `references(implements) X` and `references(concerns) X` coexist.
    #[test]
    fn references_role_idempotency_keys_on_the_triple() {
        let (once, o1) = append_relation_row(
            "id = 1\n",
            RelationLabel::References,
            Some(Role::Concerns),
            "SL-002",
        )
        .unwrap();
        assert_eq!(o1, AppendOutcome::Wrote);
        // Same triple → Noop.
        let (again, o2) = append_relation_row(
            &once,
            RelationLabel::References,
            Some(Role::Concerns),
            "SL-002",
        )
        .unwrap();
        assert_eq!(o2, AppendOutcome::Noop);
        assert_eq!(once, again, "re-link of the same triple is byte-identical");
        // Different role, same (label, target) → a NEW row (distinct edge).
        let (two, o3) = append_relation_row(
            &once,
            RelationLabel::References,
            Some(Role::Implements),
            "SL-002",
        )
        .unwrap();
        assert_eq!(
            o3,
            AppendOutcome::Wrote,
            "a different role is a distinct edge"
        );
        let edges = tier1_edges(&SLICE_KIND, &two).unwrap();
        let roles: Vec<Option<Role>> = edges
            .iter()
            .filter(|e| e.label == RelationLabel::References && e.target == "SL-002")
            .map(|e| e.role)
            .collect();
        assert!(roles.contains(&Some(Role::Concerns)));
        assert!(roles.contains(&Some(Role::Implements)));
    }

    /// VT-3 (SL-149): a hand-edited `references` row with a MISSING role, an ILLEGAL role
    /// (illegal for the source), or an UNKNOWN role spelling, OR a label-only row carrying
    /// a STRAY role key, is an `IllegalRow` (role-class); a well-formed roled row and every
    /// label-only row stay legal (no false positive).
    #[test]
    fn read_block_flags_bad_references_role() {
        let bad = |kind: &Kind, toml: &str| -> Vec<IllegalReason> {
            let doc = RelationDoc::parse(toml).unwrap();
            let (_edges, illegal) = read_block(kind, &doc);
            illegal.into_iter().map(|r| r.reason).collect()
        };

        // Missing role on a references row → IllegalRole.
        assert_eq!(
            bad(
                &SLICE_KIND,
                "[[relation]]\nlabel = \"references\"\ntarget = \"SPEC-018\"\n"
            ),
            vec![IllegalReason::IllegalRole],
            "a references row with no role is an IllegalRow"
        );

        // Illegal-for-source role: a backlog item references with `implements` (SL-only).
        assert_eq!(
            bad(
                &ISSUE_KIND,
                "[[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"SPEC-018\"\n"
            ),
            vec![IllegalReason::IllegalRole],
            "implements is SL-only — illegal for a backlog source"
        );

        // Unknown role spelling → IllegalRole.
        assert_eq!(
            bad(
                &SLICE_KIND,
                "[[relation]]\nlabel = \"references\"\nrole = \"nonsense\"\ntarget = \"SPEC-018\"\n"
            ),
            vec![IllegalReason::IllegalRole],
            "an unparseable role spelling is an IllegalRow"
        );

        // Stray role on a label-only row → IllegalRole.
        assert_eq!(
            bad(
                &SLICE_KIND,
                "[[relation]]\nlabel = \"governed_by\"\nrole = \"concerns\"\ntarget = \"ADR-010\"\n"
            ),
            vec![IllegalReason::IllegalRole],
            "a role on a label-only row is an IllegalRow"
        );

        // No false positives: a well-formed roled row AND a label-only row are both legal.
        let (edges, illegal) = read_block(
            &SLICE_KIND,
            &RelationDoc::parse(
                "[[relation]]\nlabel = \"references\"\nrole = \"implements\"\ntarget = \"SPEC-018\"\n\
                 [[relation]]\nlabel = \"governed_by\"\ntarget = \"ADR-010\"\n",
            )
            .unwrap(),
        );
        assert!(illegal.is_empty(), "well-formed rows produce no findings");
        assert_eq!(edges.len(), 2);
        assert!(
            edges
                .iter()
                .any(|e| e.label == RelationLabel::References && e.role == Some(Role::Implements))
        );
        assert!(
            edges
                .iter()
                .any(|e| e.label == RelationLabel::GovernedBy && e.role.is_none())
        );
    }

    /// `inbound_name` is the table-driven derived-inbound render text (X5/R2-M3): the
    /// three inverted labels carry their pinned spelling; every other label renders its
    /// own `name()` so shipped inbound goldens are unchanged.
    #[test]
    fn inbound_name_is_table_driven() {
        assert_eq!(inbound_name(RelationLabel::GovernedBy, None), "governs");
        assert_eq!(inbound_name(RelationLabel::Consumes, None), "consumed_by");
        assert_eq!(
            inbound_name(RelationLabel::Supersedes, None),
            "superseded by"
        );
        assert_eq!(
            inbound_name(RelationLabel::OriginatesFrom, None),
            "precursor of"
        );
        // SL-149: references is role-keyed — each role pins its own inbound verb.
        assert_eq!(
            inbound_name(RelationLabel::References, Some(Role::Implements)),
            "implemented by"
        );
        assert_eq!(
            inbound_name(RelationLabel::References, Some(Role::ScopedFrom)),
            "scoped into"
        );
        assert_eq!(
            inbound_name(RelationLabel::References, Some(Role::Concerns)),
            "concerned by"
        );
        // Every non-inverted LABEL-ONLY label renders its own name() under role None.
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
                    // references has no role-None row; it is tested role-keyed above.
                    | RelationLabel::References
            );
            if !inverted {
                assert_eq!(
                    inbound_name(label, None),
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
        match validate_link(&SLICE_KIND, "governed_by", None) {
            Ok(rule) => assert_eq!(rule.label, RelationLabel::GovernedBy),
            Err(e) => panic!("governed_by should be writable for a slice: {e}"),
        }

        // `RelationRule` has no Debug, so `.unwrap_err()` (which Debug-formats Ok) won't
        // compile — extract the refusal message by hand.
        let refusal = |src: &Kind, label: &str| -> String {
            match validate_link(src, label, None) {
                Ok(_) => panic!("expected `{label}` to be refused for {}", src.prefix),
                Err(e) => e.to_string(),
            }
        };

        // Unknown label spelling — refused, message lists legal labels.
        let e = refusal(&SLICE_KIND, "nonsense");
        assert!(e.contains("governed_by"), "lists legal labels: {e}");

        // A slice CAN author `related` (SL-095) — returns the Related rule.
        match validate_link(&SLICE_KIND, "related", None) {
            Ok(rule) => assert_eq!(rule.label, RelationLabel::Related),
            Err(e) => panic!("related should be writable for a slice (SL-095): {e}"),
        }

        // SL-145: a backlog item CAN author `governed_by` and `related` (source widened).
        match validate_link(&ISSUE_KIND, "governed_by", None) {
            Ok(rule) => assert_eq!(rule.label, RelationLabel::GovernedBy),
            Err(e) => panic!("governed_by should be writable for a backlog item (SL-145): {e}"),
        }
        match validate_link(&ISSUE_KIND, "related", None) {
            Ok(rule) => assert_eq!(rule.label, RelationLabel::Related),
            Err(e) => panic!("related should be writable for a backlog item (SL-145): {e}"),
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
    /// SL-095 widened the `[SL, RFC]` `related` row to make a slice `related`
    /// resolve (SL-145 later added BACKLOG to the same row); verify `AnyNumbered`
    /// accepts any target kind.
    #[test]
    fn check_target_kind_enforces_target_kind() {
        // `RelationRule` has no Debug — unwrap the rule by hand.
        let unwrap_rule = |r: anyhow::Result<&'static RelationRule>| -> &'static RelationRule {
            match r {
                Ok(rule) => rule,
                Err(e) => panic!("expected a writable rule: {e}"),
            }
        };
        let gov_by = unwrap_rule(validate_link(&SLICE_KIND, "governed_by", None));
        // SL-003 (a slice) is NOT a legal governed_by target — refused.
        assert!(check_target_kind(gov_by, &SLICE_KIND, "SL").is_err());
        // ADR/POL/STD all pass.
        for p in ["ADR", "POL", "STD"] {
            assert!(check_target_kind(gov_by, &SLICE_KIND, p).is_ok());
        }

        // SameKind: gov `related` from an ADR accepts an ADR target, refuses a POL.
        let related = unwrap_rule(validate_link(&ADR_KIND.kind, "related", None));
        assert!(check_target_kind(related, &ADR_KIND.kind, "ADR").is_ok());
        assert!(
            check_target_kind(related, &ADR_KIND.kind, "POL").is_err(),
            "SameKind refuses a cross-gov target"
        );

        // SL-095: slice `related` targets AnyNumbered — any kind accepted.
        let sl_related = unwrap_rule(validate_link(&SLICE_KIND, "related", None));
        assert!(check_target_kind(sl_related, &SLICE_KIND, "ADR").is_ok());
        assert!(check_target_kind(sl_related, &SLICE_KIND, "SPEC").is_ok());
        assert!(check_target_kind(sl_related, &SLICE_KIND, "RV").is_ok());

        // SL-145: a backlog source widens `governed_by`/`related` but the TARGET gate is
        // unchanged. `governed_by` still enforces Kinds(GOV) — a slice target refused,
        // ADR/POL/STD pass; `related` stays AnyNumbered — any kind accepted.
        let bk_gov = unwrap_rule(validate_link(&ISSUE_KIND, "governed_by", None));
        assert!(
            check_target_kind(bk_gov, &ISSUE_KIND, "SL").is_err(),
            "backlog governed_by still refuses a non-GOV target"
        );
        for p in ["ADR", "POL", "STD"] {
            assert!(check_target_kind(bk_gov, &ISSUE_KIND, p).is_ok());
        }
        let bk_related = unwrap_rule(validate_link(&ISSUE_KIND, "related", None));
        assert!(check_target_kind(bk_related, &ISSUE_KIND, "SL").is_ok());
        assert!(check_target_kind(bk_related, &ISSUE_KIND, "ADR").is_ok());
    }

    /// SL-066 VT-2: the `revises` rule. Source REV, targets the six authored-truth
    /// kinds (off-target — e.g. `revises SL` — refused), `TypedVerbOnly` so generic
    /// `link` is refused (naming the typed verb). The rule row exists for target
    /// validation + inbound naming ("revises"), never as a writable Tier-1 edge.
    #[test]
    fn revises_rule_is_typed_verb_only_with_authored_truth_targets() {
        use crate::revision::REV_KIND;
        // The rule resolves for REV and carries the typed-verb policy.
        let rule = lookup(&REV_KIND, RelationLabel::Revises, None).expect("revises rule for REV");
        assert_eq!(rule.link, LinkPolicy::TypedVerbOnly);
        assert_eq!(rule.tier, Tier::Typed);
        assert_eq!(rule.inbound_name, "revises");

        // `doctrine link … revises …` is refused (TypedVerbOnly), naming the typed verb.
        match validate_link(&REV_KIND, "revises", None) {
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
        // SL-145: a backlog item CAN author `governed_by` (source widened); a slice can
        // author `related`.
        assert!(lookup(&ISSUE_KIND, RelationLabel::GovernedBy, None).is_some());
        let sl_related = lookup(&SLICE_KIND, RelationLabel::Related, None);
        assert!(sl_related.is_some());
        assert!(matches!(
            sl_related.unwrap().target,
            TargetSpec::AnyNumbered
        ));
        // governed_by is legal for SL, PRD, SPEC.
        for k in [&SLICE_KIND, &PRODUCT_SPEC_KIND, &TECH_SPEC_KIND] {
            assert!(lookup(k, RelationLabel::GovernedBy, None).is_some());
        }
        // consumes is legal for PRD only, not the tech spec.
        assert!(lookup(&PRODUCT_SPEC_KIND, RelationLabel::Consumes, None).is_some());
        assert!(lookup(&TECH_SPEC_KIND, RelationLabel::Consumes, None).is_none());
        // supersedes resolves to the SL→SL rule for a slice, the gov rule for ADR.
        let sl_sup = lookup(&SLICE_KIND, RelationLabel::Supersedes, None).unwrap();
        assert_eq!(sl_sup.link, LinkPolicy::Writable);
        let adr_sup = lookup(&ADR_KIND.kind, RelationLabel::Supersedes, None).unwrap();
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
        assert!(lookup(&ASSUMPTION_KIND, RelationLabel::Shapes, None).is_some());
        assert!(lookup(&SLICE_KIND, RelationLabel::Shapes, None).is_none());
        // Spawns is legal for record kinds, illegal for SL.
        assert!(lookup(&ASSUMPTION_KIND, RelationLabel::Spawns, None).is_some());
        assert!(lookup(&SLICE_KIND, RelationLabel::Spawns, None).is_none());
        let _: &Kind = &SLICE_KIND;
    }

    // -- SL-149 PHASE-02: Role vocabulary + (label, role)-keyed table ----------

    /// `Role` round-trips through `name()`/`from_name()`, and its `Ord` is declaration
    /// order (Implements < ScopedFrom < Concerns) — the canonical role order.
    #[test]
    fn role_name_round_trips_and_ord_is_declaration_order() {
        for role in [Role::Implements, Role::ScopedFrom, Role::Concerns] {
            assert_eq!(Role::from_name(role.name()), Some(role));
        }
        assert_eq!(Role::from_name("nonsense"), None);
        let mut roles = [Role::Concerns, Role::Implements, Role::ScopedFrom];
        roles.sort();
        assert_eq!(roles, [Role::Implements, Role::ScopedFrom, Role::Concerns]);
    }

    /// VT-1 lockstep (SL-149 PHASE-05): `References` is present in the enum/table while
    /// `Specs` and `Requirements` are GONE (the migration's hard cut). Their retired wire
    /// spellings no longer parse.
    #[test]
    fn references_replaces_specs_requirements() {
        let labels = distinct_labels_in_decl_order();
        assert!(
            labels.contains(&RelationLabel::References),
            "References must be present in the table"
        );
        // The retired spellings parse to nothing (the variants are gone).
        assert!(RelationLabel::from_name("specs").is_none());
        assert!(RelationLabel::from_name("requirements").is_none());
        // References precedes Supersedes in declaration order.
        let pos = |l: RelationLabel| labels.iter().position(|x| *x == l).unwrap();
        assert!(pos(RelationLabel::References) < pos(RelationLabel::Supersedes));
    }

    /// VT-4 invariant: exactly one rule per `(source, label, role)`. No source kind is
    /// admitted by two rows sharing the same `(label, role)` key (an ambiguous lookup).
    #[test]
    fn at_most_one_rule_per_source_label_role() {
        use std::collections::HashSet;
        // Key on the stable `name()` strings — RelationLabel/Role are Ord, not Hash
        // (determinism rides Ord; REQ-077). (label.name(), role.name()) faithfully
        // identifies (label, role).
        let mut seen: HashSet<(&str, &str, Option<&str>)> = HashSet::new();
        for r in RELATION_RULES {
            let role_key = r.role.map(Role::name);
            for src in r.sources {
                assert!(
                    seen.insert((src, r.label.name(), role_key)),
                    "duplicate rule for ({src}, {}, {role_key:?})",
                    r.label.name()
                );
            }
        }
    }

    /// VT-4 invariant: each `(source, label)` is WHOLLY roleful or WHOLLY roleless — no
    /// source authors a label via both a `Some(role)` row and a `None` row. References is
    /// roleful; every other label is roleless. Drives the `MissingRole`/`RoleNotApplicable`
    /// dichotomy.
    #[test]
    fn each_source_label_is_wholly_roleful_or_roleless() {
        use std::collections::HashMap;
        // (src, label.name()) -> (saw_some, saw_none). Keyed on name() — RelationLabel is
        // Ord, not Hash (determinism rides Ord; REQ-077).
        let mut seen: HashMap<(&str, &str), (bool, bool)> = HashMap::new();
        for r in RELATION_RULES {
            for src in r.sources {
                let e = seen.entry((src, r.label.name())).or_insert((false, false));
                if r.role.is_some() {
                    e.0 = true;
                } else {
                    e.1 = true;
                }
            }
        }
        for ((src, label), (some, none)) in seen {
            assert!(
                !(some && none),
                "({src}, {label}) mixes roleful and roleless rows"
            );
        }
        // And references is the roleful one; a label-only label (governed_by) is roleless.
        assert!(
            legal_roles(&SLICE_KIND, RelationLabel::References)
                .next()
                .is_some()
        );
        assert!(
            legal_roles(&SLICE_KIND, RelationLabel::GovernedBy)
                .next()
                .is_none()
        );
    }

    /// `legal_roles` reachability (SL-149 §2.6): the roles authorable for `(source,label)`,
    /// in canonical (declaration) order. SL references → all three; a backlog item →
    /// concerns only (implements/scoped_from are SL-only); a label-only label → none.
    #[test]
    fn legal_roles_reachability() {
        let sl: Vec<Role> = legal_roles(&SLICE_KIND, RelationLabel::References).collect();
        assert_eq!(
            sl,
            [Role::Implements, Role::ScopedFrom, Role::Concerns],
            "a slice can author all three references roles, in declaration order"
        );
        let iss: Vec<Role> = legal_roles(&ISSUE_KIND, RelationLabel::References).collect();
        assert_eq!(
            iss,
            [Role::Concerns],
            "a backlog item authors only concerns (implements/scoped_from are SL-only)"
        );
        // A label-only label yields no roles for any source.
        assert_eq!(
            legal_roles(&SLICE_KIND, RelationLabel::GovernedBy).count(),
            0
        );
    }

    /// `lookup` is role-keyed: a references row is reachable ONLY with the right
    /// `Some(role)`; label-only edges pass `None`; a wrong/absent role misses.
    #[test]
    fn lookup_is_role_keyed() {
        // references(implements) for SL resolves; the same with role None or a wrong
        // role misses.
        let impl_rule = lookup(
            &SLICE_KIND,
            RelationLabel::References,
            Some(Role::Implements),
        )
        .unwrap();
        assert!(matches!(impl_rule.target, TargetSpec::Kinds(_)));
        assert!(lookup(&SLICE_KIND, RelationLabel::References, None).is_none());
        // scoped_from/concerns are distinct rows.
        assert!(
            lookup(
                &SLICE_KIND,
                RelationLabel::References,
                Some(Role::ScopedFrom)
            )
            .is_some()
        );
        assert!(lookup(&SLICE_KIND, RelationLabel::References, Some(Role::Concerns)).is_some());
        // A backlog item cannot author implements/scoped_from (SL-only) but can concerns.
        assert!(
            lookup(
                &ISSUE_KIND,
                RelationLabel::References,
                Some(Role::Implements)
            )
            .is_none()
        );
        assert!(lookup(&ISSUE_KIND, RelationLabel::References, Some(Role::Concerns)).is_some());
        // A label-only label refuses a Some(role).
        assert!(lookup(&SLICE_KIND, RelationLabel::GovernedBy, Some(Role::Concerns)).is_none());
        assert!(lookup(&SLICE_KIND, RelationLabel::GovernedBy, None).is_some());
    }

    /// VT-5: `validate_link` role taxonomy — `MissingRole`, `IllegalRole`,
    /// `RoleNotApplicable`, and the role-keyed target gate. `RelationRule` has no Debug,
    /// so refusals are extracted by hand.
    #[test]
    fn validate_link_role_taxonomy() {
        let refusal = |src: &Kind, label: &str, role: Option<Role>| -> String {
            match validate_link(src, label, role) {
                Ok(_) => panic!(
                    "expected `{label}` (role {role:?}) refused for {}",
                    src.prefix
                ),
                Err(e) => e.to_string(),
            }
        };

        // MissingRole: references with no role names the legal roles.
        let e = refusal(&SLICE_KIND, "references", None);
        assert!(
            e.contains("requires a role") && e.contains("implements"),
            "MissingRole names the legal roles: {e}"
        );

        // IllegalRole: a slice references with scoped_from is legal, but a backlog item
        // references scoped_from is NOT (SL-only) — refused as an illegal role.
        let e = refusal(&ISSUE_KIND, "references", Some(Role::ScopedFrom));
        assert!(
            e.contains("not a legal role") && e.contains("concerns"),
            "IllegalRole lists the legal roles for the source: {e}"
        );

        // RoleNotApplicable: a role given for a label-only label.
        let e = refusal(&SLICE_KIND, "governed_by", Some(Role::Concerns));
        assert!(e.contains("does not take a role"), "RoleNotApplicable: {e}");

        // A legal references(implements) for a slice validates and returns the rule.
        match validate_link(&SLICE_KIND, "references", Some(Role::Implements)) {
            Ok(rule) => {
                assert_eq!(rule.label, RelationLabel::References);
                assert_eq!(rule.role, Some(Role::Implements));
            }
            Err(e) => panic!("references(implements) should validate for a slice: {e}"),
        }

        // Role-target mismatch is refused via the role-keyed TargetSpec: the
        // implements rule (→ SPEC·PRD·REQ) refuses a backlog target; concerns accepts it.
        let unwrap_rule = |r: anyhow::Result<&'static RelationRule>| -> &'static RelationRule {
            match r {
                Ok(rule) => rule,
                Err(e) => panic!("expected a writable rule: {e}"),
            }
        };
        let impl_rule = unwrap_rule(validate_link(
            &SLICE_KIND,
            "references",
            Some(Role::Implements),
        ));
        assert!(check_target_kind(impl_rule, &SLICE_KIND, "IMP").is_err());
        for p in ["SPEC", "PRD", "REQ"] {
            assert!(check_target_kind(impl_rule, &SLICE_KIND, p).is_ok());
        }
        let conc_rule = unwrap_rule(validate_link(
            &SLICE_KIND,
            "references",
            Some(Role::Concerns),
        ));
        assert!(check_target_kind(conc_rule, &SLICE_KIND, "IMP").is_ok());
        let scoped_rule = unwrap_rule(validate_link(
            &SLICE_KIND,
            "references",
            Some(Role::ScopedFrom),
        ));
        assert!(check_target_kind(scoped_rule, &SLICE_KIND, "IMP").is_ok());
        assert!(
            check_target_kind(scoped_rule, &SLICE_KIND, "SPEC").is_err(),
            "scoped_from refuses a non-backlog target"
        );
    }

    /// VT-9 (SL-158 D6): a record (ASM/DEC/QUE/CON) may author `references` with
    /// role `concerns` — `lookup` resolves, `read_block` legalizes, `legal_roles`
    /// lists `Concerns`. Target is `AnyNumbered` so any numbered entity is accepted.
    #[test]
    fn record_authors_references_concerns() {
        // lookup resolves for a record source.
        let rule = lookup(
            &ASSUMPTION_KIND,
            RelationLabel::References,
            Some(Role::Concerns),
        )
        .expect("ASM must be able to author references(concerns)");
        assert_eq!(rule.label, RelationLabel::References);
        assert_eq!(rule.role, Some(Role::Concerns));
        assert_eq!(rule.inbound_name, "concerned by");
        assert!(
            matches!(rule.target, TargetSpec::AnyNumbered),
            "concerns target is AnyNumbered"
        );
        assert_eq!(rule.tier, Tier::One);
        assert_eq!(rule.link, LinkPolicy::Writable);

        // legal_roles includes Concerns for a record source.
        let roles: Vec<Role> = legal_roles(&ASSUMPTION_KIND, RelationLabel::References).collect();
        assert!(
            roles.contains(&Role::Concerns),
            "legal_roles for ASM must contain Concerns"
        );

        // read_block legalizes a record's references(concerns) row.
        let doc = RelationDoc::parse(
            "[[relation]]\nlabel = \"references\"\nrole = \"concerns\"\ntarget = \"SL-001\"\n",
        )
        .unwrap();
        let (edges, illegal) = read_block(&ASSUMPTION_KIND, &doc);
        assert_eq!(
            edge_pairs(&edges),
            vec![(RelationLabel::References, "SL-001")],
            "record references(concerns) emits a legal edge"
        );
        assert!(illegal.is_empty(), "no illegal rows expected");

        // concerns is the only references role a record can author (implements/scoped_from
        // are SL-only).
        assert!(
            lookup(
                &ASSUMPTION_KIND,
                RelationLabel::References,
                Some(Role::Implements)
            )
            .is_none(),
            "records cannot author implements"
        );
        assert!(
            lookup(
                &ASSUMPTION_KIND,
                RelationLabel::References,
                Some(Role::ScopedFrom)
            )
            .is_none(),
            "records cannot author scoped_from"
        );
    }
}
