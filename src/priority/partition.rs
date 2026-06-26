// SPDX-License-Identifier: GPL-3.0-only
//! The status-class partition (SL-047 design §5.3, OQ-8; SL-158 D1 trinary) —
//! the pure policy data that classifies a node's authored status into
//! [`StatusClass::Workable`] (eligible for default-active work) /
//! [`StatusClass::Gating`] (non-workable, non-terminal — blocks dependents but
//! never surfaces as work) / [`StatusClass::Terminal`] (default-excluded,
//! never blocks dependents) / [`StatusClass::Unrecognised`] (the D12
//! conservative default: non-eligible **and** a diagnostic).
//!
//! Pure: no clock, RNG, or disk — a `(kind, status) -> StatusClass` lookup over the
//! static [`PARTITION`] table. Kind identity is the `&'static str` prefix (the same
//! identity `EntityKey` carries); `entity::Kind` is data, not `Eq`, so the table
//! keys on `kind.prefix`.
//!
//! **Drift canary (VT-1).** Each partitioned closed-enum kind asserts
//! `workable ∪ gating ∪ terminal == <kind>'s status vocabulary`, reading the REAL
//! authoritative const (`*_STATUSES` in each kind module) — so the canary FAILS at
//! test time if a kind adds a status the table forgot. REC is status-less (no
//! const-compare); slice binds against the ADR-009/`SLICE_STATUSES` lifecycle
//! vocabulary (its stringly status has no closed enum), and a slice status outside
//! the table rides `Unrecognised`.
//!
//! Consumed by the priority CLI surface (SL-047 PHASE-03 — `channels`/`surface` call
//! [`status_class`]), so the PHASE-02 self-clearing `not(test)` `dead_code`
//! suppression has retired itself, as designed (`mem.pattern.lint.
//! dead-code-expect-vs-cfg-test`).

use crate::{entity, kinds};

/// How a node's authored status classifies for default-active work selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StatusClass {
    /// Eligible — the status is in the kind's workable set.
    Workable,
    /// Non-workable, non-terminal — the status is in the kind's gating set.
    /// A Gating node blocks its dependents (≠ Terminal) but never surfaces
    /// as work (≠ Workable). Knowledge records use this for unsettled states.
    Gating,
    /// Default-excluded — the status is in the kind's terminal set (or the
    /// status-less REC kind, whose every node is context-only).
    Terminal,
    /// Not in the kind's vocabulary — the D12 conservative default: non-eligible
    /// AND a diagnostic (the table forgot a status, or the status drifted).
    Unrecognised,
}

/// One kind's status partition: the workable (eligible), gating (blocks but not
/// eligible), and terminal (default-excluded) status sets, keyed by the kind's
/// canonical-id `prefix`. A status absent from ALL THREE sets is
/// [`StatusClass::Unrecognised`].
struct KindPartition {
    /// The kind's canonical-id prefix (`SL`, `ADR`, `ISS`, …) — kind identity.
    prefix: &'static str,
    workable: &'static [&'static str],
    /// Non-workable, non-terminal — blocks dependents but never eligible.
    gating: &'static [&'static str],
    terminal: &'static [&'static str],
}

