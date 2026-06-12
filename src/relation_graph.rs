// SPDX-License-Identifier: GPL-3.0-only
//! The cross-kind relation graph engine (design §5.1/§5.2).
//!
//! Sits at the engine layer (ADR-001): it imports the relation vocabulary leaf
//! ([`crate::relation`]) and every edge-authoring kind module, dispatching a
//! data-driven [`outbound_for`] over `integrity::KINDS` — kind is *data*, not a
//! trait (`mem.pattern.entity.kind-is-data-not-trait`). No kind module imports
//! back, so there is no cycle (the whole reason the vocabulary lives in the leaf).
//!
//! PHASE-02 lands only the outbound extraction dispatch. The all-kind scan, the
//! `Projection<EntityKey>`, the reference overlays, and the `inspect` query extend
//! this same file in PHASE-03 (design §5.4).
//!
//! Self-clearing `not(test)` `dead_code` expect (the `dead-code-self-clearing-leaf`
//! precedent): `outbound_for` lands ahead of its PHASE-03 scan caller (which builds
//! the graph from it) and the PHASE-04 `inspect` command. Under `cfg(test)` the
//! dispatch VTs exercise it, so the expect scopes to `not(test)`; it retires itself
//! when PHASE-03 calls `outbound_for` from the scan.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-046 PHASE-02 outbound dispatch — built ahead of its PHASE-03 \
                  all-kind scan caller; live under cfg(test), retires itself when \
                  PHASE-03 wires the scan"
    )
)]

use std::path::Path;

use crate::entity;
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
        other => match crate::backlog::kind_from_prefix(other) {
            Some(item_kind) => crate::backlog::relation_edges(root, item_kind, id),
            // Unreachable for any `integrity::KINDS` row; a defensive empty for an
            // unknown prefix keeps the dispatch total without a panic.
            None => Ok(Vec::new()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrity::KINDS;
    use crate::relation::RelationLabel;
    use std::fs;
    use std::path::PathBuf;

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

    fn tmp() -> PathBuf {
        let base = std::env::temp_dir().join(format!("sl046_relgraph_{}", std::process::id()));
        let dir = base.join(format!(
            "{:?}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    // -- VT-1 outbound correctness per kind + outbound_for dispatch ----------

    #[test]
    fn slice_outbound_specs_requirements_supersedes() {
        let root = tmp();
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
        let root = tmp();
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
        let root = tmp();
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
        let root = tmp();
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
        let root = tmp();
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
        let root = tmp();
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
        let root = tmp();
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
        let root = tmp();
        // REQ is an edge target only; the dispatch returns empty without touching disk.
        let edges = outbound_for(&root, kind_for("REQ"), 1).unwrap();
        assert!(edges.is_empty());
    }

    // -- VT-2 exclusion proof (REC decision_ref carried, not dropped) --------

    #[test]
    fn rec_decision_ref_carried_as_free_text_not_dropped() {
        let root = tmp();
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
        let root = tmp();
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
}
