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
use super::admission::admit_act;
use super::attestation::{
    ActKind, ActorClass, AgentAct, AgentActKind, ContentCoverage, CoveredSet, DisposedPass,
    IntentSubject, RecordedAct, RecoveryIntent, ReviewDisposition, ReviewPolicy, ReviewRef,
    Reviewer,
};
use super::facts::DerivedDesignFacts;
use super::fixture::{
    BLOCKING_NODE, OPEN_NODE, PASS, SECTION_A, SECTION_B, attest, blocking_set_declared,
    checkpoint_act, cleared, drafting_ready, id, pass_over, run_holding, section,
};
use super::gate::{
    ActRequirement, ActRule, Advance, AttestationRule, Binding, CONTRACTS, Cause, Condition,
    ConditionKind, Contract, Coverage, DerivationRule, EngineSource, ObservedFact, Reach,
    RequiredActor, Unmet, advance, boundary_conditions, boundary_runbook, cumulative_conditions,
    regress, requirement_for, satisfied,
};
use super::ids::{DesignId, Fingerprint, IdKind};
use super::inquiry::{
    Disposition, InquiryLifecycle, InquiryMap, InquiryNode, NodeMaterial, Provenance,
};
use super::refusal::{ActFault, Refusal};
use super::run::{DerivedInput, ObservedReview, live_reviews};
use super::runbook::{RunbookKey, RunbookStanding};
use super::snapshot::{AgentDeclarationGroup, CheckpointActGroup, DesignSnapshot};
use super::submission::{
    AgentActDeclaration, Batch, CheckpointActDeclaration, Declaration, Sparse,
};

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
    let (run, derived) = cleared();
    assert_eq!(
        advance(Stage::Exploring, Stage::Drafting, &run, &derived, None),
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
            &run,
            &derived,
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

    // DEC-067's other half: returning forward inherits no clearance. A run that
    // discharges only the drafting boundary does not re-open drafting, because
    // the *cumulative* set is re-derived against current content.
    //
    // Built by taking the cleared run's two exploring→inquiring acts away rather
    // than by constructing a partial run: what makes this assertion about
    // accumulation is that everything the crossing edge itself asks for is still
    // there, and only the edge below it is unmade.
    let (mut partial, derived) = cleared();
    partial.acts.acts.retain(|held| {
        !matches!(
            held.act,
            ActKind::GovernanceConfirmed | ActKind::GraphReviewed
        )
    });
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
            &derived,
            Some(&discharged)
        ),
        Err(Refusal::GateNotCleared {
            from: Stage::Inquiring,
            to: Stage::Drafting,
            unmet: vec![
                Unmet {
                    condition: Condition::GoverningContextRecorded,
                    causes: vec![Cause::ActMissing {
                        act: ActKind::GovernanceConfirmed,
                        lanes: vec![ActorClass::User],
                    }],
                },
                Unmet {
                    condition: Condition::InitialConcernsRecorded,
                    causes: vec![Cause::ActMissing {
                        act: ActKind::GraphReviewed,
                        lanes: vec![ActorClass::User],
                    }],
                },
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
        // The two vocabularies an act fault names are held to the same rule —
        // they are rendered into a refusal and stored on the wire, so a second
        // spelling would drift exactly where it is least visible.
        let coverage = rule.binding.coverage;
        assert_eq!(
            serde_json::to_string(&coverage).expect("a coverage serialises"),
            format!("\"{}\"", coverage.as_str())
        );
        for fact in rule.binding.observed {
            assert_eq!(
                serde_json::to_string(fact).expect("an observed fact serialises"),
                format!("\"{}\"", fact.as_str())
            );
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

// ---------------------------------------------------------------------------
// T5 — admission owns the rule/record correspondence (`EX-8`, `VT-5`).
//
// Every test below asserts the fault's **variant and payload**. A bare
// is-refused assertion passes on the wrong fault, which is exactly the drift a
// correspondence check is supposed to catch.
// ---------------------------------------------------------------------------

/// The rule an act is written against — the one lookup admission and
/// construction both go through.
fn rule_for(act: ActKind) -> ActRule {
    requirement_for(act).expect("every act is named by a contract row")
}

/// The causes `record` fails `rule` with, or an empty vector where it
/// corresponds — with the refusal's own two invariants checked in passing: it
/// names the act it refused, and its cause list is never empty.
fn faults(
    record: RecordedAct<'_>,
    rule: ActRule,
    observed: Option<&ObservedReview>,
    expected_act: ActKind,
) -> Vec<ActFault> {
    match admit_act(record, rule, observed) {
        Ok(()) => Vec::new(),
        Err(Refusal::ActAdmissionInvalid { act, causes }) => {
            assert_eq!(act, expected_act, "the refusal names the act it refused");
            assert!(!causes.is_empty(), "`causes` is documented non-empty");
            causes
        }
        Err(other) => panic!("admission refused with the wrong variant: {other:?}"),
    }
}

/// An empty covered map of either shape — enough to exercise the *selector*,
/// which is all the coverage correspondence reads.
fn covered_sections() -> CoveredSet {
    CoveredSet::Sections(ContentCoverage::of(BTreeMap::new()))
}

fn covered_nodes() -> CoveredSet {
    CoveredSet::Nodes(ContentCoverage::of(BTreeMap::new()))
}

/// Correspondence row 1: the carried map's shape is the one the rule names, and
/// `Artefact` pairs with no map at all.
///
/// The third case is the degenerate one the design argues in place: `PerSection`
/// is carried by **no** act, so a rule naming it refuses every carrying shape
/// however it is filled, while the per-section attestation — which carries no
/// map because the derivation quantifies instead — is what it corresponds to.
#[test]
fn a_covered_map_in_a_shape_its_rule_does_not_name_is_refused() {
    // `drafting-readiness-attested` binds to `Artefact`, which pairs with none.
    let mut declared = drafting_ready("agd-1");
    declared.covered = Some(covered_nodes());
    assert_eq!(
        faults(
            RecordedAct::Agent(&declared),
            rule_for(ActKind::DraftingReady),
            None,
            ActKind::DraftingReady
        ),
        vec![ActFault::CoverageMismatch {
            required: Coverage::Artefact,
            carried: Some(Coverage::InquiryMap),
        }]
    );

    // `user-acceptance-attested` binds to `EverySection`, so an act given over
    // nothing is the fault in the other direction.
    let bare = checkpoint_act("cpa-1", ActKind::DesignAccepted, "the design is right");
    assert_eq!(
        faults(
            RecordedAct::Checkpoint(&bare),
            rule_for(ActKind::DesignAccepted),
            None,
            ActKind::DesignAccepted
        ),
        vec![ActFault::CoverageMismatch {
            required: Coverage::EverySection,
            carried: None,
        }]
    );

    // `PerSection`: the attestation corresponds, the checkpoint act cannot.
    assert!(
        admit_act(
            RecordedAct::Section,
            rule_for(ActKind::SectionReviewed),
            None
        )
        .is_ok(),
        "the per-section shape is what `PerSection` corresponds to"
    );
    let quantified = ActRule {
        required: rule_for(ActKind::DesignAccepted).required,
        binding: Binding {
            coverage: Coverage::PerSection,
            observed: &[],
        },
    };
    assert_eq!(
        faults(
            RecordedAct::Checkpoint(&bare),
            quantified,
            None,
            ActKind::DesignAccepted
        ),
        vec![ActFault::CoverageMismatch {
            required: Coverage::PerSection,
            carried: None,
        }],
        "no value of a carrying shape corresponds to `PerSection` — `None` included"
    );

    // And the attestation under any other coverage, for the same reason.
    assert_eq!(
        faults(
            RecordedAct::Section,
            rule_for(ActKind::DesignAccepted),
            None,
            ActKind::SectionReviewed
        ),
        vec![ActFault::CoverageMismatch {
            required: Coverage::EverySection,
            carried: None,
        }]
    );
}

/// Correspondence row 2: the observed map's key set is **exactly** the rule's
/// fact list.
///
/// The first direction is the one that matters: an act whose map is simply
/// absent where its rule names a fact is refused rather than read as an empty
/// observation, so the conjunctive binding cannot be evaded by omitting a field.
#[test]
fn an_observed_map_that_is_not_its_rules_fact_list_is_refused() {
    let bare = checkpoint_act(
        "cpa-1",
        ActKind::GovernanceConfirmed,
        "swept the governance corpus",
    );
    assert_eq!(
        faults(
            RecordedAct::Checkpoint(&bare),
            rule_for(ActKind::GovernanceConfirmed),
            None,
            ActKind::GovernanceConfirmed
        ),
        vec![ActFault::ObservedKeys {
            missing: vec![ObservedFact::GovernanceEdges],
            extra: Vec::new(),
        }]
    );

    let mut unasked = checkpoint_act("cpa-2", ActKind::DesignAccepted, "the design is right");
    unasked.covered = Some(covered_sections());
    unasked.observed.insert(
        ObservedFact::GovernanceEdges,
        Fingerprint::new("sha256:edges"),
    );
    assert_eq!(
        faults(
            RecordedAct::Checkpoint(&unasked),
            rule_for(ActKind::DesignAccepted),
            None,
            ActKind::DesignAccepted
        ),
        vec![ActFault::ObservedKeys {
            missing: Vec::new(),
            extra: vec![ObservedFact::GovernanceEdges],
        }]
    );
}

/// Correspondence row 3: a confirmation is present exactly when the rule names a
/// declaration.
///
/// The absent direction is what keeps DEC-121's ordering — *the agent declares,
/// the user confirms* — from being droppable: a `graph-reviewed` that confirms
/// nothing would correspond to its rule perfectly if presence were not required.
#[test]
fn a_confirmation_is_required_exactly_where_its_rule_names_a_declaration() {
    let mut unconfirmed = checkpoint_act("cpa-1", ActKind::GraphReviewed, "steered the graph");
    unconfirmed.covered = Some(covered_nodes());
    assert_eq!(
        faults(
            RecordedAct::Checkpoint(&unconfirmed),
            rule_for(ActKind::GraphReviewed),
            None,
            ActKind::GraphReviewed
        ),
        vec![ActFault::Confirmation {
            expected: Some(AgentActKind::BlockingSetDeclared),
            carried: false,
        }]
    );

    let mut gratuitous = checkpoint_act("cpa-2", ActKind::SufficiencyAccepted, "enough asked");
    gratuitous.covered = Some(covered_nodes());
    gratuitous.confirms = Some(Fingerprint::new("sha256:claim"));
    assert_eq!(
        faults(
            RecordedAct::Checkpoint(&gratuitous),
            rule_for(ActKind::SufficiencyAccepted),
            None,
            ActKind::SufficiencyAccepted
        ),
        vec![ActFault::Confirmation {
            expected: None,
            carried: true,
        }]
    );
}

/// Correspondence row 4: a disposition is present exactly when the rule names
/// one — `review-disposition-attested` alone.
#[test]
fn a_disposition_is_required_exactly_where_its_rule_names_one() {
    let bare = checkpoint_act("cpa-1", ActKind::ReviewDisposed, "the pass is answered");
    assert_eq!(
        faults(
            RecordedAct::Checkpoint(&bare),
            rule_for(ActKind::ReviewDisposed),
            None,
            ActKind::ReviewDisposed
        ),
        vec![ActFault::Disposition {
            expected: true,
            carried: false,
        }]
    );

    let mut gratuitous = checkpoint_act("cpa-2", ActKind::DesignAccepted, "the design is right");
    gratuitous.covered = Some(covered_sections());
    gratuitous.disposition = Some(DisposedPass {
        pass: ReviewRef::new("RV-344"),
        disposition: ReviewDisposition::Waived {
            reason: "no reviewer was available".to_owned(),
        },
    });
    assert_eq!(
        faults(
            RecordedAct::Checkpoint(&gratuitous),
            rule_for(ActKind::DesignAccepted),
            None,
            ActKind::DesignAccepted
        ),
        vec![ActFault::Disposition {
            expected: false,
            carried: true,
        }]
    );
}

/// A `Conducted` arm may only name the pass the run is on.
///
/// The observation is supplied and concluded, so the *only* thing left for the
/// refusal to be about is which pass was named — the arrangement that stops this
/// passing on `PassNotConcluded` instead.
#[test]
fn a_conducted_disposition_naming_a_pass_the_run_is_not_on_is_refused() {
    let mut act = checkpoint_act("cpa-1", ActKind::ReviewDisposed, "the pass is answered");
    act.disposition = Some(DisposedPass {
        pass: ReviewRef::new("RV-344"),
        disposition: ReviewDisposition::Conducted {
            review: ReviewRef::new("RV-324"),
        },
    });
    let observed = ObservedReview {
        reference: ReviewRef::new("RV-324"),
        concluded: true,
        undisposed_blockers: Vec::new(),
    };
    assert_eq!(
        faults(
            RecordedAct::Checkpoint(&act),
            rule_for(ActKind::ReviewDisposed),
            Some(&observed),
            ActKind::ReviewDisposed
        ),
        vec![ActFault::ForeignPass {
            named: ReviewRef::new("RV-324"),
            current: ReviewRef::new("RV-344"),
        }]
    );
}

/// A `Conducted` arm is admissible only over a pass whose ledger says it
/// concluded — and **absence is refusal, not satisfaction**.
///
/// The third case is the one an implementer would get wrong: an observation of
/// some *other* ledger answers a question nobody asked, and reading it as this
/// pass's would be worse than reading nothing.
#[test]
fn a_conducted_disposition_over_a_pass_that_has_not_concluded_is_refused() {
    let mut act = checkpoint_act("cpa-1", ActKind::ReviewDisposed, "the pass is answered");
    act.disposition = Some(DisposedPass {
        pass: ReviewRef::new("RV-344"),
        disposition: ReviewDisposition::Conducted {
            review: ReviewRef::new("RV-344"),
        },
    });
    let unconcluded = ActFault::PassNotConcluded {
        review: ReviewRef::new("RV-344"),
    };

    let read = ObservedReview {
        reference: ReviewRef::new("RV-344"),
        concluded: false,
        undisposed_blockers: Vec::new(),
    };
    let refused = |observed: Option<&ObservedReview>| {
        faults(
            RecordedAct::Checkpoint(&act),
            rule_for(ActKind::ReviewDisposed),
            observed,
            ActKind::ReviewDisposed,
        )
    };
    assert_eq!(refused(Some(&read)), vec![unconcluded.clone()]);
    assert_eq!(
        refused(None),
        vec![unconcluded.clone()],
        "an RV the shell could not read leaves admission refusing"
    );

    let elsewhere = ObservedReview {
        reference: ReviewRef::new("RV-324"),
        concluded: true,
        undisposed_blockers: Vec::new(),
    };
    assert_eq!(
        refused(Some(&elsewhere)),
        vec![unconcluded],
        "a concluded marker on another ledger says nothing about this pass"
    );
}

/// A `Waived` arm states why the pass was declined; blank is refused.
///
/// The positive case is asserted too, and with no observation at all: a waiver
/// is admissible over any review state, which is what makes it the available
/// exit through the whole `IMP-392` interim.
#[test]
fn a_waiver_with_a_blank_reason_is_refused() {
    let waived = |reason: &str| {
        let mut act = checkpoint_act("cpa-1", ActKind::ReviewDisposed, "the pass is answered");
        act.disposition = Some(DisposedPass {
            pass: ReviewRef::new("RV-344"),
            disposition: ReviewDisposition::Waived {
                reason: reason.to_owned(),
            },
        });
        act
    };

    let blank = waived("  \n ");
    assert_eq!(
        faults(
            RecordedAct::Checkpoint(&blank),
            rule_for(ActKind::ReviewDisposed),
            None,
            ActKind::ReviewDisposed
        ),
        vec![ActFault::WaiverReasonMissing]
    );

    let stated = waived("no adversarial reviewer is available for this pass");
    assert!(
        admit_act(
            RecordedAct::Checkpoint(&stated),
            rule_for(ActKind::ReviewDisposed),
            None
        )
        .is_ok(),
        "a stated waiver is admissible over any review state"
    );
}

/// A declared blocking set names nodes of the map it was declared over.
#[test]
fn a_blocking_set_naming_nodes_outside_its_coverage_is_refused() {
    let map = map_of(vec![InquiryNode::open(
        id("inq-1"),
        "the one on the map?",
        Provenance::UserDirected,
    )]);
    let mut declared = blocking_set_declared("agd-1", &["inq-1", "inq-9"]);
    declared.covered = Some(CoveredSet::Nodes(ContentCoverage::of(map.materials())));
    assert_eq!(
        faults(
            RecordedAct::Agent(&declared),
            rule_for(ActKind::BlockingSetDeclared),
            None,
            ActKind::BlockingSetDeclared
        ),
        vec![ActFault::BlockingSetUnknownNodes {
            nodes: vec![id("inq-9")],
        }]
    );

    let held = blocking_set_declared("agd-2", &["inq-1"]);
    let mut held = held;
    held.covered = Some(CoveredSet::Nodes(ContentCoverage::of(map.materials())));
    assert!(
        admit_act(
            RecordedAct::Agent(&held),
            rule_for(ActKind::BlockingSetDeclared),
            None
        )
        .is_ok()
    );
}

/// **Every** way an act failed its rule, never the first.
///
/// The control this buys is a round-trip: an agent that fixes the coverage and
/// resubmits learns about the missing disposition now rather than one refusal
/// later.
#[test]
fn an_act_failing_twice_reports_twice() {
    // `review-disposition-attested` binds to `Artefact` and names a disposition;
    // this act gets the coverage wrong AND disposes nothing.
    let mut act = checkpoint_act("cpa-1", ActKind::ReviewDisposed, "the pass is answered");
    act.covered = Some(covered_sections());
    assert_eq!(
        faults(
            RecordedAct::Checkpoint(&act),
            rule_for(ActKind::ReviewDisposed),
            None,
            ActKind::ReviewDisposed
        ),
        vec![
            ActFault::CoverageMismatch {
                required: Coverage::Artefact,
                carried: Some(Coverage::EverySection),
            },
            ActFault::Disposition {
                expected: true,
                carried: false,
            },
        ]
    );
}

/// `requirement_for` is a **total** function of the generated table, and this is
/// what says so.
///
/// The signature returns `Option` because a search over data cannot be total to
/// the type system. What makes the `None` arm unreachable is this: every act in
/// the closed vocabulary is named by exactly one contract row. An act named by
/// none would be an unrequireable act; one named by two would make *which rule*
/// ambiguous with nothing to break the tie, and `requirement_for` would silently
/// answer with whichever row came first.
#[test]
fn every_act_kind_is_named_by_exactly_one_contract_row() {
    for act in ActKind::ALL {
        let naming: Vec<&'static str> = CONTRACTS
            .iter()
            .filter(|(_, contract)| match contract.derivation {
                DerivationRule::Attested(rule) => {
                    rule.acts.iter().any(|required| required.act == act)
                }
                DerivationRule::Engine(_) => false,
            })
            .map(|(condition, _)| condition.as_str())
            .collect();
        assert_eq!(naming.len(), 1, "`{}` is named by {naming:?}", act.as_str());
        assert_eq!(
            requirement_for(act).map(|rule| rule.required.act),
            Some(act),
            "`{}` resolves to its own requirement",
            act.as_str()
        );
    }
}

/// The claim a declaration is fingerprinted over is its act **and what it
/// declares** — which is the whole of what the confirmation link is for.
///
/// An earlier design draft hashed the act and the basis alone, so a
/// re-declaration naming different questions carried the same claim and a stale
/// `confirms` still matched. Each assertion below is one way two declarations can
/// differ; all three must move the claim.
#[test]
fn a_declaration_claims_its_act_its_blocking_set_and_its_basis() {
    let declared = AgentAct::BlockingSetDeclared {
        blocking: [id("inq-1"), id("inq-2")].into(),
    };

    let by_act = AgentAct::DraftingReady.claim_material("the sweep found these");
    let by_set = AgentAct::BlockingSetDeclared {
        blocking: [id("inq-1")].into(),
    }
    .claim_material("the sweep found these");
    let by_basis = declared.claim_material("a different sweep");
    let claim = declared.claim_material("the sweep found these");

    assert_ne!(claim, by_act, "a different act is a different claim");
    assert_ne!(
        claim, by_set,
        "a different blocking set is a different claim"
    );
    assert_ne!(claim, by_basis, "a different basis is a different claim");

    // And the material is what it says it is: the act's own kebab name, the set
    // ascending, then the basis — no id, no turn, no covered map.
    assert_eq!(
        claim,
        "blocking-set-declared\n2\ninq-1\ninq-2\nthe sweep found these\n"
    );
}

/// A basis is free text and a node id is not, so the claim must say where the
/// declared set ends.
///
/// Without that framing the two declarations below encode identically — same act,
/// and `inq-2` sitting either in the set or at the head of the basis. The agent
/// authors both halves, so a re-declaration that quietly drops a blocking
/// question could keep the user's `confirms` matching, which is exactly the
/// ordering guarantee the fingerprint exists to provide.
#[test]
fn a_basis_cannot_be_read_as_the_blocking_set_beside_it() {
    let declared = AgentAct::BlockingSetDeclared {
        blocking: [id("inq-1"), id("inq-2")].into(),
    };
    let narrowed = AgentAct::BlockingSetDeclared {
        blocking: [id("inq-1")].into(),
    };
    assert_ne!(
        declared.claim_material("the sweep found these"),
        narrowed.claim_material("inq-2\nthe sweep found these"),
        "the declared set and the basis must not be able to trade a line"
    );
}

// ── the evaluator (SL-244 PHASE-05 `T10`) ─────────────────────────────────
//
// Every test below narrows one fixture: [`cleared`] is a run in which all nine
// conditions hold, and each test unmakes the one thing it is about and asserts
// the [`Cause`]s that name it. That is what stops a passing assertion from being
// a run that was broken for some other reason — the control `F18`/`F24` had to
// build by hand for the earlier tasks is here a property of the fixture.
//
// Whole `Vec<Cause>` equality throughout, never *is-unmet*: a bare refusal
// assertion passes on the wrong cause, and the causes are the whole point of
// replacing the existential scan.

/// The causes `condition` fails with here, or a failure saying it held.
fn causes_of(condition: Condition, run: &DesignSnapshot, derived: &DerivedInput) -> Vec<Cause> {
    satisfied(condition, run, derived).expect_err("the condition must not hold here")
}

/// `condition` holds against this state.
fn assert_holds(condition: Condition, run: &DesignSnapshot, derived: &DerivedInput) {
    assert_eq!(
        satisfied(condition, run, derived),
        Ok(()),
        "`{}` must hold here",
        condition.as_str()
    );
}

/// `VT-1` — an act by the wrong actor class is not an act.
///
/// The existential scan could not express this at all: it asked whether *someone*
/// had claimed the condition against *some* subject whose bytes had not moved,
/// and a lane is not part of that question. Here the section is reviewed, by the
/// wrong reviewer, and the refusal names the lane still owed.
#[test]
fn wrong_actor_does_not_satisfy() {
    let (mut run, derived) = cleared();
    run.review
        .attestations
        .retain(|held| held.subject() != &id(SECTION_A));
    attest(
        &mut run,
        "att-adversarial",
        SECTION_A,
        Reviewer::Adversarial,
    );

    assert_eq!(
        causes_of(Condition::SectionAttestationsCurrent, &run, &derived),
        vec![Cause::SectionsUnreviewed {
            subjects: vec![(id(SECTION_A), ActorClass::User)],
        }],
        "the run's policy is HumanOnly, so an adversarial pass leaves the user lane owed"
    );
    // The control: the section the fixture left alone is not named, so this is
    // about the lane and not about the subject.
    assert!(run.sections.find(&id(SECTION_B)).is_some());
}

/// `VT-1` — a conjunction that loses one half says which half.
///
/// DEC-121 makes `initial-concerns-recorded` two acts by two actors precisely so
/// a refusal can name the missing one. Both causes are asserted, in order: the
/// user's review now confirms a declaration that is not there, and the agent's
/// declaration is missing outright.
#[test]
fn missing_conjunct_names_the_missing_act() {
    let (mut run, derived) = cleared();
    run.declarations
        .declarations
        .retain(|held| held.act.kind() != AgentActKind::BlockingSetDeclared);

    assert_eq!(
        causes_of(Condition::InitialConcernsRecorded, &run, &derived),
        vec![
            Cause::ConfirmationStale {
                act: ActKind::GraphReviewed,
                declaration: AgentActKind::BlockingSetDeclared,
            },
            Cause::ActMissing {
                act: ActKind::BlockingSetDeclared,
                lanes: vec![ActorClass::Agent],
            },
        ]
    );
}

/// `VT-1` — a review given over a map that has since moved, beside a declaration
/// that has not.
///
/// The pair is what makes this a test rather than two: the agent re-declares over
/// the new map, so the only stale half is the user's review of it. A scan for
/// *someone claimed this* would have found the current declaration and stopped.
#[test]
fn stale_conjunct_does_not_satisfy() {
    let (mut run, derived) = cleared();
    let late = id("inq-3");
    run.map
        .inquiry
        .insert(InquiryNode::open(
            late.clone(),
            "what did the graph review not see?",
            Provenance::AgentProposed,
        ))
        .expect("a fresh node closes no cycle");
    // Re-declared over the new map, with the same blocking set — so the claim
    // digest, and with it the confirmation link, does not move.
    let mut redeclared = blocking_set_declared("agd-blocking", &[BLOCKING_NODE]);
    redeclared.covered = Some(CoveredSet::Nodes(ContentCoverage::of(
        run.map.inquiry.materials(),
    )));
    run.declarations.record(redeclared);

    assert_eq!(
        causes_of(Condition::InitialConcernsRecorded, &run, &derived),
        vec![Cause::CoverageStale {
            act: ActKind::GraphReviewed,
            moved: vec![late],
        }]
    );
}

/// `VT-2` — a departing section is a failure for one coverage and not the other,
/// which is why they are two variants.
///
/// `PerSection` quantifies over the sections the run holds **now**, so a leaver
/// takes its own requirement with it. `EverySection` compares an act's covered
/// map against that same set, so a leaver is a difference the act was not given
/// over.
#[test]
fn departing_section_is_not_a_failure() {
    let (mut run, derived) = cleared();
    run.sections
        .sections
        .retain(|held| held.id != id(SECTION_B));

    assert_holds(Condition::SectionAttestationsCurrent, &run, &derived);
    assert_eq!(
        causes_of(Condition::UserAcceptanceAttested, &run, &derived),
        vec![Cause::CoverageStale {
            act: ActKind::DesignAccepted,
            moved: vec![id(SECTION_B)],
        }]
    );
}

/// `VT-2` — editing one section unmakes that section's own review and no other.
#[test]
fn per_section_invalidates_only_its_own_subject() {
    let (mut run, derived) = cleared();
    run.sections
        .upsert(section(SECTION_A, "sha256:a-redrafted"));

    assert_eq!(
        causes_of(Condition::SectionAttestationsCurrent, &run, &derived),
        vec![Cause::SectionsUnreviewed {
            subjects: vec![(id(SECTION_A), ActorClass::User)],
        }],
        "`sec-b` was not edited and is not owed"
    );
}

/// `VT-2` — a run holding no sections is `NoSections`, not satisfied.
///
/// The non-empty guard is part of what `PerSection` **means**, not a check beside
/// it: a whole-map equality would find nothing owed and lock an empty document.
/// Distinct from an empty [`Cause::SectionsUnreviewed`], which no path produces.
#[test]
fn empty_document_is_no_sections() {
    let (mut run, derived) = cleared();
    run.sections.sections.clear();

    assert_eq!(
        causes_of(Condition::SectionAttestationsCurrent, &run, &derived),
        vec![Cause::NoSections]
    );
}

/// `VT-3` — progress through the inquiry graph does not invalidate an acceptance
/// given over it; a change to the graph does.
///
/// The distinction `NodeMaterial` exists to draw (PHASE-02 `EX-2`), read here at
/// the condition rather than at the projection: answering a question is what the
/// run is *for*, and an acceptance that expired every time one was answered would
/// be unreachable.
#[test]
fn progress_does_not_invalidate_but_shape_does() {
    let (mut run, derived) = cleared();
    let open = id(OPEN_NODE);
    let disposed = run
        .map
        .inquiry
        .get(&open)
        .expect("the fixture holds the open node")
        .clone()
        .resolve(Disposition::RetainedUnresolved {
            note: "answered in passing".to_owned(),
        });
    run.map
        .inquiry
        .insert(disposed)
        .expect("disposing closes no cycle");

    assert_holds(Condition::UserAcceptsSufficiency, &run, &derived);

    run.map
        .inquiry
        .insert(InquiryNode::open(
            open.clone(),
            "is inq-2 settled, exactly?",
            Provenance::AgentProposed,
        ))
        .expect("re-wording closes no cycle");

    assert_eq!(
        causes_of(Condition::UserAcceptsSufficiency, &run, &derived),
        vec![Cause::CoverageStale {
            act: ActKind::SufficiencyAccepted,
            moved: vec![open],
        }],
        "the node that was re-worded, and no other"
    );
}

/// `VT-6` — an `EdgeLocal` row is enforced by the edge that names it and by no
/// edge above.
///
/// Observable exactly where the design argued it: `drafting-readiness-attested`
/// is a judgement that drafting may begin, and re-asserting it two stages later
/// asks a question with no meaning. The cumulative row on the same edge is the
/// positive control — the filter discriminates by reach, not by edge.
#[test]
fn edge_local_is_not_accumulated() {
    assert!(
        cumulative_conditions(Stage::Reviewing).contains(&Condition::DraftingReadinessAttested),
        "the edge that names it enforces it"
    );
    assert!(
        !cumulative_conditions(Stage::Locked).contains(&Condition::DraftingReadinessAttested),
        "and no edge above does"
    );
    assert!(
        cumulative_conditions(Stage::Locked).contains(&Condition::MaterialisationCurrent),
        "while the cumulative row on that same edge still is"
    );
}

/// `VT-6` — the bottom edge enforces two conditions, and a run holding neither
/// of their checkpoint acts is refused there naming both.
#[test]
fn bottom_edge_enforces_two_conditions() {
    assert_eq!(
        cumulative_conditions(Stage::Inquiring),
        vec![
            Condition::GoverningContextRecorded,
            Condition::InitialConcernsRecorded,
        ]
    );

    let (mut run, derived) = cleared();
    run.acts.acts.retain(|held| {
        !matches!(
            held.act,
            ActKind::GovernanceConfirmed | ActKind::GraphReviewed
        )
    });
    let refusal = advance(
        Stage::Exploring,
        Stage::Inquiring,
        &run,
        &derived,
        Some(&RunbookStanding::default()),
    )
    .expect_err("neither act is recorded");
    let Refusal::GateNotCleared { unmet, .. } = refusal else {
        panic!("the conditions are what refuse here: {refusal:?}");
    };
    assert_eq!(
        unmet
            .iter()
            .map(|held| held.condition)
            .collect::<Vec<Condition>>(),
        vec![
            Condition::GoverningContextRecorded,
            Condition::InitialConcernsRecorded,
        ],
        "both, never the first"
    );
}

/// `VT-7` — a backward move clears nothing and breaks nothing.
///
/// DEC-067's regression is a change of *position*, not of standing: no act is
/// invalidated by retreating, so the same crossing succeeds again on the same
/// acts. The half of DEC-067 that does bite — no clearance is inherited — is
/// `direct_regression_requires_a_recorded_reason`'s.
#[test]
fn backward_move_clears_nothing() {
    let (mut run, derived) = cleared();
    let (acts, declarations) = (run.acts.clone(), run.declarations.clone());
    let recorded = regress(Stage::Reviewing, Stage::Drafting, "the framing was wrong")
        .expect("a backward adjacent move is a regression");
    run.run.stage = recorded.to();

    // "Clears nothing", literally: a regression is a change of position, and the
    // record of what has been done is not part of the position.
    assert_eq!((&run.acts, &run.declarations), (&acts, &declarations));
    assert_eq!(
        advance(
            Stage::Drafting,
            Stage::Reviewing,
            &run,
            &derived,
            Some(&RunbookStanding::default())
        ),
        Ok(Stage::Reviewing),
        "nothing was spent by retreating"
    );
}

/// `VT-7` — an excursion re-earns only what moved during it.
///
/// A section redrafted while the run stood at `drafting` costs that section's own
/// review and nothing else: the crossing back up asks nothing about section
/// attestations, and the lock asks about exactly the one subject that changed.
#[test]
fn excursion_re_earns_only_what_moved() {
    let (mut run, derived) = cleared();
    run.run.stage = Stage::Drafting;
    run.sections
        .upsert(section(SECTION_A, "sha256:a-redrafted"));

    assert_eq!(
        advance(
            Stage::Drafting,
            Stage::Reviewing,
            &run,
            &derived,
            Some(&RunbookStanding::default())
        ),
        Ok(Stage::Reviewing),
        "no condition on this edge is about section review"
    );
    assert_eq!(
        causes_of(Condition::SectionAttestationsCurrent, &run, &derived),
        vec![Cause::SectionsUnreviewed {
            subjects: vec![(id(SECTION_A), ActorClass::User)],
        }],
        "and the lock owes the edited section, and only it"
    );
}

/// `VT-9` — the `Waived` arm clears the lock over live findings, and dismisses
/// none of them.
///
/// DEC-138 fixes the two arms as answering different questions: a waiver says
/// *no adversarial pass is available*, which is true whatever the ledger holds,
/// and it is the arm that stays crossable through the whole `IMP-392` interim
/// (`A3`). The control is the other arm over the **same** observation — the
/// findings are not dismissed, they simply are not what a waiver answers.
#[test]
fn waiver_clears_over_live_findings_and_dismisses_none() {
    let (run, mut derived) = cleared();
    let findings = vec!["F-1".to_owned(), "F-4".to_owned()];
    derived.observed_review = Some(ObservedReview {
        reference: ReviewRef::new(PASS),
        concluded: false,
        undisposed_blockers: findings.clone(),
    });

    assert_holds(Condition::ReviewDispositionAttested, &run, &derived);

    let mut conducted = run.clone();
    let act = conducted
        .acts
        .acts
        .iter_mut()
        .find(|held| held.act == ActKind::ReviewDisposed)
        .expect("the fixture disposes the pass");
    act.disposition = Some(DisposedPass {
        pass: ReviewRef::new(PASS),
        disposition: ReviewDisposition::Conducted {
            review: ReviewRef::new(PASS),
        },
    });

    assert_eq!(
        causes_of(Condition::ReviewDispositionAttested, &conducted, &derived),
        vec![Cause::BlockersUndisposed { findings }],
        "the same findings hold the edge under the arm that reads them"
    );
}

/// `VT-9` — a disposition expires with the pass it answered, and an unreadable
/// ledger is a refusal rather than a silence.
///
/// Two ways one stored act stops being the answer, asserted together because both
/// are the *live* half of the split `T5` drew: admission asks whether the claim
/// was true when written, and this asks whether it still is.
#[test]
fn a_disposition_expires_with_the_pass_it_answered() {
    let (mut run, derived) = cleared();
    let superseding = "RV-245";
    run.review.pass = Some(pass_over(&run, superseding));

    assert_eq!(
        causes_of(Condition::ReviewDispositionAttested, &run, &derived),
        vec![Cause::PassSuperseded {
            disposed: ReviewRef::new(PASS),
            current: ReviewRef::new(superseding),
        }],
        "both references, because the repair is to dispose the new pass"
    );

    // The other axis: the run is on the pass the act disposed, the act names an
    // `RV` by the `Conducted` arm, and the shell could not read it.
    let (mut run, mut derived) = cleared();
    let act = run
        .acts
        .acts
        .iter_mut()
        .find(|held| held.act == ActKind::ReviewDisposed)
        .expect("the fixture disposes the pass");
    act.disposition = Some(DisposedPass {
        pass: ReviewRef::new(PASS),
        disposition: ReviewDisposition::Conducted {
            review: ReviewRef::new(PASS),
        },
    });
    derived.observed_review = None;

    assert_eq!(
        causes_of(Condition::ReviewDispositionAttested, &run, &derived),
        vec![Cause::ReviewUnavailable {
            review: ReviewRef::new(PASS),
        }],
        "an unreadable ledger names no findings — nobody has seen them"
    );
}

/// `VT-11` — the confirmation link runs one way and expires on its own axis.
///
/// The agent re-declares a wider blocking set after the user reviewed the narrow
/// one. The declaration is current and its coverage is current; what is stale is
/// that this is no longer the claim the user was shown.
///
/// The moved digest is set directly. The claim material it stands for is the
/// shell's to hash, and `a_basis_cannot_be_read_as_the_blocking_set_beside_it`
/// is where that encoding is pinned — here the fingerprint is the input.
#[test]
fn late_declaration_does_not_satisfy() {
    let (mut run, derived) = cleared();
    let mut relisted = blocking_set_declared("agd-blocking", &[BLOCKING_NODE, OPEN_NODE]);
    relisted.covered = Some(CoveredSet::Nodes(ContentCoverage::of(
        run.map.inquiry.materials(),
    )));
    relisted.fingerprint = Fingerprint::new("sha256:agd-blocking-relisted");
    run.declarations.record(relisted);

    assert_eq!(
        causes_of(Condition::InitialConcernsRecorded, &run, &derived),
        vec![Cause::ConfirmationStale {
            act: ActKind::GraphReviewed,
            declaration: AgentActKind::BlockingSetDeclared,
        }]
    );
}

/// `VT-11` — the material moving does not move the claim digest, so the two
/// mechanisms fail separately.
///
/// A second test rather than a second assertion in the one above, because the
/// fingerprint and the coverage answer different questions — *is this the claim
/// the user was shown* and *has the material moved since* — and one test would
/// let either mechanism cover for the other's absence.
#[test]
fn coverage_does_not_move_the_declaration_fingerprint() {
    let (mut run, derived) = cleared();
    let digest = |run: &DesignSnapshot| {
        run.declarations
            .declarations
            .iter()
            .find(|held| held.act.kind() == AgentActKind::BlockingSetDeclared)
            .expect("the fixture declares a blocking set")
            .fingerprint
            .clone()
    };
    let before = digest(&run);

    let open = id(OPEN_NODE);
    run.map
        .inquiry
        .insert(InquiryNode::open(
            open.clone(),
            "is inq-2 settled, exactly?",
            Provenance::AgentProposed,
        ))
        .expect("re-wording closes no cycle");

    assert_eq!(
        digest(&run),
        before,
        "the claim digest is over the declared set, not over the map"
    );
    let causes = causes_of(Condition::InitialConcernsRecorded, &run, &derived);
    assert_eq!(
        causes,
        vec![
            Cause::CoverageStale {
                act: ActKind::GraphReviewed,
                moved: vec![open.clone()],
            },
            Cause::CoverageStale {
                act: ActKind::BlockingSetDeclared,
                moved: vec![open],
            },
        ],
        "both acts were given over the map, so both lost their coverage"
    );
    assert!(
        !causes
            .iter()
            .any(|cause| matches!(*cause, Cause::ConfirmationStale { .. })),
        "and `confirms` still matches, because nothing about the claim moved"
    );
}
