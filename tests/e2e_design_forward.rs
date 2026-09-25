// SPDX-License-Identifier: GPL-3.0-only
//! SL-262 PHASE-03 — the forward edge end to end, over the built binary
//! (design `sec-8` end-to-end list; `REQ-437` for the growing-run bound).
//!
//! **What this crate is for.** `PHASE-02` proved the derivation and the
//! rendering in the leaf's own suite. This is the other half: the same
//! behaviour driven through `design start` / `design apply` / `design show`, so
//! a disagreement between the pure model and the shell's fact assembly is a
//! failure here rather than an argument in a design review.
//!
//! The pure model is `#[path]`-included rather than imported because the binary
//! is the only other artifact: including the leaf lets the assertions read step
//! text, condition tokens and the envelope's own caps from the code the binary
//! compiles, rather than re-typing them beside the assertion (`STD-001`).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use serde_json::{Value, json};

mod common;
mod design_act;
mod design_fixture;
mod runbook_fixture;

/// The pure model, from source — `design_run` is a leaf with crate out-degree
/// zero, so it compiles standalone here exactly as it does in the binary.
#[path = "../src/design_run/mod.rs"]
#[allow(
    dead_code,
    unused_imports,
    reason = "the whole leaf tree is included; no single test exercises all of it"
)]
mod design_run;

use design_fixture::{DesignRun, SLICE, fail, run};
use design_run::attestation::{ActKind, AgentAct, ReviewDisposition};
use design_run::render::under_test as caps;
use design_run::runbook::{Runbook, RunbookKey};
use design_run::snapshot::{self, DesignSnapshot, Receipt};

// ── the shipped runbooks, from the single source ──────────────────────────

/// The embedded `exploring` asset. Read here rather than re-typing a step's
/// text beside the assertion: the row's contract is "the step with its text",
/// and a second copy of the text is a second thing that can drift (`STD-001`).
const EXPLORING_BOOK: &str = include_str!("../install/design-prompts/exploring.toml");

/// The text of `id` in the embedded `exploring` runbook.
fn exploring_text(id: &str) -> String {
    Runbook::parse(RunbookKey::Exploring, EXPLORING_BOOK)
        .expect("the embedded exploring runbook parses")
        .steps()
        .iter()
        .find(|step| step.id() == id)
        .unwrap_or_else(|| panic!("the exploring runbook declares `{id}`"))
        .text()
        .to_owned()
}

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

    /// The turn envelope as JSON — the same model the prompt renders.
    fn envelope(&self, extra: &[&str]) -> Value {
        serde_json::from_str(&self.show_as("json", extra)).expect("the JSON rendering parses")
    }
}

// ── reading the forward rows off a rendering ──────────────────────────────

/// The forward block of a prompt/resume rendering: the `forward …` header and
/// every row beneath it, up to the next top-level line.
///
/// A helper rather than whole-output `contains`: the assertions are about which
/// row says what, and a bare substring search cannot tell the header from a row
/// and would pass on a rendering that dropped the indentation (`e2e_design_runbook`
/// makes the same argument for `line_naming`).
fn forward_block(rendered: &str) -> Vec<&str> {
    let mut lines = rendered
        .lines()
        .skip_while(|line| !line.starts_with("forward "))
        .peekable();
    let mut block = Vec::new();
    while let Some(line) = lines.next() {
        if block.is_empty() || line.starts_with("  ") {
            block.push(line);
        } else {
            break;
        }
    }
    assert!(!block.is_empty(), "no `forward …` row in:\n{rendered}");
    block
}

/// Discharge each named `exploring` step in order.
fn discharge_exploring(designed: &DesignRun) {
    for step in runbook_fixture::EXPLORING_STEPS {
        designed.apply(
            &runbook_fixture::discharge_label(step),
            &runbook_fixture::discharge_body(step),
        );
    }
}

/// A research baseline the shipped `explore.research` check accepts.
///
/// Written directly rather than minted through `doctrine slice research`: that
/// verb confirms a slice ENTITY before minting under it, and a throwaway design
/// tree has none. One copy of the spelling, mirroring `e2e_design_runbook.rs`'s
/// helper — the constants it would otherwise share are private to that crate.
fn stamp_research_baseline(designed: &DesignRun) {
    let dir = designed
        .root
        .join(common::SLICE_DIR)
        .join(design_fixture::SLICE_NUMBER)
        .join("research");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("baseline.toml"),
        format!("slice = \"{SLICE}\"\ndate = \"2026-07-31\"\n[hashes]\n"),
    )
    .unwrap();
}

