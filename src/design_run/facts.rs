// SPDX-License-Identifier: GPL-3.0-only
//! Mechanically derived facts, injected into the pure core (design §5.1).
//!
//! The core never reads disk, git, or a clock. The shell gathers what it can
//! observe — the current fingerprint of every subject, and the evidence recorded
//! against those subjects — and hands it in as [`DerivedDesignFacts`].
//!
//! Clearance is derived here rather than stored (DEC-066): a condition holds
//! when *live* evidence exists for it, and evidence is live only while the
//! subject's current fingerprint still matches the one it was recorded against.
//! Material change therefore invalidates exactly the evidence bound to what
//! changed, and nothing else.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::gate::Condition;
use super::ids::{DesignId, Fingerprint};

/// One recorded clearance, bound to the content it was cleared against.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Evidence {
    condition: Condition,
    subject: DesignId,
    fingerprint: Fingerprint,
}

impl Evidence {
    /// The condition this evidence clears.
    pub(crate) const fn condition(&self) -> Condition {
        self.condition
    }

    /// The subject the evidence was recorded against.
    pub(crate) const fn subject(&self) -> &DesignId {
        &self.subject
    }

    /// The subject fingerprint at the moment of recording.
    pub(crate) const fn fingerprint(&self) -> &Fingerprint {
        &self.fingerprint
    }
}

/// The injected fact set the pure core reasons over.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DerivedDesignFacts {
    /// Current fingerprint of every subject the shell can observe.
    subjects: BTreeMap<DesignId, Fingerprint>,
    /// Every clearance recorded so far, live or not.
    evidence: Vec<Evidence>,
}

impl DerivedDesignFacts {
    /// Report a subject's current fingerprint.
    #[must_use]
    pub(crate) fn observe(mut self, subject: DesignId, fingerprint: Fingerprint) -> Self {
        self.subjects.insert(subject, fingerprint);
        self
    }

    /// Record a clearance of `condition` against `subject` at `fingerprint`.
    #[must_use]
    pub(crate) fn record(
        mut self,
        condition: Condition,
        subject: DesignId,
        fingerprint: Fingerprint,
    ) -> Self {
        self.evidence.push(Evidence {
            condition,
            subject,
            fingerprint,
        });
        self
    }

    /// Whether the evidence is still bound to current content.
    ///
    /// A subject the shell can no longer observe is treated as changed, not as
    /// unchanged — absence of a fingerprint is not proof the content still
    /// matches.
    pub(crate) fn is_live(&self, evidence: &Evidence) -> bool {
        self.subjects.get(evidence.subject()) == Some(evidence.fingerprint())
    }

    /// Every clearance still bound to current content (DEC-066).
    pub(crate) fn live_evidence(&self) -> impl Iterator<Item = &Evidence> {
        self.evidence
            .iter()
            .filter(|evidence| self.is_live(evidence))
    }

    /// Whether `condition` holds against current content.
    pub(crate) fn satisfies(&self, condition: Condition) -> bool {
        self.live_evidence()
            .any(|evidence| evidence.condition() == condition)
    }
}
