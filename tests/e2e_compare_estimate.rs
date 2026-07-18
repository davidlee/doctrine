//! SL-219 PHASE-06 (design §6.10) — the estimate comparison domain as BLACK-BOX
//! CLI goldens: capture `more-work` → compile → project → feed → a VISIBLE score
//! shift in `explain`; the full sizing-probe round-trip (elicit → record →
//! re-elicit shows the subject sized); and the domain-tagged finding render with
//! `--json` parity.
//!
//! Mirrors `tests/e2e_compare_elicit.rs`'s harness (spawn the BUILT binary,
//! `mem.pattern.testing.black-box-cli-golden`). Every verb here — `explain`,
//! `compare elicit`, `findings` — is READ-classed (`commands/guard.rs`, D18), so
//! the suite runs inside a dispatch WORKER jail; `compare record` is
//! WRITE-classed and a spawned record hits the jail's authored-write refusal, so
//! [`capture_more_work`] hand-authors the SAME est-domain session-of-one the real
//! command would mint (the sanctioned harness adaptation, mirroring the elicit
//! suite).

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

fn iss(id: u32) -> String {
    format!("ISS-{id:03}")
}

/// Seed an open value-bearing backlog issue with a verbatim facet tail
/// (`[estimate]` / `[value]` tables, before the empty `[relationships]`).
fn seed(root: &Path, id: u32, facets: &str) {
    write(
        root,
        &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.toml"),
        &format!(
            "id = {id}\nslug = \"i{id}\"\ntitle = \"Issue {id}\"\nkind = \"issue\"\n\
             status = \"open\"\nresolution = \"\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{facets}[relationships]\n"
        ),
    );
    write(
        root,
        &format!(".doctrine/backlog/issue/{id:03}/backlog-{id:03}.md"),
        "b\n",
    );
}

/// Hand-author an est-domain `more-work` session-of-one over the pair (the wire
/// shape `compare record --frame more-work` mints — the worker-jail adaptation).
/// `prefer-a` ⇒ `a` is the costlier item (D5); `equal` merges the pair's cost;
/// `incomparable` compiles to `NoConstraint`.
fn capture_more_work(root: &Path, slot: &str, a: &str, b: &str, resp: &str) {
    write(
        root,
        &format!(".doctrine/comparisons/2026-01-01-{slot}.toml"),
        &format!(
            "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
             [session]\nuid = \"sess-{slot}\"\ndate = \"2026-01-01\"\n\n\
             [[judgement]]\nuid = \"row-{slot}\"\nseq = 0\na = \"{a}\"\nb = \"{b}\"\n\
             response = \"{resp}\"\ndomain = \"estimate\"\nframe = \"more-work\"\n\
             form = \"order\"\nrater = \"human\"\ndate = \"2026-01-01\"\n"
        ),
    );
}

/// Hand-author an estimate-domain anchor row (cost-anchor frame, human-tier).
/// PHASE-06: anchors are claim-derived — this mint a Pin/Human claim row
/// so the est system has a claim-derived anchor.
fn capture_cost_anchor(root: &Path, uid: &str, item: &str, lower: f64, upper: f64) {
    write(
        root,
        &format!(".doctrine/comparisons/2026-01-01-{uid}.toml"),
        &format!(
            "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
             [session]\nuid = \"sess-{uid}\"\ndate = \"2026-01-01\"\n\n\
             [[judgement]]\nuid = \"{uid}\"\nseq = 0\na = \"{item}\"\n\
             domain = \"estimate\"\nframe = \"cost-anchor\"\n\
             form = \"anchor\"\nrater = \"human\"\ndate = \"2026-01-01\"\n\
             est_lower = {lower}\nest_upper = {upper}\n"
        ),
    );
}

fn explain(root: &Path, id: &str) -> String {
    ok(&run(root, &["explain", id]))
}

fn elicit(root: &Path, extra: &[&str]) -> String {
    let mut args = vec!["compare", "elicit"];
    args.extend_from_slice(extra);
    ok(&run(root, &args))
}

