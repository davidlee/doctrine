// SPDX-License-Identifier: GPL-3.0-only
//! SL-233 PHASE-09 — the evaluation kit's own assertions (`EX-5`, `EX-7`).
//!
//! The kit is **authored data** under `.doctrine/slice/233/evaluation/` plus this
//! file. Nothing about it lands in `src/` (sheet D1, POL-002): it is an experiment
//! instrument for one chore (`CHR-049`), not engine behaviour.
//!
//! **What these two tests are for.** `EX-5` asks that the kit be *mechanically
//! verified — the assertions run, not merely exist*. Two properties of the kit are
//! worth machine-checking, and `EX-7` names them:
//!
//! 1. the kit really covers all five evidence classes separately, and its
//!    subject-routing really holds — the property `VA-5` exists to catch, which a
//!    reader can talk themselves past but a set-disjointness check cannot;
//! 2. the rubric can strictly order a deliberately bad transcript below a good one
//!    — `VA-3`'s negative control. A rubric that cannot separate them measures
//!    nothing, and the kit would be decorative.
//!
//! **The scoring is computed here, never declared in the fixtures.** A transcript
//! records only which evidence keys the moderator observed; the band it lands in
//! is derived from the rubric. A fixture that stated its own score would make the
//! ordering test tautological — the fixture would be asserting the conclusion.
//!
//! **Read `evaluation/pre-registration.md`, not `plan.toml`'s `EX-8`/`VA-5`.**
//! Those two criteria are append-only and carry eleven rounds of amendment, so
//! read linearly they present withdrawn requirements before their withdrawals.
//! The pre-registration is the settled state and §8 registers every withdrawal.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::collections::BTreeSet;
use std::path::PathBuf;

mod common;

/// The shipped `exploring` step ids, from the module four other crates already
/// share. The gate table's first five rows key those same steps, and typing a
/// second copy here would be a fifth chance to drift from the asset (STD-001,
/// sheet D2 — ride the seam, do not refork it).
#[allow(
    dead_code,
    reason = "the module carries discharge helpers this crate does not drive"
)]
mod runbook_fixture;

use common::{SLICE_DIR, repo_root};
use runbook_fixture::EXPLORING_STEPS;

// ── the kit on disk ───────────────────────────────────────────────────────

/// The slice whose evaluation kit this is.
const SLICE_NUMBER: &str = "233";

/// The five evidence classes, in RFC-021 adherence-pipeline order. `EX-2` fixes
/// both the membership and the separation; `VA-5` adds that collapsing them into
/// one score defeats the criterion however many signals the rubric names.
const EVIDENCE_CLASSES: [&str; 5] = ["adopt", "adhere", "refresh", "recover", "complete"];

/// Kit root: `.doctrine/slice/233/evaluation/`.
fn kit_dir() -> PathBuf {
    repo_root()
        .join(SLICE_DIR)
        .join(SLICE_NUMBER)
        .join("evaluation")
}

/// Parse one kit file as TOML, naming the file in the failure so a missing or
/// malformed artefact reads as itself rather than as a scoring bug.
fn kit_toml(rel: &str) -> toml::Value {
    let path = kit_dir().join(rel);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("kit artefact {} is not readable: {e}", path.display()));
    toml::from_str(&text)
        .unwrap_or_else(|e| panic!("kit artefact {} is not valid TOML: {e}", path.display()))
}

/// Read one kit file as prose.
fn kit_text(rel: &str) -> String {
    let path = kit_dir().join(rel);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("kit artefact {} is not readable: {e}", path.display()))
}

/// The `[[…]]` array of tables at `key`, or a named failure.
fn array<'v>(v: &'v toml::Value, key: &str) -> &'v [toml::Value] {
    v.get(key)
        .unwrap_or_else(|| panic!("expected key `{key}`"))
        .as_array()
        .unwrap_or_else(|| panic!("expected `{key}` to be an array of tables"))
}

/// The string at `key`, or a named failure.
fn string<'v>(v: &'v toml::Value, key: &str) -> &'v str {
    v.get(key)
        .unwrap_or_else(|| panic!("expected key `{key}`"))
        .as_str()
        .unwrap_or_else(|| panic!("expected `{key}` to be a string"))
}

/// The string array at `key` as a set.
fn string_set(v: &toml::Value, key: &str) -> BTreeSet<String> {
    array(v, key)
        .iter()
        .map(|e| {
            e.as_str()
                .unwrap_or_else(|| panic!("expected `{key}` to hold strings"))
                .to_owned()
        })
        .collect()
}

