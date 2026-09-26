// SPDX-License-Identifier: GPL-3.0-only
//! RV derived status (SL-268 D4, engine tier): the pure `derived_status`
//! summary over finding states (design §8, D-C8 / D7).

use super::schema::{ReviewDoc, parse_finding_status};
use super::vocab::{Await, FindingStatus, ReviewStatus};

// ---------------------------------------------------------------------------
// Derived status (design §8, D-C8 / D7)
// ---------------------------------------------------------------------------

/// The status carrier `derived_status` reads — the finding's current status is
/// all the summary needs. The full authored `Finding` (with severity/title/…)
/// lands in PHASE-02/03; this keeps the pure core dependency-free.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FindingState {
    pub(crate) status: FindingStatus,
}

/// The review's derived status + summarized turn (design §8, D-C8). Total over
/// the finding-status enum; never stored (computed at `show`/`list`/`status`).
///
/// - empty ⇒ `(Done, None)` — no findings, nothing to reconcile.
/// - any `open`/`contested` ⇒ `(Active, Responder)` — work awaits the responder.
/// - else any `answered` ⇒ `(Active, Raiser)` — work awaits the raiser.
/// - all `∈ {verified, withdrawn}` ⇒ `(Done, None)`.
///
/// `await` is a *priority summary* (open/contested wins display), never an
/// exclusive gate — the turn gate is per-finding `can` (D7).
pub(crate) fn derived_status(findings: &[FindingState]) -> (ReviewStatus, Await) {
    if findings.is_empty() {
        return (ReviewStatus::Done, Await::None);
    }
    if findings
        .iter()
        .any(|f| matches!(f.status, FindingStatus::Open | FindingStatus::Contested))
    {
        return (ReviewStatus::Active, Await::Responder);
    }
    if findings.iter().any(|f| f.status == FindingStatus::Answered) {
        return (ReviewStatus::Active, Await::Raiser);
    }
    (ReviewStatus::Done, Await::None)
}

/// Map an authored `FindingRow`'s status string to the pure [`FindingState`] for
/// `derived_status`/baton reconciliation (the conservative read of §8).
pub(crate) fn finding_states_of(doc: &ReviewDoc) -> Vec<FindingState> {
    doc.finding
        .iter()
        .map(|f| FindingState {
            status: parse_finding_status(&f.status),
        })
        .collect()
}
