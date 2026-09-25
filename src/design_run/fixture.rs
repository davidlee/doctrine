// SPDX-License-Identifier: GPL-3.0-only
//! Test fixtures shared by the design-run suites.
//!
//! One home rather than a copy per test module: the pure-engine suite
//! ([`super::tests`]) and the storage suite ([`super::snapshot`]) both build runs
//! holding sections and attest them, and two spellings of *a run holding these
//! sections* would drift the moment a field joins [`Section`].
//!
//! Fixtures only — no assertions, and nothing here is reachable outside `cfg(test)`.

use std::collections::BTreeMap;

use super::Stage;
use super::attestation::{
    AcceptanceAttestation, ActKind, ActorClass, AgentAct, AgentActKind, AgentDeclaration,
    Attestation, CheckpointAct, ContentCoverage, CoveredSet, DisposedPass, ReviewDisposition,
    ReviewPass, ReviewRef, Reviewer,
};
use super::bounds::DESIGN_ID_BYTES;
use super::gate::{
    Cause, Condition, Coverage, DerivationRule, EngineSource, ObservedFact, ObservedFacts,
};
use super::ids::{DesignId, Fingerprint};
use super::inquiry::{Disposition, InquiryNode, Provenance};
use super::run::{DerivedInput, GateFacts};
use super::snapshot::{DesignSnapshot, Section};
use super::submission::Declaration;

/// A well-formed run-local id, or a failure naming the bad literal.
pub(super) fn id(raw: &str) -> DesignId {
    DesignId::parse(raw).expect("test fixture id must be well-formed")
}

/// One declaration, as a caller actually sends it.
///
/// Built through serde rather than through builders: the keys under test are the
/// *wire's*, and half of them have no Rust builder because nothing in the tree
/// constructs a declaration that way.
pub(super) fn declared(json: &str) -> Declaration {
    serde_json::from_str(json).expect("the fixture is a well-formed declaration")
}

/// A section at a stated fingerprint.
pub(super) fn section(raw: &str, digest: &str) -> Section {
    Section {
        id: id(raw),
        title: raw.to_owned(),
        body: format!("## {raw}\n"),
        fingerprint: Fingerprint::new(digest),
        seq: 0,
        source_line: None,
    }
}

/// A fresh run holding `sections`, reviewed by nothing.
pub(super) fn run_holding(sections: &[(&str, &str)]) -> DesignSnapshot {
    let mut snapshot = DesignSnapshot::new("dr-test", 233, None);
    for (raw, digest) in sections {
        snapshot.sections.upsert(section(raw, digest));
    }
    snapshot
}

/// A review pass over the sections `snapshot` holds **now**, naming `review`.
///
/// Opened against current content for the same reason [`attest`] binds to it: a
/// test that wants a stale pass opens one here and then moves a section, which is
/// how staleness actually happens.
pub(super) fn pass_over(snapshot: &DesignSnapshot, review: &str) -> ReviewPass {
    ReviewPass::over(
        ReviewRef::new(review),
        ContentCoverage::of(snapshot.sections.fingerprints()),
    )
}

/// Attest `subject` at the fingerprint it carries **now**.
///
/// Binding to current content is what makes a later edit invalidate the
/// attestation through DEC-066 rather than through a second mechanism — so a test
/// that wants a stale attestation records one here and then moves the section,
/// which is how staleness actually happens.
pub(super) fn attest(
    snapshot: &mut DesignSnapshot,
    attestation: &str,
    subject: &str,
    reviewer: Reviewer,
) {
    let subject = id(subject);
    let fingerprint = snapshot
        .sections
        .find(&subject)
        .expect("fixture attests a section the run holds")
        .fingerprint
        .clone();
    snapshot.review.attestations.push(Attestation::bind(
        id(attestation),
        subject,
        fingerprint,
        reviewer,
    ));
}

/// A checkpoint act of `kind`, accepted on `basis` and covering nothing.
///
/// The degenerate `Coverage::Artefact` shape — enough to exercise recording,
/// replacement and the wire. A test that needs coverage, an observed fact, a
/// confirmation or a disposition fills the slot it is about and leaves the rest.
pub(super) fn checkpoint_act(raw: &str, act: ActKind, basis: &str) -> CheckpointAct {
    CheckpointAct {
        id: id(raw),
        act,
        acceptance: AcceptanceAttestation::bind(basis, None, Fingerprint::new("sha256:accepted")),
        covered: None,
        observed: BTreeMap::new(),
        confirms: None,
        disposition: None,
    }
}

