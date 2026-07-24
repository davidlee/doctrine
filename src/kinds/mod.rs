//! The kind-identity vocabulary: canonical prefix per kind + the relation
//! source/target groupings. Leaf tier (ADR-001) — depends on nothing in-crate,
//! so the engine borrows identity without reaching up into command modules.
//! The prefix is the canonical kind identity (compared by `==` everywhere).
#![allow(
    dead_code,
    reason = "consumed by relation engine + command modules in PHASE-02/03"
)]

pub(crate) const SL: &str = "SL";
pub(crate) const PRD: &str = "PRD";
pub(crate) const SPEC: &str = "SPEC";
pub(crate) const CM: &str = "CM";
pub(crate) const REQ: &str = "REQ";
pub(crate) const ADR: &str = "ADR";
pub(crate) const POL: &str = "POL";
pub(crate) const STD: &str = "STD";
pub(crate) const RV: &str = "RV";
pub(crate) const REC: &str = "REC";
pub(crate) const REV: &str = "REV";
pub(crate) const RFC: &str = "RFC";
pub(crate) const ISS: &str = "ISS";
pub(crate) const IMP: &str = "IMP";
pub(crate) const CHR: &str = "CHR";
pub(crate) const RSK: &str = "RSK";
pub(crate) const IDE: &str = "IDE";
pub(crate) const ASM: &str = "ASM";
pub(crate) const DEC: &str = "DEC";
pub(crate) const QUE: &str = "QUE";
pub(crate) const CON: &str = "CON";
pub(crate) const EVD: &str = "EVD";
pub(crate) const HYP: &str = "HYP";
pub(crate) const CPT: &str = "CPT";

// --- dispatch ref-name prefixes (STD-001) ----------------------------------

/// Coordination ref prefix: `refs/heads/dispatch/<slice>`.
pub(crate) const DISPATCH_REF_PREFIX: &str = "refs/heads/dispatch/";

/// Per-phase evidence ref prefix: `refs/heads/phase/<slice>-NN`.
pub(crate) const PHASE_REF_PREFIX: &str = "refs/heads/phase/";

/// Review-bundle ref prefix: `refs/heads/review/<slice>`.
pub(crate) const REVIEW_REF_PREFIX: &str = "refs/heads/review/";

/// Candidate ref prefix: `refs/heads/candidate/<slice>/<label>`.
pub(crate) const CANDIDATE_REF_PREFIX: &str = "refs/heads/candidate/";

/// Every governance kind — `supersedes`/`related` source-set + `governed_by` targets.
pub(crate) const GOV: &[&str] = &[ADR, POL, STD];
/// Every backlog item kind — they share one `relation_edges` accessor.
pub(crate) const BACKLOG: &[&str] = &[ISS, IMP, CHR, RSK, IDE];
/// Every knowledge-record kind — the single source of truth for record-kind
/// membership. Sites that combine RECORD with other kinds should cite this
/// constant in a comment so a reader can trace back. The
/// `combined_constants_cover_record` test guards drift.
pub(crate) const RECORD: &[&str] = &[ASM, DEC, QUE, CON, EVD, HYP, CPT];

// --- Combined groups (RECORD-complete; keep in sync with RECORD) ------------

/// All kinds surfaced by default in `doctrine search`.
pub(crate) const SEARCH_DEFAULT: &[&str] = &[
    SL, ADR, PRD, SPEC, RFC, ISS, IMP, CHR, RSK, IDE, ASM, DEC, QUE, CON, EVD, HYP, CPT,
];
/// Every numbered kind (the census).
pub(crate) const ALL_KINDS: &[&str] = &[
    SL, ADR, POL, STD, PRD, SPEC, REQ, ISS, IMP, CHR, RSK, IDE, RV, REC, ASM, DEC, QUE, CON, EVD,
    HYP, CPT, CM, REV, RFC,
];
/// All taggable kinds.
pub(crate) const TAGGABLE: &[&str] = &[
    SL, ADR, POL, STD, RFC, ISS, IMP, CHR, RSK, IDE, ASM, CM, DEC, QUE, CON, EVD, HYP, CPT, PRD,
    SPEC, REQ, REC, REV, RV,
];
/// Work-like kinds — can author dep/seq edges.
pub(crate) const WORK_LIKE: &[&str] = &[SL, ISS, IMP, CHR, RSK, IDE, REV];
/// Work-like ∪ RECORD — admissible dep/seq targets (SL-158 D2).
pub(crate) const ADMISSIBLE_DEP_TARGETS: &[&str] = &[
    SL, ISS, IMP, CHR, RSK, IDE, REV, ASM, DEC, QUE, CON, EVD, HYP, CPT,
];

