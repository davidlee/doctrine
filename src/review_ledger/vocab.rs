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

    /// Parse an authored finding-status token against the closed 5-set (the
    /// [`Severity::parse`] pattern). There is **no fallback** (SL-268 D15): an
    /// unknown token is an error naming the whole set, and a reader that must go
    /// on reads it through [`Vocab`] instead of guessing a status for it.
    pub(crate) fn parse(s: &str) -> Result<Self, String> {
        match s {
            "open" => Ok(Self::Open),
            "answered" => Ok(Self::Answered),
            "contested" => Ok(Self::Contested),
            "verified" => Ok(Self::Verified),
            "withdrawn" => Ok(Self::Withdrawn),
            other => Err(format!(
                "unknown finding status `{other}` (known: {})",
                FINDING_STATUSES.join(", ")
            )),
        }
    }
}

/// The `FindingStatus` known-set. Lockstep-guarded by
/// `finding_status_known_set_matches_variants`.
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

/// The responder's answer to a finding (design sec-2/sec-4, SL-268 D8): closed
/// on write, open on read (existing ledgers' free-text values are unaffected —
/// only a fresh `dispose`/`amend` write is bound to this vocabulary). The
/// template is [`Severity`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", try_from = "String", into = "String")]
pub(crate) enum Disposition {
    Aligned,
    FixNow,
    DesignWrong,
    FollowUp,
    Tolerated,
}

impl Disposition {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Aligned => "aligned",
            Self::FixNow => "fix-now",
            Self::DesignWrong => "design-wrong",
            Self::FollowUp => "follow-up",
            Self::Tolerated => "tolerated",
        }
    }

    /// Parse a `--disposition` token against the closed 5-set (design sec-2,
    /// SL-268 D8). A `route:` prefix is refused HERE — not merely unknown — so
    /// clap (the CLI) and serde (MCP) both refuse the retired
    /// `--disposition "route:<route> <vocab>"` form through this one function,
    /// pointing the caller at `--route` (design sec-4).
    pub(crate) fn parse(s: &str) -> Result<Self, String> {
        if s.starts_with(LEGACY_ROUTE_PREFIX) {
            return Err(format!(
                "route: is not part of a disposition; pass the route with --route (known routes: {})",
                ROUTES.join(", ")
            ));
        }
        match s {
            "aligned" => Ok(Self::Aligned),
            "fix-now" => Ok(Self::FixNow),
            "design-wrong" => Ok(Self::DesignWrong),
            "follow-up" => Ok(Self::FollowUp),
            "tolerated" => Ok(Self::Tolerated),
            other => Err(format!(
                "unknown disposition `{other}` (known: {})",
                DISPOSITIONS.join(", ")
            )),
        }
    }
}

impl TryFrom<String> for Disposition {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

impl From<Disposition> for String {
    fn from(d: Disposition) -> Self {
        d.as_str().to_owned()
    }
}

/// The `Disposition` known-set. Lockstep-guarded by
/// `disposition_known_set_matches_variants` (`vocab.rs`'s own test module —
/// VT-4).
pub(crate) const DISPOSITIONS: &[&str] = &[
    "aligned",
    "fix-now",
    "design-wrong",
    "follow-up",
    "tolerated",
];

/// Where a dispose/amend answer sends the finding (design sec-2/sec-4, SL-268
/// D1/D8): the `--route` field split out of the retired
/// `--disposition "route:<route> <vocab>"` prose token. Closed on write, open
/// on read, like [`Disposition`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", try_from = "String", into = "String")]
pub(crate) enum Route {
    Review,
    Demonstrate,
    Probe,
    Control,
    Dedupe,
    Refresh,
}

impl Route {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Review => "review",
            Self::Demonstrate => "demonstrate",
            Self::Probe => "probe",
            Self::Control => "control",
            Self::Dedupe => "dedupe",
            Self::Refresh => "refresh",
        }
    }

    /// Parse a `--route` token against the closed 6-set (design sec-2,
    /// SL-268 D8; SL-270 DEC-330 split `owner-fix` into `dedupe` and `refresh`).
    pub(crate) fn parse(s: &str) -> Result<Self, String> {
        match s {
            "review" => Ok(Self::Review),
            "demonstrate" => Ok(Self::Demonstrate),
            "probe" => Ok(Self::Probe),
            "control" => Ok(Self::Control),
            "dedupe" => Ok(Self::Dedupe),
            "refresh" => Ok(Self::Refresh),
            other => Err(format!(
                "unknown route `{other}` (known: {})",
                ROUTES.join(", ")
            )),
        }
    }
}