/// A started run whose slice record is readable — the precondition a
/// `governance-confirmed` act needs (`ObservedFact::GovernanceEdges` is
/// projected from the slice's own outbound relations, and an unreadable record
/// reads as an unobservable fact, which refuses the act).
fn started_with_record() -> DesignRun {
    let designed = DesignRun::start();
    design_fixture::seed_slice_record(&designed.root, design_fixture::SLICE_NUMBER);
    designed
}

/// The `apply` payload the prompt's forward edge printed, verbatim.
///
/// Stripped from the rendering rather than rebuilt: the test's whole claim is
/// that the printed payload is applicable as shown, so a rebuilt one would pass
/// while the printed one was wrong.
fn printed_payload(designed: &DesignRun) -> String {
    let rendered = designed.show(&[]);
    rendered
        .lines()
        .find_map(|line| line.strip_prefix("  apply "))
        .unwrap_or_else(|| panic!("the forward edge printed no `apply` row:\n{rendered}"))
        .to_owned()
}

/// Bring `exploring→inquiring` to `ready`: runbook discharged, both attested
/// conditions claimed. Returns nothing — the caller reads the edge it produced.
fn crossable_exploring(designed: &DesignRun) {
    discharge_exploring(designed);
    designed.apply(
        "governance",
        &json!({ "checkpoint_act": design_act::checkpoint_act(
            ActKind::GovernanceConfirmed,
            "the governing artefacts are the ones found",
        ) }),
    );
    designed.apply(
        "graph",
        &json!({
            "checkpoint_act": design_act::checkpoint_act(
                ActKind::GraphReviewed,
                "the empty blocking set is right",
            ),
        }),
    );
}

// ── a fresh run ───────────────────────────────────────────────────────────

/// Design `sec-8` — *a fresh run names its first step*.
///
/// The envelope's whole purpose is that the first row is the next act. A fresh
/// run has nothing done, so the runbook's cursor is that act and its text
/// travels with it: an id alone is a reference, not an instruction.
#[test]
fn fresh_run_names_its_first_step() {
    let designed = DesignRun::start();
    let rendered = designed.show(&[]);

    assert!(
        rendered.contains("forward exploring→inquiring blocked"),
        "the edge heads the rows:\n{rendered}"
    );
    let expected = format!(
        "  runbook exploring 1/5 explore.scope — {}",
        exploring_text("explore.scope")
    );
    assert!(
        rendered.lines().any(|line| line == expected),
        "the first row is the cursor step, with its text:\n{rendered}"
    );
    // The steps behind the cursor are named, not narrated: only the cursor
    // carries prose (the token regression `EX-14` forbids).
    assert!(
        rendered.contains("  runbook outstanding explore.research"),
        "every outstanding step is named:\n{rendered}"
    );
    assert!(
        !rendered.contains(&exploring_text("explore.research")),
        "and only the cursor step carries its text:\n{rendered}"
    );
}

/// Design `sec-8` — *a discharged runbook names the conditions*.
///
/// Once the edge's entry ritual is done, the runbook rows fall silent and what
/// remains is the gate's own conditions. A following stage attempt must refuse
/// with that same set — the forward rows and the refusal are two renderings of
/// one derivation, and a caller who acts on the rows must not then discover
/// something else.
#[test]
fn discharged_runbook_names_the_conditions() {
    let designed = DesignRun::start();
    discharge_exploring(&designed);

    let rendered = designed.show(&[]);
    let block = forward_block(&rendered);
    assert_eq!(block.first(), Some(&"forward exploring→inquiring blocked"));
    assert!(
        !block.iter().any(|line| line.contains("runbook exploring")),
        "a cleared runbook renders no runbook row:\n{rendered}"
    );

    let envelope = designed.envelope(&[]);
    let named: Vec<String> = envelope["forward"]["unmet"]
        .as_array()
        .expect("unmet rows")
        .iter()
        .map(|row| row["condition"].as_str().unwrap().to_owned())
        .collect();
    assert!(
        named.contains(&"governing-context-recorded".to_owned())
            && named.contains(&"initial-concerns-recorded".to_owned()),
        "the two edge conditions are named: {named:?}"
    );

    let refusal = designed.refuse("advance", &json!({ "stage": { "to": "inquiring" } }));
    for condition in &named {
        assert!(
            refusal.contains(condition),
            "the refusal names the same condition `{condition}` the rows do: {refusal}"
        );
    }
}

