// SPDX-License-Identifier: GPL-3.0-only
//! The per-revision change log — **storage** (SL-233 PHASE-03 EX-13/EX-14(a)).
//!
//! Snapshots are atomically replaced, not retained, so there is no historical
//! snapshot to diff: the material-change delta must be *recorded*, not computed
//! (projection-bounds sketch §(d)). Each row is therefore **self-contained** —
//! it carries everything its rendering needs without consulting history.
//!
//! **The stored row and the rendered row are different artefacts.** This module
//! is storage: every term is kept at full fidelity — the whole regression reason
//! exactly as accepted, the whole digest. No `ENVELOPE_*` constant applies here,
//! and by construction none can be named from this module ([`super::render`]
//! keeps them private).
//!
//! What the renderer needs in order to treat a term correctly is not the term's
//! *key* but its **value kind**: identity and closed vocabulary render whole,
//! digests abbreviate, and only prose elides. [`PayloadValue`] carries that
//! distinction in the type, so the layer rule's admission half ("identity is
//! never truncated at emission") is a property of the data rather than a rule
//! the renderer has to remember.

use serde::{Deserialize, Serialize};

use super::bounds::{
    CHANGE_LOG_REVISIONS, DESIGN_EVENT_NAME_BYTES, DESIGN_ID_BYTES, DESIGN_STAGE_LABEL_BYTES,
};
use super::ids::DesignId;
use super::refusal::Refusal;

/// The widest member of a closed vocabulary, at compile time.
const fn widest(rest: &[ChangeEvent]) -> usize {
    match rest {
        [] => 0,
        [head, tail @ ..] => {
            let head = head.as_str().len();
            let tail = widest(tail);
            if head > tail { head } else { tail }
        }
    }
}

/// The provenance of [`DESIGN_EVENT_NAME_BYTES`], **proved rather than
/// asserted** (EX-16(a)): the event vocabulary is closed, so the bound is
/// derivable from it, and a new event name that outgrew the bound would stop the
/// build rather than quietly widen a rendered row.
const _: () = assert!(widest(&ChangeEvent::ALL) <= DESIGN_EVENT_NAME_BYTES);

/// The closed material-change vocabulary (projection-bounds sketch §(d) table).
///
/// Closed on purpose: the containment check enumerates it, so a new event kind
/// that nobody sized cannot slip past a hand-picked example. Cursor moves,
/// posture changes, receipt eviction and fragment receipts are deliberately
/// **not** members — they are state, not delta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ChangeEvent {
    NodeCreated,
    NodeLifecycle,
    NodeReparented,
    NeedsAdded,
    NeedsRemoved,
    StageMoved,
    EvidenceInvalidated,
    SectionCreated,
    SectionFingerprintChanged,
    ReviewAttested,
    ReviewInvalidated,
    IntegratedReviewRecorded,
    FindingRaised,
    FindingDisposed,
    AcceptanceAttested,
    ReviewPolicyChanged,
    CheckpointDisposed,
    ObligationDelegated,
    ProposalRecorded,
    ProposalAccepted,
    ProposalRefused,
    /// A runbook step was discharged (SL-233 PHASE-16).
    StepDischarged,
}

impl ChangeEvent {
    /// Every event, in the sketch's declaration order — the closed vocabulary,
    /// single-sourced so an exhaustive table test cannot silently miss a variant
    /// (STD-001).
    pub(crate) const ALL: [ChangeEvent; 22] = [
        ChangeEvent::NodeCreated,
        ChangeEvent::NodeLifecycle,
        ChangeEvent::NodeReparented,
        ChangeEvent::NeedsAdded,
        ChangeEvent::NeedsRemoved,
        ChangeEvent::StageMoved,
        ChangeEvent::EvidenceInvalidated,
        ChangeEvent::SectionCreated,
        ChangeEvent::SectionFingerprintChanged,
        ChangeEvent::ReviewAttested,
        ChangeEvent::ReviewInvalidated,
        ChangeEvent::IntegratedReviewRecorded,
        ChangeEvent::FindingRaised,
        ChangeEvent::FindingDisposed,
        ChangeEvent::AcceptanceAttested,
        ChangeEvent::ReviewPolicyChanged,
        ChangeEvent::CheckpointDisposed,
        ChangeEvent::ObligationDelegated,
        ChangeEvent::ProposalRecorded,
        ChangeEvent::ProposalAccepted,
        ChangeEvent::ProposalRefused,
        ChangeEvent::StepDischarged,
    ];

