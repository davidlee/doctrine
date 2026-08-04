// SPDX-License-Identifier: GPL-3.0-only
//! Structured refusals — the pure core's negative return (design §5.1).
//!
//! The core returns a validated candidate plus effects, or *these*. A refusal
//! names what was refused and why in data, never in a formatted string, so the
//! shell owns rendering and a test asserts on the variant rather than on prose.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::Stage;
use super::attestation::{ActKind, AgentActKind, ReviewRef};
use super::gate::{Condition, Coverage, ObservedFact};
use super::ids::{DesignId, IdKind};

/// One way a recorded act fails to correspond to the rule it is written against
/// (design `sec-4`).
///
/// Four of the variants are the four correspondence rows — coverage, observed
/// facts, confirmation, disposition — and the other four are the complement of
/// the generated `const` assertion in [`super::gate`]: that assertion fixes which
/// *slots* a rule may name, and these are about the *value* a record put in one.
///
/// Deliberately **not** merged with the gate's own unmet-condition vocabulary.
/// The two answer different questions at different times: a fault is *this record
/// is malformed against its rule*, raised once on write; an unmet condition is
/// *this condition is not met by the records that exist*, re-derived at every
/// crossing. Folding them would give the gate variants it can never raise and
/// admission variants it can never check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ActFault {
    /// The rule names this coverage; the act carries that one, or none.
    ///
    /// [`Coverage::PerSection`] is carried by **no** act — it is a
    /// quantification the derivation performs over the section set — so a rule
    /// naming it refuses every record that carries a covered map at all.
    CoverageMismatch {
        required: Coverage,
        carried: Option<Coverage>,
    },
    /// The observed map's key set is not the rule's [`ObservedFact`] list.
    ///
    /// An act whose map is simply **absent** where its rule names a fact is
    /// refused rather than read as an empty observation, so the conjunctive
    /// binding cannot be evaded by omitting the field.
    ObservedKeys {
        missing: Vec<ObservedFact>,
        extra: Vec<ObservedFact>,
    },
    /// A confirmation is present where the rule names none, or absent where it
    /// names one. Presence-exactly-when, so one variant with a direction.
    Confirmation {
        expected: Option<AgentActKind>,
        carried: bool,
    },
    /// Likewise for a disposition.
    Disposition { expected: bool, carried: bool },
    /// A `Conducted` arm naming a review that is not the run's current pass.
    ForeignPass {
        named: ReviewRef,
        current: ReviewRef,
    },
    /// A `Conducted` arm over a pass whose ledger carries no concluded marker —
    /// **including one the shell could not read at all**. Absence is refusal, not
    /// satisfaction: a review Doctrine cannot see cannot have concluded.
    PassNotConcluded { review: ReviewRef },
    /// A `Waived` arm whose reason is empty or whitespace.
    WaiverReasonMissing,
    /// A blocking set naming nodes outside the map it was declared over.
    BlockingSetUnknownNodes { nodes: Vec<DesignId> },
}

