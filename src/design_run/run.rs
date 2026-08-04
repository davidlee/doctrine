// SPDX-License-Identifier: GPL-3.0-only
//! Admission and the validated candidate — the pure mutation engine.
//!
//! **Home (ADR-001).** This is a `design_run` sibling of [`super::snapshot`]
//! (storage), [`super::submission`] (the payload shape) and [`super::render`]
//! (emission): it is the *domain* step between admitting a payload and storing
//! what it produced, and it depends only downward on leaf peers. It is
//! deliberately not in the command shell — nothing here reads a clock, a disk,
//! or git; every derived fact (a digest, the bytes of `design.md`) arrives as an
//! argument.
//!
//! Two entry points, in the order the shell must call them:
//!
//! 1. [`admit`] answers *may this payload be applied at all* — run identity, the
//!    revision compare-and-swap, submission idempotency, and the replay window.
//! 2. [`apply`] validates the **complete candidate** before any mutation
//!    (DEC-063) and returns the next snapshot plus the change rows it produced.
//!    On any refusal the caller's snapshot is untouched, because the candidate is
//!    built on a clone.
//!
//! Change-row order within a revision is a deterministic function of the
//! declaration *set*, never of submission order: adoption rows by section id,
//! then declaration rows by subject id (a batch refuses duplicate subjects, so
//! subject order is total), then the derived invalidation rows, then the stage
//! move. One `apply` is an unordered batch, so any other rule would be inventing
//! an order the contract says does not exist.

use std::collections::{BTreeMap, BTreeSet};

use super::Stage;
use super::attestation::{
    AcceptanceAttestation, Attestation, ContentCoverage, IntentState, IntentSubject,
    LockAcceptance, RecoveryIntent, ReviewPass, ReviewRef, Reviewer,
};
use super::bounds::DESIGN_ID_BYTES;
use super::change_log::{ChangeEvent, ChangeRow, PayloadKey, PayloadTerm};
use super::delegation::{Delegation, DelegationState, Proposal};
use super::facts::DerivedDesignFacts;
use super::gate;
use super::ids::{DesignId, Fingerprint, IdKind};
use super::inquiry::{Disposition, InquiryLifecycle, InquiryNode, Provenance};
use super::refusal::Refusal;
use super::runbook::{Discharge, Runbook, RunbookKey, StepVerification};
use super::snapshot::{DesignSnapshot, Finding, Pin, Receipt, Section};
use super::submission::{
    ApplyRequest, Batch, Declaration, DelegationAct, DischargeClaim, DischargeDeclaration, Dispose,
    Sparse, SubmissionEnvelope, TraversalDeclaration,
};
use super::traversal::{Authority, Cursor, TraversalPosture};

/// What admitting a payload concluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Admission {
    /// Never seen — apply it.
    Fresh,
    /// Already applied, byte-for-byte. A retry resumes rather than repeats: the
    /// run does not advance, and the caller is told which revision the original
    /// produced.
    Resumed { revision: u64 },
}

/// One marker-addressed section as Doctrine reads it out of `design.md`.
///
/// **Named rather than a tuple** because it is read at the construction site,
/// the completeness check, the order and the seating, and
/// `(usize, String, Fingerprint)` says nothing about which is which. It carries
/// the BODY, not only its digest: a digest alone is what made re-adoption
/// structurally unable to adopt prose, so the run recorded a fingerprint for
/// bytes its own snapshot did not hold and the next materialise reverted the
/// user's edit with the watermark certifying it (EX-5).
///
/// It carries `position` for the same reason it carries `body`: document order
/// is a fact about the document that only the reader can observe, and a
/// [`BTreeMap`] keyed by id has already lost it. A parallel order vector beside
/// the map would be a second structure that can disagree with the first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuthoredSection {
    /// Where this section's marker sits in the document — the order EX-7 adopts.
    pub(crate) position: usize,
    /// The section's authored body, byte for byte.
    pub(crate) body: String,
    /// Its digest, computed by the shell.
    pub(crate) fingerprint: Fingerprint,
}

/// The mechanically derived facts [`apply`] needs and cannot compute (AGENTS.md
/// pure/imperative split).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DerivedInput {
    /// The digest of each declared section body, computed by the shell.
    pub(crate) section_digests: BTreeMap<DesignId, Fingerprint>,
    /// Each marker-addressed section Doctrine reads out of `design.md`, for a
    /// re-adoption. Empty on every other path: nothing but adoption reads it.
    pub(crate) authored_sections: BTreeMap<DesignId, AuthoredSection>,
    /// The fingerprint of `design.md` as Doctrine reads it now.
    pub(crate) authored_fingerprint: Option<Fingerprint>,
    /// The runbook guarding the run's current outbound edge, if that edge has
    /// one (SL-233 PHASE-16).
    ///
    /// Asset data the shell read and digested, arriving where every other
    /// shell-derived fact arrives. Deliberately **not** caller-authored payload:
    /// `StageDeclaration` has no slot for it, and routing it through
    /// `request.evidence` compiles and is then refused by
    /// [`Refusal::DerivedConditionClaimed`] — because a fact about what an asset
    /// says is one Doctrine derives, never one it takes on a caller's word.
    pub(crate) runbook: Option<RunbookFacts>,
    /// What executing a step's `verify` returned, for every check the shell ran
    /// this invocation.
    ///
    /// Derived, for the same reason the asset data above is: whether a check
    /// passed is a fact Doctrine establishes, never one it takes on a caller's
    /// word (`EX-10`). It arrives here rather than on `ApplyRequest` so there is
    /// no wire shape to claim it with — the caller's vocabulary tops out at
    /// `attested`, and `verified` is this input's to grant.
    ///
    /// A collection, not one result, because two seams read it: the step a
    /// payload discharges (`EX-10`), and every already-verified required step
    /// re-checked before a stage act crosses the edge (`EX-11`). One payload can
    /// carry both acts, so both sets of results ride one field and each seam
    /// takes the results that name its steps.
    pub(crate) verifications: Vec<StepVerification>,
    /// The `RV` this invocation must resolve, because an act names one (SL-244
    /// `sec-3`).
    ///
    /// Shell-read and never persisted, arriving where every other shell-derived
    /// fact arrives and for the same reason `runbook` states: what a ledger says
    /// is a fact Doctrine derives, never one it takes on a caller's word.
    ///
    /// `None` is **not** *nothing to check* — it is *nothing could be read*, and
    /// the reader must treat it as refusal rather than satisfaction. An RV the
    /// shell could not open and an act naming none are one answer here on purpose;
    /// the shell distinguishes them at the point it can, and neither clears an
    /// edge.
    pub(crate) observed_review: Option<ObservedReview>,
}

/// One `RV` as the shell read it, for the acts that name one (SL-244 `sec-3`).
///
/// The two fields are read by different checks and that split is the whole of the
/// admission/gate divide: **admission** reads `concluded`, because a `Conducted`
/// arm naming a review that never ran is a false claim to refuse on write; the
/// **gate** reads `undisposed_blockers`, because that is a live property to be
/// re-derived at every crossing rather than a fact about the moment the act was
/// written.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-244 PHASE-05: `review-disposition-attested` and the act that names a \
                  review land there; PHASE-04 puts the input in place ahead of its readers"
    )
)]
pub(crate) struct ObservedReview {
    /// The review the act named. `sec-4`'s type.
    pub(crate) reference: ReviewRef,
    /// Whether the ledger carries the concluded-pass marker.
    pub(crate) concluded: bool,
    /// Findings that are `blocker`-severity AND `open` or `contested` — DEC-138's
    /// predicate, deliberately not D-C9b's `doc_unresolved_blockers`. Carried as
    /// the ledger's own `F-n` ids: they identify rows on the `RV`, not subjects in
    /// the run, so they are not [`DesignId`]s.
    pub(crate) undisposed_blockers: Vec<String>,
}

/// One runbook as the shell read it, with the digest of each step's definition
/// as it stands **now**.
///
/// The digests are here rather than recomputed in the core because hashing is a
/// shell concern: this leaf owns [`super::runbook::Step::material`] and never
/// hashes it (`EX-18`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RunbookFacts {
    pub(crate) key: RunbookKey,
    pub(crate) book: Runbook,
    /// Step id → digest of [`super::runbook::Step::material`].
    pub(crate) digests: BTreeMap<String, String>,
}