/// An agent declaration of the questions it considers blocking.
pub(super) fn blocking_set_declared(raw: &str, blocking: &[&str]) -> AgentDeclaration {
    agent_declaration(
        raw,
        AgentAct::BlockingSetDeclared {
            blocking: blocking.iter().map(|node| id(node)).collect(),
        },
    )
}

/// An agent declaration that drafting may begin.
pub(super) fn drafting_ready(raw: &str) -> AgentDeclaration {
    agent_declaration(raw, AgentAct::DraftingReady)
}

/// The shared shape behind the two above — the fingerprint stands in for the
/// shell-computed claim digest, which no pure test can compute.
fn agent_declaration(raw: &str, act: AgentAct) -> AgentDeclaration {
    AgentDeclaration {
        id: id(raw),
        act,
        basis: format!("fixture declaration {raw}"),
        turn: None,
        covered: None,
        fingerprint: Fingerprint::new(format!("sha256:{raw}")),
    }
}

/// The two sections [`cleared`] holds, so a test naming one does not restate the
/// literal (STD-001).
pub(super) const SECTION_A: &str = "sec-a";
/// The second section, which is what makes *only that section's own act* an
/// assertion with something to distinguish it from *every act*.
pub(super) const SECTION_B: &str = "sec-b";
/// The one inquiry node [`cleared`] declares blocking, and disposes.
pub(super) const BLOCKING_NODE: &str = "inq-1";
/// A second node, declared **not** blocking and left open — the control that
/// keeps `blocking-inquiries-dispositioned` about the declared set.
pub(super) const OPEN_NODE: &str = "inq-2";
/// The `RV` the run's pass is minted over.
pub(super) const PASS: &str = "RV-244";
/// The fingerprint `design.md` stands at, watermark and observation alike.
pub(super) const AUTHORED: &str = "sha256:authored";
/// The governance edge set the run's `GovernanceConfirmed` act was given over.
pub(super) const EDGES: &str = "sha256:edges";

