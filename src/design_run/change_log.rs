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

/// The one wire token no member of the vocabulary spells — `T11`'s pre-rename
/// spelling of [`ChangeEvent::ActInvalidated`], whose doc carries why it is still
/// read.
///
/// Written here once and read once, by [`ChangeEvent::try_from`] (STD-001). It is
/// the only token in this module not spelled by [`ChangeEvent::as_str`], and it
/// can be without reintroducing the drift that single-sourcing removed: it spells
/// a **retired** token, so there is no live `as_str` arm for it to disagree with.
const LEGACY_ACT_INVALIDATED: &str = "evidence_invalidated";

/// The three labels a [`ChangeEvent::NodeBlockingChanged`] row renders its two
/// terms from — a closed three-member vocabulary, because a node's stored
/// judgement has three states and the row has to be able to say each of them
/// (STD-001: named here rather than spelled at the row's construction site).
///
/// Three rather than two is the whole reason the terms are `label`s: a legacy
/// node carries *no* judgement, so a row reporting `unjudged → non-blocking` is
/// a true reading of a real mutation, and rendering it as `blocking →
/// non-blocking` (the complement of the new value) would report a flip that did
/// not happen.
const JUDGEMENT_UNJUDGED: &str = "unjudged";
const JUDGEMENT_BLOCKING: &str = "blocking";
const JUDGEMENT_NON_BLOCKING: &str = "non-blocking";

/// The label for a node's stored blocking judgement, as its row renders it.
///
/// `None` is **unjudged** — the state only a node preexisting the attribute can
/// hold (`SL-264` sec-3).
pub(crate) const fn judgement_label(blocking: Option<bool>) -> &'static str {
    match blocking {
        None => JUDGEMENT_UNJUDGED,
        Some(true) => JUDGEMENT_BLOCKING,
        Some(false) => JUDGEMENT_NON_BLOCKING,
    }
}

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

/// Whether two tokens are the same string, at compile time. `PartialEq` is not
/// `const`, so the bytes are walked in `widest`'s slice-recursion idiom — which
/// is also what keeps this clear of `clippy::indexing_slicing`.
const fn same_token(left: &[u8], right: &[u8]) -> bool {
    match (left, right) {
        ([], []) => true,
        ([left_head, left_tail @ ..], [right_head, right_tail @ ..]) => {
            *left_head == *right_head && same_token(left_tail, right_tail)
        }
        _ => false,
    }
}

/// Whether a roster holds an event spelling `needle`'s token.
const fn holds_token(roster: &[ChangeEvent], needle: ChangeEvent) -> bool {
    match roster {
        [] => false,
        [head, tail @ ..] => {
            same_token(head.as_str().as_bytes(), needle.as_str().as_bytes())
                || holds_token(tail, needle)
        }
    }
}

/// Whether every member of `subset` is also spelled by a member of `superset`.
///
/// Compared by [`ChangeEvent::as_str`] because the token is the event's identity
/// everywhere else (STD-001). Two shorter spellings are closed off by the
/// workspace lint gate and are named here so nobody re-derives them: comparing
/// discriminants trips `clippy::as_conversions`, and deriving one roster from the
/// other by a const-block copy trips `clippy::indexing_slicing`.
const fn is_subset(subset: &[ChangeEvent], superset: &[ChangeEvent]) -> bool {
    match subset {
        [] => true,
        [head, tail @ ..] => holds_token(superset, *head) && is_subset(tail, superset),
    }
}

/// The provenance of [`DESIGN_EVENT_NAME_BYTES`], **proved rather than
/// asserted** (EX-16(a)): the event vocabulary is closed, so the bound is
/// derivable from it, and a new event name that outgrew the bound would stop the
/// build rather than quietly widen a rendered row.
///
/// Quantified over [`ChangeEvent::READABLE`] (SL-256 `EX-2`): a historical row
/// must stay renderable within the same budget, so the bound is owed by every
/// event a snapshot may *contain*, not only by those this binary may write.
const _: () = assert!(widest(&ChangeEvent::READABLE) <= DESIGN_EVENT_NAME_BYTES);

