// SPDX-License-Identifier: GPL-3.0-only
//! The design §9.1 pure-engine suite — eight tests, named by SL-233 PHASE-02
//! EX-8, operating on values and injected [`DerivedDesignFacts`] only.
//!
//! No clock, disk, git, or rng is reachable from here, because none is reachable
//! from the module under test.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test code — the repo's panic-avoidance denials target production paths"
)]

use std::collections::BTreeMap;

use super::Stage;
use super::attestation::{
    ActKind, ActorClass, AgentAct, AgentActKind, ContentCoverage, DisposedPass, IntentSubject,
    RecoveryIntent, ReviewDisposition, ReviewPolicy, ReviewRef, Reviewer,
};
use super::facts::DerivedDesignFacts;
use super::fixture::{
    attest, blocking_set_declared, checkpoint_act, drafting_ready, id, pass_over, run_holding,
    section,
};
use super::gate::{
    ActRequirement, Advance, AttestationRule, Binding, CONTRACTS, Condition, ConditionKind,
    Contract, Coverage, DerivationRule, EngineSource, ObservedFact, Reach, RequiredActor,
    ReviewStanding, advance, boundary_conditions, boundary_runbook, cumulative_conditions, regress,
};
use super::ids::{DesignId, Fingerprint, IdKind};
use super::inquiry::{
    Disposition, InquiryLifecycle, InquiryMap, InquiryNode, NodeMaterial, Provenance,
};
use super::refusal::Refusal;
use super::run::live_reviews;
use super::runbook::{RunbookKey, RunbookStanding};
use super::snapshot::{AgentDeclarationGroup, CheckpointActGroup};
use super::submission::{
    AgentActDeclaration, Batch, CheckpointActDeclaration, Declaration, Sparse,
};

/// Facts in which every *claimed* cumulative condition up to `stage` holds, each
/// cleared against a distinct subject at a known fingerprint.
///
/// The derived conditions are filtered out rather than recorded here: recording
/// them would build a fact set that looks like clearance the gate does not read
/// (a claim on a derived condition is refused at admission, and
/// [`Condition::is_derived`] is why). Their absence is what a
/// [`ReviewStanding`] answers for.
fn cleared_through(stage: Stage) -> DerivedDesignFacts {
    let mut facts = DerivedDesignFacts::default();
    for (index, condition) in cumulative_conditions(stage)
        .into_iter()
        .filter(|condition| !condition.is_derived())
        .enumerate()
    {
        let subject = id(&format!("sec-{index}"));
        let fingerprint = Fingerprint::new(format!("sha256:{index}"));
        facts = facts.observe(subject.clone(), fingerprint.clone()).record(
            condition,
            subject,
            fingerprint,
        );
    }
    facts
}

#[test]
fn stage_gate_table_admits_only_legal_forward_moves() {
    let mut admitted = Vec::new();
    for from in Stage::ALL {
        for to in Stage::ALL {
            if let Some(edge) = Advance::between(from, to) {
                admitted.push(edge);
            }
        }
    }

    // Exhaustive over all 25 ordered pairs; the four adjacent forward moves of
    // design §5.4 and nothing else — no self-move, no skip, no backward move.
    // Asserted against `Advance::ALL` rather than a hand-written pair list: the
    // type is now the forward graph's only home, so the expectation and the
    // table are the same statement (SL-244 PHASE-01 EX-1).
    assert_eq!(admitted, Advance::ALL.to_vec());

    // The verb rides the same table: a skip is refused even when every condition
    // in the run holds, so legality is not something clearance can buy.
    let facts = cleared_through(Stage::Locked);
    assert_eq!(
        advance(
            Stage::Exploring,
            Stage::Drafting,
            &facts,
            ReviewStanding::default(),
            None
        ),
        Err(Refusal::IllegalStageMove {
            from: Stage::Exploring,
            to: Stage::Drafting,
        })
    );
    assert_eq!(
        // SL-233 PHASE-16: this edge now also carries a runbook, so clearance
        // takes a discharged standing beside the conditions. A default standing
        // holds nothing outstanding — the "ritual done" case — which keeps this
        // assertion about the CONDITION table, which is what it is here to test.
        advance(
            Stage::Exploring,
            Stage::Inquiring,
            &facts,
            ReviewStanding::default(),
            Some(&RunbookStanding::default())
        ),
        Ok(Stage::Inquiring)
    );
}

/// The closed type's negative half (SL-244 PHASE-01 VT-1): every pair that is
/// *not* one of the four adjacent forward moves resolves to `None`.
///
/// Separate from the admission test because the two fail differently. That one
/// catches a missing edge; this one catches an extra one, and enumerates the
/// three ways an extra could arrive — a self-move, a skip, a backward move —
/// so a regression names which class it let through.
#[test]
fn advance_between_refuses_every_unlawful_pair() {
    for stage in Stage::ALL {
        assert_eq!(Advance::between(stage, stage), None, "self-move {stage:?}");
    }

    for (from, to) in [
        (Stage::Exploring, Stage::Drafting),
        (Stage::Exploring, Stage::Reviewing),
        (Stage::Exploring, Stage::Locked),
        (Stage::Inquiring, Stage::Reviewing),
        (Stage::Inquiring, Stage::Locked),
        (Stage::Drafting, Stage::Locked),
    ] {
        assert_eq!(Advance::between(from, to), None, "skip {from:?} → {to:?}");
    }

    // Every backward pair, derived rather than listed: a backward move is not an
    // illegal transition, it is a different verb (`regress`), and `Advance` is
    // deliberately the forward relation only.
    for from in Stage::ALL {
        for to in Stage::ALL {
            if to < from {
                assert_eq!(
                    Advance::between(from, to),
                    None,
                    "backward {from:?} → {to:?}"
                );
            }
        }
    }
}

/// `from_stage` answers *which edge am I standing on the origin of* (SL-244
/// PHASE-01 VT-2), and agrees with the two functions that already answer a
/// stage-keyed question about the same edge.
#[test]
fn advance_from_stage_is_none_at_locked() {
    assert_eq!(
        Advance::from_stage(Stage::Exploring),
        Some(Advance::ExploringInquiring)
    );
    assert_eq!(
        Advance::from_stage(Stage::Inquiring),
        Some(Advance::InquiringDrafting)
    );
    assert_eq!(
        Advance::from_stage(Stage::Drafting),
        Some(Advance::DraftingReviewing)
    );
    assert_eq!(
        Advance::from_stage(Stage::Reviewing),
        Some(Advance::ReviewingLocked)
    );

    // `Locked` is terminal, so there is no outbound forward edge to name — the
    // same real answer `Fragment::for_stage` already gives there, asserted
    // beside it so that `None` reads as the machine's shape and not as a gap.
    assert_eq!(Advance::from_stage(Stage::Locked), None);
    assert_eq!(super::prompt::Fragment::for_stage(Stage::Locked), None);

    // The origin-keyed and edge-keyed selectors are the same question: each
    // non-terminal stage's runbook is the one on the edge `from_stage` names.
    for stage in Stage::ALL {
        assert_eq!(
            Advance::from_stage(stage).map(boundary_runbook),
            match stage {
                Stage::Exploring => Some(RunbookKey::Exploring),
                Stage::Inquiring => Some(RunbookKey::Inquiring),
                Stage::Drafting => Some(RunbookKey::Drafting),
                Stage::Reviewing => Some(RunbookKey::Reviewing),
                Stage::Locked => None,
            }
        );
    }
}

