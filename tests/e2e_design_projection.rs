// SPDX-License-Identifier: GPL-3.0-only
//! SL-233 PHASE-04 — the bounded `TurnEnvelope` projection and the command
//! surface it feeds, over the built binary (design §9.2, EX-10).
//!
//! **Exactly the seven tests EX-10 names, and the names are the contract.**
//!
//! The pure model is `#[path]`-included rather than imported for the same reason
//! `e2e_design_state.rs` does it: this crate is binary-only, so an integration
//! test can only spawn the binary — and including the leaf means the caps these
//! tests assert against are the exact bytes the binary compiles rather than
//! numbers re-typed beside the assertion (EX-4, VA-5).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use serde_json::{Value, json};

mod common;
mod design_fixture;

/// The pure model, from source — `design_run` is a leaf with crate out-degree
/// zero, so it compiles standalone here exactly as it does in the binary.
#[path = "../src/design_run/mod.rs"]
#[allow(
    dead_code,
    unused_imports,
    reason = "the whole leaf tree is included; no single test exercises all of it"
)]
mod design_run;

mod design_act;

use design_run::attestation::AgentAct;
use design_run::render::under_test as caps;

use design_fixture::{DesignRun, SLICE, fail, run};

// ── fixture ───────────────────────────────────────────────────────────────

/// The model-reading half of the fixture, local to this crate by design: these
/// read the snapshot through the `#[path]`-included leaf, which the shared
/// bootstrap in `tests/design_fixture/` deliberately does not depend on. A second
/// inherent `impl` is legal because [`DesignRun`] is defined in a module of this
/// same crate.
impl DesignRun {
    fn revision(&self) -> u64 {
        let text = std::fs::read_to_string(&self.snapshot).unwrap();
        design_run::snapshot::parse(&text).unwrap().run.revision
    }

    fn bytes(&self) -> Vec<u8> {
        std::fs::read(&self.snapshot).unwrap()
    }

    /// A payload carrying the current revision and `submission`, plus `body`'s
    /// keys merged in.
    fn payload(&self, submission: &str, body: Value) -> String {
        let mut object = json!({
            "run_uid": self.uid,
            "known_revision": self.revision(),
            "submission_id": submission,
        });
        let map = object.as_object_mut().unwrap();
        for (key, value) in body.as_object().unwrap() {
            map.insert(key.clone(), value.clone());
        }
        object.to_string()
    }

    fn apply(&self, submission: &str, body: Value) -> String {
        let payload = self.payload(submission, body);
        run(
            &self.root,
            &["design", "apply", SLICE, "-p", ".", "--input", &payload],
        )
    }

    fn refuse(&self, submission: &str, body: Value) -> String {
        let payload = self.payload(submission, body);
        fail(
            &self.root,
            &["design", "apply", SLICE, "-p", ".", "--input", &payload],
        )
    }

    /// The turn envelope itself, as the JSON rendering of the same model.
    fn envelope(&self, extra: &[&str]) -> Value {
        let mut args = vec!["--format", "json"];
        args.extend_from_slice(extra);
        serde_json::from_str(&self.show(&args)).expect("the JSON rendering parses")
    }
}

/// A node id, zero-padded so batch order (which is subject-id order) always
/// creates a parent before its children.
fn node(index: u32) -> String {
    format!("inq-{index:04}")
}

/// The length of a JSON array field on the envelope.
fn len(envelope: &Value, field: &str) -> usize {
    envelope[field]
        .as_array()
        .unwrap_or_else(|| panic!("`{field}` is an array: {envelope}"))
        .len()
}

/// The `id` of every entry in a JSON array field, in order.
fn ids(envelope: &Value, field: &str) -> Vec<String> {
    envelope[field]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|entry| entry["id"].as_str().unwrap_or_default().to_owned())
        .collect()
}

/// Prose designed to cost the most bytes per character it can: multi-byte
/// scripts, combining marks, an emoji with a modifier, and the two characters a
/// JSON encoder must escape.
const HOSTILE: &str = "ᚻᛖ᛫ᚳᚹᚫᚦ \"quoted\\escaped\" 👩🏽‍🔬 ẛ̣ ｆｕｌｌｗｉｄｔｈ — все смешалось";

