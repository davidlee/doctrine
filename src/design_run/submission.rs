// SPDX-License-Identifier: GPL-3.0-only
//! Sparse declarations and the unordered batch (DEC-063).
//!
//! Three states, three meanings, and the whole point is that they do not
//! collapse into two: **omission persists** prior state, **`null` clears** a
//! nullable scalar, and an **empty collection clears** the collection. A model
//! that maps a missing key onto `None` cannot express "leave it alone", and an
//! agent then has to resend every field it did not intend to touch.
//!
//! One batch is unordered. [`Batch::validate`] refuses duplicate subjects and
//! returns the declarations in a **subject-determined** order, so the same set of
//! declarations submitted in any order yields the same sequence — the property
//! PHASE-03's change-log index depends on.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::Stage;
use super::attestation::{ActKind, AgentAct, ReviewDisposition, ReviewPolicy, Reviewer};
use super::ids::{DesignId, IdKind};
use super::inquiry::{DispositionForm, InquiryLifecycle, Provenance};
use super::refusal::Refusal;
use super::traversal::{Authority, Posture};

/// A sparsely submitted value.
///
/// [`Sparse::Omitted`] is the [`Default`], which is what makes `#[serde(default)]`
/// on a field mean "the key was absent" rather than "the value was null".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Sparse<T> {
    /// The key was absent — prior state persists.
    #[default]
    Omitted,
    /// The key was present and `null` — the value is cleared.
    Null,
    /// The key carried a value.
    Value(T),
}

impl<T> Sparse<T> {
    /// Apply this declaration to a prior value.
    ///
    /// The three arms are the contract: persist, clear, replace.
    pub(crate) fn apply(self, prior: Option<T>) -> Option<T> {
        match self {
            Sparse::Omitted => prior,
            Sparse::Null => None,
            Sparse::Value(value) => Some(value),
        }
    }

    /// Whether the key was absent from the submission.
    pub(crate) const fn is_omitted(&self) -> bool {
        matches!(self, Sparse::Omitted)
    }
}

impl<T> Sparse<Vec<T>> {
    /// Apply a collection declaration to a prior collection.
    ///
    /// An empty [`Sparse::Value`] clears — distinct from [`Sparse::Omitted`],
    /// which leaves the prior collection intact. `null` clears too; the two
    /// clearing spellings agree, and neither is reachable by omission.
    pub(crate) fn apply_collection(self, prior: Vec<T>) -> Vec<T> {
        match self {
            Sparse::Omitted => prior,
            Sparse::Null => Vec::new(),
            Sparse::Value(values) => values,
        }
    }
}

impl<T: Serialize> Serialize for Sparse<T> {
    /// `Omitted` serialises as null; a field carrying it should be skipped by
    /// `skip_serializing_if` so the key is genuinely absent on the wire.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Sparse::Value(value) => serializer.serialize_some(value),
            Sparse::Null | Sparse::Omitted => serializer.serialize_none(),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Sparse<T> {
    /// A present key deserialises to `Value` or `Null`. `Omitted` is unreachable
    /// here by construction — it is what `#[serde(default)]` supplies when the
    /// key never appears, which is exactly the distinction being preserved.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<T>::deserialize(deserializer).map(|value| match value {
            Some(value) => Sparse::Value(value),
            None => Sparse::Null,
        })
    }
}

/// One declaration in a batch: a subject plus the sparse fields being asserted.
///
/// PHASE-03 binds the field set to the snapshot's groups, and the binding is by
/// the subject's **kind** rather than by a separate discriminator: an `inq-`
/// subject declares a node, a `sec-` subject a draft section, an `att-` subject a
/// review attestation, an `int-` subject the integrated pass, a `fnd-` subject a
/// finding, a `cp-` subject a checkpoint disposition. The id's prefix already
/// carries that fact ([`DesignId::kind`]), so a second, redundant "type" field
/// could only disagree with it.
///
/// The list is the *declarable* kinds and not [`IdKind`] entire: a `dlg-` subject
/// is refused here, because a delegation is acted on through
/// [`ApplyRequest::delegation`] and a declaration route to it would be a second
/// way into the state the sole-writer boundary keeps single (`EX-2`).
///
/// `deny_unknown_fields` (EX-14) is the mechanism that makes a *removed* field a
/// refusal rather than a silent no-op. Without it, PHASE-06's two removals —
/// `title` and the `record`/`adopt_record` annotation pair — would land as keys
/// serde quietly swallows, so a caller still sending them would be told nothing
/// and get a section with no title or a checkpoint with no disposition. It is
/// deliberately NOT on [`ApplyRequest`], which carries a `#[serde(flatten)]`
/// envelope: serde cannot reconcile the two, and the flattened field's keys
/// would be refused as unknown.
///
/// [`IdKind`]: super::ids::IdKind
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Declaration {
    subject: DesignId,
    #[serde(default, skip_serializing_if = "Sparse::is_omitted")]
    question: Sparse<String>,
    #[serde(default, skip_serializing_if = "Sparse::is_omitted")]
    needs: Sparse<Vec<DesignId>>,
    /// The primary parent. `null` clears it — a node may be detached without
    /// being deleted (v1 has lifecycle transitions rather than deletion).
    #[serde(default, skip_serializing_if = "Sparse::is_omitted")]
    parent: Sparse<DesignId>,
    /// Where a newly created node came from. Defaults to agent-proposed, which
    /// is the conservative reading: user direction must be *stated*, never
    /// inferred (design R2/R12).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    provenance: Option<Provenance>,
    /// A non-resolving lifecycle move. Resolution goes through `disposition`,
    /// because DEC-062 makes a disposition part of resolving rather than a
    /// field one may forget.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    lifecycle: Option<InquiryLifecycle>,
    /// Section prose, **beginning with the section's own heading**. Stored
    /// unbounded; the shell hands in its digest, and
    /// [`super::section::derive_title`] reads the title back out of it. There is
    /// no `title` field beside this one: a derived title cannot desynchronise
    /// from the prose, because there is nothing for it to disagree with.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    /// The section an `att-` subject attests to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    attests: Option<DesignId>,
    /// Who reviewed. Defaults to the v1 default reviewer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reviewer: Option<Reviewer>,
    /// The section a `fnd-` subject's finding is about.
    ///
    /// Deliberately not `attests` under a wider name: *this section was
    /// reviewed and cleared* and *this section has a problem* are opposite
    /// claims, and one field carrying both would let a typo turn one into the
    /// other.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    concerns: Option<DesignId>,
    /// What a `fnd-` subject's finding says.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    /// Whether a finding holds the lock gate open. Absent means non-blocking —
    /// a finding blocks only when it is *said* to (design R2/R12).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    blocking: Option<bool>,
    /// How a finding was disposed. Its presence, with content, is what
    /// `blocking-findings-disposed` reads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    resolution: Option<String>,
    /// The inquiry a `cp-` subject disposes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    disposes: Option<DesignId>,
    /// The disposition Doctrine is asked to **effect** (DEC-062, DEC-086).
    ///
    /// One tagged field rather than four optional ones, so the four forms are
    /// mutually exclusive by construction and the vocabulary has one owner
    /// ([`DispositionForm`]).
    ///
    /// The **only** spelling of a disposition (EX-12). A `record`/`adopt_record`
    /// pair used to sit beside it saying *this already happened*, reaching
    /// [`super::inquiry::Disposition::Adopted`] and `Created` without passing the
    /// adoption validation this field's `adopt` form goes through. Two ways to
    /// say one thing, one of them unvalidated, is the defect — not a
    /// convenience.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    dispose: Option<Dispose>,
    /// The canonical id Doctrine claimed for a `create` disposition.
    ///
    /// `skip`ped on the wire in **both** directions, which is the mechanism and
    /// not a formality: a caller cannot name the id of a record Doctrine has not
    /// yet claimed, so the only route to this field is
    /// [`Declaration::resolved_record`], called by the shell between DEC-086
    /// steps 4 and 6.
    #[serde(skip)]
    resolved_record: Option<String>,
}