/// Design `sec-8` / `DEC-294` — *exploring discloses its skipped check*.
///
/// `advance` re-runs a step's `verify`; a read does not. So a read that owns a
/// live discharge of a checked step must say it skipped the check — otherwise a
/// green `forward` would claim the check passed for a run whose world has since
/// moved (`STD-003`).
#[test]
fn exploring_discloses_its_skipped_check() {
    let designed = DesignRun::start();
    stamp_research_baseline(&designed);
    designed.apply(
        "explore.scope",
        &runbook_fixture::discharge_body("explore.scope"),
    );
    designed.apply(
        "explore.research",
        &json!({ "discharge": { "step": "explore.research", "outcome": "attested" } }),
    );

    let envelope = designed.envelope(&[]);
    assert_eq!(
        envelope["forward"]["unchecked"],
        json!(["explore.research"]),
        "the one checked step the read did not re-run: {envelope}"
    );
    let rendered = designed.show(&[]);
    assert!(
        rendered.contains(
            "  unchecked explore.research — advance re-runs its check; this read did not"
        ),
        "and the row says so:\n{rendered}"
    );
    // A step with no check is never named: there is no exit code to disclose.
    assert!(
        !envelope["forward"]["unchecked"]
            .as_array()
            .unwrap()
            .iter()
            .any(|id| id == "explore.scope"),
        "a step with no verifier has no check to skip: {envelope}"
    );
}

/// Design `sec-8` / `sec-2` *Rendering* — *JSON carries forward*.
///
/// The JSON rendering is the machine-readable form of the same envelope, so it
/// carries the same rows: version 2, `forward` present, `next_obligation` gone,
/// every unmet row with its remedy, and a complete `ready` payload when nothing
/// blocks.
#[test]
fn json_carries_forward() {
    let designed = DesignRun::start();
    let envelope = designed.envelope(&[]);

    assert_eq!(
        envelope["version"],
        json!(2),
        "envelope version: {envelope}"
    );
    assert!(
        envelope.get("next_obligation").is_none(),
        "the retired key is gone: {envelope}"
    );
    let forward = &envelope["forward"];
    assert_eq!(forward["from"], json!("exploring"));
    assert_eq!(forward["to"], json!("inquiring"));
    assert_eq!(forward["ready"], Value::Null, "nothing crosses yet");
    assert_eq!(forward["diverged"], Value::Null);

    let rows = forward["unmet"].as_array().expect("unmet rows");
    assert_eq!(
        rows.len(),
        2,
        "the edge's two conditions are unmet on a fresh run: {forward}"
    );
    for row in rows {
        assert!(
            row["remedy"]
                .as_str()
                .is_some_and(|remedy| !remedy.is_empty()),
            "every row carries the discharging act: {row}"
        );
    }
    assert!(
        rows.iter()
            .any(|row| row["condition"] == json!("governing-context-recorded")),
        "and names the conditions the gate named: {forward}"
    );
}

/// Design `sec-8` / `DEC-293` — *resume carries forward, not a runbook section*.
///
/// `resume` used to append a `runbook exploring obligation …` block outside the
/// envelope. The forward edge subsumes it, so resume carries the rows and the
/// retired block is gone — one projection cannot carry a fact the envelope lacks.
#[test]
fn resume_carries_forward() {
    let designed = DesignRun::start();
    let rendered = designed.resume(&[]);

    let block = forward_block(&rendered);
    assert_eq!(block.first(), Some(&"forward exploring→inquiring blocked"));
    assert!(
        block.iter().any(|line| line.contains("explore.scope")),
        "the runbook row rides inside forward:\n{rendered}"
    );
    assert!(
        !rendered.contains("runbook exploring obligation"),
        "the retired out-of-envelope block is gone:\n{rendered}"
    );
    // Nothing else renders a runbook row: every line naming the runbook is an
    // indented forward row.
    for line in rendered.lines() {
        if line.contains("runbook exploring") {
            assert!(
                line.starts_with("  "),
                "a runbook row outside forward's indentation:\n{rendered}"
            );
        }
    }
}

// ── the ready edge ────────────────────────────────────────────────────────

/// Seed a retained receipt under `submission`, from the snapshot's own wire
/// type.
///
/// The binary cannot reach this state on its own: a submission sharing the id
/// at the same revision would advance the revision, so the id it minted could
/// never be pre-squatted. The fixture writes the receipt through
/// [`snapshot::to_toml`] rather than splicing TOML, so the seed is the same
/// shape the binary writes (`STD-001`).
fn squat(designed: &DesignRun, submission: &str) {
    let text = std::fs::read_to_string(&designed.snapshot).unwrap();
    let mut run = snapshot::parse(&text).unwrap();
    let revision = run.run.revision;
    run.receipts.receipts.push(Receipt {
        submission: submission.to_owned(),
        revision,
        digest: "sha256:a different submission wearing the minted name".to_owned(),
        delegation: None,
        delegation_state: None,
    });
    std::fs::write(&designed.snapshot, snapshot::to_toml(&run).unwrap()).unwrap();
}