/// `sec-2`'s roster relation — everything this binary may write is something a
/// snapshot may contain — **proved rather than asserted at runtime** (`DEC-239`),
/// in the same idiom as the bound above.
///
/// It is also load-bearing against the lint gate, which is why it must not later
/// be deleted as redundant: this is the only reader of
/// [`ChangeEvent::EMITTABLE`] in `src/`, both roster tests living in
/// `tests/e2e_design_state.rs`, a separate compilation unit. The module's
/// dead-code exemption is `not(test)`-scoped and the crate denies `unused`, so
/// without this line `cargo check` passes and `cargo test --bin doctrine` fails
/// to compile. No attribute substitutes: `cfg(test)` holds in **both** units, so
/// a `cfg_attr(test, expect(dead_code, …))` is unfulfilled in the e2e unit and
/// stops that build instead.
const _: () = assert!(is_subset(&ChangeEvent::EMITTABLE, &ChangeEvent::READABLE));

/// The closed material-change vocabulary (projection-bounds sketch §(d) table).
///
/// Closed on purpose: the containment check enumerates it, so a new event kind
/// that nobody sized cannot slip past a hand-picked example. Cursor moves,
/// posture changes, receipt eviction and fragment receipts are deliberately
/// **not** members — they are state, not delta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub(crate) enum ChangeEvent {
    NodeCreated,
    NodeLifecycle,
    NodeReparented,
    /// A node's blocking judgement moved (`SL-264` sec-3, `REQ-478`).
    ///
    /// Emitted by the **declaration** that flips it, and by nothing else: the
    /// judgement is node state rather than a recorded act, so there is no
    /// free-standing declaration whose recording this mirrors. A creation carries
    /// its judgement inside [`ChangeEvent::NodeCreated`] and owes no second row —
    /// the node did not change what it was born with.
    ///
    /// The row carries both judgements changed, because it is self-contained by
    /// construction (a snapshot keeps no history to diff) and *which way* the mark
    /// moved is what the next reader of the feed is asking.
    NodeBlockingChanged,
    NeedsAdded,
    NeedsRemoved,
    StageMoved,
    /// The run now holds this act.
    ///
    /// The mirror of [`ChangeEvent::ActInvalidated`]: same subject convention,
    /// same single `act` term. Emitted **explicitly** at the point of record
    /// rather than derived, because a recording is an occurrence and no
    /// before/after difference over `live_acts` can see one — for a slot already
    /// holding `v`, *no operation* and *record the same value* both read
    /// `before = v, after = v` (`SL-256` `DEC-238`).
    ActRecorded,
    /// A recorded act stopped being bound to the content it was given over.
    ///
    /// **Renamed from `EvidenceInvalidated` at `T11`** (`EX-11`, `F1`), when the
    /// feed re-sourced from the retired evidence set to the act set. The alias is
    /// not optional: `ChangeEvent` deserialises **strictly**, so one unrecognised
    /// `event` fails the whole snapshot rather than one row, and the change log is
    /// append-only *history* — the vocabulary a run writes is not the vocabulary
    /// it must read. Eight rows on this repo's own live `SL-244` run carry the
    /// old token; dropping the alias is how `ISS-315` happened.
    ///
    /// The alias is no longer a serde attribute. `SL-256` made [`ChangeEvent::as_str`]
    /// the token's only source, so the retired spelling is resolved by
    /// [`ChangeEvent::try_from`] against [`LEGACY_ACT_INVALIDATED`] instead — same
    /// tokens accepted, one place they are written.
    ActInvalidated,
    SectionCreated,
    SectionFingerprintChanged,
    ReviewAttested,
    ReviewInvalidated,
    /// The run's review pass was disposed of, on one of `DEC-125`'s two arms
    /// (`T12`, `EX-13`).
    ///
    /// The row exists so the *choice* is legible in the log rather than only
    /// inside the snapshot: `sec-4` makes both arms defensible by **authority
    /// and visibility, not prohibition**, and this is the visibility half for
    /// the arm that clears the edge over live findings.
    ReviewDisposed,
    FindingRaised,
    FindingDisposed,
    /// Read-only history. Superseded by [`ChangeEvent::ActRecorded`] once
    /// acceptance flowed through the shared record seam (`SL-256` `sec-3`);
    /// retained because the change log is append-only history and `ChangeEvent`
    /// deserialises strictly.
    ///
    /// The **Rust** name is the only thing that moved. The wire token, the
    /// rendered token and the stored run-wide, term-free payload shape are all
    /// unchanged, and that is the point: the type warns every future
    /// construction site that the variant is history, while nine rows across
    /// seven of this repo's own live design runs keep parsing exactly as
    /// written. `DEC-239` refused both cheaper repairs on the record — a bare
    /// serde alias renames the variant but not the row, so an old row would
    /// render as `act_recorded` with an empty payload; whole-row normalisation
    /// at deserialise would manufacture a subject and an `act` term the writer
    /// never stored, and the next write would persist the invention as history.
    ///
    /// Member of [`ChangeEvent::READABLE`] and deliberately **not** of
    /// [`ChangeEvent::EMITTABLE`].
    LegacyAcceptanceAttested,
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
    /// Every event a persisted snapshot may **contain**, in the sketch's
    /// declaration order — the closed read vocabulary, single-sourced so an
    /// exhaustive table test cannot silently miss a variant (STD-001).
    ///
    /// Governs rendering and bounds: a historical row must stay renderable
    /// within the payload budget, so the widest-name assert and the containment
    /// check both quantify over this roster rather than over what a writer may
    /// still produce ([`ChangeEvent::EMITTABLE`], `sec-2`).
    pub(crate) const READABLE: [ChangeEvent; 24] = [
        ChangeEvent::NodeCreated,
        ChangeEvent::NodeLifecycle,
        ChangeEvent::NodeReparented,
        ChangeEvent::NodeBlockingChanged,
        ChangeEvent::NeedsAdded,
        ChangeEvent::NeedsRemoved,
        ChangeEvent::StageMoved,
        ChangeEvent::ActRecorded,
        ChangeEvent::ActInvalidated,
        ChangeEvent::SectionCreated,
        ChangeEvent::SectionFingerprintChanged,
        ChangeEvent::ReviewAttested,
        ChangeEvent::ReviewInvalidated,
        ChangeEvent::ReviewDisposed,
        ChangeEvent::FindingRaised,
        ChangeEvent::FindingDisposed,
        ChangeEvent::LegacyAcceptanceAttested,
        ChangeEvent::ReviewPolicyChanged,
        ChangeEvent::CheckpointDisposed,
        ChangeEvent::ObligationDelegated,
        ChangeEvent::ProposalRecorded,
        ChangeEvent::ProposalAccepted,
        ChangeEvent::ProposalRefused,
        ChangeEvent::StepDischarged,
    ];
    /// Every event this binary may newly **write** — [`ChangeEvent::READABLE`]
    /// less the retired member. A strict subset, proved as one directly below
    /// the roster (`DEC-239`).
    ///
    /// Governs writer coverage, and in one direction only: the coverage check
    /// proves every member *is* driven. Nothing in the type proves the converse,
    /// that nothing outside the roster is written — [`Pending`]'s constructors
    /// take any [`ChangeEvent`] — so that half is bought with evidence in
    /// `tests/e2e_design_state.rs` rather than with types (`sec-2`, `RV-360`
    /// `F-2`).
    ///
    /// Written out rather than derived from `READABLE`: a const-block copy trips
    /// `clippy::indexing_slicing`, and the subset assert is what holds the two
    /// declarations together.
    pub(crate) const EMITTABLE: [ChangeEvent; 23] = [
        ChangeEvent::NodeCreated,
        ChangeEvent::NodeLifecycle,
        ChangeEvent::NodeReparented,
        ChangeEvent::NodeBlockingChanged,
        ChangeEvent::NeedsAdded,
        ChangeEvent::NeedsRemoved,
        ChangeEvent::StageMoved,
        ChangeEvent::ActRecorded,
        ChangeEvent::ActInvalidated,
        ChangeEvent::SectionCreated,
        ChangeEvent::SectionFingerprintChanged,
        ChangeEvent::ReviewAttested,
        ChangeEvent::ReviewInvalidated,
        ChangeEvent::ReviewDisposed,
        ChangeEvent::FindingRaised,
        ChangeEvent::FindingDisposed,
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
    ///
    /// **The only source.** Serde reads this rather than deriving a second
    /// spelling from the Rust identifier, so a variant can be renamed without
    /// moving the wire token — see the [`From`] and [`TryFrom`] impls below.
    /// Editing an arm here changes what is written to disk and what already-stored
    /// snapshots are matched against; it is not a cosmetic change.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            ChangeEvent::NodeCreated => "node_created",
            ChangeEvent::NodeLifecycle => "node_lifecycle",
            ChangeEvent::NodeReparented => "node_reparented",
            ChangeEvent::NodeBlockingChanged => "node_blocking_changed",
            ChangeEvent::NeedsAdded => "needs_added",
            ChangeEvent::NeedsRemoved => "needs_removed",
            ChangeEvent::StageMoved => "stage_moved",
            ChangeEvent::ActRecorded => "act_recorded",
            ChangeEvent::ActInvalidated => "act_invalidated",
            ChangeEvent::SectionCreated => "section_created",
            ChangeEvent::SectionFingerprintChanged => "section_fingerprint_changed",
            ChangeEvent::ReviewAttested => "review_attested",
            ChangeEvent::ReviewInvalidated => "review_invalidated",
            ChangeEvent::ReviewDisposed => "review_disposed",
            ChangeEvent::FindingRaised => "finding_raised",
            ChangeEvent::FindingDisposed => "finding_disposed",
            ChangeEvent::LegacyAcceptanceAttested => "acceptance_attested",
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
            // Both judgements, as labels: the vocabulary is closed and has three
            // members (`unjudged` / `blocking` / `non-blocking`), which is what
            // makes a label the honest kind. A single term would have to be read
            // as the complement of the held value, and it is not one — a node that
            // predates the attribute moves from *unjudged*, which has no
            // complement to render.
            ChangeEvent::NodeLifecycle | ChangeEvent::NodeBlockingChanged => &[
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
            // One term, not more, and both halves of `DEC-237`'s reason apply to
            // each: the subject id and the act kind together answer *did what I
            // sent land?* completely, and a fourth term anywhere in this
            // vocabulary would silently falsify [`super::render`]'s hardcoded
            // widest-payload exemplar. Merged because `ActRecorded` is the exact
            // mirror of `ActInvalidated` — one shape, stated once.
            ChangeEvent::ActRecorded | ChangeEvent::ActInvalidated => {
                &[(PayloadKey::Act, ValueKind::Token)]
            }
            ChangeEvent::SectionCreated => &[(PayloadKey::Fingerprint, ValueKind::Digest)],
            ChangeEvent::ReviewAttested | ChangeEvent::ReviewInvalidated => &[
                (PayloadKey::Section, ValueKind::Token),
                (PayloadKey::Attestation, ValueKind::Token),
            ],
            // The arm, and the waiver's reason when it is the arm taken — which
            // is the whole of what `EX-13` asks the log to carry. The disposed
            // pass's `RV` is deliberately NOT a term: it is the run's current
            // pass by construction (a disposition naming another is refused at
            // admission), and a third term would push this event's saturated
            // payload past [`super::render`]'s budget and make *this* the widest
            // event, moving arithmetic the projection-bounds sketch is pinned
            // to. Stored whole on the act; the change log is a bounded
            // rendering, exactly as [`ChangeEvent::StepDischarged`] argues for
            // its skip reason.
            ChangeEvent::ReviewDisposed => &[
                (PayloadKey::Disposition, ValueKind::Label),
                (PayloadKey::Reason, ValueKind::Prose),
            ],
            ChangeEvent::FindingRaised | ChangeEvent::FindingDisposed => {
                &[(PayloadKey::Section, ValueKind::Token)]
            }
            // Run-wide and term-free: an acceptance has no run-local id at all —
            // its basis and authority are snapshot state, not delta.
            ChangeEvent::LegacyAcceptanceAttested => &[],
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
            // The step id is identity, bounded by the id slot it rides
            // (`runbook::RUNBOOK_STEP_ID_BYTES` derives from the same constant).
            // The outcome is a closed TWO-member vocabulary — `attested` or
            // `skipped` — so it is a label, exactly as `CheckpointDisposed`
            // declares its disposition. This arm read `Token` for both until
            // `DEC-247`: the construction has always built a label, and nothing
            // compared the two, so the declaration claimed an admission bound
            // (32 B) the term was never admitted against (16 B). The DECLARATION
            // was the error, not the construction (`ISS-290`).
            //
            // The skip reason is deliberately NOT a term. It is stored whole on
            // the discharge record, and the change log is a bounded rendering.
            ChangeEvent::StepDischarged => &[
                (PayloadKey::Step, ValueKind::Token),
                (PayloadKey::Outcome, ValueKind::Label),
            ],
        }
    }

    /// Put `terms` into this event's declared shape — its canonical order, and
    /// only terms the declaration names — or refuse.
    ///
    /// **Order.** A row's terms are built by whichever branch noticed the
    /// change, so without this the rendered order would follow construction
    /// rather than the documented shape — and a reader comparing two rows of the
    /// same kind would have to allow for both.
    ///
    /// **Kind** (`DEC-247`, `ISS-290`). Nothing used to compare the kind a term
    /// was *constructed* at with the kind its event *declares*, and the two carry
    /// different admission bounds — so a term could be admitted against a bound
    /// its own event does not claim for it. `StepDischarged` had drifted exactly
    /// that way. The check is here because this is the earliest moment both are
    /// in scope: a [`PayloadTerm`] has no event until a row pairs them, and
    /// [`PayloadTerm::admit`] therefore cannot see one.
    ///
    /// **Why fused with the ordering rather than sitting beside it.** The
    /// declaration governs both, and one method means a row cannot be built
    /// through the sorter alone. Fixing `StepDischarged` without closing the
    /// class is how `SL-233` `PHASE-08` left this behind after repairing the
    /// sibling key (`runbook::RUNBOOK_STEP_ID_BYTES`).
    ///
    /// A **subset** of the declared shape is legitimate — a disposition without
    /// a reason emits one of `ReviewDisposed`'s two terms — so the question is
    /// whether every term is declared, never whether every declaration is
    /// termed.
    ///
    /// The **stored** side is deliberately untouched: a persisted term re-enters
    /// through [`PayloadTerm::admit`] carrying its own kind, and re-deriving that
    /// kind from today's declaration would fail every row written under an older
    /// one — refusing history, which `DEC-249` forbids.
    pub(crate) fn shaped(self, terms: Vec<PayloadTerm>) -> Result<Vec<PayloadTerm>, Refusal> {
        let shape = self.payload_terms();
        // The declaration's index IS the membership proof, so there is no
        // no-such-key case left for the sort to invent an answer for. It used to
        // be checked with `contains` and then looked up again with a
        // `position(..).unwrap_or(usize::MAX)` fallback the guard had already
        // made unreachable — which read as if an undeclared key were handled by
        // sorting it last, the exact model this refusal exists to retire
        // (RV-367 `F-3`, `ISS-451`).
        let mut ordered = Vec::with_capacity(terms.len());
        for term in terms {
            let Some(at) = shape
                .iter()
                .position(|declared| *declared == (term.key(), term.kind()))
            else {
                return Err(Refusal::UndeclaredTerm {
                    event: self,
                    key: term.key(),
                    kind: term.kind(),
                });
            };
            ordered.push((at, term));
        }
        ordered.sort_by_key(|(at, _)| *at);
        Ok(ordered.into_iter().map(|(_, term)| term).collect())
    }
}