/// A disposition Doctrine is asked to effect — the four DEC-062 forms, tagged by
/// [`DispositionForm`]'s own token vocabulary.
///
/// The two record-bearing forms differ in *who materialises the record*: `create`
/// runs the DEC-086 six-step protocol and hands back a canonical id, `adopt`
/// binds one that already exists. The two note-bearing forms produce no record at
/// all, which is exactly why they exist — without them a resolved node that
/// produced no record is unrepresentable, and "we discussed it" gets laundered
/// into a durable claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "form")]
pub(crate) enum Dispose {
    /// Doctrine creates a durable knowledge record for this inquiry.
    Create(CreateRecord),
    /// An existing canonical record is adopted.
    Adopt { record: String },
    /// The outcome is explicitly retained as unresolved.
    Unresolved { note: String },
    /// The exchange is intentionally non-durable.
    NonDurable { note: String },
}

impl Dispose {
    /// Which form this is — read off the value, never supplied beside it.
    pub(crate) const fn form(&self) -> DispositionForm {
        match self {
            Dispose::Create(_) => DispositionForm::Create,
            Dispose::Adopt { .. } => DispositionForm::Adopt,
            Dispose::Unresolved { .. } => DispositionForm::RetainUnresolved,
            Dispose::NonDurable { .. } => DispositionForm::NonDurable,
        }
    }
}

/// One facet value as it arrives on the wire — free text, or a list of strings.
///
/// **Why not `knowledge::RawValue`, which this maps onto one-for-one (SL-249
/// `D-A`).** Same reason [`CreateRecord::kind`] is a `String`: this is a leaf
/// with crate out-degree zero (ADR-001, `layering.toml`), so it may not name a
/// command-tier type, and the shell does the mapping. Untagged, so the wire form
/// is the natural JSON — `"x"` or `["a","b"]` — with no tag a caller must supply.
///
/// **Why not spelled `FacetValue`.** `CHR-060` is an open chore aimed at that
/// name for `facet_write::FacetField`; taking it here would plant the very
/// two-modules-one-name defect `ISS-329` was raised for, and would foreclose an
/// open backlog item by squatting on its target. `WireKey` below is the
/// same-file precedent for the `Wire` prefix: the wire form of a concept the
/// crate already names elsewhere takes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum WireFacetValue {
    List(Vec<String>),
    Text(String),
}

/// What a `create` disposition asks Doctrine to materialise.
///
/// `kind` is a **string** and stays one: this is a leaf with crate out-degree
/// zero, so the knowledge-kind vocabulary lives where it already lives
/// (`crate::knowledge::RecordKind`) and the shell resolves the token against it.
/// A second enum here would be the parallel wire type there is no room for.
///
/// There is deliberately no `status` field. Status is not caller-settable through
/// a checkpoint: a created record keeps its kind's seeded default unless an
/// [`AcceptanceDeclaration`] unlocks the accepted state (DEC-088), which is the
/// whole of "a semantic payload cannot self-declare accepted status".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CreateRecord {
    /// The knowledge kind's token — `decision`, `question`, `assumption`, …
    pub(crate) kind: String,
    /// The record's title.
    pub(crate) title: String,
    /// An explicit slug; else derived from the title the way `knowledge new` does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) slug: Option<String>,
    /// The record's `.md` prose (SL-249).
    ///
    /// Spelled `body`, not `prose`: [`Declaration::body`] is a design
    /// *section's* body, this is a *record's* body, and both name the same
    /// thing — the `.md` content of what the subject names — which is what
    /// `entity::write_body` and `memory edit --body` already call it. The
    /// SL-248 loss this closes was a level error (record prose sent through a
    /// slot for section prose), not a naming collision, so the fix keeps the
    /// established word rather than inventing a second one for the same
    /// concept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) body: Option<String>,
    /// The record's `[facet]` content, keyed by field name (SL-249).
    ///
    /// A **map**, not a typed per-kind struct: [`CreateRecord::kind`] is a
    /// runtime token, so no per-kind struct can be selected at compile time. The
    /// keys are therefore unvalidated here and validated at admission by
    /// `knowledge::plan_facet_edits` — the same function the CLI runs, before any
    /// id is reserved (`D5`).
    ///
    /// `BTreeMap` and not a `HashMap`: its serde form is key-ordered, so the
    /// payload digest is stable under a caller's key order (§5.3).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) facet: BTreeMap<String, WireFacetValue>,
    /// The user's acceptance of this record as true (DEC-088).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) acceptance: Option<AcceptanceDeclaration>,
}

impl CreateRecord {
    /// A `CreateRecord` carrying **every** wire key, for `sec-8` pin 1.
    ///
    /// An exhaustive struct literal with no `..` update syntax, deliberately:
    /// that is what makes a newly added field a compile error *here*, before it
    /// can be a silently missing row in `payload_contract::CREATE_RECORD`. No
    /// value may be one its own `skip_serializing_if` would drop — `facet` is
    /// non-empty for exactly that reason — or the key leaves the wire and the
    /// key-set equality passes on a smaller set.
    #[cfg(test)]
    pub(super) fn fully_populated() -> CreateRecord {
        CreateRecord {
            kind: "decision".to_owned(),
            title: "A sample record".to_owned(),
            slug: Some("a-sample-record".to_owned()),
            body: Some("## A sample record\n".to_owned()),
            facet: BTreeMap::from([
                (
                    "basis".to_owned(),
                    WireFacetValue::Text("observation".to_owned()),
                ),
                (
                    "supersedes".to_owned(),
                    WireFacetValue::List(vec!["DEC-000".to_owned()]),
                ),
            ]),
            acceptance: Some(AcceptanceDeclaration::fully_populated()),
        }
    }
}

