// SPDX-License-Identifier: GPL-3.0-only
//! Review and recovery — two more orthogonal state models.
//!
//! They are separate types because R3 says so: stage, inquiry lifecycle,
//! cursor/posture, review, delegation, and recovery each move on their own, and
//! collapsing any pair of them produces the hierarchical machine DEC-065 rejects.
//! Delegation, the third of that group, moved to [`super::delegation`] when
//! PHASE-10 gave it state of its own — it is the one model that stores a caller's
//! [`Declaration`]s, so it carries an edge to [`super::submission`] that review
//! and recovery do not.
//!
//! [`Declaration`]: super::submission::Declaration

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::gate::ObservedFact;
use super::ids::{DesignId, Fingerprint};
use super::inquiry::NodeMaterial;

/// Who reviewed a section (DEC-073, DEC-074).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Reviewer {
    /// The v1 default.
    Human,
    /// Opt-in per section; integrated adversarial review stays mandatory.
    Adversarial,
}

/// Every act a condition may require (design sec-4, *the acts and where each
/// lives*). **Closed at eight**, in the order that table lists them.
///
/// The gate asks one question of a recorded act — *is there one of this kind, by
/// this actor, still current against this binding?* — so the kind is the
/// vocabulary the contract table is written in, and the record shapes below
/// (three of them, for these eight) are storage rather than identity.
///
/// [`AgentActKind`] narrows this to the two an agent may author. The narrowing
/// runs one way only: every agent act is an act, and no widening exists in the
/// other direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ActKind {
    /// The user confirms the governing context that was found (DEC-121).
    GovernanceConfirmed,
    /// The user reviews and steers the seeded inquiry graph (DEC-121).
    GraphReviewed,
    /// The agent declares which questions it considers blocking (DEC-121).
    BlockingSetDeclared,
    /// The user accepts that interrogation is sufficient.
    SufficiencyAccepted,
    /// The agent declares the draft ready to be reviewed.
    DraftingReady,
    /// A reviewer attests one section, in one lane.
    SectionReviewed,
    /// The user disposes of an adversarial pass — conducted or waived (DEC-125).
    ReviewDisposed,
    /// The user accepts the design, at run level.
    DesignAccepted,
}

/// The two acts an **agent** may author, as a vocabulary of its own.
///
/// A separate fieldless type rather than a predicate on [`ActKind`], because it
/// is what [`super::gate::ActRequirement`]'s `confirms` slot ranges over: a
/// requirement may name an agent declaration to be confirmed, and naming a *user*
/// act there is a contradiction that should not be spellable. The `From` below is
/// the one-way widening.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AgentActKind {
    /// The agent declares which questions it considers blocking.
    BlockingSetDeclared,
    /// The agent declares the draft ready to be reviewed.
    DraftingReady,
}

impl ActKind {
    /// The kebab token this act is spelled with everywhere — the stored value,
    /// the rendered remedy, the refusal text (STD-001). It agrees with the serde
    /// rename by construction of the test that compares them, which is
    /// [`ReviewPolicy::as_str`]'s arrangement.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            ActKind::GovernanceConfirmed => "governance-confirmed",
            ActKind::GraphReviewed => "graph-reviewed",
            ActKind::BlockingSetDeclared => "blocking-set-declared",
            ActKind::SufficiencyAccepted => "sufficiency-accepted",
            ActKind::DraftingReady => "drafting-ready",
            ActKind::SectionReviewed => "section-reviewed",
            ActKind::ReviewDisposed => "review-disposed",
            ActKind::DesignAccepted => "design-accepted",
        }
    }
}

impl From<AgentActKind> for ActKind {
    fn from(agent: AgentActKind) -> Self {
        match agent {
            AgentActKind::BlockingSetDeclared => ActKind::BlockingSetDeclared,
            AgentActKind::DraftingReady => ActKind::DraftingReady,
        }
    }
}

