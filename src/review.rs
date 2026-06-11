// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine review` — the RV adversarial-review ledger kind (SL-040, ADR-007).
//! One generic `facet`-parameterized ledger reviews any subject via the outbound
//! `reviews` edge, coordinated (in later phases) by a turn-based baton.
//!
//! This file follows the `worktree.rs` one-file shape: a **pure core** (this
//! phase — the closed vocabulary enums, the `derived_status` summary, the `can`
//! transition predicate, and the `toml_string`-escaped render fns) and a future
//! **impure shell** (the `entity::Kind` row, verb handlers, baton/lock/cache
//! coordination — PHASE-02+). PHASE-01 ships the pure core only: no engine row,
//! no `main.rs` command surface, no I/O.
//!
//! Every closed vocabulary enum carries an `as_str` render mirror plus a
//! `&[&str]` const array, kept in lockstep by a per-enum drift canary test
//! (the `adr.rs`/`backlog.rs` pattern). The arrays are stood up for the
//! PHASE-02 engine/CLI vocabulary; the suppression is `cfg_attr(not(test), …)`
//! so the canary round-trips (real uses) do not leave it unfulfilled in the
//! test build (mem.pattern.lint.dead-code-expect-vs-cfg-test /
//! mem.pattern.lint.dead-code-self-clearing-leaf).
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "pure-core vocabulary + predicates stood up in SL-040 PHASE-01; first non-test consumers (engine row, verb handlers) land in PHASE-02/03 — self-clearing"
    )
)]

use crate::tomlfmt::toml_string;

// ---------------------------------------------------------------------------
// Closed vocabulary enums (each: `as_str` render mirror + a `&[&str]` known-set,
// lockstep-guarded by a drift canary test).
// ---------------------------------------------------------------------------

/// What a review reviews — the facet (design §5, D-C11). The closed 7-set with
/// **no `drift`** (D-C11 dropped it → the future Drift Ledger kind, IMP-022).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
const FACETS: &[&str] = &[
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
const FINDING_STATUSES: &[&str] = &["open", "answered", "contested", "verified", "withdrawn"];

/// A finding's severity (design §5, raiser-owned, fixed at raise). Only
/// `blocker` gates `/close` (D-C9b).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

/// The `Severity` known-set. Lockstep-guarded by
/// `severity_known_set_matches_variants`.
const SEVERITIES: &[&str] = &["blocker", "major", "minor", "nit"];

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
const ROLES: &[&str] = &["raiser", "responder"];

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
const REVIEW_STATUSES: &[&str] = &["active", "done"];

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

/// The five write verbs that move a finding's status (design §5). `status` and
/// the read/coordination verbs are not transition verbs and are not modelled
/// here — `can` answers "may this verb fire on a finding in `from` for `role`?".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Verb {
    Raise,
    Dispose,
    Verify,
    Contest,
    Withdraw,
}

impl Verb {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Raise => "raise",
            Self::Dispose => "dispose",
            Self::Verify => "verify",
            Self::Contest => "contest",
            Self::Withdraw => "withdraw",
        }
    }

    /// The role a verb statically requires, knowable without a finding (design
    /// §6 responsibility split): raise/verify/contest/withdraw are the raiser's;
    /// dispose is the responder's. The `with_turn` wrapper checks this; the
    /// per-finding `can` check (state-dependent) is the closure's job.
    pub(crate) const fn required_role(self) -> Role {
        match self {
            Self::Raise | Self::Verify | Self::Contest | Self::Withdraw => Role::Raiser,
            Self::Dispose => Role::Responder,
        }
    }
}

// ---------------------------------------------------------------------------
// Transition predicate (design §5, D-C4/D-C5)
// ---------------------------------------------------------------------------