    /// The token this event is spelled with everywhere — stored value, rendered
    /// name (STD-001). Bounded at admission by
    /// [`super::bounds::DESIGN_EVENT_NAME_BYTES`]: the vocabulary is closed, so
    /// membership *is* the admission check.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            ChangeEvent::NodeCreated => "node_created",
            ChangeEvent::NodeLifecycle => "node_lifecycle",
            ChangeEvent::NodeReparented => "node_reparented",
            ChangeEvent::NeedsAdded => "needs_added",
            ChangeEvent::NeedsRemoved => "needs_removed",
            ChangeEvent::StageMoved => "stage_moved",
            ChangeEvent::EvidenceInvalidated => "evidence_invalidated",
            ChangeEvent::SectionCreated => "section_created",
            ChangeEvent::SectionFingerprintChanged => "section_fingerprint_changed",
            ChangeEvent::ReviewAttested => "review_attested",
            ChangeEvent::ReviewInvalidated => "review_invalidated",
            ChangeEvent::IntegratedReviewRecorded => "integrated_review_recorded",
            ChangeEvent::FindingRaised => "finding_raised",
            ChangeEvent::FindingDisposed => "finding_disposed",
            ChangeEvent::AcceptanceAttested => "acceptance_attested",
            ChangeEvent::ReviewPolicyChanged => "review_policy_changed",
            ChangeEvent::CheckpointDisposed => "checkpoint_disposed",
            ChangeEvent::ObligationDelegated => "obligation_delegated",
            ChangeEvent::ProposalRecorded => "proposal_recorded",
            ChangeEvent::ProposalAccepted => "proposal_accepted",
            ChangeEvent::ProposalRefused => "proposal_refused",
            ChangeEvent::StepDischarged => "step_discharged",
        }
    }

    /// The payload shape this event carries: its terms, in order, each with the
    /// **value kind** that decides how emission may treat it.
    ///
    /// The sketch's §(d) table, in code, and the single source for two things
    /// that would otherwise drift apart: the canonical term order a row renders
    /// in ([`ChangeEvent::ordered`]), and the saturation the containment check
    /// applies to prove the budget holds for every member.
    pub(crate) const fn payload_terms(self) -> &'static [(PayloadKey, ValueKind)] {
        match self {
            ChangeEvent::NodeCreated => &[
                (PayloadKey::Parent, ValueKind::Token),
                (PayloadKey::Provenance, ValueKind::Label),
            ],
            ChangeEvent::NodeLifecycle => &[
                (PayloadKey::From, ValueKind::Label),
                (PayloadKey::To, ValueKind::Label),
            ],
            ChangeEvent::NeedsAdded | ChangeEvent::NeedsRemoved => &[
                (PayloadKey::From, ValueKind::Token),
                (PayloadKey::To, ValueKind::Token),
            ],
            ChangeEvent::NodeReparented => &[
                (PayloadKey::Old, ValueKind::Token),
                (PayloadKey::New, ValueKind::Token),
            ],
            ChangeEvent::SectionFingerprintChanged => &[
                (PayloadKey::Old, ValueKind::Digest),
                (PayloadKey::New, ValueKind::Digest),
            ],
            ChangeEvent::StageMoved => &[
                (PayloadKey::From, ValueKind::Label),
                (PayloadKey::To, ValueKind::Label),
                (PayloadKey::Reason, ValueKind::Prose),
            ],
            ChangeEvent::EvidenceInvalidated => &[
                (PayloadKey::Gate, ValueKind::Token),
                (PayloadKey::Fingerprint, ValueKind::Digest),
            ],
            ChangeEvent::SectionCreated => &[(PayloadKey::Fingerprint, ValueKind::Digest)],
            ChangeEvent::ReviewAttested | ChangeEvent::ReviewInvalidated => &[
                (PayloadKey::Section, ValueKind::Token),
                (PayloadKey::Attestation, ValueKind::Token),
            ],
            ChangeEvent::FindingRaised | ChangeEvent::FindingDisposed => {
                &[(PayloadKey::Section, ValueKind::Token)]
            }
            // Run-wide and term-free. The integrated pass is identified by its
            // own subject id, and an acceptance has no run-local id at all — its
            // basis and authority are snapshot state, not delta.
            ChangeEvent::IntegratedReviewRecorded | ChangeEvent::AcceptanceAttested => &[],
            // Both policies are closed tokens rendered by name, so the row reads
            // as the change it is — `human-only → adversarial-only` — rather than
            // requiring the reader to fetch the run to learn what moved.
            ChangeEvent::ReviewPolicyChanged => &[
                (PayloadKey::Old, ValueKind::Label),
                (PayloadKey::New, ValueKind::Label),
            ],
            ChangeEvent::CheckpointDisposed => &[
                (PayloadKey::Node, ValueKind::Token),
                (PayloadKey::Record, ValueKind::Token),
                (PayloadKey::Disposition, ValueKind::Label),
            ],
            // Every delegation row is *about the assignment* — the subject is the
            // `dlg-` id and the obligation rides a term. That keeps "rows about
            // this node" free of delegation bookkeeping, which matters because a
            // proposal's currency is a question about the obligation and not
            // about the assignment.
            ChangeEvent::ObligationDelegated => &[(PayloadKey::Node, ValueKind::Token)],
            // Attribution is unverified free text, so it is **prose**: no
            // admission bound is derivable for it, and the layer rule says an
            // underivable bound is not invented. It elides at emission like any
            // other prose and is stored whole.
            ChangeEvent::ProposalRecorded | ChangeEvent::ProposalAccepted => &[
                (PayloadKey::Node, ValueKind::Token),
                (PayloadKey::By, ValueKind::Prose),
            ],
            ChangeEvent::ProposalRefused => &[
                (PayloadKey::Node, ValueKind::Token),
                (PayloadKey::Reason, ValueKind::Prose),
            ],
            // The step id and the outcome are both closed tokens; the skip
            // reason is deliberately NOT a term. It is stored whole on the
            // discharge record, and the change log is a bounded rendering.
            ChangeEvent::StepDischarged => &[
                (PayloadKey::Step, ValueKind::Token),
                (PayloadKey::Outcome, ValueKind::Token),
            ],
        }
    }

    /// Put `terms` into this event's declared order.
    ///
    /// A row's terms are built by whichever branch noticed the change, so
    /// without this the rendered order would follow construction rather than the
    /// documented shape — and a reader comparing two rows of the same kind would
    /// have to allow for both.
    pub(crate) fn ordered(self, mut terms: Vec<PayloadTerm>) -> Vec<PayloadTerm> {
        let shape = self.payload_terms();
        terms.sort_by_key(|term| {
            shape
                .iter()
                .position(|(key, _)| *key == term.key())
                .unwrap_or(usize::MAX)
        });
        terms
    }
}