/// Who an act is attributable to, as the gate classifies *recorded acts*
/// (design sec-3).
///
/// Deliberately not a merge of the two incumbents. [`Reviewer`] records who
/// reviewed a section and [`super::traversal::Authority`] records who directed a
/// traversal; neither is this axis, and collapsing all three is the hierarchical
/// state machine DEC-065 rejects. What the gate needs is one *direction* — a
/// reviewer expressed as an actor class — and that is the `From` below.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ActorClass {
    /// The user, acting in their own name.
    User,
    /// The agent driving the run.
    Agent,
    /// An adversarial reviewer, acting as a check on the agent.
    Adversarial,
}

impl From<Reviewer> for ActorClass {
    fn from(reviewer: Reviewer) -> Self {
        match reviewer {
            Reviewer::Human => ActorClass::User,
            Reviewer::Adversarial => ActorClass::Adversarial,
        }
    }
}

/// The reviewer lanes a run requires, and the order it intends them in
/// (DEC-073). Membership is what the gate checks; order is DECLARED, not
/// enforced.
///
/// Four values rather than a `Vec<Reviewer>`, because [`Reviewer`] is closed at
/// two and the lawful policies are therefore exactly these. A vector described as
/// *ordered, non-empty, duplicate-free* has none of those three properties — it
/// admits `[]` and `[Human, Human]` — so the prose would claim a guarantee
/// nothing enforced, which is ISS-310's own defect shape reappearing inside its
/// fix. The enum makes all three structural.
///
/// [`ReviewPolicy::HumanOnly`] is the default (DEC-074's posture) and the header
/// field is `#[serde(default)]`, so an existing snapshot parses and an existing
/// run behaves exactly as it did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ReviewPolicy {
    /// The v1 default: a human reviews each section.
    #[default]
    HumanOnly,
    /// An adversarial reviewer acts as a human proxy (DEC-074 grants this).
    AdversarialOnly,
    /// Both lanes, human first by intent.
    HumanThenAdversarial,
    /// Both lanes, adversarial first by intent.
    AdversarialThenHuman,
}

impl ReviewPolicy {
    /// The lanes a section must carry a live attestation in — the single home of
    /// membership, and what PHASE-05's `RequiredActor::RunPolicy` resolves
    /// through.
    ///
    /// The two ordered variants share an arm, which *is* the claim that order is
    /// declared and not enforced: they present the same requirement to the gate,
    /// and the distinction they draw is read by the renderer and the runbook.
    pub(crate) const fn lanes(self) -> &'static [ActorClass] {
        match self {
            ReviewPolicy::HumanOnly => &[ActorClass::User],
            ReviewPolicy::AdversarialOnly => &[ActorClass::Adversarial],
            ReviewPolicy::HumanThenAdversarial | ReviewPolicy::AdversarialThenHuman => {
                &[ActorClass::User, ActorClass::Adversarial]
            }
        }
    }

    /// The kebab token this policy is spelled with everywhere — the stored value,
    /// the change row's terms, the rendered label (STD-001). It agrees with the
    /// serde rename by construction of the test that compares them.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            ReviewPolicy::HumanOnly => "human-only",
            ReviewPolicy::AdversarialOnly => "adversarial-only",
            ReviewPolicy::HumanThenAdversarial => "human-then-adversarial",
            ReviewPolicy::AdversarialThenHuman => "adversarial-then-human",
        }
    }

    /// Every policy, in declaration order — the closed vocabulary, single-sourced
    /// so an exhaustive test cannot silently miss a member (STD-001).
    pub(crate) const ALL: [ReviewPolicy; 4] = [
        ReviewPolicy::HumanOnly,
        ReviewPolicy::AdversarialOnly,
        ReviewPolicy::HumanThenAdversarial,
        ReviewPolicy::AdversarialThenHuman,
    ];
}

/// A content-bound review attestation (DEC-073).
///
/// The fingerprint is not decoration: an attestation names the exact bytes it
/// attests to, so a later edit invalidates it through the same DEC-066 rule that
/// invalidates gate evidence rather than through a second mechanism.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Attestation {
    id: DesignId,
    subject: DesignId,
    fingerprint: Fingerprint,
    reviewer: Reviewer,
}

