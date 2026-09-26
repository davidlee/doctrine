// SPDX-License-Identifier: GPL-3.0-only
//! RV derived status (SL-268 D4, engine tier): the pure `derived_status`
//! summary over finding states (design §8, D-C8 / D7), and the closed-vocabulary
//! defect disclosure every read surface renders (SL-268 D15, DEC-319).

use serde::Serialize;

use super::schema::{ReviewDoc, TurnRow};
use super::transition::Act;
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

// ---------------------------------------------------------------------------
// Counters — base plus count (SL-268 sec-2, D1)
// ---------------------------------------------------------------------------

/// A review's observability counters: every turn taken, and how many were
/// contests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Counters {
    pub(crate) rounds: u32,
    pub(crate) contests: u32,
}

/// Every turn in the ledger, finding level and review level. Unknown acts are
/// turns too (sec-3): the count reads raw strings.
fn turns(doc: &ReviewDoc) -> impl Iterator<Item = &TurnRow> {
    doc.finding
        .iter()
        .flat_map(|f| f.turn.iter())
        .chain(doc.review.turn.iter())
}

/// The seed a first journalled write stores (sec-2): the legacy baton's
/// `(rounds, contests)`, when the ledger has neither a base nor any turn yet.
/// `None` once seeded, or once any turn exists — the seed is written once.
pub(crate) fn seed(doc: &ReviewDoc, baton: (u32, u32)) -> Option<(u32, u32)> {
    (doc.review.rounds_base.is_none() && turns(doc).next().is_none()).then_some(baton)
}

/// The counters a ledger reports (sec-2). A ledger never journalled (no base,
/// no turn) reports the legacy baton's values; otherwise
/// `rounds = rounds_base + turns` and `contests = contests_base + contest turns`.
pub(crate) fn counters(doc: &ReviewDoc, baton: (u32, u32)) -> Counters {
    if let Some((rounds, contests)) = seed(doc, baton) {
        return Counters { rounds, contests };
    }
    let count = |pred: fn(&TurnRow) -> bool| {
        u32::try_from(turns(doc).filter(|t| pred(t)).count()).unwrap_or(u32::MAX)
    };
    Counters {
        rounds: doc
            .review
            .rounds_base
            .unwrap_or(0)
            .saturating_add(count(|_| true)),
        contests: doc
            .review
            .contests_base
            .unwrap_or(0)
            .saturating_add(count(|t| t.act == Act::Contest.as_str())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal ledger with the given `[review]` extras and one finding whose
    /// journal is `finding_turns` (inline TOML, `[[finding.turn]]` bodies).
    fn ledger(review_extra: &str, finding_turns: &[&str]) -> ReviewDoc {
        let mut text = format!(
            "id = 1\nslug = \"s\"\ntitle = \"t\"\n[review]\nfacet = \"design\"\n\
             raiser = \"r\"\nresponder = \"s\"\n{review_extra}\n[target]\nref = \"SL-001\"\n\
             [[finding]]\nid = \"F-1\"\nstatus = \"open\"\nseverity = \"major\"\n\
             title = \"t\"\ndetail = \"d\"\n"
        );
        for turn in finding_turns {
            text.push_str("[[finding.turn]]\n");
            text.push_str(turn);
            text.push('\n');
        }
        toml::from_str(&text).unwrap()
    }

    const RAISE: &str = "act = \"raise\"\nrole = \"raiser\"";
    const CONTEST: &str = "act = \"contest\"\nrole = \"raiser\"\nnote = \"n\"";

    /// A never-journalled ledger needs a seed, and the seed is the baton's; once
    /// seeded at (5, 2), three turns (one a contest) read (8, 3).
    #[test]
    fn counters_seed_from_baton() {
        let fresh = ledger("", &[]);
        assert_eq!(seed(&fresh, (5, 2)), Some((5, 2)));
        let seeded = ledger(
            "rounds_base = 5\ncontests_base = 2",
            &[RAISE, CONTEST, "act = \"dispose\"\nrole = \"responder\""],
        );
        assert_eq!(seed(&seeded, (9, 9)), None, "seeded once");
        assert_eq!(
            counters(&seeded, (9, 9)),
            Counters {
                rounds: 8,
                contests: 3
            }
        );
    }

    /// No baton seeds a zero base: the count alone.
    #[test]
    fn counters_zero_base_without_baton() {
        let doc = ledger("rounds_base = 0\ncontests_base = 0", &[RAISE, CONTEST]);
        assert_eq!(
            counters(&doc, (0, 0)),
            Counters {
                rounds: 2,
                contests: 1
            }
        );
    }

    /// Review-level turns count, and so do unknown acts (sec-3); only `contest`
    /// counts as a contest.
    #[test]
    fn counters_base_plus_count() {
        let doc = ledger(
            "rounds_base = 3\ncontests_base = 1\n[[review.turn]]\nact = \"conclude\"\nrole = \"raiser\"",
            &[RAISE, "act = \"frobnicate\"\nrole = \"gremlin\"", CONTEST],
        );
        assert_eq!(
            counters(&doc, (0, 0)),
            Counters {
                rounds: 7,
                contests: 2
            }
        );
        // A turn anywhere, even with no base (a hand-edit), is journalled:
        // the base reads 0 and no seed is due.
        let unbased = ledger("", &[RAISE]);
        assert_eq!(seed(&unbased, (4, 4)), None);
        assert_eq!(
            counters(&unbased, (4, 4)),
            Counters {
                rounds: 1,
                contests: 0
            }
        );
    }

    /// No base and no turn: the legacy baton's values stand.
    #[test]
    fn counters_legacy_reads_baton() {
        assert_eq!(
            counters(&ledger("", &[]), (6, 2)),
            Counters {
                rounds: 6,
                contests: 2
            }
        );
    }
}