/// A validated candidate: the next snapshot, and the rows it produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Applied {
    pub(crate) snapshot: DesignSnapshot,
    pub(crate) rows: Vec<ChangeRow>,
}

/// May this payload be applied?
///
/// The order of the checks is the contract. Idempotency is answered *before* the
/// compare-and-swap, because a retry legitimately carries the revision it was
/// built against and would otherwise look like a stale writer. The replay window
/// is answered next: below the retained floor an unknown id cannot be told apart
/// from one already applied and evicted, so it is refused as **expired** rather
/// than silently treated as new.
pub(crate) fn admit(
    prior: &DesignSnapshot,
    envelope: &SubmissionEnvelope,
    payload_digest: &str,
) -> Result<Admission, Refusal> {
    if envelope.run_uid != prior.run.uid {
        return Err(Refusal::RunMismatch {
            declared: envelope.run_uid.clone(),
            current: prior.run.uid.clone(),
        });
    }
    if let Some(receipt) = prior.receipts.find(&envelope.submission_id) {
        return if receipt.digest == payload_digest {
            Ok(Admission::Resumed {
                revision: receipt.revision,
            })
        } else {
            Err(Refusal::SubmissionReplayed {
                submission: envelope.submission_id.clone(),
            })
        };
    }
    if envelope.known_revision < prior.receipts.floor {
        return Err(Refusal::SubmissionExpired {
            known: envelope.known_revision,
            floor: prior.receipts.floor,
        });
    }
    if envelope.known_revision != prior.run.revision {
        return Err(Refusal::StaleRevision {
            known: envelope.known_revision,
            current: prior.run.revision,
        });
    }
    Ok(Admission::Fresh)
}

/// A row before its index is assigned — index is a property of the *sequence*,
/// so assigning it while building would tie it to construction order.
struct Pending {
    event: ChangeEvent,
    subject: Option<DesignId>,
    terms: Vec<PayloadTerm>,
}

impl Pending {
    /// A row about one run-local subject.
    fn about(event: ChangeEvent, subject: &DesignId, terms: Vec<PayloadTerm>) -> Self {
        Pending {
            event,
            subject: Some(subject.clone()),
            terms: event.ordered(terms),
        }
    }

    /// A row about the run itself.
    fn run_wide(event: ChangeEvent, terms: Vec<PayloadTerm>) -> Self {
        Pending {
            event,
            subject: None,
            terms: event.ordered(terms),
        }
    }
}

/// What the shell resolved for this candidate — provisional on the first
/// validation pass, real on the second (D2).
///
/// One struct rather than a growing positional list. Deliberately **not**
/// [`DerivedInput`]: that is where *observed* facts arrive, and a minted id is
/// resolved rather than observed.
#[derive(Debug, Default)]
pub(crate) struct Resolution {
    /// Each checkpoint's canonical record. A parameter rather than a pre-applied
    /// rewrite of `request` because an accepted proposal's declarations exist
    /// only inside [`apply`] (`EX-3`, D1).
    pub(crate) checkpoints: BTreeMap<DesignId, String>,
    /// The `RV` minted for a pass this candidate opens, if it opens one.
    pub(crate) review_pass: Option<ReviewRef>,
}

/// Build the whole candidate, or refuse and leave `prior` untouched.
pub(crate) fn apply(
    prior: &DesignSnapshot,
    request: &ApplyRequest,
    derived: &DerivedInput,
    payload_digest: &str,
    resolved: &Resolution,
) -> Result<Applied, Refusal> {
    // Before anything is touched: a payload may not claim a clearance Doctrine
    // derives for itself. Checked over the whole candidate rather than inside the
    // recording loop, so the refusal does not depend on where in the list the
    // claim sits (DEC-063).
    if let Some(claimed) = request
        .evidence
        .iter()
        .find(|evidence| evidence.condition.is_derived())
    {
        return Err(Refusal::DerivedConditionClaimed {
            condition: claimed.condition,
        });
    }

    // And before anything is touched: a delegate's channel may not carry a writer
    // act (EX-2, DEC-068). Checked over the whole payload for the same reason the
    // claim check is — the refusal must not depend on which key came first.
    if let Some(act) = request.delegation.as_ref()
        && act.is_proposal()
        && let Some(what) = request.writer_act()
    {
        return Err(Refusal::DelegateCannotAdvance { what });
    }

    let mut next = prior.clone();
    let revision = prior.run.revision.saturating_add(1);
    next.run.revision = revision;

    let live_evidence_before = live_evidence(&prior.gate);
    // Unfiltered by the run's review policy, on purpose: this pair feeds
    // `invalidation_rows`, which reports the death of a recorded act. Whether an
    // attestation satisfied a lane the run requires is a different question,
    // asked at the gate — see `live_reviews`.
    let live_reviews_before = live_reviews(prior);

    let mut pending: Vec<Pending> = Vec::new();

    if let Some(adopt) = request.adopt_authored.as_ref() {
        pending.extend(adopt_authored(&mut next, adopt, derived)?);
    }

    // The delegation act runs BEFORE the declaration loop, because an `accept`
    // contributes declarations to it: the delegate's proposed map changes join the
    // coordinator's own in ONE batch, so they go through the one declaration
    // engine, take the one deterministic row order, and a subject named by both is
    // refused as the duplicate it is rather than silently resolved by an order the
    // contract says does not exist (DEC-063).
    //
    // The consequence to know: an `export` names an obligation that must already
    // exist — a node created by this same batch is not yet in the map.
    let accepted = match request.delegation.as_ref() {
        Some(act) => {
            let (rows, declarations) = delegate(&mut next, act, revision)?;
            pending.extend(rows);
            declarations
        }
        None => Vec::new(),
    };

    // `EX-3` / D1 — bind each resolved canonical id onto the declaration that
    // named it, HERE, where the two origins have just become one batch.
    //
    // The shell cannot do this. It never holds an accepted proposal's
    // declarations: this core re-reads them out of the prior snapshot itself
    // (`delegate` above), deliberately, so neither side trusts the other's copy.
    // A shell-side walk therefore has nothing to bind them onto — which is why
    // the incumbent `bind_resolved` was not merely incomplete but structurally
    // incapable of covering them, and why it is deleted rather than left beside
    // this (two implementations of "bind the resolved id onto the declaration
    // that named it" is the duplication the no-parallel-implementation rule is
    // about).
    //
    // Ordering: before `validate`, and therefore before `declare` reaches
    // `checkpoint_disposition`'s read of `resolved_record` — which is the read
    // that has to see it.
    let mut declarations = request.declare.clone();
    declarations.extend(accepted);
    for declaration in &mut declarations {
        if let Some(record) = resolved.checkpoints.get(declaration.subject()) {
            *declaration = declaration.clone().resolving(record);
        }
    }
    for (_, declaration) in Batch::of(declarations).validate()? {
        pending.extend(declare(
            &mut next,
            &declaration,
            derived,
            &request.envelope.submission_id,
        )?);
    }

    // Sections may have moved, so re-observe every subject before evidence is
    // recorded or liveness re-derived: DEC-066 binds evidence to *current*
    // content, and a clearance recorded in this batch is recorded against what
    // this batch left behind.
    reobserve(&mut next);
    for evidence in &request.evidence {
        let fingerprint = current_fingerprint(&next, &evidence.subject)?;
        next.gate = std::mem::take(&mut next.gate).record(
            evidence.condition,
            evidence.subject.clone(),
            fingerprint,
        );
    }

    // After re-observation, so the coverage is of what this batch left behind,
    // and before the stage move, so a lock may be accepted and taken in one
    // submission.
    if let Some(declared) = request.acceptance.as_ref() {
        if declared.basis.trim().is_empty() {
            return Err(Refusal::AcceptanceBasisMissing);
        }
        next.review.acceptance = Some(LockAcceptance::over(
            AcceptanceAttestation::bind(
                declared.basis.clone(),
                declared.turn.clone(),
                Fingerprint::new(payload_digest),
            ),
            ContentCoverage::of(next.sections.fingerprints()),
        ));
        pending.push(Pending::run_wide(
            ChangeEvent::AcceptanceAttested,
            Vec::new(),
        ));
    }

    // Beside the acceptance and in the same shape, because it is the same kind of
    // act: a user judgement about the run as a whole. The policy is mutable on
    // purpose — see `ReviewPolicyDeclaration` — so the fence here is authority and
    // visibility rather than prohibition: the basis is required, the acceptance is
    // bound through the one route that carries `AcceptanceAuthority::User`, and
    // the change is logged.
    if let Some(declared) = request.review_policy.as_ref() {
        if declared.acceptance.basis.trim().is_empty() {
            return Err(Refusal::AcceptanceBasisMissing);
        }
        let previous = next.run.review_policy;
        // Only when the value actually moves: a payload re-declaring the policy
        // already in force has changed nothing, and a row saying otherwise would
        // make the log report acts that did not happen.
        if previous != declared.policy {
            next.run.review_policy = declared.policy;
            pending.push(Pending::run_wide(
                ChangeEvent::ReviewPolicyChanged,
                vec![
                    PayloadTerm::label(PayloadKey::Old, previous.as_str())?,
                    PayloadTerm::label(PayloadKey::New, declared.policy.as_str())?,
                ],
            ));
        }
    }

    pending.extend(invalidation_rows(
        &live_evidence_before,
        &live_evidence(&next.gate),
        &live_reviews_before,
        &live_reviews(&next),
    )?);

    // Traversal direction is *state*, not delta (sketch §(d): cursor moves and
    // posture changes are deliberately not members of the material-change
    // vocabulary), so it produces no rows — but it is still validated with the
    // rest of the candidate, before anything is written.
    direct_traversal(&mut next, &request.traversal)?;

    if let Some(declared) = request.discharge.as_ref() {
        pending.push(discharge_step(
            &mut next,
            declared,
            derived.runbook.as_ref(),
            &derived.verifications,
        )?);
    }

    if let Some(stage) = request.stage.as_ref() {
        pending.push(stage_move(
            &mut next,
            stage.to,
            stage.reason.as_deref(),
            derived.runbook.as_ref(),
            &derived.verifications,
        )?);
    }

    // DEC-125: entry to `reviewing` opens a review pass over the content as it
    // stands. Read off the candidate rather than the request's declared target —
    // a refused advance never reaches here, so a pass is never opened for a move
    // that did not happen, and the shell never pre-judges this core's admission.
    //
    // Assignment, not insertion: a later entry **replaces** the pass and never
    // reopens the old one (`EX-4`). The `RV` the old pass named stays authored;
    // what moves is which pass the run is on.
    if next.run.stage == Stage::Reviewing
        && prior.run.stage != Stage::Reviewing
        && let Some(review) = resolved.review_pass.clone()
    {
        next.review.pass = Some(ReviewPass::over(
            review,
            ContentCoverage::of(next.sections.fingerprints()),
        ));
    }

    let rows: Vec<ChangeRow> = pending
        .into_iter()
        .enumerate()
        .map(|(index, row)| ChangeRow {
            revision,
            index: u32::try_from(index).unwrap_or(u32::MAX),
            event: row.event,
            subject: row.subject,
            terms: row.terms,
        })
        .collect();

    next.change_log.record(revision, rows.clone());
    // Only an EXPORT pins its receipt. The pin exists so retention cannot evict
    // the receipt of the submission an outstanding assignment was cut at
    // (PHASE-03 EX-4, consumed here rather than restated); the later acts are
    // ordinary submissions, and pinning them would hold receipts for a reason
    // nothing needs.
    let (delegation, delegation_state) = match request.delegation.as_ref() {
        Some(DelegationAct::Export { id, .. }) => (
            Some(id.as_str().to_owned()),
            Some(DelegationState::Outstanding),
        ),
        _ => (None, None),
    };
    next.receipts.record(Receipt {
        submission: request.envelope.submission_id.clone(),
        revision,
        digest: payload_digest.to_owned(),
        delegation,
        delegation_state,
    });
    Ok(Applied {
        snapshot: next,
        rows,
    })
}