impl Attestation {
    /// Bind an attestation to the exact content reviewed.
    pub(crate) const fn bind(
        id: DesignId,
        subject: DesignId,
        fingerprint: Fingerprint,
        reviewer: Reviewer,
    ) -> Self {
        Attestation {
            id,
            subject,
            fingerprint,
            reviewer,
        }
    }

    /// This attestation's id.
    pub(crate) const fn id(&self) -> &DesignId {
        &self.id
    }

    /// The section attested to.
    pub(crate) const fn subject(&self) -> &DesignId {
        &self.subject
    }

    /// The content fingerprint attested to.
    pub(crate) const fn fingerprint(&self) -> &Fingerprint {
        &self.fingerprint
    }

    /// Who reviewed — read by [`DesignSnapshot::missing_lanes`], which is what
    /// closes ISS-310: the gate now derives review standing from the lane an
    /// attestation was given in and not from coverage alone.
    ///
    /// [`DesignSnapshot::missing_lanes`]: super::snapshot::DesignSnapshot::missing_lanes
    pub(crate) const fn reviewer(&self) -> Reviewer {
        self.reviewer
    }
}

/// Who holds authority over **accepted truth** (DEC-088).
///
/// One member, and the single membership is the point: acceptance is the user's
/// and nobody else's, so there is no variant an agent-authored payload could
/// select. It is an enum rather than a `const AUTHORITY: &str = "user"` because
/// the closed vocabulary is the claim being made — a second authority would have
/// to be added here, in the open, rather than appear as a string somewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AcceptanceAuthority {
    /// The user accepted this record as true.
    User,
}

/// A **user-acceptance** attestation (DEC-088) — a different thing from
/// [`Attestation`], and deliberately not a `Reviewer` variant.
///
/// [`Attestation`] answers *who reviewed this section* (DEC-073). This answers
/// *who accepted this record as true*, and only a user can. Folding the second
/// into the first as a third `Reviewer` would make authority over accepted truth
/// a sub-state of review — the hierarchical machine DEC-065 rejects — and would
/// let an `adversarial` reviewer clear an accepted-decision gate.
///
/// The `digest` is **derived by Doctrine and bound here**, never supplied: it
/// covers the checkpoint payload fingerprint, the inquiry disposition, and the
/// run revision current when the acceptance was given. So an acceptance cannot be
/// lifted onto different content, a later revision, or a different disposition —
/// it stops matching, exactly the way a [`Attestation`]'s section fingerprint
/// stops matching under DEC-066.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AcceptanceAttestation {
    authority: AcceptanceAuthority,
    basis: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    turn: Option<String>,
    digest: Fingerprint,
}

impl AcceptanceAttestation {
    /// Bind a user acceptance to the digest Doctrine derived for it.
    ///
    /// The four accessors below carry the same disclosure as
    /// [`Attestation::reviewer`]: nothing reads them yet.
    ///
    /// `authority` is set here rather than accepted here: the constructor is the
    /// only route to the type, so a value of it always carries user authority.
    pub(crate) fn bind(
        basis: impl Into<String>,
        turn: Option<String>,
        digest: Fingerprint,
    ) -> Self {
        AcceptanceAttestation {
            authority: AcceptanceAuthority::User,
            basis: basis.into(),
            turn,
            digest,
        }
    }

    /// Whose acceptance this is.
    #[expect(
        dead_code,
        reason = "SL-233: read surface with no reader (see the impl doc)"
    )]
    pub(crate) const fn authority(&self) -> AcceptanceAuthority {
        self.authority
    }

    /// The stated basis for accepting it.
    #[expect(
        dead_code,
        reason = "SL-233: read surface with no reader (see the impl doc)"
    )]
    pub(crate) fn basis(&self) -> &str {
        &self.basis
    }

    /// The harness turn the acceptance was given in, when the caller knew it.
    #[expect(
        dead_code,
        reason = "SL-233: read surface with no reader (see the impl doc)"
    )]
    pub(crate) fn turn(&self) -> Option<&str> {
        self.turn.as_deref()
    }

    /// The content binding Doctrine derived.
    #[expect(
        dead_code,
        reason = "SL-233: read surface with no reader (see the impl doc)"
    )]
    pub(crate) const fn digest(&self) -> &Fingerprint {
        &self.digest
    }
}