/// The user-acceptance half a caller may supply (DEC-088).
///
/// Authority is not a field: this declaration *is* the user's, and offering an
/// `authority` key would let a payload claim one. Nor is the digest — Doctrine
/// derives and binds it over the checkpoint payload, the disposition and the run
/// revision, so an acceptance cannot be transplanted onto different content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AcceptanceDeclaration {
    /// Why the user accepts it — concise and required.
    pub(crate) basis: String,
    /// The harness turn the acceptance was given in, when the caller knows it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) turn: Option<String>,
}

impl AcceptanceDeclaration {
    /// An `AcceptanceDeclaration` carrying **every** wire key, for `sec-8`
    /// pin 1.
    ///
    /// An exhaustive struct literal with no `..` update syntax, deliberately:
    /// that is what makes a newly added field a compile error *here*, before it
    /// can be a silently missing row in
    /// `payload_contract::ACCEPTANCE_DECLARATION`. No value may be one its own
    /// `skip_serializing_if` would drop, or the key leaves the wire and the
    /// key-set equality passes on a smaller set.
    #[cfg(test)]
    pub(super) fn fully_populated() -> AcceptanceDeclaration {
        AcceptanceDeclaration {
            basis: "the user accepts it as true".to_owned(),
            turn: Some("turn-7".to_owned()),
        }
    }
}

/// What a caller may say about one runbook step (SL-233 PHASE-16, `EX-7`).
///
/// **Two arms, not three.** The record distinguishes attested / verified /
/// skipped, but `verified` is not on this wire: a verifier result is exactly the
/// kind of fact Doctrine must derive rather than accept on a caller's word. Gate
/// conditions make the same point one tier stronger since `SL-244`: there is no
/// wire slot to claim one through, so the category error is unspellable rather
/// than refused. Whether an admitted discharge is recorded attested
/// or verified is decided by the step's own definition and by what the shell
/// executed, never by what the payload said.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DischargeClaim {
    /// The agent says it did the work.
    Attested,
    /// The agent cannot do the work, and says why.
    Skipped,
}

/// One discharge act: which step, and what the caller claims about it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DischargeDeclaration {
    /// The step id — the one at the cursor, or the act is refused.
    pub(crate) step: String,
    pub(crate) outcome: DischargeClaim,
    /// Required when [`DischargeClaim::Skipped`], and checked at admission
    /// rather than by serde so the failure is a typed `Refusal` naming the step
    /// (the [`AcceptanceDeclaration::basis`] precedent).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) reason: Option<String>,
}

impl DischargeDeclaration {
    /// A `DischargeDeclaration` carrying **every** wire key, for `sec-8` pin 1.
    ///
    /// An exhaustive struct literal with no `..` update syntax, deliberately:
    /// that is what makes a newly added field a compile error *here*, before it
    /// can be a silently missing row in
    /// `payload_contract::DISCHARGE_DECLARATION`. No value may be one its own
    /// `skip_serializing_if` would drop, or the key leaves the wire and the
    /// key-set equality passes on a smaller set.
    #[cfg(test)]
    pub(super) fn fully_populated() -> DischargeDeclaration {
        DischargeDeclaration {
            step: "run-the-suite".to_owned(),
            outcome: DischargeClaim::Attested,
            reason: Some("the suite is green".to_owned()),
        }
    }
}

impl Declaration {
    /// A declaration naming `subject` and asserting nothing.
    pub(crate) const fn about(subject: DesignId) -> Self {
        Declaration {
            subject,
            question: Sparse::Omitted,
            needs: Sparse::Omitted,
            parent: Sparse::Omitted,
            provenance: None,
            lifecycle: None,
            body: None,
            attests: None,
            reviewer: None,
            concerns: None,
            summary: None,
            blocking: None,
            resolution: None,
            disposes: None,
            dispose: None,
            resolved_record: None,
        }
    }

    /// Assert the question text.
    #[must_use]
    pub(crate) fn question(mut self, question: Sparse<String>) -> Self {
        self.question = question;
        self
    }

    /// Assert the `needs` collection.
    #[must_use]
    #[expect(
        dead_code,
        reason = "SL-233: PHASE-03 declares `needs` over the JSON wire, not through this builder; \
                  PHASE-04's envelope construction is its first Rust caller"
    )]
    pub(crate) fn needs(mut self, needs: Sparse<Vec<DesignId>>) -> Self {
        self.needs = needs;
        self
    }

    /// The subject this declaration is about.
    pub(crate) const fn subject(&self) -> &DesignId {
        &self.subject
    }

    /// The asserted question.
    pub(crate) const fn question_declaration(&self) -> &Sparse<String> {
        &self.question
    }

    /// The asserted `needs`.
    pub(crate) const fn needs_declaration(&self) -> &Sparse<Vec<DesignId>> {
        &self.needs
    }

    /// The asserted primary parent.
    pub(crate) const fn parent_declaration(&self) -> &Sparse<DesignId> {
        &self.parent
    }

    /// The asserted provenance, if the caller stated one.
    pub(crate) const fn provenance(&self) -> Option<&Provenance> {
        self.provenance.as_ref()
    }

    /// The asserted non-resolving lifecycle move.
    pub(crate) const fn lifecycle(&self) -> Option<InquiryLifecycle> {
        self.lifecycle
    }

    /// The asserted section prose.
    pub(crate) fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    /// The section an attestation binds to.
    pub(crate) const fn attests(&self) -> Option<&DesignId> {
        self.attests.as_ref()
    }

    /// Who reviewed.
    pub(crate) const fn reviewer(&self) -> Option<Reviewer> {
        self.reviewer
    }

    /// The section this finding concerns.
    pub(crate) const fn concerns(&self) -> Option<&DesignId> {
        self.concerns.as_ref()
    }

    /// What this finding says.
    pub(crate) fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }

    /// Whether this finding blocks the lock gate, defaulting to non-blocking.
    pub(crate) fn blocking(&self) -> bool {
        self.blocking.unwrap_or(false)
    }

    /// How this finding was disposed.
    pub(crate) fn resolution(&self) -> Option<&str> {
        self.resolution.as_deref()
    }

    /// The inquiry a checkpoint disposes.
    pub(crate) const fn disposes(&self) -> Option<&DesignId> {
        self.disposes.as_ref()
    }

    /// The disposition Doctrine is asked to effect.
    pub(crate) const fn dispose(&self) -> Option<&Dispose> {
        self.dispose.as_ref()
    }

    /// The canonical id Doctrine claimed for a `create` disposition, once
    /// DEC-086 step 3 has journalled it.
    pub(crate) fn resolved_record(&self) -> Option<&str> {
        self.resolved_record.as_deref()
    }

    /// Bind the canonical id Doctrine claimed for this checkpoint (DEC-086).
    ///
    /// The shell calls this between the id claim and the snapshot write, which is
    /// why the field has no wire route: a `create` declaration that reached the
    /// candidate without one is an ordering bug, and it is refused as one.
    #[must_use]
    pub(crate) fn resolving(mut self, record: impl Into<String>) -> Self {
        self.resolved_record = Some(record.into());
        self
    }
}

