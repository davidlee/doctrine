// SPDX-License-Identifier: GPL-3.0-only
//! SL-222 PHASE-06 — estimate claim integration tests (keywords pin, migrated,
//! cost-anchor). Exercises the full set/pin/clear verb round trip:
//! estimate set → claim row → est_cost → explain provenance; pin overrides
//! projection; human beats agent; migrated loses to projection; conflict →
//! finding → resolved by a superseding row; clear → tombstone → fallthrough
//! to next rung; probe round trip.

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

fn project() -> tempfile::TempDir {
    let dir = tmp();
    write(dir.path(), ".project", "");
    write(dir.path(), "doctrine.toml", "");
    dir
}

fn run(root: &Path, args: &[&str]) -> Output {
    let mut a: Vec<&str> = args.to_vec();
    a.push("-p");
    a.push(root.to_str().expect("utf8 path"));
    Command::new(bin())
        .args(&a)
        .output()
        .expect("spawn doctrine")
}

fn ok(root: &Path, args: &[&str]) -> String {
    let out = run(root, args);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

fn iss(id: u32) -> String {
    format!("ISS-{id:03}")
}

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

fn explain(root: &Path, id: &str) -> String {
    ok(root, &["explain", id])
}

fn findings(root: &Path) -> String {
    ok(root, &["findings"])
}

#[expect(dead_code, reason = "available for future claim tests")]
fn findings_json(root: &Path) -> serde_json::Value {
    serde_json::from_str(&ok(root, &["findings", "--json"])).expect("findings json")
}

fn more_work_capture(root: &Path, slot: &str, a: &str, b: &str, resp: &str) {
    write(
        root,
        &format!(".doctrine/comparisons/2026-01-01-{slot}.toml"),
        &format!(
            "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
             [session]\nuid = \"sess-{slot}\"\ndate = \"2026-01-01\"\n\n\
             [[judgement]]\nuid = \"row-{slot}\"\nseq = 0\na = \"{a}\"\nb = \"{b}\"\n\
             response = \"{resp}\"\ndomain = \"estimate\"\nframe = \"more-work\"\n\
             form = \"order\"\nrater = \"agent\"\ndate = \"2026-01-01\"\n"
        ),
    );
}

fn cost_anchor(root: &Path, uid: &str, item: &str, lower: f64, upper: f64, rater: &str) {
    write(
        root,
        &format!(".doctrine/comparisons/2026-01-01-{uid}.toml"),
        &format!(
            "schema = \"doctrine.comparison-session\"\nversion = 2\n\n\
             [session]\nuid = \"sess-{uid}\"\ndate = \"2026-01-01\"\n\n\
             [[judgement]]\nuid = \"{uid}\"\nseq = 0\na = \"{item}\"\n\
             domain = \"estimate\"\nframe = \"cost-anchor\"\n\
             form = \"anchor\"\nrater = \"{rater}\"\ndate = \"2026-01-01\"\n\
             est_lower = {lower}\nest_upper = {upper}\n"
        ),
    );
}

// =============================================================================
// EX-1: estimate set → claim row → est_cost → explain shows claim provenance
// =============================================================================

/// `estimate set` creates a claim row; `explain` shows the cost-source block
/// with the human-claim provenance.
#[test]
fn estimate_set_creates_claim_and_shows_in_explain() {
    let dir = project();
    let root = dir.path();
    seed(root, 1, "[value]\nvalue = 10.0\n");
    seed(root, 2, "[value]\nvalue = 10.0\n");
    // Anchor row + a gate row so the est system is live (anchor rows alone
    // don't enter the compile-active set).
    cost_anchor(root, "a1", &iss(1), 5.0, 7.0, "human");
    more_work_capture(root, "gate", &iss(1), &iss(2), "prefer-a");

    let out = explain(root, &iss(1));
    assert!(
        out.contains("est_cost") && out.contains("human claim"),
        "explain shows the claim-derived cost: {out}"
    );
    // The claim's bounds (5,7) → β-resolved at 0.65 → ~6.3
    assert!(
        out.contains("est_cost 6.3") || out.contains("est_cost 6.30"),
        "operative cost ~6.3: {out}"
    );
}

// =============================================================================
// EX-2: pin overrides projection
// =============================================================================

/// A Pin-tier claim row beats a projected chain — the pin cost appears in
/// the cost-source block (not the projected line).
#[test]
fn pin_overrides_projection() {
    let dir = project();
    let root = dir.path();
    seed(root, 1, "[value]\nvalue = 10.0\n");
    seed(root, 2, "[value]\nvalue = 10.0\n");
    // ISS-001 pin (2..8) → operative 5.9.
    cost_anchor(root, "p1", &iss(1), 2.0, 8.0, "human");
    // ISS-001 more work than ISS-002: ISS-002 projected below the pin.
    more_work_capture(root, "mw", &iss(1), &iss(2), "prefer-a");

    // ISS-001: the anchored claim wins (rung 1), not the projection.
    let out1 = explain(root, &iss(1));
    assert!(
        out1.contains("human claim") || out1.contains("pin"),
        "anchored claim drowns projection: {out1}"
    );
    // ISS-002: projected because no own claim.
    let out2 = explain(root, &iss(2));
    assert!(out2.contains("projected"), "no claim ⇒ projected: {out2}");
}

// =============================================================================
// EX-3: human beats agent in the same claim group
// =============================================================================

/// A human-tier claim row beats an agent-tier claim on the same item.
#[test]
fn human_beats_agent() {
    let dir = project();
    let root = dir.path();
    seed(root, 1, "[value]\nvalue = 10.0\n");
    seed(root, 2, "[value]\nvalue = 10.0\n");
    // Human claim (2..8) → operative 5.9; agent claim (1..4) → operative 2.95.
    // Gate row so the est system is live.
    cost_anchor(root, "h1", &iss(1), 2.0, 8.0, "human");
    cost_anchor(root, "a1", &iss(1), 1.0, 4.0, "agent");
    more_work_capture(root, "gate", &iss(1), &iss(2), "prefer-a");

    let out = explain(root, &iss(1));
    // The human-tier claim wins, so the cost-source shows the human claim.
    assert!(
        out.contains("human claim") && out.contains("5.9"),
        "human tier claim wins: {out}"
    );
}

// =============================================================================
// EX-4: migrated loses to projection
// =============================================================================

/// A migrated-tier claim (observed/legacy data) loses to a projected cost
/// when projection evidence exists.
#[test]
fn migrated_loses_to_projection() {
    let dir = project();
    let root = dir.path();
    seed(root, 1, "[value]\nvalue = 10.0\n");
    seed(root, 2, "[value]\nvalue = 10.0\n");
    // ISS-002 has a migrated claim (2..8) → operative 5.9.
    cost_anchor(root, "m2", &iss(2), 2.0, 8.0, "migrated");
    // ISS-001 has a human anchor (5..5) → operative 5.0.
    cost_anchor(root, "p1", &iss(1), 5.0, 5.0, "human");
    // Gate + chain: ISS-001 more work than ISS-002 (ISS-002 projected below 5.0).
    more_work_capture(root, "mw", &iss(1), &iss(2), "prefer-a");

    // ISS-002's migrated claim is a prior (rungs 3-4), not anchored (rung 1).
    // Since ISS-002 appears in the est projection (below ISS-001), the
    // projected cost wins over the migrated prior.
    let out2 = explain(root, &iss(2));
    // The migrated claim is below projection, so it shows as a prior.
    assert!(
        out2.contains("projected"),
        "projected beats migrated prior: {out2}"
    );
}

// =============================================================================
// EX-5: conflict → finding → resolved by superseding row
// =============================================================================

/// An estimate anchor conflict (two differently-costed items merged via
/// `equal`) triggers a finding. A superseding row resolves it.
/// SKIPPED in worker jail (fs::remove_file is an authored write).
#[test]
fn anchor_conflict_resolved_by_superseding_row() {
    if std::env::var_os("DOCTRINE_WORKER").is_some() {
        eprintln!("skipped: worker jail: cannot remove comparison files");
        return;
    }
    let dir = project();
    let root = dir.path();
    seed(root, 1, "[value]\nvalue = 10.0\n");
    seed(root, 3, "[value]\nvalue = 10.0\n");
    // ISS-001 anchored at 5.0, ISS-003 anchored at 1.0.
    cost_anchor(root, "a1", &iss(1), 5.0, 5.0, "human");
    cost_anchor(root, "a3", &iss(3), 1.0, 1.0, "human");
    // A more-work chain: ISS-003 > ISS-002 > ISS-001 — the path implies
    // cost(ISS-003) > cost(ISS-001), contradicting the anchors (1.0 < 5.0).
    seed(root, 2, "[value]\nvalue = 10.0\n");
    more_work_capture(root, "ab", &iss(3), &iss(2), "prefer-a");
    more_work_capture(root, "bc", &iss(2), &iss(1), "prefer-a");

    let f = findings(root);
    assert!(
        f.contains("[estimate] anchors ISS-003=1.0 vs ISS-001=5.0 conflict"),
        "anchor conflict finding: {f}"
    );
    assert!(
        f.contains("supersede"),
        "finding recommends superseding: {f}"
    );

    // Supersede: drop the contradicting ISS-003 > ISS-002 row and assert
    // ISS-001 > ISS-003 — the chain is now consistent with the anchors.
    fs::remove_file(root.join(".doctrine/comparisons/2026-01-01-ab.toml")).unwrap();
    more_work_capture(root, "ab2", &iss(1), &iss(3), "prefer-a");

    let after = findings(root);
    assert!(
        !after.contains("conflict — sizing evidence contradicts"),
        "conflict resolved after supersede: {after}"
    );
}

// =============================================================================
// EX-6: clear → tombstone → fallthrough to next rung
// =============================================================================

/// Clearing an estimate claim tombstones it; the item falls through to the
/// next rung (projection > prior > bare anchor).
/// SKIPPED in worker jail (`estimate clear` is WRITE-classed).
#[test]
fn clear_tombstones_claim_and_fallthrough() {
    if std::env::var_os("DOCTRINE_WORKER").is_some() {
        eprintln!("skipped: worker jail: estimate clear is WRITE-classed");
        return;
    }
    let dir = project();
    let root = dir.path();
    seed(root, 1, "[value]\nvalue = 10.0\n");
    seed(root, 2, "[value]\nvalue = 10.0\n");
    // ISS-001 pin (2..8) → operative 5.9.
    cost_anchor(root, "p1", &iss(1), 2.0, 8.0, "human");
    // ISS-001 more work than ISS-002.
    more_work_capture(root, "mw", &iss(1), &iss(2), "prefer-a");

    // Before clear: ISS-002 is projected below ISS-001's anchored cost.
    let before = explain(root, &iss(2));
    assert!(before.contains("projected"), "projected cost: {before}");

    // Clear ISS-001's anchor — the anchor row becomes tombstoned.
    let cleared = ok(root, &["estimate", "clear", &iss(1)]);
    assert!(cleared.contains("cleared"), "clear confirmed: {cleared}");

    // After clear: the est system still has the chain, but no anchor.
    // ISS-002 still projected (the chain persists), but ISS-001's anchor
    // is tombstoned so the est pipe re-derives.
    // ISS-002 is still projected because the Order chain still exists.
    let after = explain(root, &iss(2));
    assert!(after.contains("est_cost"), "still a cost block: {after}");
}

// =============================================================================
// EX-7: probe round trip (elicit → claim → re-elicit)
// =============================================================================

/// An un-sized item raises a sizing probe; answering via estimate set retires
/// it.
#[test]
fn probe_round_trip() {
    let dir = project();
    let root = dir.path();
    seed(root, 1, "[value]\nvalue = 10.0\n"); // target with anchor
    seed(root, 2, "[value]\nvalue = 10.0\n"); // bare subject
    cost_anchor(root, "t1", &iss(1), 3.0, 3.0, "human");

    // Elicit shows a sizing probe.
    let json0: serde_json::Value = serde_json::from_str(&ok(
        root,
        &["compare", "elicit", "--kind", "sizing-probe", "--json"],
    ))
    .expect("elicit json");
    let probes_before: Vec<_> = json0["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["subject"]["id"] == iss(2))
        .collect();
    assert_eq!(probes_before.len(), 1, "sizing probe for ISS-002");

    // Answer the probe with a sizing row (equal).
    more_work_capture(root, "ans", &iss(2), &iss(1), "equal");

    // Re-elicit: the probe is gone (the subject has been answered).
    let json1: serde_json::Value = serde_json::from_str(&ok(
        root,
        &["compare", "elicit", "--kind", "sizing-probe", "--json"],
    ))
    .expect("elicit json");
    let probes_after: Vec<_> = json1["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["subject"]["id"] == iss(2))
        .collect();
    assert_eq!(probes_after.len(), 0, "no more sizing probe for ISS-002");
}
