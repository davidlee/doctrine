//! SL-217 PHASE-03 — the elicitation queue as BLACK-BOX CLI goldens.
//!
//! Mirrors `tests/e2e_compare_inference.rs`'s harness (spawn the BUILT binary,
//! `mem.pattern.testing.black-box-cli-golden`). `compare elicit` is READ-classed
//! (`commands/guard.rs`, D18), so it runs fine inside a dispatch WORKER jail —
//! but `compare record` is WRITE-classed and a spawned record hits the jail's
//! authored-write refusal, so [`capture`] below hand-authors the SAME
//! session-of-one TOML the real command would mint (the exact wire shape
//! `comparison::wire` documents) — the sanctioned harness adaptation, mirroring
//! the inference suite.
//!
//! - VT-1 (§5.7): JSON goldens both kinds incl. the anchor-review `yield_note`
//!   conditional-yield disclosure (RV-269 F-3), the median-probe reason + the
//!   bare-estimate mask, and the D15 stall / stable / candidates state wording;
//!   `--kind` filters post-ranking, `--limit` caps display over a ranked pool.
//! - VT-2 (§5.6): determinism — `shuffled_load_order` byte-identity of the queue
//!   and `--json` across fixed session-file permutations.
//! - VT-3: `capture_loop_round_trip` — a surfaced candidate, answered by
//!   appending a ledger row, is consumed by the next elicit refresh.
//! - VT-4 (§5.9): `cost_ceiling_eval_corpus` — elicit over a frozen 32-row /
//!   K = 8 snapshot completes within the normal test-time envelope (completion,
//!   not a benchmark).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

/// A tempdir `root::find` resolves as a project root.
fn project() -> tempfile::TempDir {
    let dir = tmp();
    write(dir.path(), ".project", "");
    write(dir.path(), "doctrine.toml", "");
    dir
}