// ── the wire-key correspondence (SL-249, ISS-318, design §5.3) ─────────────

/// Each [`Declaration`] wire key, as a named constant.
///
/// The [`WRITER_ACT_STAGE`] idiom and its reason: these are the keys a caller
/// reads in a refusal and then removes from their own JSON, so the refusal has
/// to name the key that is actually there (STD-001).
const KEY_SUBJECT: &str = "subject";
const KEY_QUESTION: &str = "question";
const KEY_NEEDS: &str = "needs";
const KEY_PARENT: &str = "parent";
const KEY_PROVENANCE: &str = "provenance";
const KEY_LIFECYCLE: &str = "lifecycle";
const KEY_BODY: &str = "body";
const KEY_ATTESTS: &str = "attests";
const KEY_REVIEWER: &str = "reviewer";
const KEY_CONCERNS: &str = "concerns";
const KEY_SUMMARY: &str = "summary";
const KEY_BLOCKING: &str = "blocking";
const KEY_RESOLUTION: &str = "resolution";
const KEY_DISPOSES: &str = "disposes";
const KEY_DISPOSE: &str = "dispose";

/// The nested key a record's prose actually rides (SL-249 PHASE-01) — the one
/// remedy the correspondence can name today.
const KEY_DISPOSE_CREATE_BODY: &str = "dispose.create.body";

/// Where a wire key is honoured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyHome {
    /// The addressing key: carried by every declaration, inert at none.
    Universal,
    /// Honoured at exactly one subject kind, and inert at every other.
    At(IdKind),
}

/// One row of the correspondence: the key, where it is honoured, and the test
/// for whether a declaration carries it.
type WireKey = (&'static str, KeyHome, fn(&Declaration) -> bool);

/// The key the caller wanted, where their intent is unambiguous (SL-249 `EX-2`).
///
/// A table rather than a clause inlined in the refusal, and a small one: there
/// is no rule deriving a remedy from a pair, so each row is a judgement someone
/// made. `body` on a checkpoint is `SL-248`'s loss exactly — record prose sent
/// through the slot for *section* prose — and the key it wanted is the one
/// PHASE-01 added.
fn remedy(key: &str, kind: IdKind) -> Option<&'static str> {
    match (key, kind) {
        (KEY_BODY, IdKind::Checkpoint) => Some(KEY_DISPOSE_CREATE_BODY),
        _ => None,
    }
}

impl Declaration {
    /// The `Declaration` wire key → honouring subject kind correspondence
    /// (design § 5.3, `ISS-318`).
    ///
    /// One flat struct carries every key while each is honoured at exactly one
    /// subject kind, and serde cannot express that — so the table does, and
    /// [`Declaration::inert_key`] is the check that reads it.
    ///
    /// Each row pairs its key with the **predicate that tests for it**, on
    /// [`ApplyRequest::WRITER_ACTS`]'s reasoning: the predicates ARE the guard,
    /// so a row cannot be added without saying how presence is detected, and a
    /// row whose predicate reads the wrong field is caught by a test rather than
    /// by review. Pinned two independent ways — `I9`, that this key set equals a
    /// fully populated declaration's serde key set, and `I10`, the behavioural
    /// matrix. `I9` alone proves inventory and not mapping (`RV-349` `F-4`).
    ///
    /// `resolved_record` is absent because it carries `#[serde(skip)]` and is
    /// therefore not a wire key — excluded by construction rather than by an
    /// exception list (`EX-3`).
    pub(crate) const WIRE_KEYS: [WireKey; 15] = [
        (KEY_SUBJECT, KeyHome::Universal, |_| true),
        (KEY_QUESTION, KeyHome::At(IdKind::Inquiry), |declared| {
            !declared.question.is_omitted()
        }),
        (KEY_NEEDS, KeyHome::At(IdKind::Inquiry), |declared| {
            !declared.needs.is_omitted()
        }),
        (KEY_PARENT, KeyHome::At(IdKind::Inquiry), |declared| {
            !declared.parent.is_omitted()
        }),
        (KEY_PROVENANCE, KeyHome::At(IdKind::Inquiry), |declared| {
            declared.provenance.is_some()
        }),
        (KEY_LIFECYCLE, KeyHome::At(IdKind::Inquiry), |declared| {
            declared.lifecycle.is_some()
        }),
        (KEY_BODY, KeyHome::At(IdKind::Section), |declared| {
            declared.body.is_some()
        }),
        (KEY_ATTESTS, KeyHome::At(IdKind::Attestation), |declared| {
            declared.attests.is_some()
        }),
        (KEY_REVIEWER, KeyHome::At(IdKind::Attestation), |declared| {
            declared.reviewer.is_some()
        }),
        (KEY_CONCERNS, KeyHome::At(IdKind::Finding), |declared| {
            declared.concerns.is_some()
        }),
        (KEY_SUMMARY, KeyHome::At(IdKind::Finding), |declared| {
            declared.summary.is_some()
        }),
        (KEY_BLOCKING, KeyHome::At(IdKind::Finding), |declared| {
            declared.blocking.is_some()
        }),
        (KEY_RESOLUTION, KeyHome::At(IdKind::Finding), |declared| {
            declared.resolution.is_some()
        }),
        (KEY_DISPOSES, KeyHome::At(IdKind::Checkpoint), |declared| {
            declared.disposes.is_some()
        }),
        (KEY_DISPOSE, KeyHome::At(IdKind::Checkpoint), |declared| {
            declared.dispose.is_some()
        }),
    ];