/// The content a whole-run attestation was given over (DEC-066, lifted from one
/// subject to the set).
///
/// A section attestation is live while a stored [`Fingerprint`] still equals its
/// subject's current one. An integrated review and a user acceptance are about the
/// *document*, which has no single stored fingerprint — so the same rule is
/// applied to the whole map: the coverage is current while every section it
/// covered still carries the fingerprint it was covered at, and no section has
/// been added or removed.
///
/// Deliberately **not** a composite digest. The pure layer never hashes, and the
/// shell cannot supply a post-batch digest as a derived input because
/// `DerivedInput` is built before `apply` runs the batch. Comparing the map is
/// pure, needs no new injection, and cannot bind to a stale value.
///
/// One type, three users — the integrated review, the lock acceptance, and the
/// inquiry-map coverage — so whole-run currency has a single owner rather than a
/// spelling in each.
///
/// Generic over *what* is covered rather than over the map: a section is covered
/// at its [`Fingerprint`], and an inquiry node at its
/// [`NodeMaterial`](super::inquiry::NodeMaterial), because nodes carry no
/// fingerprint and cannot be given a trustworthy one — they are mutated by pure
/// code after any shell digest would have been taken. Same comparison, different
/// covered type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ContentCoverage<T> {
    covered: BTreeMap<DesignId, T>,
}

impl<T: Eq> ContentCoverage<T> {
    /// Cover exactly what `current` holds now.
    pub(crate) const fn of(current: BTreeMap<DesignId, T>) -> Self {
        ContentCoverage { covered: current }
    }

    /// The subjects that have moved since this coverage was taken — present on
    /// one side only, or carrying something other than what they were covered
    /// at. Id-ordered, so a refusal naming them renders deterministically.
    ///
    /// One expression covers all three ways a map can move, because a joiner and
    /// a leaver are just the cases where one side's lookup is `None`.
    pub(crate) fn diff(&self, current: &BTreeMap<DesignId, T>) -> Vec<DesignId> {
        self.covered
            .keys()
            .chain(current.keys())
            .collect::<BTreeSet<&DesignId>>()
            .into_iter()
            .filter(|subject| self.covered.get(*subject) != current.get(*subject))
            .cloned()
            .collect()
    }

    /// Whether every covered subject still carries what it was covered at — and
    /// nothing has joined or left.
    ///
    /// Defined *through* [`ContentCoverage::diff`] rather than beside it: one
    /// comparison with one home, so the verdict and the explanation can never
    /// disagree.
    pub(crate) fn is_current(&self, current: &BTreeMap<DesignId, T>) -> bool {
        self.diff(current).is_empty()
    }
}

/// A canonical `RV` id, as the run records it (SL-244 `sec-4`).
///
/// Deliberately **not** a [`DesignId`]: run-local ids are one space and an `RV`
/// sits outside it, which is the defect DEC-125 names when it says the run
/// *"cannot hold or resolve an RV"*. Opaque here on purpose — nothing in the pure
/// layer parses the `RV-` prefix, so `design_run` names a review without
/// depending on the review module (ADR-001). A constructor that checked the
/// prefix would be that dependency spelled differently; validation belongs to the
/// wire boundary, exactly as [`Fingerprint`] is handed a digest the pure layer
/// never computes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct ReviewRef(String);

impl ReviewRef {
    /// Wrap a canonical ref the shell has already resolved.
    pub(crate) fn new(canonical: impl Into<String>) -> ReviewRef {
        ReviewRef(canonical.into())
    }
}

/// The review pass the run is currently on (DEC-125, SL-244 `sec-3`).
///
/// Minted on entry to `reviewing` and **replaced, never reopened** on a later
/// entry. Deliberately independent of the disposition that answers for it: the
/// pass exists before that row is answered and under both arms of the answer,
/// which is what lets the warnings derive on a waived run and what gives a
/// disposition something to bind to.
///
/// This is the shape `IntegratedReview` became — the same record with a
/// [`ReviewRef`] where a [`DesignId`] never belonged (SL-244 `EX-2`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ReviewPass {
    /// The `RV` minted for this pass.
    pub(crate) review: ReviewRef,
    /// The section digests the pass was opened over.
    pub(crate) covered: ContentCoverage<Fingerprint>,
}