impl fmt::Display for ActFault {
    /// One clause, on [`Refusal`]'s terms: the *data* is the contract and this
    /// is what crosses a `Display` boundary.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActFault::CoverageMismatch { required, carried } => write!(
                f,
                "its rule binds to `{}` coverage and the act carries {}",
                required.as_str(),
                carried.map_or_else(
                    || "none".to_owned(),
                    |carried| format!("`{}`", carried.as_str())
                )
            ),
            ActFault::ObservedKeys { missing, extra } => {
                let list = |facts: &[ObservedFact]| {
                    facts
                        .iter()
                        .map(|fact| fact.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                write!(
                    f,
                    "its observed facts are not the ones its rule names — missing: [{}], \
                     unasked-for: [{}]",
                    list(missing),
                    list(extra)
                )
            }
            // `Some(_)` with a confirmation carried is agreement, not a fault, so
            // only two of the four combinations reach here — and the match stays
            // total on the direction rather than on the pair.
            ActFault::Confirmation { expected, carried } => match expected {
                Some(declaration) => write!(
                    f,
                    "its rule requires it to confirm the current `{}` and it confirms nothing",
                    ActKind::from(*declaration).as_str()
                ),
                None => write!(
                    f,
                    "it confirms a declaration its rule does not name (carried: {carried})"
                ),
            },
            ActFault::Disposition { expected, .. } => f.write_str(if *expected {
                "its rule requires it to dispose of the review pass and it disposes nothing"
            } else {
                "it disposes of a review pass its rule does not name"
            }),
            ActFault::ForeignPass { named, current } => write!(
                f,
                "it disposes `{}`, which is not the pass this run is on (`{}`)",
                named.as_str(),
                current.as_str()
            ),
            ActFault::PassNotConcluded { review } => write!(
                f,
                "`{}` carries no concluded-pass marker Doctrine can read, so a conducted \
                 disposition over it is a claim about a pass that has not finished",
                review.as_str()
            ),
            ActFault::WaiverReasonMissing => f.write_str(
                "a waiver states why the pass was declined, and this one states nothing",
            ),
            ActFault::BlockingSetUnknownNodes { nodes } => write!(
                f,
                "it declares nodes the map it was declared over does not hold: {}",
                nodes
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

/// Why the pure core refused a submission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "refusal")]
pub(crate) enum Refusal {
    /// The stage move is not an edge of the [`super::gate::Advance`] relation.
    IllegalStageMove { from: Stage, to: Stage },
    /// The move is a legal edge, but a cumulative gate condition does not hold
    /// against current content (DEC-066/DEC-067).
    GateNotCleared {
        from: Stage,
        to: Stage,
        missing: Vec<Condition>,
    },
    /// A direct regression was submitted without a recorded reason (DEC-067).
    RegressionReasonMissing { from: Stage, to: Stage },
    /// A stage move that is neither a legal forward edge nor a regression.
    NotARegression { from: Stage, to: Stage },
    /// A run-local id is malformed — unknown prefix, or an empty body.
    MalformedId { raw: String },
    /// A run-local id exceeds its admission bound. A refusal, never a trim: a
    /// truncated identity is a *wrong* identity rather than a shorter one.
    IdTooLong { raw: String, limit: usize },
    /// A payload term exceeds the admission bound for its value kind.
    ValueTooLong {
        what: &'static str,
        raw: String,
        limit: usize,
    },
    /// A parent or `needs` edge would close a cycle (design §5.3).
    CyclicEdge { from: DesignId, to: DesignId },
    /// An edge references a node the map does not hold.
    UnknownNode { id: DesignId },
    /// A node was moved to `resolved` without a semantic disposition (DEC-062).
    DispositionMissing { id: DesignId },
    /// One unordered batch named the same subject twice (DEC-063).
    DuplicateSubject { id: DesignId },
    /// An agent proposal tried to move a user-locked cursor (EX-7).
    CursorLocked { at: Option<DesignId> },
    /// The payload asserted a different run (DEC-059 stale-context detection).
    RunMismatch { declared: String, current: String },
    /// The compare-and-swap failed: another writer moved the run on.
    StaleRevision { known: u64, current: u64 },
    /// The submission id is unknown *and* its declared revision is below the
    /// retained window, so "already applied" and "never seen" cannot be told
    /// apart. Refused as expired rather than silently treated as new.
    SubmissionExpired { known: u64, floor: u64 },
    /// A used submission id arrived carrying different bytes — a different
    /// submission wearing a used name, not a retry.
    SubmissionReplayed { submission: String },
    /// A re-adoption declared a fingerprint that is not what Doctrine reads.
    AdoptionStale {
        declared: String,
        observed: Option<String>,
    },
    /// A re-adoption's stable-marker map is not complete and exact.
    AdoptionMarkersInvalid {
        missing: Vec<DesignId>,
        unknown: Vec<DesignId>,
        mismatched: Vec<DesignId>,
    },
    /// A `\r` byte reached the authored document's boundary — at declaration
    /// admission or in the document itself. ONE refusal on both doors: a `\r`
    /// accepted at declare and refused only at parse would let a body reach
    /// title derivation and materialise through a door the document boundary has
    /// already closed.
    CarriageReturnInDocument,
    /// Non-blank bytes stand before the document's first marker line — a
    /// preamble nobody declared, or a first marker whose syntax was broken.
    /// Whitespace does not trigger it: a leading blank line is what a formatter
    /// produces, and refusing that would make the document unformattable.
    MarkerFreeAddition,
    /// Non-blank bytes stand before the first heading of a document being
    /// **imported** (SL-233 PHASE-11 D2). The marker path's
    /// [`Refusal::MarkerFreeAddition`] is the same shape at a different door and
    /// deliberately not reused: it names a marker, and an imported document has
    /// none. Carries the line so the refusal names the fix rather than the file.
    UnheadedPreamble { line: usize },
    /// A fenced code block opened and the imported document ended with it still
    /// open (SL-233 PHASE-11 D6). `CommonMark` would close it at EOF; import is
    /// deliberately stricter, because closing it means reading the tail as code,
    /// which drops sections **silently** — a run that looks imported and merely
    /// is not all there.
    UnclosedFence { line: usize },
    /// One section id marks two regions — a section was copy-pasted.
    DuplicateMarker { id: DesignId },
    /// A marker names a section the run does not hold — an id was edited, or a
    /// section was invented in prose. It outranks [`Refusal::MissingMarker`],
    /// which an edited id also produces, because it names the token actually on
    /// the line in front of the user.
    UnknownMarker { id: DesignId },
    /// A section the run holds has no marker in the document — its marker line
    /// was deleted while its prose was kept. Reported against the CAUSE (the
    /// absent marker) rather than the symptom (the preceding section's
    /// fingerprint moving, because the orphaned prose merged into its region).
    MissingMarker { id: DesignId },
    /// A marker is present and its region is empty — a section's prose was
    /// deleted while its marker was kept.
    StructuralDeletion { id: DesignId },
    /// The document's final region is non-empty and does not end in a newline.
    /// Reachable only at the end of the document: a marker line begins at column
    /// 0, so every earlier region is empty or ends in one.
    UnterminatedDocument,
    /// A section declaration carried no body, so there is nothing to fingerprint.
    SectionBodyMissing { id: DesignId },
    /// An imported `OQ-*` entry reached seeding with no precomputed digest. The
    /// shell derives every entry digest from the same regions this core reads,
    /// so the map is total by construction and this is a broken shell contract
    /// rather than a document defect — refused rather than seeded with a
    /// placeholder, because a node carrying an empty fingerprint is the very
    /// thing `EX-9` closes.
    ImportedEntryDigestMissing { line: u32 },
    /// A section's body holds no non-blank line, so arm 1 of the title
    /// derivation has nothing to read (EX-13(b)).
    SectionBodyEmpty { id: DesignId },
    /// A section's body does not begin with an ATX heading, so its title is
    /// underivable. Requiring the heading is what makes the derivation total:
    /// the alternative is a section whose title silently defaults to nothing.
    SectionBodyHeadingMissing { id: DesignId },
    /// A section's opening heading carries no text — a bare `##`, or one whose
    /// whole content is a closing sequence.
    SectionTitleEmpty { id: DesignId },
    /// An attestation named no section, or named one the run does not hold.
    AttestationSubjectMissing { id: DesignId },
    /// A payload claimed a clearance Doctrine derives for itself
    /// ([`Condition::is_derived`]).
    ///
    /// Refused rather than ignored: a caller that believes it has cleared the
    /// lock gate, and is silently not believed, learns nothing. This is the
    /// self-attestation bypass closed at admission — the reviewing conditions are
    /// read off the run's review state, so a claim about them is a category error
    /// and not merely redundant.
    ///
    /// [`Condition::is_derived`]: super::gate::Condition::is_derived
    DerivedConditionClaimed { condition: Condition },
    /// A user acceptance arrived with a blank basis. DEC-088 calls the basis
    /// concise and *required*: an acceptance with nothing stated is an
    /// unattributable claim, and the lock gate's whole point is that the claim is
    /// auditable.
    ///
    /// Run-level, so it carries no id — the acceptance is about the design.
    AcceptanceBasisMissing,
    /// A runbook step was skipped with no reason stated (`EX-9`).
    ///
    /// Carries the step id as a plain `String`, not a [`DesignId`]: a runbook
    /// step id is dot-separated and author-assigned, and `DesignId::parse`
    /// admits neither the dots nor an unprefixed body.
    DischargeReasonMissing { step: String },
    /// A forward edge carries a runbook and it is not fully discharged
    /// (`EX-8`).
    ///
    /// Separate from [`Refusal::GateNotCleared`] because it carries what that
    /// one structurally cannot: `GateNotCleared` holds `Vec<Condition>`, and a
    /// `Condition` is payload-free with nowhere for a step identity to ride. An
    /// **empty** `outstanding` is the fail-closed case — the guard applies and
    /// no standing was derived for it.
    ///
    /// `regressed` names the subset whose discharge stands and whose check has
    /// since stopped clearing (`EX-11`). A field rather than a second variant:
    /// the fact refused is one fact — the edge's runbook is not discharged —
    /// and `EX-8` budgets exactly one gate refusal. What differs is the repair,
    /// so the message says which steps were never done and which came undone.
    RunbookNotDischarged {
        from: Stage,
        to: Stage,
        outstanding: Vec<String>,
        regressed: Vec<String>,
    },
    /// A step's check did not clear, so the step is not discharged (`EX-10`).
    ///
    /// One variant for two outcomes because they are one fact — the check did
    /// not say yes — and the caller acts on both the same way. `exit` tells them
    /// apart: `Some(code)` is a check that ran and objected, and its `output` is
    /// what it objected with; `None` is the fail-closed case, no result supplied
    /// where a check applies.
    VerifierFailed {
        step: String,
        exit: Option<i32>,
        output: String,
    },
    /// A discharge named a step that is not the one at the cursor — ahead of it,
    /// behind it, or absent from the runbook. One refusal, because they are one
    /// fact: this is not the obligation at hand.
    DischargeNotAtCursor {
        step: String,
        /// The step the runbook expects, or `None` when every step is discharged.
        expected: Option<String>,
    },
    /// A discharge arrived while the run stands at a stage whose outbound edge
    /// carries no runbook. Distinct from a fully-discharged one: nothing here
    /// was ever asked for.
    RunbookAbsent { stage: Stage },
    /// A finding declaration carried no summary, so there is nothing to review.
    FindingSummaryMissing { id: DesignId },
    /// A finding named no section, or named one the run does not hold.
    FindingSubjectMissing { id: DesignId },
    /// A checkpoint declaration is missing the inquiry it disposes or the record
    /// that disposes it — the two halves of a DEC-062 disposition.
    CheckpointIncomplete { id: DesignId },
    /// A checkpoint resolved a node without declaring any of the four DEC-062
    /// dispositions. "We discussed it" is not one of them: the intentionally
    /// non-durable case is *declared*, never defaulted into.
    CheckpointDispositionMissing { id: DesignId },
    // There is no `CheckpointDispositionConflict`. It named a declaration
    // carrying both the pre-effect `record` annotation and a `dispose` — a state
    // EX-12 removed from the wire, so the refusal became unconstructible. A
    // vocabulary that keeps a word for a state the wire cannot express is a
    // vocabulary that lies about the wire.
    /// A `create` disposition reached the validated candidate with no canonical
    /// id bound to it. An internal ordering fault, not a caller error: DEC-086
    /// journals the claimed id at step 3, well before the snapshot at step 6.
    CheckpointRecordUnresolved { id: DesignId },
    /// A proposal-bearing payload also carried something that writes (`EX-2`,
    /// DEC-068).
    ///
    /// The guard is on the **act**, not on the actor: v1 authenticates no worker
    /// identity, so what is refused is the *composition* — a delegate's channel
    /// cannot express an advance, whoever is holding it. `what` names the payload
    /// key that was found, because the remedy is to remove that key and resubmit.
    DelegateCannotAdvance { what: &'static str },
    /// An act named an assignment this run does not hold.
    UnknownDelegation { id: DesignId },
    /// A subject appeared in a declaration batch whose kind is not addressable
    /// that way — its state is written through a run-level payload field
    /// instead, and a declaration route to it would be a second way into state
    /// the sole-writer boundary keeps single.
    ///
    /// One variant for three kinds rather than one each: the rule is the same
    /// rule, and only the field it names differs — which the message reads off
    /// the id's own kind.
    SubjectNotDeclarable { id: DesignId },
    /// An export named an obligation that is already spoken for — one bounded
    /// obligation means one outstanding assignment at a time.
    DelegationOutstanding { id: DesignId, obligation: DesignId },
    /// The coordinator acted on an assignment that has no proposal to act on.
    ProposalMissing { id: DesignId },
    /// The obligation is no longer the one that was assigned, so the proposal
    /// answers a question the run has moved on from (`EX-3`, design §5.4).
    ///
    /// Refused rather than rebased, and the proposal is **left where it is** —
    /// still recorded, still readable, still unapplied. Silently rebasing a stale
    /// proposal onto current content is a named rejected alternative (design §7):
    /// it would apply the delegate's answer to a question they were never asked.
    ProposalStale {
        id: DesignId,
        obligation: DesignId,
        exported_at: u64,
    },
    /// A recorded act does not correspond to the rule it is written against
    /// (design `sec-4`).
    ///
    /// Raised on WRITE, before anything is persisted — so every stored act
    /// satisfies the correspondence by construction and the gate never re-checks
    /// it. **One variant, not eight refusals**, on [`Refusal::RunbookNotDischarged`]'s
    /// own precedent and its stated reason: the fact refused is one fact — this
    /// act does not correspond to its rule — and what differs is which slot, so
    /// the slot rides a field.
    ///
    /// `causes` is non-empty, and carries **every** way the act failed rather
    /// than the first: an agent that repairs one slot and resubmits should not
    /// discover the rest one round-trip at a time.
    ActAdmissionInvalid { act: ActKind, causes: Vec<ActFault> },
    /// A disposition arrived while the run is on **no review pass**.
    ///
    /// Refused at construction rather than reported as a fault, because it is not
    /// one: the correspondence asks whether a record matches its rule, and here
    /// there is no record to make — a [`DisposedPass`] binds to the pass it
    /// disposes and the run has none to name. The [`Refusal::RunbookAbsent`]
    /// precedent, one act along.
    ///
    /// Run-level, so it carries no id: only `ReviewDisposed` may dispose, and
    /// naming it would restate what the rule already says.
    ///
    /// [`DisposedPass`]: super::attestation::DisposedPass
    ReviewPassAbsent,
    /// An agent declaration arrived with no shell-computed claim digest.
    ///
    /// The pure layer never hashes, so the digest is a derived fact like any
    /// other — and a record carrying an empty fingerprint would read as though it
    /// had one, silently matching whatever `confirms` was written beside it. The
    /// [`Refusal::ImportedEntryDigestMissing`] rule, applied to the other
    /// shell-supplied digest: refused, never seeded with a placeholder.
    DeclarationDigestMissing { act: AgentActKind },
    /// The eviction ladder was exhausted and the **no-drop set alone** still
    /// exceeds the whole-envelope ceiling. The one irreducible state, refused
    /// rather than emitted as a quietly malformed envelope.
    EnvelopeIrreducible { budget: usize, rendered: usize },
}

impl fmt::Display for Refusal {
    /// A terse, single-line rendering. The *data* is the contract — tests assert
    /// on the variant, never on this text — but a refusal has to be able to cross
    /// a `Display` boundary, and serde's `try_from` is one of them.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::IllegalStageMove { from, to } => {
                write!(f, "illegal stage move: {} → {}", from.as_str(), to.as_str())
            }
            // The conditions are **named**, not counted. The type has always
            // carried every missing one for the reason the module doc gives — an
            // agent that fixes one and retries should not discover the rest one
            // round-trip at a time — but the only surface a black-box caller has
            // is this line, and a count told it nothing it could act on.
            Refusal::GateNotCleared { from, to, missing } => write!(
                f,
                "gate not cleared for {} → {}: {} outstanding",
                from.as_str(),
                to.as_str(),
                missing
                    .iter()
                    .map(|condition| condition.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Refusal::AcceptanceBasisMissing => write!(
                f,
                "a user acceptance must state its basis; an auditable claim with nothing stated \
                 is not one"
            ),
            Refusal::DischargeReasonMissing { step } => write!(
                f,
                "step `{step}` was skipped with no reason; a skip is a DISCLOSED deviation, \
                 and one with nothing disclosed is a silent skip wearing its name"
            ),
            Refusal::RunbookNotDischarged {
                from,
                to,
                outstanding,
                regressed,
            } => {
                if outstanding.is_empty() {
                    write!(
                        f,
                        "the runbook guarding {} → {} could not be read, so the edge \
                         stays shut",
                        from.as_str(),
                        to.as_str()
                    )
                } else {
                    write!(
                        f,
                        "{} → {} requires its runbook discharged; outstanding: {}",
                        from.as_str(),
                        to.as_str(),
                        outstanding.join(", ")
                    )?;
                    // Named separately because the repair is different: these
                    // steps WERE discharged and their definitions are unchanged
                    // — it is what their checks report that has moved.
                    if regressed.is_empty() {
                        Ok(())
                    } else {
                        write!(
                            f,
                            " — regressed since discharge (the check no longer clears): {}",
                            regressed.join(", ")
                        )
                    }
                }
            }
            Refusal::VerifierFailed { step, exit, output } => match exit {
                Some(code) => write!(
                    f,
                    "step `{step}`'s check exited {code}, so the step is not discharged:\n{}",
                    output.trim_end()
                ),
                None => write!(
                    f,
                    "step `{step}` carries a check and no result for it was produced; \
                     the step stays outstanding rather than discharged on the claim alone"
                ),
            },
            Refusal::DischargeNotAtCursor { step, expected } => match expected {
                Some(expected) => write!(
                    f,
                    "step `{step}` is not the one at the cursor — this runbook is a \
                     SEQUENCE and expects `{expected}` next"
                ),
                None => write!(
                    f,
                    "step `{step}` cannot be discharged: every step of this runbook is \
                     already discharged against its current definition"
                ),
            },
            Refusal::RunbookAbsent { stage } => write!(
                f,
                "no runbook guards the edge out of `{}`, so there is nothing here to \
                 discharge",
                stage.as_str()
            ),
            Refusal::DerivedConditionClaimed { condition } => write!(
                f,
                "{} is derived from the run's review state and cannot be claimed as evidence",
                condition.as_str()
            ),
            Refusal::RegressionReasonMissing { from, to } => write!(
                f,
                "regression {} → {} needs a recorded reason",
                from.as_str(),
                to.as_str()
            ),
            Refusal::NotARegression { from, to } => {
                write!(f, "{} → {} is not a regression", from.as_str(), to.as_str())
            }
            Refusal::MalformedId { raw } => write!(f, "malformed run-local id: `{raw}`"),
            Refusal::IdTooLong { raw, limit } => write!(
                f,
                "run-local id is {} bytes, over the {limit}-byte admission bound: `{raw}` \
                 (identity is refused, never truncated)",
                raw.len()
            ),
            Refusal::ValueTooLong { what, raw, limit } => write!(
                f,
                "{what} is {} bytes, over the {limit}-byte admission bound: `{raw}`",
                raw.len()
            ),
            Refusal::CyclicEdge { from, to } => write!(f, "edge {from} → {to} closes a cycle"),
            Refusal::UnknownNode { id } => write!(f, "unknown node: {id}"),
            Refusal::DispositionMissing { id } => {
                write!(f, "{id} cannot resolve without a disposition")
            }
            Refusal::DuplicateSubject { id } => write!(f, "duplicate subject in batch: {id}"),
            Refusal::CursorLocked { at } => match at {
                Some(at) => write!(f, "cursor is user-locked at {at}"),
                None => f.write_str("cursor is user-locked"),
            },
            Refusal::RunMismatch { declared, current } => write!(
                f,
                "payload asserts run `{declared}` but the live run is `{current}` — \
                 this context is stale; re-read the run before submitting"
            ),
            Refusal::StaleRevision { known, current } => write!(
                f,
                "conflict: payload asserts known_revision {known} but the run is at \
                 revision {current} — another writer moved it on; re-read and resubmit"
            ),
            Refusal::SubmissionExpired { known, floor } => write!(
                f,
                "submission expired: it asserts revision {known}, below the retained \
                 replay window which starts at revision {floor} — Doctrine can no longer \
                 tell a retry from a new submission, so it refuses rather than guess"
            ),
            Refusal::SubmissionReplayed { submission } => write!(
                f,
                "submission id `{submission}` was already applied with different bytes — \
                 a retry must carry the same payload"
            ),
            Refusal::AdoptionStale { declared, observed } => match observed {
                Some(observed) => write!(
                    f,
                    "adopt_authored declares fingerprint `{declared}` but design.md reads \
                     `{observed}` — re-read the document and declare what it says now"
                ),
                None => write!(
                    f,
                    "adopt_authored declares fingerprint `{declared}` but design.md is absent"
                ),
            },
            Refusal::AdoptionMarkersInvalid {
                missing,
                unknown,
                mismatched,
            } => write!(
                f,
                "adopt_authored's marker map is not complete and exact: {} missing, \
                 {} unknown, {} mismatched",
                missing.len(),
                unknown.len(),
                mismatched.len()
            ),
            Refusal::CarriageReturnInDocument => f.write_str(
                "a carriage return (`\\r`) reached the authored document boundary — \
                 design.md is LF-only, so CRLF prose is refused at declare rather \
                 than stored and refused later at parse; convert it first \
                 (`dos2unix`)",
            ),
            Refusal::MarkerFreeAddition => f.write_str(
                "design.md holds text before its first section marker — a new section is \
                 declared through the run, not typed into the document; if a marker line \
                 was edited, restore it exactly as Doctrine wrote it",
            ),
            Refusal::UnheadedPreamble { line } => write!(
                f,
                "design.md holds text at line {line}, before its first heading — import \
                 seats every region under a heading of its own and will not invent a \
                 title for one; put a heading above that text, or move it below one"
            ),
            Refusal::UnclosedFence { line } => write!(
                f,
                "the fenced code block opened at line {line} is never closed — import \
                 would read the rest of design.md as code and silently drop the \
                 sections after it; close the fence (a fence of n backticks closes \
                 only on n or more) and import again"
            ),
            Refusal::DuplicateMarker { id } => write!(
                f,
                "section {id} is marked twice — one id marks exactly one region; \
                 remove the copy, or declare the new section through the run"
            ),
            Refusal::UnknownMarker { id } => write!(
                f,
                "design.md marks {id}, which this run does not hold — a marker id was \
                 edited, or a section was invented in prose; declare a new section \
                 through the run instead"
            ),
            Refusal::MissingMarker { id } => write!(
                f,
                "section {id} has no marker in the document — its marker line was \
                 deleted while the run still holds it; restore the marker, or retire \
                 the section through the run"
            ),
            Refusal::StructuralDeletion { id } => write!(
                f,
                "section {id}'s marker is present but its region is empty — the \
                 prose was deleted and the marker kept; restore the prose or \
                 remove the section through the run"
            ),
            Refusal::UnterminatedDocument => f.write_str(
                "the document's last section does not end in a newline — restore \
                 the trailing newline the file was written with",
            ),
            Refusal::SectionBodyMissing { id } => {
                write!(f, "section {id} was declared without a body")
            }
            Refusal::ImportedEntryDigestMissing { line } => write!(
                f,
                "the open-questions entry at line {line} reached import with no \
                 content fingerprint — re-run the import; if it recurs, the \
                 entry digests are being derived from a different reading of \
                 the document than the one being imported"
            ),
            Refusal::SectionBodyEmpty { id } => write!(
                f,
                "section {id} was declared with an empty body — a section's title is \
                 derived from its own first heading, so the body must hold one"
            ),
            Refusal::SectionBodyHeadingMissing { id } => write!(
                f,
                "section {id}'s body does not begin with an ATX heading (`## Title`) — \
                 a section's title is derived from its own first heading, never \
                 declared beside it"
            ),
            Refusal::SectionTitleEmpty { id } => write!(
                f,
                "section {id}'s opening heading has no text — the title is derived \
                 from it, and an empty one is refused rather than stored"
            ),
            Refusal::FindingSummaryMissing { id } => {
                write!(f, "finding {id} states nothing")
            }
            Refusal::FindingSubjectMissing { id } => {
                write!(f, "finding {id} names no section this run holds")
            }
            Refusal::AttestationSubjectMissing { id } => {
                write!(f, "attestation {id} names no section this run holds")
            }
            Refusal::CheckpointIncomplete { id } => write!(
                f,
                "checkpoint {id} needs both the inquiry it disposes and the record that \
                 disposes it"
            ),
            Refusal::CheckpointDispositionMissing { id } => write!(
                f,
                "checkpoint {id} resolves a node without a disposition — declare one of: {} \
                 (resolution requires a semantic outcome; an intentionally non-durable \
                 exchange is declared, not defaulted into)",
                super::inquiry::DispositionForm::vocabulary()
            ),
            Refusal::CheckpointRecordUnresolved { id } => write!(
                f,
                "checkpoint {id} asks Doctrine to create a record but no canonical id was \
                 bound to it before the snapshot — the run does not advance"
            ),
            Refusal::DelegateCannotAdvance { what } => write!(
                f,
                "a delegated proposal may not also carry `{what}` — a delegate proposes and \
                 the coordinator writes (DEC-068); submit the proposal alone, and let the \
                 coordinator apply what it accepts"
            ),
            Refusal::UnknownDelegation { id } => {
                write!(f, "this run holds no delegation `{id}`")
            }
            Refusal::SubjectNotDeclarable { id } => write!(
                f,
                "{id} cannot be declared — it is written through the payload's `{}` field",
                match id.kind() {
                    IdKind::CheckpointAct => "checkpoint_act",
                    IdKind::AgentDeclaration => "agent_declaration",
                    _ => "delegation",
                }
            ),
            Refusal::DelegationOutstanding { id, obligation } => write!(
                f,
                "{obligation} is already assigned to {id}, which is still awaiting the \
                 coordinator — dispose that assignment before cutting another"
            ),
            Refusal::ProposalMissing { id } => {
                write!(f, "delegation {id} holds no proposal to act on")
            }
            Refusal::ProposalStale {
                id,
                obligation,
                exported_at,
            } => write!(
                f,
                "{id}'s proposal is stale: {obligation} is no longer the obligation assigned \
                 at revision {exported_at} — the proposal stays recorded and unapplied, and \
                 is never rebased onto content it did not answer; re-export the obligation \
                 to delegate it as it stands now"
            ),
            Refusal::ActAdmissionInvalid { act, causes } => write!(
                f,
                "the `{}` act does not correspond to its rule: {}",
                act.as_str(),
                causes
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            Refusal::ReviewPassAbsent => write!(
                f,
                "the act disposes of a review pass and the run is on none"
            ),
            Refusal::DeclarationDigestMissing { act } => write!(
                f,
                "the `{}` declaration arrived with no claim digest",
                ActKind::from(*act).as_str()
            ),
            Refusal::EnvelopeIrreducible { budget, rendered } => write!(
                f,
                "the turn envelope cannot be reduced to its {budget}-byte ceiling: every \
                 bounded list is empty and the undroppable fields alone render {rendered} \
                 bytes — read the run with `design show --full` instead of trusting a \
                 malformed projection"
            ),
        }
    }
}