/// A run standing at `reviewing` in which **every** gate condition holds, and the
/// derived input that makes it hold.
///
/// The positive-control fixture the whole gate suite narrows: a test unmakes the
/// one thing it is about and asserts the [`Cause`] that names it, so a passing
/// assertion cannot be a run that was broken for some other reason. Built to
/// clear the *top* edge, which by cumulative reach clears every edge below it —
/// including `drafting-readiness-attested`, which no longer reaches this crossing
/// but is held so a test can take the run back a stage without rebuilding it.
///
/// Everything here is bound to content the run holds **now**, for [`attest`]'s
/// reason: a test that wants staleness moves the content, which is how staleness
/// actually happens.
///
/// [`Cause`]: super::gate::Cause
pub(super) fn cleared() -> (DesignSnapshot, DerivedInput) {
    let mut run = run_holding(&[(SECTION_A, "sha256:a"), (SECTION_B, "sha256:b")]);
    run.run.stage = Stage::Reviewing;
    run.authored.materialised = true;
    run.authored.watermark = Some(Fingerprint::new(AUTHORED));

    // Two nodes, one declared blocking and disposed, one open and undeclared.
    // The second is the control that keeps the engine row about the *declared*
    // set rather than about every question on the map.
    //
    // Each carries the judgement the declaration below gives it, so the fixture
    // is consistent about its own blocking set under both representations: the
    // stored act names `inq-1`, and `inq-1` says so itself (`SL-264` sec-3). An
    // unjudged node would read the same to today's engine — the set still derives
    // from the act — but would model a run predating the attribute, which is not
    // what this fixture is for.
    for (raw, resolved) in [(BLOCKING_NODE, true), (OPEN_NODE, false)] {
        let node = InquiryNode::open(
            id(raw),
            format!("is {raw} settled?"),
            Provenance::AgentProposed,
            Some(resolved),
        );
        let node = if resolved {
            node.resolve(Disposition::RetainedUnresolved {
                note: "settled by fiat, for the fixture".to_owned(),
            })
        } else {
            node
        };
        run.map
            .inquiry
            .insert(node)
            .expect("fixture node is acyclic");
    }

    let sections = ContentCoverage::of(run.sections.fingerprints());
    let materials = run.map.inquiry.materials();
    let nodes = || CoveredSet::Nodes(ContentCoverage::of(materials.clone()));

    // Every section carries the lane its policy requires. `HumanOnly` is the
    // default, so one attestation each.
    for (index, raw) in [SECTION_A, SECTION_B].into_iter().enumerate() {
        attest(&mut run, &format!("att-{index}"), raw, Reviewer::Human);
    }
    run.review.pass = Some(pass_over(&run, PASS));

    // The declaration carries the map its rule binds to; `DraftingReady`'s rule
    // binds to `Artefact`, so that one carries nothing.
    let mut declaration = blocking_set_declared("agd-blocking", &[BLOCKING_NODE]);
    declaration.covered = Some(nodes());
    let confirms = declaration.fingerprint.clone();
    run.declarations.record(declaration);
    run.declarations.record(drafting_ready("agd-ready"));

    let mut governance = checkpoint_act(
        "cpa-gov",
        ActKind::GovernanceConfirmed,
        "the sweep found these",
    );
    governance
        .observed
        .insert(ObservedFact::GovernanceEdges, Fingerprint::new(EDGES));
    let mut graph = checkpoint_act(
        "cpa-graph",
        ActKind::GraphReviewed,
        "the blocking set is right",
    );
    graph.covered = Some(nodes());
    graph.confirms = Some(confirms);
    let mut sufficiency =
        checkpoint_act("cpa-suff", ActKind::SufficiencyAccepted, "enough to draft");
    sufficiency.covered = Some(nodes());
    let mut disposed = checkpoint_act("cpa-disp", ActKind::ReviewDisposed, "the pass is answered");
    disposed.disposition = Some(DisposedPass {
        pass: ReviewRef::new(PASS),
        disposition: ReviewDisposition::Waived {
            reason: "no adversarial pass is available".to_owned(),
        },
    });
    let mut accepted = checkpoint_act("cpa-accept", ActKind::DesignAccepted, "the design is right");
    accepted.covered = Some(CoveredSet::Sections(sections));
    for act in [governance, graph, sufficiency, disposed, accepted] {
        run.acts.record(act);
    }

    let derived = DerivedInput {
        gate: GateFacts {
            authored_fingerprint: Some(Fingerprint::new(AUTHORED)),
            observed_facts: ObservedFacts {
                facts: [(ObservedFact::GovernanceEdges, Fingerprint::new(EDGES))]
                    .into_iter()
                    .collect(),
            },
            ..GateFacts::default()
        },
        ..DerivedInput::default()
    };
    (run, derived)
}

/// One of every [`Cause`] variant, each list-carrying variant holding `members`
/// members of the widest form — the worst case a forward row can render.
pub(super) fn every_cause(members: usize) -> Vec<Cause> {
    let ids = || -> Vec<DesignId> {
        (0..members)
            .map(|n| id(&format!("inq-{n:0>width$}", width = DESIGN_ID_BYTES - 4)))
            .collect()
    };
    let review = || ReviewRef::new("RV-9999");
    vec![
        Cause::ActMissing {
            act: ActKind::SufficiencyAccepted,
            lanes: vec![ActorClass::Adversarial; members],
        },
        Cause::SectionsUnreviewed {
            subjects: ids()
                .into_iter()
                .map(|subject| (subject, ActorClass::Adversarial))
                .collect(),
        },
        Cause::NoSections,
        Cause::CoverageStale {
            act: ActKind::SufficiencyAccepted,
            moved: ids(),
        },
        Cause::ObservedStale {
            act: ActKind::SufficiencyAccepted,
            fact: ObservedFact::GovernanceEdges,
        },
        Cause::ConfirmationStale {
            act: ActKind::SufficiencyAccepted,
            declaration: AgentActKind::BlockingSetDeclared,
        },
        Cause::BlockersUndisposed {
            findings: (0..members).map(|n| format!("F-{n:0>5}")).collect(),
        },
        Cause::PassSuperseded {
            disposed: review(),
            current: review(),
        },
        Cause::ReviewUnavailable { review: review() },
        Cause::InquiriesOpen { nodes: ids() },
        Cause::MaterialisationStale,
    ]
}