impl ReviewPass {
    /// Open a pass over the content it reviews.
    pub(crate) const fn over(review: ReviewRef, covered: ContentCoverage<Fingerprint>) -> Self {
        ReviewPass { review, covered }
    }

    /// Whether it still covers current content.
    ///
    /// Coverage, not presence: a section that joined the run after the pass opened
    /// is content nobody looked at, and stales the pass exactly as a covered
    /// section moving does. Defined through [`ContentCoverage::is_current`] so the
    /// currency lamp and the diff that explains it cannot disagree.
    pub(crate) fn is_current(&self, current: &BTreeMap<DesignId, Fingerprint>) -> bool {
        self.covered.is_current(current)
    }
}

/// A covered map in the shape the act's [`Coverage`](super::gate::Coverage)
/// selector names.
///
/// A sum rather than two optional fields: an act covers sections or nodes, never
/// both and never neither-while-claiming-coverage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CoveredSet {
    /// Section digests — what `EverySection` compares.
    Sections(ContentCoverage<Fingerprint>),
    /// Inquiry-node material — what `InquiryMap` compares. Material rather than
    /// a digest of it, because nodes are mutated by pure code after any shell
    /// digest would have been taken.
    Nodes(ContentCoverage<NodeMaterial>),
}

/// DEC-125's two arms, given a home. Admissibility is DEC-138's, checked at
/// admission rather than here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ReviewDisposition {
    /// A pass was run and is being disposed of. The `review` field is retained
    /// beside [`DisposedPass::pass`] rather than folded into it because the two
    /// can only differ by being refused — a value that says the same thing twice
    /// is checkable.
    Conducted {
        /// The `RV` whose findings answer for this pass.
        review: ReviewRef,
    },
    /// The user declines a pass, on the record. Admissible over any review
    /// state; the reason must be non-blank, which is the only thing admission
    /// checks here.
    Waived {
        /// Why the pass was declined. Blank is refused at admission.
        reason: String,
    },
}

/// A disposition and the pass it was given over (design `sec-4`).
///
/// The pass reference sits **beside** the arm rather than inside `Conducted`,
/// because both arms bind to it and only one names an `RV` for its own reasons.
/// The row is satisfied only while `pass` equals the run's current
/// [`ReviewPass::review`], which is what makes a waiver dispose of **one** pass
/// rather than of review as such.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DisposedPass {
    /// The pass being disposed of.
    pub(crate) pass: ReviewRef,
    /// How it was disposed.
    pub(crate) disposition: ReviewDisposition,
}

/// A user act, in the form the gate reads it (DEC-121).
///
/// Named for the four checkpoint acts; `DesignAccepted` is the run-level fifth,
/// which is why [`LockAcceptance`] is subsumed rather than kept beside it — that
/// type is exactly this one with `act: DesignAccepted`, `covered: Sections(…)`
/// and every other slot empty.
///
/// **The fields are `pub(crate)` and no constructor guards them**, deliberately:
/// which slots a given act may fill is its *rule's* statement, and admission is
/// the single place that checks the record against it. A constructor enforcing a
/// subset of the same correspondence would be a second checker that could
/// disagree with the first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CheckpointAct {
    /// Engine-allocated, on the run's `cpa-` prefix.
    pub(crate) id: DesignId,
    /// Which act this is — and the key it is replaced by.
    pub(crate) act: ActKind,
    /// The acceptance itself. Embedded, not widened: its constructor still sets
    /// [`AcceptanceAuthority::User`], so DEC-088's guarantee is carried rather
    /// than re-argued. Its non-blank `basis` is also what makes DEC-121's empty
    /// case strict — a sweep that found nothing must still state what was
    /// searched, or the act is not admitted.
    pub(crate) acceptance: AcceptanceAttestation,
    /// What it was given over. `None` is `Coverage::Artefact`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) covered: Option<CoveredSet>,
    /// Each observed fact as it stood when the act was given.
    ///
    /// Deliberately the bare map rather than
    /// [`ObservedFacts`](super::gate::ObservedFacts): that type is transient by
    /// construction and says so, and this one is persisted.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) observed: BTreeMap<ObservedFact, Fingerprint>,
    /// The agent declaration this act confirms, where its rule names one — the
    /// declaration's own claim fingerprint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) confirms: Option<Fingerprint>,
    /// How the act disposes of a review, and **which** pass it disposed. `Some`
    /// on `ReviewDisposed` alone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) disposition: Option<DisposedPass>,
}