fn elicit_json(root: &Path, extra: &[&str]) -> serde_json::Value {
    serde_json::from_str(&elicit(root, extra)).expect("elicit json")
}

// =============================================================================
// EX-2 / §6.10 — capture more-work → visible score shift in explain
// =============================================================================

/// EX-2: an est-domain `more-work` capture flows compile → project → feed → the
/// scoring divisor, so `explain` shows BOTH a moved score AND the new
/// cost-source block. Before any est row the block is absent (byte-identical to
/// pre-SL-219 explain); after, the bare item is projected below the anchor, its
/// `est_cost` drops, and `value_dim`/score rise.
#[test]
fn capture_more_work_shifts_score_and_reveals_cost_source() {
    let dir = project();
    let root = dir.path();
    // ISS-010 authors an 8.0 anchor; ISS-011 is bare (divisor = bare anchor 9.0).
    seed(
        root,
        10,
        "[estimate]\nlower = 8.0\nupper = 8.0\n[value]\nvalue = 10.0\n",
    );
    seed(root, 11, "[value]\nvalue = 10.0\n");

    let before = explain(root, &iss(11));
    assert!(
        !before.contains("est_cost"),
        "no est engagement ⇒ no cost-source block: {before}"
    );
    let score_before = score_line(&before);

    // ISS-010 is more work than ISS-011 ⇒ ISS-011 projects below the 8.0 anchor.
    // PHASE-06: add an anchor-form claim row for ISS-010 so the est system
    // has a claim-derived anchor (the old facet anchor is dead).
    capture_cost_anchor(root, "a10", &iss(10), 8.0, 8.0);
    capture_more_work(root, "mw", &iss(10), &iss(11), "prefer-a");

    let after = explain(root, &iss(11));
    let score_after = score_line(&after);
    assert_ne!(
        score_before, score_after,
        "the fed projected cost moves the score: {score_before} → {score_after}"
    );
    assert!(
        after.contains("est_cost")
            && after.contains("projected · bounds")
            && after.contains("constraining sizing judgements (1 human, 0 agent)"),
        "ISS-011 shows the projected cost-source block: {after}"
    );
    // The anchored claim shows the human-claim shape (rater=human → tier=Human).
    assert!(
        explain(root, &iss(10)).contains("est_cost 8.0 — human claim [8.0 ‥ 8.0] · β 0.65"),
        "ISS-010 shows the human claim shape"
    );
}

/// The `score: T (…)` line of an `explain` render (the shift assertion's key).
fn score_line(explain_out: &str) -> String {
    explain_out
        .lines()
        .find(|l| l.trim_start().starts_with("score:"))
        .unwrap_or_default()
        .to_string()
}

// =============================================================================
// EX-2 / §6.10 — full probe round-trip (elicit → record → re-elicit shows sized)
// =============================================================================