    /// The first key this declaration carries that is inert at its subject's
    /// kind, as the refusal a caller should see (`ISS-318`).
    ///
    /// **The first, not every one**, unlike [`super::admission::admit_act`]'s
    /// collect-them-all rule: a caller who spelled one key at the wrong subject
    /// has almost always spelled the *declaration* at the wrong subject, so the
    /// second key is a consequence of the first mistake rather than a second one
    /// to repair.
    ///
    /// `None` for a subject that may not be declared at all
    /// ([`IdKind::declarable`]): the engine refuses those whole, and its message
    /// names the run-level field to use — shadowing it here would trade a remedy
    /// for a symptom.
    ///
    /// Scoped to the **kind** axis (`DEC-183`). A key honoured at this kind but
    /// ignored in this subject's *state* is `ISS-327`, not this.
    pub(crate) fn inert_key(&self) -> Option<Refusal> {
        let kind = self.subject.kind();
        if !kind.declarable() {
            return None;
        }
        Declaration::WIRE_KEYS
            .iter()
            .find_map(|&(key, home, carried)| match home {
                KeyHome::At(honoured_by) if honoured_by != kind && carried(self) => {
                    Some(Refusal::InertKey {
                        subject: self.subject.clone(),
                        key,
                        honoured_by,
                        remedy: remedy(key, kind),
                    })
                }
                KeyHome::Universal | KeyHome::At(_) => None,
            })
    }

    /// A declaration carrying **every** wire key, for `I9`.
    ///
    /// An exhaustive struct literal with no `..` update syntax, deliberately:
    /// that is what makes a newly added field a compile error *here*, before it
    /// can be a silently missing row in [`Declaration::WIRE_KEYS`]. The values
    /// are arbitrary — only each key's presence on the wire is observed — but
    /// none may be a value its field's `skip_serializing_if` would drop.
    #[cfg(test)]
    pub(super) fn fully_populated(subject: DesignId) -> Declaration {
        Declaration {
            subject,
            question: Sparse::Value("why?".to_owned()),
            needs: Sparse::Value(Vec::new()),
            parent: Sparse::Value(DesignId::parse("inq-0").expect("a literal id")),
            provenance: Some(Provenance::AgentProposed),
            lifecycle: Some(InquiryLifecycle::Open),
            body: Some("## a section\n".to_owned()),
            attests: Some(DesignId::parse("sec-0").expect("a literal id")),
            reviewer: Some(Reviewer::Human),
            concerns: Some(DesignId::parse("sec-0").expect("a literal id")),
            summary: Some("a finding".to_owned()),
            blocking: Some(true),
            resolution: Some("disposed".to_owned()),
            disposes: Some(DesignId::parse("inq-0").expect("a literal id")),
            dispose: Some(Dispose::Unresolved {
                note: "retained".to_owned(),
            }),
            resolved_record: Some("DEC-000".to_owned()),
        }
    }
}

// ── the apply payload ──────────────────────────────────────────────────────

/// What every `apply` payload asserts, irrespective of the optional CLI
/// addressing flags (design §5.2, EX-9).
///
/// All three are mandatory and none is defaultable: run identity catches a stale
/// context, `known_revision` is the compare-and-swap, and `submission_id` is the
/// idempotency key. A payload that may omit one of them cannot be made
/// idempotent later without breaking every caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SubmissionEnvelope {
    pub(crate) run_uid: String,
    pub(crate) known_revision: u64,
    pub(crate) submission_id: String,
}

impl SubmissionEnvelope {
    /// A `SubmissionEnvelope` carrying **every** wire key, for `sec-8` pin 1.
    ///
    /// An exhaustive struct literal with no `..` update syntax, deliberately:
    /// that is what makes a newly added field a compile error *here*, before it
    /// can be a silently missing row in `payload_contract::PAYLOAD` — this
    /// struct's three keys are rendered **at the root** by `#[serde(flatten)]`,
    /// so it is the one closure struct with no `TypeContract` of its own and its
    /// pin is the root's disjoint union rather than a key set of its own.
    #[cfg(test)]
    pub(super) fn fully_populated() -> SubmissionEnvelope {
        SubmissionEnvelope {
            run_uid: "run-0000".to_owned(),
            known_revision: 7,
            submission_id: "sub-0001".to_owned(),
        }
    }
}

/// The sole lawful crossing of an authored-watermark divergence (DEC-092 rule 2)
/// — a protocol, not a bypass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AdoptAuthored {
    /// The exact current fingerprint of `design.md`, as the caller reads it. It
    /// must match what Doctrine reads.
    pub(crate) fingerprint: String,
    /// The complete stable-marker map: every section Doctrine knows, and no
    /// other, each naming the fingerprint the caller read for it.
    #[serde(default)]
    pub(crate) sections: BTreeMap<DesignId, String>,
}

impl AdoptAuthored {
    /// An `AdoptAuthored` carrying **every** wire key, for `sec-8` pin 1.
    ///
    /// An exhaustive struct literal with no `..` update syntax, deliberately:
    /// that is what makes a newly added field a compile error *here*, before it
    /// can be a silently missing row in `payload_contract::ADOPT_AUTHORED`. No
    /// value may be one its own `skip_serializing_if` would drop, or the key
    /// leaves the wire and the key-set equality passes on a smaller set.
    #[cfg(test)]
    pub(super) fn fully_populated() -> AdoptAuthored {
        AdoptAuthored {
            fingerprint: "sha256:design".to_owned(),
            sections: BTreeMap::from([(
                DesignId::parse("sec-0").expect("a literal id"),
                "sha256:sec-0".to_owned(),
            )]),
        }
    }
}

/// A declared stage move. A backward move carries its reason (DEC-067); the type
/// keeps the reason optional only because a *forward* move has none.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StageDeclaration {
    pub(crate) to: Stage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) reason: Option<String>,
}

impl StageDeclaration {
    /// A `StageDeclaration` carrying **every** wire key, for `sec-8` pin 1.
    ///
    /// An exhaustive struct literal with no `..` update syntax, deliberately:
    /// that is what makes a newly added field a compile error *here*, before it
    /// can be a silently missing row in `payload_contract::STAGE_DECLARATION`.
    /// No value may be one its own `skip_serializing_if` would drop, or the key
    /// leaves the wire and the key-set equality passes on a smaller set.
    #[cfg(test)]
    pub(super) fn fully_populated() -> StageDeclaration {
        StageDeclaration {
            to: Stage::Drafting,
            reason: Some("the inquiry map is sufficient".to_owned()),
        }
    }
}

/// A declared change of traversal direction (design §5.3, EX-8).
///
/// Pin, cursor and posture are separate fields carrying one shared
/// [`Authority`], because *where attention sits*, *what must stay visible* and
/// *how the map is being walked* are independent facts — folding them into one
/// value is the accidental hierarchical state machine R3 forbids.
///
/// `authority` defaults to [`Authority::AgentProposed`], which is the
/// conservative reading and the whole of R2/R12's answer: user direction must be
/// **stated**, never inferred from the fact that something was declared. Defer
/// and prune are not here — they are node lifecycle transitions, declared
/// against the node they move, and they carry the same distinction through
/// [`Provenance`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TraversalDeclaration {
    /// The node to keep visible every turn. `null` clears the pin.
    #[serde(default, skip_serializing_if = "Sparse::is_omitted")]
    pub(crate) pin: Sparse<DesignId>,
    /// Where attention moves to. `null` clears the cursor.
    #[serde(default, skip_serializing_if = "Sparse::is_omitted")]
    pub(crate) cursor: Sparse<DesignId>,
    /// Breadth-first across major branches, or depth-first down one arm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) posture: Option<Posture>,
    /// Who is speaking.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) authority: Option<Authority>,
}

