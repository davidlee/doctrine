// SPDX-License-Identifier: GPL-3.0-only
//! SL-233 PHASE-16 — the obligation runbook runner.
//!
//! Design: `.doctrine/slice/233/sketches/runbook-runner.md` revision 3.
//!
//! Seven named behavioural assertions (`VT-1`). Six drive the CLI end-to-end
//! against the ONE shipped runbook; one — [`editing_a_step_definition_makes_its_discharge_stale`]
//! — is **fixture-based over the `#[path]`-included pure model** and says so,
//! because under DEC-102's ruling (b) the only runbook source is the embed, so
//! no black-box test can vary a step definition. It is not waived: the digest
//! binding is the one thing that must not ship unverified.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::items_after_statements,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

mod common;
mod design_fixture;
mod runbook_fixture;

use design_fixture::{DesignRun, SLICE, SLICE_NUMBER, fail, run};
use runbook_fixture::{EXPLORING_STEPS, discharge_body};

/// The pure model, from source. `design_run` is a leaf with crate out-degree
/// zero, so it compiles standalone here exactly as it does in the binary.
#[path = "../src/design_run/mod.rs"]
#[allow(
    dead_code,
    unused_imports,
    reason = "the whole leaf tree is included; no single test exercises all of it"
)]
mod design_run;

use design_run::runbook::{Discharge, DischargeOutcome, Runbook, RunbookKey};
use design_run::snapshot::{self, DesignSnapshot};

// ── driving the run ───────────────────────────────────────────────────────

/// Model-reading and payload helpers, local to this crate by the rule
/// `tests/design_fixture/mod.rs` states: the shared bootstrap stays free of the
/// `design_run` leaf so crates that only drive the CLI need not compile it.
impl DesignRun {
    /// The parsed snapshot.
    fn read(&self) -> DesignSnapshot {
        snapshot::parse(&std::fs::read_to_string(&self.snapshot).unwrap()).unwrap()
    }

    /// A payload carrying the current revision and `submission`, plus `body`'s
    /// top-level keys merged in.
    fn payload(&self, submission: &str, body: &Value) -> String {
        let mut object = json!({
            "run_uid": self.uid,
            "known_revision": self.read().run.revision,
            "submission_id": submission,
        });
        let map = object.as_object_mut().unwrap();
        for (key, value) in body.as_object().unwrap() {
            map.insert(key.clone(), value.clone());
        }
        object.to_string()
    }

    /// Apply a payload, expecting success; returns stdout.
    fn apply(&self, submission: &str, body: &Value) -> String {
        let body = self.payload(submission, body);
        run(
            &self.root,
            &["design", "apply", SLICE, "-p", ".", "--input", &body],
        )
    }

    /// Apply a payload, expecting refusal; returns stderr.
    fn refuse(&self, submission: &str, body: &Value) -> String {
        let body = self.payload(submission, body);
        fail(
            &self.root,
            &["design", "apply", SLICE, "-p", ".", "--input", &body],
        )
    }
}

/// A two-step sequence standing in for a project's own runbook. Not the shipped
/// asset: this test is about what happens when a definition *changes*, which the
/// embed cannot express.
const FIXTURE: &str = r#"mode = "sequence"

[[step]]
id       = "explore.scope"
text     = "Read the slice scope."
required = true

[[step]]
id       = "explore.canon"
text     = "Run /canon."
required = true
"#;

/// The digest convention the shell uses, mirrored so a fixture can bind a
/// discharge the way `design apply` would. The *material* is the pure model's
/// (`Step::material`); only the hash is shell-side, which is exactly the split
/// `EX-18` requires.
fn digests(book: &Runbook) -> BTreeMap<String, String> {
    book.steps()
        .iter()
        .map(|step| {
            (
                step.id().to_owned(),
                common::sha256(step.material().as_bytes()),
            )
        })
        .collect()
}