/// Design `sec-8` — *ready is applied as printed*.
///
/// The ready row's contract is that its payload crosses the edge exactly as
/// shown: the `apply` line's JSON, pasted into `design apply --input`, advances
/// the stage, and pasting it again resumes on its submission id rather than
/// moving twice.
#[test]
fn ready_is_applied_as_printed() {
    let designed = started_with_record();
    crossable_exploring(&designed);

    let rendered = designed.show(&[]);
    assert!(
        rendered.contains("forward exploring→inquiring ready"),
        "nothing blocks, so the edge reads ready:\n{rendered}"
    );
    let envelope = designed.envelope(&[]);
    let ready = envelope["forward"]["ready"].clone();
    assert_eq!(ready["run_uid"], json!(designed.uid));
    assert_eq!(ready["known_revision"], json!(designed.read().run.revision));
    assert_eq!(ready["stage"]["to"], json!("inquiring"));
    assert_eq!(ready["stage"]["reason"], Value::Null);
    assert!(
        ready.get("declare").is_none(),
        "only what it sets is serialised: {ready}"
    );

    // The printed row and the envelope's payload are the same bytes.
    let payload = printed_payload(&designed);
    assert_eq!(serde_json::from_str::<Value>(&payload).unwrap(), ready);

    let before = designed.read().run.revision;
    run(
        &designed.root,
        &["design", "apply", SLICE, "-p", ".", "--input", &payload],
    );
    assert_eq!(designed.read().run.stage.as_str(), "inquiring");
    assert_eq!(designed.read().run.revision, before + 1);

    // Re-applying resumes: one receipt, one move.
    run(
        &designed.root,
        &["design", "apply", SLICE, "-p", ".", "--input", &payload],
    );
    assert_eq!(designed.read().run.stage.as_str(), "inquiring");
    assert_eq!(
        designed.read().run.revision,
        before + 1,
        "the resumed submission does not move the run a second time"
    );
}

/// Design `sec-8` — *a squatted id does not break ready*.
///
/// The minted id is chosen clear of retained receipts, so a run whose next id
/// is already taken still carries a payload that crosses: the mint steps past
/// it and the printed payload crosses.
#[test]
fn squatted_id_does_not_break_ready() {
    let designed = started_with_record();
    crossable_exploring(&designed);

    let minted =
        serde_json::from_str::<Value>(&printed_payload(&designed)).unwrap()["submission_id"]
            .as_str()
            .expect("the ready payload names its submission")
            .to_owned();
    squat(&designed, &minted);

    let payload = printed_payload(&designed);
    let id = serde_json::from_str::<Value>(&payload).unwrap()["submission_id"]
        .as_str()
        .expect("the ready payload names its submission")
        .to_owned();
    assert_eq!(
        id,
        format!("{minted}-2"),
        "the mint steps past the retained receipt"
    );
    run(
        &designed.root,
        &["design", "apply", SLICE, "-p", ".", "--input", &payload],
    );
    assert_eq!(designed.read().run.stage.as_str(), "inquiring");
}

// ── an edited document ────────────────────────────────────────────────────

/// Design `sec-8` — *an edited document blocks ready*.
///
/// A hand edit to `design.md` is refused before the gate is asked, so it must
/// head the forward rows too: an edge claiming `ready` on a document the run no
/// longer describes would send a caller straight into the refusal. Its sentence
/// is the refusal's own — one classifier, one sentence.
#[test]
fn edited_document_blocks_ready() {
    let designed = started_with_record();
    crossable_exploring(&designed);
    assert!(
        designed
            .show(&[])
            .contains("forward exploring→inquiring ready"),
        "the premise: the edge is ready before the edit"
    );

    let document = designed
        .root
        .join(common::SLICE_DIR)
        .join(design_fixture::SLICE_NUMBER)
        .join("design.md");
    std::fs::write(&document, "# A design written by hand\n").unwrap();

    let rendered = designed.show(&[]);
    let block = forward_block(&rendered);
    assert_eq!(block.first(), Some(&"forward exploring→inquiring blocked"));
    let diverged = block
        .get(1)
        .and_then(|line| line.strip_prefix("  diverged "))
        .unwrap_or_else(|| panic!("the divergence is the first row:\n{rendered}"));
    assert!(
        !rendered.contains("  apply "),
        "no payload crosses a divergent edge:\n{rendered}"
    );

    // The refusal for a stage submission carries the same sentence.
    let refusal = designed.refuse("advance", &json!({ "stage": { "to": "inquiring" } }));
    assert!(
        refusal.contains(diverged),
        "the refusal and the row are one sentence:\nrow: {diverged}\nrefusal: {refusal}"
    );
    assert_eq!(
        designed.read().run.stage.as_str(),
        "exploring",
        "the refused move left the run where it was"
    );
}