/// Serialisation reads [`ChangeEvent::as_str`], so the wire token has exactly one
/// source (STD-001).
///
/// `#[serde(rename_all)]` used to derive a second one from the Rust identifier,
/// and nothing held the two together: renaming a variant moved serde's token
/// while `as_str`'s stayed, and both halves still compiled. That is why
/// `AcceptanceAttested` could not be renamed until this impl existed — see
/// [`ChangeEvent::LegacyAcceptanceAttested`].
impl From<ChangeEvent> for String {
    fn from(event: ChangeEvent) -> String {
        event.as_str().to_owned()
    }
}

/// Deserialisation resolves against [`ChangeEvent::READABLE`] — everything a
/// persisted snapshot may contain — plus the one retired spelling.
///
/// **Strictness is preserved, not relaxed.** An unmatched token is a
/// [`Refusal::UnknownChangeEvent`], exactly as the derive refused it. A change
/// log that quietly accepted an event it could not name would be a change log
/// that lies about what the run did, and `ChangeEvent` deserialising strictly is
/// what makes the roster split load-bearing rather than cosmetic (`ISS-315`).
///
/// The same seam three other types in this leaf already cross: [`PayloadTerm`],
/// `DesignId` and `IntentSubject`.
impl TryFrom<String> for ChangeEvent {
    type Error = Refusal;