/// The widest cause set `satisfied` can report for `condition`, each list at
/// `members`: its arms, followed per derivation rule rather than every variant
/// on every row. Over-approximates only in giving every act the disposition
/// causes, which only `review-disposed` can carry.
pub(super) fn widest_causes(condition: Condition, members: usize) -> Vec<Cause> {
    let every = every_cause(members);
    let pick = |wanted: fn(&Cause) -> bool| -> Vec<Cause> {
        every
            .iter()
            .filter(|cause| wanted(cause))
            .cloned()
            .collect()
    };
    match condition.contract().derivation {
        DerivationRule::Engine(EngineSource::Dispositions) => {
            pick(|cause| matches!(cause, Cause::InquiriesOpen { .. }))
        }
        DerivationRule::Engine(EngineSource::Materialisation) => {
            pick(|cause| matches!(cause, Cause::MaterialisationStale))
        }
        DerivationRule::Attested(rule) if rule.binding.coverage == Coverage::PerSection => {
            pick(|cause| matches!(cause, Cause::SectionsUnreviewed { .. }))
        }
        DerivationRule::Attested(rule) => rule
            .acts
            .iter()
            .flat_map(|required| {
                let mut causes = pick(|cause| matches!(cause, Cause::CoverageStale { .. }));
                for _ in rule.binding.observed {
                    causes.extend(pick(|cause| matches!(cause, Cause::ObservedStale { .. })));
                }
                if required.confirms.is_some() {
                    causes.extend(pick(|cause| {
                        matches!(cause, Cause::ConfirmationStale { .. })
                    }));
                }
                causes.extend(pick(|cause| {
                    matches!(
                        cause,
                        Cause::PassSuperseded { .. } | Cause::BlockersUndisposed { .. }
                    )
                }));
                causes
            })
            .collect(),
    }
}

/// A **stored** snapshot that predates the blocking judgement, frozen as text
/// (`SL-264` sec-6 VT-6, `RV-389` F-8/F-13).
///
/// Written once by [`cleared`]'s shape serialised with every node **unjudged**,
/// then frozen, so it is the bytes a pre-change binary wrote rather than a value
/// today's types build: no node carries `blocking`, the `blocking-set-declared`
/// act names `inq-1` and `inq-4`, and the change log holds the legacy act's
/// `act_recorded` row and a `node_created` row in its two-term shape. `inq-3` joined
/// after the acts, so both map-bound acts were stale by an addition alone before
/// the change; `cpa-graph` also confirms a digest the stored declaration no longer
/// carries (`ConfirmationStale`). Never regenerate it from today's serialiser:
/// the point is that it was not written by it.
pub(super) const LEGACY_SNAPSHOT: &str = r##"schema = "doctrine.design-run"
version = 1

[run]
uid = "dr-test"
slice = 233
revision = 1
stage = "reviewing"
review_policy = "human-only"

[receipts]
floor = 1
receipt = []

[map]
next_seq = 4

[map.inquiry.nodes.inq-1]
id = "inq-1"
question = "is inq-1 settled?"
lifecycle = "resolved"
needs = []
seq = 0

[map.inquiry.nodes.inq-1.provenance]
provenance = "agent-proposed"

[map.inquiry.nodes.inq-1.disposition]
disposition = "retained-unresolved"
note = "settled by fiat, for the fixture"

[map.inquiry.nodes.inq-2]
id = "inq-2"
question = "is inq-2 settled?"
lifecycle = "open"
needs = []
seq = 1

[map.inquiry.nodes.inq-2.provenance]
provenance = "agent-proposed"

[map.inquiry.nodes.inq-3]
id = "inq-3"
question = "is inq-3 settled?"
lifecycle = "open"
needs = []
seq = 3

[map.inquiry.nodes.inq-3.provenance]
provenance = "agent-proposed"

[map.inquiry.nodes.inq-4]
id = "inq-4"
question = "is inq-4 settled?"
lifecycle = "open"
needs = []
seq = 2

[map.inquiry.nodes.inq-4.provenance]
provenance = "agent-proposed"

[map.cursor]

[map.posture]
posture = "breadth"
authority = "agent-proposed"

[[sections.section]]
id = "sec-a"
title = "sec-a"
body = """
## sec-a
"""
fingerprint = "sha256:a"
seq = 0

[[sections.section]]
id = "sec-b"
title = "sec-b"
body = """
## sec-b
"""
fingerprint = "sha256:b"
seq = 0

[review]
finding = []

[[review.attestation]]
id = "att-0"
subject = "sec-a"
fingerprint = "sha256:a"
reviewer = "human"

[[review.attestation]]
id = "att-1"
subject = "sec-b"
fingerprint = "sha256:b"
reviewer = "human"