/// The acts an agent may declare, each carrying what it declares.
///
/// Tagged with its payload rather than paired with an optional field beside it,
/// so `DraftingReady` cannot carry a blocking set and `BlockingSetDeclared`
/// cannot omit one. Closed, and deliberately **not** [`ActKind`]: an
/// agent-authored `DesignAccepted` is the value that must not exist, and the fix
/// is unrepresentability rather than a check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AgentAct {
    /// The inquiries the agent considers blocking — DEC-121's artefact, and the
    /// thing the user's `GraphReviewed` confirms. Every id must be a node of the
    /// covered map; an id outside it is refused at admission.
    BlockingSetDeclared {
        /// The blocking node ids.
        blocking: BTreeSet<DesignId>,
    },
    /// The agent's judgement that drafting may begin. Its basis is the claim.
    DraftingReady,
}

impl AgentAct {
    /// Widen to the discriminant — the half a rule can name, and the only
    /// direction needed.
    pub(crate) const fn kind(&self) -> AgentActKind {
        match *self {
            AgentAct::BlockingSetDeclared { .. } => AgentActKind::BlockingSetDeclared,
            AgentAct::DraftingReady => AgentActKind::DraftingReady,
        }
    }
}

/// An agent's declaration about the state of its own work (DEC-121).
///
/// Deliberately **not** an [`AcceptanceAttestation`]: nothing here is accepted
/// truth, and that type's single-member authority enum is the claim it makes.
/// Widening it to admit an agent would delete DEC-088's guarantee, which is not
/// this slice's to spend.
///
/// It carries no `observed`, `confirms` or `disposition`, and that is the
/// correspondence rule rather than a shape decision: no requirement whose actor
/// is an agent names any of the three, so there is nothing for those slots to
/// hold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AgentDeclaration {
    /// Engine-allocated, on the run's `agd-` prefix.
    pub(crate) id: DesignId,
    /// The act with its payload. `act.kind()` is what rules and refusals name,
    /// and the key it is replaced by.
    pub(crate) act: AgentAct,
    /// The stated basis, as an acceptance carries one.
    pub(crate) basis: String,
    /// The harness turn it was declared in, when the caller knew it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) turn: Option<String>,
    /// What it was declared over. `None` is `Coverage::Artefact` — the
    /// declaration's own content is its binding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) covered: Option<CoveredSet>,
    /// The digest of this declaration's **claim** — `act` and `basis`, and
    /// nothing else. Shell-computed and arriving on `DerivedInput`; a confirming
    /// [`CheckpointAct`] names it.
    ///
    /// `id` is excluded because the engine allocates it and it is not content;
    /// `turn` because it is a harness detail rather than part of the claim; and
    /// `covered` because its currency is already the coverage mechanism's job.
    /// Those are two questions on two mechanisms — *is this the claim the user
    /// was shown*, and *has the material moved since* — and they are two on
    /// purpose.
    pub(crate) fingerprint: Fingerprint,
}

/// A user acceptance of the design as locked, and the content it accepted.
///
/// The two questions stay separate rather than being crushed into one digest:
/// [`AcceptanceAttestation`] answers *who accepted, on what basis, in which
/// submission* (DEC-088, and its `digest` is the submission payload digest that
/// decision arrived in); [`ContentCoverage`] answers *of what content*. Binding
/// currency to the payload digest instead would make every later apply invalidate
/// the acceptance, and binding it to nothing would let an acceptance be lifted
/// onto content the user never saw.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LockAcceptance {
    attestation: AcceptanceAttestation,
    covered: ContentCoverage<Fingerprint>,
}