    fn try_from(raw: String) -> Result<ChangeEvent, Refusal> {
        ChangeEvent::READABLE
            .into_iter()
            .find(|event| event.as_str() == raw)
            .or_else(|| (raw == LEGACY_ACT_INVALIDATED).then_some(ChangeEvent::ActInvalidated))
            .ok_or(Refusal::UnknownChangeEvent { raw })
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
    /// Which recorded act a row is about (`EX-11`).
    Act,
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
            PayloadKey::Act => "act",
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
    /// The token this kind renders as in a refusal.
    ///
    /// Hand-written beside `#[serde(rename_all)]`, as [`super::attestation::ActKind`]
    /// and the rest of this leaf's closed vocabularies are: `clippy::use_debug`
    /// is denied, so a diagnostic naming a kind needs one, and
    /// `value_kind_tokens_match_their_serde_spelling` holds the two spellings
    /// together rather than trusting them to stay aligned — the drift
    /// [`ChangeEvent`]'s own serde impls were written to end.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            ValueKind::Token => "token",
            ValueKind::Label => "label",
            ValueKind::Digest => "digest",
            ValueKind::Prose => "prose",
        }
    }

    /// The closed kind vocabulary, so a sweep over value kinds reads its
    /// members from the enum rather than from whichever three a test author
    /// remembered.
    pub(crate) const ALL: [ValueKind; 4] = [
        ValueKind::Token,
        ValueKind::Label,
        ValueKind::Digest,
        ValueKind::Prose,
    ];

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

