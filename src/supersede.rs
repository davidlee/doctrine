// SPDX-License-Identifier: GPL-3.0-only
//! Cross-kind supersession policy gate consumed by `doctrine supersede`.
//!
//! `supersede_policy()` answers "can this kind be the actor (NEW) in a
//! supersession?" and returns the vocabulary the verb needs: the outbound
//! `supersedes` array name, the sanctioned reverse carve-out `superseded_by` on
//! OLD (ADR-004 §5), and the terminal status OLD is flipped into.
//!
//! Extracted from `src/adr.rs` (SL-097 PHASE-02) — ADR is one arm among five;
//! record-kind arms (ASM/DEC/QUE/CON) join here. POL/STD/slice arms are
//! future work (IMP-063).

use crate::entity::Kind;
use crate::knowledge::RecordKind;

/// The supersession vocabulary for one kind — field names + terminal status.
#[derive(Copy, Clone)]
pub(crate) struct SupersedePolicy {
    /// NEW's outbound edge array — `[relationships].supersedes` (ADR-004 §5).
    pub(crate) supersedes_field: &'static str,
    /// OLD's reverse carve-out array — `[relationships].superseded_by`.
    pub(crate) carveout_field: &'static str,
    /// The terminal status OLD is flipped into (D2 per-kind table).
    pub(crate) superseded_status: &'static str,
}

/// The supersession capability boundary: returns `Some(policy)` for every kind
/// that can be the NEW actor in a `doctrine supersede` transaction.
///
/// ADR is the governance arm (SL-062); the four record kinds (ASM/DEC/QUE/CON)
/// are the knowledge arms (SL-097). Every other kind returns `None`, and the
/// verb refuses it with a "not yet supported" message.
pub(crate) fn supersede_policy(kind: &Kind) -> Option<SupersedePolicy> {
    match kind.prefix {
        "ADR" | "DEC" | "CON" => Some(SupersedePolicy {
            supersedes_field: "supersedes",
            carveout_field: "superseded_by",
            superseded_status: "superseded",
        }),
        "ASM" | "QUE" => Some(SupersedePolicy {
            supersedes_field: "supersedes",
            carveout_field: "superseded_by",
            superseded_status: "obsolete",
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
    use RecordKind::{Assumption, Constraint, Decision, Question};
    #[expect(clippy::unnested_or_patterns, reason = "clear matrix representation")]
    {
        matches!(
            (old, new),
            (Assumption, Assumption | Decision | Constraint)
                | (Question, Question | Decision | Constraint | Assumption)
                | (Decision, Decision | Constraint)
                | (Constraint, Constraint | Decision)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::RecordKind::*;

    #[test]
    fn validate_matrix_assumption_predecessor() {
        // Assumption (old) can be superseded by: Assumption, Decision, Constraint
        assert!(validate_matrix(Assumption, Assumption));
        assert!(validate_matrix(Decision, Assumption));
        assert!(validate_matrix(Constraint, Assumption));
        assert!(!validate_matrix(Question, Assumption));
    }

    #[test]
    fn validate_matrix_question_predecessor() {
        // Question (old) can be superseded by: Question, Decision, Constraint, Assumption
        assert!(validate_matrix(Question, Question));
        assert!(validate_matrix(Decision, Question));
        assert!(validate_matrix(Constraint, Question));
        assert!(validate_matrix(Assumption, Question));
    }

    #[test]
    fn validate_matrix_decision_predecessor() {
        // Decision (old) can be superseded by: Decision, Constraint
        assert!(validate_matrix(Decision, Decision));
        assert!(validate_matrix(Constraint, Decision));
        assert!(!validate_matrix(Assumption, Decision));
        assert!(!validate_matrix(Question, Decision));
    }

    #[test]
    fn validate_matrix_constraint_predecessor() {
        // Constraint (old) can be superseded by: Constraint, Decision
        assert!(validate_matrix(Constraint, Constraint));
        assert!(validate_matrix(Decision, Constraint));
        assert!(!validate_matrix(Assumption, Constraint));
        assert!(!validate_matrix(Question, Constraint));
    }
}
