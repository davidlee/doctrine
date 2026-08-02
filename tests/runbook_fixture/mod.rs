// SPDX-License-Identifier: GPL-3.0-only
//! The shipped obligation runbooks, as the e2e crates have to walk them
//! (SL-233 PHASE-16, extended by PHASE-08).
//!
//! Since the clearance clause landed, a runbook is a REAL guard on the edge it
//! keys, so every fixture that drives a run past that stage discharges it first.
//! That is the behaviour change, not a fixture inconvenience. PHASE-16 shipped
//! one; PHASE-08 gave every forward edge one, so a fixture crossing three edges
//! now walks three lists.
//!
//! **Why its own module.** This is two items of data, and its callers are the
//! crates that cross that edge — not the same set as any existing sibling's.
//! `tests/common/` is declared by ~30 crates and is for genuinely universal
//! helpers; `tests/design_fixture/` is a *bootstrap*, and its includers take it
//! for [`DesignRun`](../design_fixture/struct.DesignRun.html), which the crates
//! needing this one already have their own version of. Putting the steps in
//! either would make some crate pay for something it does not use
//! (`mem.pattern.tests.shared-helper-placement`).
//!
//! **No preconditions.** Unlike `design_fixture`, this module reaches nothing
//! outside itself, so declaring it costs a crate exactly these two items.

/// The steps the shipped `exploring` runbook asset declares, in cursor order —
/// `RunbookKey::Exploring` is what names its store path, and this module does
/// not repeat it. Single-sourced because four crates walk this list, and a
/// private copy per crate is four chances to drift from the asset.
pub(crate) const EXPLORING_STEPS: [&str; 5] = [
    "explore.scope",
    "explore.research",
    "explore.canon",
    "explore.memory",
    "explore.triage",
];

/// The steps guarding the edge out of `inquiring` (SL-233 PHASE-08).
#[allow(
    dead_code,
    reason = "declared by several crates; each walks the edges it crosses"
)]
pub(crate) const INQUIRING_STEPS: [&str; 2] = ["inquire.knowledge", "inquire.scope"];

/// The steps guarding the edge out of `drafting` (SL-233 PHASE-08). One: the
/// only obligation on that edge that completes at it.
#[allow(
    dead_code,
    reason = "declared by several crates; each walks the edges it crosses"
)]
pub(crate) const DRAFTING_STEPS: [&str; 1] = ["draft.selectors"];

/// The steps guarding the edge out of `reviewing` (SL-233 PHASE-08).
#[allow(
    dead_code,
    reason = "declared by several crates; each walks the edges it crosses"
)]
pub(crate) const REVIEWING_STEPS: [&str; 3] = ["review.scope", "review.selectors", "review.passes"];

/// A submission label for discharging `step`, inside the payload label bound.
///
/// A step id is an identity term (32 bytes) and a submission label is a label
/// term (16), so the two are deliberately different strings — `inquire.knowledge`
/// alone overruns the label bound. Keyed on the stage prefix's first two bytes
/// plus the step's own suffix, because the suffix alone collides: three edges
/// each carry a `scope`.
#[allow(
    dead_code,
    reason = "declared by several crates; each walks the edges it crosses"
)]
pub(crate) fn discharge_label(step: &str) -> String {
    let (head, tail) = step.split_once('.').unwrap_or(("step", step));
    format!("{}.{tail}", head.get(..2).unwrap_or(head))
}

/// The `discharge` body clearing one step — the caller merges its own envelope,
/// which differs per crate where these bodies do not.
///
/// `explore.research` is SKIPPED with a reason rather than attested, because it
/// is the one step carrying a verifier and a throwaway tree has no research
/// round for that check to find. A fixture should not depend on a check about
/// the fixture.
pub(crate) fn discharge_body(step: &str) -> serde_json::Value {
    if step == "explore.research" {
        serde_json::json!({ "discharge": {
            "step": step,
            "outcome": "skipped",
            "reason": "a throwaway fixture tree has no research round to be current",
        } })
    } else {
        serde_json::json!({ "discharge": { "step": step, "outcome": "attested" } })
    }
}