impl TraversalDeclaration {
    /// Whether this declaration asserts anything at all.
    pub(crate) const fn is_empty(&self) -> bool {
        self.pin.is_omitted() && self.cursor.is_omitted() && self.posture.is_none()
    }

    /// On whose authority, defaulting to agent-proposed.
    pub(crate) fn authority(&self) -> Authority {
        self.authority.unwrap_or(Authority::AgentProposed)
    }

    /// A `TraversalDeclaration` carrying **every** wire key, for `sec-8` pin 1.
    ///
    /// An exhaustive struct literal with no `..` update syntax, deliberately:
    /// that is what makes a newly added field a compile error *here*, before it
    /// can be a silently missing row in
    /// `payload_contract::TRAVERSAL_DECLARATION`. No value may be one its own
    /// `skip_serializing_if` would drop — every `Sparse` field is a `Value`, and
    /// the whole declaration must fail [`TraversalDeclaration::is_empty`] or
    /// `ApplyRequest`'s own `traversal` key leaves the wire with it.
    #[cfg(test)]
    pub(super) fn fully_populated() -> TraversalDeclaration {
        TraversalDeclaration {
            pin: Sparse::Value(DesignId::parse("inq-0").expect("a literal id")),
            cursor: Sparse::Value(DesignId::parse("inq-1").expect("a literal id")),
            posture: Some(Posture::Depth),
            authority: Some(Authority::UserPinned),
        }
    }
}

// ── delegation (SL-233 PHASE-10, DEC-068) ─────────────────────────────────

/// The wire key each writer act is named by — and the noun
/// [`Refusal::DelegateCannotAdvance`] reports (STD-001).
///
/// Named constants rather than literals in the guard because they are the payload
/// keys a caller reads in the refusal and then removes from their own JSON: the
/// refusal has to name the key that is actually there.
const WRITER_ACT_STAGE: &str = "stage";
const WRITER_ACT_ACCEPTANCE: &str = "acceptance";
const WRITER_ACT_ADOPT_AUTHORED: &str = "adopt_authored";
const WRITER_ACT_DECLARE: &str = "declare";
const WRITER_ACT_TRAVERSAL: &str = "traversal";
/// The seventh act (SL-233 PHASE-16). `discharge` rather than `attest`: the
/// latter reads closer to the semantics but collides twice with bindings that
/// are content-bound — [`Declaration::attests`] (a section attestation,
/// DEC-073) and [`AcceptanceDeclaration`] (DEC-088). A runbook discharge binds
/// an asset *definition*, not run content, so borrowing either word would give
/// two different bindings one name.
const WRITER_ACT_DISCHARGE: &str = "discharge";
/// The eighth act (SL-244 PHASE-03, DEC-073). Changing which reviewer lanes a
/// run requires is a user judgement, not housekeeping, so it is its own act
/// rather than a field an agent can move in passing.
const WRITER_ACT_REVIEW_POLICY: &str = "review_policy";
/// The ninth and tenth acts (SL-244 PHASE-05, DEC-121). A recorded act is the
/// only thing an attested condition reads, so a payload carrying one changes what
/// the gate will say — which is precisely what makes it a writer act.
const WRITER_ACT_CHECKPOINT_ACT: &str = "checkpoint_act";
const WRITER_ACT_AGENT_DECLARATION: &str = "agent_declaration";

/// A change to the run's review policy, which is a user act like any other
/// (DEC-073, ISS-310).
///
/// The acceptance is **required rather than optional**, and that is the whole
/// fence: the policy is mutable on purpose — a user may legitimately change their
/// mind, and a rule nobody can revise is one they route around by hand-editing
/// runtime state — so what the design buys is not prohibition but visibility.
/// Loosening the policy is refused without a declaration carrying a non-empty
/// basis, and it leaves a change row naming both values.
///
/// Stated plainly because the design should not imply more than it delivers: the
/// review policy is a declaration of intent, not a security boundary. In
/// particular this declaration is **not** bound into an
/// [`AcceptanceAttestation`] — the policy is a scalar on the run header with no
/// record to hold one (RV-345 F-6). [`super::attestation::CheckpointAct`] is the
/// act that does bind, and it is the one that carries
/// `AcceptanceAuthority::User`.
///
/// [`AcceptanceAttestation`]: super::attestation::AcceptanceAttestation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ReviewPolicyDeclaration {
    pub(crate) policy: ReviewPolicy,
    pub(crate) acceptance: AcceptanceDeclaration,
}

impl ReviewPolicyDeclaration {
    /// A `ReviewPolicyDeclaration` carrying **every** wire key, for `sec-8`
    /// pin 1.
    ///
    /// An exhaustive struct literal with no `..` update syntax, deliberately:
    /// that is what makes a newly added field a compile error *here*, before it
    /// can be a silently missing row in
    /// `payload_contract::REVIEW_POLICY_DECLARATION`. The acceptance is composed
    /// from its own fixture rather than spelled again here, so a field joining
    /// `AcceptanceDeclaration` is one compile error and not four.
    #[cfg(test)]
    pub(super) fn fully_populated() -> ReviewPolicyDeclaration {
        ReviewPolicyDeclaration {
            policy: ReviewPolicy::HumanThenAdversarial,
            acceptance: AcceptanceDeclaration::fully_populated(),
        }
    }
}

/// A user act at a checkpoint, as a **caller** may state it (design `sec-4`,
/// `EX-7b`).
///
/// The record it becomes ([`super::attestation::CheckpointAct`]) also holds an
/// allocated `id`, a covered map, observed fingerprints and a confirmed
/// declaration's digest — every one of them engine-authored from state the caller
/// cannot assert. So they are **absent here**, and `deny_unknown_fields` is what
/// makes that absence a refusal rather than a key serde quietly swallows: a
/// caller who supplies `covered` would otherwise be told nothing and get the
/// engine's value, which is the silent-no-op class [`Declaration`]'s own
/// `deny_unknown_fields` exists to close. Admission cannot cover for it — by then
/// the key is already gone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CheckpointActDeclaration {
    /// Which of the five checkpoint acts this is.
    pub(crate) act: ActKind,
    /// The user's acceptance, carrying the basis. Routed through
    /// [`AcceptanceAttestation::bind`] at construction, so authority stays
    /// unclaimable (DEC-088).
    ///
    /// [`AcceptanceAttestation::bind`]: super::attestation::AcceptanceAttestation::bind
    pub(crate) acceptance: AcceptanceDeclaration,
    /// How the act disposes of the current pass. Present exactly on
    /// [`ActKind::ReviewDisposed`], per the rule/record correspondence — and
    /// checked there rather than here, so the failure is a typed refusal naming
    /// the act (the [`DischargeDeclaration::reason`] precedent).
    ///
    /// The **pass** it disposes is not on the wire: the run knows which pass is
    /// current and a caller naming a different one could only be wrong.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) disposition: Option<ReviewDisposition>,
}