/// A run with hundreds of nodes: a deep chain the cursor sits at the bottom of,
/// a wide fan of children beneath it, blockers, sections, and dispositions.
///
/// **Every bounded list is deliberately over its cap before projection** — a
/// bound the fixture never reaches is a bound nobody proved (VA-5).
fn large_run() -> DesignRun {
    let fixture = DesignRun::start();

    // The root, then a chain of twelve — deeper than ENVELOPE_ACTIVE_PATH_DEPTH.
    let mut declare: Vec<Value> = vec![json!({
        "subject": node(0),
        "question": "the root question",
        "provenance": {"provenance": "user-directed"},
    })];
    for step in 1..=12_u32 {
        declare.push(json!({
            "subject": node(step),
            "question": if step == 12 { HOSTILE.to_owned() } else { format!("chain step {step}") },
            "parent": node(step - 1),
        }));
    }
    // A wide fan beneath the chain's foot — more candidates than
    // ENVELOPE_FRONTIER_NODES, alternating provenance so user-directed and
    // agent-proposed are both present.
    for index in 100..140_u32 {
        declare.push(json!({
            "subject": node(index),
            "question": format!("fan {index} {HOSTILE}"),
            "parent": node(12),
            "provenance": {"provenance": if index % 2 == 0 { "user-directed" } else { "agent-proposed" }},
        }));
    }
    // Bulk: two hundred more leaves under the chain's head, so the map is large
    // but the neighbourhood is not.
    for index in 200..400_u32 {
        declare.push(json!({
            "subject": node(index),
            "question": format!("distant {index}"),
            "parent": node(0),
        }));
    }
    fixture.apply("seed", json!({ "declare": declare }));

    // Blockers: eight nodes needing the (open) root — more than ENVELOPE_BLOCKERS.
    let blockers: Vec<Value> = (500..508_u32)
        .map(|index| {
            json!({
                "subject": node(index),
                "question": format!("blocked {index}"),
                "parent": node(0),
                "needs": [node(0)],
            })
        })
        .collect();
    fixture.apply("blockers", json!({ "declare": blockers }));

    // Twenty sections — more than ENVELOPE_SECTION_ROWS.
    let sections: Vec<Value> = (1..=20_u32)
        .map(|index| {
            json!({
                "subject": format!("sec-{index:02}"),
                // EX-13(b): the title IS the body's opening heading. There is no
                // second field to state it in, so the hostile bytes ride the
                // heading — which is where a rendering has to survive them.
                "body": format!("## section {index} {HOSTILE}\n\nbody {index}\n{HOSTILE}"),
            })
        })
        .collect();
    fixture.apply("sections", json!({ "declare": sections }));

    // Ten dispositions — more than ENVELOPE_DURABLE_RECORDS. EX-12: `dispose`
    // is the only spelling, so Doctrine mints these ten decisions through
    // DEC-086 rather than the payload asserting ids that exist nowhere. In a
    // tree holding no other decision they come back as DEC-001..DEC-010, which
    // is what the `resume` assertions below name.
    let checkpoints: Vec<Value> = (1..=10_u32)
        .map(|index| {
            json!({
                "subject": format!("cp-{index:02}"),
                "disposes": node(199 + index),
                "dispose": {
                    "form": "create",
                    "kind": "decision",
                    "title": format!("Checkpointed decision {index}"),
                },
            })
        })
        .collect();
    fixture.apply("dispose", json!({ "declare": checkpoints }));

    // An assumption, so `resume` has one to report.
    fixture.apply(
        "assume",
        json!({ "declare": [{
            "subject": "cp-90",
            "disposes": node(210),
            "dispose": {
                "form": "create",
                "kind": "assumption",
                "title": "Checkpointed assumption",
            },
        }] }),
    );

    // A recorded act, so `resume` has an evidence reference to report. `T11`
    // re-sourced that section from the retired evidence store to the act set,
    // so this is the same fixture intent in the new vocabulary.
    fixture.apply(
        "declare-ready",
        json!({
            "agent_declaration": design_act::agent_declaration(
                AgentAct::DraftingReady,
                "the draft is ready for review",
            ),
        }),
    );

    // The cursor at the chain's foot, on the user's authority.
    fixture.apply(
        "focus",
        json!({ "traversal": { "cursor": node(12), "authority": "user-pinned" } }),
    );
    fixture
}