/// DEC-092 rule 2: the sole lawful crossing of a divergence, as a protocol.
///
/// The declared fingerprint must be what Doctrine reads, and the stable-marker
/// map must be **complete and exact** — every section the run holds, no unknown
/// one, and every digest matching what Doctrine read. Affected evidence is
/// invalidated by the DEC-066 rule that already governs it (the section's
/// fingerprint moves, so evidence bound to the old one stops being live); no
/// clearance is inherited across the crossing, because clearance is derived and
/// never stored.
fn adopt_authored(
    next: &mut DesignSnapshot,
    adopt: &super::submission::AdoptAuthored,
    derived: &DerivedInput,
) -> Result<Vec<Pending>, Refusal> {
    let observed = derived.authored_fingerprint.as_ref();
    if observed.map(Fingerprint::as_str) != Some(adopt.fingerprint.as_str()) {
        return Err(Refusal::AdoptionStale {
            declared: adopt.fingerprint.clone(),
            observed: observed.map(|f| f.as_str().to_owned()),
        });
    }
    let held: BTreeSet<DesignId> = next.sections.ids();
    let declared: BTreeSet<DesignId> = adopt.sections.keys().cloned().collect();
    let missing: Vec<DesignId> = held.difference(&declared).cloned().collect();
    let unknown: Vec<DesignId> = declared.difference(&held).cloned().collect();
    let mismatched: Vec<DesignId> = adopt
        .sections
        .iter()
        .filter(|(id, digest)| {
            derived
                .authored_sections
                .get(*id)
                .map(|authored| authored.fingerprint.as_str())
                != Some(digest.as_str())
        })
        .map(|(id, _)| id.clone())
        .collect();
    if !missing.is_empty() || !unknown.is_empty() || !mismatched.is_empty() {
        return Err(Refusal::AdoptionMarkersInvalid {
            missing,
            unknown,
            mismatched,
        });
    }

    // DOCUMENT ORDER IS AUTHORITATIVE (EX-7). Adoption walks the marker
    // sequence and re-claims `seq` from the run's counter, so hand-reordering
    // sections is a supported edit rather than a sixth refusal: it loses no
    // information, and refusing it would make the authored tier less editable
    // than a plain file. The values are claimed AFRESH rather than renumbered to
    // 0..n, because `MapGroup::next_seq` is shared with the inquiry map and its
    // stated invariant is monotonic-and-never-reused; the observable contract —
    // document order equals marker order — holds either way.
    //
    // The asymmetry with re-DECLARE, which carries `seq` forward, is deliberate:
    // the document is authoritative about order only where the human has edited
    // the document.
    let mut ordered: Vec<(&DesignId, &AuthoredSection)> =
        derived.authored_sections.iter().collect();
    ordered.sort_by_key(|(_, authored)| authored.position);

    let mut rows = Vec::new();
    for (id, authored) in ordered {
        let Some(section) = next.sections.find(id).cloned() else {
            continue;
        };
        let seq = next.map.claim_seq();
        seat_section(next, id, &authored.body, &authored.fingerprint, seq, None)?;
        if section.fingerprint == authored.fingerprint {
            continue;
        }
        rows.push(Pending::about(
            ChangeEvent::SectionFingerprintChanged,
            id,
            vec![
                PayloadTerm::digest(PayloadKey::Old, section.fingerprint.as_str()),
                PayloadTerm::digest(PayloadKey::New, authored.fingerprint.as_str()),
            ],
        ));
    }
    Ok(rows)
}

/// Seat a section: derive its title from its own body, then store it.
///
/// **The only place a [`Section`] is constructed** (EX-6, VA-7). Both doors into
/// the section group — a structured `declare` and a re-adoption of hand-edited
/// prose — come through here, so the derivation cannot be bypassed by one of
/// them. Two construction sites each calling `derive_title` would be two ways to
/// say one thing with only one of them guaranteed to stay right, which is the
/// dual-spelling defect EX-12 exists to close, reintroduced at a different seam.
fn seat_section(
    next: &mut DesignSnapshot,
    id: &DesignId,
    body: &str,
    fingerprint: &Fingerprint,
    seq: u64,
    source_line: Option<usize>,
) -> Result<(), Refusal> {
    let title = super::section::derive_title(id, body)?.to_owned();
    // Provenance is claimed once, at import, and CARRIED. Every other door
    // passes `None` — which must not mean "erase what import recorded", or a
    // section would stop being imported the first time it is re-adopted, and the
    // three doors would disagree about a fact none of them can re-derive.
    let source_line =
        source_line.or_else(|| next.sections.find(id).and_then(|held| held.source_line));
    next.sections.upsert(Section {
        id: id.clone(),
        title,
        body: body.to_owned(),
        fingerprint: fingerprint.clone(),
        seq,
        source_line,
    });
    Ok(())
}