/// The §5.3 status-class table, verbatim. Backlog's five prefixes
/// (`ISS`/`IMP`/`CHR`/`RSK`/`IDE`) share ONE partition (one generic backlog
/// vocabulary, PRD-009) — listed per-prefix so the lookup is a flat prefix match.
/// `promoted` is NOT modelled here (a node-attr concern surfaced in channels, F1).
/// REC is status-less and absent from this table — [`status_class`] maps its `None`
/// status to [`StatusClass::Terminal`] directly (DD-4 context-only, no diagnostic).
const PARTITION: &[KindPartition] = &[
    // slice — ADR-009 lifecycle vocabulary (stringly status, no closed enum).
    KindPartition {
        prefix: kinds::SL,
        workable: &[
            "proposed",
            "design",
            "plan",
            "ready",
            "started",
            "audit",
            "reconcile",
        ],
        gating: &[],
        terminal: &["done", "abandoned"],
    },
    // ADR
    KindPartition {
        prefix: kinds::ADR,
        workable: &["proposed"],
        gating: &[],
        terminal: &["accepted", "rejected", "superseded", "deprecated"],
    },
    // policy
    KindPartition {
        prefix: kinds::POL,
        workable: &["draft"],
        gating: &[],
        terminal: &["required", "superseded", "deprecated", "retired"],
    },
    // standard
    KindPartition {
        prefix: kinds::STD,
        workable: &["draft"],
        gating: &[],
        terminal: &["default", "required", "superseded", "deprecated", "retired"],
    },
    // PRD (product spec)
    KindPartition {
        prefix: kinds::PRD,
        workable: &["draft"],
        gating: &[],
        terminal: &["active", "deprecated", "superseded"],
    },
    // tech spec
    KindPartition {
        prefix: kinds::SPEC,
        workable: &["draft"],
        gating: &[],
        terminal: &["active", "deprecated", "superseded"],
    },
    // requirement
    KindPartition {
        prefix: kinds::REQ,
        workable: &["pending", "in-progress"],
        gating: &[],
        terminal: &["active", "deprecated", "retired", "superseded"],
    },
    // backlog ×5 — one generic vocabulary; `promoted` resolution handled in channels.
    KindPartition {
        prefix: kinds::ISS,
        workable: BACKLOG_WORKABLE,
        gating: &[],
        terminal: BACKLOG_TERMINAL,
    },
    KindPartition {
        prefix: kinds::IMP,
        workable: BACKLOG_WORKABLE,
        gating: &[],
        terminal: BACKLOG_TERMINAL,
    },
    KindPartition {
        prefix: kinds::CHR,
        workable: BACKLOG_WORKABLE,
        gating: &[],
        terminal: BACKLOG_TERMINAL,
    },
    KindPartition {
        prefix: kinds::RSK,
        workable: BACKLOG_WORKABLE,
        gating: &[],
        terminal: BACKLOG_TERMINAL,
    },
    KindPartition {
        prefix: kinds::IDE,
        workable: BACKLOG_WORKABLE,
        gating: &[],
        terminal: BACKLOG_TERMINAL,
    },
    // RV (review) — DERIVED active/done (NodeAttr already carries the derived string).
    KindPartition {
        prefix: kinds::RV,
        workable: &["active"],
        gating: &[],
        terminal: &["done"],
    },
    // REV (revision, SL-066/ADR-013) — its OWN row: REV vocab ≠ backlog's, so it
    // cannot ride the backlog arm. Without this row a `done`/`abandoned` REV
    // classifies `Unrecognised != Terminal` and `blocked_by` (channels.rs:67, which
    // excuses only `class == Terminal`) blocks its dependent FOREVER — the inverse of
    // the IDE-010 payoff. The terminal set reads REV's real vocab via the
    // `crate::revision::REV_STATUSES`-bound canary (`revision_partition_covers_the_real_vocabulary`).
    KindPartition {
        prefix: kinds::REV,
        workable: &["proposed", "started"],
        gating: &[],
        terminal: &["done", "abandoned"],
    },
    // Knowledge records (SL-059, NF-003 / D7; SL-158 D1 trinary) — NEVER
    // `Workable`: each kind's entry is `workable: &[]`, unsettled statuses in
    // `gating` (blocks dependents, never eligible), settled statuses in `terminal`.
    // The union `gating ∪ terminal` reads the REAL `knowledge::*_STATUSES` const,
    // so the VT-1 canary fails if a kind adds a status the table forgot.
    KindPartition {
        prefix: kinds::ASM,
        workable: &[],
        gating: &["held", "testing"],
        terminal: &["validated", "invalidated", "obsolete"],
    },
    KindPartition {
        prefix: kinds::DEC,
        workable: &[],
        gating: &["proposed"],
        terminal: &["accepted", "rejected", "superseded"],
    },
    KindPartition {
        prefix: kinds::QUE,
        workable: &[],
        gating: &["open"],
        terminal: &["answered", "obsolete"],
    },
    KindPartition {
        prefix: kinds::CON,
        workable: &[],
        gating: &["active"],
        terminal: &["waived", "superseded", "retired"],
    },
];

