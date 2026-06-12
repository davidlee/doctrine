// SPDX-License-Identifier: GPL-3.0-only
//! The status-class partition (SL-047 design §5.3, OQ-8) — the pure policy data
//! that classifies a node's authored status into [`StatusClass::Workable`]
//! (eligible for default-active work) / [`StatusClass::Terminal`]
//! (default-excluded) / [`StatusClass::Unrecognised`] (the D12 conservative
//! default: non-eligible **and** a diagnostic).
//!
//! Pure: no clock, RNG, or disk — a `(kind, status) -> StatusClass` lookup over the
//! static [`PARTITION`] table. Kind identity is the `&'static str` prefix (the same
//! identity `EntityKey` carries); `entity::Kind` is data, not `Eq`, so the table
//! keys on `kind.prefix`.
//!
//! **Drift canary (VT-1).** Each partitioned closed-enum kind asserts
//! `workable ∪ terminal == <kind>'s status vocabulary`, reading the REAL authoritative
//! const (`*_STATUSES` in each kind module) — so the canary FAILS at test time if a
//! kind adds a status the table forgot. REC is status-less (no const-compare); slice
//! binds against the ADR-009/`SLICE_STATUSES` lifecycle vocabulary (its stringly status
//! has no closed enum), and a slice status outside the table rides `Unrecognised`.
//!
//! Consumed by the priority CLI surface (SL-047 PHASE-03 — `channels`/`surface` call
//! [`status_class`]), so the PHASE-02 self-clearing `not(test)` `dead_code`
//! suppression has retired itself, as designed (`mem.pattern.lint.
//! dead-code-expect-vs-cfg-test`).

use crate::entity;

/// How a node's authored status classifies for default-active work selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StatusClass {
    /// Eligible — the status is in the kind's workable set.
    Workable,
    /// Default-excluded — the status is in the kind's terminal set (or the
    /// status-less REC kind, whose every node is context-only).
    Terminal,
    /// Not in the kind's vocabulary — the D12 conservative default: non-eligible
    /// AND a diagnostic (the table forgot a status, or the status drifted).
    Unrecognised,
}

/// One kind's status partition: the workable (eligible) and terminal
/// (default-excluded) status sets, keyed by the kind's canonical-id `prefix`.
/// A status absent from BOTH sets is [`StatusClass::Unrecognised`].
struct KindPartition {
    /// The kind's canonical-id prefix (`SL`, `ADR`, `ISS`, …) — kind identity.
    prefix: &'static str,
    workable: &'static [&'static str],
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
        prefix: "SL",
        workable: &[
            "proposed",
            "design",
            "plan",
            "ready",
            "started",
            "audit",
            "reconcile",
        ],
        terminal: &["done", "abandoned"],
    },
    // ADR
    KindPartition {
        prefix: "ADR",
        workable: &["proposed"],
        terminal: &["accepted", "rejected", "superseded", "deprecated"],
    },
    // policy
    KindPartition {
        prefix: "POL",
        workable: &["draft"],
        terminal: &["required", "deprecated", "retired"],
    },
    // standard
    KindPartition {
        prefix: "STD",
        workable: &["draft"],
        terminal: &["default", "required", "deprecated", "retired"],
    },
    // PRD (product spec)
    KindPartition {
        prefix: "PRD",
        workable: &["draft"],
        terminal: &["active", "deprecated", "superseded"],
    },
    // tech spec
    KindPartition {
        prefix: "SPEC",
        workable: &["draft"],
        terminal: &["active", "deprecated", "superseded"],
    },
    // requirement
    KindPartition {
        prefix: "REQ",
        workable: &["pending", "in-progress"],
        terminal: &["active", "deprecated", "retired", "superseded"],
    },
    // backlog ×5 — one generic vocabulary; `promoted` resolution handled in channels.
    KindPartition {
        prefix: "ISS",
        workable: BACKLOG_WORKABLE,
        terminal: BACKLOG_TERMINAL,
    },
    KindPartition {
        prefix: "IMP",
        workable: BACKLOG_WORKABLE,
        terminal: BACKLOG_TERMINAL,
    },
    KindPartition {
        prefix: "CHR",
        workable: BACKLOG_WORKABLE,
        terminal: BACKLOG_TERMINAL,
    },
    KindPartition {
        prefix: "RSK",
        workable: BACKLOG_WORKABLE,
        terminal: BACKLOG_TERMINAL,
    },
    KindPartition {
        prefix: "IDE",
        workable: BACKLOG_WORKABLE,
        terminal: BACKLOG_TERMINAL,
    },
    // RV (review) — DERIVED active/done (NodeAttr already carries the derived string).
    KindPartition {
        prefix: "RV",
        workable: &["active"],
        terminal: &["done"],
    },
];

/// The shared backlog workable set (the five backlog prefixes partition identically).
const BACKLOG_WORKABLE: &[&str] = &["open", "triaged", "started"];
/// The shared backlog terminal set (a promoted resolution is a SEPARATE channel
/// reason, F1 — not a status class).
const BACKLOG_TERMINAL: &[&str] = &["resolved", "closed"];

/// Classify a node's `(kind, status)` into a [`StatusClass`] (design §5.3).
///
/// - `Some(s)` → table lookup on `kind.prefix`: `s ∈ workable → Workable`,
///   `s ∈ terminal → Terminal`, **else `Unrecognised`** (the D12 conservative
///   default — non-eligible plus a diagnostic).
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
    } else if part.terminal.contains(&status) {
        StatusClass::Terminal
    } else {
        StatusClass::Unrecognised
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{adr, backlog, policy, requirement, review, slice, spec, standard};
    use std::collections::BTreeSet;

    /// Look up a partition entry by prefix (test helper).
    fn part(prefix: &str) -> &'static KindPartition {
        PARTITION
            .iter()
            .find(|p| p.prefix == prefix)
            .expect("prefix in PARTITION")
    }

    /// `workable ∪ terminal` for a prefix, as a set.
    fn vocab(prefix: &str) -> BTreeSet<&'static str> {
        let p = part(prefix);
        p.workable
            .iter()
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

    // -- VT-3: conservative / status-less classification ----------------------

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