/// The wire keys the tolerant reader names a **second** time.
///
/// The derives above already spell them once; [`classify`] and [`place`] read a
/// failed row's fields back off its raw table, and a spelling that drifted from
/// the derive would silently stop classifying rather than fail (STD-001). Note
/// `terms` is [`ChangeRow`]'s Rust name and `row.term` its wire name — the
/// rename is the reason this group is worth naming at all.
const ROW_REVISION: &str = "revision";
const ROW_INDEX: &str = "index";
const ROW_EVENT: &str = "event";
const ROW_TERMS: &str = "term";
const TERM_KEY: &str = "key";
const TERM_KIND: &str = "kind";
const TERM_VALUE: &str = "value";

/// Why a stored row could not be read — a vocabulary this binary does not
/// recognise, classified at the point of failure.
///
/// Closed, and deliberately **not** a member of [`ChangeEvent`] (`DEC-251`):
/// widening the event vocabulary to carry an unreadable token would cost `Copy`,
/// `const fn as_str`, and through them both compile-time proofs above — and
/// would still cover only one of these four causes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unreadable {
    /// The `event` token names no member of [`ChangeEvent`].
    Event,
    /// A term's `key` names no member of [`PayloadKey`].
    PayloadKey,
    /// A term's `kind` names no member of [`ValueKind`].
    ValueKind,
    /// A term's value is longer than its [`ValueKind`] admits.
    TermTooLong,
}

