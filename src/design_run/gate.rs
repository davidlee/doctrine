// SPDX-License-Identifier: GPL-3.0-only
//! Forward boundaries, cumulative clearance, and direct regression.
//!
//! [`Advance`] is the closed forward relation and the single owner of the edge
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
use super::attestation::{ActKind, ActorClass, AgentActKind, ReviewPolicy};
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

// ---------------------------------------------------------------------------
// The contract vocabulary (design sec-3). What a condition requires, what
// subject it binds to, and how it is discharged — stated in types the program
// can reach, rather than in prose no run can read.
//
// Declarations only at this point in the phase: the table that instantiates
// them, and the derivation that reads them, land with the generated vocabulary.
// The `cfg_attr(not(test), expect(dead_code))` gates below are that staging, and
// each one must be DELETED — not left — by the task that gives its subject a
// production reader, or the unfulfilled expectation fails the lint gate.
// ---------------------------------------------------------------------------

/// How far a guard is re-derived (design sec-3, *reach*).
///
/// A per-row commitment with **no global default**, because the right answer is
/// contextual: research invalidated by a later change should plausibly force a
/// restamp, while document review invalidated by every revision would regress the
/// run backwards after every dispositioned finding. Same mechanism, opposite
/// right answer — so a model reading *cumulative unless excepted* gets one of
/// those two cases wrong by construction.
///
/// DEC-067's cumulative re-derivation is the **mechanism** this selects, not the
/// default it departs from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "SL-244 PHASE-05 T9 writes the table this keys")
)]
pub(crate) enum Reach {
    /// Re-derived at this edge and every edge above it (DEC-067).
    Cumulative,
    /// Evaluated only on the edge that names it.
    EdgeLocal,
}

/// The run-owned subject set an attestation is current against (design sec-3).
///
/// Independent of [`Reach`], and conflating the two is a live error: reach says
/// *when* a guard is re-derived, coverage says *what can make it fail*. A row
/// that is `Cumulative` over `Artefact` is re-derived on every forward move and
/// can still only fail if its own artefact changes — coherent, and much weaker
/// than "cumulative" sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "SL-244 PHASE-05 T9 writes the table this keys")
)]
pub(crate) enum Coverage {
    /// The attested artefact's own recorded content, and nothing else. The
    /// degenerate case: it covers nothing but itself.
    Artefact,
    /// The act carries a covered map that must equal every section's current
    /// digest — whole-map equality, so a section joining *or leaving*
    /// invalidates.
    EverySection,
    /// The act carries a covered map that must equal every inquiry node's
    /// current canonical material.
    InquiryMap,
    /// Quantified over subjects instead of carried by the act: every current
    /// section must have its OWN live act of the required kind, one per resolved
    /// lane, against that section's current digest.
    ///
    /// **Not [`Coverage::EverySection`] with the acts spread out.** Two
    /// properties whole-map equality does not have: a departing section drops
    /// out of the quantification rather than failing it (the reviews that remain
    /// are still good), and the empty case is *refused* — an empty stored
    /// coverage compares equal to an empty current map, so equality would let a
    /// document with no sections lock. The non-empty guard is part of what this
    /// variant means, not a check beside it.
    PerSection,
}

/// Canonical state the run does not own, observed by the shell this invocation
/// (design sec-3, *observed facts*).
///
/// Every member owes a **typed projection and a deterministic encoding**: a
/// fingerprint over an unspecified projection is not a comparison, since
/// attestation-time and evaluation-time code can hash semantically identical
/// state differently and the condition then expires or survives for no reason
/// anybody can name. That obligation is part of the member, not of the call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "SL-244 PHASE-05 T7 projects this, T9 tables it")
)]
pub(crate) enum ObservedFact {
    /// The slice's canonical governance relationship edge set.
    GovernanceEdges,
}

/// What an attestation is current against — its own coverage, **and**
/// conjunctively every observed fact it named.
///
/// Conjunctive rather than alternative, and the plain reason is why: under
/// alternation a row whose observed fact had moved would still read current off
/// its own unchanged artefact, so observing an external fact would buy nothing.
/// An attestation is always over its own recorded content; observed facts are an
/// additional conjunct, never a substitute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "SL-244 PHASE-05 T9 writes the table this keys")
)]
pub(crate) struct Binding {
    /// What the attestation's own recorded content covers.
    pub(crate) coverage: Coverage,
    /// Canonical facts outside the run that must ALSO still match. Conjunctive
    /// with the coverage, and with each other.
    pub(crate) observed: &'static [ObservedFact],
}