/// Whether `verb` may fire on a finding currently in `from` (or, for `raise`,
/// not yet existing — `None`) when asserted by `role`. Pure and total: the
/// single-owner edge table (design §5), every other combination refused.
///
/// | verb     | from               | role      | → |
/// |----------|--------------------|-----------|---|
/// | raise    | (none)             | raiser    | open |
/// | dispose  | open \| contested  | responder | answered |
/// | verify   | answered           | raiser    | verified (terminal) |
/// | contest  | answered           | raiser    | contested |
/// | withdraw | open \| answered   | raiser    | withdrawn (terminal) |
pub(crate) const fn can(verb: Verb, from: Option<FindingStatus>, role: Role) -> bool {
    // Static role check first — the half `with_turn` also owns; refuse a
    // role/verb mismatch regardless of state.
    if !role_eq(role, verb.required_role()) {
        return false;
    }
    matches!(
        (verb, from),
        (Verb::Raise, None)
            | (
                Verb::Dispose,
                Some(FindingStatus::Open | FindingStatus::Contested)
            )
            | (Verb::Verify | Verb::Contest, Some(FindingStatus::Answered))
            | (
                Verb::Withdraw,
                Some(FindingStatus::Open | FindingStatus::Answered)
            )
    )
}

/// Const-context `Role` equality (the derived `PartialEq` is not `const`).
const fn role_eq(a: Role, b: Role) -> bool {
    matches!(
        (a, b),
        (Role::Raiser, Role::Raiser) | (Role::Responder, Role::Responder)
    )
}

// ---------------------------------------------------------------------------
// Derived status (design §8, D-C8 / D7)
// ---------------------------------------------------------------------------

/// The status carrier `derived_status` reads — the finding's current status is
/// all the summary needs. The full authored `Finding` (with severity/title/…)
/// lands in PHASE-02/03; this keeps the pure core dependency-free.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FindingState {
    pub(crate) status: FindingStatus,
}

/// The review's derived status + summarized turn (design §8, D-C8). Total over
/// the finding-status enum; never stored (computed at `show`/`list`/`status`).
///
/// - empty ⇒ `(Active, Raiser)` — the raiser goes first; empty ≠ done (the
///   SL-009-divergence-proof case).
/// - any `open`/`contested` ⇒ `(Active, Responder)` — work awaits the responder.
/// - else any `answered` ⇒ `(Active, Raiser)` — work awaits the raiser.
/// - all `∈ {verified, withdrawn}` ⇒ `(Done, None)`.
///
/// `await` is a *priority summary* (open/contested wins display), never an
/// exclusive gate — the turn gate is per-finding `can` (D7).
pub(crate) fn derived_status(findings: &[FindingState]) -> (ReviewStatus, Await) {
    if findings.is_empty() {
        return (ReviewStatus::Active, Await::Raiser);
    }
    if findings
        .iter()
        .any(|f| matches!(f.status, FindingStatus::Open | FindingStatus::Contested))
    {
        return (ReviewStatus::Active, Await::Responder);
    }
    if findings.iter().any(|f| f.status == FindingStatus::Answered) {
        return (ReviewStatus::Active, Await::Raiser);
    }
    (ReviewStatus::Done, Await::None)
}

// ---------------------------------------------------------------------------
// Render (pure — user free-text spliced via `toml_string`, design §4/§5)
// ---------------------------------------------------------------------------

/// A finding as raised, for rendering. Raiser-owned fixed fields plus the
/// responder-owned mutable pair; `status` is transition-graph-owned. The
/// authored on-disk shape of a `[[finding]]` (design §5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Finding {
    pub(crate) id: String,
    pub(crate) status: FindingStatus,
    pub(crate) severity: Severity,
    pub(crate) title: String,
    pub(crate) detail: String,
    pub(crate) disposition: Option<String>,
    pub(crate) response: Option<String>,
}