impl Unreadable {
    /// The disclosed token. Single-sourced here, read by the renderer (STD-001).
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Unreadable::Event => "event",
            Unreadable::PayloadKey => "payload_key",
            Unreadable::ValueKind => "value_kind",
            Unreadable::TermTooLong => "term_too_long",
        }
    }
}

/// A row retained at full fidelity because this binary cannot read its
/// vocabulary (`DEC-249`).
///
/// `revision` and `index` are **required**, not tolerated: they are what places
/// the row in the retention window and the delta order, so a row without them is
/// not a degraded read but an unplaceable one, and still refuses the parse. The
/// tolerance is for vocabulary, never for shape.
///
/// `why` is carried, not re-derived. `STD-003` asks a degraded read to disclose
/// *what* was skipped **and why**, and the ordered try that produced this row is
/// the one place the cause is known (`RV-365` `F-4`).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RawRow {
    pub(crate) revision: u64,
    pub(crate) index: u32,
    /// The row exactly as stored, so re-serialising it returns the bytes the
    /// writer wrote rather than this reader's idea of them (`DEC-239`).
    pub(crate) raw: toml::Value,
    pub(crate) why: Unreadable,
}

/// One row as **stored**: read, or retained opaquely.
///
/// The asymmetry is deliberate and states something true. Only deserialisation
/// can produce [`StoredRow::Unreadable`] — a row this binary has just built is
/// readable by construction — which is why [`ChangeLog::record`] still takes
/// `ChangeRow` and wraps here.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StoredRow {
    Read(ChangeRow),
    Unreadable(RawRow),
}

