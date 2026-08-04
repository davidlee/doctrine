// SPDX-License-Identifier: GPL-3.0-only
//! The rule/record correspondence, checked on write (design `sec-4`, `EX-8`).
//!
//! [`super::gate`] owns the **rules** — what an act of a given kind must have
//! been given over. [`super::attestation`] owns the **records** — what an act
//! actually was given over. They are two things that must agree, not one thing
//! written twice, and this module is the single place the agreement is checked.
//!
//! # Why it lives here and not at either end
//!
//! The check belongs to neither owner. Putting it in `gate` would fold admission
//! into the gate, and the design is explicit that the two are different questions
//! asked at different times: this runs **once, on write**, so that every stored
//! act satisfies the correspondence by construction and the gate never re-checks
//! it. Putting it in `run` would bury a self-contained pure predicate in the
//! batch engine.
//!
//! # The one qualification on *the gate never re-checks it*
//!
//! Admission runs on write and never re-runs, so it cannot reach an act stored
//! before its rule changed. A rule that has since grown an [`ObservedFact`] does
//! not make such an act *invalid*; it makes it **unmet**, because the gate reads
//! the record through the rule and an absent stored fingerprint reads as changed.
//! A rule change is the one thing that can retire an act, and it does so by that
//! route rather than through this one.

use std::collections::BTreeSet;

#[cfg(doc)]
use super::attestation::Attestation;
use super::attestation::{
    ActKind, AgentAct, AgentActKind, AgentDeclaration, CheckpointAct, CoveredSet, DisposedPass,
    ReviewDisposition,
};
use super::gate::{ActRule, Coverage, ObservedFact};
use super::ids::DesignId;
use super::refusal::{ActFault, Refusal};
use super::run::ObservedReview;

/// One recorded act, in whichever of the three shapes holds it.
///
/// A borrowed sum rather than a check per shape, because the correspondence is
/// **one** rule that ranges over all three: three of its four rows reach
/// [`CheckpointAct`] alone, and reaching that conclusion per call site is how a
/// row gets forgotten for a shape. Every arm below is total over the three, and
/// a fourth record shape would not compile until it answered every row.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-244 PHASE-05 T6 constructs the records and admits them here"
    )
)]
pub(crate) enum RecordedAct<'a> {
    /// One of the five acts a user gives at a checkpoint — the only shape with a
    /// slot for a confirmation, a disposition or an observed fact.
    Checkpoint(&'a CheckpointAct),
    /// One of the two acts an agent declares about its own work.
    Agent(&'a AgentDeclaration),
    /// The one act that is per-section, recorded as a content-bound
    /// [`Attestation`].
    ///
    /// **Carries no attestation, and the absence is the finding.** All four rows
    /// resolve for this shape without reading a field: it has no covered map
    /// because the derivation quantifies over the section set instead, and no
    /// slot for an observed fact, a confirmation or a disposition. So the answer
    /// is a function of the rule alone — *does it name `PerSection`* — and every
    /// attestation gives the same one. Carrying the record here to look
    /// symmetrical would be carrying a value nothing reads.
    ///
    /// What an attestation *is* checked for is elsewhere by construction: its
    /// subject at [`Refusal::AttestationSubjectMissing`], its lane at the gate.
    Section,
}

impl<'a> RecordedAct<'a> {
    /// Which act this record is of — what a rule names it by, and what a refusal
    /// reports.
    fn kind(self) -> ActKind {
        match self {
            RecordedAct::Checkpoint(act) => act.act,
            RecordedAct::Agent(declaration) => ActKind::from(declaration.act.kind()),
            RecordedAct::Section => ActKind::SectionReviewed,
        }
    }

    /// The disposition this record carries, where its shape has a slot for one.
    fn disposition(self) -> Option<&'a DisposedPass> {
        match self {
            RecordedAct::Checkpoint(act) => act.disposition.as_ref(),
            RecordedAct::Agent(_) | RecordedAct::Section => None,
        }
    }
}

/// Admit one recorded act against the rule it is written against.
///
/// **One pass over the whole record, collecting every fault** — never the first.
/// An agent that repairs one slot and resubmits should not discover the rest one
/// round-trip at a time, which is the rule [`Refusal::GateNotCleared`] already
/// follows for conditions.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-244 PHASE-05 T6 runs this over each act the batch records"
    )
)]
pub(crate) fn admit_act(
    record: RecordedAct<'_>,
    rule: ActRule,
    observed: Option<&ObservedReview>,
) -> Result<(), Refusal> {
    let mut causes = Vec::new();
    causes.extend(coverage_fault(record, rule.binding.coverage));
    causes.extend(observed_fault(record, rule.binding.observed));
    causes.extend(confirmation_fault(record, rule.required.confirms));
    let disposed = record.disposition();
    causes.extend(disposition_fault(disposed, rule.required.disposes_review));
    if let Some(disposed) = disposed {
        disposition_faults(disposed, observed, &mut causes);
    }
    if let RecordedAct::Agent(declaration) = record {
        causes.extend(blocking_set_fault(declaration));
    }
    if causes.is_empty() {
        Ok(())
    } else {
        Err(Refusal::ActAdmissionInvalid {
            act: record.kind(),
            causes,
        })
    }
}