/// The closed key vocabulary a payload term is named by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PayloadKey {
    Parent,
    Provenance,
    From,
    To,
    Old,
    New,
    Reason,
    Gate,
    Fingerprint,
    Section,
    Attestation,
    Node,
    Record,
    Disposition,
    By,
    /// Which runbook step a discharge named (SL-233 PHASE-16).
    Step,
    /// What the discharge concluded.
    Outcome,
}

impl PayloadKey {
    /// The token this key renders as, left of the `=`.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            PayloadKey::Parent => "parent",
            PayloadKey::Provenance => "provenance",
            PayloadKey::From => "from",
            PayloadKey::To => "to",
            PayloadKey::Old => "old",
            PayloadKey::New => "new",
            PayloadKey::Reason => "reason",
            PayloadKey::Gate => "gate",
            PayloadKey::Fingerprint => "fingerprint",
            PayloadKey::Section => "section",
            PayloadKey::Attestation => "attestation",
            PayloadKey::Node => "node",
            PayloadKey::Record => "record",
            PayloadKey::Step => "step",
            PayloadKey::Outcome => "outcome",
            PayloadKey::Disposition => "disposition",
            PayloadKey::By => "by",
        }
    }
}

/// What *kind* of value a payload term holds — and therefore how emission may
/// treat it.
///
/// This is the layer rule expressed in the type rather than in a convention:
/// [`ValueKind::Token`] and [`ValueKind::Label`] are identity and closed
/// vocabulary, bounded at admission and rendered **whole**;
/// [`ValueKind::Digest`] is an abbreviation with a stated collision budget; only
/// [`ValueKind::Prose`] degrades gracefully and may be elided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ValueKind {
    /// Identity: a run-local id, a gate id, a canonical record ref. Bounded at
    /// admission by [`super::bounds::DESIGN_ID_BYTES`]; never truncated.
    Token,
    /// A closed-vocabulary label: stage name, provenance, lifecycle, disposition
    /// form. Bounded at admission by [`super::bounds::DESIGN_STAGE_LABEL_BYTES`];
    /// never truncated.
    Label,
    /// A content fingerprint, stored whole and abbreviated when rendered.
    Digest,
    /// Gracefully degrading prose — a regression reason. **Stored with no bound
    /// at all** (EX-16(b)); elided only at render time.
    Prose,
}