/// Seat every region of an imported legacy `design.md`, in document order
/// (SL-233 PHASE-11, `EX-2`).
///
/// The **third door** into the section group, and it comes through
/// [`seat_section`] like the other two: one derivation, no bypass (`VA-7`).
/// What it adds over them is only what import knows and they do not — a minted
/// id, and the source line the region's heading stands on.
///
/// Ids are minted `sec-1`, `sec-2`, … in document order, from
/// [`IdKind::Section`]'s own prefix rather than a re-spelled literal (STD-001).
/// Minting is safe precisely because import is a **one-time** path into a fresh
/// run (`EX-1`): there is nothing seated to collide with, so no id-allocation
/// question is being answered here and none should be invented.
///
/// Nothing else is manufactured. No attestation, no receipt, no gate evidence,
/// no stage move — the run stays where [`DesignSnapshot::new`] left it, and
/// prose that calls itself locked does not move it (`EX-5`, DEC-084 principle
/// 6). That is a property of what this function *does not do*, so there is no
/// code to point at; the enumeration in `EX-5`'s test is the guard.
///
/// Returns the seated ids in document order, index-aligned with `regions`, so a
/// later pass can name the section a region's prose belongs to without
/// re-deriving the pairing.
///
/// # The inquiry map it seeds (`EX-3`, `EX-4`)
///
/// Two sources, in this order, and no third:
///
/// 1. every **direct non-terminal shaping QUE** the shell found linked to the
///    slice, as a durable [`Provenance::ShapingQuestion`] node. These are facts
///    about the knowledge corpus, so the shell reads them and hands them in;
/// 2. every conventional `OQ-*` entry of the **explicit Open Questions
///    section**, as an unverified [`Provenance::ImportedProse`] node carrying
///    the section it stands in and the line it stands on. An `OQ-*` anywhere
///    else is ordinary body prose and stays that way.
///
/// The two **merge only on an explicit canonical citation**: an entry that names
/// a record already seeded at (1) adds nothing, because the durable node is
/// already that question. Two entries whose text is byte-identical stay two
/// nodes. There is no function here that compares question texts to each other,
/// which is what makes "text never merges them" a property of the code rather
/// than of a test that happens to assert one case (`VA-3`).
pub(crate) fn import(
    next: &mut DesignSnapshot,
    regions: &[(super::legacy::Region<'_>, Fingerprint)],
    shaping: &[ShapingQuestion],
    entry_digests: &BTreeMap<usize, Fingerprint>,
) -> Result<Vec<DesignId>, Refusal> {
    let mut seated = Vec::with_capacity(regions.len());
    for (position, (region, fingerprint)) in regions.iter().enumerate() {
        let id = DesignId::parse(&format!(
            "{}{}",
            IdKind::Section.prefix(),
            position.saturating_add(1)
        ))?;
        let seq = next.map.claim_seq();
        seat_section(next, &id, region.body, fingerprint, seq, Some(region.line))?;
        seated.push(id);
    }

    let mut cited: BTreeSet<&str> = BTreeSet::new();
    for question in shaping {
        seed_node(
            next,
            InquiryNode::open(
                mint_inquiry_id(next)?,
                question.question.clone(),
                Provenance::ShapingQuestion {
                    record: question.record.clone(),
                },
            ),
        )?;
        cited.insert(question.record.as_str());
    }

    for (id, (region, _)) in seated.iter().zip(regions) {
        let Some(section) = next.sections.find(id) else {
            continue;
        };
        if !super::legacy::is_open_questions(&section.title) {
            continue;
        }
        for entry in super::legacy::open_questions(region) {
            if entry.citation.is_some_and(|record| cited.contains(record)) {
                continue;
            }
            let line = u32::try_from(entry.line).unwrap_or(u32::MAX);
            let Some(fingerprint) = entry_digests.get(&entry.line) else {
                return Err(Refusal::ImportedEntryDigestMissing { line });
            };
            seed_node(
                next,
                InquiryNode::open(
                    mint_inquiry_id(next)?,
                    entry.question.to_owned(),
                    Provenance::ImportedProse {
                        section: id.clone(),
                        line,
                        label: entry.label.to_owned(),
                        fingerprint: fingerprint.clone(),
                    },
                ),
            )?;
        }
    }
    Ok(seated)
}

/// One direct non-terminal shaping QUE, as the shell reads it out of the
/// knowledge corpus — a fact the pure core cannot observe and is handed
/// (AGENTS.md pure/imperative split).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ShapingQuestion {
    /// Its canonical id, e.g. `QUE-177` — what a citation must name to merge.
    pub(crate) record: String,
    /// Its question text, as the record states it. Import never restates it.
    pub(crate) question: String,
}

/// The next free inquiry id: `inq-1`, `inq-2`, … past whatever the map holds.
///
/// Import is one-time into a fresh run, so this counts up from empty in
/// practice; it steps over an occupied id rather than assuming that, because an
/// id collision would be a silent overwrite of somebody else's node.
fn mint_inquiry_id(next: &DesignSnapshot) -> Result<DesignId, Refusal> {
    let mut ordinal = next.map.inquiry.len().saturating_add(1);
    loop {
        let id = DesignId::parse(&format!("{}{ordinal}", IdKind::Inquiry.prefix()))?;
        if next.map.inquiry.get(&id).is_none() {
            return Ok(id);
        }
        ordinal = ordinal.saturating_add(1);
    }
}

/// Seed one imported node, sequenced from the run's own counter.
fn seed_node(next: &mut DesignSnapshot, node: InquiryNode) -> Result<(), Refusal> {
    let seq = next.map.claim_seq();
    next.map.inquiry.insert(node.sequenced(seq))
}

/// Apply one delegation act (DEC-068, `EX-1`/`EX-2`/`EX-3`).
///
/// Returns the rows it produced and the declarations it contributes to this
/// batch — empty for every act but `accept`, whose whole substance is the
/// coordinator taking the delegate's proposed map changes as its own.
///
/// **Nothing here applies a declaration.** `propose` stores them; `accept` hands
/// them back to [`apply`] to put through [`declare`]. That is the whole of "the
/// coordinator is sole writer": there is no route from a delegate's bytes to the
/// map that does not pass through the coordinator's act.
fn delegate(
    next: &mut DesignSnapshot,
    act: &DelegationAct,
    revision: u64,
) -> Result<(Vec<Pending>, Vec<Declaration>), Refusal> {
    let id = act.id();
    match act {
        DelegationAct::Export { obligation, .. } => {
            let Some(node) = next.map.inquiry.get(obligation).cloned() else {
                return Err(Refusal::UnknownNode {
                    id: obligation.clone(),
                });
            };
            if let Some(held) = next.delegation.outstanding_for(obligation) {
                return Err(Refusal::DelegationOutstanding {
                    id: held.id().clone(),
                    obligation: obligation.clone(),
                });
            }
            next.delegation
                .upsert(Delegation::exported(id.clone(), node, revision));
            Ok((
                vec![delegation_row(
                    ChangeEvent::ObligationDelegated,
                    id,
                    obligation,
                    None,
                )?],
                Vec::new(),
            ))
        }
        DelegationAct::Propose {
            by,
            summary,
            declare,
            ..
        } => {
            let held = held_delegation(next, id)?;
            let obligation = held.obligation().clone();
            next.delegation
                .upsert(held.proposed(Proposal::of(by, summary, declare.clone())));
            next.receipts
                .restate_delegation(id.as_str(), DelegationState::Proposed);
            Ok((
                vec![delegation_row(
                    ChangeEvent::ProposalRecorded,
                    id,
                    &obligation,
                    Some(PayloadTerm::prose(PayloadKey::By, by)),
                )?],
                Vec::new(),
            ))
        }
        DelegationAct::Accept { .. } => {
            let held = held_delegation(next, id)?;
            let Some(proposal) = held.proposal().cloned() else {
                return Err(Refusal::ProposalMissing { id: id.clone() });
            };
            // Measured against the map as it stands BEFORE this batch's
            // declarations, so the accepted proposal cannot make itself current.
            if held.is_stale(&next.map.inquiry) {
                return Err(Refusal::ProposalStale {
                    id: id.clone(),
                    obligation: held.obligation().clone(),
                    exported_at: held.exported_at(),
                });
            }
            let obligation = held.obligation().clone();
            next.delegation.upsert(held.accepted());
            next.receipts
                .restate_delegation(id.as_str(), DelegationState::Accepted);
            Ok((
                vec![delegation_row(
                    ChangeEvent::ProposalAccepted,
                    id,
                    &obligation,
                    Some(PayloadTerm::prose(PayloadKey::By, proposal.by())),
                )?],
                proposal.declarations().to_vec(),
            ))
        }
        DelegationAct::Refuse { reason, .. } => {
            let held = held_delegation(next, id)?;
            if held.proposal().is_none() {
                return Err(Refusal::ProposalMissing { id: id.clone() });
            }
            let obligation = held.obligation().clone();
            next.delegation.upsert(held.refused(reason));
            next.receipts
                .restate_delegation(id.as_str(), DelegationState::Refused);
            Ok((
                vec![delegation_row(
                    ChangeEvent::ProposalRefused,
                    id,
                    &obligation,
                    Some(PayloadTerm::prose(PayloadKey::Reason, reason)),
                )?],
                Vec::new(),
            ))
        }
    }
}

/// The delegation an act names, or a refusal naming what was asked for.
fn held_delegation(next: &DesignSnapshot, id: &DesignId) -> Result<Delegation, Refusal> {
    next.delegation
        .find(id.as_str())
        .cloned()
        .ok_or_else(|| Refusal::UnknownDelegation { id: id.clone() })
}

/// One delegation row: subject is the **assignment**, the obligation rides a
/// term, and the act's own prose (attribution or reason) follows it.
fn delegation_row(
    event: ChangeEvent,
    id: &DesignId,
    obligation: &DesignId,
    prose: Option<PayloadTerm>,
) -> Result<Pending, Refusal> {
    let mut terms = vec![PayloadTerm::token(PayloadKey::Node, obligation.as_str())?];
    terms.extend(prose);
    Ok(Pending::about(event, id, terms))
}

/// Apply one subject-addressed declaration. What it means is derived from the
/// subject's kind, so a mistyped prefix is a refusal rather than a mismatch.
fn declare(
    next: &mut DesignSnapshot,
    declaration: &Declaration,
    derived: &DerivedInput,
    submission: &str,
) -> Result<Vec<Pending>, Refusal> {
    match declaration.subject().kind() {
        IdKind::Inquiry => declare_node(next, declaration),
        IdKind::Section => declare_section(next, declaration, derived),
        IdKind::Attestation => declare_attestation(next, declaration),
        IdKind::Finding => declare_finding(next, declaration),
        IdKind::Checkpoint => declare_checkpoint(next, declaration, submission),
        // None of these three is a declaration subject; each is written through
        // a run-level payload field of its own. A `dlg-` assignment is about the
        // run's obligation rather than about a subject the batch declares, and a
        // recorded act or agent declaration is constructed by the engine after
        // the batch has been applied. A subject route to any of them would be a
        // *second* way into state the sole-writer boundary keeps single.
        IdKind::Delegation | IdKind::CheckpointAct | IdKind::AgentDeclaration => {
            Err(Refusal::SubjectNotDeclarable {
                id: declaration.subject().clone(),
            })
        }
    }
}

/// Create a node, or move an existing one.
fn declare_node(
    next: &mut DesignSnapshot,
    declaration: &Declaration,
) -> Result<Vec<Pending>, Refusal> {
    let id = declaration.subject();
    let Some(existing) = next.map.inquiry.get(id).cloned() else {
        let question = match declaration.question_declaration() {
            Sparse::Value(question) => question.clone(),
            Sparse::Omitted | Sparse::Null => String::new(),
        };
        let provenance = declaration
            .provenance()
            .cloned()
            .unwrap_or(Provenance::AgentProposed);
        let mut node = InquiryNode::open(id.clone(), question, provenance.clone())
            .sequenced(next.map.claim_seq());
        let parent = match declaration.parent_declaration() {
            Sparse::Value(parent) => Some(parent.clone()),
            Sparse::Omitted | Sparse::Null => None,
        };
        if let Some(parent) = parent.clone() {
            node = node.with_parent(parent);
        }
        if let Sparse::Value(needs) = declaration.needs_declaration() {
            for need in needs {
                node = node.needing(need.clone());
            }
        }
        next.map.inquiry.insert(node)?;
        let mut terms = Vec::new();
        if let Some(parent) = parent {
            terms.push(PayloadTerm::token(PayloadKey::Parent, parent.as_str())?);
        }
        terms.push(PayloadTerm::label(
            PayloadKey::Provenance,
            provenance.label(),
        )?);
        return Ok(vec![Pending::about(ChangeEvent::NodeCreated, id, terms)]);
    };

    let mut rows = Vec::new();
    let mut parent = existing.parent().cloned();
    let mut needs: BTreeSet<DesignId> = existing.needs().clone();
    let mut lifecycle = existing.lifecycle();
    // The three sparse states, on the one prose scalar a node carries: omission
    // PERSISTS the prior question, `null` clears it, a value replaces it. A
    // question change is not a material change (it is not a member of the closed
    // §(d) vocabulary), so it produces no row — it is state, not delta.
    let question = declaration
        .question_declaration()
        .clone()
        .apply(Some(existing.question().to_owned()))
        .unwrap_or_default();

    match declaration.parent_declaration() {
        Sparse::Value(declared) if parent.as_ref() != Some(declared) => {
            let mut terms = Vec::new();
            if let Some(old) = parent.as_ref() {
                terms.push(PayloadTerm::token(PayloadKey::Old, old.as_str())?);
            }
            terms.push(PayloadTerm::token(PayloadKey::New, declared.as_str())?);
            rows.push(Pending::about(ChangeEvent::NodeReparented, id, terms));
            parent = Some(declared.clone());
        }
        Sparse::Null if parent.is_some() => {
            let mut terms = Vec::new();
            if let Some(old) = parent.as_ref() {
                terms.push(PayloadTerm::token(PayloadKey::Old, old.as_str())?);
            }
            rows.push(Pending::about(ChangeEvent::NodeReparented, id, terms));
            parent = None;
        }
        _ => {}
    }

    if let Sparse::Value(declared) = declaration.needs_declaration() {
        let declared: BTreeSet<DesignId> = declared.iter().cloned().collect();
        for added in declared.difference(&needs) {
            rows.push(Pending::about(
                ChangeEvent::NeedsAdded,
                id,
                vec![
                    PayloadTerm::token(PayloadKey::From, id.as_str())?,
                    PayloadTerm::token(PayloadKey::To, added.as_str())?,
                ],
            ));
        }
        for removed in needs.difference(&declared) {
            rows.push(Pending::about(
                ChangeEvent::NeedsRemoved,
                id,
                vec![
                    PayloadTerm::token(PayloadKey::From, id.as_str())?,
                    PayloadTerm::token(PayloadKey::To, removed.as_str())?,
                ],
            ));
        }
        needs = declared;
    }

    if let Some(declared) = declaration.lifecycle()
        && declared != lifecycle
    {
        rows.push(Pending::about(
            ChangeEvent::NodeLifecycle,
            id,
            vec![
                PayloadTerm::label(PayloadKey::From, lifecycle.as_str())?,
                PayloadTerm::label(PayloadKey::To, declared.as_str())?,
            ],
        ));
        lifecycle = declared;
    }

    let rebuilt = rebuild(&existing, &question, parent, needs, lifecycle)?;
    next.map.inquiry.insert(rebuilt)?;
    Ok(rows)
}

/// Rebuild a node with a new edge set and lifecycle.
///
/// `needs` removal has no in-place setter by design — the node's fields are
/// private and its builders only add — so the honest move is to rebuild from the
/// facts that survive rather than to open the type up.
fn rebuild(
    existing: &InquiryNode,
    question: &str,
    parent: Option<DesignId>,
    needs: BTreeSet<DesignId>,
    lifecycle: InquiryLifecycle,
) -> Result<InquiryNode, Refusal> {
    let mut node = InquiryNode::open(
        existing.id().clone(),
        question,
        existing.provenance().clone(),
    )
    .sequenced(existing.seq());
    if let Some(parent) = parent {
        node = node.with_parent(parent);
    }
    for need in needs {
        node = node.needing(need);
    }
    match (lifecycle, existing.disposition().cloned()) {
        (InquiryLifecycle::Resolved, Some(disposition)) => Ok(node.resolve(disposition)),
        (InquiryLifecycle::Resolved, None) => Err(Refusal::DispositionMissing {
            id: existing.id().clone(),
        }),
        (other, _) => node.transition(other),
    }
}

/// Create a section, or record that its content moved.
///
/// The title is **derived** from the body's own opening heading (EX-13(b)) by
/// [`seat_section`], which both this door and re-adoption go through, so a body
/// that cannot yield a title is refused rather than seated with an empty one —
/// through either door.
///
/// Two admissions run before it, in this order (EX-4, EX-6):
///
/// - a body carrying `\r` is refused on the **document boundary**, so it cannot
///   reach derivation or materialise through a door the document parse has
///   already closed;
/// - a NEW section claims a `seq` from the run's existing counter, and a
///   RE-DECLARED one carries its prior `seq` forward. Claiming afresh on every
///   edit would move existing prose to the end of the document — the defect
///   document order exists to close.
fn declare_section(
    next: &mut DesignSnapshot,
    declaration: &Declaration,
    derived: &DerivedInput,
) -> Result<Vec<Pending>, Refusal> {
    let id = declaration.subject();
    let (Some(body), Some(fingerprint)) = (declaration.body(), derived.section_digests.get(id))
    else {
        return Err(Refusal::SectionBodyMissing { id: id.clone() });
    };
    if body.contains('\r') {
        return Err(Refusal::CarriageReturnInDocument);
    }
    let prior = next.sections.find(id).cloned();
    let seq = prior
        .as_ref()
        .map_or_else(|| next.map.claim_seq(), |held| held.seq);
    seat_section(next, id, body, fingerprint, seq, None)?;
    Ok(match prior {
        None => vec![Pending::about(
            ChangeEvent::SectionCreated,
            id,
            vec![PayloadTerm::digest(
                PayloadKey::Fingerprint,
                fingerprint.as_str(),
            )],
        )],
        Some(prior) if &prior.fingerprint != fingerprint => vec![Pending::about(
            ChangeEvent::SectionFingerprintChanged,
            id,
            vec![
                PayloadTerm::digest(PayloadKey::Old, prior.fingerprint.as_str()),
                PayloadTerm::digest(PayloadKey::New, fingerprint.as_str()),
            ],
        )],
        Some(_) => Vec::new(),
    })
}

/// Bind a review attestation to the exact bytes reviewed (DEC-073).
fn declare_attestation(
    next: &mut DesignSnapshot,
    declaration: &Declaration,
) -> Result<Vec<Pending>, Refusal> {
    let id = declaration.subject();
    let Some(subject) = declaration.attests() else {
        return Err(Refusal::AttestationSubjectMissing { id: id.clone() });
    };
    let Some(section) = next.sections.find(subject).cloned() else {
        return Err(Refusal::AttestationSubjectMissing { id: id.clone() });
    };
    next.review.attestations.retain(|held| held.id() != id);
    next.review.attestations.push(Attestation::bind(
        id.clone(),
        subject.clone(),
        section.fingerprint,
        declaration.reviewer().unwrap_or(Reviewer::Human),
    ));
    Ok(vec![Pending::about(
        ChangeEvent::ReviewAttested,
        id,
        vec![
            PayloadTerm::token(PayloadKey::Section, subject.as_str())?,
            PayloadTerm::token(PayloadKey::Attestation, id.as_str())?,
        ],
    )])
}

/// Raise a finding, or dispose one already raised.
///
/// The two acts are one route because they are one record's life, but they are
/// **not** interchangeable: raising requires the summary and the section, and
/// disposing requires the finding to exist. A `resolution` naming a finding the
/// run does not hold is a refusal, not a raise with an empty summary.
fn declare_finding(
    next: &mut DesignSnapshot,
    declaration: &Declaration,
) -> Result<Vec<Pending>, Refusal> {
    let id = declaration.subject();
    if let Some(held) = next.review.findings.iter_mut().find(|held| &held.id == id) {
        if let Some(resolution) = declaration.resolution() {
            held.resolution = Some(resolution.to_owned());
        }
        if let Some(summary) = declaration.summary() {
            summary.clone_into(&mut held.summary);
        }
        let section = held.subject.clone();
        let event = if declaration.resolution().is_some() {
            ChangeEvent::FindingDisposed
        } else {
            ChangeEvent::FindingRaised
        };
        return Ok(vec![Pending::about(
            event,
            id,
            vec![PayloadTerm::token(PayloadKey::Section, section.as_str())?],
        )]);
    }

    let Some(summary) = declaration.summary().filter(|text| !text.trim().is_empty()) else {
        return Err(Refusal::FindingSummaryMissing { id: id.clone() });
    };
    let Some(section) = declaration.concerns() else {
        return Err(Refusal::FindingSubjectMissing { id: id.clone() });
    };
    if next.sections.find(section).is_none() {
        return Err(Refusal::FindingSubjectMissing { id: id.clone() });
    }
    next.review.findings.push(Finding {
        id: id.clone(),
        subject: section.clone(),
        summary: summary.to_owned(),
        blocking: declaration.blocking(),
        resolution: declaration.resolution().map(str::to_owned),
    });
    Ok(vec![Pending::about(
        ChangeEvent::FindingRaised,
        id,
        vec![PayloadTerm::token(PayloadKey::Section, section.as_str())?],
    )])
}

/// Which of the four DEC-062 dispositions a checkpoint declaration names.
///
/// **One route** (EX-12). A `record`/`adopt_record` pair used to reach
/// [`Disposition::Adopted`] and `Created` from here too, saying *this already
/// happened* rather than *do this* — and it reached them without the adoption
/// validation `dispose` goes through in the shell, so a checkpoint could name a
/// record the corpus does not hold and be recorded as having adopted it. A
/// second, unvalidated spelling of one outcome is the defect; declaring none is
/// the refusal EX-7 asks for.
fn checkpoint_disposition(declaration: &Declaration) -> Result<Disposition, Refusal> {
    let id = declaration.subject();
    match declaration.dispose() {
        Some(Dispose::Create(_)) => declaration.resolved_record().map_or_else(
            || Err(Refusal::CheckpointRecordUnresolved { id: id.clone() }),
            |record| {
                Ok(Disposition::Created {
                    record: record.to_owned(),
                })
            },
        ),
        Some(Dispose::Adopt { record }) => Ok(Disposition::Adopted {
            record: record.clone(),
        }),
        Some(Dispose::Unresolved { note }) => {
            Ok(Disposition::RetainedUnresolved { note: note.clone() })
        }
        Some(Dispose::NonDurable { note }) => Ok(Disposition::NonDurable { note: note.clone() }),
        None => Err(Refusal::CheckpointDispositionMissing { id: id.clone() }),
    }
}

/// Dispose an inquiry through a checkpoint, and journal the recoverable intent.
fn declare_checkpoint(
    next: &mut DesignSnapshot,
    declaration: &Declaration,
    submission: &str,
) -> Result<Vec<Pending>, Refusal> {
    let id = declaration.subject();
    let Some(disposes) = declaration.disposes() else {
        return Err(Refusal::CheckpointIncomplete { id: id.clone() });
    };
    let disposition = checkpoint_disposition(declaration)?;
    if let Some(record) = disposition.record()
        && record.len() > DESIGN_ID_BYTES
    {
        return Err(Refusal::ValueTooLong {
            what: "canonical record reference",
            raw: record.to_owned(),
            limit: DESIGN_ID_BYTES,
        });
    }
    let Some(node) = next.map.inquiry.get(disposes).cloned() else {
        return Err(Refusal::UnknownNode {
            id: disposes.clone(),
        });
    };

    let mut terms = vec![PayloadTerm::token(PayloadKey::Node, disposes.as_str())?];
    // The record term is present only for the two forms that name one: a
    // note-bearing disposition produces no record, and rendering an empty one
    // would be inventing a fact the row does not hold.
    if let Some(record) = disposition.record() {
        terms.push(PayloadTerm::token(PayloadKey::Record, record)?);
    }
    terms.push(PayloadTerm::label(
        PayloadKey::Disposition,
        disposition.form().as_str(),
    )?);

    let subject = IntentSubject::Checkpoint(id.clone());
    let mut intent = RecoveryIntent::journalled(submission, subject.clone());
    if let Some(record) = disposition.record() {
        intent = intent.reserving(record);
    }
    // By the time a candidate is stored, DEC-086 steps 1..5 have landed and the
    // snapshot write IS step 6 — so the intent the snapshot carries is complete
    // by construction. The journal, which is the recovery artefact, records the
    // intermediate states.
    let intent = intent.reaching(IntentState::Complete);

    next.map.inquiry.insert(node.resolve(disposition))?;
    // Keyed by submission, not by checkpoint id: recovery resumes *the
    // submission's* first incomplete effect, and a retry of the same submission
    // must find its own intent rather than create a second one (DEC-086).
    next.checkpoint
        .intents
        .retain(|held| held.submission() != submission || held.subject() != &subject);
    next.checkpoint.intents.push(intent);
    Ok(vec![Pending::about(
        ChangeEvent::CheckpointDisposed,
        id,
        terms,
    )])
}

/// Apply a declared change of traversal direction (EX-8).
///
/// Every arm refuses rather than guesses: a pin or cursor naming a node the map
/// does not hold is [`Refusal::UnknownNode`], and an agent proposal against a
/// user-locked cursor is [`Refusal::CursorLocked`] — the refusal that is the
/// whole reason authority is a field rather than a convention.
fn direct_traversal(
    next: &mut DesignSnapshot,
    declared: &TraversalDeclaration,
) -> Result<(), Refusal> {
    let authority = declared.authority();

    match declared.pin.clone() {
        Sparse::Omitted => {}
        Sparse::Null => next.map.pin = None,
        Sparse::Value(at) => {
            if next.map.inquiry.get(&at).is_none() {
                return Err(Refusal::UnknownNode { id: at });
            }
            next.map.pin = Some(Pin { at, authority });
        }
    }

    match declared.cursor.clone() {
        Sparse::Omitted => {}
        Sparse::Null => next.map.cursor = Cursor::default(),
        Sparse::Value(at) => {
            if next.map.inquiry.get(&at).is_none() {
                return Err(Refusal::UnknownNode { id: at });
            }
            next.map.cursor = match authority {
                // An agent proposal must pass the locked-cursor guard; a user
                // may always move their own cursor.
                Authority::AgentProposed => next.map.cursor.propose(at)?,
                Authority::UserPinned | Authority::UserLocked => Cursor::placed(at, authority),
            };
        }
    }

    if let Some(posture) = declared.posture {
        next.map.posture = TraversalPosture::set(posture, authority);
    }
    Ok(())
}

/// A forward advance or a direct regression — one declaration, two verbs, chosen
/// by the target rather than by a flag the caller could set inconsistently.
fn stage_move(
    next: &mut DesignSnapshot,
    to: Stage,
    reason: Option<&str>,
    facts: Option<&RunbookFacts>,
    verifications: &[StepVerification],
) -> Result<Pending, Refusal> {
    let from = next.run.stage;
    let mut terms = vec![
        PayloadTerm::label(PayloadKey::From, from.as_str())?,
        PayloadTerm::label(PayloadKey::To, to.as_str())?,
    ];
    if to < from {
        let regression = gate::regress(from, to, reason.unwrap_or_default())?;
        // Stored whole, exactly as accepted: no bound of any kind applies to a
        // stored regression reason (EX-16(b)). The projection elides it.
        terms.push(PayloadTerm::prose(PayloadKey::Reason, regression.reason()));
        next.run.stage = regression.to();
    } else {
        let standing = next.review_standing();
        // Derived here rather than stored, and derived for THIS edge: a set of
        // facts loaded for a different edge answers a different question, so it
        // is discarded rather than trusted.
        let runbook = facts
            .filter(|facts| {
                gate::Advance::between(from, to).map(gate::boundary_runbook) == Some(facts.key)
            })
            .map(|facts| {
                facts
                    .book
                    .standing(&next.runbook.discharges, &facts.digests, verifications)
            });
        next.run.stage = gate::advance(from, to, &next.gate, standing, runbook.as_ref())?;
        if let Some(reason) = reason {
            terms.push(PayloadTerm::prose(PayloadKey::Reason, reason));
        }
    }
    Ok(Pending::run_wide(ChangeEvent::StageMoved, terms))
}

/// Admit one runbook discharge (SL-233 PHASE-16).
///
/// Shape first: a skip with nothing disclosed is refused before any question
/// about *which* step or *what* the runbook says, because a malformed
/// declaration is malformed in every context — the ordering
/// [`Refusal::AcceptanceBasisMissing`] already follows.
fn discharge_step(
    next: &mut DesignSnapshot,
    declared: &DischargeDeclaration,
    facts: Option<&RunbookFacts>,
    verifications: &[StepVerification],
) -> Result<Pending, Refusal> {
    if declared.outcome == DischargeClaim::Skipped
        && declared
            .reason
            .as_deref()
            .is_none_or(|reason| reason.trim().is_empty())
    {
        return Err(Refusal::DischargeReasonMissing {
            step: declared.step.clone(),
        });
    }
    let Some(facts) = facts else {
        return Err(Refusal::RunbookAbsent {
            stage: next.run.stage,
        });
    };

    // `sequence`: the cursor is the only admissible subject. Naming any other
    // step — ahead of the cursor, behind it, or absent from the runbook — is one
    // refusal, because they are one fact: this is not the obligation at hand.
    let standing = facts
        .book
        .standing(&next.runbook.discharges, &facts.digests, verifications);
    if standing.cursor.as_deref() != Some(declared.step.as_str()) {
        return Err(Refusal::DischargeNotAtCursor {
            step: declared.step.clone(),
            expected: standing.cursor,
        });
    }
    let (Some(step), Some(digest)) = (
        facts
            .book
            .steps()
            .iter()
            .find(|step| step.id() == declared.step),
        facts.digests.get(&declared.step),
    ) else {
        return Err(Refusal::RunbookAbsent {
            stage: next.run.stage,
        });
    };

    let revision = next.run.revision;
    let recorded = match declared.outcome {
        // A step carrying a check is never recorded on the claim alone: the
        // caller may say `attested`, and what that becomes — `verified`, or a
        // refusal — is decided here by what the check did (`EX-10`, D1). A step
        // with no check has nothing to corroborate it, so the claim stands.
        DischargeClaim::Attested => {
            if step.verify().is_none() {
                Discharge::attested(facts.key, &declared.step, digest, revision)
            } else {
                let checked = verifications
                    .iter()
                    .find(|result| result.step == declared.step)
                    .ok_or_else(|| Refusal::VerifierFailed {
                        step: declared.step.clone(),
                        exit: None,
                        // Fail closed, the rule the runbook clause on `advance`
                        // already follows: no result where a check applies
                        // leaves the step outstanding, never discharged.
                        output: String::new(),
                    })?;
                // Bind the OBSERVED code rather than re-stating the guard's
                // conclusion: the discharge record is the auditable trace of
                // what the verifier did, and widening this predicate to a set
                // of accepted codes must not leave it recording a literal 0.
                let Some(exit) = checked.exit.filter(|&code| code == 0) else {
                    return Err(Refusal::VerifierFailed {
                        step: declared.step.clone(),
                        exit: checked.exit,
                        output: checked.output.clone(),
                    });
                };
                Discharge::verified(
                    facts.key,
                    &declared.step,
                    digest,
                    revision,
                    exit,
                    checked.output.clone(),
                )
            }
        }
        DischargeClaim::Skipped => Discharge::skipped(
            facts.key,
            &declared.step,
            digest,
            revision,
            declared.reason.clone().unwrap_or_default(),
        ),
    };
    next.runbook.upsert(recorded);
    Ok(Pending::run_wide(
        ChangeEvent::StepDischarged,
        vec![
            PayloadTerm::token(PayloadKey::Step, &declared.step)?,
            PayloadTerm::label(PayloadKey::Outcome, outcome_label(declared.outcome))?,
        ],
    ))
}

/// The wire label a discharge outcome is recorded under in the change log.
const fn outcome_label(claim: DischargeClaim) -> &'static str {
    match claim {
        DischargeClaim::Attested => "attested",
        DischargeClaim::Skipped => "skipped",
    }
}