/// A run whose *neighbourhood* looks like [`large_run`]'s — a capped-full
/// frontier — over a map that is nine nodes rather than hundreds.
fn neighbourhood_twin() -> DesignRun {
    let fixture = DesignRun::start();
    let mut declare: Vec<Value> = vec![json!({"subject": node(0), "question": "root"})];
    for index in 1..=8_u32 {
        declare.push(json!({
            "subject": node(index),
            "question": format!("twin {index}"),
            "parent": node(0),
        }));
    }
    fixture.apply("seed", json!({ "declare": declare }));
    fixture.apply(
        "focus",
        json!({"traversal": {"cursor": node(0), "authority": "user-pinned"}}),
    );
    fixture
}

// ── EX-10's seven, and only these seven ───────────────────────────────────

/// EX-4 / VA-5 — every bounded list in the normal envelope sits at or under its
/// NAMED cap, and the whole budgeted rendering fits `ENVELOPE_NORMAL_BUDGET_BYTES`.
///
/// The fixture exceeds every one of those caps before projection, and that is
/// asserted first: a bound the input never reaches is unproven. The comparison
/// is against the constants the binary compiles, never against whatever the code
/// happened to produce.
#[test]
fn normal_envelope_stays_within_named_limits_on_a_large_run() {
    let fixture = large_run();
    // The delta is measured from the run's FIRST revision, so the row cap has
    // something to bind against — a cap tested against one revision's rows is a
    // cap the fixture never reaches.
    let full = fixture.envelope(&["--full", "--known-revision", "1"]);
    let normal = fixture.envelope(&["--known-revision", "1"]);

    // (1) The fixture EXCEEDS every cap — otherwise the caps below prove nothing.
    assert!(
        len(&full, "frontier") > caps::FRONTIER_NODES,
        "fixture must offer more candidates than the cap: {}",
        len(&full, "frontier")
    );
    assert!(len(&full, "active_path") > caps::ACTIVE_PATH_DEPTH);
    assert!(len(&full, "blockers") > caps::BLOCKERS);
    assert!(len(&full, "sections") > caps::SECTION_ROWS);
    assert!(len(&full, "durable_records") > caps::DURABLE_RECORDS);
    assert!(
        full["changes"]["rows"].as_array().unwrap().len() > caps::CHANGE_ROWS,
        "and more material changes than the row cap"
    );

    // (2) The normal envelope is bounded by the NAMED constants.
    assert_eq!(len(&normal, "frontier"), caps::FRONTIER_NODES);
    assert_eq!(len(&normal, "active_path"), caps::ACTIVE_PATH_DEPTH);
    assert_eq!(len(&normal, "blockers"), caps::BLOCKERS);
    assert_eq!(len(&normal, "sections"), caps::SECTION_ROWS);
    assert_eq!(len(&normal, "durable_records"), caps::DURABLE_RECORDS);
    assert_eq!(
        normal["changes"]["rows"].as_array().unwrap().len(),
        caps::CHANGE_ROWS
    );
    for entry in normal["frontier"].as_array().unwrap() {
        assert!(
            entry["question"].as_str().unwrap().len() <= caps::QUESTION_BYTES,
            "a rendered question is bounded in BYTES, not characters"
        );
    }

    // (3) No drop is silent.
    assert_eq!(normal["truncated"], json!(true));
    assert!(normal["omitted"]["frontier"].as_u64().unwrap() > 0);
    assert!(normal["totals"]["open_outside_frontier"].as_u64().unwrap() > 0);

    // (4) Design R2, repaired (sketch §(g)): two envelopes with the SAME visible
    // frontier size and radically different maps must not read as equally small.
    // An omitted count measures what a *cap* discarded and says nothing about
    // what *selection* discarded — the global totals are what carry that.
    let twin = neighbourhood_twin();
    let twin_envelope = twin.envelope(&[]);
    assert_eq!(
        len(&twin_envelope, "frontier"),
        len(&normal, "frontier"),
        "the two runs render an identically sized frontier"
    );
    assert!(
        normal["totals"]["nodes"].as_u64().unwrap()
            > twin_envelope["totals"]["nodes"].as_u64().unwrap() + 100,
        "yet the totals tell them apart: {} against {}",
        normal["totals"],
        twin_envelope["totals"]
    );

    // (5) The ceiling itself, MEASURED on the budgeted rendering.
    let rendered = fixture.show(&["--known-revision", "1"]);
    assert!(
        rendered.len() <= caps::NORMAL_BUDGET_BYTES,
        "the budgeted rendering measured {} bytes against a {} byte ceiling",
        rendered.len(),
        caps::NORMAL_BUDGET_BYTES
    );
}