impl ValueKind {
    /// What a value of this kind is *admitted* against, and the noun a refusal
    /// names it by. `None` for the two kinds no admission bound can be derived
    /// for: a digest is abbreviated at emission and stored whole, and a stored
    /// regression reason is deliberately unbounded (EX-16(b), provenance rule).
    const fn admission_bound(self) -> Option<(&'static str, usize)> {
        match self {
            // Identity rides the id slot of a change payload, so it is bounded
            // by the same constant the row arithmetic is derived from.
            ValueKind::Token => Some(("payload identity term", DESIGN_ID_BYTES)),
            // A label is a closed-vocabulary token — stage, lifecycle,
            // provenance, disposition form — and shares the stage-label bound.
            ValueKind::Label => Some(("payload label term", DESIGN_STAGE_LABEL_BYTES)),
            ValueKind::Digest | ValueKind::Prose => None,
        }
    }
}

/// One `key=value` term of a stored payload, kept at full fidelity.
///
/// # Admission by construction (RV-321 F-1)
///
/// The fields are **private** and [`PayloadTerm::admit`] is the only route to a
/// value — including the deserialization route, which enters through
/// `try_from = "PayloadTermWire"` rather than a derived `Deserialize`. That
/// second route is the one that matters: guarding only the constructors leaves
/// a hand-edited or corrupt snapshot free to claim an arbitrarily long value is
/// a `Token`, which breaches the 160-byte rendered-payload premise the budgeted
/// projection load-bears. EX-16(d)'s rationale generalises exactly: per-path
/// checks are insufficient because it is the *unenumerated* route that carries
/// the defect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "PayloadTermWire")]
pub(crate) struct PayloadTerm {
    key: PayloadKey,
    kind: ValueKind,
    value: String,
}

/// The wire shape a stored term deserialises through, so re-entry is an
/// admission point rather than a bypass.
#[derive(Deserialize)]
struct PayloadTermWire {
    key: PayloadKey,
    kind: ValueKind,
    value: String,
}

impl TryFrom<PayloadTermWire> for PayloadTerm {
    type Error = Refusal;

    fn try_from(wire: PayloadTermWire) -> Result<PayloadTerm, Refusal> {
        PayloadTerm::admit(wire.key, wire.kind, wire.value)
    }
}

impl PayloadTerm {
    /// The one validating constructor. Refuses a value over the admission bound
    /// its [`ValueKind`] carries — a **refusal, never a trim**, because a
    /// truncated identity is a *wrong* identity rather than a shorter one.
    fn admit(key: PayloadKey, kind: ValueKind, value: String) -> Result<PayloadTerm, Refusal> {
        if let Some((what, limit)) = kind.admission_bound()
            && value.len() > limit
        {
            return Err(Refusal::ValueTooLong {
                what,
                raw: value,
                limit,
            });
        }
        Ok(PayloadTerm { key, kind, value })
    }

    /// An identity term — rendered whole, bounded at admission by
    /// [`super::bounds::DESIGN_ID_BYTES`].
    pub(crate) fn token(key: PayloadKey, value: impl Into<String>) -> Result<Self, Refusal> {
        PayloadTerm::admit(key, ValueKind::Token, value.into())
    }

