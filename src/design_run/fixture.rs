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
    AcceptanceAttestation, ActKind, AgentAct, AgentDeclaration, Attestation, CheckpointAct,
    ContentCoverage, CoveredSet, DisposedPass, ReviewDisposition, ReviewPass, ReviewRef, Reviewer,
};
use super::gate::{ObservedFact, ObservedFacts};
use super::ids::{DesignId, Fingerprint};
use super::inquiry::{Disposition, InquiryNode, Provenance};
use super::run::DerivedInput;
use super::snapshot::{DesignSnapshot, Section};

/// A well-formed run-local id, or a failure naming the bad literal.
pub(super) fn id(raw: &str) -> DesignId {
    DesignId::parse(raw).expect("test fixture id must be well-formed")
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
    for (raw, resolved) in [(BLOCKING_NODE, true), (OPEN_NODE, false)] {
        let node = InquiryNode::open(
            id(raw),
            format!("is {raw} settled?"),
            Provenance::AgentProposed,
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
        authored_fingerprint: Some(Fingerprint::new(AUTHORED)),
        observed_facts: ObservedFacts {
            facts: [(ObservedFact::GovernanceEdges, Fingerprint::new(EDGES))]
                .into_iter()
                .collect(),
        },
        ..DerivedInput::default()
    };
    (run, derived)
}