#[test]
fn direct_regression_requires_a_recorded_reason() {
    assert_eq!(
        regress(Stage::Drafting, Stage::Exploring, ""),
        Err(Refusal::RegressionReasonMissing {
            from: Stage::Drafting,
            to: Stage::Exploring,
        })
    );
    // Whitespace is not a reason.
    assert_eq!(
        regress(Stage::Drafting, Stage::Exploring, "   \n\t"),
        Err(Refusal::RegressionReasonMissing {
            from: Stage::Drafting,
            to: Stage::Exploring,
        })
    );
    // A forward move is not a regression, and must not be laundered into one by
    // supplying a reason.
    assert_eq!(
        regress(Stage::Exploring, Stage::Drafting, "reason"),
        Err(Refusal::NotARegression {
            from: Stage::Exploring,
            to: Stage::Drafting,
        })
    );

    let recorded =
        regress(Stage::Drafting, Stage::Exploring, "the framing was wrong").expect("legal");
    assert_eq!(recorded.from(), Stage::Drafting);
    assert_eq!(recorded.to(), Stage::Exploring);
    assert_eq!(recorded.reason(), "the framing was wrong");

    // DEC-067's other half: returning forward inherits no clearance. Facts that
    // satisfy only the drafting boundary do not re-open drafting, because the
    // *cumulative* set is re-derived against current content.
    let mut partial = DerivedDesignFacts::default();
    for condition in [
        Condition::BlockingInquiriesDispositioned,
        Condition::UserAcceptsSufficiency,
    ] {
        let subject = id("sec-late");
        let fingerprint = Fingerprint::new("sha256:late");
        partial = partial
            .observe(subject.clone(), fingerprint.clone())
            .record(condition, subject, fingerprint);
    }
    // The runbook standing is CLEARED here on purpose. SL-233 PHASE-08 gave this
    // edge a runbook, and the gate fails closed on a missing standing *before*
    // it derives conditions — so passing `None` would stop the advance one check
    // earlier and this assertion would no longer be about cumulative conditions
    // at all. Clearing it keeps the subject the same assertion always had.
    let discharged = RunbookStanding::default();
    assert!(discharged.cleared(), "no outstanding required steps");
    assert_eq!(
        advance(
            Stage::Inquiring,
            Stage::Drafting,
            &partial,
            ReviewStanding::default(),
            Some(&discharged)
        ),
        Err(Refusal::GateNotCleared {
            from: Stage::Inquiring,
            to: Stage::Drafting,
            missing: vec![
                Condition::GoverningContextRecorded,
                Condition::InitialConcernsRecorded,
            ],
        })
    );
}

#[test]
fn changed_fingerprint_invalidates_only_affected_evidence() {
    let moved = id("sec-moved");
    let stable = id("sec-stable");
    let before = Fingerprint::new("sha256:aaa");
    let after = Fingerprint::new("sha256:bbb");
    let untouched = Fingerprint::new("sha256:ccc");

    let facts = DerivedDesignFacts::default()
        .observe(moved.clone(), before.clone())
        .observe(stable.clone(), untouched.clone())
        .record(Condition::DraftingReadinessAttested, moved.clone(), before)
        .record(
            Condition::MaterialisationCurrent,
            stable.clone(),
            untouched.clone(),
        );

    assert_eq!(facts.live_evidence().count(), 2);
    assert!(facts.satisfies(Condition::DraftingReadinessAttested));
    assert!(facts.satisfies(Condition::MaterialisationCurrent));

    // One subject's content moves. Only its evidence dies.
    let after_edit = facts.observe(moved.clone(), after);
    let live: Vec<&DesignId> = after_edit
        .live_evidence()
        .map(super::facts::Evidence::subject)
        .collect();
    assert_eq!(live, vec![&stable]);
    assert!(!after_edit.satisfies(Condition::DraftingReadinessAttested));
    assert!(after_edit.satisfies(Condition::MaterialisationCurrent));

    // A subject the shell can no longer observe is treated as changed, not as
    // unchanged — absence is not proof the bytes still match.
    let unobservable =
        DerivedDesignFacts::default().record(Condition::MaterialisationCurrent, stable, untouched);
    assert_eq!(unobservable.live_evidence().count(), 0);
}

#[test]
fn parent_and_needs_cycles_are_refused() {
    let (a, b) = (id("inq-a"), id("inq-b"));

    // Parent relation: a → b → a.
    let mut map = InquiryMap::default();
    map.insert(InquiryNode::open(a.clone(), "a?", Provenance::UserDirected))
        .expect("root");
    map.insert(
        InquiryNode::open(b.clone(), "b?", Provenance::AgentProposed).with_parent(a.clone()),
    )
    .expect("child");
    assert_eq!(
        map.insert(
            InquiryNode::open(a.clone(), "a?", Provenance::UserDirected).with_parent(b.clone())
        ),
        // The refusal names the edge that *closes* the cycle, which is the one
        // reached last on the walk, not the one just submitted.
        Err(Refusal::CyclicEdge {
            from: b.clone(),
            to: a.clone(),
        })
    );
    // The refused insert left the map as it was.
    assert_eq!(map.get(&a).and_then(InquiryNode::parent), None);

    // `needs` relation: the same cycle through the other edge kind.
    let mut needs_map = InquiryMap::default();
    needs_map
        .insert(InquiryNode::open(a.clone(), "a?", Provenance::UserDirected))
        .expect("root");
    needs_map
        .insert(InquiryNode::open(b.clone(), "b?", Provenance::AgentProposed).needing(a.clone()))
        .expect("dependant");
    assert_eq!(
        needs_map.insert(
            InquiryNode::open(a.clone(), "a?", Provenance::UserDirected).needing(b.clone())
        ),
        Err(Refusal::CyclicEdge { from: b, to: a })
    );

    // A diamond is not a cycle. `needs` makes them routine, and a checker that
    // reports every re-reached node would refuse this.
    let (d, e, f, g) = (id("inq-d"), id("inq-e"), id("inq-f"), id("inq-g"));
    let mut diamond = InquiryMap::default();
    diamond
        .insert(InquiryNode::open(g.clone(), "g?", Provenance::UserDirected))
        .expect("sink");
    diamond
        .insert(InquiryNode::open(e.clone(), "e?", Provenance::UserDirected).needing(g.clone()))
        .expect("left");
    diamond
        .insert(InquiryNode::open(f.clone(), "f?", Provenance::UserDirected).needing(g))
        .expect("right");
    assert_eq!(
        diamond.insert(
            InquiryNode::open(d.clone(), "d?", Provenance::UserDirected)
                .needing(e)
                .needing(f)
        ),
        Ok(())
    );
    assert_eq!(diamond.len(), 4);
}

