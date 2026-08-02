// SPDX-License-Identifier: GPL-3.0-only
//! Forward boundaries, cumulative clearance, and direct regression.
//!
//! [`can_advance`] is a `const fn` `matches!` table over the single-owner edge
//! set, modelled on `src/review.rs::can` (EX-3). It is *modelled on*, not reused:
//! `review` is tier `command` and this module is `leaf`, so importing it would be
//! an upward edge. What is ridden is the idiom — one total, pure, const-evaluable
//! predicate that owns legality, with every other combination refused.
//!
//! Clearance is never stored. [`advance`] re-derives *every* cumulative condition
//! up to the target stage from current evidence, so DEC-067 falls out of the
//! shape rather than out of a flag: after a direct regression there is nothing to
//! un-set, and returning forward cannot inherit clearance it no longer earns.

use serde::{Deserialize, Serialize};

use super::Stage;
use super::bounds::DESIGN_ID_BYTES;
use super::facts::DerivedDesignFacts;
use super::refusal::Refusal;
use super::runbook::{RunbookKey, RunbookStanding};

/// A named precondition of a forward boundary (design §5.4).
///
/// Conditions are the closed vocabulary the gate table is written in; a fact is
/// satisfied when [`DerivedDesignFacts`] holds *live* evidence for it — evidence
/// whose subject fingerprint still matches current content (DEC-066).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Condition {
    /// exploring → inquiring: governing context recorded.
    GoverningContextRecorded,
    /// exploring → inquiring: initial concerns recorded.
    InitialConcernsRecorded,
    /// inquiring → drafting: blocking inquiries dispositioned.
    BlockingInquiriesDispositioned,
    /// inquiring → drafting: the user accepts sufficiency.
    UserAcceptsSufficiency,
    /// drafting → reviewing: required sections exist.
    RequiredSectionsExist,
    /// drafting → reviewing: materialisation is current.
    MaterialisationCurrent,
    /// reviewing → locked: section attestations are current.
    SectionAttestationsCurrent,
    /// reviewing → locked: an integrated review exists.
    IntegratedReviewPresent,
    /// reviewing → locked: blocking findings are disposed.
    BlockingFindingsDisposed,
    /// reviewing → locked: user acceptance is attested (an auditable agent
    /// claim in v1, not authenticated proof of a human act).
    UserAcceptanceAttested,
}

impl Condition {
    /// Every condition — the closed vocabulary, single-sourced so the
    /// containment check can enumerate gate ids rather than hand-pick one
    /// (STD-001).
    pub(crate) const ALL: [Condition; 10] = [
        Condition::GoverningContextRecorded,
        Condition::InitialConcernsRecorded,
        Condition::BlockingInquiriesDispositioned,
        Condition::UserAcceptsSufficiency,
        Condition::RequiredSectionsExist,
        Condition::MaterialisationCurrent,
        Condition::SectionAttestationsCurrent,
        Condition::IntegratedReviewPresent,
        Condition::BlockingFindingsDisposed,
        Condition::UserAcceptanceAttested,
    ];

    /// Whether this condition is **derived from the run's own review state**
    /// rather than claimed by a caller (design §5.4, the `reviewing → locked`
    /// row).
    ///
    /// The four members of that boundary are answers Doctrine can read off the
    /// run: which sections carry a live attestation, whether an integrated pass
    /// covers current content, whether a blocking finding is outstanding, whether
    /// the user's acceptance still covers what is being locked. Letting a payload
    /// *claim* them would make the review machinery decorative — a caller could
    /// lock a design it had never reviewed by asserting that it had, which is
    /// exactly the self-attestation DEC-088 refuses for accepted truth.
    ///
    /// The other six remain claimed, and that asymmetry is deliberate rather than
    /// finished: `materialisation-current` in particular is mechanically
    /// derivable, but this boundary is what the phase owns. An exhaustive match,
    /// so a new condition cannot join the vocabulary without answering here.
    pub(crate) const fn is_derived(self) -> bool {
        match self {
            Condition::SectionAttestationsCurrent
            | Condition::IntegratedReviewPresent
            | Condition::BlockingFindingsDisposed
            | Condition::UserAcceptanceAttested => true,
            Condition::GoverningContextRecorded
            | Condition::InitialConcernsRecorded
            | Condition::BlockingInquiriesDispositioned
            | Condition::UserAcceptsSufficiency
            | Condition::RequiredSectionsExist
            | Condition::MaterialisationCurrent => false,
        }
    }