[review.pass]
review = "RV-244"

[review.pass.covered.covered]
sec-a = "sha256:a"
sec-b = "sha256:b"

[[acts.act]]
id = "cpa-gov"
act = "governance-confirmed"

[acts.act.acceptance]
authority = "user"
basis = "the sweep found these"
digest = "sha256:accepted"

[acts.act.observed]
governance-edges = "sha256:edges"

[[acts.act]]
id = "cpa-graph"
act = "graph-reviewed"
confirms = "sha256:agd-blocking-relisted"

[acts.act.acceptance]
authority = "user"
basis = "the blocking set is right"
digest = "sha256:accepted"

[acts.act.covered.nodes.covered.inq-1]
question = "is inq-1 settled?"
seq = 0

[acts.act.covered.nodes.covered.inq-1.provenance]
provenance = "agent-proposed"

[acts.act.covered.nodes.covered.inq-2]
question = "is inq-2 settled?"
seq = 1

[acts.act.covered.nodes.covered.inq-2.provenance]
provenance = "agent-proposed"

[acts.act.covered.nodes.covered.inq-4]
question = "is inq-4 settled?"
seq = 2

[acts.act.covered.nodes.covered.inq-4.provenance]
provenance = "agent-proposed"

[[acts.act]]
id = "cpa-suff"
act = "sufficiency-accepted"

[acts.act.acceptance]
authority = "user"
basis = "enough to draft"
digest = "sha256:accepted"

[acts.act.covered.nodes.covered.inq-1]
question = "is inq-1 settled?"
seq = 0

[acts.act.covered.nodes.covered.inq-1.provenance]
provenance = "agent-proposed"

[acts.act.covered.nodes.covered.inq-2]
question = "is inq-2 settled?"
seq = 1

[acts.act.covered.nodes.covered.inq-2.provenance]
provenance = "agent-proposed"

[acts.act.covered.nodes.covered.inq-4]
question = "is inq-4 settled?"
seq = 2

[acts.act.covered.nodes.covered.inq-4.provenance]
provenance = "agent-proposed"

[[acts.act]]
id = "cpa-disp"
act = "review-disposed"

[acts.act.acceptance]
authority = "user"
basis = "the pass is answered"
digest = "sha256:accepted"

[acts.act.disposition]
pass = "RV-244"

[acts.act.disposition.disposition.waived]
reason = "no adversarial pass is available"

[[acts.act]]
id = "cpa-accept"
act = "design-accepted"

[acts.act.acceptance]
authority = "user"
basis = "the design is right"
digest = "sha256:accepted"

[acts.act.covered.sections.covered]
sec-a = "sha256:a"
sec-b = "sha256:b"

[[declarations.declaration]]
id = "agd-blocking"
basis = "fixture declaration agd-blocking"
fingerprint = "sha256:agd-blocking"

[declarations.declaration.act.blocking-set-declared]
blocking = ["inq-1", "inq-4"]

[declarations.declaration.covered.nodes.covered.inq-1]
question = "is inq-1 settled?"
seq = 0

[declarations.declaration.covered.nodes.covered.inq-1.provenance]
provenance = "agent-proposed"

[declarations.declaration.covered.nodes.covered.inq-2]
question = "is inq-2 settled?"
seq = 1

[declarations.declaration.covered.nodes.covered.inq-2.provenance]
provenance = "agent-proposed"

[declarations.declaration.covered.nodes.covered.inq-4]
question = "is inq-4 settled?"
seq = 2

[declarations.declaration.covered.nodes.covered.inq-4.provenance]
provenance = "agent-proposed"

[[declarations.declaration]]
id = "agd-ready"
act = "drafting-ready"
basis = "fixture declaration agd-ready"
fingerprint = "sha256:agd-ready"

[delegation]
delegation = []

[fragments]
fragment = []

[runbook]
discharge = []

[checkpoint]
intent = []

[authored]
watermark = "sha256:authored"
materialised = true

[change_log]
floor = 1

[[change_log.row]]
revision = 1
index = 0
event = "node_created"
subject = "inq-3"

[[change_log.row.term]]
key = "provenance"
kind = "label"
value = "agent-proposed"

[[change_log.row]]
revision = 1
index = 1
event = "act_recorded"
subject = "agd-blocking"

[[change_log.row.term]]
key = "act"
kind = "token"
value = "blocking-set-declared"
"##;