/// Value-bearing kinds (SL-089 D2): a slice plus the five backlog kinds — the set
/// that carries a value facet and feeds priority value/burndown. A STRICT SUBSET
/// of `dep_seq::is_work_like`: `value_bearing` ⊂ `work_like`, parted by REV (a
/// Revision is work-like for dep/seq but NOT value-bearing). Governance and
/// knowledge records are excluded.
pub(crate) const VALUE_BEARING: &[&str] = &[SL, ISS, IMP, CHR, RSK, IDE];

pub(crate) fn is_value_bearing(prefix: &str) -> bool {
    VALUE_BEARING.contains(&prefix)
}

/// Membership predicate over [`RECORD`] — the single source for "is this a
/// knowledge-record kind?" so adding/renaming a record kind edits RECORD,
/// not every call site.
pub(crate) fn is_record(prefix: &str) -> bool {
    RECORD.contains(&prefix)
}

// ---------------------------------------------------------------------------
// The `Kind` identity descriptor
// ---------------------------------------------------------------------------

/// A `Kind` is *data*, not a trait: one dispatch site, no per-kind state (D2).
/// Placement is no longer a `Kind` field — it is a runtime `MaterialiseRequest`,
/// because a named entity carries its uid only at call time (D8).
#[derive(serde::Serialize)]
pub(crate) struct Kind {
    /// Entity-tree root, relative to the project root, e.g. `.doctrine/slice`.
    /// Also the base every artifact path is joined to (H1).
    pub dir: &'static str,
    /// Canonical-id prefix, e.g. `SL` → `SL-003` (the `{{ref}}` token). Unused
    /// by named kinds, which have no numeric canonical id.
    pub prefix: &'static str,
    /// File stem for `<stem>-NNN.toml` / `<stem>-NNN.md` file names. Empty for
    /// sub-kinds that nest under a parent entity's numeric directory.
    #[serde(skip)]
    pub stem: &'static str,
}

// ---------------------------------------------------------------------------
// Entity-tree roots (STD-001 — one named `*_DIR` const per kind's `dir`, so the
// path literal has a single source of truth). Origin modules that still reach
// for a root re-export the leaf const; the rest are leaf-private.
// ---------------------------------------------------------------------------

pub(crate) const SLICE_DIR: &str = ".doctrine/slice";
pub(crate) const CONCEPT_MAP_DIR: &str = ".doctrine/concept-map";
pub(crate) const REV_DIR: &str = ".doctrine/revision";
pub(crate) const REC_DIR: &str = ".doctrine/rec";
pub(crate) const REVIEW_DIR: &str = ".doctrine/review";
pub(crate) const REQUIREMENT_DIR: &str = ".doctrine/requirement";
pub(crate) const RFC_DIR: &str = ".doctrine/rfc";
const ADR_DIR: &str = ".doctrine/adr";
const POLICY_DIR: &str = ".doctrine/policy";
const STANDARD_DIR: &str = ".doctrine/standard";
const PRODUCT_SPEC_DIR: &str = ".doctrine/spec/product";
const TECH_SPEC_DIR: &str = ".doctrine/spec/tech";
const ASSUMPTION_DIR: &str = ".doctrine/knowledge/assumption";
const DECISION_DIR: &str = ".doctrine/knowledge/decision";
const QUESTION_DIR: &str = ".doctrine/knowledge/question";
const CONSTRAINT_DIR: &str = ".doctrine/knowledge/constraint";
const EVIDENCE_DIR: &str = ".doctrine/knowledge/evidence";
const HYPOTHESIS_DIR: &str = ".doctrine/knowledge/hypothesis";
const CONCEPT_DIR: &str = ".doctrine/knowledge/concept";
const ISSUE_DIR: &str = ".doctrine/backlog/issue";
const IMPROVEMENT_DIR: &str = ".doctrine/backlog/improvement";
const CHORE_DIR: &str = ".doctrine/backlog/chore";
const RISK_DIR: &str = ".doctrine/backlog/risk";
const IDEA_DIR: &str = ".doctrine/backlog/idea";

// ---------------------------------------------------------------------------
// Per-kind identity descriptors — the leaf owns exactly one `Kind` value per
// kind; every origin module re-exports the value it needs, so call sites and
// the local `KINDS` table stay put (SL-204 PHASE-02).
// ---------------------------------------------------------------------------