// ── an unobservable fact ──────────────────────────────────────────────────

/// Design `sec-8` — *an unobservable fact renders unmet*.
///
/// `governing-context-recorded` binds the slice's own governance edge set. When
/// that record cannot be read, the fact is **absent**, and absence is refusal —
/// the same fail-closed rule `advance` follows. A read must render it as unmet
/// with the `ObservedStale` cause rather than silently agreeing with a claim
/// made over a fact nobody can see (`STD-003`).
#[test]
fn unobservable_fact_renders_unmet() {
    let designed = started_with_record();
    discharge_exploring(&designed);
    designed.apply(
        "governance",
        &json!({ "checkpoint_act": design_act::checkpoint_act(
            ActKind::GovernanceConfirmed,
            "the governing artefacts are the ones found",
        ) }),
    );
    assert!(
        designed.envelope(&[])["forward"]["ready"] == Value::Null,
        "the premise: the initial-concerns acts are still owed"
    );

    // The record moves out from under the act that was given over it.
    let record = designed
        .root
        .join(common::SLICE_DIR)
        .join(design_fixture::SLICE_NUMBER)
        .join(format!("slice-{}.toml", design_fixture::SLICE_NUMBER));
    std::fs::remove_file(&record).unwrap();

    let envelope = designed.envelope(&[]);
    let row = envelope["forward"]["unmet"]
        .as_array()
        .expect("unmet rows")
        .iter()
        .find(|row| row["condition"] == json!("governing-context-recorded"))
        .unwrap_or_else(|| panic!("the row is still unmet: {envelope}"));
    let observed_stale = row["causes"]
        .as_array()
        .expect("the row's causes")
        .iter()
        .find_map(|cause| cause["cause"].get("observed-stale"))
        .unwrap_or_else(|| panic!("an `ObservedStale` cause, got: {row}"));
    assert_eq!(observed_stale["fact"], json!("governance-edges"));
    assert_eq!(observed_stale["act"], json!("governance-confirmed"));
    assert!(
        row["remedy"]
            .as_str()
            .is_some_and(|remedy| !remedy.is_empty()),
        "and carries the discharging act: {row}"
    );
}

// ── the growing run ───────────────────────────────────────────────────────

/// Discharge every step of one edge's runbook.
fn discharge_edge(designed: &DesignRun, steps: &[&str]) {
    for step in steps {
        designed.apply(
            &runbook_fixture::discharge_label(step),
            &runbook_fixture::discharge_body(step),
        );
    }
}

/// A run driven to `reviewing` with its runbook cleared — crossable, and the
/// one stage whose cumulative condition set carries all four list causes.
fn ladder_to_reviewing() -> DesignRun {
    let designed = started_with_record();
    discharge_exploring(&designed);
    designed.apply(
        "governance",
        &json!({ "checkpoint_act": design_act::checkpoint_act(
            ActKind::GovernanceConfirmed,
            "the governing artefacts are the ones found",
        ) }),
    );
    designed.apply(
        "graph",
        &json!({
            "checkpoint_act": design_act::checkpoint_act(
                ActKind::GraphReviewed,
                "the empty blocking set is right",
            ),
        }),
    );
    designed.apply("to-inquiring", &json!({ "stage": { "to": "inquiring" } }));
    discharge_edge(&designed, &runbook_fixture::INQUIRING_STEPS);
    designed.apply(
        "sufficiency",
        &json!({ "checkpoint_act": design_act::checkpoint_act(
            ActKind::SufficiencyAccepted,
            "there is nothing outstanding to interrogate",
        ) }),
    );
    designed.apply("to-drafting", &json!({ "stage": { "to": "drafting" } }));
    discharge_edge(&designed, &runbook_fixture::DRAFTING_STEPS);
    run(&designed.root, &["design", "materialise", SLICE, "-p", "."]);
    designed.apply(
        "ready",
        &json!({ "agent_declaration": design_act::agent_declaration(
            AgentAct::DraftingReady,
            "the draft is ready for review",
        ) }),
    );
    designed.apply("to-reviewing", &json!({ "stage": { "to": "reviewing" } }));
    discharge_edge(&designed, &runbook_fixture::REVIEWING_STEPS);
    designed
}

