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

use super::attestation::{
    AcceptanceAttestation, ActKind, AgentAct, AgentDeclaration, Attestation, CheckpointAct,
    ContentCoverage, ReviewPass, ReviewRef, Reviewer,
};
use super::ids::{DesignId, Fingerprint};
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