// ── the scorer ────────────────────────────────────────────────────────────

/// The band a transcript reaches in one evidence class: the **highest-scoring**
/// band whose `requires` keys were all observed.
///
/// Derived, never declared. The rubric owns the thresholds and the transcript owns
/// the observations, so neither fixture can assert its own result. Band 0 requires
/// nothing, so every class always resolves.
fn class_score(rubric: &toml::Value, class_id: &str, observed: &BTreeSet<String>) -> i64 {
    let class = array(rubric, "class")
        .iter()
        .find(|c| string(c, "id") == class_id)
        .unwrap_or_else(|| panic!("rubric declares no class `{class_id}`"));

    array(class, "band")
        .iter()
        .filter(|band| string_set(band, "requires").is_subset(observed))
        .map(|band| {
            band.get("score")
                .and_then(toml::Value::as_integer)
                .unwrap_or_else(|| panic!("band in class `{class_id}` has no integer `score`"))
        })
        .max()
        .unwrap_or_else(|| panic!("class `{class_id}` has no band reachable with no evidence"))
}

/// Every evidence key a transcript's moderator recorded as observed.
fn observed(transcript: &toml::Value) -> BTreeSet<String> {
    string_set(transcript, "evidence")
}

// ── EX-7 test 1 ───────────────────────────────────────────────────────────