/// The growing run of design `sec-5`: every list cause over its cap while the
/// run itself is real.
///
/// Not a hand-built envelope — the point is that a *run this size* still renders
/// under the ceiling through the binary, and every bound the envelope relies on
/// is reached before projection.
fn growing_run() -> DesignRun {
    let designed = ladder_to_reviewing();

    // 300 inquiries, each judged blocking where it is born. The judgements are
    // what make them *blocking* (`SL-264` sec-3): the set is derived from the
    // nodes, not from a declared act. The map's growth stales the graph review
    // (`ReviewedGraph`, the ids whose effective judgement is blocking over the
    // full set), which `reblock` below re-records — **not** `user-accepts-sufficiency`,
    // whose `InquiryMap` coverage compares only the keys the act carried, so a pure
    // addition does not move it (`SL-264` sec-2, *invisible*).
    let nodes: Vec<Value> = (0..300)
        .map(|index| {
            json!({
                "subject": node(index),
                "question": format!("blocking question {index}"),
                // Judged where each node is born: a judgement is owed at
                // creation (`SL-264` sec-3), and this fixture's nodes are the
                // blocking ones by construction.
                "blocking": true,
            })
        })
        .collect();
    designed.apply("growth", &json!({ "declare": nodes }));
    designed.apply(
        "reblock",
        &json!({
            "checkpoint_act": design_act::checkpoint_act(
                ActKind::GraphReviewed,
                "the grown blocking set is right",
            ),
        }),
    );

    // 300 sections, none reviewed.
    let sections: Vec<Value> = (0..300)
        .map(|index| {
            json!({
                "subject": format!("sec-{index:03}"),
                "body": format!("## section {index}\n\nbody {index}\n"),
            })
        })
        .collect();
    designed.apply("sections", &json!({ "declare": sections }));

    // 50 undisposed blockers on the run's own pass, whose ledger is concluded so
    // a `Conducted` disposition is admissible over it.
    let pass = designed
        .read()
        .review
        .pass
        .as_ref()
        .expect("a run in `reviewing` holds a pass")
        .review
        .clone();
    for index in 0..50 {
        run(
            &designed.root,
            &[
                "review",
                "raise",
                pass.as_str(),
                "--severity",
                "blocker",
                "--title",
                &format!("blocker {index}"),
                "--detail",
                "a blocking finding left open",
                "-p",
                ".",
            ],
        );
    }
    run(
        &designed.root,
        &["review", "conclude", pass.as_str(), "-p", "."],
    );
    designed.apply(
        "dispose",
        &json!({ "checkpoint_act": design_act::review_disposed(
            "the pass is disposed of at the close of review",
            ReviewDisposition::Conducted { review: pass },
        ) }),
    );
    designed
}

/// A node id, zero-padded so batch order matches declaration order.
fn node(index: u32) -> String {
    format!("inq-{index:04}")
}

