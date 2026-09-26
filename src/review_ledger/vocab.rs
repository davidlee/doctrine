// SPDX-License-Identifier: GPL-3.0-only
//! RV ledger closed vocabularies (SL-268 D4, engine tier): `Facet`, `FindingStatus`,
//! `Severity`, `Role`, `ReviewStatus`, `Await` — each with an `as_str` render
//! mirror and a `&[&str]` known-set kept in lockstep by a drift canary test.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Closed vocabulary enums (each: `as_str` render mirror + a `&[&str]` known-set,
// lockstep-guarded by a drift canary test).
// ---------------------------------------------------------------------------

/// What a review reviews — the facet (design §5, D-C11). The closed 7-set with
/// **no `drift`** (D-C11 dropped it → the future Drift Ledger kind, IMP-022).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", try_from = "String", into = "String")]
pub(crate) enum Facet {
    Scope,
    Design,
    Plan,
    PhasePlan,
    Implementation,
    CodeReview,
    Reconciliation,
}

impl Facet {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Scope => "scope",
            Self::Design => "design",
            Self::Plan => "plan",
            Self::PhasePlan => "phase-plan",
            Self::Implementation => "implementation",
            Self::CodeReview => "code-review",
            Self::Reconciliation => "reconciliation",
        }
    }
}

/// The `Facet` known-set — closed, no `drift` (D-C11). Lockstep-guarded against
/// the enum by `facet_known_set_matches_variants`.
pub(crate) const FACETS: &[&str] = &[
    "scope",
    "design",
    "plan",
    "phase-plan",
    "implementation",
    "code-review",
    "reconciliation",
];

/// A finding's lifecycle status (design §5). Single-owner edges only — never
/// free-edited; `verified`/`withdrawn` are terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FindingStatus {
    Open,
    Answered,
    Contested,
    Verified,
    Withdrawn,
}

impl FindingStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Answered => "answered",
            Self::Contested => "contested",
            Self::Verified => "verified",
            Self::Withdrawn => "withdrawn",
        }
    }

    /// Whether this status is terminal (`verified`/`withdrawn`) — a finding that
    /// no verb moves on. A review is `Done` iff every finding is terminal
    /// (design §8, D-C9a).
    pub(crate) const fn is_terminal(self) -> bool {
        matches!(self, Self::Verified | Self::Withdrawn)
    }
}

/// The `FindingStatus` known-set. Lockstep-guarded by
/// `finding_status_known_set_matches_variants`.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "read only by the review drift canaries (SL-268 D4 split)"
    )
)]
pub(crate) const FINDING_STATUSES: &[&str] =
    &["open", "answered", "contested", "verified", "withdrawn"];

/// A finding's severity (design §5, raiser-owned, fixed at raise). Only
/// `blocker` gates `/close` (D-C9b).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", try_from = "String", into = "String")]
pub(crate) enum Severity {
    Blocker,
    Major,
    Minor,
    Nit,
}

impl Severity {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Blocker => "blocker",
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Nit => "nit",
        }
    }

    /// Parse a `--severity` token against the closed 4-set (the `Facet::parse`
    /// pattern — keeps the pure-core enum clap-free). `blocker` is the only
    /// severity that gates `/close` (D-C9b).
    pub(crate) fn parse(s: &str) -> Result<Self, String> {
        match s {
            "blocker" => Ok(Self::Blocker),
            "major" => Ok(Self::Major),
            "minor" => Ok(Self::Minor),
            "nit" => Ok(Self::Nit),
            other => Err(format!(
                "unknown severity `{other}` (known: {})",
                SEVERITIES.join(", ")
            )),
        }
    }
}

impl TryFrom<String> for Severity {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

impl From<Severity> for String {
    fn from(s: Severity) -> Self {
        s.as_str().to_owned()
    }
}

/// The `Severity` known-set. Lockstep-guarded by
/// `severity_known_set_matches_variants`.
pub(crate) const SEVERITIES: &[&str] = &["blocker", "major", "minor", "nit"];

/// The party asserting a verb (`--as`, design §5). Cooperative role assertion,
/// not security (ADR-007 Negative).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Role {
    Raiser,
    Responder,
}

impl Role {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Raiser => "raiser",
            Self::Responder => "responder",
        }
    }
}

/// The `Role` known-set. Lockstep-guarded by `role_known_set_matches_variants`.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "read only by the review drift canaries (SL-268 D4 split)"
    )
)]
pub(crate) const ROLES: &[&str] = &["raiser", "responder"];

/// A review's derived status (design §8, D-C8) — **never stored**, computed from
/// the findings at read time. Total over the finding-status enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReviewStatus {
    Active,
    Done,
}

impl ReviewStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Done => "done",
        }
    }
}

/// The `ReviewStatus` known-set. Lockstep-guarded by
/// `review_status_known_set_matches_variants`.
pub(crate) const REVIEW_STATUSES: &[&str] = &["active", "done"];

/// Whose turn the review summarizes to (design §8, D-C2 — the baton caches this).
/// A *priority summary* for display/handoff routing, **not** an independent gate
/// (the gate is per-finding `can()`, D7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Await {
    Raiser,
    Responder,
    None,
}

impl Await {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Raiser => "raiser",
            Self::Responder => "responder",
            Self::None => "none",
        }
    }
}

impl Facet {
    /// Parse a `--facet` token against the closed 7-set (the `memory.rs`
    /// `MemoryType::parse` pattern — keeps the pure-core enum free of clap). The
    /// error names every valid facet, mirroring `listing::validate_statuses`.
    pub(crate) fn parse(s: &str) -> Result<Self, String> {
        match s {
            "scope" => Ok(Self::Scope),
            "design" => Ok(Self::Design),
            "plan" => Ok(Self::Plan),
            "phase-plan" => Ok(Self::PhasePlan),
            "implementation" => Ok(Self::Implementation),
            "code-review" => Ok(Self::CodeReview),
            "reconciliation" => Ok(Self::Reconciliation),
            other => Err(format!(
                "unknown facet `{other}` (known: {})",
                FACETS.join(", ")
            )),
        }
    }
}

impl TryFrom<String> for Facet {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

impl From<Facet> for String {
    fn from(f: Facet) -> Self {
        f.as_str().to_owned()
    }
}