/// EX-3 / EX-5 / sketch §(f) — `--full` may scale with the run; normal may not.
/// And the relation between them is **order-preserving subsequence**, not
/// prefix: normal retains the cursor end of the active path and the newest end
/// of the delta, both of which are suffixes.
#[test]
fn show_full_may_scale_but_normal_show_does_not() {
    let small = DesignRun::start();
    small.apply(
        "seed",
        json!({ "declare": [
            {"subject": node(0), "question": "root"},
            {"subject": node(1), "question": "one", "parent": node(0)},
        ] }),
    );
    let small_full = small.envelope(&["--full"]);

    let large = large_run();
    let large_full = large.envelope(&["--full"]);
    let large_normal = large.envelope(&[]);

    // `--full` scales with the run; normal does not.
    assert!(
        len(&large_full, "frontier") > len(&small_full, "frontier"),
        "--full grows with the map"
    );
    assert_eq!(len(&large_normal, "frontier"), caps::FRONTIER_NODES);

    // Subsequence, per list, with the retained end (b)'s table names.
    for field in ["frontier", "blockers", "sections", "durable_records"] {
        let full = ids(&large_full, field);
        let normal = ids(&large_normal, field);
        assert!(
            normal.len() <= full.len(),
            "{field} is not wider than --full"
        );
        assert_eq!(
            normal,
            full[..normal.len()].to_vec(),
            "{field}: normal must be an order-preserving subsequence of --full"
        );
    }
    let full_path = ids(&large_full, "active_path");
    let normal_path = ids(&large_normal, "active_path");
    assert_eq!(
        normal_path,
        full_path[full_path.len() - normal_path.len()..].to_vec(),
        "the active path retains the CURSOR end and drops the root end"
    );

    // `--full` never inlines authored prose — it cites sections.
    let full_text = large.show(&["--full"]);
    assert!(
        !full_text.contains("body 1\n"),
        "--full cites sections; it does not inline their bodies"
    );
}

/// EX-7 / DEC-063 — the three sparse states are three meanings, and they do not
/// collapse into two: an omitted key persists prior state, `null` clears a
/// nullable scalar, and `[]` clears a collection.
#[test]
fn omitted_key_persists_null_clears_scalar_empty_clears_collection() {
    let fixture = DesignRun::start();
    fixture.apply(
        "seed",
        json!({ "declare": [
            {"subject": node(0), "question": "root"},
            {"subject": node(1), "question": "the original question", "parent": node(0)},
            {"subject": node(2), "question": "needed", "parent": node(0)},
            {"subject": node(3), "question": "child", "parent": node(1), "needs": [node(2)]},
        ] }),
    );
    fixture.apply(
        "focus",
        json!({"traversal": {"cursor": node(1), "authority": "user-pinned"}}),
    );
    let before = fixture.envelope(&["--full"]);
    let question_of = |envelope: &Value, id: &str| -> String {
        envelope["frontier"]
            .as_array()
            .into_iter()
            .flatten()
            .chain(envelope["active_path"].as_array().into_iter().flatten())
            .find(|entry| entry["id"] == json!(id))
            .map(|entry| entry["question"].as_str().unwrap_or_default().to_owned())
            .unwrap_or_default()
    };
    assert_eq!(question_of(&before, &node(1)), "the original question");

    // One declaration, three assertions: `needs` omitted, `parent` null,
    // and a stated question. Nothing else about inq-3 is touched.
    fixture.apply(
        "sparse",
        json!({ "declare": [
            {"subject": node(3), "parent": null},
            {"subject": node(1), "question": "a restated question"},
        ] }),
    );
    let after = fixture.envelope(&["--full"]);

    // Omission PERSISTS: inq-3's `needs` survived a declaration that did not
    // mention it, so it is still blocked by inq-2.
    assert_eq!(
        after["totals"]["blocked"], before["totals"]["blocked"],
        "an omitted `needs` key persisted the prior collection"
    );
    // `null` CLEARED the scalar: inq-3 is now parentless, so it is a root.
    assert_eq!(question_of(&after, &node(1)), "a restated question");

    // `[]` clears the collection — a different spelling from omission, and it
    // must not be reachable by omitting the key.
    fixture.apply(
        "clear",
        json!({ "declare": [{"subject": node(3), "needs": []}] }),
    );
    let cleared = fixture.envelope(&["--full"]);
    assert_eq!(
        cleared["totals"]["blocked"],
        json!(0),
        "an empty collection CLEARED `needs`, so nothing is blocked any more"
    );
    assert_ne!(
        cleared["totals"]["blocked"], before["totals"]["blocked"],
        "and `[]` is therefore distinguishable from omission"
    );
}