/// Correspondence row 1: the carried [`CoveredSet`] variant is the one the rule's
/// [`Coverage`] names, and `Artefact` pairs with `None`.
fn coverage_fault(record: RecordedAct<'_>, required: Coverage) -> Option<ActFault> {
    let carried = match record {
        // The per-section shape **is** the quantification `PerSection` names: it
        // carries no covered map because the derivation walks the section set
        // instead. So it corresponds to that coverage and to no other.
        RecordedAct::Section => {
            return (required != Coverage::PerSection).then_some(ActFault::CoverageMismatch {
                required,
                carried: None,
            });
        }
        RecordedAct::Checkpoint(act) => act.covered.as_ref(),
        RecordedAct::Agent(declaration) => declaration.covered.as_ref(),
    }
    .map(|covered| match *covered {
        CoveredSet::Sections(_) => Coverage::EverySection,
        CoveredSet::Nodes(_) => Coverage::InquiryMap,
    });
    let agrees = match required {
        Coverage::Artefact => carried.is_none(),
        Coverage::EverySection | Coverage::InquiryMap => carried == Some(required),
        // Carried by no act at all, so no value of a carrying shape corresponds
        // to it — including `None`, which is `Artefact`'s.
        Coverage::PerSection => false,
    };
    (!agrees).then_some(ActFault::CoverageMismatch { required, carried })
}

/// Correspondence row 2: the observed map's key set is **exactly** the rule's
/// fact list — no missing fact, no extra one.
fn observed_fault(record: RecordedAct<'_>, required: &[ObservedFact]) -> Option<ActFault> {
    let carried: BTreeSet<ObservedFact> = match record {
        RecordedAct::Checkpoint(act) => act.observed.keys().copied().collect(),
        // Neither shape has the slot, so an act of either can only ever be given
        // the empty set — which is why no rule naming an observed fact may name
        // their acts, and why that is a build error rather than a check here.
        RecordedAct::Agent(_) | RecordedAct::Section => BTreeSet::new(),
    };
    let missing: Vec<ObservedFact> = required
        .iter()
        .copied()
        .filter(|fact| !carried.contains(fact))
        .collect();
    let extra: Vec<ObservedFact> = carried
        .into_iter()
        .filter(|fact| !required.contains(fact))
        .collect();
    (!missing.is_empty() || !extra.is_empty()).then_some(ActFault::ObservedKeys { missing, extra })
}

/// Correspondence row 3: a confirmation is present exactly when the rule names a
/// declaration, and absent exactly when it does not.
fn confirmation_fault(record: RecordedAct<'_>, expected: Option<AgentActKind>) -> Option<ActFault> {
    let carried = matches!(record, RecordedAct::Checkpoint(act) if act.confirms.is_some());
    (expected.is_some() != carried).then_some(ActFault::Confirmation { expected, carried })
}

/// Correspondence row 4: likewise for a disposition.
fn disposition_fault(disposed: Option<&DisposedPass>, expected: bool) -> Option<ActFault> {
    let carried = disposed.is_some();
    (expected != carried).then_some(ActFault::Disposition { expected, carried })
}

/// What a carried disposition must additionally be — the generated `const`
/// assertion's complement, which is about the *value* rather than the slot.
fn disposition_faults(
    disposed: &DisposedPass,
    observed: Option<&ObservedReview>,
    causes: &mut Vec<ActFault>,
) {
    match disposed.disposition {
        // Admissible over any review state; the reason is the only thing
        // admission checks here.
        ReviewDisposition::Waived { ref reason } => {
            if reason.trim().is_empty() {
                causes.push(ActFault::WaiverReasonMissing);
            }
        }
        ReviewDisposition::Conducted { ref review } => {
            if *review != disposed.pass {
                causes.push(ActFault::ForeignPass {
                    named: review.clone(),
                    current: disposed.pass.clone(),
                });
            }
            // **Absence is refusal, not satisfaction.** An `RV` the shell could
            // not open and one whose ledger has not concluded are one answer,
            // because the question is *can Doctrine see this pass finished* and
            // it cannot, either way. The observation must also be OF this
            // review: one taken over another ledger answers a question nobody
            // asked, and reading it as this one's would be worse than reading
            // nothing.
            if !observed.is_some_and(|seen| seen.reference == *review && seen.concluded) {
                causes.push(ActFault::PassNotConcluded {
                    review: review.clone(),
                });
            }
        }
    }
}

/// A declared blocking set names nodes of the map it was declared over.
///
/// Checked only where a node map is actually carried: where it is not, the
/// coverage row has already reported that the shape is wrong, and listing every
/// declared node beside it would bury that answer under a consequence of it.
fn blocking_set_fault(declaration: &AgentDeclaration) -> Option<ActFault> {
    let AgentAct::BlockingSetDeclared { ref blocking } = declaration.act else {
        return None;
    };
    let Some(CoveredSet::Nodes(covered)) = declaration.covered.as_ref() else {
        return None;
    };
    let nodes: Vec<DesignId> = blocking
        .iter()
        .filter(|node| !covered.covers(node))
        .cloned()
        .collect();
    (!nodes.is_empty()).then_some(ActFault::BlockingSetUnknownNodes { nodes })
}