/// Re-observe every section's current fingerprint, so DEC-066 liveness is
/// evaluated against what this batch left behind.
fn reobserve(next: &mut DesignSnapshot) {
    let fingerprints = next.sections.fingerprints();
    let mut facts = std::mem::take(&mut next.gate);
    for (id, fingerprint) in fingerprints {
        facts = facts.observe(id, fingerprint);
    }
    next.gate = facts;
}

/// The current fingerprint of a subject, for binding a clearance to it.
fn current_fingerprint(next: &DesignSnapshot, subject: &DesignId) -> Result<Fingerprint, Refusal> {
    next.sections
        .find(subject)
        .map(|section| section.fingerprint.clone())
        .ok_or_else(|| Refusal::UnknownNode {
            id: subject.clone(),
        })
}

/// Every clearance currently bound to live content, as a comparable key.
fn live_evidence(facts: &DerivedDesignFacts) -> BTreeSet<(gate::Condition, DesignId, Fingerprint)> {
    facts
        .live_evidence()
        .map(|evidence| {
            (
                evidence.condition(),
                evidence.subject().clone(),
                evidence.fingerprint().clone(),
            )
        })
        .collect()
}

/// Every attestation still bound to its section's current content.
///
/// **Deliberately not policy-filtered** (design § *Where the policy is read, and
/// where it is not*). This is the third reader of the attestation set and the one
/// that must not take the run's [`ReviewPolicy`]: it feeds [`invalidation_rows`],
/// which reports the *death of a recorded act*. An adversarial attestation going
/// stale is a fact whatever lanes the run currently requires, and filtering here
/// would silently stop reporting it. A sweep for "readers of `attestations`" gets
/// this one wrong, which is why the exclusion is written down rather than left to
/// be re-derived.
///
/// `pub(super)` for the test that pins it — the sweep above is the failure mode,
/// so the seam is asserted directly rather than only through a row it feeds.
///
/// [`ReviewPolicy`]: super::attestation::ReviewPolicy
pub(super) fn live_reviews(
    snapshot: &DesignSnapshot,
) -> BTreeSet<(DesignId, DesignId, Fingerprint)> {
    snapshot
        .review
        .attestations
        .iter()
        .filter(|attestation| {
            snapshot
                .sections
                .find(attestation.subject())
                .is_some_and(|section| &section.fingerprint == attestation.fingerprint())
        })
        .map(|attestation| {
            (
                attestation.id().clone(),
                attestation.subject().clone(),
                attestation.fingerprint().clone(),
            )
        })
        .collect()
}