/// `EX-2` / `EX-18` / `VA-7` — an id solves reference, not equivalence.
///
/// A discharge binds the digest of the step's *definition*. Editing the step's
/// text while keeping its id must make the discharge stale by construction, and
/// restoring the text byte-for-byte must make it live again — the second half is
/// what distinguishes a digest binding from a blunt invalidate-on-any-edit.
#[test]
fn editing_a_step_definition_makes_its_discharge_stale() {
    let book =
        Runbook::parse(RunbookKey::Exploring, FIXTURE).expect("the fixture is a valid runbook");
    let bound = digests(&book);
    let discharge = Discharge::attested(
        RunbookKey::Exploring,
        "explore.canon",
        bound.get("explore.canon").unwrap(),
        7,
    );

    let live = book.standing(std::slice::from_ref(&discharge), &bound, &[]);
    assert!(
        live.stale.is_empty(),
        "a discharge bound to the current definition is not stale: {live:?}"
    );
    assert!(
        !live.outstanding.iter().any(|id| id == "explore.canon"),
        "a live discharge discharges its step: {live:?}"
    );

    // The text mutation sketch §3 names: one sentence becomes a larger one, and
    // the agent attested to the smaller.
    let widened = Runbook::parse(
        RunbookKey::Exploring,
        &FIXTURE.replace(
            "Run /canon.",
            "Run /canon and read every ADR tagged for this surface.",
        ),
    )
    .expect("the widened fixture is still a valid runbook");
    let after = widened.standing(std::slice::from_ref(&discharge), &digests(&widened), &[]);
    assert_eq!(
        after.stale,
        vec!["explore.canon".to_owned()],
        "editing a step's text while keeping its id must surface the discharge as stale"
    );
    assert!(
        after.outstanding.iter().any(|id| id == "explore.canon"),
        "a stale discharge leaves its step outstanding again: {after:?}"
    );

    // Byte-for-byte restoration yields the same digest, so the discharge stands.
    let restored =
        Runbook::parse(RunbookKey::Exploring, FIXTURE).expect("the restored fixture parses");
    let again = restored.standing(std::slice::from_ref(&discharge), &digests(&restored), &[]);
    assert!(
        again.stale.is_empty(),
        "restoring the definition byte-for-byte must restore the discharge, not merely \
         invalidate on any edit: {again:?}"
    );
}

/// `EX-9` — a skip is a *disclosed deviation*, and a disclosure with nothing
/// disclosed is not one.
///
/// Discharge-with-reason exists so a runbook carrying a step its agent cannot
/// satisfy does not wedge the run once the gate blocks on required steps. That
/// only holds while the reason is mandatory: an optional one degrades the escape
/// hatch into a silent skip, which is the outcome it was added to beat.
#[test]
fn a_skip_without_a_reason_is_refused() {
    let designed = DesignRun::start();
    let refusal = designed.refuse(
        "skip-bare",
        &json!({ "discharge": { "step": "explore.scope", "outcome": "skipped" } }),
    );
    assert!(
        refusal.contains("explore.scope") && refusal.contains("reason"),
        "the refusal must name the step and what it wanted, said: {refusal}"
    );

    let blank = designed.refuse(
        "skip-blank",
        &json!({ "discharge": { "step": "explore.scope", "outcome": "skipped", "reason": "   " } }),
    );
    assert!(
        blank.contains("reason"),
        "whitespace is not a disclosure, said: {blank}"
    );
}

/// `EX-7` — progression IS attestation of a NAMED step, and `sequence` means
/// the one at the cursor.
///
/// There is no bare "next": no verb means *move on* with no name attached to
/// why. So a discharge naming any other step is refused — ahead of the cursor
/// (which would be skipping), behind it (which would be re-attesting work the
/// run already holds), or nowhere in the runbook at all.
#[test]
fn a_sequence_refuses_a_discharge_that_is_not_at_the_cursor() {
    let designed = DesignRun::start();

    let ahead = designed.refuse(
        "ahead",
        &json!({ "discharge": { "step": "explore.canon", "outcome": "attested" } }),
    );
    assert!(
        ahead.contains("explore.canon") && ahead.contains("explore.scope"),
        "the refusal names what was offered AND what was expected, so a caller \
         does not discover the cursor one round-trip at a time: {ahead}"
    );

    let unknown = designed.refuse(
        "unknown",
        &json!({ "discharge": { "step": "explore.nonesuch", "outcome": "attested" } }),
    );
    assert!(unknown.contains("explore.scope"), "said: {unknown}");

    designed.apply(
        "first",
        &json!({ "discharge": { "step": "explore.scope", "outcome": "attested" } }),
    );
    let held = designed.read().runbook.discharges;
    assert_eq!(held.len(), 1, "the discharge is persisted: {held:?}");
    assert_eq!(held.first().unwrap().step(), "explore.scope");

    let behind = designed.refuse(
        "behind",
        &json!({ "discharge": { "step": "explore.scope", "outcome": "attested" } }),
    );
    assert!(
        behind.contains("explore.research"),
        "discharging advances the cursor, so the step just discharged is no longer \
         the one expected: {behind}"
    );
}

/// Discharge each named step in order.
fn discharge_through(designed: &DesignRun, steps: &[&str]) {
    for step in steps {
        designed.apply(&format!("d-{step}"), &discharge_body(step));
    }
}

