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

use super::attestation::{
    AgentAct, AgentActKind, AgentDeclaration, CoveredSet, DisposedPass, RecordedAct,
    ReviewDisposition,
};
use super::gate::{ActRule, Coverage, ObservedFact};
use super::ids::DesignId;
use super::refusal::{ActFault, Refusal};
use super::run::ObservedReview;

/// Admit one recorded act against the rule it is written against.
///
/// **One pass over the whole record, collecting every fault** — never the first.
/// An agent that repairs one slot and resubmits should not discover the rest one
/// round-trip at a time, which is the rule [`Refusal::GateNotCleared`] already
/// follows for conditions.
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
    // The per-section shape **is** the quantification `PerSection` names: it
    // carries no covered map because the derivation walks the section set
    // instead. So it corresponds to that coverage and to no other.
    if matches!(record, RecordedAct::Section) {
        return (required != Coverage::PerSection).then_some(ActFault::CoverageMismatch {
            required,
            carried: None,
        });
    }
    let carried = record.covered().map(|covered| match *covered {
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
    let carried: BTreeSet<ObservedFact> = record.observed().keys().copied().collect();
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
    let carried = record.confirms().is_some();
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