/// The rows for evidence and attestations that stopped being live during this
/// apply — derived from the before/after difference rather than recorded by
/// whoever changed the content, so a new way of moving a fingerprint cannot
/// forget to report what it killed.
fn invalidation_rows(
    evidence_before: &BTreeSet<(gate::Condition, DesignId, Fingerprint)>,
    evidence_after: &BTreeSet<(gate::Condition, DesignId, Fingerprint)>,
    reviews_before: &BTreeSet<(DesignId, DesignId, Fingerprint)>,
    reviews_after: &BTreeSet<(DesignId, DesignId, Fingerprint)>,
) -> Result<Vec<Pending>, Refusal> {
    let mut rows: Vec<Pending> = Vec::new();
    for (condition, subject, fingerprint) in evidence_before.difference(evidence_after) {
        rows.push(Pending::about(
            ChangeEvent::EvidenceInvalidated,
            subject,
            vec![
                PayloadTerm::token(PayloadKey::Gate, condition.as_str())?,
                PayloadTerm::digest(PayloadKey::Fingerprint, fingerprint.as_str()),
            ],
        ));
    }
    for (attestation, subject, _fingerprint) in reviews_before.difference(reviews_after) {
        rows.push(Pending::about(
            ChangeEvent::ReviewInvalidated,
            attestation,
            vec![
                PayloadTerm::token(PayloadKey::Section, subject.as_str())?,
                PayloadTerm::token(PayloadKey::Attestation, attestation.as_str())?,
            ],
        ));
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::super::submission::EvidenceDeclaration;
    use super::*;

    /// A well-formed id, or a failure naming the bad literal.
    fn id(raw: &str) -> DesignId {
        DesignId::parse(raw).expect("test fixture id must be well-formed")
    }

    /// A run holding one section, at revision 1.
    fn run_with_a_section() -> DesignSnapshot {
        let mut snapshot = DesignSnapshot::new("dr-test", 233, None);
        snapshot.sections.upsert(Section {
            id: id("sec-a"),
            title: "sec-a".to_owned(),
            body: "## sec-a\n".to_owned(),
            fingerprint: Fingerprint::new("sha256:a"),
            seq: 0,
            source_line: None,
        });
        snapshot
    }

    /// `EX-9` — an entry with no shell-supplied digest is REFUSED, never seeded
    /// with a placeholder.
    ///
    /// The map is total by construction (both sides run the same parse over the
    /// same regions), so this pins the contract rather than a reachable path:
    /// the failure mode worth foreclosing is a node that carries an empty
    /// fingerprint and reads as though it had one, which is the shape of the
    /// defect `EX-9` closes.
    #[test]
    fn an_entry_without_a_supplied_digest_is_refused_rather_than_seeded_empty() {
        let document = "## Open Questions\n\n- **OQ-1:** does the digest arrive?\n";
        let regions: Vec<(super::super::legacy::Region<'_>, Fingerprint)> =
            super::super::legacy::read(document)
                .expect("the fixture document decomposes")
                .into_iter()
                .map(|region| (region, Fingerprint::new("sha256:section")))
                .collect();

        let mut run = DesignSnapshot::new("dr-test", 233, None);
        assert_eq!(
            import(&mut run, &regions, &[], &BTreeMap::new()),
            Err(Refusal::ImportedEntryDigestMissing { line: 3 }),
            "an OQ entry the shell did not digest refuses the whole import"
        );

        // The positive control: with the digest supplied, the SAME fixture
        // imports and the node carries it — so the refusal above is the missing
        // digest, not a fixture that could never import at all.
        let mut run = DesignSnapshot::new("dr-test", 233, None);
        let digests = BTreeMap::from([(3, Fingerprint::new("sha256:headline"))]);
        import(&mut run, &regions, &[], &digests).expect("the fixture imports once digested");
        let seeded: Vec<&Provenance> = run
            .map
            .inquiry
            .nodes()
            .map(InquiryNode::provenance)
            .collect();
        assert!(
            matches!(
                seeded.as_slice(),
                [Provenance::ImportedProse { label, fingerprint, .. }]
                    if label == "OQ-1" && fingerprint.as_str() == "sha256:headline"
            ),
            "the node carries the label and the supplied digest: {seeded:?}"
        );
    }

    /// One `apply` payload claiming `condition` against `sec-a`.
    fn claiming(prior: &DesignSnapshot, condition: gate::Condition) -> ApplyRequest {
        ApplyRequest {
            envelope: SubmissionEnvelope {
                run_uid: prior.run.uid.clone(),
                known_revision: prior.run.revision,
                submission_id: "s1".to_owned(),
            },
            adopt_authored: None,
            traversal: TraversalDeclaration::default(),
            stage: None,
            acceptance: None,
            evidence: vec![EvidenceDeclaration {
                condition,
                subject: id("sec-a"),
            }],
            declare: Vec::new(),
            delegation: None,
            discharge: None,
            review_policy: None,
            checkpoint_act: None,
            agent_declaration: None,
        }
    }

    #[test]
    fn a_derived_condition_cannot_be_claimed_as_evidence() {
        let prior = run_with_a_section();
        let derived = DerivedInput::default();

        // The self-claim bypass, closed at admission: without this refusal a
        // caller could lock a design it never reviewed by asserting that it had.
        let derived_conditions = gate::Condition::ALL
            .into_iter()
            .filter(|condition| condition.is_derived());
        for condition in derived_conditions {
            assert_eq!(
                apply(
                    &prior,
                    &claiming(&prior, condition),
                    &derived,
                    "sha256:pay",
                    &Resolution::default(),
                ),
                Err(Refusal::DerivedConditionClaimed { condition }),
                "{} is derived and must not be claimable",
                condition.as_str()
            );
        }

        // The positive control: the refusal is targeted, not a blanket ban on
        // claiming clearance. A claimed condition Doctrine cannot derive still
        // lands.
        let applied = apply(
            &prior,
            &claiming(&prior, gate::Condition::RequiredSectionsExist),
            &derived,
            "sha256:pay",
            &Resolution::default(),
        )
        .expect("a claimed condition is still evidence");
        assert!(
            applied
                .snapshot
                .gate
                .satisfies(gate::Condition::RequiredSectionsExist)
        );
    }
}