/// The top-level reserved slice kind: toml + md + slug symlink.
pub(crate) const SLICE_KIND: Kind = Kind {
    dir: SLICE_DIR,
    prefix: SL,
    stem: "slice",
};

/// The top-level reserved concept-map kind: toml + md + slug symlink.
pub(crate) const CONCEPT_MAP_KIND: Kind = Kind {
    dir: CONCEPT_MAP_DIR,
    prefix: CM,
    stem: "concept-map",
};

/// The REV kind: `revision-NNN.toml` + `revision-NNN.md` + `NNN-slug` symlink.
pub(crate) const REV_KIND: Kind = Kind {
    dir: REV_DIR,
    prefix: REV,
    stem: "revision",
};

/// The REC kind: `rec-NNN.toml` + `rec-NNN.md` + `NNN-slug` symlink.
pub(crate) const REC_KIND: Kind = Kind {
    dir: REC_DIR,
    prefix: REC,
    stem: "rec",
};

/// The review kind: `review-NNN.toml` + `review-NNN.md` + `NNN-slug` symlink.
pub(crate) const REVIEW_KIND: Kind = Kind {
    dir: REVIEW_DIR,
    prefix: RV,
    stem: "review",
};

/// The top-level reserved requirement kind: `requirement-NNN.{toml,md}` + slug.
pub(crate) const REQUIREMENT_KIND: Kind = Kind {
    dir: REQUIREMENT_DIR,
    prefix: REQ,
    stem: "requirement",
};

/// The product-spec subtype: `members.toml`, no interactions.
pub(crate) const PRODUCT_SPEC_KIND: Kind = Kind {
    dir: PRODUCT_SPEC_DIR,
    prefix: PRD,
    stem: "spec",
};

/// The tech-spec subtype: identity + flat fields, `members`/`interactions`.
pub(crate) const TECH_SPEC_KIND: Kind = Kind {
    dir: TECH_SPEC_DIR,
    prefix: SPEC,
    stem: "spec",
};

/// The assumption kind: a working belief held until validated.
pub(crate) const ASSUMPTION_KIND: Kind = Kind {
    dir: ASSUMPTION_DIR,
    prefix: ASM,
    stem: "record",
};

/// The decision kind: a recorded choice and its rationale.
pub(crate) const DECISION_KIND: Kind = Kind {
    dir: DECISION_DIR,
    prefix: DEC,
    stem: "record",
};

/// The question kind: an open question whose answer shapes the work.
pub(crate) const QUESTION_KIND: Kind = Kind {
    dir: QUESTION_DIR,
    prefix: QUE,
    stem: "record",
};

/// The constraint kind: a standing limit on the solution space.
pub(crate) const CONSTRAINT_KIND: Kind = Kind {
    dir: CONSTRAINT_DIR,
    prefix: CON,
    stem: "record",
};

/// The evidence kind: an observed datum with provenance and confidence.
pub(crate) const EVIDENCE_KIND: Kind = Kind {
    dir: EVIDENCE_DIR,
    prefix: EVD,
    stem: "record",
};

/// The hypothesis kind: a testable proposition that predicts an outcome.
pub(crate) const HYPOTHESIS_KIND: Kind = Kind {
    dir: HYPOTHESIS_DIR,
    prefix: HYP,
    stem: "record",
};

/// The concept kind: a durable concept definition in the knowledge corpus.
pub(crate) const CONCEPT_KIND: Kind = Kind {
    dir: CONCEPT_DIR,
    prefix: CPT,
    stem: "record",
};

/// The issue kind: a defect / problem to fix.
pub(crate) const ISSUE_KIND: Kind = Kind {
    dir: ISSUE_DIR,
    prefix: ISS,
    stem: "backlog",
};

/// The improvement kind: an enhancement to existing behaviour.
pub(crate) const IMPROVEMENT_KIND: Kind = Kind {
    dir: IMPROVEMENT_DIR,
    prefix: IMP,
    stem: "backlog",
};

/// The chore kind: maintenance with no user-visible behaviour change.
pub(crate) const CHORE_KIND: Kind = Kind {
    dir: CHORE_DIR,
    prefix: CHR,
    stem: "backlog",
};

/// The risk kind: a tracked risk — the only kind carrying a `[facet]`.
pub(crate) const RISK_KIND: Kind = Kind {
    dir: RISK_DIR,
    prefix: RSK,
    stem: "backlog",
};

/// The idea kind: a speculative possibility, not yet committed work.
pub(crate) const IDEA_KIND: Kind = Kind {
    dir: IDEA_DIR,
    prefix: IDE,
    stem: "backlog",
};