/// `doctrine <args...> -p <root>` over the built binary.
fn run(root: &Path, args: &[&str]) -> Output {
    let mut a: Vec<&str> = args.to_vec();
    a.push("-p");
    a.push(root.to_str().expect("utf8 path"));
    Command::new(bin())
        .args(&a)
        .output()
        .expect("spawn doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}
fn ok(out: &Output) -> String {
    assert!(out.status.success(), "stderr: {}", stderr(out));
    stdout(out)
}

/// `doctrine compare elicit [extra] -p <root>`, asserted success.
fn elicit(root: &Path, extra: &[&str]) -> String {
    let mut args = vec!["compare", "elicit"];
    args.extend_from_slice(extra);
    ok(&run(root, &args))
}

/// Seed a value-bearing backlog issue (non-RSK — comparison admits it) with an
/// optional verbatim facet tail (`[value]` / `[estimate]` tables after the empty
/// `[relationships]`). The id dir is zero-padded (the scan resolves `ISS-040`
/// from `issue/040/`, never `issue/40/`).
fn seed(root: &Path, id: u32, extra: &str) {
    write(
        root,
        &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.toml"),
        &format!(
            "id = {id}\nslug = \"i{id}\"\ntitle = \"Issue {id}\"\nkind = \"issue\"\n\
             status = \"open\"\nresolution = \"\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n[relationships]\n{extra}"
        ),
    );
    write(
        root,
        &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.md"),
        "b\n",
    );
}

fn iss(id: u32) -> String {
    format!("ISS-{id:03}")
}

/// Hand-author a prefer-a session-of-one over the pair (the wire shape `compare
/// record` mints) at a fixed session/row uid — the worker-jail adaptation (see
/// the module doc). `slot` fixes the filename so callers control load order.
fn capture(root: &Path, slot: &str, a: &str, b: &str) {
    let body = format!(
        "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
         [session]\nuid = \"sess-{slot}\"\ndate = \"2026-01-01\"\n\n\
         [[judgement]]\nuid = \"row-{slot}\"\nseq = 0\na = \"{a}\"\nb = \"{b}\"\n\
         response = \"prefer-a\"\ndomain = \"value\"\nframe = \"equal-effort\"\nform = \"order\"\n\
         rater = \"agent\"\ndate = \"2026-01-01\"\n"
    );
    write(
        root,
        &format!(".doctrine/comparisons/2026-01-01-{slot}.toml"),
        &body,
    );
}

// =============================================================================
// VT-1 — JSON goldens (both kinds) + median-probe + bare-estimate mask
// =============================================================================

/// The anchor-conflict corpus (design §5.3): `A(1) > B > C(3)` — the chain order
/// contradicts the A=1 / C=3 anchors, so both anchors are suspect. Deterministic,
/// no rng.
fn seed_anchor_conflict(root: &Path) {
    seed(root, 40, "[value]\nvalue = 1.0\n");
    seed(root, 41, "");
    seed(root, 42, "[value]\nvalue = 3.0\n");
    capture(root, "ab", &iss(40), &iss(41));
    capture(root, "bc", &iss(41), &iss(42));
}

/// VT-1: the anchor-review JSON entry carries the conditional-yield `yield_note`
/// (RV-269 F-3), per-answer `exits` keyed by the answer tokens, and a subject
/// naming the conflict pair + quarantined closure — for BOTH suspects.
#[test]
fn json_anchor_review_carries_yield_note_exits_and_suspects() {
    let dir = project();
    let root = dir.path();
    seed_anchor_conflict(root);

    let json: serde_json::Value =
        serde_json::from_str(&elicit(root, &["--kind", "anchor-review", "--json"])).unwrap();
    let entries = json["entries"].as_array().unwrap();

    // A single stale-anchor conflict names two anchors ⇒ two suspects (D12).
    let subjects: Vec<&str> = entries
        .iter()
        .map(|e| e["subject"]["id"].as_str().unwrap())
        .collect();
    assert!(
        subjects.contains(&"ISS-040") && subjects.contains(&"ISS-042"),
        "both suspects surface: {subjects:?}"
    );

    let first = &entries[0];
    assert_eq!(first["kind"], "anchor-review");
    assert_eq!(first["yield_basis"], "canonical-resolving-actions");
    let ask = &first["ask"];
    // The conditional-yield disclosure (the VT-1 `yield_note` keyword contract).
    assert!(
        ask["yield_note"]
            .as_str()
            .unwrap()
            .contains("RESOLVING revision"),
        "yield_note discloses the conditional yield"
    );
    // exits keyed by the two answer tokens; revise re-authors, uphold retires.
    let exits = &ask["exits"];
    assert_eq!(
        exits["revise-anchor"][0],
        format!(
            "doctrine value set {} <v>",
            first["subject"]["id"].as_str().unwrap()
        )
    );
    assert!(
        exits["uphold-anchor"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c.as_str().unwrap().starts_with("doctrine compare withdraw")),
        "uphold suggests withdrawing the cited rows"
    );
    // No comparison-only fields leaked onto the anchor ask.
    assert!(ask.get("frame").is_none(), "anchor ask carries no frame");
}

/// The median-probe corpus: two constrained items + one un-constrained
/// (zero-row, no-anchor, no-estimate) item — the un-constrained item yields a
/// median-probe candidate against the projected median of its comparable set.
fn seed_median_probe(root: &Path) {
    seed(root, 50, "");
    seed(root, 51, "");
    seed(root, 52, "");
    capture(root, "m", &iss(50), &iss(51));
}

/// VT-1: an un-constrained top-K item surfaces a `median`-probe comparison
/// candidate (D14) — the reason keyword contract — and a bare participant is
/// masked (D17), the PRESENT half of the mask golden.
#[test]
fn json_median_probe_surfaces_for_unconstrained_item() {
    let dir = project();
    let root = dir.path();
    seed_median_probe(root);

    let json: serde_json::Value =
        serde_json::from_str(&elicit(root, &["--kind", "comparison", "--json"])).unwrap();
    let entries = json["entries"].as_array().unwrap();
    assert!(
        entries.iter().any(|e| e["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["code"] == "median-probe")),
        "un-constrained item calibrates against the projected median"
    );
    // A bare (estimate-free) participant is masked with a null estimate.
    let masked_bare = entries
        .iter()
        .flat_map(|e| e["participants"].as_array().into_iter().flatten())
        .any(|p| {
            p["estimate"].is_null()
                && p["annotations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|a| a == "projection masked by bare estimate")
        });
    assert!(
        masked_bare,
        "a projected-but-bare participant carries the mask (D17)"
    );
}

/// The frontier-pair corpus: two constrained-but-mutually-indeterminate items
/// with estimate facets (ISS-070/071), plus a second such pair left bare
/// (ISS-072/073) — one indeterminate frontier pair each (D5), so both the
/// estimated-unmasked and the bare-masked participant shapes surface together.
fn seed_frontier_pairs(root: &Path) {
    seed(root, 70, "[estimate]\nlower = 2.0\nupper = 2.0\n");
    seed(root, 71, "[estimate]\nlower = 2.0\nupper = 2.0\n");
    seed(root, 72, "");
    seed(root, 73, "");
    capture(root, "ax", &iss(70), &iss(72));
    capture(root, "by", &iss(71), &iss(73));
}

/// VT-1: a comparison entry carries the equal-effort/value frame, structural
/// value bounds (`{kind, value?}` — never a flattened scalar), and the estimate
/// present/absent split: an estimated participant reports a scalar `est_cost`
/// with NO mask; a bare one reports `null` WITH the mask (D6/D17).
#[test]
fn json_comparison_carries_value_bounds_and_estimate_mask_split() {
    let dir = project();
    let root = dir.path();
    seed_frontier_pairs(root);

    let json: serde_json::Value =
        serde_json::from_str(&elicit(root, &["--kind", "comparison", "--json"])).unwrap();
    let entries = json["entries"].as_array().unwrap();
    assert!(!entries.is_empty(), "indeterminate frontier pairs surface");

    let e = &entries[0];
    assert_eq!(e["ask"]["frame"], "equal-effort");
    assert_eq!(e["ask"]["domain"], "value");

    let participants: Vec<&serde_json::Value> = entries
        .iter()
        .flat_map(|e| e["participants"].as_array().into_iter().flatten())
        .collect();
    // Structural bounds mirror the Bound enum (never a flat scalar).
    for p in &participants {
        assert!(
            p["value"]["provenance"].is_string(),
            "value provenance present"
        );
        assert!(
            p["value"]["bounds"]["lower"]["kind"].is_string(),
            "structural bounds carry a kind: {}",
            p["value"]
        );
    }
    // The estimated pair (ISS-070/071) reports a scalar estimate and NO mask.
    let est = participants
        .iter()
        .find(|p| p["id"] == "ISS-070")
        .expect("ISS-070 present");
    assert!(
        est["estimate"].is_number(),
        "estimated participant reports est_cost"
    );
    assert!(
        est["annotations"].as_array().unwrap().is_empty(),
        "estimated participant is NOT masked"
    );
    // The bare pair (ISS-072/073) reports null + the mask.
    let bare = participants
        .iter()
        .find(|p| p["id"] == "ISS-072")
        .expect("ISS-072 present");
    assert!(bare["estimate"].is_null(), "bare estimate is null");
    assert!(
        bare["annotations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a == "projection masked by bare estimate"),
        "bare participant is masked (D17)"
    );
}

// =============================================================================
// VT-1 — D15 state wording (candidates / stalled / stable)
// =============================================================================

/// VT-1: the stall render names the depth and disclaims stability — a zero-yield
/// bridge (`T(5) > A,B > L(-5)` with differing A/B costs) admits no candidate but
/// leaves an indeterminate pair (D15). Exercises the `stall` keyword contract.
#[test]
fn render_stall_names_depth_and_disclaims_stability() {
    let dir = project();
    let root = dir.path();
    seed(root, 60, "[value]\nvalue = 5.0\n");
    seed(root, 61, "[estimate]\nlower = 1.0\nupper = 1.0\n");
    seed(root, 62, "[estimate]\nlower = 4.0\nupper = 4.0\n");
    seed(root, 63, "[value]\nvalue = -5.0\n");
    capture(root, "ta", &iss(60), &iss(61));
    capture(root, "al", &iss(61), &iss(63));
    capture(root, "tb", &iss(60), &iss(62));
    capture(root, "bl", &iss(62), &iss(63));

    let out = elicit(root, &["--depth", "2"]);
    assert!(
        out.contains("stalled at depth 2") && out.contains("NOT a stability claim"),
        "stall names depth + disclaims stability: {out}"
    );
}

/// VT-1: a fully-determined top-K renders `stable` with the member-scoped
/// value_dim-order wording (never prefix-membership, D5/D15).
#[test]
fn render_stable_is_member_scoped() {
    let dir = project();
    let root = dir.path();
    // Two anchored, consistent items ⇒ the pair is determined, no candidate.
    seed(root, 65, "[value]\nvalue = 1.0\n");
    seed(root, 66, "[value]\nvalue = 5.0\n");
    capture(root, "d", &iss(66), &iss(65));

    let out = elicit(root, &[]);
    assert!(
        out.contains("stable: value_dim order among the current top-")
            && out.contains("not membership"),
        "stable is member-scoped: {out}"
    );
}

// =============================================================================
// VT-1 / T5 — post-ranking --kind filter + --limit display cap
// =============================================================================

/// VT-1/T5: `--kind` filters entries post-ranking; `--limit` caps the DISPLAY
/// while the full pool is still ranked (a limit of 1 shows exactly one entry).
#[test]
fn kind_filter_and_limit_cap_the_view() {
    let dir = project();
    let root = dir.path();
    seed_anchor_conflict(root); // yields anchor-review entries (two suspects)
    seed(root, 45, ""); // an extra un-constrained item ⇒ a comparison entry too

    // --kind anchor-review keeps only anchor entries.
    let anchors: serde_json::Value =
        serde_json::from_str(&elicit(root, &["--kind", "anchor-review", "--json"])).unwrap();
    assert!(
        anchors["entries"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["kind"] == "anchor-review"),
        "--kind filters post-ranking"
    );

    // --limit 1 caps the display to a single entry.
    let capped: serde_json::Value = serde_json::from_str(&elicit(
        root,
        &["--kind", "anchor-review", "--json", "--limit", "1"],
    ))
    .unwrap();
    assert_eq!(
        capped["entries"].as_array().unwrap().len(),
        1,
        "--limit caps the displayed pool"
    );
}

// =============================================================================
// VT-2 — determinism under shuffled session-file load order
// =============================================================================

const SESSION_A: &str = "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
[session]\nuid = \"sess-a\"\ndate = \"2026-01-01\"\n\n\
[[judgement]]\nuid = \"row-a\"\nseq = 0\na = \"ISS-090\"\nb = \"ISS-091\"\n\
response = \"prefer-a\"\ndomain = \"value\"\nframe = \"equal-effort\"\nform = \"order\"\n\
rater = \"agent\"\ndate = \"2026-01-01\"\n";
const SESSION_B: &str = "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
[session]\nuid = \"sess-b\"\ndate = \"2026-01-01\"\n\n\
[[judgement]]\nuid = \"row-b\"\nseq = 0\na = \"ISS-091\"\nb = \"ISS-092\"\n\
response = \"prefer-a\"\ndomain = \"value\"\nframe = \"equal-effort\"\nform = \"order\"\n\
rater = \"agent\"\ndate = \"2026-01-01\"\n";
const SESSION_C: &str = "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
[session]\nuid = \"sess-c\"\ndate = \"2026-01-01\"\n\n\
[[judgement]]\nuid = \"row-c\"\nseq = 0\na = \"ISS-092\"\nb = \"ISS-093\"\n\
response = \"prefer-a\"\ndomain = \"value\"\nframe = \"equal-effort\"\nform = \"order\"\n\
rater = \"agent\"\ndate = \"2026-01-01\"\n";

/// Write the three sessions under filenames whose sort order is `order` (a
/// permutation of the file BODIES) — the shuffle is over which body lands in
/// which alphabetical slot, never over content.
fn write_shuffled(root: &Path, order: [&str; 3]) {
    for (slot, body) in ["aaa", "bbb", "ccc"].iter().zip(order) {
        write(
            root,
            &format!(".doctrine/comparisons/2026-01-01-{slot}.toml"),
            body,
        );
    }
}

/// VT-2 (§5.6): the same merged file set + statuses + config + params yields a
/// byte-identical queue AND `--json` across fixed session-file permutations.
#[test]
fn shuffled_load_order_yields_byte_identical_queue_and_json() {
    let permutations: [[&str; 3]; 3] = [
        [SESSION_A, SESSION_B, SESSION_C],
        [SESSION_C, SESSION_A, SESSION_B],
        [SESSION_B, SESSION_C, SESSION_A],
    ];

    let mut humans = Vec::new();
    let mut jsons = Vec::new();
    for order in permutations {
        let dir = project();
        let root = dir.path();
        for id in 90..=93 {
            seed(root, id, "");
        }
        write_shuffled(root, order);
        humans.push(elicit(root, &[]));
        jsons.push(elicit(root, &["--json"]));
    }

    assert!(
        humans.windows(2).all(|w| w[0] == w[1]),
        "elicit human is byte-identical across shuffled load order: {humans:?}"
    );
    assert!(
        jsons.windows(2).all(|w| w[0] == w[1]),
        "elicit --json is byte-identical across shuffled load order: {jsons:?}"
    );
}

// =============================================================================
// VT-3 — capture loop round trip
// =============================================================================

/// VT-3: a surfaced median-probe candidate, answered by appending a ledger row
/// (hand-authored — the jail adaptation), is consumed by the next elicit
/// refresh: the newly-constrained item no longer raises its median-probe.
#[test]
fn capture_loop_round_trip_consumes_the_answered_pair() {
    let dir = project();
    let root = dir.path();
    seed_median_probe(root); // ISS-052 is un-constrained ⇒ a median-probe subject

    let before: serde_json::Value =
        serde_json::from_str(&elicit(root, &["--kind", "comparison", "--json"])).unwrap();
    let probes_052 = |v: &serde_json::Value| -> bool {
        v["entries"].as_array().unwrap().iter().any(|e| {
            e["reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["code"] == "median-probe")
                && e["participants"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|p| p["id"] == "ISS-052")
        })
    };
    assert!(
        probes_052(&before),
        "ISS-052 starts as a median-probe subject"
    );

    // Answer: a comparison row constrains ISS-052 against ISS-051.
    capture(root, "answer", &iss(52), &iss(51));

    let after: serde_json::Value =
        serde_json::from_str(&elicit(root, &["--kind", "comparison", "--json"])).unwrap();
    assert_ne!(before, after, "the refresh consumes the appended answer");
    assert!(
        !probes_052(&after),
        "the now-constrained ISS-052 no longer raises a median-probe"
    );
}

// =============================================================================
// VT-4 — cost ceiling over a frozen eval-corpus snapshot
// =============================================================================

/// A FROZEN snapshot of the Phase-B eval corpus scale (32 value-bearing items,
/// frontier K = 8), generated deterministically in-test — NEVER the live
/// `.doctrine/comparisons/` ledger (design §5.9). A chain of comparison rows
/// keeps every item comparison-engaged so the pool is exercised at scale.
fn seed_eval_corpus(root: &Path) {
    for id in 100..132 {
        seed(root, id, "");
    }
    // A connected chain ISS-100 > ISS-101 > … > ISS-131 (31 rows) + one anchor
    // pair to exercise the anchor-review path over the corpus.
    for id in 100..131 {
        capture(root, &format!("c{id}"), &iss(id), &iss(id + 1));
    }
}

/// VT-4 (§5.9): elicit over the frozen 32-row / K = 8 snapshot completes within
/// the normal test-time envelope — an assertion of COMPLETION (both surfaces
/// succeed), not a benchmark. `cost_ceiling_eval_corpus`.
#[test]
fn cost_ceiling_eval_corpus_completes() {
    let dir = project();
    let root = dir.path();
    seed_eval_corpus(root);

    // Both surfaces complete over the corpus at the default depth (K = 8).
    let human = elicit(root, &[]);
    assert!(
        human.contains("state:"),
        "human render completes with a footer"
    );
    let json: serde_json::Value = serde_json::from_str(&elicit(root, &["--json"])).unwrap();
    assert_eq!(json["context"]["depth"], 8, "default frontier K = 8");
    assert_eq!(json["schema"], "doctrine.elicit-queue");
}

// =============================================================================
// SL-218 PHASE-01 — VT-F: demote_agent_evidence on the elicit surface
// =============================================================================

/// VT-F: knob-on, a pair previously retired by agent testimony re-enters the
/// queue as a live comparison candidate, the state line follows, and the
/// surface carries the demotion disclosure line.
#[test]
fn demote_agent_evidence_reenters_pair_with_disclosure() {
    let dir = project();
    let root = dir.path();
    seed(root, 80, "");
    seed(root, 81, "");
    capture(root, "kd", &iss(80), &iss(81)); // rater = agent (fixture wire shape)

    let before = elicit(root, &[]);
    assert!(
        before.contains("stable: value_dim order"),
        "knob-off: the agent row retires the pair: {before}"
    );
    assert!(
        !before.contains("agent evidence demoted"),
        "no disclosure when the knob is off: {before}"
    );

    write(
        root,
        ".doctrine/doctrine.toml",
        "[priority.compare]\ndemote_agent_evidence = true\n",
    );
    let after = elicit(root, &[]);
    assert!(
        after.contains("[comparison]") && after.contains(&iss(80)) && after.contains(&iss(81)),
        "knob-on: the pair re-enters as a candidate: {after}"
    );
    assert!(
        after.contains("candidates outstanding"),
        "state line reflects the reopened queue: {after}"
    );
    assert!(
        after.contains(
            "agent evidence demoted: agent judgements propose orderings but do not retire \
             questions"
        ),
        "disclosure line present knob-on: {after}"
    );

    // JSON parity: the disclosure rides an ADDITIVE key, knob-on only.
    let json: serde_json::Value = serde_json::from_str(&elicit(root, &["--json"])).unwrap();
    assert_eq!(json["agent_evidence_demoted"], true);
}