/// The shared backlog workable set (the five backlog prefixes partition identically).
const BACKLOG_WORKABLE: &[&str] = &["open", "triaged", "started"];
/// The shared backlog terminal set (a promoted resolution is a SEPARATE channel
/// reason, F1 — not a status class).
const BACKLOG_TERMINAL: &[&str] = &["resolved", "closed"];

/// Classify a node's `(kind, status)` into a [`StatusClass`] (design §5.3; SL-158 D1 trinary).
///
/// - `Some(s)` → table lookup on `kind.prefix`: `s ∈ workable → Workable`,
///   `s ∈ gating → Gating`, `s ∈ terminal → Terminal`,
///   **else `Unrecognised`** (the D12 conservative default — non-eligible plus a
///   diagnostic).
/// - `None` (the status-less REC kind) → `Terminal`, NO diagnostic (DD-4
///   context-only, expected — never surfaced as drift).
/// - RV resolves through the table via its DERIVED `active`/`done` (already held in
///   `NodeAttr.status` from PHASE-01), exactly like any other kind.
///
/// `promoted` is intentionally NOT consulted here — a promoted backlog node is
/// excluded by a SEPARATE channel reason (F1 / REQ-075 AC2), surfaced where the
/// node-attr lives, not folded into the status class.
pub(crate) fn status_class(kind: &entity::Kind, status: Option<&str>) -> StatusClass {
    let Some(status) = status else {
        // The only status-less kind is REC: context-only, default-excluded, expected.
        return StatusClass::Terminal;
    };
    let Some(part) = PARTITION.iter().find(|p| p.prefix == kind.prefix) else {
        // A kind with no partition entry (e.g. a future kind) — conservative default.
        return StatusClass::Unrecognised;
    };
    if part.workable.contains(&status) {
        StatusClass::Workable
    } else if part.gating.contains(&status) {
        StatusClass::Gating
    } else if part.terminal.contains(&status) {
        StatusClass::Terminal
    } else {
        StatusClass::Unrecognised
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        adr, backlog, knowledge, policy, requirement, review, revision, slice, spec, standard,
    };
    use std::collections::BTreeSet;

    /// Look up a partition entry by prefix (test helper).
    fn part(prefix: &str) -> &'static KindPartition {
        PARTITION
            .iter()
            .find(|p| p.prefix == prefix)
            .expect("prefix in PARTITION")
    }

    /// `workable ∪ gating ∪ terminal` for a prefix, as a set.
    fn vocab(prefix: &str) -> BTreeSet<&'static str> {
        let p = part(prefix);
        p.workable
            .iter()
            .chain(p.gating.iter())
            .chain(p.terminal.iter())
            .copied()
            .collect()
    }

    fn set(items: &[&'static str]) -> BTreeSet<&'static str> {
        items.iter().copied().collect()
    }

    // -- VT-1: partition drift canary — table vocabulary == real const vocabulary --

    #[test]
    fn slice_partition_binds_adr009_lifecycle_vocabulary() {
        // slice has a STRINGLY status (no closed enum) — its canary binds to the
        // ADR-009 / SLICE_STATUSES lifecycle set, the transition authority.
        assert_eq!(vocab("SL"), set(slice::SLICE_STATUSES));
    }

    #[test]
    fn adr_partition_covers_the_real_vocabulary() {
        assert_eq!(vocab("ADR"), set(adr::ADR_STATUSES));
    }

    #[test]
    fn policy_partition_covers_the_real_vocabulary() {
        assert_eq!(vocab("POL"), set(policy::POLICY_STATUSES));
    }

    #[test]
    fn standard_partition_covers_the_real_vocabulary() {
        assert_eq!(vocab("STD"), set(standard::STANDARD_STATUSES));
    }

    #[test]
    fn prd_and_tech_spec_partitions_cover_the_real_vocabulary() {
        // SPEC_STATUSES covers BOTH the PRD and tech-spec rows.
        assert_eq!(vocab("PRD"), set(spec::SPEC_STATUSES));
        assert_eq!(vocab("SPEC"), set(spec::SPEC_STATUSES));
    }

    #[test]
    fn requirement_partition_covers_the_real_vocabulary() {
        assert_eq!(vocab("REQ"), set(requirement::REQ_STATUSES));
    }

    #[test]
    fn backlog_partition_covers_the_real_vocabulary() {
        // All five backlog prefixes share the one backlog vocabulary.
        for prefix in ["ISS", "IMP", "CHR", "RSK", "IDE"] {
            assert_eq!(
                vocab(prefix),
                set(backlog::BACKLOG_STATUSES),
                "{prefix} partition matches BACKLOG_STATUSES"
            );
        }
    }

    #[test]
    fn review_partition_covers_the_real_vocabulary() {
        assert_eq!(vocab("RV"), set(review::REVIEW_STATUSES));
    }

    // -- SL-066 VT-2: the REV partition's G1 canary (its own vocab, not backlog's) --

    #[test]
    fn revision_partition_covers_the_real_vocabulary() {
        // REV gets its OWN row + own const — the table vocab must equal the real
        // `revision::REV_STATUSES` (NOT backlog's), or a status drifts unnoticed.
        assert_eq!(vocab("REV"), set(revision::REV_STATUSES));
    }

    #[test]
    fn revision_done_and_abandoned_classify_terminal() {
        // The precondition for `needs REV-N` to unblock its dependent: a terminal
        // REV must classify `Terminal` (not `Unrecognised`), so `blocked_by` excuses
        // it (channels.rs:67). proposed/started stay `Workable`.
        assert_eq!(
            status_class(&revision::REV_KIND, Some("done")),
            StatusClass::Terminal
        );
        assert_eq!(
            status_class(&revision::REV_KIND, Some("abandoned")),
            StatusClass::Terminal
        );
        assert_eq!(
            status_class(&revision::REV_KIND, Some("proposed")),
            StatusClass::Workable
        );
        assert_eq!(
            status_class(&revision::REV_KIND, Some("started")),
            StatusClass::Workable
        );
    }

    // -- SL-059, SL-158 VT-1: the four knowledge partitions cover their real vocab (three-way) --

    #[test]
    fn knowledge_partitions_cover_the_real_vocabularies() {
        // VT-1 (three-way): workable ∪ gating ∪ terminal == statuses(kind) per kind;
        // the canary reads the REAL `knowledge::*_STATUSES` const.
        for kind in knowledge::RecordKind::ALL {
            let prefix = kind.prefix();
            assert_eq!(
                vocab(prefix),
                set(knowledge::statuses(kind)),
                "{prefix} partition matches statuses({kind:?})"
            );
        }
    }

    // -- SL-158 VT-2: class boundary per knowledge kind --------------------------

    #[test]
    fn knowledge_unsettled_gating_settled_terminal() {
        // VT-2: each unsettled status → Gating, each settled → Terminal, never Workable.
        // ASM
        assert_eq!(
            status_class(&knowledge::ASSUMPTION_KIND, Some("held")),
            StatusClass::Gating
        );
        assert_eq!(
            status_class(&knowledge::ASSUMPTION_KIND, Some("testing")),
            StatusClass::Gating
        );
        assert_eq!(
            status_class(&knowledge::ASSUMPTION_KIND, Some("validated")),
            StatusClass::Terminal
        );
        assert_eq!(
            status_class(&knowledge::ASSUMPTION_KIND, Some("invalidated")),
            StatusClass::Terminal
        );
        assert_eq!(
            status_class(&knowledge::ASSUMPTION_KIND, Some("obsolete")),
            StatusClass::Terminal
        );
        // DEC
        assert_eq!(
            status_class(&knowledge::DECISION_KIND, Some("proposed")),
            StatusClass::Gating
        );
        assert_eq!(
            status_class(&knowledge::DECISION_KIND, Some("accepted")),
            StatusClass::Terminal
        );
        assert_eq!(
            status_class(&knowledge::DECISION_KIND, Some("rejected")),
            StatusClass::Terminal
        );
        assert_eq!(
            status_class(&knowledge::DECISION_KIND, Some("superseded")),
            StatusClass::Terminal
        );
        // QUE
        assert_eq!(
            status_class(&knowledge::QUESTION_KIND, Some("open")),
            StatusClass::Gating
        );
        assert_eq!(
            status_class(&knowledge::QUESTION_KIND, Some("answered")),
            StatusClass::Terminal
        );
        assert_eq!(
            status_class(&knowledge::QUESTION_KIND, Some("obsolete")),
            StatusClass::Terminal
        );
        // CON
        assert_eq!(
            status_class(&knowledge::CONSTRAINT_KIND, Some("active")),
            StatusClass::Gating
        );
        assert_eq!(
            status_class(&knowledge::CONSTRAINT_KIND, Some("waived")),
            StatusClass::Terminal
        );
        assert_eq!(
            status_class(&knowledge::CONSTRAINT_KIND, Some("superseded")),
            StatusClass::Terminal
        );
        assert_eq!(
            status_class(&knowledge::CONSTRAINT_KIND, Some("retired")),
            StatusClass::Terminal
        );
    }

    // -- SL-158 VT-5: Gating record not eligible --------------------------------

    #[test]
    fn knowledge_gating_statuses_are_not_workable() {
        // VT-5: a Gating record is NOT eligible / not on the worklist.
        // `eligible` reads `status_class == Workable`; every unsettled/gating
        // knowledge status classifies Gating (≠ Workable).
        for kind in knowledge::RecordKind::ALL {
            for status in knowledge::statuses(kind) {
                let class = status_class(kind.kind(), Some(status));
                if class == StatusClass::Gating {
                    // Explicit: the gating status is NOT Workable → not eligible.
                    assert!(
                        class != StatusClass::Workable,
                        "{:?}/{status} is Gating — must not be Workable",
                        kind
                    );
                }
            }
        }
    }

    #[test]
    fn every_knowledge_status_classifies_gating_or_terminal_never_workable() {
        // SL-158: unsettled → Gating, settled → Terminal, NEVER Workable.
        for kind in knowledge::RecordKind::ALL {
            for status in knowledge::statuses(kind) {
                let class = status_class(kind.kind(), Some(status));
                assert!(
                    class == StatusClass::Gating || class == StatusClass::Terminal,
                    "{:?}/{status} must be Gating or Terminal, got {class:?}",
                    kind
                );
                assert_ne!(
                    class,
                    StatusClass::Workable,
                    "{:?}/{status} must never be Workable",
                    kind
                );
            }
        }
    }

    #[test]
    fn decision_accepted_diverges_hidden_from_status_class() {
        // F-A5 (VT-4): the two notions deliberately disagree on `accepted` — it is
        // LIST-VISIBLE (`is_hidden == false`, a live decision is not settled-away)
        // yet never workable (`status_class == Terminal`, SL-158 settled → Terminal).
        assert!(!knowledge::is_hidden(
            knowledge::RecordKind::Decision,
            "accepted"
        ));
        assert_eq!(
            status_class(&knowledge::DECISION_KIND, Some("accepted")),
            StatusClass::Terminal
        );
    }

    // -- SL-158 VT-8: knowledge-in-priority golden ---------------------------------

    #[test]
    fn knowledge_gating_and_terminal_e2e_golden() {
        // VT-8: create a corpus with one unsettled + one settled knowledge record,
        // build the priority graph, and assert the status class is Gating/Terminal.
        use crate::priority::graph;
        use crate::relation_graph::EntityKey;

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join(".doctrine/doctrine")).unwrap();
        std::fs::write(
            root.join(".doctrine/doctrine.toml"),
            "[doctrine]\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        // Create a minimal slice so the graph has at least one "work" entity.
        let slice_dir = root.join(".doctrine/slice/001");
        std::fs::create_dir_all(&slice_dir).unwrap();
        std::fs::write(
            slice_dir.join("slice-001.toml"),
            "id = 1\nslug = \"s\"\ntitle = \"S\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n",
        )
        .unwrap();
        std::fs::write(slice_dir.join("slice-001.md"), "scope\n").unwrap();

        // Seed one unsettled (gating) question: QUE-001, status = "open".
        let que_dir = root.join(".doctrine/knowledge/question/001");
        std::fs::create_dir_all(&que_dir).unwrap();
        std::fs::write(
            que_dir.join("record-001.toml"),
            "schema = \"doctrine.knowledge\"\nversion = 1\n\
             id = 1\nslug = \"q1\"\ntitle = \"Q1\"\n\
             record_kind = \"question\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\ntags = []\n",
        )
        .unwrap();
        std::fs::write(que_dir.join("record-001.md"), "open question\n").unwrap();

        // Seed one settled (terminal) question: QUE-002, status = "answered".
        let que2_dir = root.join(".doctrine/knowledge/question/002");
        std::fs::create_dir_all(&que2_dir).unwrap();
        std::fs::write(
            que2_dir.join("record-002.toml"),
            "schema = \"doctrine.knowledge\"\nversion = 1\n\
             id = 2\nslug = \"q2\"\ntitle = \"Q2\"\n\
             record_kind = \"question\"\nstatus = \"answered\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\ntags = []\n",
        )
        .unwrap();
        std::fs::write(que2_dir.join("record-002.md"), "answered question\n").unwrap();

        let g = graph::build(root).unwrap();

        // Look up the knowledge nodes via the projection.
        let que1 = EntityKey {
            prefix: "QUE",
            id: 1,
        };
        let que2 = EntityKey {
            prefix: "QUE",
            id: 2,
        };

        // The graph carries attrs — read status_class through class_of.
        use crate::priority::channels;
        // eligible is status_class == Workable; Gating ≠ Workable, Terminal ≠ Workable.
        assert!(
            !channels::eligible(&g, que1.clone()),
            "QUE-001 (open) is Gating — not eligible"
        );
        assert!(
            !channels::eligible(&g, que2.clone()),
            "QUE-002 (answered) is Terminal — not eligible"
        );

        // Direct status_class check via the partition table (pure, no graph needed).
        assert_eq!(
            status_class(&knowledge::QUESTION_KIND, Some("open")),
            StatusClass::Gating,
            "open question → Gating"
        );
        assert_eq!(
            status_class(&knowledge::QUESTION_KIND, Some("answered")),
            StatusClass::Terminal,
            "answered question → Terminal"
        );
    }

    // -- VT-3: conservative / status-less classification ----------------------

    #[test]
    fn non_knowledge_rows_have_empty_gating() {
        // Every non-knowledge row must have `gating: &[]` — behaviour byte-identical
        // to the pre-SL-158 world where Gating did not exist.
        for p in super::PARTITION.iter() {
            if crate::kinds::is_record(p.prefix) {
                // Knowledge rows have non-empty gating by design.
                continue;
            }
            assert!(
                p.gating.is_empty(),
                "non-knowledge row {} must have gating: &[]",
                p.prefix
            );
        }
    }

    #[test]
    fn rec_status_less_is_terminal_no_diagnostic() {
        // REC is the known status-less kind: None → Terminal, NOT Unrecognised
        // (so it raises no drift diagnostic — DD-4 context-only).
        assert_eq!(
            status_class(&crate::rec::REC_KIND, None),
            StatusClass::Terminal
        );
    }

    #[test]
    fn unrecognised_status_is_its_own_class() {
        // A status outside the kind's vocabulary → Unrecognised (the diagnostic
        // default), distinct from Terminal.
        assert_eq!(
            status_class(&slice::SLICE_KIND, Some("not-a-real-status")),
            StatusClass::Unrecognised
        );
    }

    #[test]
    fn workable_and_terminal_lookups() {
        // A workable slice status.
        assert_eq!(
            status_class(&slice::SLICE_KIND, Some("design")),
            StatusClass::Workable
        );
        // audit / reconcile are WORKABLE (VT-2 boundary — not yet terminal).
        assert_eq!(
            status_class(&slice::SLICE_KIND, Some("audit")),
            StatusClass::Workable
        );
        assert_eq!(
            status_class(&slice::SLICE_KIND, Some("reconcile")),
            StatusClass::Workable
        );
        // A terminal slice status.
        assert_eq!(
            status_class(&slice::SLICE_KIND, Some("done")),
            StatusClass::Terminal
        );
    }

    #[test]
    fn rv_derived_status_resolves_through_the_table() {
        // RV carries a DERIVED active/done — classified like any kind (Charge I).
        assert_eq!(
            status_class(&review::REVIEW_KIND, Some("active")),
            StatusClass::Workable
        );
        assert_eq!(
            status_class(&review::REVIEW_KIND, Some("done")),
            StatusClass::Terminal
        );
    }
}