impl StoredRow {
    /// The revision that produced the row, off either arm — what retention reads.
    pub(crate) const fn revision(&self) -> u64 {
        match self {
            StoredRow::Read(row) => row.revision,
            StoredRow::Unreadable(row) => row.revision,
        }
    }

    /// Position within that revision, off either arm — what ordering reads.
    pub(crate) const fn index(&self) -> u32 {
        match self {
            StoredRow::Read(row) => row.index,
            StoredRow::Unreadable(row) => row.index,
        }
    }

    /// The row's read form, if it has one.
    pub(crate) const fn read(&self) -> Option<&ChangeRow> {
        match self {
            StoredRow::Read(row) => Some(row),
            StoredRow::Unreadable(_) => None,
        }
    }
}

impl Serialize for StoredRow {
    /// An opaque row serialises as the table it was read from, byte-for-byte in
    /// content: the writer's row, not the reader's reconstruction of it.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            StoredRow::Read(row) => row.serialize(serializer),
            StoredRow::Unreadable(row) => row.raw.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for StoredRow {
    /// The **ordered try** (`DEC-251`).
    ///
    /// The whole row first, so the happy path pays for nothing. On failure the
    /// cause is classified where it is known and the row is placed; a row that
    /// cannot be *both* classified and placed refuses with the error it actually
    /// failed on, which is how "tolerance is for vocabulary, never for shape"
    /// stays a property of the structure rather than a second guard.
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<StoredRow, D::Error> {
        let raw = toml::Value::deserialize(deserializer)?;
        match ChangeRow::deserialize(raw.clone()) {
            Ok(row) => Ok(StoredRow::Read(row)),
            Err(refused) => match (classify(&raw), place(&raw)) {
                (Some(why), Some((revision, index))) => Ok(StoredRow::Unreadable(RawRow {
                    revision,
                    index,
                    raw,
                    why,
                })),
                _ => Err(serde::de::Error::custom(refused)),
            },
        }
    }
}

/// Where a row sits, if it says. `None` is a row that cannot be placed.
fn place(raw: &toml::Value) -> Option<(u64, u32)> {
    let revision = u64::try_from(raw.get(ROW_REVISION)?.as_integer()?).ok()?;
    let index = u32::try_from(raw.get(ROW_INDEX)?.as_integer()?).ok()?;
    Some((revision, index))
}

/// Which unrecognised vocabulary a row failed on, in the order the row is read.
///
/// `None` means no vocabulary explains the failure — the row is broken in some
/// other way, and the caller refuses it rather than retaining it under a guessed
/// cause.
fn classify(raw: &toml::Value) -> Option<Unreadable> {
    if let Some(event) = raw.get(ROW_EVENT)
        && ChangeEvent::deserialize(event.clone()).is_err()
    {
        return Some(Unreadable::Event);
    }
    raw.get(ROW_TERMS)
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .find(|term| PayloadTerm::deserialize((*term).clone()).is_err())
        .and_then(term_cause)
}

/// Which vocabulary the first unreadable term failed on.
///
/// Ordered as the term is read — key, then kind, then the bound its kind carries
/// — so the cause is the first thing that could not be read, not the last thing
/// checked. A term missing any of the three is shape, not vocabulary.
fn term_cause(term: &toml::Value) -> Option<Unreadable> {
    let (Some(key), Some(kind), Some(value)) = (
        term.get(TERM_KEY),
        term.get(TERM_KIND),
        term.get(TERM_VALUE),
    ) else {
        return None;
    };
    if PayloadKey::deserialize(key.clone()).is_err() {
        return Some(Unreadable::PayloadKey);
    }
    if ValueKind::deserialize(kind.clone()).is_err() {
        return Some(Unreadable::ValueKind);
    }
    // Key and kind both read and the value is a string, so the only admission
    // this term can have failed is the bound [`ValueKind`] carries.
    value.is_str().then_some(Unreadable::TermTooLong)
}

/// The change-log snapshot group: an explicitly recorded floor plus the retained
/// rows.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct ChangeLog {
    /// The oldest revision the log still covers, recorded **explicitly** and
    /// never inferred from the oldest surviving row: inference breaks when
    /// intervening revisions produced no material rows, and would report a
    /// complete delta as unavailable.
    pub(crate) floor: u64,
    #[serde(default, rename = "row")]
    pub(crate) rows: Vec<StoredRow>,
}