#[test]
fn blocked_is_derived_not_stored() {
    let (waiting, blocker) = (id("inq-waiting"), id("inq-blocker"));
    let mut map = InquiryMap::default();
    map.insert(InquiryNode::open(
        blocker.clone(),
        "settle me first",
        Provenance::UserDirected,
    ))
    .expect("blocker");
    map.insert(
        InquiryNode::open(waiting.clone(), "depends", Provenance::AgentProposed)
            .needing(blocker.clone()),
    )
    .expect("dependant");

    assert!(map.is_blocked(&waiting));
    assert!(!map.is_blocked(&blocker));
    let before = map.get(&waiting).expect("present").clone();

    // Settling the blocker unblocks the dependant without touching it. The
    // dependant's value is byte-identical across the change, which is what
    // proves `blocked` is not a field somebody has to remember to update.
    let settled = map
        .get(&blocker)
        .expect("present")
        .clone()
        .resolve(Disposition::Created {
            record: "DEC-999".to_owned(),
        });
    map.insert(settled).expect("resolve");

    assert!(!map.is_blocked(&waiting));
    assert_eq!(map.get(&waiting), Some(&before));
    assert_eq!(map.blocked().count(), 0);
}

#[test]
fn resolved_node_without_disposition_is_refused() {
    let node_id = id("inq-1");
    let node = InquiryNode::open(node_id.clone(), "why?", Provenance::UserDirected);

    // The lifecycle-only route cannot reach `resolved`.
    assert_eq!(
        node.clone().transition(InquiryLifecycle::Resolved),
        Err(Refusal::DispositionMissing { id: node_id })
    );

    // The other non-resolved lifecycles are reachable and carry no disposition.
    for lifecycle in [InquiryLifecycle::Deferred, InquiryLifecycle::Pruned] {
        let moved = node.clone().transition(lifecycle).expect("legal");
        assert_eq!(moved.lifecycle(), lifecycle);
        assert_eq!(moved.disposition(), None);
    }

    // Resolution goes through the disposition-carrying route, and keeps it.
    let disposition = Disposition::RetainedUnresolved {
        note: "parked pending SPEC-024".to_owned(),
    };
    let resolved = node.resolve(disposition.clone());
    assert_eq!(resolved.lifecycle(), InquiryLifecycle::Resolved);
    assert_eq!(resolved.disposition(), Some(&disposition));

    // Moving back off `resolved` drops the disposition rather than leaving a
    // stale one attached to a node that is no longer resolved.
    let reopened = resolved.transition(InquiryLifecycle::Open).expect("legal");
    assert_eq!(reopened.disposition(), None);
}

#[test]
fn sparse_omission_null_and_empty_collection_differ() {
    let prior = || Some("kept".to_owned());

    // Three spellings, three outcomes — the scalar case.
    assert_eq!(Sparse::Omitted.apply(prior()), Some("kept".to_owned()));
    assert_eq!(Sparse::<String>::Null.apply(prior()), None);
    assert_eq!(
        Sparse::Value("replaced".to_owned()).apply(prior()),
        Some("replaced".to_owned())
    );

    // The collection case: an empty `Value` clears, omission does not. These are
    // the two an `Option<Vec<_>>` model collapses.
    let existing = || vec![id("inq-1"), id("inq-2")];
    assert_eq!(Sparse::Omitted.apply_collection(existing()), existing());
    assert_eq!(
        Sparse::<Vec<DesignId>>::Value(Vec::new()).apply_collection(existing()),
        Vec::new()
    );
    assert_eq!(
        Sparse::<Vec<DesignId>>::Null.apply_collection(existing()),
        Vec::new()
    );
    assert_ne!(
        Sparse::Omitted.apply_collection(existing()),
        Sparse::<Vec<DesignId>>::Value(Vec::new()).apply_collection(existing())
    );

    // Omission is the default, which is what lets `#[serde(default)]` mean
    // "absent key" rather than "null value".
    assert!(Sparse::<String>::default().is_omitted());
    assert!(!Sparse::<String>::Null.is_omitted());
}

#[test]
fn unordered_batch_refuses_duplicate_subjects() {
    let subject = id("inq-1");
    let duplicated = Batch::of(vec![
        Declaration::about(subject.clone()).question(Sparse::Value("first".to_owned())),
        Declaration::about(subject.clone()).question(Sparse::Value("second".to_owned())),
    ]);
    assert_eq!(
        duplicated.validate(),
        Err(Refusal::DuplicateSubject { id: subject })
    );

    // The batch is unordered: the same declarations submitted in either order
    // validate to the same candidate, in the same sequence.
    let (one, two, three) = (id("inq-1"), id("inq-2"), id("inq-3"));
    let declarations = |order: [&DesignId; 3]| {
        Batch::of(
            order
                .into_iter()
                .map(|subject| Declaration::about(subject.clone()))
                .collect(),
        )
    };
    let forward = declarations([&one, &two, &three])
        .validate()
        .expect("valid");
    let reversed = declarations([&three, &two, &one])
        .validate()
        .expect("valid");
    assert_eq!(forward, reversed);
    assert_eq!(
        forward.keys().collect::<Vec<&DesignId>>(),
        vec![&one, &two, &three]
    );
}

/// `diff` is the payoff for comparing material rather than a digest: a refusal
/// can name the subjects that moved. All three ways a map can move — a subject
/// leaving, one joining, one changing value — are the same comparison, so each
/// must appear exactly once and in id order.
#[test]
fn content_coverage_diff_names_only_what_moved() {
    let at = |raw: &str, mark: &str| (id(raw), Fingerprint::new(format!("sha256:{mark}")));
    let covered: BTreeMap<DesignId, Fingerprint> =
        [at("sec-1", "1"), at("sec-2", "2"), at("sec-3", "3")]
            .into_iter()
            .collect();
    let coverage = ContentCoverage::of(covered.clone());

    assert!(coverage.diff(&covered).is_empty());
    assert!(coverage.is_current(&covered));

    let mut moved = covered.clone();
    moved.remove(&id("sec-1"));
    let (joiner, joined_at) = at("sec-4", "4");
    moved.insert(joiner, joined_at);
    moved.insert(id("sec-2"), Fingerprint::new("sha256:edited"));

    assert_eq!(
        coverage.diff(&moved),
        vec![id("sec-1"), id("sec-2"), id("sec-4")],
        "a leaver, a changed value and a joiner, each once, in id order"
    );
    assert!(!coverage.is_current(&moved));
}