/// The ADR governance kind's plain identity (the `GovKind` descriptor in
/// `crate::adr` borrows this via `&crate::kinds::ADR_KIND`).
pub(crate) const ADR_KIND: Kind = Kind {
    dir: ADR_DIR,
    prefix: ADR,
    stem: "adr",
};

/// The policy governance kind's plain identity (borrowed by `crate::policy`).
pub(crate) const POLICY_KIND: Kind = Kind {
    dir: POLICY_DIR,
    prefix: POL,
    stem: "policy",
};

/// The standard governance kind's plain identity (borrowed by `crate::standard`).
pub(crate) const STANDARD_KIND: Kind = Kind {
    dir: STANDARD_DIR,
    prefix: STD,
    stem: "standard",
};

/// The RFC governance kind's plain identity (borrowed by `crate::rfc`).
pub(crate) const RFC_KIND: Kind = Kind {
    dir: RFC_DIR,
    prefix: RFC,
    stem: "rfc",
};

// ---------------------------------------------------------------------------
// The id-scan table + resolvers (SL-204 PHASE-03, design D2). `KindRef` + `KINDS`
// reference the plain `Kind` statics above DIRECTLY (one identity, no copies), and
// the resolver probes live in `resolve`, re-exported so consumers have ONE retarget
// target (`crate::kinds::…`). All leaf.
// ---------------------------------------------------------------------------

mod resolve;
pub(crate) use resolve::*;

/// A numbered entity kind's identity for the id scan — a *referencing view* over
/// the engine [`Kind`] each kind-owning module already declares (its canonical
/// `prefix` and tree `dir`), plus the two facts the engine leaf does not carry:
/// `stem` names the metadata file (`slice-007.toml`), and `state_dir` is the
/// gitignored runtime phase-state tree (`.doctrine/state/slice`) a kind owns —
/// `Some` only for slice today — which `reseat` refuses to strand (F3).
pub(crate) struct KindRef {
    pub(crate) kind: &'static Kind,
    pub(crate) state_dir: Option<&'static str>,
}