impl CheckpointActDeclaration {
    /// A `CheckpointActDeclaration` carrying **every** wire key, for `sec-8`
    /// pin 1.
    ///
    /// An exhaustive struct literal with no `..` update syntax, deliberately:
    /// that is what makes a newly added field a compile error *here*, before it
    /// can be a silently missing row in
    /// `payload_contract::CHECKPOINT_ACT_DECLARATION`. No value may be one its
    /// own `skip_serializing_if` would drop, or the key leaves the wire and the
    /// key-set equality passes on a smaller set.
    #[cfg(test)]
    pub(super) fn fully_populated() -> CheckpointActDeclaration {
        CheckpointActDeclaration {
            act: ActKind::ReviewDisposed,
            acceptance: AcceptanceDeclaration::fully_populated(),
            disposition: Some(ReviewDisposition::Conducted {
                review: super::attestation::ReviewRef::new("RV-001"),
            }),
        }
    }
}

/// An agent's declaration about its own work, as the agent states it (DEC-121,
/// `EX-7b`).
///
/// `act` is the payload-tagged [`AgentAct`] rather than its discriminant: a wire
/// carrying only the kind could not deliver a blocking set. The engine supplies
/// the `id`, the covered map and the claim fingerprint — the last being why a
/// declaration and the act confirming it can arrive in one submission with no
/// caller-computed digest — so none of the three is spellable here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentActDeclaration {
    /// The act with what it declares.
    pub(crate) act: AgentAct,
    /// The stated basis. Non-blank, checked at admission.
    pub(crate) basis: String,
    /// The harness turn it was declared in, when the caller knows it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) turn: Option<String>,
}

impl AgentActDeclaration {
    /// An `AgentActDeclaration` carrying **every** wire key, for `sec-8` pin 1.
    ///
    /// An exhaustive struct literal with no `..` update syntax, deliberately:
    /// that is what makes a newly added field a compile error *here*, before it
    /// can be a silently missing row in
    /// `payload_contract::AGENT_ACT_DECLARATION`. No value may be one its own
    /// `skip_serializing_if` would drop, or the key leaves the wire and the
    /// key-set equality passes on a smaller set.
    #[cfg(test)]
    pub(super) fn fully_populated() -> AgentActDeclaration {
        AgentActDeclaration {
            act: AgentAct::BlockingSetDeclared {
                blocking: std::collections::BTreeSet::from([
                    DesignId::parse("inq-0").expect("a literal id")
                ]),
            },
            basis: "these three gate the draft".to_owned(),
            turn: Some("turn-7".to_owned()),
        }
    }
}

/// One delegation act (DEC-068).
///
/// **One tagged field, four acts** — the [`Dispose`] idiom, for the same reason:
/// the acts are mutually exclusive by construction, so there is no state in which
/// a payload asserts two of them and no conflict refusal to write for a case the
/// wire cannot express. Three optional sibling fields would have both.
///
/// `id` is uniform across the four: on `export` it is the assignment being cut, on
/// the other three the assignment being acted on. A caller never has to remember
/// which key this act spells its subject with.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "act")]
pub(crate) enum DelegationAct {
    /// The coordinator cuts an assignment for one bounded obligation.
    Export { id: DesignId, obligation: DesignId },
    /// The delegate proposes back: attribution, conclusion, and the map changes
    /// it proposes — **none of which this act applies**.
    Propose {
        id: DesignId,
        /// Who did the work. Attribution, not authentication.
        by: String,
        /// What they concluded. Prose, never interpreted.
        summary: String,
        /// The proposed map changes, in the same shape the coordinator's own
        /// declarations arrive in, so acceptance needs no second interpreter.
        #[serde(default)]
        declare: Vec<Declaration>,
    },
    /// The coordinator accepts: the proposal's declarations land as its own act.
    Accept { id: DesignId },
    /// The coordinator refuses, on the record.
    Refuse { id: DesignId, reason: String },
}

impl DelegationAct {
    /// The assignment this act is about.
    pub(crate) const fn id(&self) -> &DesignId {
        match self {
            DelegationAct::Export { id, .. }
            | DelegationAct::Propose { id, .. }
            | DelegationAct::Accept { id }
            | DelegationAct::Refuse { id, .. } => id,
        }
    }

    /// Whether this is the **delegate's** act rather than the coordinator's.
    ///
    /// The distinction is the act, not the actor: v1 has no authenticated worker
    /// identity and inventing one would be the authority laundering design R2/R12
    /// warns about. What can be checked is that a proposal-shaped payload carries
    /// nothing that writes.
    pub(crate) const fn is_proposal(&self) -> bool {
        matches!(self, DelegationAct::Propose { .. })
    }
}

/// One `apply` payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ApplyRequest {
    #[serde(flatten)]
    pub(crate) envelope: SubmissionEnvelope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) adopt_authored: Option<AdoptAuthored>,
    #[serde(default, skip_serializing_if = "TraversalDeclaration::is_empty")]
    pub(crate) traversal: TraversalDeclaration,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) stage: Option<StageDeclaration>,
    /// The user's acceptance of the design as locked (DEC-088).
    ///
    /// Run-level rather than a declaration: it is about the whole design, which
    /// has no run-local id, and it reuses the same wire type a checkpoint
    /// acceptance rides so authority and the digest stay unclaimable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) acceptance: Option<AcceptanceDeclaration>,
    #[serde(default)]
    pub(crate) declare: Vec<Declaration>,
    /// One delegation act (DEC-068). Run-level, like the acceptance: an
    /// assignment is about the run's obligation, not about a declaration subject.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) delegation: Option<DelegationAct>,
    /// One runbook discharge (SL-233 PHASE-16, DEC-101). Run-level, and an
    /// `Option` rather than a `Vec`: `sequence` mode admits exactly the step at
    /// the cursor, so a batch of two would need an order [`Batch`] explicitly
    /// refuses to carry (DEC-063).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) discharge: Option<DischargeDeclaration>,
    /// One review-policy change (DEC-073). Run-level, like the acceptance: the
    /// policy is a property of the run and not of any declaration subject.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) review_policy: Option<ReviewPolicyDeclaration>,
    /// One user act at a checkpoint (DEC-121). `Option` rather than a `Vec` for
    /// [`Batch`]'s reason: two acts in one submission would need an order the
    /// batch refuses to carry (DEC-063), and a second act of one kind would
    /// replace the first with no rule saying which won.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) checkpoint_act: Option<CheckpointActDeclaration>,
    /// One agent declaration (DEC-121), on the same terms — and beside the act
    /// above rather than inside it, because DEC-121's interaction is *the agent
    /// declares, the user confirms*: both may ride one submission, and the build
    /// order is what makes the confirmation land on the declaration made with it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) agent_declaration: Option<AgentActDeclaration>,
}