/// Design `sec-8` / `REQ-437` / `sec-5` — *a large run still renders*.
///
/// The forward edge is in the no-drop set, so its size must be bounded by the
/// binary rather than by the run. A run large enough to exceed every cause cap
/// still renders under the budgeted ceiling, every capped cause says how many it
/// dropped, and `--full` shows the whole set — no member is lost silently
/// (`STD-003`).
///
/// The run grew by **pure additions** after `user-accepts-sufficiency` was given:
/// so it stays current (`SL-264` sec-2, *invisible*) and is not one of the
/// over-cap causes — asserted below, because that is the narrowing this slice
/// exists to make. The blocking additions do move the graph review, which
/// `reblock` re-records, and `blocking-inquiries-dispositioned` still reports the
/// 300 open blockers.
#[test]
fn large_run_still_renders() {
    let designed = growing_run();
    let envelope = designed.envelope(&[]);
    let rows = envelope["forward"]["unmet"]
        .as_array()
        .unwrap_or_else(|| panic!("forward unmet rows: {envelope}"));

    // (1) Every list cause is present and over its cap before projection — a
    // bound the fixture never reaches is a bound nobody proved.
    let listed = [
        "blocking-inquiries-dispositioned",
        "section-attestations-current",
        "review-disposition-attested",
    ];
    for condition in listed {
        let row = rows
            .iter()
            .find(|row| row["condition"] == json!(condition))
            .unwrap_or_else(|| panic!("`{condition}` is unmet in: {envelope}"));
        let capped = row["causes"]
            .as_array()
            .expect("causes")
            .iter()
            .any(|cause| cause["omitted"].as_u64().unwrap_or(0) > 0);
        assert!(capped, "`{condition}` exceeds the cause cap: {row}");
    }

    // (1b) The narrowing, pinned at e2e altitude: a pure addition does not
    // re-face sufficiency. `cleared()`-style flips on a *covered* node are a
    // different event and are pinned in the unit suite.
    assert!(
        !rows
            .iter()
            .any(|row| row["condition"] == json!("user-accepts-sufficiency")),
        "sufficiency is not re-faced by pure additions: {envelope}"
    );

    // (2) The rendered envelope stays under the ceiling the binary compiles.
    let rendered = designed.show(&[]);
    assert!(
        rendered.len() <= caps::NORMAL_BUDGET_BYTES,
        "the large run renders {} bytes against a {} byte ceiling",
        rendered.len(),
        caps::NORMAL_BUDGET_BYTES
    );

    // (3) No drop is silent: every capped row names the count it dropped.
    let block = forward_block(&rendered);
    for condition in listed {
        let line = block
            .iter()
            .find(|line| line.contains(condition))
            .unwrap_or_else(|| panic!("`{condition}` renders a row in:\n{rendered}"));
        assert!(
            line.contains("(+"),
            "`{condition}` names what the cap dropped: {line}"
        );
    }
    assert!(
        rendered.contains("(+295 more)"),
        "300 inquiries capped at 5"
    );
    assert!(rendered.contains("(+45 more)"), "50 blockers capped at 5");

    // (4) `--full` is uncapped, so the whole set is reachable.
    let full = designed.envelope(&["--full"]);
    let full_rows = full["forward"]["unmet"].as_array().expect("unmet rows");
    assert!(
        full_rows.iter().all(|row| row["causes"]
            .as_array()
            .expect("causes")
            .iter()
            .all(|cause| cause["omitted"].as_u64().unwrap_or(0) == 0)),
        "`--full` drops nothing: {full}"
    );
    let inquiries = full_rows
        .iter()
        .find(|row| row["condition"] == json!("blocking-inquiries-dispositioned"))
        .expect("the inquiries row");
    let nodes = inquiries["causes"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|cause| cause["cause"].get("inquiries-open"))
        .expect("an `InquiriesOpen` cause")["nodes"]
        .as_array()
        .expect("node ids");
    assert_eq!(nodes.len(), 300, "every blocking inquiry is listed");

    // (5) `EX-4` — the cause cap's derivation, measured against this real run.
    // `ENVELOPE_CAUSE_MEMBERS`' provenance claims a capped cause stays small
    // enough that the unmet rows cannot approach the ceiling. The ceiling's
    // per-row share is `NORMAL_BUDGET_BYTES / ENVELOPE_FORWARD_UNMET`, and the
    // row count is `Condition::ALL.len()` — both from the constants, not
    // re-typed here.
    let per_row = caps::NORMAL_BUDGET_BYTES / design_run::gate::Condition::ALL.len();
    let widest_row = block
        .iter()
        .filter(|line| line.starts_with("  unmet "))
        .map(|line| line.len())
        .max()
        .expect("the forward edge renders unmet rows");
    assert!(
        widest_row <= per_row,
        "a capped cause row measured {widest_row} B against a {per_row} B share of the 
         ceiling — the provenance comment on `ENVELOPE_CAUSE_MEMBERS` no longer holds"
    );
}

/// Phase `EX-3` — the batched-submission boundary.
///
/// `forward` is derived from the STORED snapshot, while `advance` is evaluated
/// against the snapshot the batch produces. A submission carrying the acts that
/// satisfy an edge **and** the stage move is therefore admitted even though the
/// read before it said `blocked`. That is not two answers to one question: the
/// read cannot see acts that do not exist yet. It does mark forward's boundary —
/// it describes what the current state still needs, not what a submission may
/// supply in the same breath.
#[test]
fn batched_acts_and_the_move_are_admitted() {
    let designed = started_with_record();
    discharge_exploring(&designed);
    designed.apply(
        "governance",
        &json!({ "checkpoint_act": design_act::checkpoint_act(
            ActKind::GovernanceConfirmed,
            "the governing artefacts are the ones found",
        ) }),
    );

    // The read before the batch: the graph's two halves are still owed.
    let before = designed.envelope(&[]);
    assert_eq!(before["forward"]["ready"], Value::Null);
    assert!(
        before["forward"]["unmet"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["condition"] == json!("initial-concerns-recorded")),
        "the read names the condition the batch will satisfy: {before}"
    );

    // The act AND the move, in one submission.
    designed.apply(
        "batched",
        &json!({
            "checkpoint_act": design_act::checkpoint_act(
                ActKind::GraphReviewed,
                "the empty blocking set is right",
            ),
            "stage": { "to": "inquiring" },
        }),
    );
    assert_eq!(
        designed.read().run.stage.as_str(),
        "inquiring",
        "the batch is admitted: admission evaluates the snapshot it produces"
    );
    // And the next read reports the next edge, not the one just crossed.
    assert_eq!(designed.envelope(&[])["forward"]["to"], json!("drafting"));
}