/// Every node in `nodes`, inserted in order, or a test failure — the fixtures
/// here are all legal maps, so a refusal means the fixture is wrong.
fn map_of(nodes: Vec<InquiryNode>) -> InquiryMap {
    let mut map = InquiryMap::default();
    for node in nodes {
        map.insert(node)
            .expect("fixture nodes must form a legal map");
    }
    map
}

/// What the user reviewed under DEC-121 is the set of questions and how they
/// relate. A question later being answered is *progress through* that graph, not
/// a change to it — so lifecycle and disposition are outside the material, and
/// re-wording, re-parenting, arriving and departing are all inside it.
///
/// The contrast is one test because the claim is the contrast: a material that
/// moved on nothing would pass the second half alone, and one that moved on
/// everything would pass the first half alone.
#[test]
fn node_material_ignores_progress_and_observes_shape() {
    let root = || {
        InquiryNode::open(
            id("inq-1"),
            "does the gate need a contract?",
            Provenance::UserDirected,
        )
        .sequenced(0)
    };
    let child = || {
        InquiryNode::open(
            id("inq-2"),
            "what does a refusal owe its reader?",
            Provenance::AgentProposed,
        )
        .sequenced(1)
        .with_parent(id("inq-1"))
    };
    let sibling = || {
        InquiryNode::open(
            id("inq-3"),
            "where does the material live?",
            Provenance::AgentProposed,
        )
        .sequenced(2)
    };

    let coverage: ContentCoverage<NodeMaterial> =
        ContentCoverage::of(map_of(vec![root(), child(), sibling()]).materials());

    let progressed = map_of(vec![
        root()
            .transition(InquiryLifecycle::Deferred)
            .expect("deferred is not resolved"),
        child().resolve(Disposition::Created {
            record: "DEC-140".to_owned(),
        }),
        sibling(),
    ])
    .materials();
    assert!(
        coverage.diff(&progressed).is_empty(),
        "deferring and disposing are progress through the graph, not a change to it"
    );
    assert!(coverage.is_current(&progressed));

    let reworded = map_of(vec![
        root(),
        InquiryNode::open(
            id("inq-2"),
            "what does a refusal owe its reader, exactly?",
            Provenance::AgentProposed,
        )
        .sequenced(1)
        .with_parent(id("inq-1")),
        sibling(),
    ])
    .materials();
    assert_eq!(coverage.diff(&reworded), vec![id("inq-2")], "re-worded");

    let reparented = map_of(vec![
        root(),
        sibling(),
        InquiryNode::open(
            id("inq-2"),
            "what does a refusal owe its reader?",
            Provenance::AgentProposed,
        )
        .sequenced(1)
        .with_parent(id("inq-3")),
    ])
    .materials();
    assert_eq!(coverage.diff(&reparented), vec![id("inq-2")], "re-parented");

    let joined = map_of(vec![
        root(),
        child(),
        sibling(),
        InquiryNode::open(id("inq-4"), "and who reads it?", Provenance::UserDirected).sequenced(3),
    ])
    .materials();
    assert_eq!(coverage.diff(&joined), vec![id("inq-4")], "a node arrived");

    let departed = map_of(vec![root(), child()]).materials();
    assert_eq!(coverage.diff(&departed), vec![id("inq-3")], "a node left");
}

/// The policy's membership is what the gate reads, and the two ordered variants
/// present the *same* membership — the difference between them is order, which
/// DEC-073 declares and nothing enforces.
///
/// Asserted rather than left to the reader because a gate seen discarding a
/// distinction its own type draws reads as a bug. It is the design's intent.
#[test]
fn ordered_policies_present_identical_membership() {
    assert_eq!(ReviewPolicy::HumanOnly.lanes(), [ActorClass::User]);
    assert_eq!(
        ReviewPolicy::AdversarialOnly.lanes(),
        [ActorClass::Adversarial]
    );
    assert_eq!(
        ReviewPolicy::HumanThenAdversarial.lanes(),
        ReviewPolicy::AdversarialThenHuman.lanes(),
        "order is declared, not enforced: the lanes required are the same pair"
    );
    assert_eq!(
        ReviewPolicy::HumanThenAdversarial.lanes(),
        [ActorClass::User, ActorClass::Adversarial]
    );

    // The default is DEC-074's posture, and it is what an existing run reads.
    assert_eq!(ReviewPolicy::default(), ReviewPolicy::HumanOnly);

    // One spelling, not two: the token a row renders is the token the snapshot
    // stores, so a rename cannot drift them apart silently (STD-001).
    for policy in ReviewPolicy::ALL {
        let stored = serde_json::to_string(&policy).expect("a policy serialises");
        assert_eq!(stored, format!("\"{}\"", policy.as_str()));
    }

    // A mapping, not a merge: `Reviewer` stays its own vocabulary and gains one
    // direction into the actor axis (design sec-3).
    assert_eq!(ActorClass::from(Reviewer::Human), ActorClass::User);
    assert_eq!(
        ActorClass::from(Reviewer::Adversarial),
        ActorClass::Adversarial
    );
}

/// ISS-310, at the surface it was reported from: the required lane is the
/// **run's**, not a constant. The same attestation is insufficient under one
/// policy and sufficient under another, and the run says which lane is missing
/// rather than leaving the caller to infer it from a bare `false`.
#[test]
fn policy_decides_the_required_lane() {
    let mut run = run_holding(&[("sec-a", "sha256:a")]);
    attest(&mut run, "att-a", "sec-a", Reviewer::Adversarial);

    assert_eq!(run.run.review_policy, ReviewPolicy::HumanOnly);
    assert!(
        !run.review_standing().sections_attested,
        "an adversarial review does not satisfy a human lane"
    );
    assert_eq!(
        run.sections_unreviewed(),
        vec![(id("sec-a"), ActorClass::User)],
        "the missing lane is named"
    );

    run.run.review_policy = ReviewPolicy::AdversarialOnly;
    assert!(
        run.review_standing().sections_attested,
        "the same attestation satisfies the lane the run now requires"
    );
    assert!(run.sections_unreviewed().is_empty());
}