/// One writer act: the wire key a refusal names it by, and the test for whether a
/// payload carries it.
type WriterAct = (&'static str, fn(&ApplyRequest) -> bool);

impl ApplyRequest {
    /// Every writer act, each paired with the test for its presence — the closed
    /// vocabulary, single-sourced in [`Condition::ALL`](super::gate::Condition::ALL)'s shape (STD-001).
    ///
    /// Enumerated over the run-level fields rather than derived from one flag,
    /// because "does this payload write" is a question about the *whole* payload:
    /// a new field that can change state and did not join this list is the gap,
    /// and an exhaustive list is at least visible in review. The order is the
    /// order a caller most likely meant to remove them in.
    ///
    /// The pairing is what makes the list authoritative rather than merely
    /// parallel to [`Self::writer_act`]: the predicates ARE the guard, so a new
    /// act cannot be checked without joining the vocabulary a test can enumerate.
    /// A bare list of keys beside a hand-written branch chain would let a seventh
    /// branch widen the class silently, which is the shape `RV-324` F-6 found in
    /// the e2e table.
    pub(crate) const WRITER_ACTS: [WriterAct; 9] = [
        (WRITER_ACT_STAGE, |request| request.stage.is_some()),
        (WRITER_ACT_DECLARE, |request| !request.declare.is_empty()),
        (WRITER_ACT_ACCEPTANCE, |request| {
            request.acceptance.is_some()
        }),
        (WRITER_ACT_ADOPT_AUTHORED, |request| {
            request.adopt_authored.is_some()
        }),
        (WRITER_ACT_TRAVERSAL, |request| {
            !request.traversal.is_empty()
        }),
        (WRITER_ACT_DISCHARGE, |request| request.discharge.is_some()),
        (WRITER_ACT_REVIEW_POLICY, |request| {
            request.review_policy.is_some()
        }),
        (WRITER_ACT_CHECKPOINT_ACT, |request| {
            request.checkpoint_act.is_some()
        }),
        (WRITER_ACT_AGENT_DECLARATION, |request| {
            request.agent_declaration.is_some()
        }),
    ];

    /// The first writer act this payload carries, if any (`EX-2`).
    pub(crate) fn writer_act(&self) -> Option<&'static str> {
        Self::WRITER_ACTS
            .iter()
            .find(|(_, carried)| carried(self))
            .map(|(act, _)| *act)
    }

    /// An `ApplyRequest` carrying **every** wire key, for `sec-8` pin 1.
    ///
    /// An exhaustive struct literal with no `..` update syntax, deliberately:
    /// that is what makes a newly added field a compile error *here*, before it
    /// can be a silently missing row in `payload_contract::PAYLOAD`. No value
    /// may be one its own `skip_serializing_if` would drop — `traversal` in
    /// particular is composed from a declaration that fails
    /// [`TraversalDeclaration::is_empty`], or its key would leave the wire and
    /// the key-set equality would pass on a smaller set.
    ///
    /// Every nested value is composed by **calling** the type's own fixture
    /// rather than spelled again here: a field joining `AcceptanceDeclaration`
    /// is then one compile error at that fixture, not one per site, which is the
    /// same drift this contract exists to prevent.
    #[cfg(test)]
    pub(super) fn fully_populated() -> ApplyRequest {
        ApplyRequest {
            envelope: SubmissionEnvelope::fully_populated(),
            adopt_authored: Some(AdoptAuthored::fully_populated()),
            traversal: TraversalDeclaration::fully_populated(),
            stage: Some(StageDeclaration::fully_populated()),
            acceptance: Some(AcceptanceDeclaration::fully_populated()),
            declare: vec![Declaration::fully_populated(
                DesignId::parse("inq-1").expect("a literal id"),
            )],
            delegation: Some(DelegationAct::Propose {
                id: DesignId::parse("dlg-1").expect("a literal id"),
                by: "a delegate".to_owned(),
                summary: "what it concluded".to_owned(),
                declare: vec![Declaration::fully_populated(
                    DesignId::parse("inq-2").expect("a literal id"),
                )],
            }),
            discharge: Some(DischargeDeclaration::fully_populated()),
            review_policy: Some(ReviewPolicyDeclaration::fully_populated()),
            checkpoint_act: Some(CheckpointActDeclaration::fully_populated()),
            agent_declaration: Some(AgentActDeclaration::fully_populated()),
        }
    }
}

/// One `apply` payload: an unordered set of declarations.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Batch {
    declarations: Vec<Declaration>,
}

impl Batch {
    /// Build a batch in submission order.
    pub(crate) const fn of(declarations: Vec<Declaration>) -> Self {
        Batch { declarations }
    }

    /// Validate the whole candidate before any mutation (DEC-063).
    ///
    /// Two refusals, and the batch is the right home for both because neither is
    /// about one declaration's *effect* — the engine's arms own those — and
    /// because this runs before any arm has touched the snapshot, so a refused
    /// batch cannot have half-applied even into the working copy.
    ///
    /// - A **repeated subject**: two declarations about one subject in a batch
    ///   with no order is genuinely ambiguous, and picking last-wins would be
    ///   inventing an order the contract says does not exist.
    /// - An **inert key** ([`Declaration::inert_key`], `ISS-318`): a key that
    ///   means nothing at its subject's kind, which serde admits because
    ///   `Declaration` is one flat struct and which nothing else would catch.
    ///
    /// The inert-key check runs first. A declaration repeated at the wrong
    /// subject kind is two mistakes with one cause, and the key-level refusal
    /// names the cause.
    ///
    /// Returns the declarations keyed by subject, so iteration order is
    /// determined by the subjects present and not by how they were submitted.
    pub(crate) fn validate(self) -> Result<BTreeMap<DesignId, Declaration>, Refusal> {
        let mut candidate = BTreeMap::new();
        for declaration in self.declarations {
            if let Some(inert) = declaration.inert_key() {
                return Err(inert);
            }
            let subject = declaration.subject().clone();
            if candidate.insert(subject.clone(), declaration).is_some() {
                return Err(Refusal::DuplicateSubject { id: subject });
            }
        }
        Ok(candidate)
    }
}