// ── the fitness re-measure (SL-264 PHASE-05) ──────────────────────────────

/// The inquiry map's size as `(nodes, needs edges)` — read from the snapshot the
/// binary wrote, not from the envelope's capped projections. The edge count is
/// the `needs`-relation out-degree summed over the nodes.
fn map_shape(designed: &DesignRun) -> (usize, usize) {
    let map = designed.read().map;
    let nodes: Vec<_> = map.inquiry.nodes().collect();
    let edges = nodes.iter().map(|node| node.needs().len()).sum();
    (nodes.len(), edges)
}

/// The condition tokens the run's outbound forward edge still owes, in table
/// order.
fn forward_unmet_conditions(designed: &DesignRun) -> Vec<String> {
    designed.envelope(&[])["forward"]["unmet"]
        .as_array()
        .expect("the forward edge lists its unmet conditions")
        .iter()
        .map(|row| {
            row["condition"]
                .as_str()
                .expect("an unmet row carries a condition token")
                .to_owned()
        })
        .collect()
}

/// Assert the run's two attested human gates are current on the forward edge.
fn assert_human_gates_clear(designed: &DesignRun) {
    let unmet = forward_unmet_conditions(designed);
    for gate in ["initial-concerns-recorded", "user-accepts-sufficiency"] {
        assert!(
            !unmet.iter().any(|condition| condition.as_str() == gate),
            "`{gate}` is re-faced while it should be current: {unmet:?}"
        );
    }
}

/// `SL-264` PHASE-05 `VA` — the fitness re-measure, landed as a run the audit
/// re-derives rather than as a number in a hand-back.
///
/// `RFC-031`'s bar is *inquiry nodes/edges added after `user-accepts-sufficiency`:
/// `0` — and each one voids the accepted judgement → `≥ 1`, without voiding it*.
/// This drives a run through the binary to `reviewing` with both attested
/// conditions recorded, then adds nodes and reads the gate's verdict.
///
/// **The numbers are asserted, so they are the measurement.** The ladder crosses
/// every edge below `reviewing` over an empty map (it declares no node), so the
/// map at the sufficiency acceptance was empty and stayed so until the growth
/// below — which adds two nodes and one `needs` edge, all therefore *after* the
/// acceptance. The gate re-faces `initial-concerns-recorded` for the blocking
/// addition alone, and never sufficiency.
#[test]
fn map_growth_after_sufficiency_re_faces_only_for_a_blocking_addition() {
    let designed = ladder_to_reviewing();

    // The baseline: empty at the acceptance, so every node observed later is the
    // measured quantity.
    assert_eq!(
        map_shape(&designed),
        (0, 0),
        "the ladder declares no nodes, so the map is empty at the acceptance"
    );

    // Positive control: before the growth both attested human gates are in good
    // standing, so their later appearance is the addition's doing, not the
    // ladder's.
    assert_human_gates_clear(&designed);

    // (1) A non-blocking addition leaves the effective blocking set unchanged, so
    // neither gate is re-faced.
    designed.apply(
        "non-blocking",
        &json!({ "declare": [
            { "subject": "inq-growth-a", "question": "a question that does not block", "blocking": false },
        ]}),
    );
    assert_human_gates_clear(&designed);

    // (2) A blocking addition reaches the user through `initial-concerns-recorded`
    // (`ReviewedGraph`, the full-set blocking comparison), while sufficiency
    // (`InquiryMap`, carried keys only) stays current for the same addition.
    designed.apply(
        "blocking",
        &json!({ "declare": [
            { "subject": "inq-growth-b", "question": "a question the agent judges blocking",
              "blocking": true, "needs": ["inq-growth-a"] },
        ]}),
    );
    let unmet = forward_unmet_conditions(&designed);
    assert!(
        unmet
            .iter()
            .any(|condition| condition == "initial-concerns-recorded"),
        "a new blocking node re-faces the graph review: {unmet:?}"
    );
    assert!(
        !unmet
            .iter()
            .any(|condition| condition == "user-accepts-sufficiency"),
        "sufficiency is not re-faced by an addition: {unmet:?}"
    );

    // (3) The measured quantity: nodes and edges added after the acceptance.
    assert_eq!(
        map_shape(&designed),
        (2, 1),
        "RFC-031's quantity: nodes and `needs` edges added after the acceptance"
    );
}