/// The quantification is nested — every section, every lane the policy resolves
/// to — which is where a single-lane policy cannot reach: a run may be complete
/// in one lane and owe the other on one section only.
#[test]
fn both_lanes_required_per_section() {
    let mut run = run_holding(&[("sec-a", "sha256:a"), ("sec-b", "sha256:b")]);
    run.run.review_policy = ReviewPolicy::HumanThenAdversarial;
    attest(&mut run, "att-a1", "sec-a", Reviewer::Human);
    attest(&mut run, "att-b1", "sec-b", Reviewer::Human);
    attest(&mut run, "att-b2", "sec-b", Reviewer::Adversarial);

    assert_eq!(
        run.sections_unreviewed(),
        vec![(id("sec-a"), ActorClass::Adversarial)],
        "one section owes one lane; the other owes nothing"
    );
    assert!(!run.review_standing().sections_attested);

    attest(&mut run, "att-a2", "sec-a", Reviewer::Adversarial);
    assert!(run.review_standing().sections_attested);
}

/// DEC-073 says *intended* order, and `Attestation` carries no turn, sequence or
/// timestamp — so order is not derivable from what is stored and the gate does
/// not police it. Recording the lanes in the order the policy does **not** intend
/// clears the condition exactly as the intended order would.
#[test]
fn order_is_declared_not_enforced() {
    let mut run = run_holding(&[("sec-a", "sha256:a")]);
    run.run.review_policy = ReviewPolicy::HumanThenAdversarial;

    attest(&mut run, "att-a2", "sec-a", Reviewer::Adversarial);
    assert_eq!(
        run.sections_unreviewed(),
        vec![(id("sec-a"), ActorClass::User)],
        "the lane recorded second by intent is recorded first, and the other is owed"
    );

    attest(&mut run, "att-a1", "sec-a", Reviewer::Human);
    assert!(
        run.review_standing().sections_attested,
        "both lanes are present, and the order they arrived in is not a fact the gate holds"
    );

    // The sibling variant differs only in declared order, so it demands the same
    // pair of the same run.
    run.run.review_policy = ReviewPolicy::AdversarialThenHuman;
    assert!(run.review_standing().sections_attested);
}

/// The third reader of the attestation set, and the one the policy must **not**
/// reach. `live_reviews` feeds the invalidation rows, which report the death of a
/// recorded act — and an adversarial attestation going stale is a fact whatever
/// lanes the run currently requires.
///
/// Asserted rather than trusted because the design predicts the mistake: a sweep
/// for readers of `attestations`, applying the policy uniformly, gets this one
/// wrong and the loss is silent. The two questions are asked side by side here so
/// the difference between them is the test.
#[test]
fn invalidation_is_not_policy_filtered() {
    let mut run = run_holding(&[("sec-a", "sha256:a")]);
    attest(&mut run, "att-a", "sec-a", Reviewer::Adversarial);
    assert_eq!(run.run.review_policy, ReviewPolicy::HumanOnly);

    // The gate says this section owes a lane; the recorded act is live all the
    // same. Insufficient is not the same fact as dead.
    assert_eq!(
        run.sections_unreviewed(),
        vec![(id("sec-a"), ActorClass::User)]
    );
    let before = live_reviews(&run);
    assert_eq!(
        before.len(),
        1,
        "an attestation satisfying no required lane is still a live record"
    );

    // Editing the section is what kills it, and the difference these two sets
    // report is the invalidation row.
    run.sections.upsert(section("sec-a", "sha256:a-revised"));
    let after = live_reviews(&run);
    assert!(after.is_empty());
    assert_eq!(
        before.difference(&after).count(),
        1,
        "the death of the act is reported under a policy that never required it"
    );
}

/// The pass is bound to the content it was opened over, not to the run — which is
/// what lets a later edit stale it without anything storing a verdict (SL-244
/// PHASE-04 `VT-2`).
///
/// Coverage, not presence: a section joining the run after the pass opened is
/// content nobody looked at, and reads exactly like a covered section moving. The
/// two are asserted side by side because a `covered.contains`-style implementation
/// passes the second and fails the first, silently.
#[test]
fn review_pass_covers_the_sections_it_opened_over() {
    let mut run = run_holding(&[("sec-a", "sha256:a"), ("sec-b", "sha256:b")]);
    let pass = pass_over(&run, "RV-344");

    assert_eq!(
        pass.review,
        ReviewRef::new("RV-344"),
        "the pass names the RV it was minted for"
    );
    assert!(pass.is_current(&run.sections.fingerprints()));

    // A covered section moving is the ordinary staleness.
    run.sections.upsert(section("sec-a", "sha256:a-revised"));
    assert!(!pass.is_current(&run.sections.fingerprints()));

    // A section ARRIVING is content the pass never looked at, and stales it just
    // as hard — the case a presence check gets wrong.
    let mut widened = run_holding(&[("sec-a", "sha256:a"), ("sec-b", "sha256:b")]);
    let pass = pass_over(&widened, "RV-344");
    widened.sections.upsert(section("sec-c", "sha256:c"));
    assert!(!pass.is_current(&widened.sections.fingerprints()));
}

/// The intent subject is one string-coded slot, and the two arms cannot collide
/// in it: a checkpoint keys on its bare `DesignId`, and the run-level pass keys
/// on a reserved token no id can spell. Asserted over the wire bytes rather than
/// the enum, because the collision this rules out is a *parsing* one.
#[test]
fn the_review_pass_token_cannot_be_spelled_by_a_checkpoint_id() {
    let held: RecoveryIntent =
        toml::from_str("submission = \"sub-1\"\nsubject = \"cp-1\"\n").unwrap();
    assert_eq!(held.subject().checkpoint(), Some(&id("cp-1")));

    let pass = RecoveryIntent::journalled("sub-2", IntentSubject::ReviewPass);
    let wire = toml::to_string(&pass).unwrap();
    assert!(wire.contains("review-pass"), "the reserved token: {wire}");
    assert_eq!(toml::from_str::<RecoveryIntent>(&wire).unwrap(), pass);
    assert_eq!(pass.subject().checkpoint(), None, "a pass names no node");
}

/// `RequiredActor` names *where the actor comes from*, and resolution is what
/// fixes the conjunction's arity (design sec-3, `RequiredActor`).
///
/// The `RunPolicy` arm is asserted against `ReviewPolicy::lanes()` itself rather
/// than against a written-out lane list: the rule rides the policy's single home
/// of membership, and a second lane table beside it is the parallel
/// implementation `VA-2` refuses.
#[test]
fn a_required_actor_resolves_to_the_lanes_an_act_must_satisfy() {
    // Fixed is the singleton case — seven of the eight requirements.
    assert_eq!(
        RequiredActor::Fixed(ActorClass::User).resolve(ReviewPolicy::HumanOnly),
        [ActorClass::User]
    );
    assert_eq!(
        RequiredActor::Fixed(ActorClass::Agent).resolve(ReviewPolicy::AdversarialOnly),
        [ActorClass::Agent],
        "a fixed actor is fixed: the run's policy does not reach it"
    );

    // RunPolicy is the one requirement whose arity the run fixes, and it yields
    // exactly what the policy's own membership says — one lane or two.
    for policy in ReviewPolicy::ALL {
        assert_eq!(
            RequiredActor::RunPolicy.resolve(policy),
            policy.lanes(),
            "the actor slot reads DEC-073's policy, it does not restate it"
        );
    }
    assert_eq!(
        RequiredActor::RunPolicy
            .resolve(ReviewPolicy::HumanOnly)
            .len(),
        1
    );
    assert_eq!(
        RequiredActor::RunPolicy
            .resolve(ReviewPolicy::HumanThenAdversarial)
            .len(),
        2,
        "one requirement standing for two required acts"
    );
}