/// When adding a numbered kind, add its `KindRef` row here and bump the count
/// in `kinds_table_covers_the_numbered_kinds`.
///
/// Every numbered kind, in canonical order. The one place this list lives; a new
/// numbered kind must be added here or it silently escapes `validate` (R-b — a
/// drift surface this table accepts in exchange for not threading a registry
/// through every kind-owning module).
pub(crate) const KINDS: &[KindRef] = &[
    KindRef {
        kind: &SLICE_KIND,
        state_dir: Some(".doctrine/state/slice"),
    },
    KindRef {
        kind: &ADR_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &POLICY_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &STANDARD_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &PRODUCT_SPEC_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &TECH_SPEC_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &REQUIREMENT_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &ISSUE_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &IMPROVEMENT_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &CHORE_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &RISK_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &IDEA_KIND,
        state_dir: None,
    },
    // Review (SL-040) — the 2nd kind with a runtime state tree (baton/lock/cache,
    // PHASE-03+), mirroring slice. Its authored toml is status-LESS (derived,
    // D-C8); the scan reads `.id` via the id-only reader (D2), so a status-less
    // ledger scans cleanly while the strict `Meta` stays untouched.
    KindRef {
        kind: &REVIEW_KIND,
        state_dir: Some(".doctrine/state/review"),
    },
    // REC (SL-042) — the reconciliation-record kind. Status-LESS like review
    // (D-Q3: one REC per act, no lifecycle), so the scan reads `.id` via the
    // id-only reader (meta::read_id). Owns no runtime state tree (state_dir None).
    KindRef {
        kind: &REC_KIND,
        state_dir: None,
    },
    // Knowledge records (SL-059) — seven numbered kinds over one engine, each its
    // own tree + reservation namespace. Status-ful (scanned via the standard
    // `meta::Meta` path), one shared `record-NNN.{toml,md}` stem, no runtime state
    // tree. Their `outbound_for` arm (`relation_graph.rs`, L7) co-lands so the
    // KINDS-driven dispatch stays total (a row with no arm panics every debug-build
    // graph scan).
    KindRef {
        kind: &ASSUMPTION_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &DECISION_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &QUESTION_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &CONSTRAINT_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &EVIDENCE_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &HYPOTHESIS_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &CONCEPT_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &CONCEPT_MAP_KIND,
        state_dir: None,
    },
    // Revision (SL-066, ADR-013) — the REV change-axis kind. Status-ful (scanned via
    // the standard id-only reader; its `revision-NNN.toml` carries `status`), stem
    // `revision`, no runtime state tree (state_dir None). Its THREE corpus-walk arms
    // (G1 `priority::partition` REV row, G2 `relation_graph::dep_seq_for` REV arm,
    // G3 `relation_graph::outbound_for` REV arm) co-land with this row, or the
    // debug-build corpus scan panics/mis-classifies the moment a REV is minted.
    KindRef {
        kind: &REV_KIND,
        state_dir: None,
    },
    // RFC (SL-122) — the governance-neutral discussion kind. Status-ful (scanned via
    // the standard meta::Meta path; its `rfc-NNN.toml` carries `status`), stem `rfc`,
    // no runtime state tree. Its `outbound_for` arm co-lands with this row, or the
    // debug-build corpus scan panics the moment an RFC is minted.
    KindRef {
        kind: &RFC_KIND,
        state_dir: None,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groupings_match_documented_membership() {
        assert_eq!(GOV, &[ADR, POL, STD]);
        assert_eq!(BACKLOG, &[ISS, IMP, CHR, RSK, IDE]);
        assert_eq!(RECORD, &[ASM, DEC, QUE, CON, EVD, HYP, CPT]);
    }

    /// IMP-184: every combined constant that claims to cover RECORD must
    /// actually include every record kind.
    #[test]
    fn combined_constants_cover_record() {
        for c in [SEARCH_DEFAULT, ALL_KINDS, TAGGABLE, ADMISSIBLE_DEP_TARGETS] {
            for &r in RECORD {
                assert!(c.contains(&r), "{r} missing from combined constant");
            }
        }
    }

    /// SL-177: VALUE_BEARING == SL + BACKLOG; every elem is work-like; REV is
    /// work-like but NOT value-bearing.
    #[test]
    fn value_bearing_is_sl_plus_backlog_strict_subset_of_work_like() {
        // VALUE_BEARING == [SL] + BACKLOG
        let expected: &[&str] = &[SL, ISS, IMP, CHR, RSK, IDE];
        assert_eq!(VALUE_BEARING, expected);
        // Every VALUE_BEARING elem is work-like.
        let work_like: &[&str] = &["SL", "ISS", "IMP", "CHR", "RSK", "IDE", "REV"];
        for &prefix in VALUE_BEARING {
            assert!(work_like.contains(&prefix), "{prefix} must be work-like");
        }
        // REV is work-like but NOT value-bearing.
        assert!(
            !VALUE_BEARING.contains(&"REV"),
            "REV is work-like for dep/seq but NOT value-bearing"
        );
        // Governance and records are neither.
        for &prefix in &[ADR, POL, STD, ASM, DEC, QUE, CON, EVD, HYP, CPT] {
            assert!(
                !is_value_bearing(prefix),
                "{prefix} must NOT be value-bearing"
            );
        }
    }

    #[test]
    fn kinds_table_covers_the_numbered_kinds() {
        assert_eq!(KINDS.len(), 24, "add/remove a KindRef row? bump this count");
        let prefixes: Vec<_> = KINDS.iter().map(|k| k.kind.prefix).collect();
        assert_eq!(
            prefixes,
            [
                "SL", "ADR", "POL", "STD", "PRD", "SPEC", "REQ", "ISS", "IMP", "CHR", "RSK", "IDE",
                "RV", "REC", "ASM", "DEC", "QUE", "CON", "EVD", "HYP", "CPT", "CM", "REV", "RFC"
            ]
        );
        // Slice and review (SL-040) own a runtime state tree (F3 guard surface).
        // REC (SL-042) is status-less but stateless — no runtime tree. The six
        // knowledge kinds (SL-059) are status-ful but stateless — no runtime tree.
        let stateful: Vec<_> = KINDS
            .iter()
            .filter(|k| k.state_dir.is_some())
            .map(|k| k.kind.prefix)
            .collect();
        assert_eq!(stateful, ["SL", "RV"]);
    }

    #[test]
    fn kinds_prefixes_are_corpus_wide_disjoint() {
        // NF-002 / F-A6: every numbered-kind prefix is distinct — the seven SL-059
        // additions (ASM/DEC/QUE/CON/EVD/HYP/CPT) collide with NO existing corpus prefix. A
        // duplicate prefix here would route two kinds to one namespace.
        use std::collections::BTreeSet;
        let prefixes: Vec<_> = KINDS.iter().map(|k| k.kind.prefix).collect();
        let distinct: BTreeSet<_> = prefixes.iter().copied().collect();
        assert_eq!(
            distinct.len(),
            prefixes.len(),
            "all KINDS prefixes are distinct: {prefixes:?}"
        );
    }
}