/// EX-7 — one batch is unordered, so two declarations about one subject are
/// genuinely ambiguous. Refused rather than resolved last-wins, which would
/// invent an order the contract says does not exist.
#[test]
fn duplicate_subject_in_one_batch_is_refused() {
    let fixture = DesignRun::start();
    let before = fixture.bytes();
    let error = fixture.refuse(
        "duplicate",
        json!({ "declare": [
            {"subject": node(1), "question": "first"},
            {"subject": node(1), "question": "second"},
        ] }),
    );
    assert!(error.contains("duplicate subject"), "{error}");
    assert!(error.contains(&node(1)), "and it names which: {error}");
    assert_eq!(fixture.bytes(), before, "the run did not advance");
}

/// EX-6 — the WHOLE candidate is validated before any mutation, so a batch whose
/// last declaration is unlawful leaves the state byte-identical. The first
/// declarations are lawful on purpose: a half-applied batch would be visible.
#[test]
fn validation_failure_leaves_state_byte_identical() {
    let fixture = DesignRun::start();
    fixture.apply(
        "seed",
        json!({ "declare": [{"subject": node(0), "question": "root"}] }),
    );
    let before = fixture.bytes();

    let error = fixture.refuse(
        "invalid",
        json!({ "declare": [
            {"subject": node(1), "question": "lawful", "parent": node(0)},
            {"subject": node(2), "question": "lawful too", "parent": node(0)},
            {"subject": node(3), "question": "unlawful", "parent": "inq-9999"},
        ] }),
    );
    assert!(error.contains("unknown node"), "{error}");
    assert_eq!(
        fixture.bytes(),
        before,
        "byte-identical after a refusal — not merely 'mostly unchanged'"
    );

    // The same holds for a traversal declaration validated in the same pass.
    let error = fixture.refuse(
        "invalid-pin",
        json!({
            "declare": [{"subject": node(1), "question": "lawful", "parent": node(0)}],
            "traversal": {"pin": "inq-9999"},
        }),
    );
    assert!(error.contains("unknown node"), "{error}");
    assert_eq!(fixture.bytes(), before, "and still byte-identical");
}