/// `ConditionKind` is a projection of the derivation rule, never a stored field
/// (design sec-3, target behaviour). `Claimed` is DEC-120's defect class and is
/// not representable — asserted by the type having two variants that both
/// project, with no third to reach.
#[test]
fn a_condition_kind_is_projected_from_the_derivation_rule() {
    assert_eq!(
        DerivationRule::Engine(EngineSource::Dispositions).kind(),
        ConditionKind::Derived
    );
    assert_eq!(
        DerivationRule::Engine(EngineSource::Materialisation).kind(),
        ConditionKind::Derived
    );

    let attested = DerivationRule::Attested(AttestationRule {
        acts: &[ActRequirement {
            act: ActKind::DraftingReady,
            actor: RequiredActor::Fixed(ActorClass::Agent),
            confirms: None,
            disposes_review: false,
        }],
        binding: Binding {
            coverage: Coverage::Artefact,
            observed: &[],
        },
    });
    assert_eq!(attested.kind(), ConditionKind::Attested);
}

/// The vocabulary composes into a whole contract, over each coverage the nine
/// rows will need — the shape `T9`'s table instantiates.
///
/// Written as construction rather than assertion because that is the property
/// under test: `Contract` states derivation, reach and prose key and *nothing
/// else*, and in particular carries no `remedy` string beside the rule it would
/// be rendered from. A field added there would fail this test by not compiling.
#[test]
fn a_contract_states_its_derivation_reach_and_prose_key_and_nothing_else() {
    let engine = Contract {
        derivation: DerivationRule::Engine(EngineSource::Materialisation),
        reach: Reach::Cumulative,
        prose: "materialisation-current",
    };
    assert_eq!(engine.derivation.kind(), ConditionKind::Derived);

    // Reach and coverage are independent axes: an edge-local row over whole-map
    // coverage, and a cumulative row over an artefact, are both coherent.
    for (reach, coverage) in [
        (Reach::Cumulative, Coverage::Artefact),
        (Reach::Cumulative, Coverage::EverySection),
        (Reach::EdgeLocal, Coverage::InquiryMap),
        (Reach::EdgeLocal, Coverage::PerSection),
    ] {
        let attested = Contract {
            derivation: DerivationRule::Attested(AttestationRule {
                acts: &[ActRequirement {
                    act: ActKind::SectionReviewed,
                    actor: RequiredActor::RunPolicy,
                    confirms: None,
                    disposes_review: false,
                }],
                binding: Binding {
                    coverage,
                    observed: &[ObservedFact::GovernanceEdges],
                },
            }),
            reach,
            prose: "section-attestations-current",
        };
        assert_eq!(attested.derivation.kind(), ConditionKind::Attested);
    }

    // The `confirms` slot ranges over agent acts only — naming a user act there
    // is a contradiction the type does not admit. The widening runs one way.
    assert_eq!(
        ActKind::from(AgentActKind::DraftingReady),
        ActKind::DraftingReady
    );
    assert_eq!(
        ActKind::from(AgentActKind::BlockingSetDeclared),
        ActKind::BlockingSetDeclared
    );
}

/// A recorded act group round-trips, and **replacement is by act** — the key
/// that distinguishes these records from `Attestation`'s.
///
/// The third assertion is the anti-regression that proves the two keys really do
/// differ: two attestations on one section coexist (different lanes), while two
/// checkpoint acts of one kind do not. Keying acts by id, or attestations by
/// subject, would each fail exactly one half of this.
#[test]
fn a_recorded_act_is_replaced_by_kind_where_an_attestation_is_replaced_by_id() {
    let mut acts = CheckpointActGroup::default();
    acts.record(checkpoint_act(
        "cpa-1",
        ActKind::GovernanceConfirmed,
        "first",
    ));
    acts.record(checkpoint_act(
        "cpa-2",
        ActKind::GraphReviewed,
        "other kind",
    ));
    acts.record(checkpoint_act(
        "cpa-3",
        ActKind::GovernanceConfirmed,
        "second",
    ));

    assert_eq!(acts.acts.len(), 2, "a second act of one kind displaces it");
    let held = acts
        .acts
        .iter()
        .find(|held| held.act == ActKind::GovernanceConfirmed)
        .expect("the surviving act");
    assert_eq!(
        held.id,
        id("cpa-3"),
        "the later act wins, id notwithstanding"
    );

    // The group survives the wire unchanged — and stores in a deterministic
    // order, so an unrelated re-record cannot churn the snapshot's bytes.
    let wire = toml::to_string(&acts).unwrap();
    assert_eq!(toml::from_str::<CheckpointActGroup>(&wire).unwrap(), acts);
    let mut reordered = CheckpointActGroup::default();
    reordered.record(checkpoint_act(
        "cpa-3",
        ActKind::GovernanceConfirmed,
        "second",
    ));
    reordered.record(checkpoint_act(
        "cpa-2",
        ActKind::GraphReviewed,
        "other kind",
    ));
    assert_eq!(toml::to_string(&reordered).unwrap(), wire);

    // The same rule for agent declarations, keyed on the narrower vocabulary.
    let mut declared = AgentDeclarationGroup::default();
    declared.record(blocking_set_declared("agd-1", &["inq-1"]));
    declared.record(blocking_set_declared("agd-2", &["inq-1", "inq-2"]));
    declared.record(drafting_ready("agd-3"));
    assert_eq!(declared.declarations.len(), 2);
    assert_eq!(
        declared
            .declarations
            .iter()
            .find(|held| held.act.kind() == AgentActKind::BlockingSetDeclared)
            .map(|held| &held.id),
        Some(&id("agd-2")),
        "a second declaration displaces the first however its set differs"
    );

    // The anti-regression: `Attestation` keys on ID, so one section reviewed in
    // two lanes holds two live attestations. If acts had been keyed the same
    // way, the displacement above would not have happened.
    let mut run = run_holding(&[("sec-a", "sha256:a")]);
    attest(&mut run, "att-1", "sec-a", Reviewer::Human);
    attest(&mut run, "att-2", "sec-a", Reviewer::Adversarial);
    assert_eq!(run.review.attestations.len(), 2);
}