/// `EX-8` — without the clearance clause the runner is decoration and adherence
/// stays voluntary, which is the defect it exists to fix.
///
/// The clause is a THIRD DERIVED INPUT to `gate::advance`, not a new
/// `Condition`: the existing vocabulary is payload-free, its satisfaction test
/// is existential where this needs universal, and `GateNotCleared` carries only
/// conditions so it could not name a step.
#[test]
fn gate_refuses_to_advance_while_a_required_step_is_undischarged() {
    let designed = DesignRun::start();
    let (last, leading) = EXPLORING_STEPS.split_last().unwrap();
    discharge_through(&designed, leading);

    let refusal = designed.refuse("early", &json!({ "stage": { "to": "inquiring" } }));
    assert!(
        refusal.contains(last),
        "the refusal must NAME the undischarged step — `GateNotCleared` carries only \
         `Condition`s and cannot, which is why this is a separate refusal: {refusal}"
    );
    assert_eq!(
        designed.read().run.stage.as_str(),
        "exploring",
        "a refused advance leaves the run where it was"
    );

    // Discharging the last step clears the runbook clause. The edge does not
    // open — its two incumbent conditions are still unclaimed — and that is the
    // point: the refusal must now be about THOSE, not about the runbook.
    discharge_through(&designed, &[last]);
    let after = designed.refuse("late", &json!({ "stage": { "to": "inquiring" } }));
    assert!(
        !after.contains(last) && after.contains("recorded"),
        "once every required step is discharged the runbook clause is silent and the \
         incumbent conditions speak: {after}"
    );
}

/// `EX-10` / `VA-6` — a check that fails refuses the discharge, and what the
/// check *said* is the refusal's detail.
///
/// A verifier whose failure surfaced only as "the check failed" would leave the
/// agent with nothing to act on, which is the outcome running a check at all is
/// meant to beat. `explore.research` is the one shipped step carrying a
/// verifier, and a throwaway tree genuinely has no research baseline — so the
/// failure is the real one, not a staged exit code.
#[test]
fn a_failing_verifier_refuses_the_discharge_and_surfaces_its_output() {
    let designed = DesignRun::start();
    discharge_through(&designed, &["explore.scope"]);

    let refusal = designed.refuse(
        "verify-fails",
        &json!({ "discharge": { "step": "explore.research", "outcome": "attested" } }),
    );
    assert!(
        refusal.contains("explore.research") && refusal.contains("research baseline"),
        "the refusal must carry the verifier's OWN output, not merely that it failed: \
         {refusal}"
    );
    assert!(
        designed
            .read()
            .runbook
            .discharges
            .iter()
            .all(|held| held.step() != "explore.research"),
        "a failed check leaves the step undischarged — otherwise the record would \
         claim work the check says was not done"
    );
}

/// A research baseline the shipped `explore.research` check accepts, and the
/// intent doc whose later appearance drifts it.
///
/// Written directly rather than minted through `doctrine slice research`: that
/// verb confirms a slice ENTITY before minting under it, and a throwaway design
/// tree has none. The recorded hash set is empty because the tree carries no
/// intent docs yet — which is what makes the drift trigger a one-line write.
///
/// One helper owns both spellings, the bar `EX-11(d)` sets for a black-box
/// crate: `crate::research`'s constants are private to a binary-only crate, so
/// zero copies is unreachable and one is the honest floor.
fn stamp_research_baseline(root: &Path) {
    let dir = root
        .join(common::SLICE_DIR)
        .join(SLICE_NUMBER)
        .join("research");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("baseline.toml"),
        format!("slice = \"{SLICE}\"\ndate = \"2026-07-31\"\n[hashes]\n"),
    )
    .unwrap();
}

/// The slice intent doc whose appearance drifts the baseline above.
fn scope_doc(root: &Path) -> PathBuf {
    root.join(common::SLICE_DIR)
        .join(SLICE_NUMBER)
        .join(format!("slice-{SLICE_NUMBER}.md"))
}

/// The rendered line naming `step`.
///
/// Per LINE, not per output: a whole-output `contains("verified")` cannot tell
/// which step the word attached to, so it would pass on a renderer that labelled
/// every discharge identically — the exact collapse `EX-15` forbids.
fn line_naming<'a>(rendered: &'a str, step: &str) -> &'a str {
    rendered
        .lines()
        .find(|line| line.contains(step))
        .unwrap_or_else(|| panic!("no rendered line names `{step}`:\n{rendered}"))
}