impl TryFrom<String> for Route {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

impl From<Route> for String {
    fn from(r: Route) -> Self {
        r.as_str().to_owned()
    }
}

/// The retired prose form of a route: the first token of a disposition string
/// (`route:probe fix-now`). Refused on write; still found in older ledgers.
pub(crate) const LEGACY_ROUTE_PREFIX: &str = "route:";

/// The `Route` known-set. Lockstep-guarded by `route_known_set_matches_variants`
/// (`vocab.rs`'s own test module — VT-4).
pub(crate) const ROUTES: &[&str] = &[
    "review",
    "demonstrate",
    "probe",
    "control",
    "dedupe",
    "refresh",
];

// ---------------------------------------------------------------------------
// Fail-safe authored reads (SL-268 D15, DEC-319)
// ---------------------------------------------------------------------------

/// A closed vocabulary an authored ledger field is read against — the bound
/// [`Vocab`] reads through. `parse_known` is the strict parse with the error
/// dropped: the caller keeps the raw string, so nothing is lost by it.
pub(crate) trait ClosedVocab: Copy {
    fn parse_known(raw: &str) -> Option<Self>;
    fn token(self) -> &'static str;
}

impl ClosedVocab for FindingStatus {
    fn parse_known(raw: &str) -> Option<Self> {
        Self::parse(raw).ok()
    }
    fn token(self) -> &'static str {
        self.as_str()
    }
}

impl ClosedVocab for Severity {
    fn parse_known(raw: &str) -> Option<Self> {
        Self::parse(raw).ok()
    }
    fn token(self) -> &'static str {
        self.as_str()
    }
}

/// One authored closed-vocabulary value, read **without a fallback** (SL-268
/// D15): either a known token or the raw string exactly as authored. A reader
/// never substitutes a guessed value — each consumer decides what `Unknown`
/// means for it (fail-safe: non-terminal status, blocker severity) and the read
/// surfaces disclose it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Vocab<T> {
    Known(T),
    Unknown(String),
}

impl<T: ClosedVocab> Vocab<T> {
    /// Read an authored token: `Known` if it is in the vocabulary, else the raw
    /// string verbatim.
    pub(crate) fn read(raw: &str) -> Self {
        T::parse_known(raw).map_or_else(|| Self::Unknown(raw.to_owned()), Self::Known)
    }

    /// The token as rendered: the canonical token when known, the raw authored
    /// string when not — never a substitute.
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Known(t) => t.token(),
            Self::Unknown(raw) => raw,
        }
    }

    /// The known value, or `None` for an out-of-vocabulary read.
    pub(crate) fn known(&self) -> Option<T> {
        match self {
            Self::Known(t) => Some(*t),
            Self::Unknown(_) => None,
        }
    }
}

impl Vocab<FindingStatus> {
    /// Whether this status is *known* terminal. An out-of-vocabulary status is
    /// not (SL-268 D15) — the fail-safe read keeps it holding its review open.
    pub(crate) fn is_known_terminal(&self) -> bool {
        self.known().is_some_and(FindingStatus::is_terminal)
    }
}

/// Serialised as its rendered token — `Unknown` carries its raw string onto the
/// wire, so a JSON/MCP reader sees what the ledger says.
impl<T: ClosedVocab> Serialize for Vocab<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    /// VT-4: the `Disposition` known-set matches the enum's own variants —
    /// PHASE-05's drift canary, in the same shape as `severity_known_set_matches_variants`.
    #[test]
    fn disposition_known_set_matches_variants() {
        let from_variants: Vec<&str> = [
            Disposition::Aligned,
            Disposition::FixNow,
            Disposition::DesignWrong,
            Disposition::FollowUp,
            Disposition::Tolerated,
        ]
        .iter()
        .map(|d| d.as_str())
        .collect();
        assert_eq!(from_variants, DISPOSITIONS.to_vec());
    }

    /// VT-4: the `Route` known-set matches the enum's own variants.
    #[test]
    fn route_known_set_matches_variants() {
        let from_variants: Vec<&str> = [
            Route::Review,
            Route::Demonstrate,
            Route::Probe,
            Route::Control,
            Route::Dedupe,
            Route::Refresh,
        ]
        .iter()
        .map(|r| r.as_str())
        .collect();
        assert_eq!(from_variants, ROUTES.to_vec());
    }

    /// SL-270 VT-1: `Route::parse` accepts exactly the six routes (DEC-330),
    /// round-tripping each through `as_str`.
    #[test]
    fn route_parse_accepts_exactly_the_six() {
        for known in ROUTES {
            assert_eq!(Route::parse(known).map(Route::as_str), Ok(*known));
        }
        assert_eq!(ROUTES.len(), 6);
    }

    /// SL-270 VT-1: the retired `owner-fix` is refused on write, and the
    /// refusal names the six known routes (DEC-330: split into `dedupe` and
    /// `refresh`; a stored one still reads, since reads never parse).
    #[test]
    fn route_parse_refuses_the_retired_owner_fix() {
        let err = Route::parse("owner-fix").unwrap_err();
        assert_eq!(
            err,
            "unknown route `owner-fix` (known: review, demonstrate, probe, control, dedupe, refresh)"
        );
    }

    /// VT-3: `Disposition::parse("route:probe fix-now")` is refused — the
    /// `route:` prefix names `--route`, not an unknown-token message.
    #[test]
    fn disposition_parse_refuses_a_route_prefix() {
        let err = Disposition::parse("route:probe fix-now").unwrap_err();
        assert!(err.contains("--route"), "{err}");
        assert!(err.contains("refresh"), "{err}");
    }
}