/// `AgentAct` is tagged with its payload, so the two illegal shapes are
/// unrepresentable rather than refused: `DraftingReady` cannot carry a blocking
/// set, and `BlockingSetDeclared` cannot omit one.
///
/// `kind()` is the widening a rule reads, and it is the only direction — there
/// is no narrowing from `ActKind` back.
#[test]
fn an_agent_act_carries_its_payload_and_widens_to_a_kind_a_rule_can_name() {
    let declared = AgentAct::BlockingSetDeclared {
        blocking: [id("inq-2"), id("inq-1")].into_iter().collect(),
    };
    assert_eq!(declared.kind(), AgentActKind::BlockingSetDeclared);
    assert_eq!(
        ActKind::from(declared.kind()),
        ActKind::BlockingSetDeclared,
        "a requirement names an ActKind; the view answers by widening"
    );
    assert_eq!(AgentAct::DraftingReady.kind(), AgentActKind::DraftingReady);
}

/// The disposition's two arms both bind to a pass, and the pass reference sits
/// *beside* the arm rather than inside `Conducted` — because both arms dispose of
/// one pass and only one of them names an `RV` for its own reasons.
#[test]
fn a_disposition_binds_to_the_pass_it_disposed_under_either_arm() {
    let conducted = DisposedPass {
        pass: ReviewRef::new("RV-344"),
        disposition: ReviewDisposition::Conducted {
            review: ReviewRef::new("RV-344"),
        },
    };
    let waived = DisposedPass {
        pass: ReviewRef::new("RV-344"),
        disposition: ReviewDisposition::Waived {
            reason: "the design is a one-line token rename".to_owned(),
        },
    };
    assert_eq!(conducted.pass, waived.pass, "one pass, two ways to dispose");
    assert_ne!(conducted.disposition, waived.disposition);

    for held in [&conducted, &waived] {
        let wire = toml::to_string(held).unwrap();
        assert_eq!(&toml::from_str::<DisposedPass>(&wire).unwrap(), held);
    }
}

/// The generated table says what the design's classification says (`EX-1`,
/// `EX-2`).
///
/// Deliberately **not** a set-equality test over the four generated artefacts —
/// vocabulary, `ALL`, `CONTRACTS` and `boundary_conditions` come from one source
/// and no disagreement is expressible, so such a test could only ever pass. What
/// is not guaranteed by construction is the table's *content*: a row could name
/// the wrong coverage, the wrong actor, or the wrong reach and the build would be
/// perfectly happy. That is what this asserts, against the design's own
/// classification table.
#[test]
fn the_contract_table_classifies_every_condition_as_the_design_says() {
    assert_eq!(
        CONTRACTS.len(),
        Condition::ALL.len(),
        "one row per condition"
    );
    for condition in Condition::ALL {
        assert_eq!(
            CONTRACTS
                .iter()
                .filter(|(keyed, _)| *keyed == condition)
                .count(),
            1,
            "{} has exactly one contract",
            condition.as_str()
        );
    }

    // Two Derived, seven Attested, zero Claimed — DEC-126's count.
    let derived: Vec<Condition> = CONTRACTS
        .iter()
        .filter(|(_, contract)| contract.derivation.kind() == ConditionKind::Derived)
        .map(|(condition, _)| *condition)
        .collect();
    assert_eq!(
        derived,
        vec![
            Condition::BlockingInquiriesDispositioned,
            Condition::MaterialisationCurrent
        ]
    );

    // Every condition sits on exactly one boundary. Not a set-equality check —
    // membership is generated — but the *edge* a row was filed under is a
    // content decision, and filing one under the wrong edge compiles.
    for (condition, edge) in [
        (
            Condition::GoverningContextRecorded,
            Advance::ExploringInquiring,
        ),
        (
            Condition::InitialConcernsRecorded,
            Advance::ExploringInquiring,
        ),
        (
            Condition::BlockingInquiriesDispositioned,
            Advance::InquiringDrafting,
        ),
        (
            Condition::UserAcceptsSufficiency,
            Advance::InquiringDrafting,
        ),
        (
            Condition::DraftingReadinessAttested,
            Advance::DraftingReviewing,
        ),
        (
            Condition::MaterialisationCurrent,
            Advance::DraftingReviewing,
        ),
        (
            Condition::SectionAttestationsCurrent,
            Advance::ReviewingLocked,
        ),
        (
            Condition::ReviewDispositionAttested,
            Advance::ReviewingLocked,
        ),
        (Condition::UserAcceptanceAttested, Advance::ReviewingLocked),
    ] {
        assert!(
            boundary_conditions(edge).contains(&condition),
            "{} guards {edge:?}",
            condition.as_str()
        );
    }

    // The three slots the `EX-4` const assertion polices, each named by exactly
    // one row. The assertion proves no row names a slot its record shape lacks;
    // this proves the rows that SHOULD name one still do — the complement, which
    // a const predicate over an empty set would also satisfy.
    let rules: Vec<(Condition, &AttestationRule)> = CONTRACTS
        .iter()
        .filter_map(|(condition, contract)| match &contract.derivation {
            DerivationRule::Attested(rule) => Some((*condition, rule)),
            DerivationRule::Engine(_) => None,
        })
        .collect();
    assert_eq!(rules.len(), 7, "seven Attested rows");

    let named = |slot: fn(&ActRequirement, &AttestationRule) -> bool| -> Vec<Condition> {
        rules
            .iter()
            .filter(|(_, rule)| rule.acts.iter().any(|act| slot(act, rule)))
            .map(|(condition, _)| *condition)
            .collect()
    };
    assert_eq!(
        named(|act, _| act.disposes_review),
        vec![Condition::ReviewDispositionAttested]
    );
    assert_eq!(
        named(|act, _| act.confirms == Some(AgentActKind::BlockingSetDeclared)),
        vec![Condition::InitialConcernsRecorded]
    );
    assert_eq!(
        named(|_, rule| rule
            .binding
            .observed
            .contains(&ObservedFact::GovernanceEdges)),
        vec![Condition::GoverningContextRecorded]
    );

    // The one edge-local row, and the one whose actor comes from the run's
    // policy rather than from the rule (ISS-310).
    assert_eq!(
        CONTRACTS
            .iter()
            .filter(|(_, contract)| contract.reach == Reach::EdgeLocal)
            .map(|(condition, _)| *condition)
            .collect::<Vec<_>>(),
        vec![Condition::DraftingReadinessAttested]
    );
    assert_eq!(
        named(|act, _| act.actor == RequiredActor::RunPolicy),
        vec![Condition::SectionAttestationsCurrent]
    );

    // DEC-121's two-act conjunction is two acts, and stays two.
    let concerns = rules
        .iter()
        .find(|(condition, _)| *condition == Condition::InitialConcernsRecorded)
        .expect("the row exists")
        .1;
    assert_eq!(concerns.acts.len(), 2);
    assert_eq!(concerns.binding.coverage, Coverage::InquiryMap);
}

