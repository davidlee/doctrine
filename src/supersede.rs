// SPDX-License-Identifier: GPL-3.0-only
//! Cross-kind supersession policy gate consumed by `doctrine supersede`.
//!
//! `supersede_policy()` answers "can this kind be the actor (NEW) in a
//! supersession?" and returns the vocabulary the verb needs: the storage
//! write mechanism (`StorageTarget`), the sanctioned reverse carve-out
//! `superseded_by` on OLD (ADR-004 §5), and the terminal status OLD is
//! flipped into.
//!
//! Extracted from `src/adr.rs` (SL-097 PHASE-02). Governance (ADR/POL/STD)
//! arms added by SL-095 PHASE-03. Records (ASM/DEC/QUE/CON) arms are SL-097.

use crate::entity::Kind;
use crate::knowledge::RecordKind;

/// How the `doctrine supersede` verb writes the outbound edge for this kind.
#[derive(Copy, Clone)]
pub(crate) enum StorageTarget {
    /// Write via `relation::append_edge` — governance (ADR/POL/STD) post-SL-095.
    RelationRow,
    /// Write via `dep_seq::apply_string_append` — records (ASM/DEC/QUE/CON).
    TypedArray { field: &'static str },
}

/// The supersession vocabulary for one kind — storage mechanism + carved-in
/// field names + terminal status.
#[derive(Copy, Clone)]
pub(crate) struct SupersedePolicy {
    /// The verb's write mechanism for NEW's outbound edge.
    pub(crate) storage: StorageTarget,
    /// OLD's reverse carve-out array — `[relationships].superseded_by`.
    pub(crate) carveout_field: &'static str,
    /// The terminal status OLD is flipped into (D2 per-kind table).
    pub(crate) superseded_status: &'static str,
}

/// The supersession capability boundary: returns `Some(policy)` for every kind
/// that can be the NEW actor in a `doctrine supersede` transaction.
///
/// Governance arms (ADR/POL/STD) write `[[relation]]` rows via `RelationRow`.
/// Record arms (ASM/DEC/QUE/CON) write typed arrays via `TypedArray`.
/// Every other kind returns `None`.
pub(crate) fn supersede_policy(kind: &Kind) -> Option<SupersedePolicy> {
    match kind.prefix {
        "ADR" | "POL" | "STD" => Some(SupersedePolicy {
            storage: StorageTarget::RelationRow,
            carveout_field: "superseded_by",
            superseded_status: "superseded",
        }),
        "ASM" | "QUE" => Some(SupersedePolicy {
            storage: StorageTarget::TypedArray {
                field: "supersedes",
            },
            carveout_field: "superseded_by",
            superseded_status: "obsolete",
        }),
        "DEC" | "CON" | "EVD" => Some(SupersedePolicy {
            storage: StorageTarget::TypedArray {
                field: "supersedes",
            },
            carveout_field: "superseded_by",
            superseded_status: "superseded",
        }),
        _ => None,
    }
}

/// Cross-kind supersession matrix validator (§6 of SL-097 scope).
/// Returns `true` if `new` kind may supersede `old` kind according to the matrix:
///
/// - OLD assumption → assumption, decision, constraint
/// - OLD question → question, decision, constraint, assumption
/// - OLD decision → decision, constraint
/// - OLD constraint → constraint, decision
pub(crate) fn validate_matrix(new: RecordKind, old: RecordKind) -> bool {
    use RecordKind::{Assumption, Constraint, Decision, Evidence, Hypothesis, Question};
    #[expect(clippy::unnested_or_patterns, reason = "clear matrix representation")]
    {
        matches!(
            (old, new),
            (Assumption, Assumption | Decision | Constraint)
                | (Question, Question | Decision | Constraint | Assumption)
                | (Decision, Decision | Constraint)
                | (Constraint, Constraint | Decision)
                | (Evidence, Evidence | Decision | Constraint)
                | (Hypothesis, Hypothesis | Decision | Constraint | Assumption)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::RecordKind::*;

    // --- validate_matrix tests (SL-097) ---

    #[test]
    fn validate_matrix_assumption_predecessor() {
        assert!(validate_matrix(Assumption, Assumption));
        assert!(validate_matrix(Decision, Assumption));
        assert!(validate_matrix(Constraint, Assumption));
        assert!(!validate_matrix(Question, Assumption));
    }

    #[test]
    fn validate_matrix_question_predecessor() {
        assert!(validate_matrix(Question, Question));
        assert!(validate_matrix(Decision, Question));
        assert!(validate_matrix(Constraint, Question));
        assert!(validate_matrix(Assumption, Question));
    }

    #[test]
    fn validate_matrix_decision_predecessor() {
        assert!(validate_matrix(Decision, Decision));
        assert!(validate_matrix(Constraint, Decision));
        assert!(!validate_matrix(Assumption, Decision));
        assert!(!validate_matrix(Question, Decision));
    }

    #[test]
    fn validate_matrix_constraint_predecessor() {
        assert!(validate_matrix(Constraint, Constraint));
        assert!(validate_matrix(Decision, Constraint));
        assert!(!validate_matrix(Assumption, Constraint));
        assert!(!validate_matrix(Question, Constraint));
    }

    // --- supersede_policy tests (SL-095 PHASE-03) ---

    fn governance_kind(prefix: &str) -> &'static crate::entity::Kind {
        for kref in crate::integrity::KINDS {
            if kref.kind.prefix == prefix {
                return kref.kind;
            }
        }
        panic!("no kind with prefix {prefix}");
    }

    #[test]
    fn supersede_policy_returns_some_for_pol() {
        let kind = governance_kind("POL");
        let policy = supersede_policy(kind).unwrap();
        assert_eq!(policy.carveout_field, "superseded_by");
        assert_eq!(policy.superseded_status, "superseded");
        assert!(matches!(policy.storage, StorageTarget::RelationRow));
    }

    #[test]
    fn supersede_policy_returns_some_for_std() {
        let kind = governance_kind("STD");
        let policy = supersede_policy(kind).unwrap();
        assert_eq!(policy.carveout_field, "superseded_by");
        assert_eq!(policy.superseded_status, "superseded");
        assert!(matches!(policy.storage, StorageTarget::RelationRow));
    }

    #[test]
    fn supersede_policy_returns_some_for_adr() {
        let kind = governance_kind("ADR");
        let policy = supersede_policy(kind).unwrap();
        assert_eq!(policy.superseded_status, "superseded");
        assert!(matches!(policy.storage, StorageTarget::RelationRow));
    }

    #[test]
    fn supersede_policy_storage_is_relation_row_for_governance() {
        for prefix in ["ADR", "POL", "STD"] {
            let kind = governance_kind(prefix);
            let policy = supersede_policy(kind).unwrap();
            assert!(
                matches!(policy.storage, StorageTarget::RelationRow),
                "{prefix} should use RelationRow"
            );
        }
    }

    #[test]
    fn supersede_policy_storage_is_typed_array_for_records() {
        for (prefix, expected_status) in [
            ("ASM", "obsolete"),
            ("DEC", "superseded"),
            ("QUE", "obsolete"),
            ("CON", "superseded"),
            ("EVD", "superseded"),
        ] {
            let kind = governance_kind(prefix);
            let policy = supersede_policy(kind).unwrap();
            assert_eq!(policy.superseded_status, expected_status);
            assert!(
                matches!(
                    policy.storage,
                    StorageTarget::TypedArray {
                        field: "supersedes"
                    }
                ),
                "{prefix} should use TypedArray"
            );
        }
    }

    #[test]
    fn supersede_policy_returns_none_for_unsupported_kinds() {
        for prefix in ["SL", "IMP", "RV", "PRD", "REQ", "RSK"] {
            let kind = governance_kind(prefix);
            assert!(supersede_policy(kind).is_none(), "{prefix} should be None");
        }
    }
}