impl LockAcceptance {
    /// Bind a user acceptance to the content it was given over.
    pub(crate) const fn over(
        attestation: AcceptanceAttestation,
        covered: ContentCoverage<Fingerprint>,
    ) -> Self {
        LockAcceptance {
            attestation,
            covered,
        }
    }

    /// The acceptance itself.
    #[expect(
        dead_code,
        reason = "SL-233: read surface with no reader (see [`Attestation::reviewer`])"
    )]
    pub(crate) const fn attestation(&self) -> &AcceptanceAttestation {
        &self.attestation
    }

    /// Whether it still covers current content.
    pub(crate) fn is_current(&self, current: &BTreeMap<DesignId, Fingerprint>) -> bool {
        self.covered.is_current(current)
    }
}

/// How far a journalled checkpoint got (DEC-086).
///
/// The states are the *effects that have landed*, in the six-step order, and the
/// gaps between them are deliberate: DEC-086's step 2 (claiming the id) has no
/// state of its own, because a crash there may leave an empty or partial
/// reservation and nothing else — there is nothing to resume and nothing to name.
/// From [`IntentState::Reserved`] onward the journal names the exact canonical
/// target, and recovery resumes the first incomplete effect **against that id**,
/// never against a fresh one.
///
/// Ordered, so "has this reached at least X?" is a comparison rather than a
/// hand-written table of which states imply which.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum IntentState {
    /// Step 1: the intent is journalled, keyed by the apply submission.
    Journalled,
    /// Step 3: the claimed canonical id is journalled. Every later step names it.
    Reserved,
    /// Step 4: the record's authored bytes are on disk.
    Materialised,
    /// Step 5: its status and legal relation edges are applied.
    Applied,
    /// Step 6: the snapshot carries the canonical disposition; nothing remains.
    Complete,
}

impl IntentState {
    /// The kebab token this state is spelled with everywhere (STD-001).
    #[expect(
        dead_code,
        reason = "SL-233: the serde rename carries the token on the wire; nothing renders it yet"
    )]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            IntentState::Journalled => "journalled",
            IntentState::Reserved => "reserved",
            IntentState::Materialised => "materialised",
            IntentState::Applied => "applied",
            IntentState::Complete => "complete",
        }
    }
}

/// The reserved wire token for the run-level review pass. No [`IdKind`] carries a
/// `review-` prefix, so no [`DesignId`] can spell it and the string coding below
/// stays unambiguous by construction (STD-001).
///
/// [`IdKind`]: super::ids::IdKind
const REVIEW_PASS_TOKEN: &str = "review-pass";

/// What a journalled intent is *about* (DEC-086, widened by DEC-125).
///
/// String-coded on the wire, so the `Checkpoint` arm stays a bare `DesignId` and
/// the pass rides a reserved token beside it — the shape a tagged table would
/// only make heavier. The `Checkpoint` spelling is **pinned**, and the
/// `RecoveryIntent` field keeps its pre-DEC-125 key as a serde alias: intents are
/// serialised into the design-run **snapshot** (`[[checkpoint.intent]]` in
/// `design.toml`), not only into the per-submission journal, and a snapshot
/// outlives many binaries — a run spans weeks. Dropping the alias made
/// `design show` fail outright on this repo's own live `SL-243`/`SL-244` runs
/// (SL-244 PHASE-04, reverted the same day).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub(crate) enum IntentSubject {
    /// A checkpoint disposing an inquiry node.
    Checkpoint(DesignId),
    /// The run's review pass, minted on entry to `reviewing` (DEC-125). There is
    /// at most one per run, so the subject needs no id of its own — the
    /// submission key already distinguishes one mint from the next.
    ReviewPass,
}