/// The remedy is rendered from the rule, and the one row with two ways through
/// renders both (`EX-2`, design `sec-6`).
///
/// The two-arm row is the point. A remedy saying only *the user performs
/// `review-disposed`* would name the obligation while hiding the waiver — the
/// only arm crossable through the whole `IMP-392` interim — and PHASE-06's
/// `VT-2` quantifies over every row, so it would pass on the easy eight and
/// never reach this one.
#[test]
fn the_remedy_renders_from_the_rule_including_the_row_with_two_arms() {
    let remedy = |wanted: Condition| {
        CONTRACTS
            .iter()
            .find(|(condition, _)| *condition == wanted)
            .map(|(_, contract)| contract.remedy())
            .expect("every condition has a contract")
    };

    // An engine row names no act, so its remedy describes work.
    assert_eq!(
        remedy(Condition::BlockingInquiriesDispositioned),
        "dispose every blocking inquiry on the map"
    );

    // A one-act row names the actor and the act's own token.
    assert_eq!(
        remedy(Condition::UserAcceptsSufficiency),
        "the user performs `sufficiency-accepted`"
    );

    // Two acts by two actors, still one way through — and the confirmation is
    // rendered, because the ordering is part of what must be done.
    let concerns = remedy(Condition::InitialConcernsRecorded);
    assert!(
        concerns.contains("naming the current `blocking-set-declared`"),
        "the confirmation rides the remedy: {concerns}"
    );
    assert!(concerns.contains("the agent performs `blocking-set-declared`"));

    // The lane-resolved row does not pretend to know the lanes.
    assert!(
        remedy(Condition::SectionAttestationsCurrent)
            .starts_with("every lane the run's review policy requires performs")
    );

    // The ninth row: two doors, and the remedy says so.
    let disposition = remedy(Condition::ReviewDispositionAttested);
    assert!(disposition.contains("conducted:"), "{disposition}");
    assert!(disposition.contains("waived:"), "{disposition}");
    assert_eq!(
        disposition.lines().count(),
        3,
        "the one multi-line discharge: {disposition}"
    );

    // Every remedy is non-empty, so no row can be added with nothing to say.
    for (condition, contract) in CONTRACTS {
        assert!(
            !contract.remedy().trim().is_empty(),
            "{} renders a remedy",
            condition.as_str()
        );
    }

    // One spelling, not two: an act's rendered token is the token it stores
    // (STD-001), quantified over the acts the table actually names — which is
    // all eight, so a hand-listed array would only be a weaker version of this.
    for (_, contract) in CONTRACTS {
        let DerivationRule::Attested(rule) = contract.derivation else {
            continue;
        };
        for required in rule.acts {
            let stored = serde_json::to_string(&required.act).expect("an act serialises");
            assert_eq!(stored, format!("\"{}\"", required.act.as_str()));
        }
    }
}

/// The act wire types carry the **claim** and nothing the engine authors
/// (`EX-7b`).
///
/// `deny_unknown_fields` is the whole guarantee here, and nothing else catches
/// this: without it a caller supplying `covered`, `observed`, `confirms` or `id`
/// would be told nothing and silently get the engine's value instead of theirs —
/// the same silent-no-op class `Declaration`'s own `deny_unknown_fields` was
/// added for (submission.rs `EX-14`). Admission cannot catch it either, because
/// by the time admission runs the key is already gone.
///
/// The positive case is asserted first, so a refusal that came from a malformed
/// payload rather than from the rejected key cannot pass as the guarantee.
#[test]
fn the_act_wire_types_carry_the_claim_and_refuse_engine_authored_slots() {
    let checkpoint = serde_json::json!({
        "act": "review-disposed",
        "acceptance": {"basis": "the pass was conducted and its findings answered"},
        "disposition": {"conducted": {"review": "RV-344"}},
    });
    let parsed: CheckpointActDeclaration = serde_json::from_value(checkpoint.clone()).unwrap();
    assert_eq!(parsed.act, ActKind::ReviewDisposed);
    assert_eq!(
        parsed.disposition,
        Some(ReviewDisposition::Conducted {
            review: ReviewRef::new("RV-344")
        })
    );

    let agent = serde_json::json!({
        "act": {"blocking-set-declared": {"blocking": ["inq-1", "inq-2"]}},
        "basis": "these two questions gate the draft",
    });
    let parsed: AgentActDeclaration = serde_json::from_value(agent.clone()).unwrap();
    assert_eq!(parsed.act.kind(), AgentActKind::BlockingSetDeclared);
    assert_eq!(
        parsed.turn, None,
        "the turn is optional, not engine-authored"
    );

    // Each engine-authored slot, refused on the wire it does not belong on.
    for slot in ["covered", "observed", "confirms", "id"] {
        let mut payload = checkpoint.clone();
        payload
            .as_object_mut()
            .unwrap()
            .insert(slot.to_owned(), serde_json::json!(null));
        assert!(
            serde_json::from_value::<CheckpointActDeclaration>(payload).is_err(),
            "a checkpoint act declaration carrying `{slot}` must not deserialise"
        );
    }
    for slot in ["covered", "id"] {
        let mut payload = agent.clone();
        payload
            .as_object_mut()
            .unwrap()
            .insert(slot.to_owned(), serde_json::json!(null));
        assert!(
            serde_json::from_value::<AgentActDeclaration>(payload).is_err(),
            "an agent declaration carrying `{slot}` must not deserialise"
        );
    }
}

/// `IdKind::ALL`'s **order is load-bearing**, and this is what says so.
///
/// `DesignId::parse` and `DesignId::kind` both resolve a prefix by walking `ALL`
/// and taking the first match, so two kinds sharing a stem must appear
/// longest-first. `cpa-` and `cp-` are the first such pair (SL-244 PHASE-05 T3);
/// before them every prefix was distinct at byte three and the ordering was free.
///
/// The negative control is the point: with the rows swapped, `cpa-1` parses as a
/// checkpoint whose body is `a1`, silently and with no error anywhere.
#[test]
fn id_kinds_sharing_a_stem_are_ordered_longest_first() {
    for (index, kind) in IdKind::ALL.iter().enumerate() {
        for other in IdKind::ALL.iter().skip(index + 1) {
            assert!(
                !other.prefix().starts_with(kind.prefix()),
                "{} precedes {}, which extends it — the longer prefix can never match",
                kind.prefix(),
                other.prefix()
            );
        }
    }

    // The pair that made this a rule, resolved both ways.
    assert_eq!(id("cpa-1").kind(), IdKind::CheckpointAct);
    assert_eq!(id("cp-1").kind(), IdKind::Checkpoint);
    assert_eq!(id("agd-1").kind(), IdKind::AgentDeclaration);
}