#[test]
fn evaluation_kit_assertions_run_over_all_five_evidence_classes() {
    let rubric = kit_toml("rubric.toml");
    let collectors = kit_toml("collectors.toml");
    let good = kit_toml("fixtures/transcript-good.toml");
    let bad = kit_toml("fixtures/transcript-bad.toml");
    let readme = kit_text("README.md");
    let rubric_md = kit_text("rubric.md");

    // ── EX-2: five classes, named and ordered, and no composite ──────────
    let declared = string_set(&rubric, "classes");
    let expected: BTreeSet<String> = EVIDENCE_CLASSES.iter().map(|c| (*c).to_owned()).collect();
    assert_eq!(
        declared, expected,
        "the rubric must score exactly the five RFC-021 pipeline classes"
    );
    // Checked structurally, not by scanning the prose for the word: a rubric is
    // free to *discuss* why it has no composite, and a negative grep would punish
    // it for saying so. What EX-2 forbids is the machinery — a top-level roll-up,
    // or per-class weights, which are a composite with the summation left implicit.
    assert!(
        rubric.get("composite").is_none() && rubric.get("total").is_none(),
        "EX-2 forbids collapsing the five classes into one score"
    );
    assert!(
        array(&rubric, "class")
            .iter()
            .all(|c| c.get("weight").is_none()),
        "per-class weights are a composite with the summation left to the reader"
    );

    // ── every class is separately scorable, on evidence, in bands ────────
    for class_id in EVIDENCE_CLASSES {
        let class = array(&rubric, "class")
            .iter()
            .find(|c| string(c, "id") == class_id)
            .unwrap_or_else(|| panic!("rubric declares no class `{class_id}`"));

        let bands = array(class, "band");
        assert!(
            bands.len() >= 3,
            "class `{class_id}` needs enough bands to separate two transcripts"
        );

        let scores: Vec<i64> = bands
            .iter()
            .map(|b| b.get("score").and_then(toml::Value::as_integer).unwrap())
            .collect();
        let distinct: BTreeSet<i64> = scores.iter().copied().collect();
        assert_eq!(
            distinct.len(),
            scores.len(),
            "class `{class_id}` has duplicate band scores, so its ordering is not strict"
        );
        assert_eq!(
            scores.iter().copied().min(),
            Some(0),
            "class `{class_id}` needs a zero band reachable with no evidence"
        );

        for band in bands {
            let requires = string_set(band, "requires");
            let score = band.get("score").and_then(toml::Value::as_integer).unwrap();
            assert!(
                !string(band, "descriptor").trim().is_empty(),
                "band {score} of `{class_id}` needs a descriptor a moderator can apply"
            );
            assert_eq!(
                requires.is_empty(),
                score == 0,
                "band {score} of `{class_id}`: only the zero band may require no evidence"
            );
            assert!(
                requires
                    .iter()
                    .all(|k| k.starts_with(&format!("{class_id}."))),
                "band {score} of `{class_id}` requires evidence keyed to another class"
            );
        }

        // The assertions genuinely run over this class, on both fixtures.
        let _ = class_score(&rubric, class_id, &observed(&good));
        let _ = class_score(&rubric, class_id, &observed(&bad));

        // Two-tier coherence: the machine rubric and the rubric a human accepts
        // under `VH-1` must not drift apart.
        assert!(
            rubric_md.contains(class_id),
            "rubric.md does not document class `{class_id}`"
        );
    }

    // ── VA-5: subject-routing, checked as set disjointness ───────────────
    let signals = array(&collectors, "signal");
    assert_eq!(signals.len(), 4, "the pre-registration fixes four signals");

    let mut delivery_evidence = BTreeSet::new();
    let mut classification_evidence = BTreeSet::new();
    for signal in signals {
        let id = string(signal, "id");
        let subject = string(signal, "subject");
        let consequence = string(signal, "consequence_kind");
        match subject {
            "delivery" => {
                assert_eq!(
                    consequence, "mechanism",
                    "{id} is a delivery signal, so it cannot argue for a placement revision"
                );
                delivery_evidence.extend(string_set(signal, "admissible_evidence"));
            }
            "classification" => {
                assert_eq!(
                    consequence, "placement",
                    "{id} is the classification signal and must reach the placement"
                );
                classification_evidence.extend(string_set(signal, "admissible_evidence"));
            }
            other => panic!("{id} declares unknown subject `{other}`"),
        }
    }
    assert!(
        !delivery_evidence.is_empty() && !classification_evidence.is_empty(),
        "both subjects must admit evidence, or the routing is vacuous"
    );
    assert!(
        delivery_evidence.is_disjoint(&classification_evidence),
        "a shared evidence key lets a result be routed to whichever revision is cheapest \
         — the trade VA-5 exists to catch"
    );

    // The classification signal fires only on positive evidence, all four items.
    let s4 = signals
        .iter()
        .find(|s| string(s, "subject") == "classification")
        .expect("no classification signal");
    let firing = array(s4, "firing_condition");
    assert_eq!(firing.len(), 4, "the firing condition is four items");
    assert!(
        firing
            .iter()
            .all(|f| f.get("required").and_then(toml::Value::as_bool) == Some(true)),
        "all four firing items are required"
    );
    assert_eq!(
        string(s4, "default"),
        "deny",
        "the firing condition is default-deny: satisfied only on positive evidence"
    );

    // ── the reopening: two arms, reclassify the default, reword earns it ──
    let reopening = collectors
        .get("reopening")
        .expect("collectors declare no reopening");
    let arms = array(reopening, "arm");
    assert_eq!(arms.len(), 2, "`stand` was withdrawn at round 6 — two arms");
    assert_eq!(
        string(reopening, "default_arm"),
        "reclassify",
        "offering the arms as equivalent restores `stand` under another name"
    );
    let reword = arms
        .iter()
        .find(|a| string(a, "id") == "reword")
        .expect("no reword arm");
    assert_eq!(
        reword
            .get("requires_artefact")
            .and_then(toml::Value::as_bool),
        Some(true),
        "no candidate step text, no reword"
    );
    assert_eq!(
        reword.get("one_shot").and_then(toml::Value::as_bool),
        Some(true),
        "a second firing on a reworded step reclassifies with no further reopening"
    );

    // Mechanism-side candidates are two, not three — the envelope-visible digest
    // was withdrawn as vacuous at round 6.
    assert_eq!(
        array(&collectors, "mechanism_candidate").len(),
        2,
        "two mechanism-side candidates: re-emission semantics, fragment size/split"
    );

    // ── the gate record: nine obligations, five reachable, denominator named ─
    let gate = collectors
        .get("gate")
        .expect("collectors carry no gate record");
    let obligations = array(gate, "obligation");
    assert_eq!(
        obligations.len(),
        9,
        "the gate ran over nine 2a obligations"
    );

    let visible: Vec<&toml::Value> = obligations
        .iter()
        .filter(|o| o.get("state_visible").and_then(toml::Value::as_bool) == Some(true))
        .collect();
    assert_eq!(
        gate.get("covered").and_then(toml::Value::as_integer),
        Some(visible.len() as i64),
        "the covered fraction must equal the rows it counts, not a number typed beside them"
    );
    assert_eq!(
        gate.get("total").and_then(toml::Value::as_integer),
        Some(obligations.len() as i64),
    );
    assert_eq!(visible.len(), 5, "five of nine are state-visible");

    for obligation in obligations {
        assert!(
            !string(obligation, "condition").trim().is_empty(),
            "every 2a obligation states a truthful completion condition in its text"
        );
    }

    // The four the classification signal cannot reach are named, not merely
    // subtracted — a rate over an uncovered denominator fails VA-5.
    let unreachable: BTreeSet<&str> = obligations
        .iter()
        .filter(|o| o.get("state_visible").and_then(toml::Value::as_bool) == Some(false))
        .map(|o| string(o, "id"))
        .collect();
    assert_eq!(unreachable.len(), 4);
    assert!(
        unreachable
            .iter()
            .all(|id| string_set(gate, "unreachable").contains(*id)),
        "the kit must name the four obligations the signal cannot reach"
    );

    // The runbook rows key the shipped step ids, single-sourced.
    let gate_ids: BTreeSet<&str> = obligations.iter().map(|o| string(o, "id")).collect();
    for step in EXPLORING_STEPS {
        assert!(
            gate_ids.contains(step),
            "gate table lost the shipped step `{step}`"
        );
    }

    // ── EX-1 / EX-3 / EX-4: the instruments exist and are collectible ─────
    let instruments = array(&collectors, "instrument");
    let instrument_ids: BTreeSet<&str> = instruments.iter().map(|i| string(i, "id")).collect();
    for required in ["cost.interaction", "cost.token", "acceptance-basis"] {
        assert!(
            instrument_ids.contains(required),
            "the kit does not measure `{required}`"
        );
    }
    for instrument in instruments {
        assert!(
            !string(instrument, "collected_as").trim().is_empty(),
            "instrument `{}` names no field a moderator records",
            string(instrument, "id")
        );
    }

    // ── EX-6: the honesty statement, in the kit's own voice ──────────────
    assert!(
        readme.contains("not proof of behavioural adoption") && readme.contains("CHR-049"),
        "EX-6: the README must say that a successful outcome alone is not proof, \
         and that CHR-049's live exercise is the actual evidence"
    );

    // ── the moderator protocol's standing obligation has a named field ────
    let protocol = kit_text("protocol.md");
    let field = string(&collectors, "context_state_field");
    assert!(
        protocol.contains(field),
        "protocol.md never names `{field}`, so the firing condition's item (4) \
         has no recorded evidence to be satisfied by"
    );

    // ── VA-4: both treatments are concrete artefacts, not an unnamed before ─
    for fixture in [&good, &bad] {
        assert!(
            !string(fixture, "treatment").trim().is_empty(),
            "a transcript with no named treatment says only that the fragment arrived"
        );
    }
    assert!(
        kit_dir().join("fixtures/baseline.md").is_file(),
        "VA-4 requires the comparison to name a concrete baseline artefact"
    );
}