/// EX-2: a bare top-K item with an authored-estimate calibration target raises a
/// `sizing-probe` (frame `more-work`, domain `estimate`, the exact answer
/// command). Answering `equal` lands the subject in the target's anchored class;
/// the next elicit no longer probes it, and `explain` shows it sized via the
/// class anchor (provenance Authored, no own facet — §4).
#[test]
fn sizing_probe_round_trip_lands_the_subject_sized() {
    let dir = project();
    let root = dir.path();
    seed(root, 1, "[value]\nvalue = 10.0\n"); // bare subject
    seed(
        root,
        2,
        "[estimate]\nlower = 4.0\nupper = 4.0\n[value]\nvalue = 10.0\n",
    ); // target
    // PHASE-06: add an anchor-form claim row for the target so the est
    // system has a claim-derived anchor (the authored facet still grounds
    // the target pool via estimated_costs).
    capture_cost_anchor(root, "a2", &iss(2), 4.0, 4.0);

    // The probe surfaces: subject, target, ask (more-work / estimate), command.
    let json = elicit_json(root, &["--kind", "sizing-probe", "--json"]);
    let entries = json["entries"].as_array().unwrap();
    let probe = entries
        .iter()
        .find(|e| e["subject"]["id"] == "ISS-001")
        .expect("a sizing probe for the bare subject");
    assert_eq!(probe["kind"], "sizing-probe");
    assert_eq!(probe["target"]["id"], "ISS-002");
    assert_eq!(probe["target"]["estimate"], 4.0);
    assert_eq!(probe["ask"]["frame"], "more-work");
    assert_eq!(probe["ask"]["domain"], "estimate");

    // The human render carries the spine, ask line, and the exact answer command.
    let human = elicit(root, &["--kind", "sizing-probe"]);
    assert!(
        human.contains("[sizing-probe]")
            && human.contains("which is more work vs ISS-002")
            && human.contains("doctrine compare record ISS-001 ISS-002")
            && human.contains("--frame more-work"),
        "probe human render carries the answer command: {human}"
    );

    // Answer `equal` (hand-authored — the jail adaptation) merges the subject
    // into the target's anchored cost class.
    capture_more_work(root, "ans", &iss(1), &iss(2), "equal");

    let after = elicit_json(root, &["--kind", "sizing-probe", "--json"]);
    assert!(
        after["entries"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["subject"]["id"] != "ISS-001"),
        "the answered subject no longer raises a sizing probe: {after}"
    );
    assert!(
        explain(root, &iss(1)).contains("est_cost 4.0 — anchored (via class anchor)"),
        "the subject is sized via the target's class anchor"
    );
}

// =============================================================================
// §6.9 — domain-tagged finding render + JSON parity (est AnchorConflict, D1/D9)
// =============================================================================

/// §6.9 / D9: an est-domain anchor conflict renders with the `[estimate]` domain
/// tag and the D1 wording (sizing evidence contradicts the β-resolved costs),
/// and the `--json` finding carries the matching `domain` discriminator — the
/// value-domain findings stay UNtagged (extend, don't replace).
#[test]
fn est_anchor_conflict_is_domain_tagged_with_json_parity() {
    let dir = project();
    let root = dir.path();
    // A(1) > B > C(3) in cost, contradicting the 1/3 anchors (design §5.3, est).
    seed(
        root,
        20,
        "[estimate]\nlower = 1.0\nupper = 1.0\n[value]\nvalue = 10.0\n",
    );
    seed(root, 21, "[value]\nvalue = 10.0\n");
    seed(
        root,
        22,
        "[estimate]\nlower = 3.0\nupper = 3.0\n[value]\nvalue = 10.0\n",
    );
    // PHASE-06: anchor-form claim rows for the two anchor items.
    capture_cost_anchor(root, "a20", &iss(20), 1.0, 1.0);
    capture_cost_anchor(root, "a22", &iss(22), 3.0, 3.0);
    capture_more_work(root, "ab", &iss(20), &iss(21), "prefer-a");
    capture_more_work(root, "bc", &iss(21), &iss(22), "prefer-a");

    let human = ok(&run(root, &["findings"]));
    assert!(
        human.contains("[estimate] anchors ISS-020=1.0 vs ISS-022=3.0 conflict")
            && human.contains("sizing evidence contradicts the β-resolved costs")
            && human.contains("revise the estimate or supersede the row"),
        "est AnchorConflict carries the domain tag + D1 wording: {human}"
    );

    let json: serde_json::Value =
        serde_json::from_str(&ok(&run(root, &["findings", "--json"]))).unwrap();
    let findings = json["findings"].as_array().unwrap();
    let conflict = findings
        .iter()
        .find(|f| f["kind"] == "anchor conflicts" && f["domain"] == "estimate")
        .expect("an est-domain anchor-conflict finding in JSON");
    let anchors: Vec<&str> = conflict["anchors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["entity"].as_str().unwrap())
        .collect();
    assert!(
        anchors.contains(&"ISS-020") && anchors.contains(&"ISS-022"),
        "JSON parity: both suspect anchors carried: {conflict}"
    );
}
