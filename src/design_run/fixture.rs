// SPDX-License-Identifier: GPL-3.0-only
//! Test fixtures shared by the design-run suites.
//!
//! One home rather than a copy per test module: the pure-engine suite
//! ([`super::tests`]) and the storage suite ([`super::snapshot`]) both build runs
//! holding sections and attest them, and two spellings of *a run holding these
//! sections* would drift the moment a field joins [`Section`].
//!
//! Fixtures only — no assertions, and nothing here is reachable outside `cfg(test)`.

use super::attestation::{Attestation, Reviewer};
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