/// Where a requirement's actor comes from — **not** who it is (design sec-3).
///
/// Naming the source rather than the actor is what keeps the table `&'static`
/// and total while `section-attestations-current`'s required lanes are declared
/// per run under DEC-073. A per-*section* actor would not fit any shape of this
/// slot; a per-*run* one is a single value an evaluation reads once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "SL-244 PHASE-05 T9 writes the table this keys")
)]
pub(crate) enum RequiredActor {
    /// Fixed by the rule. Seven of the eight requirements.
    Fixed(ActorClass),
    /// Read from the run's review policy (DEC-073). Exactly one requirement.
    RunPolicy,
}

impl RequiredActor {
    /// The lanes an act must satisfy — **all** of them, never any of them.
    ///
    /// Resolution is what fixes the conjunction's arity, so the `&'static` entry
    /// count is not the requirement count: [`RequiredActor::Fixed`] is the
    /// singleton case, while [`RequiredActor::RunPolicy`] stands for one required
    /// act or for two depending on the run. Said plainly because a refusal owes
    /// the missing **lane**, not merely the missing act.
    ///
    /// Rides [`ReviewPolicy::lanes`] — membership has one home, and a second lane
    /// table here would be the parallel implementation this slice refuses.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "SL-244 PHASE-05 T10 evaluates over this")
    )]
    pub(crate) const fn resolve(self, policy: ReviewPolicy) -> &'static [ActorClass] {
        match self {
            RequiredActor::Fixed(ActorClass::User) => &[ActorClass::User],
            RequiredActor::Fixed(ActorClass::Agent) => &[ActorClass::Agent],
            RequiredActor::Fixed(ActorClass::Adversarial) => &[ActorClass::Adversarial],
            RequiredActor::RunPolicy => policy.lanes(),
        }
    }
}

/// One act a condition requires, and everything about *that act* the gate needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "SL-244 PHASE-05 T9 writes the table this keys")
)]
pub(crate) struct ActRequirement {
    /// The act that must be present.
    pub(crate) act: ActKind,
    /// Who must have authored it. Required, not optional — DEC-126's
    /// discriminator *is* actor identity, so a rule omitting the actor could not
    /// express the conditions it classifies.
    pub(crate) actor: RequiredActor,
    /// The agent declaration this act must confirm, where the interaction has an
    /// order. `Some` on exactly one requirement today; `None` means this act
    /// confirms nothing **and its record must carry no confirmation**.
    ///
    /// Coverage makes conjuncts current together but says nothing about order —
    /// that the user's confirmation came after, and over, the agent's
    /// declaration. This slot is the rule half of that; the record carries the
    /// digest that answers whether it is still the one confirmed.
    pub(crate) confirms: Option<AgentActKind>,
    /// Whether this act disposes of a review, and therefore must carry DEC-125's
    /// two-armed value. True on `ReviewDisposed` alone.
    pub(crate) disposes_review: bool,
}

/// Derivation over recorded attestations of one or more acts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "SL-244 PHASE-05 T9 writes the table this keys")
)]
pub(crate) struct AttestationRule {
    /// Every act that must be present. A **conjunction** — two entries mean
    /// *both*, never *either*. DEC-121 is explicit that initial concerns are two
    /// acts and not one, and a rule carrying one act could neither say so nor
    /// let a refusal name which half is missing.
    pub(crate) acts: &'static [ActRequirement],
    /// What the attestations bind to, and therefore what invalidates them.
    pub(crate) binding: Binding,
}

/// Run-owned state a condition may be recomputed from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "SL-244 PHASE-05 T9 writes the table this keys")
)]
pub(crate) enum EngineSource {
    /// The inquiry map's dispositions.
    Dispositions,
    /// The authored watermark against current section digests.
    Materialisation,
}

/// How one condition is derived — **the discharging act, stated exactly once**
/// (design sec-3, DEC-123).
///
/// One coupled value rather than independent `kind` and `subject` fields:
/// independent fields made *`Derived` derived over an attestation* representable
/// and meaningless, and the coupling makes that contradiction unspellable rather
/// than merely discouraged.
///
/// There is deliberately **no `remedy: &'static str` beside it**. A refusal's
/// remedy text is *rendered from* this rule, so the two cannot disagree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "SL-244 PHASE-05 T9 writes the table this keys")
)]
pub(crate) enum DerivationRule {
    /// Recomputed from run-owned state; the variant names which state.
    Engine(EngineSource),
    /// Derived over recorded attestations of one or more acts.
    Attested(AttestationRule),
}