/// Render a single `[[finding]]` TOML block (design §5). Every user free-text
/// field (`title`, `detail`, `disposition`, `response`) is emitted through
/// `toml_string` so a `"`, `\`, newline, or `]` can neither break the document
/// nor inject a key (mem.pattern.render.toml-splice-escape-user-values). The
/// id/status/severity fields are closed vocabularies, rendered bare.
pub(crate) fn render_finding(finding: &Finding) -> String {
    let mut out = String::new();
    out.push_str("[[finding]]\n");
    push_line(&mut out, "id", &toml_string(&finding.id));
    push_line(&mut out, "status", &toml_string(finding.status.as_str()));
    push_line(
        &mut out,
        "severity",
        &toml_string(finding.severity.as_str()),
    );
    push_line(&mut out, "title", &toml_string(&finding.title));
    push_line(&mut out, "detail", &toml_string(&finding.detail));
    if let Some(disposition) = &finding.disposition {
        push_line(&mut out, "disposition", &toml_string(disposition));
    }
    if let Some(response) = &finding.response {
        push_line(&mut out, "response", &toml_string(response));
    }
    out
}

/// Emit a single `key = value` line (no `push`/`format!` of literals — repo
/// clippy bans the noisy forms; this is the sanctioned string-assembly shape).
fn push_line(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = ");
    out.push_str(value);
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- derived_status: total + named cases (VT-1 / VT-2) -------------------

    fn states(statuses: &[FindingStatus]) -> Vec<FindingState> {
        statuses
            .iter()
            .map(|&status| FindingState { status })
            .collect()
    }

    #[test]
    fn derived_status_empty_is_active_raiser() {
        assert_eq!(derived_status(&[]), (ReviewStatus::Active, Await::Raiser));
    }

    #[test]
    fn derived_status_any_open_or_contested_is_active_responder() {
        assert_eq!(
            derived_status(&states(&[FindingStatus::Open])),
            (ReviewStatus::Active, Await::Responder)
        );
        assert_eq!(
            derived_status(&states(&[FindingStatus::Contested])),
            (ReviewStatus::Active, Await::Responder)
        );
        // open + answered ⇒ open wins ⇒ Responder.
        assert_eq!(
            derived_status(&states(&[FindingStatus::Answered, FindingStatus::Open])),
            (ReviewStatus::Active, Await::Responder)
        );
    }

    #[test]
    fn derived_status_answered_and_none_open_is_active_raiser() {
        assert_eq!(
            derived_status(&states(&[FindingStatus::Answered])),
            (ReviewStatus::Active, Await::Raiser)
        );
        // answered + a terminal one, none open ⇒ Raiser.
        assert_eq!(
            derived_status(&states(&[FindingStatus::Answered, FindingStatus::Verified])),
            (ReviewStatus::Active, Await::Raiser)
        );
    }

    #[test]
    fn derived_status_all_terminal_is_done_none() {
        assert_eq!(
            derived_status(&states(&[
                FindingStatus::Verified,
                FindingStatus::Withdrawn
            ])),
            (ReviewStatus::Done, Await::None)
        );
        assert_eq!(
            derived_status(&states(&[FindingStatus::Verified])),
            (ReviewStatus::Done, Await::None)
        );
        assert_eq!(
            derived_status(&states(&[FindingStatus::Withdrawn])),
            (ReviewStatus::Done, Await::None)
        );
    }

    /// VT-1: total over the enum — every combination of up to two statuses
    /// yields a `(ReviewStatus, Await)` without panic or gap.
    #[test]
    fn derived_status_total_over_enum() {
        let all = [
            FindingStatus::Open,
            FindingStatus::Answered,
            FindingStatus::Contested,
            FindingStatus::Verified,
            FindingStatus::Withdrawn,
        ];
        // Singletons and every ordered pair.
        for &a in &all {
            let _single = derived_status(&states(&[a]));
            for &b in &all {
                let (status, awaited) = derived_status(&states(&[a, b]));
                // The invariant the carrier must always hold: Done ⇔ None.
                assert_eq!(
                    status == ReviewStatus::Done,
                    awaited == Await::None,
                    "Done iff await=None for [{}, {}]",
                    a.as_str(),
                    b.as_str()
                );
            }
        }
    }

    // -- can(): single-owner edges (VT-3) -----------------------------------

    #[test]
    fn can_valid_single_owner_edges_pass() {
        use FindingStatus::{Answered, Contested, Open};
        assert!(can(Verb::Raise, None, Role::Raiser));
        assert!(can(Verb::Dispose, Some(Open), Role::Responder));
        assert!(can(Verb::Dispose, Some(Contested), Role::Responder));
        assert!(can(Verb::Verify, Some(Answered), Role::Raiser));
        assert!(can(Verb::Contest, Some(Answered), Role::Raiser));
        assert!(can(Verb::Withdraw, Some(Open), Role::Raiser));
        assert!(can(Verb::Withdraw, Some(Answered), Role::Raiser));
    }

    #[test]
    fn can_wrong_role_refused() {
        use FindingStatus::{Answered, Open};
        // dispose is the responder's; the raiser may not.
        assert!(!can(Verb::Dispose, Some(Open), Role::Raiser));
        // verify is the raiser's; the responder may not.
        assert!(!can(Verb::Verify, Some(Answered), Role::Responder));
        // raise is the raiser's.
        assert!(!can(Verb::Raise, None, Role::Responder));
    }

    #[test]
    fn can_wrong_from_state_refused() {
        use FindingStatus::{Answered, Open, Verified, Withdrawn};
        // dispose only from open|contested.
        assert!(!can(Verb::Dispose, Some(Answered), Role::Responder));
        // verify/contest only from answered.
        assert!(!can(Verb::Verify, Some(Open), Role::Raiser));
        assert!(!can(Verb::Contest, Some(Open), Role::Raiser));
        // withdraw only from open|answered, not contested/terminal.
        assert!(!can(
            Verb::Withdraw,
            Some(FindingStatus::Contested),
            Role::Raiser
        ));
        // nothing fires on a terminal finding.
        assert!(!can(Verb::Verify, Some(Verified), Role::Raiser));
        assert!(!can(Verb::Dispose, Some(Withdrawn), Role::Responder));
        // raise requires a fresh finding (None), never an existing one.
        assert!(!can(Verb::Raise, Some(Open), Role::Raiser));
    }

    // -- enum ↔ array drift canaries (VT-4) ---------------------------------

    #[test]
    fn facet_known_set_matches_variants() {
        let from_variants: Vec<&str> = [
            Facet::Scope,
            Facet::Design,
            Facet::Plan,
            Facet::PhasePlan,
            Facet::Implementation,
            Facet::CodeReview,
            Facet::Reconciliation,
        ]
        .iter()
        .map(|f| f.as_str())
        .collect();
        assert_eq!(from_variants, FACETS.to_vec());
        // D-C11: `drift` is NOT a facet.
        assert!(!FACETS.contains(&"drift"));
        assert_eq!(FACETS.len(), 7);
    }

    #[test]
    fn finding_status_known_set_matches_variants() {
        let from_variants: Vec<&str> = [
            FindingStatus::Open,
            FindingStatus::Answered,
            FindingStatus::Contested,
            FindingStatus::Verified,
            FindingStatus::Withdrawn,
        ]
        .iter()
        .map(|s| s.as_str())
        .collect();
        assert_eq!(from_variants, FINDING_STATUSES.to_vec());
    }

    #[test]
    fn severity_known_set_matches_variants() {
        let from_variants: Vec<&str> = [
            Severity::Blocker,
            Severity::Major,
            Severity::Minor,
            Severity::Nit,
        ]
        .iter()
        .map(|s| s.as_str())
        .collect();
        assert_eq!(from_variants, SEVERITIES.to_vec());
    }

    #[test]
    fn role_known_set_matches_variants() {
        let from_variants: Vec<&str> = [Role::Raiser, Role::Responder]
            .iter()
            .map(|r| r.as_str())
            .collect();
        assert_eq!(from_variants, ROLES.to_vec());
    }

    #[test]
    fn review_status_known_set_matches_variants() {
        let from_variants: Vec<&str> = [ReviewStatus::Active, ReviewStatus::Done]
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(from_variants, REVIEW_STATUSES.to_vec());
    }

    #[test]
    fn await_str_forms() {
        assert_eq!(Await::Raiser.as_str(), "raiser");
        assert_eq!(Await::Responder.as_str(), "responder");
        assert_eq!(Await::None.as_str(), "none");
    }

    #[test]
    fn verb_str_and_required_role() {
        assert_eq!(Verb::Raise.as_str(), "raise");
        assert_eq!(Verb::Dispose.as_str(), "dispose");
        assert_eq!(Verb::Verify.as_str(), "verify");
        assert_eq!(Verb::Contest.as_str(), "contest");
        assert_eq!(Verb::Withdraw.as_str(), "withdraw");
        // Static verb→role (design §6 responsibility split).
        assert_eq!(Verb::Raise.required_role(), Role::Raiser);
        assert_eq!(Verb::Verify.required_role(), Role::Raiser);
        assert_eq!(Verb::Contest.required_role(), Role::Raiser);
        assert_eq!(Verb::Withdraw.required_role(), Role::Raiser);
        assert_eq!(Verb::Dispose.required_role(), Role::Responder);
    }

    // -- is_terminal mirror -------------------------------------------------

    #[test]
    fn finding_status_terminal_set() {
        assert!(FindingStatus::Verified.is_terminal());
        assert!(FindingStatus::Withdrawn.is_terminal());
        assert!(!FindingStatus::Open.is_terminal());
        assert!(!FindingStatus::Answered.is_terminal());
        assert!(!FindingStatus::Contested.is_terminal());
    }

    // -- render escaping (toml_string splice) -------------------------------

    #[test]
    fn render_finding_escapes_hostile_free_text() {
        let finding = Finding {
            id: "F-1".to_owned(),
            status: FindingStatus::Open,
            severity: Severity::Major,
            // A hostile title: a quote, a backslash, a newline, and a `]`.
            title: "a\"b\\c\nd]e".to_owned(),
            detail: "plain".to_owned(),
            disposition: None,
            response: None,
        };
        let rendered = render_finding(&finding);
        // The rendered block must parse back as valid TOML with the value intact
        // — proof the splice did not break the document or inject a key.
        let parsed: toml::Value = toml::from_str(&rendered).unwrap();
        let finding_tbl = parsed["finding"].as_array().unwrap()[0].as_table().unwrap();
        assert_eq!(finding_tbl["title"].as_str().unwrap(), "a\"b\\c\nd]e");
        assert_eq!(finding_tbl["id"].as_str().unwrap(), "F-1");
        assert_eq!(finding_tbl["status"].as_str().unwrap(), "open");
        assert_eq!(finding_tbl["severity"].as_str().unwrap(), "major");
    }

    #[test]
    fn render_finding_emits_responder_fields_when_present() {
        let finding = Finding {
            id: "F-2".to_owned(),
            status: FindingStatus::Answered,
            severity: Severity::Nit,
            title: "t".to_owned(),
            detail: "d".to_owned(),
            disposition: Some("fixed".to_owned()),
            response: Some("done in r\"123".to_owned()),
        };
        let rendered = render_finding(&finding);
        let parsed: toml::Value = toml::from_str(&rendered).unwrap();
        let tbl = parsed["finding"].as_array().unwrap()[0].as_table().unwrap();
        assert_eq!(tbl["disposition"].as_str().unwrap(), "fixed");
        assert_eq!(tbl["response"].as_str().unwrap(), "done in r\"123");
    }
}