    /// The kebab token this condition is spelled with everywhere — the snapshot
    /// value, the refusal text, the gate id on a rendered change row (STD-001).
    ///
    /// The longest member, `blocking-inquiries-dispositioned`, is exactly
    /// [`super::bounds::DESIGN_ID_BYTES`] bytes: a gate id rides the id slot of a
    /// change payload, and that is the derivation of the 32-byte bound.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Condition::GoverningContextRecorded => "governing-context-recorded",
            Condition::InitialConcernsRecorded => "initial-concerns-recorded",
            Condition::BlockingInquiriesDispositioned => "blocking-inquiries-dispositioned",
            Condition::UserAcceptsSufficiency => "user-accepts-sufficiency",
            Condition::RequiredSectionsExist => "required-sections-exist",
            Condition::MaterialisationCurrent => "materialisation-current",
            Condition::SectionAttestationsCurrent => "section-attestations-current",
            Condition::IntegratedReviewPresent => "integrated-review-present",
            Condition::BlockingFindingsDisposed => "blocking-findings-disposed",
            Condition::UserAcceptanceAttested => "user-acceptance-attested",
        }
    }
}

/// The widest gate id, at compile time.
const fn widest_condition(rest: &[Condition]) -> usize {
    match rest {
        [] => 0,
        [head, tail @ ..] => {
            let head = head.as_str().len();
            let tail = widest_condition(tail);
            if head > tail { head } else { tail }
        }
    }
}

/// The provenance of [`DESIGN_ID_BYTES`], **proved rather than asserted**
/// (EX-16(a)). A gate id rides the id slot of a change payload, and
/// `blocking-inquiries-dispositioned` is the widest identifier that slot must
/// carry — at exactly 32 B. That is where the number comes from, and a condition
/// name that outgrew it would stop the build rather than silently break the
/// rendered-row arithmetic.
const _: () = assert!(widest_condition(&Condition::ALL) <= DESIGN_ID_BYTES);

/// Whether `from → to` is an edge of the forward boundary table.
///
/// Pure, total, and `const`: the four adjacent forward moves of design §5.4, and
/// every other combination — self-moves, skips, and every backward move —
/// refused. A backward move is not illegal, it is a *different verb*
/// ([`regress`]).
pub(crate) const fn can_advance(from: Stage, to: Stage) -> bool {
    matches!(
        (from, to),
        (Stage::Exploring, Stage::Inquiring)
            | (Stage::Inquiring, Stage::Drafting)
            | (Stage::Drafting, Stage::Reviewing)
            | (Stage::Reviewing, Stage::Locked)
    )
}

/// The conditions a single forward boundary requires (design §5.4). Empty for a
/// pair that is not an edge — [`can_advance`] is what refuses those.
pub(crate) const fn boundary_conditions(from: Stage, to: Stage) -> &'static [Condition] {
    match (from, to) {
        (Stage::Exploring, Stage::Inquiring) => &[
            Condition::GoverningContextRecorded,
            Condition::InitialConcernsRecorded,
        ],
        (Stage::Inquiring, Stage::Drafting) => &[
            Condition::BlockingInquiriesDispositioned,
            Condition::UserAcceptsSufficiency,
        ],
        (Stage::Drafting, Stage::Reviewing) => &[
            Condition::RequiredSectionsExist,
            Condition::MaterialisationCurrent,
        ],
        (Stage::Reviewing, Stage::Locked) => &[
            Condition::SectionAttestationsCurrent,
            Condition::IntegratedReviewPresent,
            Condition::BlockingFindingsDisposed,
            Condition::UserAcceptanceAttested,
        ],
        _ => &[],
    }
}

