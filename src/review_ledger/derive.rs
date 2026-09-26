// SPDX-License-Identifier: GPL-3.0-only
//! RV derived status (SL-268 D4, engine tier): the pure `derived_status`
//! summary over finding states (design §8, D-C8 / D7), and the closed-vocabulary
//! defect disclosure every read surface renders (SL-268 D15, DEC-319).

use serde::Serialize;

use super::schema::ReviewDoc;
use super::vocab::{Await, FindingStatus, ReviewStatus, Severity, Vocab};

// ---------------------------------------------------------------------------
// Derived status (design §8, D-C8 / D7)
// ---------------------------------------------------------------------------

/// The status carrier `derived_status` reads — the finding's current status is
/// all the summary needs, read fail-safe: an out-of-vocabulary status stays
/// [`Vocab::Unknown`] with its raw string rather than borrowing a known one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FindingState {
    pub(crate) status: Vocab<FindingStatus>,
}

/// The review's derived status + summarized turn (design §8, D-C8). Total over
/// the finding-status enum; never stored (computed at `show`/`list`/`status`).
///
/// - empty ⇒ `(Done, None)` — no findings, nothing to reconcile.
/// - any `open`/`contested`/out-of-vocabulary ⇒ `(Active, Responder)` — work
///   awaits the responder. An unknown status is **non-terminal** (SL-268 D15):
///   it can never close a review by accident.
/// - else any `answered` ⇒ `(Active, Raiser)` — work awaits the raiser.
/// - all `∈ {verified, withdrawn}` ⇒ `(Done, None)`.
///
/// `await` is a *priority summary* (open/contested wins display), never an
/// exclusive gate — the turn gate is per-finding `can` (D7).
pub(crate) fn derived_status(findings: &[FindingState]) -> (ReviewStatus, Await) {
    if findings.is_empty() {
        return (ReviewStatus::Done, Await::None);
    }
    if findings.iter().any(|f| {
        matches!(
            f.status,
            Vocab::Known(FindingStatus::Open | FindingStatus::Contested) | Vocab::Unknown(_)
        )
    }) {
        return (ReviewStatus::Active, Await::Responder);
    }
    if findings
        .iter()
        .any(|f| f.status == Vocab::Known(FindingStatus::Answered))
    {
        return (ReviewStatus::Active, Await::Raiser);
    }
    (ReviewStatus::Done, Await::None)
}

/// The pure [`FindingState`]s of an authored ledger for `derived_status`/baton
/// reconciliation — delegates to [`ReviewDoc::finding_states`], the one parse.
pub(crate) fn finding_states_of(doc: &ReviewDoc) -> Vec<FindingState> {
    doc.finding_states()
}

// ---------------------------------------------------------------------------
// Closed-vocabulary defects — the disclosure half of the fail-safe read
// (SL-268 D15, DEC-319)
// ---------------------------------------------------------------------------

/// What an out-of-vocabulary finding status is read as (STD-001).
pub(crate) const EFFECT_UNKNOWN_STATUS: &str = "reading as non-terminal";
/// What an out-of-vocabulary finding severity is read as (STD-001).
pub(crate) const EFFECT_UNKNOWN_SEVERITY: &str = "gating as blocker";
/// The lead of every rendered defect line, on every text channel.
pub(crate) const WARNING_PREFIX: &str = "warning: ";

/// The closed-vocabulary finding fields a read classifies. Only these two:
/// `disposition`, `response` and the free-text fields are carried verbatim and
/// are never defects (EX-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum VocabField {
    Status,
    Severity,
}

impl VocabField {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Severity => "severity",
        }
    }
}

/// One out-of-vocabulary value on one finding, with what the fail-safe read
/// made of it. Every surface renders through [`Self::describe`] /
/// [`Self::warning_line`], so no surface spells the disclosure itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VocabDefect {
    pub(crate) finding: String,
    pub(crate) field: VocabField,
    pub(crate) raw: String,
    pub(crate) effect: &'static str,
}

impl VocabDefect {
    /// ``{field} `{raw}` is out of vocabulary; {effect}``.
    pub(crate) fn describe(&self) -> String {
        format!(
            "{} `{}` is out of vocabulary; {}",
            self.field.as_str(),
            self.raw,
            self.effect
        )
    }

    /// ``warning: {rv} {finding} {describe}`` — the text-channel line, always
    /// naming the RV (DEC-319).
    pub(crate) fn warning_line(&self, rv: &str) -> String {
        format!("{WARNING_PREFIX}{rv} {} {}", self.finding, self.describe())
    }
}

/// Every closed-vocabulary defect in a ledger, in finding order, `status` before
/// `severity` within a finding. Pure; the read surfaces disclose what it finds.
pub(crate) fn vocabulary_defects(doc: &ReviewDoc) -> Vec<VocabDefect> {
    let mut defects = Vec::new();
    for f in &doc.finding {
        if let Vocab::Unknown(raw) = Vocab::<FindingStatus>::read(&f.status) {
            defects.push(VocabDefect {
                finding: f.id.clone(),
                field: VocabField::Status,
                raw,
                effect: EFFECT_UNKNOWN_STATUS,
            });
        }
        if let Vocab::Unknown(raw) = Vocab::<Severity>::read(&f.severity) {
            defects.push(VocabDefect {
                finding: f.id.clone(),
                field: VocabField::Severity,
                raw,
                effect: EFFECT_UNKNOWN_SEVERITY,
            });
        }
    }
    defects
}