// ── EX-7 test 2 — the kit's negative control ──────────────────────────────

#[test]
fn rubric_scores_a_known_bad_transcript_below_a_known_good_one() {
    let rubric = kit_toml("rubric.toml");
    let good = observed(&kit_toml("fixtures/transcript-good.toml"));
    let bad = observed(&kit_toml("fixtures/transcript-bad.toml"));

    // Strictly lower in EVERY class, checked class by class. Ordering on a summed
    // total would be the composite `EX-2` forbids, and would let one strong class
    // carry a transcript that failed the other four.
    for class_id in EVIDENCE_CLASSES {
        let good_score = class_score(&rubric, class_id, &good);
        let bad_score = class_score(&rubric, class_id, &bad);
        assert!(
            bad_score < good_score,
            "class `{class_id}`: known-bad scored {bad_score}, known-good {good_score} \
             — the rubric does not separate them here, so it is not measuring this class"
        );
    }

    // The separation must come from the evidence, not from a fixture asserting its
    // own result: a transcript that declared a score would make the ordering above
    // a restatement of the fixture.
    for fixture in [
        "fixtures/transcript-good.toml",
        "fixtures/transcript-bad.toml",
    ] {
        let raw = kit_toml(fixture);
        assert!(
            raw.get("score").is_none() && raw.get("band").is_none(),
            "{fixture} declares its own score, which makes the ordering tautological"
        );
    }
    assert!(
        bad.is_subset(&good),
        "the known-bad transcript must be a strict evidence subset of the known-good one, \
         so the ordering isolates the rubric rather than two unrelated runs"
    );
}