/// The runbook a single forward boundary requires discharged, if it has one.
///
/// A third column on the table [`boundary_conditions`] already is, and the same
/// shape: static, `const`, total over the pair. Guards belong on **edges**, and
/// this one adds no states — the cursor it implies is run data a guard consults,
/// not a node in the machine, exactly as [`ReviewStanding`] is (sketch §2.1).
///
/// Edge-keying and origin-state-keying are isomorphic here, because
/// [`can_advance`] is total on non-terminal stages: each has exactly one
/// outbound forward edge. So the `exploring` runbook is named for where you
/// stand while discharging it, and selected by the edge it guards.
///
/// Every forward edge now carries one (SL-233 PHASE-08). The `_ => None` arm
/// covers the backward and non-adjacent pairs only: `Locked` is terminal, so it
/// has no outbound forward edge to key a runbook to, which is the same fact that
/// makes [`super::prompt::Fragment::for_stage`] yield `None` there.
pub(crate) const fn boundary_runbook(from: Stage, to: Stage) -> Option<RunbookKey> {
    match (from, to) {
        (Stage::Exploring, Stage::Inquiring) => Some(RunbookKey::Exploring),
        (Stage::Inquiring, Stage::Drafting) => Some(RunbookKey::Inquiring),
        (Stage::Drafting, Stage::Reviewing) => Some(RunbookKey::Drafting),
        (Stage::Reviewing, Stage::Locked) => Some(RunbookKey::Reviewing),
        _ => None,
    }
}

/// Every condition on the path from [`Stage::Exploring`] up to `to` — the
/// *cumulative* set DEC-067 requires to hold again against current content.
pub(crate) fn cumulative_conditions(to: Stage) -> Vec<Condition> {
    let mut out = Vec::new();
    for pair in Stage::ALL.windows(2) {
        let (Some(&from), Some(&next)) = (pair.first(), pair.get(1)) else {
            continue;
        };
        if from >= to {
            break;
        }
        out.extend_from_slice(boundary_conditions(from, next));
    }
    out
}

/// What the run's own review state says about the four derived conditions
/// (design §5.4).
///
/// Four independent booleans, not one "reviewed" flag: the lock predicate is a
/// **conjunction**, and each component fails for its own reason and is repaired by
/// its own act. Collapsing them would report one outstanding condition where four
/// are, and would let a caller who fixed the wrong thing keep guessing.
///
/// Derived, never stored — the same rule clearance follows (DEC-066/DEC-067). It
/// is recomputed from the snapshot on every evaluation, so returning to
/// `reviewing` after a regression cannot inherit review standing it no longer
/// earns.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ReviewStanding {
    /// Every section carries an attestation bound to its current content.
    pub(crate) sections_attested: bool,
    /// An integrated adversarial pass covers current content.
    pub(crate) integrated_current: bool,
    /// No blocking finding is outstanding.
    pub(crate) findings_disposed: bool,
    /// A user acceptance covers current content.
    pub(crate) acceptance_current: bool,
}

impl ReviewStanding {
    /// This standing's answer for `condition`, or `None` when the condition is
    /// not one this type owns.
    ///
    /// The `None` arm is defence, not a case: [`Condition::is_derived`] is the
    /// partition and [`satisfied`] consults it first. If the two ever disagreed,
    /// a derived condition with no answer here reads as **unsatisfied** — the
    /// gate stays shut rather than opening on a missing answer.
    const fn holds(self, condition: Condition) -> Option<bool> {
        match condition {
            Condition::SectionAttestationsCurrent => Some(self.sections_attested),
            Condition::IntegratedReviewPresent => Some(self.integrated_current),
            Condition::BlockingFindingsDisposed => Some(self.findings_disposed),
            Condition::UserAcceptanceAttested => Some(self.acceptance_current),
            _ => None,
        }
    }
}

