// SPDX-License-Identifier: GPL-3.0-only
//! The design §9.1 pure-engine suite — eight tests, named by SL-233 PHASE-02
//! EX-8, operating on values and injected [`DerivedInput`] only.
//!
//! No clock, disk, git, or rng is reachable from here, because none is reachable
//! from the module under test.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test code — the repo's panic-avoidance denials target production paths"
)]

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::Stage;
use super::admission::admit_act;
use super::attestation::{
    ActKind, ActorClass, AgentAct, AgentActKind, ContentCoverage, CoveredSet, DisposedPass,
    IntentSubject, RecordedAct, RecoveryIntent, ReviewDisposition, ReviewPolicy, ReviewRef,
    Reviewer,
};
use super::change_log::{ChangeEvent, ChangeRow, PayloadKey, PayloadTerm, ValueKind};
use super::contract_check::refuse_unknown_keys;
use super::fixture::{
    BLOCKING_NODE, OPEN_NODE, PASS, SECTION_A, SECTION_B, attest, blocking_set_declared,
    checkpoint_act, cleared, declared, drafting_ready, id, pass_over, run_holding, section,
};
use super::gate::{
    ActRequirement, ActRule, Advance, AttestationRule, Binding, CONTRACTS, Cause, Condition,
    ConditionKind, Contract, Coverage, DerivationRule, EngineSource, ObservedFact, Reach,
    RequiredActor, Unmet, advance, boundary_conditions, boundary_runbook, cumulative_conditions,
    forward_unmet, regress, requirement_for, satisfied,
};
use super::ids::{DesignId, Fingerprint, IdKind, SubjectState};
use super::inquiry::{
    Disposition, InquiryLifecycle, InquiryMap, InquiryNode, NodeMaterial, Provenance,
};
use super::payload_contract::{
    ACCEPTANCE_DECLARATION, ACT_KIND, AGENT_ACT, AGENT_ACT_DECLARATION, CHECKPOINT_ACT_DECLARATION,
    CREATE_RECORD, DECLARATION, DISCHARGE_DECLARATION, Fields, KeyContract, MapKey, PAYLOAD,
    Placement, Presence, RETIRED_KEYS, REVIEW_POLICY_DECLARATION, RetiredKey, STAGE,
    STAGE_DECLARATION, TRAVERSAL_DECLARATION, Tagging, TokenSource, TypeContract, TypeForm,
    UnknownKeys, VariantContract, VariantPayload, VariantSample, WireType, claims, closure_types,
    is_legacy_token, place, required_keys,
};
use super::prompt::contract_block;
use super::refusal::{ActFault, Refusal};
use super::run::{
    Applied, AuthoredSection, Crossing, DerivedInput, GateFacts, ObservedReview, Resolution,
    ShapingQuestion, apply, declare, import, live_reviews, subject_state,
};
use super::runbook::{RunbookKey, RunbookStanding};
use super::snapshot::{AgentDeclarationGroup, CheckpointActGroup, DesignSnapshot, Finding};
use super::submission::{
    AcceptanceDeclaration, AgentActDeclaration, ApplyRequest, Batch, CheckpointActDeclaration,
    CreateRecord, Declaration, DischargeDeclaration, KeyHome, KeyWhen, ReviewPolicyDeclaration,
    Sparse, StageDeclaration, SubmissionEnvelope, TraversalDeclaration,
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
        advance(Stage::Exploring, Stage::Drafting, &run, &derived.gate, None),
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
            &derived.gate,
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

/// The edge's wire token round-trips, and no two edges share one (SL-244
/// PHASE-06 T1).
///
/// `--known-contracts` takes this token, so `parse` is the inverse of `as_str`
/// or a caller can declare a receipt no edge answers to. Driven off
/// [`Advance::ALL`] rather than a literal list: the vocabulary and the test are
/// then the same statement, and a fifth edge cannot arrive untested.
#[test]
fn every_edge_token_round_trips_and_no_two_edges_share_one() {
    for edge in Advance::ALL {
        assert_eq!(Advance::parse(edge.as_str()), Some(edge), "{edge:?}");
    }

    let tokens: BTreeSet<&'static str> = Advance::ALL.iter().map(|edge| edge.as_str()).collect();
    assert_eq!(tokens.len(), Advance::ALL.len());

    // A token no edge answers to is `None`, not a near-match: the receipt's
    // fail-open rule turns on this answer, and a lenient parse would elide
    // bodies for a caller that declared nothing real.
    assert_eq!(Advance::parse("exploring"), None);
    assert_eq!(Advance::parse("exploring-locked"), None);
    assert_eq!(Advance::parse(""), None);
}

/// The token names the two stages the edge joins, and `to` names the second of
/// them (SL-244 PHASE-06 T1, `D3`).
///
/// One test for both because they are one claim. `from_stage` supplies the
/// origin, so the relationship is pinned without `Advance` growing a `from()` —
/// the accessor PHASE-01 refused, and this is why it is still not needed.
#[test]
fn an_edge_token_names_the_two_stages_it_joins() {
    let mut checked = 0usize;
    for stage in Stage::ALL {
        let Some(edge) = Advance::from_stage(stage) else {
            continue;
        };
        assert_eq!(
            edge.as_str(),
            format!("{}-{}", stage.as_str(), edge.to().as_str()),
            "{edge:?}"
        );
        // `to` is pinned against the forward graph itself, not against a second
        // list of destinations: the edge leaving `stage` and arriving at
        // `edge.to()` must be the edge we started from.
        assert_eq!(Advance::between(stage, edge.to()), Some(edge));
        checked += 1;
    }
    assert_eq!(checked, Advance::ALL.len());
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
            &derived.gate,
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
fn parent_and_needs_cycles_are_refused() {
    let (a, b) = (id("inq-a"), id("inq-b"));

    // Parent relation: a → b → a.
    let mut map = InquiryMap::default();
    map.insert(InquiryNode::open(
        a.clone(),
        "a?",
        Provenance::UserDirected,
        Some(false),
    ))
    .expect("root");
    map.insert(
        InquiryNode::open(b.clone(), "b?", Provenance::AgentProposed, Some(false))
            .with_parent(a.clone()),
    )
    .expect("child");
    assert_eq!(
        map.insert(
            InquiryNode::open(a.clone(), "a?", Provenance::UserDirected, Some(false))
                .with_parent(b.clone())
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
        .insert(InquiryNode::open(
            a.clone(),
            "a?",
            Provenance::UserDirected,
            Some(false),
        ))
        .expect("root");
    needs_map
        .insert(
            InquiryNode::open(b.clone(), "b?", Provenance::AgentProposed, Some(false))
                .needing(a.clone()),
        )
        .expect("dependant");
    assert_eq!(
        needs_map.insert(
            InquiryNode::open(a.clone(), "a?", Provenance::UserDirected, Some(false))
                .needing(b.clone())
        ),
        Err(Refusal::CyclicEdge { from: b, to: a })
    );

    // A diamond is not a cycle. `needs` makes them routine, and a checker that
    // reports every re-reached node would refuse this.
    let (d, e, f, g) = (id("inq-d"), id("inq-e"), id("inq-f"), id("inq-g"));
    let mut diamond = InquiryMap::default();
    diamond
        .insert(InquiryNode::open(
            g.clone(),
            "g?",
            Provenance::UserDirected,
            Some(false),
        ))
        .expect("sink");
    diamond
        .insert(
            InquiryNode::open(e.clone(), "e?", Provenance::UserDirected, Some(false))
                .needing(g.clone()),
        )
        .expect("left");
    diamond
        .insert(
            InquiryNode::open(f.clone(), "f?", Provenance::UserDirected, Some(false)).needing(g),
        )
        .expect("right");
    assert_eq!(
        diamond.insert(
            InquiryNode::open(d.clone(), "d?", Provenance::UserDirected, Some(false))
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
        Some(false),
    ))
    .expect("blocker");
    map.insert(
        InquiryNode::open(
            waiting.clone(),
            "depends",
            Provenance::AgentProposed,
            Some(false),
        )
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
    let node = InquiryNode::open(
        node_id.clone(),
        "why?",
        Provenance::UserDirected,
        Some(false),
    );

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
    // Neither declaration carries a state-axis key, so the state cannot decide
    // this refusal; `Absent` is the empty run these bare subjects imply.
    assert_eq!(
        duplicated.validate(|_| SubjectState::Absent),
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
        .validate(|_| SubjectState::Absent)
        .expect("valid");
    let reversed = declarations([&three, &two, &one])
        .validate(|_| SubjectState::Absent)
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
            Some(false),
        )
        .sequenced(0)
    };
    let child = || {
        InquiryNode::open(
            id("inq-2"),
            "what does a refusal owe its reader?",
            Provenance::AgentProposed,
            Some(false),
        )
        .sequenced(1)
        .with_parent(id("inq-1"))
    };
    let sibling = || {
        InquiryNode::open(
            id("inq-3"),
            "where does the material live?",
            Provenance::AgentProposed,
            Some(false),
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
            Some(false),
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
            Some(false),
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
        InquiryNode::open(
            id("inq-4"),
            "and who reads it?",
            Provenance::UserDirected,
            Some(false),
        )
        .sequenced(3),
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
        !run.sections_unreviewed().is_empty(),
        "an adversarial review does not satisfy a human lane"
    );
    assert_eq!(
        run.sections_unreviewed(),
        vec![(id("sec-a"), ActorClass::User)],
        "the missing lane is named"
    );

    run.run.review_policy = ReviewPolicy::AdversarialOnly;
    assert!(
        run.sections_unreviewed().is_empty(),
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
    assert!(!run.sections_unreviewed().is_empty());

    attest(&mut run, "att-a2", "sec-a", Reviewer::Adversarial);
    assert!(run.sections_unreviewed().is_empty());
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
        run.sections_unreviewed().is_empty(),
        "both lanes are present, and the order they arrived in is not a fact the gate holds"
    );

    // The sibling variant differs only in declared order, so it demands the same
    // pair of the same run.
    run.run.review_policy = ReviewPolicy::AdversarialThenHuman;
    assert!(run.sections_unreviewed().is_empty());
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

/// SL-249 `VT-4` — the pre-upgrade journal shape. An intent written before the
/// payload digest existed carries no such key at all, and must resume exactly as
/// it does today: the guard is additive and never converts a recoverable state
/// into a stuck one (`R9`). Asserted over a literal wire fragment rather than a
/// round-trip, because the shape that must keep parsing is the one an older
/// binary wrote, not one this binary can still produce.
#[test]
fn an_intent_journalled_before_the_payload_digest_resumes_unguarded() {
    let held: RecoveryIntent =
        toml::from_str("submission = \"sub-1\"\nsubject = \"cp-1\"\nstate = \"materialised\"\n")
            .unwrap();

    assert_eq!(held.payload_digest(), None, "no digest was ever journalled");
    assert!(
        held.resumable_under(Some(&Fingerprint::new("sha256:anything"))),
        "an unguarded intent resumes under whatever payload the retry carries"
    );
    assert!(held.resumable_under(None), "and under none at all");
}

/// SL-249 `VT-3`, arm one — a journalled digest admits the payload it bound and
/// refuses any other. The `None`-current arm is asserted too: it is unreachable
/// today (every declaration-borne mint digests), and asserting the conservative
/// side is what keeps it unreachable rather than silently permissive.
#[test]
fn a_journalled_payload_digest_admits_only_the_payload_it_bound() {
    let bound = Fingerprint::new("sha256:one");
    let intent = RecoveryIntent::journalled("sub-1", IntentSubject::Checkpoint(id("cp-1")))
        .with_payload(Some(bound.clone()));

    assert!(intent.resumable_under(Some(&bound)), "the same payload");
    assert!(
        !intent.resumable_under(Some(&Fingerprint::new("sha256:two"))),
        "a changed payload is refused before anything is resumed"
    );
    assert!(
        !intent.resumable_under(None),
        "a guarded intent is not unguarded by a retry that offers no digest"
    );
}

/// SL-249 `VT-3`, arm two — what the digest is *over*. The material is the
/// [`Declaration`]'s serde form, so a semantically identical payload rebuilt from
/// scratch digests identically and a legitimate re-send is not refused for key
/// order.
///
/// This is the arm that fails if someone digests the raw request text or a debug
/// rendering instead: both differ between the two spellings below, which agree on
/// every value. Asserted as an equality over the serialised form rather than over
/// a hash, because the pure layer never hashes — it is handed the digest as a
/// derived fact ([`Fingerprint::new`]), and equal material hashes equally.
#[test]
fn a_declaration_rebuilt_from_scratch_digests_as_the_one_it_retries() {
    let sent = r#"{"subject":"cp-1","disposes":"inq-1","dispose":{"form":"create",
        "kind":"decision","title":"T","body":"prose"}}"#;
    // The same declaration, spelled by a caller that rebuilt it: keys in another
    // order, and an optional the first spelling omitted written out as null.
    let rebuilt = r#"{"dispose":{"title":"T","body":"prose","slug":null,
        "kind":"decision","form":"create"},"disposes":"inq-1","subject":"cp-1"}"#;

    let a: Declaration = serde_json::from_str(sent).unwrap();
    let b: Declaration = serde_json::from_str(rebuilt).unwrap();
    let (a, b) = (
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap(),
    );
    assert_eq!(
        a, b,
        "the digest material is the Declaration's serde form, not the bytes the \
         caller happened to send"
    );

    let intent = RecoveryIntent::journalled("sub-1", IntentSubject::Checkpoint(id("cp-1")))
        .with_payload(Some(Fingerprint::new(a)));
    assert!(
        intent.resumable_under(Some(&Fingerprint::new(b))),
        "so the rebuilt payload resumes rather than being refused"
    );
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

    // The `EX-4` const assertion polices the slots a record shape cannot hold.
    // This proves the rows that SHOULD name a slot still do — the complement,
    // which a const predicate over an empty set would also satisfy. `confirms`
    // is no longer among them: `SL-264` sec-3 retires the link from the rule
    // (`initial-concerns-recorded` names none), so a stored digest is read
    // through `Cause::ConfirmationStale` rather than through a row.
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
        named(|act, _| act.confirms.is_some()),
        Vec::<Condition>::new(),
        "no row names a confirmation any more — the `confirms` link left the rule"
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

    // DEC-121's two actors survive in one act: the review is the single
    // requirement, and the retired `blocking-set-declared` half no longer names
    // a row (SL-264 sec-3).
    let concerns = rules
        .iter()
        .find(|(condition, _)| *condition == Condition::InitialConcernsRecorded)
        .expect("the row exists")
        .1;
    assert_eq!(concerns.acts.len(), 1);
    assert_eq!(concerns.acts[0].act, ActKind::GraphReviewed);
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

    // A one-act row names the actor and the act's own token — and, for a user
    // act, who records it (IMP-467): the user assents, the agent submits.
    assert_eq!(
        remedy(Condition::UserAcceptsSufficiency),
        "the user performs `sufficiency-accepted` (you record it on their assent)"
    );

    // The one-act row names the actor and the act's own token — and, for a user
    // act, who records it (IMP-467): the user assents, the agent submits. The
    // retired `blocking-set-declared` confirmation no longer appears.
    assert_eq!(
        remedy(Condition::InitialConcernsRecorded),
        "the user performs `graph-reviewed` (you record it on their assent)"
    );

    // The lane-resolved row does not pretend to know the lanes.
    let sections = remedy(Condition::SectionAttestationsCurrent);
    assert!(sections.starts_with("every lane the run's review policy requires performs"));
    assert!(
        sections.contains("(you record the human lane's on the user's assent)"),
        "{sections}"
    );

    // The ninth row: two doors, and the remedy says so.
    let disposition = remedy(Condition::ReviewDispositionAttested);
    assert!(disposition.contains("conducted:"), "{disposition}");
    assert!(disposition.contains("waived:"), "{disposition}");
    assert!(
        disposition
            .starts_with("the user disposes this review pass (you record it on their assent):"),
        "{disposition}"
    );
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

/// Every field the receipt injects is spelled once, and the contract table
/// exercises the whole vocabulary (SL-244 PHASE-06 `T2`, STD-001).
///
/// The pinned literals are the wire half: these tokens ride the rendered block a
/// caller reads, so they are stated rather than derived from variant names. The
/// second leg is what stops the first from being a restatement of the match arms
/// — the tokens `CONTRACTS` actually reaches are set-equal to the pinned ones, so
/// a variant no row uses, or a seventh token a row needs and the vocabulary does
/// not have, fails here.
#[test]
fn the_injected_field_vocabulary_is_one_kebab_token_per_variant() {
    assert_eq!(Reach::Cumulative.as_str(), "cumulative");
    assert_eq!(Reach::EdgeLocal.as_str(), "edge-local");
    assert_eq!(ConditionKind::Derived.as_str(), "derived");
    assert_eq!(ConditionKind::Attested.as_str(), "attested");
    assert_eq!(EngineSource::Dispositions.as_str(), "dispositions");
    assert_eq!(EngineSource::Materialisation.as_str(), "materialisation");

    let mut reaches = BTreeSet::new();
    let mut kinds = BTreeSet::new();
    let mut sources = BTreeSet::new();
    for (_, contract) in CONTRACTS {
        reaches.insert(contract.reach.as_str());
        kinds.insert(contract.derivation.kind().as_str());
        if let DerivationRule::Engine(source) = contract.derivation {
            sources.insert(source.as_str());
        }
    }
    assert_eq!(reaches, BTreeSet::from(["cumulative", "edge-local"]));
    assert_eq!(kinds, BTreeSet::from(["attested", "derived"]));
    assert_eq!(sources, BTreeSet::from(["dispositions", "materialisation"]));
}

/// The `contract` header line a rendered block states for each condition,
/// keyed by the token that line opens with.
///
/// Structural rather than a substring search over the whole block, and that is
/// what the assertions below rest on: a test that looked for injected text
/// *anywhere* would pass on a field attached to the wrong row.
fn contract_lines(lines: &[String]) -> BTreeMap<&str, &str> {
    lines
        .iter()
        .filter_map(|line| line.strip_prefix("contract "))
        .filter_map(|rest| Some((rest.split_whitespace().next()?, rest)))
        .collect()
}

/// The condition tokens a rendered block names — [`contract_lines`]'s keys, so
/// the two cannot disagree about what a block carries.
fn contract_tokens(lines: &[String]) -> BTreeSet<&str> {
    contract_lines(lines).into_keys().collect()
}

/// The discharge each condition in a rendered block is given, keyed by the
/// token of the `contract` line it follows.
fn discharges(lines: &[String]) -> BTreeMap<&str, &str> {
    let mut out = BTreeMap::new();
    let mut current: Option<&str> = None;
    for line in lines {
        if let Some(rest) = line.strip_prefix("contract ") {
            current = rest.split_whitespace().next();
        } else if let Some(text) = line.strip_prefix("  discharge: ") {
            out.insert(
                current.expect("a discharge follows its contract line"),
                text,
            );
        }
    }
    out
}

/// The remedy is the contract's (`VT-2`, design `sec-6`, `sec-3` invariant 4).
///
/// Quantified over every row rather than sampled, and
/// `review-disposition-attested` is why the quantifier matters: it is the only
/// multi-line discharge, so a test over the easy eight would pass while the row
/// with two doors rendered one of them.
#[test]
fn remedy_equals_discharge_for_every_row() {
    let bodies = BTreeMap::new();
    let blocks: Vec<Vec<String>> = Advance::ALL
        .into_iter()
        .map(|edge| contract_block(edge, &bodies))
        .collect();

    for (condition, contract) in CONTRACTS {
        let mut enforced_by = 0usize;
        for block in &blocks {
            let stated = discharges(block);
            let Some(text) = stated.get(condition.as_str()) else {
                continue;
            };
            // The refusal's remedy and the receipt's discharge are one value
            // formatted twice, so a row whose remedy grew an arm grows it in
            // both channels or this fails.
            assert_eq!(*text, contract.remedy(), "{}", condition.as_str());
            enforced_by += 1;
        }
        assert!(
            enforced_by > 0,
            "{} is enforced by some edge and therefore rides some receipt",
            condition.as_str()
        );
    }

    let top = contract_block(Advance::ReviewingLocked, &bodies);
    let disposition = discharges(&top)["review-disposition-attested"];
    assert_eq!(disposition.lines().count(), 3, "{disposition}");
    assert!(disposition.contains("conducted:"), "{disposition}");
    assert!(disposition.contains("waived:"), "{disposition}");
    assert!(
        disposition
            .starts_with("the user disposes this review pass (you record it on their assent):"),
        "{disposition}"
    );
}

/// The receipt covers what the edge judges by, not the edge's own rows alone
/// (`VT-3`, DEC-124 read wide).
#[test]
fn receipt_covers_the_enforced_set() {
    let bodies = BTreeMap::new();
    for edge in Advance::ALL {
        let block = contract_block(edge, &bodies);
        let enforced: BTreeSet<&str> = cumulative_conditions(edge.to())
            .iter()
            .map(|condition| condition.as_str())
            .collect();
        // Set equality against the function the gate itself calls — not a
        // count, which would pass on the right number of wrong rows.
        assert_eq!(contract_tokens(&block), enforced, "{edge:?}");
    }

    // Equality alone would still hold if the enforced set were the edge's own
    // boundary, so the inherited majority is asserted too: it is the whole
    // reason the receipt takes the wider of DEC-124's two readings.
    for (edge, total, inherited) in [
        (Advance::ReviewingLocked, 8, 5),
        (Advance::DraftingReviewing, 6, 4),
    ] {
        let block = contract_block(edge, &bodies);
        let rendered = contract_tokens(&block);
        let own: BTreeSet<&str> = boundary_conditions(edge)
            .iter()
            .map(|condition| condition.as_str())
            .collect();
        assert_eq!(rendered.len(), total, "{edge:?}");
        assert_eq!(rendered.difference(&own).count(), inherited, "{edge:?}");
    }
}

/// The bottom edge's receipt is its own boundary and nothing more — and both
/// its rows reappear above, which is what an accumulating receipt means
/// (`VT-4`).
#[test]
fn bottom_edge_receipt_is_its_own_boundary() {
    let bodies = BTreeMap::new();
    let bottom = contract_block(Advance::ExploringInquiring, &bodies);
    assert_eq!(
        contract_tokens(&bottom),
        BTreeSet::from(["governing-context-recorded", "initial-concerns-recorded"])
    );

    let top = contract_block(Advance::ReviewingLocked, &bodies);
    assert!(
        contract_tokens(&bottom).is_subset(&contract_tokens(&top)),
        "the bottom edge's conditions ride every receipt above it"
    );
}

/// The block injects every structural field, so the prose may carry none of
/// them (`EX-3`, DEC-123).
#[test]
fn the_block_injects_what_the_prose_may_not_restate() {
    let bodies = BTreeMap::new();
    let bottom = contract_block(Advance::ExploringInquiring, &bodies);
    assert_eq!(
        bottom.first().map(String::as_str),
        Some("contracts exploring-inquiring")
    );

    // An attested row: kind, coverage, the observed conjunct, reach. The
    // observed set is the entire live half of this row — its `Artefact`
    // coverage is inert by construction — so a header naming only the act
    // would tell a reader the wrong thing about what keeps it current.
    assert_eq!(
        contract_lines(&bottom)["governing-context-recorded"],
        "governing-context-recorded attested artefact observes(governance-edges) cumulative"
    );

    // A derived row names the run-owned state it is recomputed from, and has
    // no coverage to name.
    let middle = contract_block(Advance::InquiringDrafting, &bodies);
    assert_eq!(
        contract_lines(&middle)["blocking-inquiries-dispositioned"],
        "blocking-inquiries-dispositioned derived engine(dispositions) cumulative"
    );

    // Reach is injected because it is what says whether discharging once is
    // the end of it — the one row that is not cumulative says so.
    let upper = contract_block(Advance::DraftingReviewing, &bodies);
    assert_eq!(
        contract_lines(&upper)["drafting-readiness-attested"],
        "drafting-readiness-attested attested artefact edge-local"
    );

    // `observes(…)` renders exactly where a rule names facts and nowhere else,
    // asserted against `CONTRACTS` rather than as a literal, so a second member
    // joins the render for free.
    let rendered: BTreeMap<&str, &str> = [&bottom, &middle, &upper]
        .into_iter()
        .flat_map(|block| contract_lines(block))
        .collect();
    for (condition, contract) in CONTRACTS {
        let Some(line) = rendered.get(condition.as_str()) else {
            continue;
        };
        let observed = match contract.derivation {
            DerivationRule::Attested(rule) => !rule.binding.observed.is_empty(),
            DerivationRule::Engine(_) => false,
        };
        assert_eq!(line.contains("observes("), observed, "{line}");
    }
}

/// A declared receipt elides the body and only the body (`EX-5`'s pure half).
///
/// The rule [`fragment_section`] follows, for the reason its doc gives: a
/// caller that declared a stale receipt, or lost the bytes it claimed, must
/// still be able to tell what it is missing.
///
/// [`fragment_section`]: crate::commands::design
#[test]
fn a_declared_block_keeps_every_header_and_places_no_body() {
    let edge = Advance::DraftingReviewing;
    let enforced = cumulative_conditions(edge.to());
    let held = contract_block(edge, &BTreeMap::new());
    let bodies: BTreeMap<Condition, String> = enforced
        .iter()
        .map(|condition| (*condition, format!("narrative for {}", condition.as_str())))
        .collect();
    let delivered = contract_block(edge, &bodies);

    assert_eq!(contract_lines(&held), contract_lines(&delivered));
    assert_eq!(discharges(&held), discharges(&delivered));
    assert_eq!(
        delivered.len(),
        held.len() + enforced.len(),
        "one body per contract, and nothing else moved"
    );
    for condition in &enforced {
        let body = format!("narrative for {}", condition.as_str());
        assert!(delivered.contains(&body), "{body} is placed");
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

/// Correspondence row 3: a carried confirmation is refused where no rule names
/// one — and `SL-264` sec-3 leaves every rule naming none.
///
/// The link `DEC-121` drew — *the agent declares, the user confirms* — left the
/// rule with the `blocking-set-declared` act, and its digest is now read only as
/// a frozen legacy read ([`Cause::ConfirmationStale`]). So the one direction left
/// here is the refusal: a newly recorded act may not carry a confirming digest,
/// which is what keeps the frozen read from ever *creating* a confirmation.
#[test]
fn a_confirmation_is_refused_where_no_rule_names_one() {
    let mut unconfirmed = checkpoint_act("cpa-1", ActKind::GraphReviewed, "steered the graph");
    unconfirmed.covered = Some(covered_nodes());
    assert!(
        admit_act(
            RecordedAct::Checkpoint(&unconfirmed),
            rule_for(ActKind::GraphReviewed),
            None
        )
        .is_ok(),
        "a fresh `graph-reviewed` carries no confirmation and corresponds exactly"
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

/// `requirement_for` is a **total** function over the non-legacy vocabulary, and
/// this is what says so.
///
/// The signature returns `Option` because a search over data cannot be total to
/// the type system. What makes the `None` arm unreachable for a non-legacy act is
/// this: every **non-legacy** act in the closed vocabulary is named by exactly one
/// contract row, and every **legacy** kind ([`ActKind::is_legacy`], `SL-264`
/// sec-3) by none — it stays readable with no rule. An act named by none would be
/// an unrequireable act; one named by two would make *which rule* ambiguous with
/// nothing to break the tie, and `requirement_for` would silently answer with
/// whichever row came first.
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
        if act.is_legacy() {
            assert!(
                naming.is_empty(),
                "legacy `{}` is named by no contract row, and this one names it from {naming:?}",
                act.as_str()
            );
            assert_eq!(
                requirement_for(act),
                None,
                "legacy `{}` resolves to no rule",
                act.as_str()
            );
        } else {
            assert_eq!(naming.len(), 1, "`{}` is named by {naming:?}", act.as_str());
            assert_eq!(
                requirement_for(act).map(|rule| rule.required.act),
                Some(act),
                "`{}` resolves to its own requirement",
                act.as_str()
            );
        }
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
    satisfied(condition, run, &derived.gate).expect_err("the condition must not hold here")
}

/// `condition` holds against this state.
fn assert_holds(condition: Condition, run: &DesignSnapshot, derived: &DerivedInput) {
    assert_eq!(
        satisfied(condition, run, &derived.gate),
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

/// `VT-1` — a lost confirmation says which declaration went missing.
///
/// DEC-121 made `initial-concerns-recorded` two acts by two actors precisely so a
/// refusal can name the missing one. `SL-264` sec-3 retires the agent's act from
/// the rule, but the user's review still carries the digest of the declaration it
/// confirmed: with the declaration gone, that carried digest matches nothing and
/// the one cause is the stale confirmation, naming `BlockingSetDeclared`.
#[test]
fn missing_conjunct_names_the_missing_act() {
    let (mut run, derived) = cleared();
    run.declarations
        .declarations
        .retain(|held| held.act.kind() != AgentActKind::BlockingSetDeclared);

    assert_eq!(
        causes_of(Condition::InitialConcernsRecorded, &run, &derived),
        vec![Cause::ConfirmationStale {
            act: ActKind::GraphReviewed,
            declaration: AgentActKind::BlockingSetDeclared,
        }]
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
            Some(false),
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
            Some(false),
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
        &derived.gate,
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

/// `VT-1` — the forward look and the gate are one evaluation (DEC-292).
///
/// For every forward edge, `forward_unmet` is exactly the `unmet` inside
/// `advance`'s `GateNotCleared`, and empty precisely when `advance` passes its
/// condition leg. The runbook standing is cleared so the only leg under test is
/// the conditions — the leg the two share.
///
/// Two runs: one holding everything, and the same run stripped of the two acts
/// the bottom edge asks for, so the comparison covers a met and an unmet set
/// rather than one repeated four times.
#[test]
fn forward_unmet_agrees_with_advance() {
    let (full, derived) = cleared();
    let mut partial = full.clone();
    partial.acts.acts.retain(|held| {
        !matches!(
            held.act,
            ActKind::GovernanceConfirmed | ActKind::GraphReviewed
        )
    });
    let standing = RunbookStanding::default();
    assert!(standing.cleared(), "no outstanding required steps");

    for (run, label) in [
        (&full, "every condition holds"),
        (&partial, "two conditions are unmet"),
    ] {
        for from in Stage::ALL {
            let Some(edge) = Advance::from_stage(from) else {
                continue;
            };
            let looked_ahead = forward_unmet(edge.to(), run, &derived.gate);
            match advance(from, edge.to(), run, &derived.gate, Some(&standing)) {
                Ok(_) => assert!(
                    looked_ahead.is_empty(),
                    "{edge:?} ({label}): advance passed, so nothing may block the look"
                ),
                Err(Refusal::GateNotCleared { unmet, .. }) => assert_eq!(
                    looked_ahead, unmet,
                    "{edge:?} ({label}): the look and the refusal are one evaluation"
                ),
                Err(other) => panic!("{edge:?} ({label}): unexpected refusal {other:?}"),
            }
        }
    }
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
            &derived.gate,
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
            &derived.gate,
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
    derived.gate.observed_review = Some(ObservedReview {
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

/// `VT-9` / `VA-3` — a blocker held the edge, and disposing it clears again.
///
/// PHASE-04 `D7` settled that this clause lands live here rather than deferring
/// to `IMP-392`: the blocker predicate is buildable today and only the
/// *concluded* marker waits. It is the `Conducted` arm's one positive case —
/// every other test of that arm approaches it from a failure — and the axis is
/// before/after on one run, where
/// `waiver_clears_over_live_findings_and_dismisses_none`'s is arm-against-arm
/// over one observation. The shared first leg is this test's control: without it
/// a derivation that never looked at the ledger would pass the second.
///
/// What it pins is that finding state is **not monotone** (design `sec-3`) —
/// `contest` reopens what `dispose` closed, so the row must re-derive from the
/// ledger on every crossing rather than latch the first answer it liked.
#[test]
fn a_re_disposed_blocker_clears_the_edge_again() {
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
    let observing = |undisposed_blockers: Vec<String>| {
        Some(ObservedReview {
            reference: ReviewRef::new(PASS),
            concluded: true,
            undisposed_blockers,
        })
    };

    derived.gate.observed_review = observing(vec!["F-2".to_owned()]);
    assert_eq!(
        causes_of(Condition::ReviewDispositionAttested, &run, &derived),
        vec![Cause::BlockersUndisposed {
            findings: vec!["F-2".to_owned()],
        }],
        "a contested blocker holds the edge"
    );

    // The responder disposes it. Nothing on the run moved — the same stored act,
    // the same pass — so a row that cleared here on anything but the ledger
    // would be reading its own history.
    derived.gate.observed_review = observing(Vec::new());
    assert_holds(Condition::ReviewDispositionAttested, &run, &derived);
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
    derived.gate.observed_review = None;

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
            Some(false),
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
        vec![Cause::CoverageStale {
            act: ActKind::GraphReviewed,
            moved: vec![open],
        }],
        "the review was given over the map, so it lost its coverage — and the \
         legacy declaration, named by no rule, is not a conjunct any more"
    );
    assert!(
        !causes
            .iter()
            .any(|cause| matches!(*cause, Cause::ConfirmationStale { .. })),
        "and `confirms` still matches, because nothing about the claim moved"
    );
}

// ---------------------------------------------------------------------------
// SL-264 PHASE-03 — legacy act kinds are a named, read-only class
// (design sec-3; `RV-386` F-7, F-16, F-17).
// ---------------------------------------------------------------------------

/// `VT-1` — a submitted legacy act is refused and the stored set is left
/// standing.
///
/// The class lands before the label leaves the wire (`RV-386` F-16): without the
/// refusal a submitted `blocking-set-declared` replaces the stored legacy set
/// and moves the effective judgement of every node that holds none of its own.
/// The refusal names the kind, so the caller learns which act is retired rather
/// than only that something was.
#[test]
fn submitted_legacy_act_is_refused_and_the_stored_set_unchanged() {
    let (run, _derived) = cleared();
    let before = run.declarations.declarations.clone();

    let refused = apply(
        &run,
        &ApplyRequest {
            agent_declaration: Some(AgentActDeclaration {
                act: AgentAct::BlockingSetDeclared {
                    blocking: BTreeSet::from([id(OPEN_NODE)]),
                },
                basis: "a second look says this one blocks".to_owned(),
                turn: None,
            }),
            ..ApplyRequest::bare(SubmissionEnvelope {
                run_uid: run.run.uid.clone(),
                known_revision: run.run.revision,
                submission_id: "s1".to_owned(),
            })
        },
        &Crossing::Ordinary,
        &DerivedInput {
            declaration_fingerprint: Some(Fingerprint::new("sha256:claimed")),
            ..DerivedInput::default()
        },
        "sha256:pay",
        &Resolution::default(),
    );

    assert_eq!(
        refused.err(),
        Some(Refusal::RetiredAct {
            kind: ActKind::BlockingSetDeclared,
        }),
        "a legacy act is retired from writing, and the refusal names it"
    );
    assert_eq!(
        run.declarations.declarations, before,
        "the stored legacy set is unchanged"
    );
}

/// `VT-2` — a map edit emits no row for the legacy act.
///
/// `live_acts` excludes legacy kinds by stated policy, in the before set and the
/// after set alike, so the set difference is empty and nothing reports an act
/// dead whose currency nothing reads (`RV-386` F-17, `STD-003`). The control is
/// the user's `graph-reviewed`, which IS currency-read and does die when the map
/// moves: the exclusion discriminates by kind, not by coverage.
#[test]
fn a_map_edit_emits_no_row_for_a_legacy_act() {
    let (run, _derived) = cleared();
    let edit = declare_over(
        &run,
        r#"{"subject": "inq-3", "question": "what did the review not see?", "blocking": false}"#,
    )
    .expect("a new node with a judgement applies");

    let invalidated = subjects(&edit, ChangeEvent::ActInvalidated);
    assert!(
        !invalidated.contains(&id("agd-blocking")),
        "no row for the legacy declaration: {invalidated:?}"
    );
    assert!(
        invalidated.contains(&id("cpa-graph")),
        "the control: the user's map-bound review does die: {invalidated:?}"
    );
}

/// `VT-4` — a stored `ConfirmationStale` verdict survives the retirement.
///
/// The `confirms` link left the rule with the legacy act, but not the read: a
/// stored `graph-reviewed` carries the digest of the declaration it confirmed,
/// and when that no longer names the stored declaration the act reads stale. The
/// comparison is carried-driven and **frozen at upgrade** — with no declaration
/// writable, nothing can create a fresh verdict — so the very write that is
/// refused leaves it standing.
#[test]
fn a_stored_confirmation_stale_verdict_survives_the_retirement() {
    let (mut run, derived) = cleared();
    // A stored snapshot in which the agent changed the set after the user's
    // review: the carried digest no longer names the stored declaration.
    let mut relisted = blocking_set_declared("agd-blocking", &[BLOCKING_NODE, OPEN_NODE]);
    relisted.covered = Some(CoveredSet::Nodes(ContentCoverage::of(
        run.map.inquiry.materials(),
    )));
    relisted.fingerprint = Fingerprint::new("sha256:agd-blocking-relisted");
    run.declarations.record(relisted);

    let stale = || causes_of(Condition::InitialConcernsRecorded, &run, &derived);
    assert_eq!(
        stale(),
        vec![Cause::ConfirmationStale {
            act: ActKind::GraphReviewed,
            declaration: AgentActKind::BlockingSetDeclared,
        }],
        "the carried digest no longer names the stored declaration"
    );

    // The control: the same run with the digest naming the stored declaration
    // holds, so the verdict above is the carried comparison and not a rule that
    // fails every review now that its `confirms` slot retired.
    let (matched, matched_derived) = cleared();
    assert_holds(
        Condition::InitialConcernsRecorded,
        &matched,
        &matched_derived,
    );
}

/// `VT-3` — the contract admits legacy enum variants without advertising them.
///
/// `SL-264` sec-3 retires the `blocking-set-declared` token from the published
/// contract: neither `ActKind` nor `AgentAct` lists it any more, so `design
/// contract` no longer advertises a writable act. But the Rust variant stays —
/// `AgentAct` derives `Deserialize` and the snapshot is parsed whole — so a
/// payload carrying the token still parses, and the key walk **admits** it to the
/// core, which refuses it as [`Refusal::RetiredAct`], rather than refusing it as a
/// mistyped key and naming the wrong fault.
#[test]
fn the_payload_contract_admits_legacy_enum_variants() {
    for contract in [&ACT_KIND, &AGENT_ACT] {
        let TypeForm::Enum { variants, .. } = contract.form else {
            panic!("{} is described as an enum", contract.name);
        };
        assert!(
            !variants
                .iter()
                .any(|variant| variant.token == Some("blocking-set-declared")),
            "{} no longer advertises the retired act",
            contract.name
        );
    }

    let legacy: AgentActDeclaration = serde_json::from_value(serde_json::json!({
        "act": {"blocking-set-declared": {"blocking": ["inq-1"]}},
        "basis": "these block drafting",
    }))
    .expect("the variant stays so a stored snapshot parses");
    assert_eq!(legacy.act.kind(), AgentActKind::BlockingSetDeclared);

    let request = ApplyRequest {
        agent_declaration: Some(legacy),
        ..ApplyRequest::bare(SubmissionEnvelope {
            run_uid: "dr-test".to_owned(),
            known_revision: 1,
            submission_id: "s1".to_owned(),
        })
    };
    let wire = serde_json::to_value(&request).expect("a payload serialises");
    assert!(
        refuse_unknown_keys(&wire).is_ok(),
        "the legacy token reaches the core rather than being refused as a key"
    );
}

// ── the inert-key refusal (SL-249 PHASE-02, ISS-318) ───────────────────────

/// `I9` — the wire-key table is total (`VT-1`, `EX-3`).
///
/// The oracle is a **serialised** declaration, not a hand-written list: a field
/// that reaches the wire without a table row fails here, and a table row naming a
/// key no field emits fails equally. `resolved_record` is absent from both sides
/// by construction rather than by an exception — it carries `#[serde(skip)]`, so
/// it is not a wire key at all.
#[test]
fn the_wire_key_table_holds_exactly_a_populated_declarations_serde_keys() {
    let populated = Declaration::fully_populated(id("inq-1"));
    let serialised = serde_json::to_value(&populated).expect("a declaration serialises");

    let on_the_wire: BTreeSet<&str> = serialised
        .as_object()
        .expect("a declaration serialises to an object")
        .keys()
        .map(String::as_str)
        .collect();
    let tabled: BTreeSet<&str> = Declaration::WIRE_KEYS
        .iter()
        .map(|(key, ..)| *key)
        .collect();

    assert_eq!(on_the_wire, tabled);
}

/// Each row's presence predicate agrees with the `skip_serializing_if` that
/// decides whether its key reaches the wire.
///
/// `I9` compares key *sets* and so cannot see a row wired to the wrong field's
/// predicate. Without this, a predicate reading `attests` under the `concerns`
/// row would leave the table looking total while the refusal fired on the wrong
/// key — the mapping-versus-inventory distinction `RV-349` `F-4` drew, one field
/// along.
#[test]
fn every_rows_predicate_agrees_with_its_keys_presence_on_the_wire() {
    for (key, _, _, carried) in &Declaration::WIRE_KEYS {
        assert!(
            carried(&Declaration::fully_populated(id("inq-1"))),
            "`{key}`'s predicate must see it on a fully populated declaration"
        );
        assert_eq!(
            carried(&Declaration::about(id("inq-1"))),
            *key == "subject",
            "`{key}`'s predicate must see it on a bare declaration only if it is \
             the addressing key"
        );
    }
}

/// The `SL-248` loss, at the unit that refuses it (`EX-1`, `EX-2`, `EX-5`).
#[test]
fn a_checkpoint_subject_carrying_section_prose_is_refused_naming_the_key_it_wanted() {
    let refused = Batch::of(vec![declared(
        r###"{"subject": "cp-4", "body": "## The two candidate sites\n"}"###,
    )])
    // A `cp-` subject has no held state at all (`subject_state`), so the state
    // axis cannot fire here whatever this answers — the refusal under test is
    // the kind axis's, and it runs first regardless.
    .validate(|_| SubjectState::Absent)
    .expect_err("`body` is section prose and means nothing on a checkpoint");

    assert_eq!(
        refused,
        Refusal::InertKey {
            subject: id("cp-4"),
            key: "body",
            honoured_by: vec![IdKind::Section],
            remedy: Some("dispose.create.body"),
        }
    );
}

/// A subject that may not be declared at all keeps its own refusal.
///
/// The per-key check yields to it deliberately: `SubjectNotDeclarable` names the
/// run-level field to use instead, and an inert-key refusal on the same payload
/// would trade that remedy for a symptom.
#[test]
fn a_non_declarable_subject_is_refused_as_such_rather_than_key_by_key() {
    let declaration = declared(r#"{"subject": "dlg-1", "body": "prose"}"#);

    assert_eq!(declaration.inert_key(), None);
}

/// `I10`'s subject fixture ids, one per kind, plus the two companions every
/// other kind's fixture points at.
///
/// The subject ids are **not** seeded into the fixture universe and the
/// companions are: a key like `provenance` is read only where a node is being
/// created, so the subject must be absent for the differential to see it.
const COMPANION_NODE: &str = "inq-2";
const COMPANION_SECTION: &str = "sec-2";

/// The run every cell is measured against, and the shell-derived facts it needs.
///
/// Holds the two companions and, under [`SubjectState::Held`], the subject of
/// the kind under test. `section_digests` carries the `sec-` subject because a
/// section body is digested by the shell and the pure layer never hashes.
///
/// `SubjectState` is the production enum (`ids.rs`), not a test-local copy: it is
/// the same vocabulary the wire-key table's state column is written in, and two
/// spellings of one concept is how a fixture drifts from the rule it measures.
fn universe(state: SubjectState, kind: IdKind) -> (DesignSnapshot, DerivedInput) {
    let mut snapshot = run_holding(&[(COMPANION_SECTION, "digest-companion")]);
    snapshot
        .map
        .inquiry
        .insert(InquiryNode::open(
            id(COMPANION_NODE),
            "a companion",
            Provenance::AgentProposed,
            Some(false),
        ))
        .expect("the companion node seats");
    if state == SubjectState::Held {
        seat_subject(&mut snapshot, kind);
    }
    let derived = DerivedInput {
        section_digests: BTreeMap::from([(id("sec-1"), Fingerprint::new("digest-subject"))]),
        ..DerivedInput::default()
    };
    (snapshot, derived)
}

/// Seat the subject `kind` addresses, for a [`SubjectState::Held`] universe.
///
/// Covers the two kinds that carry a state-axis row and **panics** for the rest,
/// on [`wire_value`]'s reasoning: a row added at a kind with no held fixture must
/// fail the matrix rather than quietly narrow it. `subject_state` reports the
/// other kinds `Absent` unconditionally, so a cell there would assert nothing.
fn seat_subject(snapshot: &mut DesignSnapshot, kind: IdKind) {
    match kind {
        IdKind::Inquiry => {
            snapshot
                .map
                .inquiry
                .insert(InquiryNode::open(
                    id("inq-1"),
                    "the subject",
                    Provenance::AgentProposed,
                    Some(false),
                ))
                .expect("the held subject seats");
        }
        IdKind::Finding => snapshot.review.findings.push(Finding {
            id: id("fnd-1"),
            subject: id(COMPANION_SECTION),
            summary: "the held finding".to_owned(),
            blocking: false,
            resolution: None,
        }),
        other => panic!(
            "no held fixture for a `{}` subject — a state-axis row at that kind needs one",
            other.prefix()
        ),
    }
}

/// A JSON value for `key`, chosen to differ from whatever the engine defaults to
/// when the key is absent — otherwise the differential cannot see it.
///
/// `None` for a key with no fixture, which **fails** the matrix rather than
/// silently narrowing it: a wire key added without a value here shows up as an
/// uncovered cell, not as a cell nobody ran.
fn wire_value(key: &str) -> Option<&'static str> {
    Some(match key {
        "question" => r#""why?""#,
        "needs" => r#"["inq-2"]"#,
        "parent" => r#""inq-2""#,
        // Internally tagged, and deliberately not the `agent-proposed` default:
        // a key whose value equals the default is invisible to a differential.
        "provenance" => r#"{"provenance": "user-directed"}"#,
        "lifecycle" => r#""deferred""#,
        "body" => r###""## a section\n""###,
        "attests" => r#""sec-2""#,
        // Likewise not `human`, which is what an absent `reviewer` means.
        "reviewer" => r#""adversarial""#,
        "concerns" => r#""sec-2""#,
        "summary" => r#""a finding""#,
        "blocking" => "true",
        "resolution" => r#""disposed""#,
        "disposes" => r#""inq-2""#,
        "dispose" => r#"{"form": "unresolved", "note": "retained"}"#,
        _ => return None,
    })
}

/// The keys a declaration at `kind` needs before the engine will apply it at all.
///
/// Not a second copy of the correspondence: these are the *required* keys, and
/// the base for a cell is this set **minus the key under test** — which is how a
/// required key's own cell gets a base that is refused, so that supplying the key
/// is observable as the difference between a refusal and an application.
///
/// `blocking` joined the inquiry row when `SL-264` made a judgement an obligation
/// at creation. Every cell at that kind now needs it in the base, or the cell
/// would measure the missing judgement instead of the key under test — see
/// [`no_wire_key_is_accepted_and_ignored_at_any_subject_kind`], where a base
/// refused for *this* reason would let an honoured key read as refused rather than
/// as effectful.
fn companions(kind: IdKind) -> &'static [&'static str] {
    match kind {
        IdKind::Inquiry => &["blocking"],
        IdKind::Section => &["body"],
        IdKind::Attestation => &["attests"],
        IdKind::Finding => &["summary", "concerns"],
        IdKind::Checkpoint => &["disposes", "dispose"],
        IdKind::Delegation | IdKind::CheckpointAct | IdKind::AgentDeclaration => &[],
    }
}

/// One declaration at `kind` carrying `keys`.
fn declaration_at(kind: IdKind, keys: &[&str]) -> Declaration {
    let mut json = format!(r#"{{"subject": "{}1""#, kind.prefix());
    for key in keys {
        let value = wire_value(key).unwrap_or_else(|| panic!("`{key}` has no value fixture"));
        json.push_str(&format!(r#", "{key}": {value}"#));
    }
    json.push('}');
    declared(&json)
}

/// What the declaration engine did — the resulting run and how it answered.
///
/// Measured on [`declare`], which does not consult the wire-key table: this is
/// the *behavioural* half of `I10`, and reading it through the admission path
/// would let the table decide its own verdict.
fn engine_outcome(
    state: SubjectState,
    kind: IdKind,
    keys: &[&str],
) -> (DesignSnapshot, Option<Refusal>) {
    let (mut next, derived) = universe(state, kind);
    let refused = declare(&mut next, &declaration_at(kind, keys), &derived, "sub-1").err();
    (next, refused)
}

/// Whether the whole admission path refuses this declaration — the wire-key
/// check first, then the engine's own arms.
fn admission_refuses(state: SubjectState, kind: IdKind, keys: &[&str]) -> bool {
    let (mut next, derived) = universe(state, kind);
    let declaration = declaration_at(kind, keys);
    // The PRODUCTION resolver, never a `|_| Absent` stand-in: a constant here
    // would make every state-axis cell vacuous while leaving the matrix green.
    match Batch::of(vec![declaration]).validate(|subject| subject_state(&next, subject)) {
        Err(_) => true,
        Ok(candidate) => candidate
            .values()
            .any(|declaration| declare(&mut next, declaration, &derived, "sub-1").is_err()),
    }
}

/// `I10` — no wire key is accepted and ignored (`VT-2`, `EX-4`, `VA-1`).
///
/// # What a cell asserts
///
/// For every (wire key × subject kind) pair, **exactly one** of two behavioural
/// facts holds:
///
/// - *effectful* — a base declaration at that kind and the same declaration
///   plus the key yield different outcomes under [`declare`]: a different run,
///   or one applying where the other is refused;
/// - *refused* — the whole admission path refuses the declaration carrying it.
///
/// Both false is the silent acceptance `I10` forbids — the `ISS-318` defect.
/// Both true is a table refusing a key its own engine honours, which is `F-4`'s
/// hazard: `I9` compares two key sets and stays green if `body` is mapped to
/// `cp-` and `dispose` to `sec-`, because the sets are identical either way.
/// Here that swap fails twice — `body` at `sec-` becomes effectful *and*
/// refused, and `body` at `cp-` becomes neither.
///
/// The effectful side is measured one layer below the check, on [`declare`],
/// which never reads the table. That is what makes the oracle behaviour rather
/// than the table under test.
///
/// # The generator
///
/// Both vocabularies are read from their own source — [`Declaration::WIRE_KEYS`]
/// and [`IdKind::ALL`] — so adding a wire key or an id kind widens the covered
/// set with no edit here, and narrowing coverage means deleting a loop rather
/// than deleting rows nobody misses (`R10`). A key added without a
/// [`wire_value`] fixture fails; it does not quietly skip.
///
/// # The subject state this is quantified at
///
/// [`SubjectState::Absent`], for every key. `I10` is the **kind**-axis matrix,
/// and the create path is where every key is observable at its honouring kind —
/// so one state suffices and the cell stays a question about the kind.
///
/// It did not always. A per-key `subject_state` fixture held `lifecycle` at
/// [`SubjectState::Held`], because the create branch dropped it on the floor and
/// an absent subject would have shown it inert. `PHASE-01` collapsed
/// `declare_node` onto one row-producing path over two priors, so `lifecycle` is
/// now honoured at both states and the fixture had nothing left to do.
///
/// The keys that ARE read on only one path — `provenance`, `concerns`,
/// `blocking` — are the state axis (`DEC-246`, `ISS-327`), and their other state
/// is pinned by `a_key_inert_at_the_subjects_state_is_refused` below rather than
/// here. `DEC-183` drew that line and it still holds: a cell here asks whether
/// *some* submission at this kind makes the key effectful.
///
/// # The addressing key
///
/// A key carried by *every* declaration is outside the frame: there is no
/// submission that omits it, so "carrying it" distinguishes nothing. Detected by
/// the row's own predicate holding on a bare declaration — a behavioural test,
/// not a name on a list, and
/// `every_rows_predicate_agrees_with_its_keys_presence_on_the_wire` pins that it
/// selects the addressing key alone.
#[test]
fn no_wire_key_is_accepted_and_ignored_at_any_subject_kind() {
    let bare = Declaration::about(id("inq-1"));
    for (key, _, _, carried) in &Declaration::WIRE_KEYS {
        if carried(&bare) {
            continue;
        }
        for kind in IdKind::ALL {
            let state = SubjectState::Absent;
            let base: Vec<&str> = companions(kind)
                .iter()
                .copied()
                .filter(|companion| companion != key)
                .collect();
            let mut carrying = base.clone();
            carrying.push(key);

            let effectful =
                engine_outcome(state, kind, &base) != engine_outcome(state, kind, &carrying);
            let refused = admission_refuses(state, kind, &carrying);

            assert_ne!(
                effectful,
                refused,
                "`{key}` at a `{}` subject is {}",
                kind.prefix(),
                if effectful {
                    "both effectful and refused — the table maps it to a kind its engine does not"
                } else {
                    "accepted and ignored — neither effectful nor refused (ISS-318)"
                }
            );
        }
    }
}

// ── the state-axis refusal (SL-259 PHASE-04, ISS-327) ──────────────────────

/// Every state-axis row, as `(key, home kind, the state that honours it)`.
///
/// Read off [`Declaration::WIRE_KEYS`] rather than listed here, on `I9`'s
/// reasoning: a hand-written list is a third spelling, free to agree with
/// neither the table nor the engine. Empty would silently pass every test
/// below, so the callers assert the count.
fn state_axis_rows() -> Vec<(&'static str, IdKind, SubjectState)> {
    Declaration::WIRE_KEYS
        .iter()
        .filter_map(|&(key, home, when, _)| match (home, when) {
            (KeyHome::At(kind), KeyWhen::Only(honoured_when)) => Some((key, kind, honoured_when)),
            (KeyHome::Universal | KeyHome::At(_), KeyWhen::EitherState | KeyWhen::Only(_)) => None,
        })
        .collect()
}

/// The keys a cell carries beneath the one under test.
///
/// [`companions`] are what the CREATE path requires, so at a state where some of
/// them are themselves inert they must be dropped — otherwise the first refusal
/// names a companion and the cell under test is never reached. Filtering by the
/// state rather than by a hardcoded list keeps this correct in both directions,
/// including for a future row honoured only where the subject is held.
fn base_at(kind: IdKind, state: SubjectState, under_test: &str) -> Vec<&'static str> {
    let inert: Vec<&str> = state_axis_rows()
        .into_iter()
        .filter(|&(_, _, honoured_when)| honoured_when != state)
        .map(|(key, ..)| key)
        .collect();
    companions(kind)
        .iter()
        .copied()
        .filter(|companion| *companion != under_test && !inert.contains(companion))
        .collect()
}

/// The state a key is NOT honoured in. Two states, so one names the other.
const fn other_than(state: SubjectState) -> SubjectState {
    match state {
        SubjectState::Absent => SubjectState::Held,
        SubjectState::Held => SubjectState::Absent,
    }
}

/// `VT-1` — a key inert at its subject's STATE is refused, naming the key
/// (`EX-1`, `DEC-246`, `ISS-327`).
///
/// # The two halves, and why neither alone would do
///
/// Each cell asserts both:
///
/// - the key is **not effectful** at this state, measured on [`declare`], which
///   never reads the wire-key table — that is the defect `ISS-327` reported, and
///   measuring it a layer below the table is what stops this test being the
///   table's own oracle (`I10`'s rule, one axis along);
/// - admission **refuses** it, with the REASON pinned as
///   [`Refusal::InertAtState`] naming that key. A refusal that fires for the
///   wrong reason is a test passing for the wrong reason (`sec-8` leg 3).
///
/// Dropping the first half would leave a test that passes over a key the engine
/// honours — a refusal widened into a ban, which is what
/// `the_same_keys_are_honoured_at_the_state_that_honours_them` guards from the
/// other side.
#[test]
fn a_key_inert_at_the_subjects_state_is_refused() {
    let rows = state_axis_rows();
    assert_eq!(
        rows.len(),
        3,
        "three cells survive `DEC-246`'s four — `PHASE-01` made `lifecycle` \
         honoured at creation too; a change in this count is a change in the rule"
    );

    for (key, kind, honoured_when) in rows {
        let inert_state = other_than(honoured_when);
        let base = base_at(kind, inert_state, key);
        let mut carrying = base.clone();
        carrying.push(key);

        assert_eq!(
            engine_outcome(inert_state, kind, &base),
            engine_outcome(inert_state, kind, &carrying),
            "`{key}` at a `{}` subject the run holds must be inert — if the engine \
             now honours it, the row is stale, not the refusal",
            kind.prefix()
        );

        let (mut next, derived) = universe(inert_state, kind);
        let refused = Batch::of(vec![declaration_at(kind, &carrying)])
            .validate(|subject| subject_state(&next, subject))
            .expect_err("a key inert at the subject's state is refused");
        assert_eq!(
            refused,
            Refusal::InertAtState {
                subject: id(&format!("{}1", kind.prefix())),
                key,
                honoured_when,
            }
        );
        // The refusal is the whole outcome: nothing reached the engine.
        assert!(
            declare(&mut next, &declaration_at(kind, &base), &derived, "sub-1").is_ok(),
            "and the base without it still applies, so the key is what was refused"
        );
    }
}

/// `VT-2` — the control: the same keys are HONOURED at the state that honours
/// them, so the refusal has not been widened into a ban (`EX-4`).
///
/// The mirror of the test above, cell for cell. Without it, refusing all three
/// keys unconditionally would pass every other assertion in this file.
#[test]
fn the_same_keys_are_honoured_at_the_state_that_honours_them() {
    let rows = state_axis_rows();
    assert_eq!(rows.len(), 3, "the same three cells, at their other state");

    for (key, kind, honoured_when) in rows {
        let base = base_at(kind, honoured_when, key);
        let mut carrying = base.clone();
        carrying.push(key);

        assert_ne!(
            engine_outcome(honoured_when, kind, &base),
            engine_outcome(honoured_when, kind, &carrying),
            "`{key}` must still change what the engine does at the state that \
             honours it"
        );
        assert!(
            !admission_refuses(honoured_when, kind, &carrying),
            "`{key}` is honoured here, so admission must not refuse it"
        );
    }
}

/// `VT-3` — `blocking` on a finding the run ALREADY HOLDS is refused, not
/// discarded.
///
/// Named on its own rather than left to the matrix because it is the cell that
/// decided the design (`sec-3`). The alternative to refusing was to read the
/// dropped key as omission-persist, and `blocking` is what defeats it: it is the
/// flag the lock gate reads, so a caller *correcting* a finding's `blocking` is
/// told they succeeded and changes nothing — and persist semantics would make
/// that silence correct by definition. `Sparse::Omitted` already means persist
/// and is reachable by not sending the key, so a PRESENT value that is discarded
/// has no honest reading as persistence (`EX-4`).
///
/// Written as literal JSON against a hand-built run, not through the matrix
/// fixtures: this cell's value is that it reads as the caller's own mistake.
#[test]
fn correcting_blocking_on_an_already_raised_finding_is_refused() {
    let (next, _) = universe(SubjectState::Held, IdKind::Finding);
    assert!(
        next.review.findings.iter().any(|held| !held.blocking),
        "the run holds the finding, and holds it non-blocking"
    );

    let refused = Batch::of(vec![declared(r#"{"subject": "fnd-1", "blocking": true}"#)])
        .validate(|subject| subject_state(&next, subject))
        .expect_err("`blocking` is read only where the finding is raised");

    assert_eq!(
        refused,
        Refusal::InertAtState {
            subject: id("fnd-1"),
            key: "blocking",
            honoured_when: SubjectState::Absent,
        }
    );
}

/// Every state-axis row names a kind whose state `subject_state` can observe.
///
/// `subject_state` answers [`SubjectState::Absent`] unconditionally for the three
/// kinds with no held state — a `cp-` subject is not a record the run holds, and
/// the other two are not declaration subjects at all. That is the honest answer,
/// and it would also silently disarm a state-axis row added at one of those
/// kinds: the row would never fire, and no test above would notice.
///
/// So the two axes are pinned against each other here. [`seat_subject`] is the
/// other half — it panics for a kind it cannot seat, which catches the same
/// mistake from the fixture side.
#[test]
fn every_state_axis_row_names_a_kind_whose_state_this_function_observes() {
    for (key, kind, _) in state_axis_rows() {
        let (next, _) = universe(SubjectState::Held, kind);
        assert_eq!(
            subject_state(&next, &id(&format!("{}1", kind.prefix()))),
            SubjectState::Held,
            "`{key}`'s row is at a `{}` subject, whose held state `subject_state` \
             must be able to see",
            kind.prefix()
        );
        // And the same id reads `Absent` in the universe that does not seat it,
        // so the answer is a reading of the snapshot rather than a constant.
        let (unseated, _) = universe(SubjectState::Absent, kind);
        assert_eq!(
            subject_state(&unseated, &id(&format!("{}1", kind.prefix()))),
            SubjectState::Absent,
        );
    }
}

// ── the payload contract's key-set pin (SL-251 PHASE-02, sec-8 pin 1) ───────

/// One contract-bearing closure struct, checked against a value of its own type.
///
/// Two assertions, both off arguments the call site already holds: the
/// serialised key set equals the contract's rows, and the contract's `name` is
/// the Rust type's own. Together they close the loop the table would otherwise
/// leave open — a row naming a key no field emits fails here, and a field that
/// reaches the wire with no row fails equally.
///
/// The oracle is a **serialised** value and never a hand-written key list, on
/// `I9`'s reasoning: a hand-written list is a third spelling, free to agree with
/// neither the type nor the table.
fn assert_keys_described<T: Serialize>(value: &T, contract: &TypeContract) {
    let TypeForm::Struct { keys, .. } = contract.form else {
        panic!(
            "{}: a closure struct is described by a struct form",
            contract.name
        );
    };

    let serialised = serde_json::to_value(value).expect("a closure value serialises");
    let on_the_wire: BTreeSet<&str> = serialised
        .as_object()
        .unwrap_or_else(|| panic!("{}: a wire struct serialises to an object", contract.name))
        .keys()
        .map(String::as_str)
        .collect();
    let described: BTreeSet<&str> = keys.iter().map(|key| key.key).collect();
    assert_eq!(on_the_wire, described, "{}", contract.name);

    // The Rust type name a refusal cites (`sec-2`, *Naming*), read off the type
    // rather than retyped beside it. `type_name` returns a path, and none of
    // these structs is generic, so its final segment is the whole of the name.
    let path = std::any::type_name::<T>();
    let bare = path
        .rsplit("::")
        .next()
        .expect("a type path has a final segment");
    assert!(!bare.contains('<'), "{path}: no closure struct is generic");
    assert_eq!(contract.name, bare);
}

/// `sec-8` pin 1 — every contract-bearing closure struct's rows are exactly the
/// keys serde writes for it, and each contract names the Rust type a refusal
/// cites.
///
/// **Ten call sites, and the closure holds eleven struct types.** The eleventh
/// is `SubmissionEnvelope`, which has no `TypeContract` to pass: `#[serde(flatten)]`
/// renders its three keys at the root, so it participates through `PAYLOAD`'s
/// composition and its pin is the disjoint union below.
#[test]
fn the_payload_table_describes_every_wire_key_of_the_ten_contract_bearing_structs() {
    assert_keys_described(&ApplyRequest::fully_populated(), &PAYLOAD);
    assert_keys_described(
        &TraversalDeclaration::fully_populated(),
        &TRAVERSAL_DECLARATION,
    );
    assert_keys_described(&StageDeclaration::fully_populated(), &STAGE_DECLARATION);
    assert_keys_described(
        &AcceptanceDeclaration::fully_populated(),
        &ACCEPTANCE_DECLARATION,
    );
    assert_keys_described(&Declaration::fully_populated(id("inq-1")), &DECLARATION);
    assert_keys_described(&CreateRecord::fully_populated(), &CREATE_RECORD);
    assert_keys_described(
        &DischargeDeclaration::fully_populated(),
        &DISCHARGE_DECLARATION,
    );
    assert_keys_described(
        &ReviewPolicyDeclaration::fully_populated(),
        &REVIEW_POLICY_DECLARATION,
    );
    assert_keys_described(
        &CheckpointActDeclaration::fully_populated(),
        &CHECKPOINT_ACT_DECLARATION,
    );
    assert_keys_described(
        &AgentActDeclaration::fully_populated(),
        &AGENT_ACT_DECLARATION,
    );
}

/// The keys `SubmissionEnvelope` contributes to the root through its flatten.
const ENVELOPE_KEYS: usize = 3;
/// The act fields `ApplyRequest` declares in its own right.
///
/// **Nine, not the eight of `ApplyRequest::WRITER_ACTS`**: that list correctly
/// omits `delegation`, whose acts are not all writes, and counting the payload's
/// keys off it would reproduce the omission the contract exists to close.
const ACT_KEYS: usize = 9;

/// `sec-8` pin 1's eleventh member — `SubmissionEnvelope`, pinned through the
/// root's composition rather than through a contract of its own.
///
/// A **disjoint** union, and the disjointness is the point rather than a
/// formality: the flatten and the act fields share one key namespace, so an act
/// field named `run_uid` would not be a twelfth key, it would silently
/// replace the envelope's. Asserting the two sides are disjoint and that their
/// sizes still sum to `PAYLOAD`'s row count is what makes that collision a
/// failure here instead of a table quietly one row longer than the wire.
#[test]
fn the_roots_twelve_rows_are_the_envelopes_keys_disjointly_united_with_the_nine_acts() {
    let envelope = serde_json::to_value(SubmissionEnvelope::fully_populated())
        .expect("an envelope serialises");
    let envelope_keys: BTreeSet<&str> = envelope
        .as_object()
        .expect("an envelope serialises to an object")
        .keys()
        .map(String::as_str)
        .collect();

    let root = serde_json::to_value(ApplyRequest::fully_populated()).expect("a payload serialises");
    let root_keys: BTreeSet<&str> = root
        .as_object()
        .expect("a payload serialises to an object")
        .keys()
        .map(String::as_str)
        .collect();

    assert!(
        envelope_keys.is_subset(&root_keys),
        "the flatten puts the envelope's keys at the root"
    );
    let act_keys: BTreeSet<&str> = root_keys.difference(&envelope_keys).copied().collect();

    assert!(
        envelope_keys.is_disjoint(&act_keys),
        "no act field shares a key with the flattened envelope"
    );
    assert_eq!(envelope_keys.len(), ENVELOPE_KEYS);
    assert_eq!(act_keys.len(), ACT_KEYS);

    let TypeForm::Struct { keys, .. } = PAYLOAD.form else {
        panic!("the root is a struct");
    };
    let described: BTreeSet<&str> = keys.iter().map(|key| key.key).collect();
    let united: BTreeSet<&str> = envelope_keys.union(&act_keys).copied().collect();
    assert_eq!(united, described);
    assert_eq!(described.len(), ENVELOPE_KEYS + ACT_KEYS);
}

// ── the payload contract's value pins (SL-251 PHASE-03, sec-8 pins 2-3) ─────
//
// Two walks over the same tree, and the whole instrument is that they do not
// share a derivation. `descend_*` holds a **JSON value** in every frame and
// records a site at arrival; `declare_*` holds the **table** and has no JSON in
// scope anywhere. The coverage equality between them is what makes an input a
// pin never reaches a failure rather than a declaration that quietly stops
// being tested (design, *The oracle discipline*).

/// The site-id grammar, spelled once (STD-001). Both walks name sites with it —
/// which is the *naming* and not the derivation; they would be incomparable
/// otherwise.
const SITE_KEY: &str = ".";
/// A variant of a named type.
const SITE_VARIANT: &str = "::";
/// A variant with no token to name it by — an untagged arm, by position.
const SITE_UNTOKENED: &str = "#";
/// A sequence's element type.
const SITE_SEQ: &str = "#seq";
/// A map's key type.
const SITE_MAP_KEY: &str = "#map-key";
/// A map's value type.
const SITE_MAP_VALUE: &str = "#map-value";
/// An untagged variant's shape.
const SITE_SHAPE: &str = "#shape";

/// The root's sequence-of-declarations key — the positive control's subject.
const DECLARE_KEY: &str = "declare";
/// The row whose admissible kinds are the engine's declarable set.
const SUBJECT_KEY: &str = "subject";

/// One key's site, rooted at whatever owns it — a `TypeContract::name` for a
/// struct's row, a variant's site for a variant's row (`PHASE-03/D2`).
fn key_site(prefix: &str, key: &str) -> String {
    format!("{prefix}{SITE_KEY}{key}")
}

/// One variant's site. An untagged arm has no token, so it is named by position.
fn variant_site(owner: &str, index: usize, token: Option<&str>) -> String {
    match token {
        Some(token) => format!("{owner}{SITE_VARIANT}{token}"),
        None => format!("{owner}{SITE_VARIANT}{SITE_UNTOKENED}{index}"),
    }
}

// ── the right side: every declaration site in the table ─────────────────────

/// Every site the contract tree declares, read off the **table** — no JSON is
/// reachable from anywhere in this walk, which is half of `EX-3`.
fn declared_sites(root: TypeContract) -> BTreeSet<String> {
    let mut sites = BTreeSet::new();
    let mut walked = BTreeSet::new();
    declare_type(root, &mut sites, &mut walked);
    sites
}

/// Memoised by type name, so the contract graph may become cyclic without this
/// hanging — and so `Declaration`, which is reached from two places, contributes
/// one set of sites rather than two spellings of it.
fn declare_type(
    contract: TypeContract,
    sites: &mut BTreeSet<String>,
    walked: &mut BTreeSet<&'static str>,
) {
    if !walked.insert(contract.name) {
        return;
    }
    match contract.form {
        TypeForm::Struct { keys, .. } => declare_keys(contract.name, keys, sites, walked),
        TypeForm::Enum { variants, .. } => {
            for (index, variant) in variants.iter().enumerate() {
                let site = variant_site(contract.name, index, variant.token);
                sites.insert(site.clone());
                declare_payload(variant.payload, &site, sites, walked);
            }
        }
    }
}

fn declare_payload(
    payload: VariantPayload,
    site: &str,
    sites: &mut BTreeSet<String>,
    walked: &mut BTreeSet<&'static str>,
) {
    match payload {
        VariantPayload::Absent => {}
        VariantPayload::Keys(rows) => declare_keys(site, rows, sites, walked),
        VariantPayload::Inlines(target) => declare_type(*target, sites, walked),
        VariantPayload::Shape(shape) => {
            let child = format!("{site}{SITE_SHAPE}");
            sites.insert(child.clone());
            declare_wire(*shape, &child, sites, walked);
        }
    }
}

fn declare_keys(
    prefix: &str,
    keys: &'static [KeyContract],
    sites: &mut BTreeSet<String>,
    walked: &mut BTreeSet<&'static str>,
) {
    for key in keys {
        let site = key_site(prefix, key.key);
        sites.insert(site.clone());
        declare_wire(key.ty, &site, sites, walked);
    }
}

fn declare_wire(
    ty: WireType,
    site: &str,
    sites: &mut BTreeSet<String>,
    walked: &mut BTreeSet<&'static str>,
) {
    match ty {
        // A scalar is the site it sits at and declares nothing below it.
        WireType::Text
        | WireType::Integer
        | WireType::Boolean
        | WireType::Id(_)
        | WireType::Token(_) => {}
        // A named edge's sites are rooted at the *target's* name, not at this
        // one, which is what makes `AcceptanceDeclaration.basis` one site rather
        // than four (`PHASE-03/D2`).
        WireType::Named(target) => declare_type(*target, sites, walked),
        WireType::Seq(inner) => {
            let child = format!("{site}{SITE_SEQ}");
            sites.insert(child.clone());
            declare_wire(*inner, &child, sites, walked);
        }
        WireType::Map { key, value } => {
            match key {
                MapKey::Of(inner) => {
                    let child = format!("{site}{SITE_MAP_KEY}");
                    sites.insert(child.clone());
                    declare_wire(*inner, &child, sites, walked);
                }
                // Pin 5's, and skipped by name rather than by a wildcard: which
                // keys are legal is chosen by a sibling field's value, which no
                // fixture can know.
                MapKey::Extern { .. } => {}
            }
            let child = format!("{site}{SITE_MAP_VALUE}");
            sites.insert(child.clone());
            declare_wire(*value, &child, sites, walked);
        }
    }
}

// ── the left side: the recursive descent (sec-8 pin 2) ──────────────────────

/// Descend a value against the type that describes it.
///
/// Faults accumulate rather than panicking, for two load-bearing reasons: an
/// untagged enum has to *count* how many declared shapes a value satisfies, and
/// the coverage left side must be recorded from arrival even on a run that will
/// fail.
fn descend_type(
    value: &Value,
    contract: TypeContract,
    seen: &mut BTreeSet<String>,
    faults: &mut Vec<String>,
) {
    match contract.form {
        TypeForm::Struct { keys, .. } => {
            let Some(object) = value.as_object() else {
                faults.push(format!(
                    "{}: a struct target is a JSON object, got {value}",
                    contract.name
                ));
                return;
            };
            descend_keys(contract.name, keys, &Fields::plain(object), seen, faults);
        }
        TypeForm::Enum { tagging, variants } => {
            descend_enum(value, contract, tagging, variants, seen, faults);
        }
    }
}

/// A key surface against its rows: the sets are equal, and each value is
/// descended against its own row. This is pin 1's assertion applied at the edge
/// rather than at the type, which is what tells two same-shaped `Named` targets
/// apart.
fn descend_keys(
    prefix: &str,
    keys: &'static [KeyContract],
    fields: &Fields<'_>,
    seen: &mut BTreeSet<String>,
    faults: &mut Vec<String>,
) {
    let on_the_wire = fields.names();
    let described: BTreeSet<&str> = keys.iter().map(|key| key.key).collect();
    if on_the_wire != described {
        faults.push(format!(
            "{prefix}: the keys on the wire {on_the_wire:?} are not the declared rows {described:?}"
        ));
    }
    for key in keys {
        if let Some(value) = fields.get(key.key) {
            let site = key_site(prefix, key.key);
            seen.insert(site.clone());
            descend_wire(value, key.ty, &site, seen, faults);
        }
    }
}

/// Select the variant, then descend into it.
///
/// Selection is read from the tagging table ([`place`]) and never restated
/// (`EX-2`). Under `Untagged` there is no token to select by, so the value must
/// satisfy **exactly one** declared shape — which is also the well-formedness
/// the untagged model needs.
fn descend_enum(
    value: &Value,
    contract: TypeContract,
    tagging: Tagging,
    variants: &'static [VariantContract],
    seen: &mut BTreeSet<String>,
    faults: &mut Vec<String>,
) {
    let selected: Vec<usize> = match tagging {
        Tagging::Internal(_) | Tagging::External => variants
            .iter()
            .enumerate()
            .filter(|(_, variant)| {
                place(contract.name, tagging, variant.payload, value)
                    .is_ok_and(|placement| placement.token() == variant.token)
            })
            .map(|(index, _)| index)
            .collect(),
        Tagging::Untagged => variants
            .iter()
            .enumerate()
            .filter(|(index, variant)| {
                let mut probe_seen = BTreeSet::new();
                let mut probe_faults = Vec::new();
                descend_variant(
                    value,
                    contract,
                    tagging,
                    *index,
                    variant,
                    &mut probe_seen,
                    &mut probe_faults,
                );
                probe_faults.is_empty()
            })
            .map(|(index, _)| index)
            .collect(),
    };

    let [index] = selected[..] else {
        faults.push(format!(
            "{}: {value} satisfies {} declared variants, not exactly one",
            contract.name,
            selected.len()
        ));
        return;
    };
    descend_variant(
        value,
        contract,
        tagging,
        index,
        &variants[index],
        seen,
        faults,
    );
}

fn descend_variant(
    value: &Value,
    contract: TypeContract,
    tagging: Tagging,
    index: usize,
    variant: &VariantContract,
    seen: &mut BTreeSet<String>,
    faults: &mut Vec<String>,
) {
    let site = variant_site(contract.name, index, variant.token);
    seen.insert(site.clone());
    let placement = match place(contract.name, tagging, variant.payload, value) {
        Ok(placement) => placement,
        Err(fault) => {
            faults.push(fault);
            return;
        }
    };
    descend_payload(&placement, variant.payload, &site, seen, faults);
}

/// The four `VariantPayload` arms, **with no wildcard** — a new arm is a compile
/// error here (`EX-1`).
fn descend_payload(
    placement: &Placement<'_>,
    payload: VariantPayload,
    site: &str,
    seen: &mut BTreeSet<String>,
    faults: &mut Vec<String>,
) {
    match payload {
        VariantPayload::Absent => {
            let Some(fields) = placement.fields() else {
                faults.push(format!("{site}: a unit variant is not a shape"));
                return;
            };
            let riding = fields.names();
            if !riding.is_empty() {
                faults.push(format!(
                    "{site}: a unit variant carries no keys, got {riding:?}"
                ));
            }
        }
        VariantPayload::Keys(rows) => {
            let Some(fields) = placement.fields() else {
                faults.push(format!("{site}: a keyed variant is not a shape"));
                return;
            };
            descend_keys(site, rows, &fields, seen, faults);
        }
        // The target's keys arrive in the variant's place, so its rows are read
        // against this placement's key surface and its sites stay rooted at its
        // own name.
        VariantPayload::Inlines(target) => {
            let Some(fields) = placement.fields() else {
                faults.push(format!("{site}: an inlining variant is not a shape"));
                return;
            };
            match target.form {
                TypeForm::Struct { keys, .. } => {
                    descend_keys(target.name, keys, &fields, seen, faults);
                }
                TypeForm::Enum { .. } => faults.push(format!(
                    "{site}: an inlining variant names a struct, and {} is an enum",
                    target.name
                )),
            }
        }
        VariantPayload::Shape(shape) => {
            let Placement::Shape(inner) = placement else {
                faults.push(format!("{site}: a shape variant is untagged"));
                return;
            };
            let child = format!("{site}{SITE_SHAPE}");
            seen.insert(child.clone());
            descend_wire(inner, *shape, &child, seen, faults);
        }
    }
}

/// The eight `WireType` arms, **with no wildcard** — a new arm is a compile
/// error here (`EX-1`). `site` is this position's site id and is already
/// recorded; what this adds is the sites nested inside it.
fn descend_wire(
    value: &Value,
    ty: WireType,
    site: &str,
    seen: &mut BTreeSet<String>,
    faults: &mut Vec<String>,
) {
    match ty {
        WireType::Text => {
            if !value.is_string() {
                faults.push(format!("{site}: `Text` is a JSON string, got {value}"));
            }
        }
        WireType::Integer => {
            if !value.is_i64() && !value.is_u64() {
                faults.push(format!("{site}: `Integer` is a JSON integer, got {value}"));
            }
        }
        WireType::Boolean => {
            if !value.is_boolean() {
                faults.push(format!("{site}: `Boolean` is a JSON boolean, got {value}"));
            }
        }
        // Per row and structural: the value parses as an id whose **own** kind is
        // one this row admits. `IdKind::declarable` is not the rule here — it is
        // a predicate over declaration *subjects*, and four `Id` rows in
        // `DELEGATION_ACT` legitimately declare a kind it calls false
        // (`PHASE-03/EX-10`). The two claims that *are* the engine's are asserted
        // at their own sites, below.
        WireType::Id(kinds) => {
            let Some(raw) = value.as_str() else {
                faults.push(format!("{site}: an `Id` is a JSON string, got {value}"));
                return;
            };
            match DesignId::parse(raw) {
                Ok(id) if kinds.contains(&id.kind()) => {}
                Ok(id) => faults.push(format!(
                    "{site}: {raw:?} is a {:?} id, which this row does not admit ({kinds:?})",
                    id.kind()
                )),
                Err(_) => faults.push(format!("{site}: {raw:?} does not parse as an id")),
            }
        }
        // The vocabulary is pin 4's (`Fixed`) or pin 5's (`Extern`); what is
        // this pin's is that a token reaches the wire as a string.
        //
        // The `Fixed` arm has **no reachable input** in the real table today —
        // every `Fixed` row is in the exemplar set, and PHASE-04 supplies the
        // first real vocabularies. It exists because `EX-1` forbids a wildcard,
        // and because a site is read off the table rather than off a value, its
        // being uninhabited costs the coverage equality nothing. Do not delete
        // it (`PHASE-03/F-5`).
        WireType::Token(TokenSource::Fixed(_)) | WireType::Token(TokenSource::Extern(_)) => {
            if !value.is_string() {
                faults.push(format!("{site}: a `Token` is a JSON string, got {value}"));
            }
        }
        WireType::Named(target) => descend_type(value, *target, seen, faults),
        WireType::Seq(inner) => {
            let Some(elements) = value.as_array() else {
                faults.push(format!("{site}: a `Seq` is a JSON array, got {value}"));
                return;
            };
            let child = format!("{site}{SITE_SEQ}");
            for element in elements {
                seen.insert(child.clone());
                descend_wire(element, *inner, &child, seen, faults);
            }
        }
        WireType::Map { key, value: row } => {
            let Some(object) = value.as_object() else {
                faults.push(format!("{site}: a `Map` is a JSON object, got {value}"));
                return;
            };
            match key {
                MapKey::Of(inner) => {
                    let child = format!("{site}{SITE_MAP_KEY}");
                    for name in object.keys() {
                        seen.insert(child.clone());
                        descend_wire(&Value::String(name.clone()), *inner, &child, seen, faults);
                    }
                }
                // Pin 5's: which keys are legal is chosen by a sibling field's
                // value. Skipped by name, never by a wildcard.
                MapKey::Extern { .. } => {}
            }
            let child = format!("{site}{SITE_MAP_VALUE}");
            for element in object.values() {
                seen.insert(child.clone());
                descend_wire(element, *row, &child, seen, faults);
            }
        }
    }
}

// ── the coverage union, and what the descent made of it ─────────────────────

fn wire<T: Serialize>(value: &T) -> Value {
    serde_json::to_value(value).expect("a closure value serialises")
}

/// Every value pin 2 walks, each paired with the contract that describes it.
///
/// The ten contract-bearing fixtures — `SubmissionEnvelope` has no contract,
/// its three keys being `PAYLOAD`'s rows through the flatten — plus, for each
/// closure enum, every one of pin 4's per-variant samples paired with that
/// enum's **real** contract. The samples are not optional: eight enums' variants
/// are reached by no fixture at all, so without them the equality could not hold
/// on a correct table (`PHASE-03/F-2`).
///
/// The root request is a parameter so the positive control can hand in a
/// deliberately impoverished one without touching a fixture.
fn coverage_union(request: &ApplyRequest) -> Vec<(Value, TypeContract)> {
    let mut roots = vec![
        (wire(request), PAYLOAD),
        (
            wire(&TraversalDeclaration::fully_populated()),
            TRAVERSAL_DECLARATION,
        ),
        (
            wire(&StageDeclaration::fully_populated()),
            STAGE_DECLARATION,
        ),
        (
            wire(&AcceptanceDeclaration::fully_populated()),
            ACCEPTANCE_DECLARATION,
        ),
        (
            wire(&Declaration::fully_populated(id("inq-1"))),
            DECLARATION,
        ),
        (wire(&CreateRecord::fully_populated()), CREATE_RECORD),
        (
            wire(&DischargeDeclaration::fully_populated()),
            DISCHARGE_DECLARATION,
        ),
        (
            wire(&ReviewPolicyDeclaration::fully_populated()),
            REVIEW_POLICY_DECLARATION,
        ),
        (
            wire(&CheckpointActDeclaration::fully_populated()),
            CHECKPOINT_ACT_DECLARATION,
        ),
        (
            wire(&AgentActDeclaration::fully_populated()),
            AGENT_ACT_DECLARATION,
        ),
    ];
    for claim in claims() {
        for sample in claim.samples {
            // A **legacy** variant keeps its Rust variant so a stored snapshot
            // parses, but its contract row retired (`SL-264` sec-3), so there is
            // no declared variant to descend into. Its read path is pinned by
            // pin 4 instead.
            if sample_token(claim.tagging, &sample).is_some_and(|token| is_legacy_token(&token)) {
                continue;
            }
            roots.push((sample.value, *claim.contract));
        }
    }
    roots
}

/// The token a sample puts on the wire, read the way [`place`] does but without
/// the declared row — enough to tell a **legacy** variant (one whose contract row
/// retired, `SL-264` sec-3) from one that was never declared at all.
fn sample_token(tagging: Tagging, sample: &VariantSample) -> Option<String> {
    match tagging {
        Tagging::External => match &sample.value {
            Value::String(token) => Some(token.clone()),
            Value::Object(map) => map.keys().next().cloned(),
            _ => None,
        },
        Tagging::Internal(tag) => sample
            .value
            .get(tag)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        Tagging::Untagged => None,
    }
}

/// Run the descent over a union and report **what it arrived at**, with whatever
/// it faulted on. The site set is recorded from arrival and from nowhere else —
/// nothing in this function or below it reads the table for the left side.
fn reached(union: &[(Value, TypeContract)]) -> (BTreeSet<String>, Vec<String>) {
    let mut seen = BTreeSet::new();
    let mut faults = Vec::new();
    for (value, contract) in union {
        descend_type(value, *contract, &mut seen, &mut faults);
    }
    (seen, faults)
}

/// `sec-8` pin 2 — every declaration site's JSON kind is the kind its declared
/// [`WireType::Named`] edge or scalar row says it is, resolved structurally.
///
/// The walk is an exhaustive match over `sec-2`'s model with no wildcard arm, so
/// a new `WireType` or `VariantPayload` is a build failure here rather than a
/// declaration that quietly stops being checked.
#[test]
fn the_descent_checks_every_declaration_site_against_its_declared_wire_type() {
    let (_, faults) = reached(&coverage_union(&ApplyRequest::fully_populated()));
    assert!(faults.is_empty(), "{faults:#?}");
}

/// The two claims the generic `Id` arm cannot make, each derived from the
/// **engine** rather than from a slice the table asserts about itself
/// (`PHASE-03/EX-10`).
#[test]
fn the_declared_id_kinds_are_the_engines_declarable_set() {
    let TypeForm::Struct { keys, .. } = DECLARATION.form else {
        panic!("a declaration is a struct");
    };
    let subject = keys
        .iter()
        .find(|key| key.key == SUBJECT_KEY)
        .expect("a declaration addresses a subject");
    let WireType::Id(declared) = subject.ty else {
        panic!("a subject is an id");
    };
    let engine: BTreeSet<IdKind> = IdKind::ALL
        .into_iter()
        .filter(|kind| kind.declarable())
        .collect();
    assert_eq!(
        declared.iter().copied().collect::<BTreeSet<IdKind>>(),
        engine,
        "`subject` admits exactly the kinds a declaration may address"
    );
}

/// `sec-8`'s coverage equality — the set of sites the descent **arrived at** is
/// the set of sites the table declares (`EX-3`).
///
/// The two sides derive independently: the left is recorded from arrival with a
/// JSON value in hand, the right is read off the table with no JSON in scope.
/// An implementation that recorded sites from the table would satisfy both at
/// once and pass on any input at all, which is what the positive control below
/// exists to catch.
///
/// **The control runs first, and it must fail.** One fixture's `Seq` is
/// `emptied` — `ApplyRequest.declare`, whose element site no other value in the
/// union reaches — and the equality is asserted to break, naming exactly that
/// site. Only then is the real union asserted clean.
#[test]
fn every_declared_site_is_reached_and_an_emptied_seq_breaks_the_coverage_equality() {
    let declared = declared_sites(PAYLOAD);

    let mut emptied = ApplyRequest::fully_populated();
    emptied.declare = Vec::new();
    let (control, _) = reached(&coverage_union(&emptied));
    assert_ne!(
        control, declared,
        "an emptied `Seq` must break the coverage equality"
    );
    let lost: Vec<&String> = declared.difference(&control).collect();
    // The key site goes with it since SL-262 made an empty `declare` skip on
    // serialisation (the forward edge's `ready` payload shows only what it
    // sets), so the key is not on the wire to arrive at either.
    let key = format!("{}{SITE_KEY}{DECLARE_KEY}", PAYLOAD.name);
    let element = format!("{key}{SITE_SEQ}");
    assert_eq!(
        lost,
        vec![&key, &element],
        "an emptied `Seq` loses its key and element sites and nothing else"
    );

    let (arrived, _) = reached(&coverage_union(&ApplyRequest::fully_populated()));
    assert_eq!(
        arrived,
        declared,
        "declared but never reached: {:?}\nreached but never declared: {:?}",
        declared.difference(&arrived).collect::<Vec<_>>(),
        arrived.difference(&declared).collect::<Vec<_>>()
    );
}

// ── pin 3: presence, over the structs ───────────────────────────────────────

/// `{keys serialising to null}` == `{keys the contract declares `Sparse`}`.
fn assert_sparse_keys_are_null<T: Serialize>(value: &T, contract: &TypeContract) {
    let TypeForm::Struct { keys, .. } = contract.form else {
        panic!(
            "{}: a closure struct is described by a struct form",
            contract.name
        );
    };
    let serialised = wire(value);
    let nulled: BTreeSet<&str> = serialised
        .as_object()
        .unwrap_or_else(|| panic!("{}: a wire struct serialises to an object", contract.name))
        .iter()
        .filter(|(_, value)| value.is_null())
        .map(|(key, _)| key.as_str())
        .collect();
    let sparse: BTreeSet<&str> = keys
        .iter()
        .filter(|key| key.presence == Presence::Sparse)
        .map(|key| key.key)
        .collect();
    assert_eq!(nulled, sparse, "{}", contract.name);
}

/// `{keys whose removal makes the payload fail to deserialise}` ==
/// `{keys the contract declares `Presence::Required`}`.
///
/// The **read** path, and never serialization: *required* is a property of what
/// `from_value` refuses, and two earlier drafts that read what a minimal value
/// emits were each wrong — `ApplyRequest.declare` is `#[serde(default)]` with no
/// `skip_serializing_if`, so it serialises as `[]` while being genuinely
/// omissible, and the serialization reading fails on a correct table.
fn assert_removal_refuses_required<T: Serialize + DeserializeOwned>(
    value: &T,
    contract: &TypeContract,
) {
    let TypeForm::Struct { keys, .. } = contract.form else {
        panic!(
            "{}: a closure struct is described by a struct form",
            contract.name
        );
    };
    let serialised = wire(value);
    let object = serialised
        .as_object()
        .unwrap_or_else(|| panic!("{}: a wire struct serialises to an object", contract.name));

    let mut refusing = BTreeSet::new();
    for key in object.keys() {
        let mut probe = object.clone();
        probe.remove(key);
        if serde_json::from_value::<T>(Value::Object(probe)).is_err() {
            refusing.insert(key.as_str());
        }
    }
    assert_eq!(
        refusing,
        required_keys(keys),
        "{}: the keys whose removal refuses are not the declared \
         `Presence::Required` rows",
        contract.name
    );
}

/// `sec-8` pin 3's first half — a fixture with every `Sparse` field `Null` and
/// every `Option` field `Some` emits JSON `null` for exactly the sparse keys.
///
/// Two fixtures, not twelve: `Declaration` and `TraversalDeclaration` are the
/// closure's only `Sparse`-bearing structs. Both sit deliberately **outside**
/// pin 2's coverage union — they exist to make containers absent, which is the
/// assertion rather than a gap.
#[test]
fn sparse_keys_are_exactly_the_keys_that_serialise_to_null() {
    assert_sparse_keys_are_null(&Declaration::sparse_nulled(id("inq-1")), &DECLARATION);
    assert_sparse_keys_are_null(
        &TraversalDeclaration::sparse_nulled(),
        &TRAVERSAL_DECLARATION,
    );
}

/// `sec-8` pin 3's second half — the read-path removal probe over the ten
/// contract-bearing fixtures.
#[test]
fn the_removal_probe_refuses_exactly_the_required_rows() {
    assert_removal_refuses_required(&ApplyRequest::fully_populated(), &PAYLOAD);
    assert_removal_refuses_required(
        &TraversalDeclaration::fully_populated(),
        &TRAVERSAL_DECLARATION,
    );
    assert_removal_refuses_required(&StageDeclaration::fully_populated(), &STAGE_DECLARATION);
    assert_removal_refuses_required(
        &AcceptanceDeclaration::fully_populated(),
        &ACCEPTANCE_DECLARATION,
    );
    assert_removal_refuses_required(&Declaration::fully_populated(id("inq-1")), &DECLARATION);
    assert_removal_refuses_required(&CreateRecord::fully_populated(), &CREATE_RECORD);
    assert_removal_refuses_required(
        &DischargeDeclaration::fully_populated(),
        &DISCHARGE_DECLARATION,
    );
    assert_removal_refuses_required(
        &ReviewPolicyDeclaration::fully_populated(),
        &REVIEW_POLICY_DECLARATION,
    );
    assert_removal_refuses_required(
        &CheckpointActDeclaration::fully_populated(),
        &CHECKPOINT_ACT_DECLARATION,
    );
    assert_removal_refuses_required(
        &AgentActDeclaration::fully_populated(),
        &AGENT_ACT_DECLARATION,
    );
}

// ── leg 3, the value axis (DEC-247, ISS-290) ──────────────────────────────

/// A term at `kind`, whatever kind that is — the one place this suite names all
/// four constructors, so the matrices below can quantify over the vocabulary
/// instead of over the three cases someone remembered.
///
/// The value is one byte, which is inside every admission bound, so a cell that
/// refuses refuses for the reason under test rather than for length.
fn term_at(key: PayloadKey, kind: ValueKind) -> PayloadTerm {
    match kind {
        ValueKind::Token => PayloadTerm::token(key, "x").expect("one byte is inside the id bound"),
        ValueKind::Label => {
            PayloadTerm::label(key, "x").expect("one byte is inside the label bound")
        }
        ValueKind::Digest => PayloadTerm::digest(key, "x"),
        ValueKind::Prose => PayloadTerm::prose(key, "x"),
    }
}

/// Every `(key, kind)` pair any event declares — the payload key vocabulary as
/// the declaration itself spells it.
///
/// Read from `payload_terms()` rather than from a written-out list so a key
/// added to an event joins these matrices without anyone remembering to add it
/// (`mem_019fe0c6db677dd1aa6a8ef8e91f3828`, point 1).
fn declared_keys() -> BTreeSet<PayloadKey> {
    ChangeEvent::EMITTABLE
        .iter()
        .flat_map(|event| event.payload_terms().iter().map(|(key, _)| *key))
        .collect()
}

/// `VT-1` / `EX-1` / `EX-4` (`DEC-247`, `ISS-290`) — a term whose `ValueKind`
/// is not the one its event declares for that key is refused, at the seam that
/// has both the event and the term in scope.
///
/// A matrix, not the one live instance: `ISS-290` survived `SL-233` `PHASE-08`
/// because its sibling key was repaired at the instance. Both vocabularies are
/// read from their own source — `ChangeEvent::EMITTABLE` and `ValueKind::ALL` —
/// so a new event, a new key or a fourth value kind joins the sweep unasked.
///
/// **Two-sided, on `I10`'s reasoning**: each declared pair is asserted
/// *admitted* as well as each undeclared one refused. Without the control a
/// seam that refused everything would read green, which is a ban rather than a
/// rule.
#[test]
fn a_term_constructed_at_an_undeclared_kind_is_refused() {
    let mut admitted = 0_usize;
    let mut refused = 0_usize;
    for event in ChangeEvent::EMITTABLE {
        // Per event, not once over the sweep: a total is blind to one event
        // dropping out of it, which is how a cell "fails by skipping" rather
        // than by failing (`mem_019fe0c6db677dd1aa6a8ef8e91f3828`, point 2). An
        // emitted row with no terms carries nothing its event does not already
        // say, so an empty shape here is a defect and not a case.
        assert!(
            !event.payload_terms().is_empty(),
            "{} declares at least one term, so it contributes cells below",
            event.as_str()
        );
        for (key, declared) in event.payload_terms() {
            assert!(
                event.shaped(vec![term_at(*key, *declared)]).is_ok(),
                "{} declares {} as {declared:?}, so a term at that kind is admitted",
                event.as_str(),
                key.as_str()
            );
            admitted += 1;
            for kind in ValueKind::ALL.iter().filter(|kind| *kind != declared) {
                let outcome = event.shaped(vec![term_at(*key, *kind)]);
                assert!(
                    matches!(
                        outcome,
                        Err(Refusal::UndeclaredTerm {
                            event: refused_event,
                            key: refused_key,
                            kind: refused_kind,
                        }) if refused_event == event && refused_key == *key && refused_kind == *kind
                    ),
                    "{} declares {} as {declared:?}, so a term at {kind:?} is refused naming it: {outcome:?}",
                    event.as_str(),
                    key.as_str()
                );
                refused += 1;
            }
        }
    }
    assert!(
        admitted > 0 && refused > 0,
        "the sweep read its vocabularies: {admitted} declared pairs, {refused} divergent cells"
    );
}

/// `EX-4`'s other half — a term carrying a key its event does not declare **at
/// all** is refused, at every kind.
///
/// The same predicate as the kind cells above rather than a second rule: a term
/// is admitted when the declaration names its `(key, kind)` pair, and there are
/// two ways to miss. Before this, such a term was silently sorted to the end of
/// the row by `ordered`'s `unwrap_or(usize::MAX)`.
#[test]
fn a_term_carrying_a_key_its_event_does_not_declare_is_refused() {
    let vocabulary = declared_keys();
    let mut cells = 0_usize;
    for event in ChangeEvent::EMITTABLE {
        let own: BTreeSet<PayloadKey> = event.payload_terms().iter().map(|(key, _)| *key).collect();
        for key in vocabulary.difference(&own) {
            for kind in ValueKind::ALL {
                let outcome = event.shaped(vec![term_at(*key, kind)]);
                assert!(
                    matches!(
                        outcome,
                        Err(Refusal::UndeclaredTerm { key: refused_key, .. })
                            if refused_key == *key
                    ),
                    "{} declares no {} term, so one at {kind:?} is refused naming it: {outcome:?}",
                    event.as_str(),
                    key.as_str()
                );
                cells += 1;
            }
        }
    }
    assert!(cells > 0, "the sweep found keys to withhold from an event");
}

/// A subset of the declared shape is legitimate and live — `ReviewDisposed`
/// declares a reason its dispositions do not always carry — so the check asks
/// whether every term is declared, never whether every declaration is termed.
#[test]
fn a_row_carrying_fewer_terms_than_its_event_declares_is_admitted() {
    for event in ChangeEvent::EMITTABLE {
        let shape = event.payload_terms();
        let Some((key, kind)) = shape.first() else {
            continue;
        };
        assert!(
            event.shaped(vec![term_at(*key, *kind)]).is_ok(),
            "{} admits a row carrying only its first declared term",
            event.as_str()
        );
    }
}

/// `ValueKind` now spells its vocabulary twice — once for serde, once for the
/// refusals that name it — so the two are held together here rather than left
/// to stay aligned by attention.
///
/// This is the drift `ChangeEvent`'s hand-written `Serialize`/`Deserialize`
/// impls were written to end: a variant renamed moves serde's token while
/// `as_str`'s stays, and both halves still compile. `ValueKind` rides
/// `rename_all` like the rest of the leaf's closed vocabularies, so it gets the
/// guard instead of the impls.
#[test]
fn value_kind_tokens_match_their_serde_spelling() {
    for kind in ValueKind::ALL {
        assert_eq!(
            serde_json::to_string(&kind).expect("a fieldless enum serialises"),
            format!("\"{}\"", kind.as_str()),
            "{kind:?} spells itself the same way twice"
        );
    }
}

// ---------------------------------------------------------------------------
// SL-261 VT-2 — the retired-key roster's invariants (DEC-278, design sec-4).
// Each pin is a pure function of a roster, so it is proved on `RETIRED_KEYS`
// and shown to bite on an injected bad row.
// ---------------------------------------------------------------------------

/// Rows whose key is still admitted by its owner — or whose owner is an enum.
///
/// An enum owner is refused as out of the roster's checked domain, not because
/// it has no keys: its variants' keys reach `walk_keys` with the enum as owner,
/// so the walk would honour such a row, but this pin cannot yet tell a key live
/// on one variant from one retired from another, and the renderer lists
/// retired rows under struct blocks only (`ISS-478`, `RV-375` `F-2`).
fn live_retirements(roster: &[RetiredKey]) -> Vec<&'static str> {
    roster
        .iter()
        .filter(|row| match row.owner.form {
            TypeForm::Struct { keys, .. } => keys.iter().any(|live| live.key == row.key),
            TypeForm::Enum { .. } => true,
        })
        .map(|row| row.key)
        .collect()
}

/// Rows whose owner the closure rooted at `PAYLOAD` does not reach, by identity.
fn unreachable_owners(roster: &[RetiredKey]) -> Vec<&'static str> {
    let closure = closure_types(&PAYLOAD);
    roster
        .iter()
        .filter(|row| {
            !closure
                .iter()
                .any(|reached| std::ptr::eq(*reached, row.owner))
        })
        .map(|row| row.key)
        .collect()
}

/// Rows with nothing to tell a caller.
fn empty_remedies(roster: &[RetiredKey]) -> Vec<&'static str> {
    roster
        .iter()
        .filter(|row| row.remedy.trim().is_empty())
        .map(|row| row.key)
        .collect()
}

/// `CreateRecord`'s name and form at a different address — a contract the
/// closure does not hold, whatever it is called.
static CREATE_RECORD_TWIN: TypeContract = TypeContract {
    name: CREATE_RECORD.name,
    form: CREATE_RECORD.form,
};

#[test]
fn retired_keys_are_never_live() {
    assert_eq!(live_retirements(RETIRED_KEYS), Vec::<&str>::new());
    static BAD: &[RetiredKey] = &[
        RetiredKey {
            owner: &CREATE_RECORD,
            key: "title",
            remedy: "still admitted",
        },
        RetiredKey {
            owner: &STAGE,
            key: "to",
            remedy: "an enum owner is out of domain",
        },
        RetiredKey {
            owner: &CREATE_RECORD,
            key: "titel",
            remedy: "a sound row",
        },
    ];
    assert_eq!(live_retirements(BAD), ["title", "to"]);
}

#[test]
fn retired_key_owners_are_reachable() {
    assert_eq!(unreachable_owners(RETIRED_KEYS), Vec::<&str>::new());
    static STRAY: TypeContract = TypeContract {
        name: "Stray",
        form: TypeForm::Struct {
            unknown_keys: UnknownKeys::Refused,
            keys: &[],
        },
    };
    static BAD: &[RetiredKey] = &[
        RetiredKey {
            owner: &STRAY,
            key: "stray",
            remedy: "outside the closure",
        },
        RetiredKey {
            owner: &CREATE_RECORD_TWIN,
            key: "twin",
            remedy: "same name, not the node",
        },
        RetiredKey {
            owner: &CREATE_RECORD,
            key: "titel",
            remedy: "a sound row",
        },
    ];
    assert_eq!(unreachable_owners(BAD), ["stray", "twin"]);
}

#[test]
fn retired_key_remedies_are_non_empty() {
    assert_eq!(empty_remedies(RETIRED_KEYS), Vec::<&str>::new());
    static BAD: &[RetiredKey] = &[
        RetiredKey {
            owner: &CREATE_RECORD,
            key: "blank",
            remedy: "  ",
        },
        RetiredKey {
            owner: &CREATE_RECORD,
            key: "titel",
            remedy: "a sound row",
        },
    ];
    assert_eq!(empty_remedies(BAD), ["blank"]);
}

// ---------------------------------------------------------------------------
// SL-261 VT-2 — the crossing, not the request, decides whether a run adopts.
// ---------------------------------------------------------------------------

/// A run holding `sec-1`, a document that re-words it, and a request carrying
/// nothing but the envelope — every input an adoption would read, so only the
/// crossing can tell the two legs apart.
fn adoption_inputs() -> (DesignSnapshot, ApplyRequest, DerivedInput) {
    let prior = run_holding(&[("sec-1", "sha256:held")]);
    let request: ApplyRequest = serde_json::from_value(serde_json::json!({
        "run_uid": prior.run.uid,
        "known_revision": prior.run.revision,
        "submission_id": "s1",
    }))
    .expect("the fixture is a well-formed request");
    let derived = DerivedInput {
        authored_sections: [(
            id("sec-1"),
            AuthoredSection {
                position: 0,
                body: "## sec-1\n\nedited by hand\n".to_owned(),
                fingerprint: Fingerprint::new("sha256:edited"),
            },
        )]
        .into(),
        gate: GateFacts {
            authored_fingerprint: Some(Fingerprint::new("sha256:document")),
            ..GateFacts::default()
        },
        ..DerivedInput::default()
    };
    (prior, request, derived)
}

#[test]
fn ordinary_crossing_never_reads_authored_sections() {
    let (prior, request, derived) = adoption_inputs();
    let held = |crossing: &Crossing| {
        let applied = apply(
            &prior,
            &request,
            crossing,
            &derived,
            "sha256:pay",
            &Resolution::default(),
        )
        .expect("the submission applies");
        let moved = applied
            .rows
            .iter()
            .any(|row| row.event == ChangeEvent::SectionFingerprintChanged);
        let section = applied.snapshot.sections.find(&id("sec-1")).cloned();
        (section.map(|section| section.fingerprint), moved)
    };

    assert_eq!(
        held(&Crossing::Ordinary),
        (Some(Fingerprint::new("sha256:held")), false),
        "an ordinary crossing leaves the held section alone"
    );
    assert_eq!(
        held(&Crossing::Adopt {
            expect: Some(Fingerprint::new("sha256:document")),
        }),
        (Some(Fingerprint::new("sha256:edited")), true),
        "the control: the same inputs adopt under Crossing::Adopt"
    );
}

// ---------------------------------------------------------------------------
// SL-261 VT-1 — the adopt verb's pure core (`SL-261` `sec-3`).
// ---------------------------------------------------------------------------

/// The fingerprint of the document every case below reads.
const DOCUMENT_FINGERPRINT: &str = "sha256:document";

/// The crossing a caller's `--expect` produces.
fn adopting(expect: Option<&str>) -> Crossing {
    Crossing::Adopt {
        expect: expect.map(Fingerprint::new),
    }
}

/// Run one adoption through the pure core.
fn adopt(
    prior: &DesignSnapshot,
    request: &ApplyRequest,
    crossing: &Crossing,
    derived: &DerivedInput,
) -> Result<Applied, Refusal> {
    apply(
        prior,
        request,
        crossing,
        derived,
        "sha256:pay",
        &Resolution::default(),
    )
}

/// Every row subject of one event, in row order.
fn subjects(applied: &Applied, event: ChangeEvent) -> Vec<DesignId> {
    applied
        .rows
        .iter()
        .filter(|row| row.event == event)
        .filter_map(|row| row.subject.clone())
        .collect()
}

/// A prior holding `held`, and the derived facts of a document whose sections are
/// `document` — `(id, body, fingerprint)` in **document** order. The request is
/// the verb's own: [`ApplyRequest::bare`], so only the crossing can decide whether
/// the run adopts.
fn adoption_case(
    held: &[(&str, &str)],
    document: &[(&str, &str, &str)],
) -> (DesignSnapshot, ApplyRequest, DerivedInput) {
    let prior = run_holding(held);
    let request = ApplyRequest::bare(SubmissionEnvelope {
        run_uid: prior.run.uid.clone(),
        known_revision: prior.run.revision,
        submission_id: "s1".to_owned(),
    });
    let derived = DerivedInput {
        authored_sections: document
            .iter()
            .enumerate()
            .map(|(position, (raw, body, digest))| {
                (
                    id(raw),
                    AuthoredSection {
                        position,
                        body: (*body).to_owned(),
                        fingerprint: Fingerprint::new(*digest),
                    },
                )
            })
            .collect(),
        gate: GateFacts {
            authored_fingerprint: Some(Fingerprint::new(DOCUMENT_FINGERPRINT)),
            ..GateFacts::default()
        },
        ..DerivedInput::default()
    };
    (prior, request, derived)
}

/// `VT-1` — the engine derives the section map from the document, so the verb's
/// request carries no caller map to be complete or exact. A changed body seats and
/// emits `SectionFingerprintChanged`; an unchanged one emits nothing.
#[test]
fn adopt_derives_sections_without_a_caller_map() {
    let (prior, request, derived) = adoption_case(
        &[("sec-1", "sha256:held-1"), ("sec-2", "sha256:held-2")],
        &[
            ("sec-1", "## sec-1\n\nedited by hand\n", "sha256:edited-1"),
            ("sec-2", "## sec-2\n", "sha256:held-2"),
        ],
    );
    let applied = adopt(&prior, &request, &adopting(None), &derived)
        .expect("a bare crossing adopts what the document reads");

    assert_eq!(
        subjects(&applied, ChangeEvent::SectionFingerprintChanged),
        vec![id("sec-1")],
        "only the section whose bytes moved is reported"
    );
    assert_eq!(
        applied
            .snapshot
            .sections
            .find(&id("sec-2"))
            .expect("sec-2 survives")
            .fingerprint
            .as_str(),
        "sha256:held-2",
        "the unchanged section keeps the fingerprint the run held"
    );
}

/// `EX-3` — the pure backstop. A locked run never adopts, whatever else is true
/// of the document; the same inputs adopt at any other stage.
#[test]
fn adopt_refuses_on_a_locked_run() {
    let (prior, request, derived) = adoption_case(
        &[("sec-1", "sha256:held")],
        &[("sec-1", "## sec-1\n\nedited\n", "sha256:edited")],
    );
    let mut locked = prior.clone();
    locked.run.stage = Stage::Locked;

    assert_eq!(
        adopt(&locked, &request, &adopting(None), &derived),
        Err(Refusal::AdoptionLocked),
        "the backstop refuses before the document is even classified"
    );
    assert_eq!(
        adopt(&prior, &request, &adopting(None), &derived)
            .expect("the control: the same inputs adopt at an unlocked stage")
            .snapshot
            .sections
            .find(&id("sec-1"))
            .expect("sec-1 survives")
            .fingerprint
            .as_str(),
        "sha256:edited"
    );
}

/// `EX-4` — an `--expect` naming a fingerprint the document does not have is
/// stale, and the refusal carries both values so the caller can name what moved.
#[test]
fn adopt_with_mismatched_expect_is_stale() {
    let (prior, request, derived) = adoption_case(
        &[("sec-1", "sha256:held")],
        &[("sec-1", "## sec-1\n\nedited\n", "sha256:edited")],
    );

    assert_eq!(
        adopt(
            &prior,
            &request,
            &adopting(Some("sha256:reviewed")),
            &derived
        ),
        Err(Refusal::AdoptionStale {
            expected: Some("sha256:reviewed".to_owned()),
            observed: Some(DOCUMENT_FINGERPRINT.to_owned()),
        })
    );
}

/// `EX-4` — an absent document has nothing to adopt, with or without `--expect`.
#[test]
fn adopt_with_absent_document_is_stale() {
    let (prior, request, mut derived) = adoption_case(
        &[("sec-1", "sha256:held")],
        &[("sec-1", "## sec-1\n\nedited\n", "sha256:edited")],
    );
    derived.authored_sections.clear();
    derived.gate.authored_fingerprint = None;

    for expect in [None, Some("sha256:reviewed")] {
        assert_eq!(
            adopt(&prior, &request, &adopting(expect), &derived),
            Err(Refusal::AdoptionStale {
                expected: expect.map(str::to_owned),
                observed: None,
            }),
            "an absent document is stale however the caller named its basis"
        );
    }
}

/// `EX-4` — without `--expect` the basis is the fingerprint read at entry, and
/// naming that same fingerprint explicitly is admitted identically.
#[test]
fn adopt_without_expect_takes_the_observed_fingerprint() {
    let (prior, request, derived) = adoption_case(
        &[("sec-1", "sha256:held")],
        &[("sec-1", "## sec-1\n\nedited\n", "sha256:edited")],
    );

    for crossing in [adopting(None), adopting(Some(DOCUMENT_FINGERPRINT))] {
        assert_eq!(
            adopt(&prior, &request, &crossing, &derived)
                .expect("the observed fingerprint is the admitted basis")
                .snapshot
                .sections
                .find(&id("sec-1"))
                .expect("sec-1 survives")
                .fingerprint
                .as_str(),
            "sha256:edited"
        );
    }
}

/// `DEC-066` — invalidation is coverage-driven, so evidence bound to the changed
/// section dies and evidence bound to an unchanged one outlives the adoption.
#[test]
fn adopt_invalidates_evidence_on_changed_sections_only() {
    let (mut prior, request, derived) = adoption_case(
        &[("sec-1", "sha256:held-1"), ("sec-2", "sha256:held-2")],
        &[
            ("sec-1", "## sec-1\n\nedited\n", "sha256:edited-1"),
            ("sec-2", "## sec-2\n", "sha256:held-2"),
        ],
    );
    // One act covering BOTH sections, and one attestation per section. The act's
    // coverage moved with sec-1, so it dies; `att-2` covered content that did not.
    let mut act = checkpoint_act("cpa-1", ActKind::SectionReviewed, "both sections were read");
    act.covered = Some(CoveredSet::Sections(ContentCoverage::of(
        prior.sections.fingerprints(),
    )));
    prior.acts.record(act);
    attest(&mut prior, "att-1", "sec-1", Reviewer::Adversarial);
    attest(&mut prior, "att-2", "sec-2", Reviewer::Adversarial);

    let applied = adopt(&prior, &request, &adopting(None), &derived).expect("the document adopts");

    assert_eq!(
        subjects(&applied, ChangeEvent::ActInvalidated),
        vec![id("cpa-1")],
        "the act's covered map moved"
    );
    assert_eq!(
        subjects(&applied, ChangeEvent::ReviewInvalidated),
        vec![id("att-1")],
        "sec-2's attestation is bound to content that did not move, so it lives"
    );
}

// ---------------------------------------------------------------------------
// SL-264 VT-1/VT-2 — `needs: null` clears through the one set-difference path
// `needs: []` already uses (design sec-4), closing `ISS-481`.
// ---------------------------------------------------------------------------

/// A run holding `inq-1`, which needs `inq-2` and `inq-3`: two edges, so a
/// clearing assertion cannot pass on a single row.
fn run_with_two_needs_edges() -> DesignSnapshot {
    let mut snapshot = run_holding(&[]);
    for raw in ["inq-2", "inq-3"] {
        snapshot
            .map
            .inquiry
            .insert(InquiryNode::open(
                id(raw),
                format!("is {raw} settled?"),
                Provenance::AgentProposed,
                Some(false),
            ))
            .expect("the fixture node seats");
    }
    snapshot
        .map
        .inquiry
        .insert(
            InquiryNode::open(
                id("inq-1"),
                "what governs this?",
                Provenance::UserDirected,
                Some(false),
            )
            .needing(id("inq-2"))
            .needing(id("inq-3")),
        )
        .expect("the dependant seats");
    snapshot
}

/// Apply one declaration through the pure core over `prior`.
fn apply_declaration(prior: &DesignSnapshot, json: &str) -> Applied {
    apply(
        prior,
        &ApplyRequest {
            declare: vec![declared(json)],
            ..ApplyRequest::bare(SubmissionEnvelope {
                run_uid: prior.run.uid.clone(),
                known_revision: prior.run.revision,
                submission_id: "s1".to_owned(),
            })
        },
        &Crossing::Ordinary,
        &DerivedInput::default(),
        "sha256:pay",
        &Resolution::default(),
    )
    .expect("the fixture declaration applies")
}

/// The `From`/`To` token pair on each `NeedsRemoved` row, in row order.
fn needs_removed_edges(applied: &Applied) -> Vec<(String, String)> {
    applied
        .rows
        .iter()
        .filter(|row| row.event == ChangeEvent::NeedsRemoved)
        .map(|row| {
            let term = |key| {
                row.terms
                    .iter()
                    .find(|term| term.key() == key)
                    .expect("the row states the term")
                    .value()
                    .to_owned()
            };
            (term(PayloadKey::From), term(PayloadKey::To))
        })
        .collect()
}

/// `needs: null` clears the set and emits one `NeedsRemoved` per removed edge,
/// with the same `From`/`To` terms `needs: []` produces — the two spellings
/// collapse by construction, so the run's rows cannot tell them apart.
#[test]
fn needs_null_clears_and_emits_one_row_per_edge() {
    let prior = run_with_two_needs_edges();
    let applied = apply_declaration(&prior, r#"{"subject": "inq-1", "needs": null}"#);

    assert_eq!(
        needs_removed_edges(&applied),
        vec![
            ("inq-1".to_owned(), "inq-2".to_owned()),
            ("inq-1".to_owned(), "inq-3".to_owned()),
        ],
        "one `needs_removed` per removed edge, term for term"
    );
    assert!(
        applied
            .snapshot
            .map
            .inquiry
            .get(&id("inq-1"))
            .expect("the node survives the clearing")
            .needs()
            .is_empty(),
        "`needs: null` clears the stored set"
    );
}

/// An edge-free `needs: null` records no mutation and emits no rows — the run's
/// row obligation is for a mutation it *records* (`REQ-478`), and a `null` that
/// removed nothing removed nothing.
///
/// The empty row list alone cannot fail before the fix: a `null` that removes
/// nothing is the identity under both the bug and the fix, so the edge-free case
/// is unobservable by construction. The control below is what makes the silence
/// mean something — the same spelling *is* honoured where there is an edge to
/// remove, so the no-op's quiet is a no-op and not a key the engine swallowed.
#[test]
fn needs_null_on_an_edge_free_node_records_no_mutation() {
    let mut prior = run_holding(&[]);
    prior
        .map
        .inquiry
        .insert(InquiryNode::open(
            id("inq-1"),
            "what governs this?",
            Provenance::UserDirected,
            Some(false),
        ))
        .expect("the fixture node seats");

    let applied = apply_declaration(&prior, r#"{"subject": "inq-1", "needs": null}"#);

    assert!(
        applied.rows.is_empty(),
        "an edge-free `null` records no mutation, so it owes no rows: {:?}",
        applied.rows
    );
    assert!(
        applied
            .snapshot
            .map
            .inquiry
            .get(&id("inq-1"))
            .expect("the node survives the no-op")
            .needs()
            .is_empty()
    );

    // The control: the same spelling is honoured where there is an edge to
    // remove, so the silence above is a genuine no-op rather than the defect.
    let cleared = apply_declaration(
        &run_with_two_needs_edges(),
        r#"{"subject": "inq-1", "needs": null}"#,
    );
    assert!(
        !cleared.rows.is_empty(),
        "the control: `needs: null` on a node with edges records the removals"
    );
}

// ---------------------------------------------------------------------------
// SL-264 PHASE-02 — `blocking` is a node attribute with two wire homes
// (design sec-3; `RV-386` `F-9`, `F-10`).
// ---------------------------------------------------------------------------

/// The pure core's answer to one declaration over `prior` — the refusal as well
/// as the application, which [`apply_declaration`] above `expect`s away.
///
/// One constructor for every criterion below, so the four differ in the payload
/// they send and nothing else.
fn declare_over(prior: &DesignSnapshot, json: &str) -> Result<Applied, Refusal> {
    apply(
        prior,
        &ApplyRequest {
            declare: vec![declared(json)],
            ..ApplyRequest::bare(SubmissionEnvelope {
                run_uid: prior.run.uid.clone(),
                known_revision: prior.run.revision,
                submission_id: "s1".to_owned(),
            })
        },
        &Crossing::Ordinary,
        &DerivedInput::default(),
        "sha256:pay",
        &Resolution::default(),
    )
}

/// The `From`/`To` term pair on `applied`'s first row of `event`.
fn from_and_to(applied: &Applied, event: ChangeEvent) -> (String, String) {
    let row = applied
        .rows
        .iter()
        .find(|row| row.event == event)
        .expect("the row the criterion is about");
    let term = |key| {
        row.terms
            .iter()
            .find(|term| term.key() == key)
            .expect("the row states the term")
            .value()
            .to_owned()
    };
    (term(PayloadKey::From), term(PayloadKey::To))
}

/// `VT-1` — a judgement is required where a node is born, `null` is refused in
/// **either** state, and an omission on an update persists what the node holds.
///
/// Driven through the JSON payload path rather than a constructed
/// [`Declaration`] (`F-9`): the distinction under test — absent versus `null` —
/// is exactly the one serde collapses into `None` for an `Option<bool>`, so a
/// test that built the value itself would be testing the field's Rust type
/// rather than the wire's. `Sparse<bool>` is what keeps them distinct, and this
/// is the criterion that can see the difference.
#[test]
fn blocking_is_required_at_creation_and_refused_as_null() {
    let prior = run_holding(&[]);

    assert_eq!(
        declare_over(&prior, r#"{"subject": "inq-1", "question": "why?"}"#)
            .expect_err("a node born without a judgement is refused"),
        Refusal::BlockingJudgementMissing { id: id("inq-1") },
        "the omission is refused, never defaulted — the engine does not choose \
         a judgement on the caller's behalf"
    );
    assert_eq!(
        declare_over(
            &prior,
            r#"{"subject": "inq-1", "question": "why?", "blocking": null}"#
        )
        .expect_err("`null` is not a judgement"),
        Refusal::BlockingJudgementWithdrawn { id: id("inq-1") },
        "`null` at creation is refused as a withdrawal rather than read as the \
         omission above (SL-259's disease)"
    );

    let judged = declare_over(
        &prior,
        r#"{"subject": "inq-1", "question": "why?", "blocking": true}"#,
    )
    .expect("a creation stating a judgement applies")
    .snapshot;
    assert_eq!(
        judged
            .map
            .inquiry
            .get(&id("inq-1"))
            .expect("the node was created")
            .blocking(),
        Some(true),
        "the judgement the creation stated is the one the node holds"
    );

    assert_eq!(
        declare_over(&judged, r#"{"subject": "inq-1", "blocking": null}"#)
            .expect_err("a held judgement cannot be withdrawn either"),
        Refusal::BlockingJudgementWithdrawn { id: id("inq-1") },
        "a judgement can be changed but not withdrawn, in either state"
    );

    let persisted = declare_over(
        &judged,
        r#"{"subject": "inq-1", "question": "why, exactly?"}"#,
    )
    .expect("an update that says nothing about the judgement applies")
    .snapshot;
    assert_eq!(
        persisted
            .map
            .inquiry
            .get(&id("inq-1"))
            .expect("the node survives the update")
            .blocking(),
        Some(true),
        "omission on an update persists the judgement the node already holds"
    );
    assert_eq!(
        persisted
            .map
            .inquiry
            .get(&id("inq-1"))
            .expect("the node survives the update")
            .question(),
        "why, exactly?",
        "and the update it did carry landed, so the persistence above is not a \
         declaration the engine dropped whole"
    );
}

/// `VT-3` — the key is admitted at an inquiry in **either** state and stays
/// create-only at a finding, which is the home `ISS-327` pinned.
///
/// The three cells are one criterion because the claim is the contrast: one key
/// with two homes, each row pairing a kind with its own state rule. A table that
/// merely widened the finding row would pass the first cell and fail the third.
#[test]
fn blocking_key_is_admitted_at_an_inquiry_and_still_create_only_at_a_finding() {
    let prior = run_holding(&[]);

    let created = declare_over(
        &prior,
        r#"{"subject": "inq-1", "question": "why?", "blocking": false}"#,
    )
    .expect("`blocking` is honoured where an inquiry is created")
    .snapshot;
    assert!(
        declare_over(&created, r#"{"subject": "inq-1", "blocking": true}"#).is_ok(),
        "and where a held inquiry is updated — the inquiry home is the `EitherState` row"
    );

    // The finding home is untouched: create-only, and the state axis still owns
    // the complaint when a held finding is corrected (`DEC-246`).
    let (with_finding, _) = universe(SubjectState::Held, IdKind::Finding);
    assert_eq!(
        declare_over(&with_finding, r#"{"subject": "fnd-1", "blocking": true}"#)
            .expect_err("the finding home keeps its create-only rule"),
        Refusal::InertAtState {
            subject: id("fnd-1"),
            key: "blocking",
            honoured_when: SubjectState::Absent,
        }
    );

    // And a kind no row names is still refused by the kind axis — naming both
    // homes, in table order, so a second row cannot leave the remedy incomplete.
    assert_eq!(
        declare_over(&prior, r#"{"subject": "sec-1", "blocking": true}"#)
            .expect_err("`blocking` means nothing on a section"),
        Refusal::InertKey {
            subject: id("sec-1"),
            key: "blocking",
            honoured_by: vec![IdKind::Finding, IdKind::Inquiry],
            remedy: None,
        }
    );
}

/// `VT-2` — a flip emits exactly one `NodeBlockingChanged` row, carrying the two
/// judgements, and a redeclaration that states the same judgement emits none.
///
/// The creation row is the row that is *absent*, deliberately: `REQ-478` obliges
/// a row for a mutation the run records, and a creation records its judgement
/// inside `node_created` — a second row would say the new node changed what it
/// was born with.
#[test]
fn blocking_flip_emits_one_node_blocking_changed_row() {
    let prior = run_holding(&[]);
    let created = apply_declaration(
        &prior,
        r#"{"subject": "inq-1", "question": "why?", "blocking": false}"#,
    );
    assert!(
        created
            .rows
            .iter()
            .all(|row| row.event != ChangeEvent::NodeBlockingChanged),
        "creation carries its judgement inside `node_created`: {:?}",
        created.rows
    );

    let flipped = apply_declaration(
        &created.snapshot,
        r#"{"subject": "inq-1", "blocking": true}"#,
    );
    let rows: Vec<&ChangeRow> = flipped
        .rows
        .iter()
        .filter(|row| row.event == ChangeEvent::NodeBlockingChanged)
        .collect();
    assert_eq!(rows.len(), 1, "exactly one row for one flip");
    assert_eq!(
        rows[0].subject.clone(),
        Some(id("inq-1")),
        "the row is about the node whose judgement moved"
    );
    assert_eq!(
        from_and_to(&flipped, ChangeEvent::NodeBlockingChanged),
        ("non-blocking".to_owned(), "blocking".to_owned()),
        "both judgements are on the row, so a reader does not have to fetch the \
         run to learn which way it moved"
    );
    assert_eq!(
        flipped
            .snapshot
            .map
            .inquiry
            .get(&id("inq-1"))
            .expect("the node survives the flip")
            .blocking(),
        Some(true),
        "a value replaces the held judgement"
    );

    let unchanged = apply_declaration(
        &flipped.snapshot,
        r#"{"subject": "inq-1", "blocking": true}"#,
    );
    assert!(
        unchanged.rows.is_empty(),
        "redeclaring the judgement it already holds records no mutation: {:?}",
        unchanged.rows
    );
}

/// `EX-1` — `blocking` is a member of the carried material, so a flipped
/// judgement on a node an act covered **is** a change to what that act was given
/// over.
///
/// **The coverage is read back out of its serialised form, never held in the
/// hand.** In production the carried material is a *deserialised* one — an act's
/// `covered` map comes out of the snapshot — so an in-process comparison would
/// pass whatever the field's serde form happens to be and would guard the claim
/// not at all. That is also what gives this criterion its force: the field is in
/// `material()` from the moment it exists, so it cannot stage a natural red, and
/// the control is the serde form (`skip_serializing` in place of
/// `skip_serializing_if`), which this test is the thing that notices.
///
/// The lifecycle arm is the control in the other direction, in the same test so a
/// [`NodeMaterial`] that carried *everything* could not pass it.
#[test]
fn a_blocking_flip_moves_the_carried_material() {
    let judged = |judgement: Option<bool>| {
        InquiryNode::open(
            id("inq-1"),
            "does it block?",
            Provenance::UserDirected,
            judgement,
        )
        .sequenced(0)
    };
    let carried = |map: &InquiryMap| -> CoveredSet {
        let covered = CoveredSet::Nodes(ContentCoverage::of(map.materials()));
        serde_json::from_value(serde_json::to_value(&covered).expect("a coverage serialises"))
            .expect("a coverage reads back")
    };

    let before = map_of(vec![judged(Some(false))]);
    let covered = carried(&before);
    assert!(
        covered
            .moved(&BTreeMap::new(), &before.materials())
            .is_empty(),
        "nothing moved while nothing moved"
    );

    let flipped = map_of(vec![judged(Some(true))]);
    assert_eq!(
        covered.moved(&BTreeMap::new(), &flipped.materials()),
        vec![id("inq-1")],
        "a flipped judgement is a change to what the node is made of"
    );

    let answered = map_of(vec![judged(Some(false)).resolve(Disposition::Created {
        record: "DEC-140".to_owned(),
    })]);
    assert!(
        covered
            .moved(&BTreeMap::new(), &answered.materials())
            .is_empty(),
        "and answering the question is still progress through the graph, not a \
         change to it"
    );
}

/// `VT-4` — every creation path judges (`RV-386` `F-10`): `design start
/// --from-design` seeds its shaping questions and its imported open-question
/// prose `blocking: true`.
///
/// Import precedes every user act, so no act can cover the seeded node and the
/// conservative default costs nothing — while an omission here would silently
/// default every question the *engine* found to *free*. Both seeded paths are
/// asserted, in one test, because the claim is about the paths and not about one
/// of them: a second seeding route added without a judgement would show up here
/// as an unjudged node rather than as green.
#[test]
fn import_seeds_blocking_true_on_every_seeded_node() {
    let document = "## Open Questions\n\n- **OQ-1:** does the import judge?\n";
    let regions: Vec<(super::legacy::Region<'_>, Fingerprint)> = super::legacy::read(document)
        .expect("the fixture document decomposes")
        .into_iter()
        .map(|region| (region, Fingerprint::new("sha256:section")))
        .collect();
    let digests = BTreeMap::from([(3, Fingerprint::new("sha256:headline"))]);
    let shaping = vec![ShapingQuestion {
        record: "QUE-1".to_owned(),
        question: "which shape does the question take?".to_owned(),
    }];

    let mut run = DesignSnapshot::new("dr-test", 233, None);
    import(&mut run, &regions, &shaping, &digests).expect("the fixture imports");

    let seeded: Vec<(Option<bool>, &Provenance)> = run
        .map
        .inquiry
        .nodes()
        .map(|node| (node.blocking(), node.provenance()))
        .collect();
    assert_eq!(
        seeded.len(),
        2,
        "the fixture seeds one shaping question and one imported entry: {seeded:?}"
    );
    assert!(
        seeded.iter().all(|(judged, _)| *judged == Some(true)),
        "every import path seeds the judgement visible, never free: {seeded:?}"
    );
}

/// The payload contract names the same kind for a `Declaration` key as the
/// wire-key table that *refuses* on it.
///
/// Two tables now carry the per-key home — [`Declaration::WIRE_KEYS`], which is
/// exercised behaviourally, and [`DECLARATION`], which is what a caller reads —
/// and only the first can be measured. A set comparison, so a home added on one
/// side and forgotten on the other, or a row dropped, fails here rather than
/// shipping a contract that describes a payload the engine does not accept.
#[test]
fn the_payload_contract_names_every_declaration_keys_home() {
    let TypeForm::Struct { keys, .. } = DECLARATION.form else {
        panic!("a declaration is a struct");
    };
    let home_of = |home: KeyHome| match home {
        KeyHome::At(kind) => Some(kind),
        // `Universal` is the addressing key: carried by every declaration, and
        // inert at no kind — so it has no home to name on either side.
        KeyHome::Universal => None,
    };

    let declared: BTreeSet<(&str, Option<IdKind>)> = keys
        .iter()
        .map(|key| (key.key, key.home.map(home_of).unwrap_or(None)))
        .collect();
    let tabled: BTreeSet<(&str, Option<IdKind>)> = Declaration::WIRE_KEYS
        .iter()
        .map(|&(key, home, ..)| (key, home_of(home)))
        .collect();

    assert_eq!(declared, tabled, "the contract and the table disagree");
    assert_eq!(
        keys.iter().filter(|key| key.key == "blocking").count(),
        2,
        "`blocking` is one key with two homes, each row stating its own"
    );
}