impl DerivationRule {
    /// This rule's kind — a **projection**, never a stored field.
    ///
    /// DEC-120's vocabulary survives as a discriminant of the rule, so a kind
    /// and a rule cannot be set to disagree. There is nothing to keep in step.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "SL-244 PHASE-05 T9 renders the contract")
    )]
    pub(crate) const fn kind(&self) -> ConditionKind {
        match *self {
            DerivationRule::Engine(_) => ConditionKind::Derived,
            DerivationRule::Attested(_) => ConditionKind::Attested,
        }
    }
}

/// DEC-120's vocabulary: **every condition is derived; what varies is who can
/// author the state it derives over.**
///
/// `Attested` names *input provenance*, not a second decision procedure — there
/// is one derivation, and this is a projection of its rule.
///
/// Two variants, not three. DEC-120 names `Claimed` as the **defect class**, and
/// it is not representable here: a tier with no legitimate members is not a tier,
/// and leaving it constructible invites a new member.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "SL-244 PHASE-05 T9 renders the contract")
)]
pub(crate) enum ConditionKind {
    /// Recomputed from run-owned state.
    Derived,
    /// Derived over acts an actor recorded.
    Attested,
}

/// What one condition requires, binds to, and is discharged by — the value the
/// closed [`Condition`] vocabulary is the key of (design sec-3, DEC-122/DEC-123).
///
/// `DEC-101` is untouched by this: the closed set is the **key** and this is the
/// **value**, a total function out of nine known members. Nothing is narrowed
/// into the vocabulary.
///
/// Structure rides this table; the narrative rides the prose asset it names. The
/// two carry different things on purpose — the table carries what the program
/// must reach, the prose carries what an agent must read — and the renderer
/// *injects* the kind and the discharging act from here into the rendered
/// contract, so the prose never restates them and cannot contradict them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "SL-244 PHASE-05 T9 emits the CONTRACTS table")
)]
pub(crate) struct Contract {
    /// How the condition is derived, and therefore how it is discharged.
    pub(crate) derivation: DerivationRule,
    /// How far the guard is re-derived.
    pub(crate) reach: Reach,
    /// Key of the narrative prose asset — the condition's own kebab token.
    pub(crate) prose: &'static str,
}

/// The design run's four guarded forward transitions (SL-244 sec-3).
///
/// The forward graph's single home. Every edge-keyed table below takes one of
/// these rather than a `(Stage, Stage)` pair, and the difference is what the
/// type buys: a pair has twenty-five values of which four are lawful, so a row
/// keyed `(Exploring, Locked)` compiles, joins every set, and is never
/// evaluated. Both directions close by construction — a table cannot name an
/// unlawful transition because there is no such value to name, and it cannot
/// leave a lawful one unguarded because the match would be non-exhaustive,
/// which is a build failure.
///
/// **Forward only, and the asymmetry is deliberate.** The backward relation
/// covers every later-to-earlier pair, not merely adjacent ones, and is barred
/// by a missing *reason* rather than by a condition ([`regress`], DEC-067).
/// Nothing is closed to enumerate on that side, so a counterpart type would
/// assert a symmetry that does not exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Advance {
    /// exploring → inquiring.
    ExploringInquiring,
    /// inquiring → drafting.
    InquiringDrafting,
    /// drafting → reviewing.
    DraftingReviewing,
    /// reviewing → locked.
    ReviewingLocked,
}

impl Advance {
    /// Every forward edge, in forward order.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "SL-244 PHASE-06/08 render the edge list")
    )]
    pub(crate) const ALL: [Advance; 4] = [
        Advance::ExploringInquiring,
        Advance::InquiringDrafting,
        Advance::DraftingReviewing,
        Advance::ReviewingLocked,
    ];

    /// The forward graph, written once.
    ///
    /// Pure, total, and `const`: the four adjacent forward moves of design §5.4,
    /// and every other combination — self-moves, skips, and every backward move
    /// — refused. A backward move is not illegal, it is a *different verb*
    /// ([`regress`]).
    pub(crate) const fn between(from: Stage, to: Stage) -> Option<Self> {
        match (from, to) {
            (Stage::Exploring, Stage::Inquiring) => Some(Advance::ExploringInquiring),
            (Stage::Inquiring, Stage::Drafting) => Some(Advance::InquiringDrafting),
            (Stage::Drafting, Stage::Reviewing) => Some(Advance::DraftingReviewing),
            (Stage::Reviewing, Stage::Locked) => Some(Advance::ReviewingLocked),
            _ => None,
        }
    }

    /// The single forward edge leaving `stage`, if it has one.
    ///
    /// [`Advance::between`] is total on non-terminal stages — each has exactly
    /// one outbound forward edge — so "which edge am I standing on the origin
    /// of" is a well-posed question with a unique answer. `Locked` is terminal
    /// and yields `None`, the same real answer
    /// [`super::prompt::Fragment::for_stage`] gives there.
    pub(crate) const fn from_stage(stage: Stage) -> Option<Self> {
        match stage {
            Stage::Exploring => Some(Advance::ExploringInquiring),
            Stage::Inquiring => Some(Advance::InquiringDrafting),
            Stage::Drafting => Some(Advance::DraftingReviewing),
            Stage::Reviewing => Some(Advance::ReviewingLocked),
            Stage::Locked => None,
        }
    }
}