/// `EX-15` / `EX-14` / `VA-3` — the runner guarantees sequenced, gated and
/// auditable; it does NOT guarantee verified, and the rendering may not imply it.
///
/// Only *some* steps carry a check (DEC-101 § what this does not claim). A single
/// "discharged" rendering would collapse an agent's word and a check's exit code
/// into one claim the mechanism is not entitled to make. `explore.scope` carries
/// no verifier and `explore.research` is the one that does, so one run exhibits
/// both — and the assertions run per line, in both directions: the attested step
/// must not say verified, and the verified step must.
#[test]
fn an_attested_step_is_never_rendered_as_verified() {
    let designed = DesignRun::start();
    // The baseline the shipped check accepts, so `explore.research` genuinely
    // verifies rather than being skipped the way the shared fixture skips it.
    stamp_research_baseline(&designed.root);

    discharge_through(&designed, &["explore.scope"]);
    designed.apply(
        "research-ok",
        &json!({ "discharge": { "step": "explore.research", "outcome": "attested" } }),
    );

    // The RECORD half of `VA-3`: two discharges, two different outcomes. Both
    // were submitted as `attested`; only the checked one became `verified`.
    let held = designed.read().runbook.discharges;
    let outcome = |step: &str| {
        held.iter()
            .find(|record| record.step() == step)
            .unwrap_or_else(|| panic!("no record for `{step}`: {held:?}"))
            .outcome()
    };
    assert_eq!(outcome("explore.scope"), DischargeOutcome::Attested);
    assert_eq!(outcome("explore.research"), DischargeOutcome::Verified);

    // The ENVELOPE half.
    let rendered = run(&designed.root, &["design", "resume", SLICE, "-p", "."]);

    let scope = line_naming(&rendered, "explore.scope");
    assert!(
        scope.contains("attested"),
        "a step the agent merely attested must say so: {scope}"
    );
    assert!(
        !scope.contains("verified"),
        "a step with NO verifier must never be rendered as verified — no exit code \
         proves an agent read something: {scope}"
    );

    let research = line_naming(&rendered, "explore.research");
    assert!(
        research.contains("verified"),
        "the one step whose check ran and exited zero is the one entitled to the \
         stronger word: {research}"
    );

    // `EX-14` — the current obligation, with its position. Two steps are
    // discharged, so the cursor stands at the third of five.
    let obligation = line_naming(&rendered, "explore.canon");
    assert!(
        obligation.contains("3/5"),
        "the obligation carries its position, so an agent knows how far in it is \
         without counting: {obligation}"
    );
    assert!(
        obligation.contains("Run /canon"),
        "the obligation carries the step's TEXT — an id alone is a reference, not \
         an instruction: {obligation}"
    );
    assert!(
        !rendered.contains("Triage the design surface"),
        "only the CURRENT obligation is carried; rendering the whole runbook is the \
         `set` half `EX-14` defers, and carrying every step's prose is the token \
         regression it forbids:\n{rendered}"
    );
}

/// `EX-11` — a check that passed at discharge and fails now must block the
/// advance.
///
/// Without gate-time re-evaluation the runbook would certify a fact about the
/// world at the moment it was recorded and never again, so a run could carry a
/// stale pass through the gate — which is the adherence theatre the whole
/// clause exists to beat. Nothing about the RUN changes here: the discharge
/// record stands, its digest still binds the definition, and only the world the
/// check reads has moved.
#[test]
fn a_verifier_that_regressed_since_discharge_blocks_the_gate() {
    let designed = DesignRun::start();
    stamp_research_baseline(&designed.root);

    discharge_through(&designed, &["explore.scope"]);
    designed.apply(
        "research-ok",
        &json!({ "discharge": { "step": "explore.research", "outcome": "attested" } }),
    );
    assert!(
        designed
            .read()
            .runbook
            .discharges
            .iter()
            .any(|held| held.step() == "explore.research"
                && held.outcome() == DischargeOutcome::Verified),
        "the premise: the check passed, so the record is `verified` and not merely \
         `attested`"
    );
    discharge_through(
        &designed,
        &["explore.canon", "explore.memory", "explore.triage"],
    );

    let cleared = designed.refuse("pre", &json!({ "stage": { "to": "inquiring" } }));
    assert!(
        !cleared.contains("explore.research"),
        "with every step discharged and its check passing, the runbook clause is silent \
         and the incumbent conditions speak: {cleared}"
    );

    // The world moves under the discharge: an intent doc appears, so the
    // baseline the check reads is no longer current.
    std::fs::write(scope_doc(&designed.root), "scope").unwrap();

    let blocked = designed.refuse("post", &json!({ "stage": { "to": "inquiring" } }));
    assert!(
        blocked.contains("explore.research") && blocked.contains("regressed"),
        "the advance must be blocked by the step whose check regressed, and say that is \
         what happened rather than report it as never done: {blocked}"
    );
    assert_eq!(
        designed.read().run.stage.as_str(),
        "exploring",
        "a refused advance leaves the run where it was"
    );
    assert!(
        designed
            .read()
            .runbook
            .discharges
            .iter()
            .any(|held| held.step() == "explore.research"),
        "the block is DERIVED: the record stands and the gate re-reads the world, rather \
         than erasing what the run holds"
    );
}
