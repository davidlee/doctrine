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
pub(crate) const RECORD: &[&str] = &[ASM, DEC, QUE, CON, EVD, HYP];

// --- Combined groups (RECORD-complete; keep in sync with RECORD) ------------

/// All kinds surfaced by default in `doctrine search`.
pub(crate) const SEARCH_DEFAULT: &[&str] = &[
    SL, ADR, PRD, SPEC, RFC, ISS, IMP, CHR, RSK, IDE, ASM, DEC, QUE, CON, EVD, HYP,
];
/// Every numbered kind (the census).
pub(crate) const ALL_KINDS: &[&str] = &[
    SL, ADR, POL, STD, PRD, SPEC, REQ, ISS, IMP, CHR, RSK, IDE, RV, REC, ASM, DEC, QUE, CON, EVD,
    HYP, CM, REV, RFC,
];
/// All taggable kinds.
pub(crate) const TAGGABLE: &[&str] = &[
    SL, ADR, POL, STD, RFC, ISS, IMP, CHR, RSK, IDE, ASM, CM, DEC, QUE, CON, EVD, HYP, PRD, SPEC,
    REQ, REC, REV, RV,
];
/// Work-like kinds — can author dep/seq edges.
pub(crate) const WORK_LIKE: &[&str] = &[SL, ISS, IMP, CHR, RSK, IDE, REV];
/// Work-like ∪ RECORD — admissible dep/seq targets (SL-158 D2).
pub(crate) const ADMISSIBLE_DEP_TARGETS: &[&str] = &[
    SL, ISS, IMP, CHR, RSK, IDE, REV, ASM, DEC, QUE, CON, EVD, HYP,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groupings_match_documented_membership() {
        assert_eq!(GOV, &[ADR, POL, STD]);
        assert_eq!(BACKLOG, &[ISS, IMP, CHR, RSK, IDE]);
        assert_eq!(RECORD, &[ASM, DEC, QUE, CON, EVD, HYP]);
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
        for &prefix in &[ADR, POL, STD, ASM, DEC, QUE, CON, EVD, HYP] {
            assert!(
                !is_value_bearing(prefix),
                "{prefix} must NOT be value-bearing"
            );
        }
    }
}