/// The conditions a single forward boundary requires (design §5.4).
///
/// Total by construction: [`Advance`] has no value that is not an edge, so
/// there is no empty arm to fall through to.
pub(crate) const fn boundary_conditions(edge: Advance) -> &'static [Condition] {
    match edge {
        Advance::ExploringInquiring => &[
            Condition::GoverningContextRecorded,
            Condition::InitialConcernsRecorded,
        ],
        Advance::InquiringDrafting => &[
            Condition::BlockingInquiriesDispositioned,
            Condition::UserAcceptsSufficiency,
        ],
        Advance::DraftingReviewing => &[
            Condition::RequiredSectionsExist,
            Condition::MaterialisationCurrent,
        ],
        Advance::ReviewingLocked => &[
            Condition::SectionAttestationsCurrent,
            Condition::IntegratedReviewPresent,
            Condition::BlockingFindingsDisposed,
            Condition::UserAcceptanceAttested,
        ],
    }
}

/// The runbook a single forward boundary requires discharged.
///
/// A third column on the table [`boundary_conditions`] already is, and the same
/// shape: static, `const`, total over the edge. Guards belong on **edges**, and
/// this one adds no states — the cursor it implies is run data a guard consults,
/// not a node in the machine, exactly as [`ReviewStanding`] is (sketch §2.1).
///
/// Edge-keying and origin-state-keying are isomorphic here, because
/// [`Advance::from_stage`] is total on non-terminal stages: each has exactly one
/// outbound forward edge. So the `exploring` runbook is named for where you
/// stand while discharging it, and selected by the edge it guards.
///
/// Every forward edge carries one (SL-233 PHASE-08), and since SL-244 PHASE-01
/// the type says so: with the backward and non-adjacent pairs unrepresentable
/// there is no `None` arm left, so the return is a bare [`RunbookKey`]. The
/// absence a caller standing at `Locked` sees is [`Advance::from_stage`]'s
/// `None`, which is where that fact belongs.
pub(crate) const fn boundary_runbook(edge: Advance) -> RunbookKey {
    match edge {
        Advance::ExploringInquiring => RunbookKey::Exploring,
        Advance::InquiringDrafting => RunbookKey::Inquiring,
        Advance::DraftingReviewing => RunbookKey::Drafting,
        Advance::ReviewingLocked => RunbookKey::Reviewing,
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
        // Each window resolves to an edge before it reaches the boundary table;
        // no `(Stage, Stage)` pair is threaded into it (SL-244 PHASE-01 EX-3).
        if let Some(edge) = Advance::between(from, next) {
            out.extend_from_slice(boundary_conditions(edge));
        }
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
    // Legality is [`Advance::between`] and nothing else — the forward graph has
    // one home, and this is the only production caller that asks it a yes/no
    // question (SL-244 sec-3).
    if Advance::between(from, to).is_none() {
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
    //
    // Unconditional since SL-244 PHASE-01: [`boundary_runbook`] is total over
    // [`Advance`], and the legality check above proves this pair is one, so the
    // `is_some` test the pair form needed has no false arm left to guard.
    //
    // Fail closed on a missing answer, the rule [`ReviewStanding::holds`]
    // already follows: no standing where the table says a runbook guards this
    // edge leaves the gate shut, not open.
    if runbook.is_none_or(|held| !held.cleared()) {
        return Err(Refusal::RunbookNotDischarged {
            from,
            to,
            outstanding: runbook.map_or_else(Vec::new, |held| held.outstanding.clone()),
            regressed: runbook.map_or_else(Vec::new, |held| held.regressed.clone()),
        });
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