/// EX-8 — pin, defer, prune and posture are first-class DECLARATIONS carrying an
/// authority, and user-directed is representably distinct from agent-proposed.
#[test]
fn pin_defer_prune_and_posture_are_user_directed_declarations() {
    let fixture = DesignRun::start();
    fixture.apply(
        "seed",
        json!({ "declare": [
            {"subject": node(0), "question": "root", "provenance": {"provenance": "user-directed"}},
            {"subject": node(1), "question": "kept", "parent": node(0),
             "provenance": {"provenance": "user-directed"}},
            {"subject": node(2), "question": "to defer", "parent": node(0)},
            {"subject": node(3), "question": "to prune", "parent": node(0)},
            {"subject": node(4), "question": "sibling", "parent": node(0)},
            {"subject": node(5), "question": "child of one", "parent": node(1)},
        ] }),
    );

    // Posture is load-bearing: it swaps kinship ranks 0 and 1 and nothing else.
    fixture.apply(
        "focus",
        json!({"traversal": {"cursor": node(1), "posture": "depth", "authority": "user-locked"}}),
    );
    let depth = fixture.envelope(&[]);
    assert_eq!(depth["run"]["posture"], json!("depth"));
    assert_eq!(depth["run"]["posture_authority"], json!("user-locked"));
    let depth_first = depth["frontier"][0]["kinship"].as_str().unwrap().to_owned();

    fixture.apply(
        "breadth",
        json!({"traversal": {"posture": "breadth", "authority": "user-pinned"}}),
    );
    let breadth = fixture.envelope(&[]);
    assert_eq!(
        (
            depth_first.as_str(),
            breadth["frontier"][0]["kinship"].as_str().unwrap()
        ),
        ("child", "sibling"),
        "posture swaps children and siblings, and nothing else"
    );

    // An AGENT proposal cannot move a user-LOCKED cursor. That refusal is the
    // whole reason authority is a field rather than a convention.
    let refused = fixture.refuse("agent-move", json!({"traversal": {"cursor": node(4)}}));
    assert!(refused.contains("user-locked"), "{refused}");

    // Pin: its own slot, carrying its own authority, and it renders with the
    // node's CURRENT state whatever that is.
    fixture.apply(
        "pin",
        json!({"traversal": {"pin": node(2), "authority": "user-pinned"}}),
    );
    fixture.apply(
        "defer-and-prune",
        json!({ "declare": [
            {"subject": node(2), "lifecycle": "deferred"},
            {"subject": node(3), "lifecycle": "pruned"},
        ] }),
    );
    let pinned = fixture.envelope(&[]);
    assert_eq!(pinned["pinned"]["id"], json!(node(2)));
    assert_eq!(pinned["pinned"]["authority"], json!("user-pinned"));
    assert_eq!(
        pinned["pinned"]["lifecycle"],
        json!("deferred"),
        "a pinned node that is now deferred still renders, WITH that state"
    );
    assert_eq!(pinned["totals"]["deferred"], json!(1));
    assert_eq!(pinned["totals"]["pruned"], json!(1));
    assert!(
        !ids(&pinned, "frontier").contains(&node(2)),
        "the pinned node is excluded from the candidate set, so it renders once"
    );

    // Provenance keeps user direction distinguishable from agent proposal.
    let provenance: Vec<&str> = pinned["frontier"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["provenance"].as_str().unwrap())
        .collect();
    assert!(
        provenance.contains(&"agent-proposed"),
        "provenance is carried per entry: {provenance:?}"
    );
    assert_eq!(
        pinned["frontier"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["id"] == json!(node(5)))
            .map(|entry| entry["provenance"].clone()),
        Some(json!("agent-proposed")),
        "a node the agent proposed is never laundered into user-directed"
    );
}

/// EX-9 — `doctrine design resume SL-NNN` needs no flags and returns the seven
/// scope §4 fields. The three optional flags ADD an assumption check or a
/// change-only projection; none is ever required addressing.
#[test]
fn resume_returns_the_seven_scope_fields_without_optional_flags() {
    let fixture = large_run();

    // The happy path: no flags at all.
    let resumed = fixture.resume(&[]);
    for field in [
        "active_path",
        "accepted_decisions",
        "open_questions",
        "assumptions",
        "evidence_references",
        "blockers",
        "next_obligation",
    ] {
        assert!(
            resumed.lines().any(|line| line.starts_with(field)),
            "resume names `{field}`: {resumed}"
        );
    }
    assert!(
        resumed.contains("DEC-010"),
        "accepted decisions carry their canonical refs, most recently linked \
         retained: {resumed}"
    );
    assert!(
        resumed.contains("ASM-001"),
        "and so do assumptions: {resumed}"
    );
    assert!(
        resumed.contains("drafting-ready — current"),
        "evidence references name the recorded act and whether it still binds: {resumed}"
    );

    // `--run` is an assumption CHECK: right passes, wrong refuses.
    let checked = fixture.resume(&["--run", &fixture.uid]);
    assert!(checked.contains("active_path"));
    let refused = fail(
        &fixture.root,
        &[
            "design",
            "resume",
            SLICE,
            "-p",
            ".",
            "--run",
            "dr-not-this-run",
        ],
    );
    assert!(refused.contains("stale"), "{refused}");

    // `--known-revision` narrows the delta; `--known-fragment` reports agreement.
    let scoped = fixture.resume(&["--known-revision", "1"]);
    assert!(scoped.contains("active_path"), "{scoped}");
    let fragment = fixture.resume(&["--known-fragment", "orientation"]);
    assert!(
        fragment.contains("known_fragment orientation NOT held by this run"),
        "{fragment}"
    );
}