impl ChangeLog {
    /// Append `rows` produced by `revision`, then evict past the retention
    /// window and advance the floor.
    ///
    /// Takes [`ChangeRow`], not [`StoredRow`]: a row this binary has just built
    /// is readable by construction, so the wrap belongs here rather than at every
    /// caller, and the opaque arm stays reachable only from deserialisation.
    pub(crate) fn record(&mut self, revision: u64, rows: Vec<ChangeRow>) {
        self.rows.extend(rows.into_iter().map(StoredRow::Read));
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
        self.rows.retain(|row| row.revision() >= keep);
    }

    /// Whether a delta from `known_revision` can be answered completely.
    ///
    /// `known_revision < floor` is **unavailable**, which is a different fact
    /// from an empty delta: "nothing changed" and "I cannot tell you what
    /// changed" are opposite answers (design R2).
    pub(crate) const fn covers(&self, known_revision: u64) -> bool {
        known_revision >= self.floor
    }

    /// Every row this binary can read, in stored order.
    ///
    /// Test-facing on purpose. Production has exactly one consumer of the log's
    /// contents ([`super::render::envelope`]) and it must see **both** arms — a
    /// production reader that skipped the opaque ones would be the silent skip
    /// `STD-003` forbids. A test asserting about a row it wrote is in the
    /// opposite position: it knows the row is readable, and saying so once here
    /// beats unwrapping the arm at every assertion.
    #[cfg(test)]
    pub(crate) fn read_rows(&self) -> impl Iterator<Item = &ChangeRow> {
        self.rows.iter().filter_map(StoredRow::read)
    }

    /// The rows in the half-open range `(known_revision, current]`, newest last.
    pub(crate) fn since(&self, known_revision: u64) -> Vec<&StoredRow> {
        let mut rows: Vec<&StoredRow> = self
            .rows
            .iter()
            .filter(|row| row.revision() > known_revision)
            .collect();
        rows.sort_by_key(|row| (row.revision(), row.index()));
        rows
    }
}
