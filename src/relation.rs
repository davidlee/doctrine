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
//! Self-clearing `not(test)` `dead_code` expect (the `dead-code-self-clearing-leaf`
//! precedent): this vocabulary leaf lands ahead of its PHASE-03 graph consumer and
//! PHASE-04 render. Under `cfg(test)` the round-trip tests exercise every item, so
//! the expect scopes to `not(test)` where the gate's plain `cargo clippy` (bins/lib,
//! no test cfg) sees the items as genuinely dead. It retires itself once
//! `relation_graph` (PHASE-03) reads `.label`/`.target`/`name()`.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-046 PHASE-02 relation vocabulary leaf — built ahead of its \
                  PHASE-03 relation_graph scan + PHASE-04 inspect render consumers; \
                  every item is live under cfg(test) and the expect retires itself \
                  as those phases wire up"
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
            RelationLabel::Slices => "slices",
            RelationLabel::Related => "related",
            RelationLabel::Reviews => "reviews",
            RelationLabel::OwningSlice => "owning_slice",
            RelationLabel::Drift => "drift",
            RelationLabel::DecisionRef => "decision_ref",
        }
    }
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
}