/// Whether one condition holds — from the run's review state if it is derived,
/// from recorded evidence if it is claimed.
fn satisfied(condition: Condition, facts: &DerivedDesignFacts, standing: ReviewStanding) -> bool {
    if condition.is_derived() {
        standing.holds(condition) == Some(true)
    } else {
        facts.satisfies(condition)
    }
}

/// Attempt a forward move.
///
/// Two refusals, and they are different facts: the move is not an edge of the
/// table, or it is an edge whose cumulative conditions do not all hold against
/// current content. The second names every missing condition rather than the
/// first — an agent that fixes one and retries should not have to discover the
/// rest one round-trip at a time.
pub(crate) fn advance(
    from: Stage,
    to: Stage,
    facts: &DerivedDesignFacts,
    standing: ReviewStanding,
    runbook: Option<&RunbookStanding>,
) -> Result<Stage, Refusal> {
    if !can_advance(from, to) {
        return Err(Refusal::IllegalStageMove { from, to });
    }
    // The runbook before the conditions, and the order is deliberate: the
    // runbook is the edge's ENTRY RITUAL and the conditions are claims made
    // after the work. Refusing "you have not done the ritual" before "you have
    // not claimed the facts" is the sequencing the design intends — today's
    // inversion is precisely that a caller can claim before doing.
    //
    // Evaluated on THIS edge only. It deliberately does NOT join
    // [`cumulative_conditions`]: a prose edit to an explore step must not block
    // a boundary two stages later. DEC-067's property survives by another route
    // — regressing to `exploring` returns the run to this edge's origin, where
    // it faces the guard again. Cumulative re-derivation is right for the
    // existing conditions because they are global facts that rot independently
    // of where the run stands; a runbook is an entry ritual for one transition.
    // The first non-cumulative condition in the machine (`EX-16`).
    if boundary_runbook(from, to).is_some() {
        // Fail closed on a missing answer, the rule [`ReviewStanding::holds`]
        // already follows: no standing where the table says a runbook guards
        // this edge leaves the gate shut, not open.
        if runbook.is_none_or(|held| !held.cleared()) {
            return Err(Refusal::RunbookNotDischarged {
                from,
                to,
                outstanding: runbook.map_or_else(Vec::new, |held| held.outstanding.clone()),
                regressed: runbook.map_or_else(Vec::new, |held| held.regressed.clone()),
            });
        }
    }
    let missing: Vec<Condition> = cumulative_conditions(to)
        .into_iter()
        .filter(|condition| !satisfied(*condition, facts, standing))
        .collect();
    if missing.is_empty() {
        Ok(to)
    } else {
        Err(Refusal::GateNotCleared { from, to, missing })
    }
}

/// A recorded direct regression (DEC-067).
///
/// The reason is not optional and not defaultable: the type cannot be built
/// without one, so "regressed for no stated reason" is unrepresentable rather
/// than merely discouraged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Regression {
    from: Stage,
    to: Stage,
    reason: String,
}

impl Regression {
    /// The stage regressed from.
    pub(crate) const fn from(&self) -> Stage {
        self.from
    }

    /// The stage regressed to.
    pub(crate) const fn to(&self) -> Stage {
        self.to
    }

    /// The recorded reason.
    pub(crate) fn reason(&self) -> &str {
        &self.reason
    }
}

/// Attempt a direct regression to an earlier stage.
///
/// Refuses a move that is not backward, and refuses a backward move whose reason
/// is absent or blank. Nothing else is required: DEC-067 buys the *re-derivation*
/// of clearance on the way forward again ([`advance`] re-evaluates the whole
/// cumulative set), not a ceremonial replay on the way back.
pub(crate) fn regress(from: Stage, to: Stage, reason: &str) -> Result<Regression, Refusal> {
    if to >= from {
        return Err(Refusal::NotARegression { from, to });
    }
    if reason.trim().is_empty() {
        return Err(Refusal::RegressionReasonMissing { from, to });
    }
    Ok(Regression {
        from,
        to,
        reason: reason.to_owned(),
    })
}