    /// A closed-vocabulary label term — rendered whole, bounded at admission by
    /// [`super::bounds::DESIGN_STAGE_LABEL_BYTES`].
    pub(crate) fn label(key: PayloadKey, value: impl Into<String>) -> Result<Self, Refusal> {
        PayloadTerm::admit(key, ValueKind::Label, value.into())
    }

    /// A fingerprint term — stored whole, abbreviated when rendered. No
    /// admission bound is derivable for a digest, so none is invented.
    pub(crate) fn digest(key: PayloadKey, value: impl Into<String>) -> Self {
        PayloadTerm {
            key,
            kind: ValueKind::Digest,
            value: value.into(),
        }
    }

    /// A prose term — stored whole and unbounded, elided when rendered.
    pub(crate) fn prose(key: PayloadKey, value: impl Into<String>) -> Self {
        PayloadTerm {
            key,
            kind: ValueKind::Prose,
            value: value.into(),
        }
    }

    /// The key this term is named by.
    pub(crate) const fn key(&self) -> PayloadKey {
        self.key
    }

    /// What kind of value it holds — and therefore how emission may treat it.
    pub(crate) const fn kind(&self) -> ValueKind {
        self.kind
    }

    /// The value, at full stored fidelity.
    pub(crate) fn value(&self) -> &str {
        &self.value
    }
}

/// One recorded material change, self-contained (sketch §(d)).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ChangeRow {
    /// The revision that produced this event.
    pub(crate) revision: u64,
    /// Position within that revision, assigned in **validated-candidate**
    /// serialisation order — a deterministic function of the declaration set,
    /// never of submission order (DEC-063 makes one apply an unordered batch).
    pub(crate) index: u32,
    pub(crate) event: ChangeEvent,
    /// The primary subject. `None` for a run-wide event — a stage move is about
    /// the run, which has no run-local id, and inventing another [`IdKind`] to
    /// give it one would widen the closed vocabulary every id test enumerates.
    ///
    /// [`IdKind`]: super::ids::IdKind
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) subject: Option<DesignId>,
    #[serde(default, rename = "term")]
    pub(crate) terms: Vec<PayloadTerm>,
}

/// The change-log snapshot group: an explicitly recorded floor plus the retained
/// rows.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ChangeLog {
    /// The oldest revision the log still covers, recorded **explicitly** and
    /// never inferred from the oldest surviving row: inference breaks when
    /// intervening revisions produced no material rows, and would report a
    /// complete delta as unavailable.
    pub(crate) floor: u64,
    #[serde(default, rename = "row")]
    pub(crate) rows: Vec<ChangeRow>,
}

impl ChangeLog {
    /// Append `rows` produced by `revision`, then evict past the retention
    /// window and advance the floor.
    pub(crate) fn record(&mut self, revision: u64, rows: Vec<ChangeRow>) {
        self.rows.extend(rows);
        self.retain_window(revision);
    }

    /// Drop rows below the retention window and record the new floor.
    ///
    /// The floor is computed from the *current revision and the window*, not
    /// from what survived — that is the whole point of recording it.
    fn retain_window(&mut self, current_revision: u64) {
        let floor = current_revision.saturating_sub(CHANGE_LOG_REVISIONS) + 1;
        if floor > self.floor {
            self.floor = floor;
        }
        let keep = self.floor;
        self.rows.retain(|row| row.revision >= keep);
    }

    /// Whether a delta from `known_revision` can be answered completely.
    ///
    /// `known_revision < floor` is **unavailable**, which is a different fact
    /// from an empty delta: "nothing changed" and "I cannot tell you what
    /// changed" are opposite answers (design R2).
    pub(crate) const fn covers(&self, known_revision: u64) -> bool {
        known_revision >= self.floor
    }

    /// The rows in the half-open range `(known_revision, current]`, newest last.
    pub(crate) fn since(&self, known_revision: u64) -> Vec<&ChangeRow> {
        let mut rows: Vec<&ChangeRow> = self
            .rows
            .iter()
            .filter(|row| row.revision > known_revision)
            .collect();
        rows.sort_by_key(|row| (row.revision, row.index));
        rows
    }
}
