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
use super::attestation::{ActorClass, ContentCoverage, ReviewPolicy, Reviewer};
use super::facts::DerivedDesignFacts;
use super::fixture::{attest, id, run_holding, section};
use super::gate::{
    Advance, Condition, ReviewStanding, advance, boundary_runbook, cumulative_conditions, regress,
};
use super::ids::{DesignId, Fingerprint};
use super::inquiry::{
    Disposition, InquiryLifecycle, InquiryMap, InquiryNode, NodeMaterial, Provenance,
};
use super::refusal::Refusal;
use super::run::live_reviews;
use super::runbook::{RunbookKey, RunbookStanding};
use super::submission::{Batch, Declaration, Sparse};

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
        .record(Condition::RequiredSectionsExist, moved.clone(), before)
        .record(
            Condition::MaterialisationCurrent,
            stable.clone(),
            untouched.clone(),
        );

    assert_eq!(facts.live_evidence().count(), 2);
    assert!(facts.satisfies(Condition::RequiredSectionsExist));
    assert!(facts.satisfies(Condition::MaterialisationCurrent));

    // One subject's content moves. Only its evidence dies.
    let after_edit = facts.observe(moved.clone(), after);
    let live: Vec<&DesignId> = after_edit
        .live_evidence()
        .map(super::facts::Evidence::subject)
        .collect();
    assert_eq!(live, vec![&stable]);
    assert!(!after_edit.satisfies(Condition::RequiredSectionsExist));
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