impl IntentSubject {
    /// The checkpoint this intent is about, if it is about one. A review pass
    /// names no inquiry node, which is what keeps the resolved-checkpoint map
    /// keyed by `DesignId` while the pass rides beside it (D2).
    pub(crate) const fn checkpoint(&self) -> Option<&DesignId> {
        match self {
            IntentSubject::Checkpoint(id) => Some(id),
            IntentSubject::ReviewPass => None,
        }
    }
}

impl From<IntentSubject> for String {
    fn from(subject: IntentSubject) -> String {
        match subject {
            IntentSubject::Checkpoint(id) => id.into(),
            IntentSubject::ReviewPass => REVIEW_PASS_TOKEN.to_owned(),
        }
    }
}

impl TryFrom<String> for IntentSubject {
    type Error = <DesignId as TryFrom<String>>::Error;

    fn try_from(raw: String) -> Result<IntentSubject, Self::Error> {
        if raw == REVIEW_PASS_TOKEN {
            return Ok(IntentSubject::ReviewPass);
        }
        DesignId::try_from(raw).map(IntentSubject::Checkpoint)
    }
}

/// A recoverable mint's intent (DEC-083, DEC-086).
///
/// Recorded *before* the authored effect, so recovery always has the exact
/// canonical target and resumes the first incomplete effect. Authored records are
/// never rolled back to repair a runtime failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RecoveryIntent {
    submission: String,
    /// What the intent is about. Read under its pre-DEC-125 key too, because the
    /// snapshot holding these rows outlives the binary that wrote them — see
    /// [`IntentSubject`].
    #[serde(alias = "checkpoint")]
    subject: IntentSubject,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reserved_record: Option<String>,
    /// How far this intent got. `#[serde(default)]` reads a pre-state journal as
    /// [`IntentState::Journalled`] — the conservative reading, which resumes from
    /// the first step rather than assuming an effect landed.
    #[serde(default = "IntentState::journalled_default")]
    state: IntentState,
    /// The user's acceptance, journalled with the intent so a resumed step 5
    /// applies the same status the original would have (DEC-088).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    acceptance: Option<AcceptanceAttestation>,
}

impl IntentState {
    /// The serde default: an intent with no recorded state has only been
    /// journalled.
    const fn journalled_default() -> IntentState {
        IntentState::Journalled
    }
}

impl RecoveryIntent {
    /// Journal the intent for `submission`, before any authored byte is written.
    pub(crate) fn journalled(submission: impl Into<String>, subject: IntentSubject) -> Self {
        RecoveryIntent {
            submission: submission.into(),
            subject,
            reserved_record: None,
            state: IntentState::Journalled,
            acceptance: None,
        }
    }

    /// Record the canonical id reserved for this checkpoint.
    #[must_use]
    pub(crate) fn reserving(mut self, record: impl Into<String>) -> Self {
        self.reserved_record = Some(record.into());
        self
    }

    /// Advance to `state`. Monotonic — an out-of-order or repeated write cannot
    /// walk a journalled effect backwards, which is what makes a resumed step
    /// idempotent rather than merely usually-safe.
    #[must_use]
    pub(crate) fn reaching(mut self, state: IntentState) -> Self {
        if state > self.state {
            self.state = state;
        }
        self
    }

    /// Attach the user acceptance this checkpoint was declared with.
    #[must_use]
    pub(crate) fn accepted(mut self, acceptance: Option<AcceptanceAttestation>) -> Self {
        self.acceptance = acceptance;
        self
    }

    /// The submission this intent is keyed by.
    pub(crate) fn submission(&self) -> &str {
        &self.submission
    }

    /// What this intent is about.
    pub(crate) const fn subject(&self) -> &IntentSubject {
        &self.subject
    }

    /// The reserved canonical id, once journalled.
    pub(crate) fn reserved_record(&self) -> Option<&str> {
        self.reserved_record.as_deref()
    }

    /// How far this intent got.
    pub(crate) const fn state(&self) -> IntentState {
        self.state
    }

    /// The user acceptance journalled with it, if any.
    pub(crate) const fn acceptance(&self) -> Option<&AcceptanceAttestation> {
        self.acceptance.as_ref()
    }
}
