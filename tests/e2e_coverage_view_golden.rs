//! SL-045 PHASE-06 — `coverage` + `spec req list` black-box CLI goldens and the
//! INV-1 / F1 seam wall (design §5.5 / §9 VT-1).
//!
//! These pin the two read-only views at the CLI surface over the BUILT binary:
//!   * byte-exact table goldens (fixed-date / non-git temp roots ⇒ deterministic),
//!   * JSON shape assertions (parsed, key presence/absence — robust to field order),
//!   * the dangling-member INV-4 / E1 contract (forbidden keys explicitly absent),
//!   * the column selector + its SL-037 unknown-column error, and the bad-ref error.
//!
//! The headline is `req_status_cell_is_fixed_while_verdict_moves` — the F1 / NF-001
//! wall. It drives the BUILT `coverage` command across the full reachable on-disk
//! coverage spectrum for ONE requirement at a FIXED authored `status`, and asserts
//! the rendered status cell is byte-identical every time while observed/verdict DO
//! move. That proves status is sourced only from `requirement::load`, never folded
//! out of the coverage scan. A pure `observed_state` helper test cannot witness this
//! wall — it must knock on the command seam, so it runs `coverage` and parses output.
//!
//! Determinism: requirement/spec authored status is hand-seeded; the temp root is
//! NOT a git repo, so the scan resolves HEAD as unborn → staleness `Unknown` for
//! every cell. That is graceful and deterministic. observed/verdict still move with
//! the entry status (planned/in-progress → forward/Coherent; failed/blocked →
//! contradicted/Divergent; absent coverage → none/Coherent), which is all the wall
//! needs while the authored status cell stays fixed.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

// --- seed helpers ---------------------------------------------------------

/// Hand-seed one requirement's authored tree.
fn seed_requirement(root: &Path, id: u32, slug: &str, title: &str, status: &str, kind: &str) {
    let dir = root.join(format!(".doctrine/requirement/{id:03}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("requirement-{id:03}.toml")),
        format!(
            "schema = \"doctrine.requirement\"\n\
             version = 1\n\
             id = {id}\n\
             slug = \"{slug}\"\n\
             title = \"{title}\"\n\
             status = \"{status}\"\n\
             kind = \"{kind}\"\n\
             description = \"{title} description.\"\n\
             tags = []\n\
             acceptance_criteria = []\n"
        ),
    )
    .unwrap();
}

/// Hand-seed a product spec (`PRD-NNN`) shell with the given members block.
fn seed_product_spec(root: &Path, id: u32, slug: &str, title: &str, members: &str) {
    let dir = root.join(format!(".doctrine/spec/product/{id:03}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("spec-{id:03}.toml")),
        format!(
            "schema = \"doctrine.spec.product\"\n\
             version = 1\n\
             id = {id}\n\
             slug = \"{slug}\"\n\
             title = \"{title}\"\n\
             status = \"active\"\n\
             kind = \"product\"\n\
             category = \"test\"\n\
             tags = []\n\
             responsibilities = []\n"
        ),
    )
    .unwrap();
    fs::write(dir.join("members.toml"), members).unwrap();
}

/// Hand-seed a tech spec (`SPEC-NNN`) shell with the given members block.
fn seed_tech_spec(root: &Path, id: u32, slug: &str, title: &str, members: &str) {
    let dir = root.join(format!(".doctrine/spec/tech/{id:03}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("spec-{id:03}.toml")),
        format!(
            "schema = \"doctrine.spec.tech\"\n\
             version = 1\n\
             id = {id}\n\
             slug = \"{slug}\"\n\
             title = \"{title}\"\n\
             status = \"active\"\n\
             kind = \"tech\"\n\
             category = \"test\"\n\
             tags = []\n\
             responsibilities = []\n"
        ),
    )
    .unwrap();
    fs::write(dir.join("members.toml"), members).unwrap();
}

/// Write a coverage.toml under a slice tree with a single entry at `entry_status`,
/// pointing at requirement REQ-`req`.
fn seed_coverage(root: &Path, slice_id: u32, req: &str, entry_status: &str) {
    let dir = root.join(format!(".doctrine/slice/{slice_id:03}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("coverage.toml"),
        format!(
            "[[entry]]\n\
             slice = \"SL-{slice_id:03}\"\n\
             requirement = \"{req}\"\n\
             contributing_change = \"SL-{slice_id:03}\"\n\
             mode = \"VT\"\n\
             status = \"{entry_status}\"\n\
             git_anchor = \"anchor-x\"\n"
        ),
    )
    .unwrap();
}

/// Remove a slice's coverage.toml (the "absent coverage" arm of the spectrum).
fn clear_coverage(root: &Path, slice_id: u32) {
    let p = root.join(format!(".doctrine/slice/{slice_id:03}/coverage.toml"));
    let _ = fs::remove_file(p);
}

// --- run / output helpers -------------------------------------------------

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .arg("-p")
        .arg(root)
        .output()
        .expect("spawn doctrine")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}
fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}
fn json(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).expect("parse json stdout")
}

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

// =========================================================================
// 1. INV-1 / F1 SEAM WALL — the headline (design §5.5 / §9 VT-1).
// =========================================================================

/// Drive the BUILT `coverage` command across the full reachable on-disk coverage
/// spectrum for ONE requirement at a FIXED authored `status` (`pending`). The
/// rendered status cell MUST be byte-identical every time, while observed/verdict
/// MOVE. This is the F1 / NF-001 proof: status is sourced only from
/// `requirement::load`, never derived from the coverage fold. Isolated to the cell
/// via `--columns status` (and cross-checked against the full `--columns
/// status,observed,verdict` row so the move is witnessed, not just the fixity).
#[test]
fn req_status_cell_is_fixed_while_verdict_moves() {
    let dir = tmp();
    let root = dir.path();
    seed_requirement(root, 1, "alpha", "Alpha", "pending", "functional");

    // The reachable spectrum in a non-git temp root. (`verified` is omitted: with
    // an unborn HEAD it collapses to stale/Indeterminate, so it would not add a
    // distinct verdict here — the moving arms below already exercise the read.)
    let spectrum: &[(&str, &str)] = &[
        ("planned", "forward"),
        ("in-progress", "forward"),
        ("failed", "contradicted"),
        ("blocked", "contradicted"),
        ("<none>", "none"),
    ];

    let mut observed_cells = std::collections::BTreeSet::new();
    for (entry_status, expect_observed) in spectrum {
        if *entry_status == "<none>" {
            clear_coverage(root, 60);
        } else {
            seed_coverage(root, 60, "REQ-001", entry_status);
        }

        // The wall: status cell, isolated, must be exactly "status\npending\n".
        let cell = run(
            root,
            &["coverage", "show", "REQ-001", "--columns", "status"],
        );
        assert!(cell.status.success(), "stderr: {}", stderr(&cell));
        assert_eq!(
            stdout(&cell),
            "status\npending\n",
            "status cell must be FIXED at authored `pending` for entry status `{entry_status}` \
             — status comes from requirement::load, never the fold (F1/NF-001)"
        );

        // The live read: observed/verdict DO move across the spectrum.
        let row = run(
            root,
            &[
                "coverage",
                "show",
                "REQ-001",
                "--columns",
                "status,observed,verdict",
            ],
        );
        assert!(row.status.success(), "stderr: {}", stderr(&row));
        let body = stdout(&row);
        let data = body.lines().nth(1).expect("a data row");
        assert!(
            data.contains(expect_observed),
            "entry status `{entry_status}`: expected observed `{expect_observed}` in `{data}`"
        );
        observed_cells.insert((*expect_observed).to_string());
    }

    // Sanity: the spectrum genuinely MOVED the observed cell (≥3 distinct states),
    // so the fixed status cell above is a real wall, not a constant input.
    assert!(
        observed_cells.len() >= 3,
        "observed cell must move across the spectrum to witness the live read; saw {observed_cells:?}"
    );
}

// =========================================================================
// 2. Black-box goldens — every surface (design §5.2).
// =========================================================================

/// `coverage REQ-NNN` table: single row, NO label column. Byte-exact.
#[test]
fn coverage_req_table_is_byte_exact_no_label_column() {
    let dir = tmp();
    let root = dir.path();
    seed_requirement(root, 1, "alpha", "Alpha", "pending", "functional");
    // No coverage.toml ⇒ observed `none`, verdict `Coherent` — deterministic.

    let out = run(root, &["coverage", "show", "REQ-001"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "id      │ status  │ observed │ verdict\n\
         REQ-001 │ pending │ none     │ Coherent\n"
    );
}

/// `coverage SPEC-NNN` table: member fan in member `order`, label column PRESENT.
/// Members are seeded OUT of order to prove the `order` sort (FR-001 before NF-001).
#[test]
fn coverage_spec_table_member_fan_in_order_with_label_column() {
    let dir = tmp();
    let root = dir.path();
    seed_requirement(root, 1, "alpha", "Alpha", "pending", "functional");
    seed_requirement(root, 2, "beta", "Beta", "active", "quality");
    seed_tech_spec(
        root,
        70,
        "probe-tech",
        "Probe Tech",
        // NF-001 (order 2) listed first on disk; FR-001 (order 1) second.
        "[[member]]\nrequirement = \"REQ-002\"\nlabel = \"NF-001\"\norder = 2\n\n\
         [[member]]\nrequirement = \"REQ-001\"\nlabel = \"FR-001\"\norder = 1\n",
    );

    let out = run(root, &["coverage", "show", "SPEC-070"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "label  │ id      │ status  │ observed │ verdict\n\
         FR-001 │ REQ-001 │ pending │ none     │ Coherent\n\
         NF-001 │ REQ-002 │ active  │ none     │ Indeterminate\n"
    );
}

/// `coverage REQ-NNN --json` healthy row: parsed. Keys present; label/divergent_reason
/// ABSENT when not applicable; envelope `{kind:"coverage", rows:[…]}`.
#[test]
fn coverage_req_json_healthy_row_keys() {
    let dir = tmp();
    let root = dir.path();
    seed_requirement(root, 1, "alpha", "Alpha", "pending", "functional");

    let out = run(root, &["coverage", "show", "REQ-001", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v = json(&out);
    assert_eq!(v["kind"], "coverage");
    let rows = v["rows"].as_array().expect("rows array");
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    let obj = row.as_object().expect("row object");

    // present
    assert_eq!(
        obj.get("requirement").and_then(|x| x.as_str()),
        Some("REQ-001")
    );
    assert_eq!(obj.get("kind").and_then(|x| x.as_str()), Some("functional"));
    assert_eq!(obj.get("status").and_then(|x| x.as_str()), Some("pending"));
    assert_eq!(obj.get("observed").and_then(|x| x.as_str()), Some("none"));
    assert_eq!(
        obj.get("verdict").and_then(|x| x.as_str()),
        Some("Coherent")
    );
    // absent when not applicable: a bare REQ has no membership label; Coherent has
    // no divergent reason.
    assert!(obj.get("label").is_none(), "no label on a bare REQ row");
    assert!(
        obj.get("divergent_reason").is_none(),
        "no divergent_reason on a Coherent row"
    );
}

/// `coverage SPEC-NNN --json` with a DANGLING member (FK has no requirement dir):
/// the dangling row MUST be `{requirement, label, dangling:true, load_error}` with
/// NO `status`/`observed`/`verdict`/`kind` keys. The healthy member still renders;
/// the fan does not abort. INV-4 / E1 contract.
#[test]
fn coverage_spec_json_dangling_member_omits_typed_keys() {
    let dir = tmp();
    let root = dir.path();
    seed_requirement(root, 1, "alpha", "Alpha", "pending", "functional");
    // REQ-999 has NO requirement dir → dangling.
    seed_product_spec(
        root,
        50,
        "probe",
        "Probe",
        "[[member]]\nrequirement = \"REQ-001\"\nlabel = \"FR-001\"\norder = 1\n\n\
         [[member]]\nrequirement = \"REQ-999\"\nlabel = \"FR-002\"\norder = 2\n",
    );

    let out = run(root, &["coverage", "show", "PRD-050", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v = json(&out);
    assert_eq!(v["kind"], "coverage");
    let rows = v["rows"].as_array().expect("rows array");
    assert_eq!(rows.len(), 2, "fan continues past the dangling member");

    // healthy row unaffected
    let healthy = rows[0].as_object().unwrap();
    assert_eq!(
        healthy.get("requirement").and_then(|x| x.as_str()),
        Some("REQ-001")
    );
    assert_eq!(
        healthy.get("status").and_then(|x| x.as_str()),
        Some("pending")
    );

    // dangling row: present keys
    let dangling = rows[1].as_object().unwrap();
    assert_eq!(
        dangling.get("requirement").and_then(|x| x.as_str()),
        Some("REQ-999")
    );
    assert_eq!(
        dangling.get("label").and_then(|x| x.as_str()),
        Some("FR-002")
    );
    assert_eq!(
        dangling.get("dangling").and_then(|x| x.as_bool()),
        Some(true)
    );
    assert!(
        dangling
            .get("load_error")
            .and_then(|x| x.as_str())
            .is_some(),
        "load_error must be present on a dangling row"
    );
    // FORBIDDEN keys — the E1 contract. Assert explicitly absent.
    for forbidden in ["status", "observed", "verdict", "kind"] {
        assert!(
            dangling.get(forbidden).is_none(),
            "dangling row must NOT carry `{forbidden}` — INV-4/E1 contract leak"
        );
    }
}

/// `coverage SPEC-NNN` table with a dangling member: the dangling row renders an
/// inline load-error note in place of the typed cells; the fan continues (the
/// healthy member row is present and complete). The absolute path inside the note
/// floats per run, so assert structurally rather than byte-exact.
#[test]
fn coverage_spec_table_dangling_member_inline_error_fan_continues() {
    let dir = tmp();
    let root = dir.path();
    seed_requirement(root, 1, "alpha", "Alpha", "pending", "functional");
    seed_product_spec(
        root,
        50,
        "probe",
        "Probe",
        "[[member]]\nrequirement = \"REQ-001\"\nlabel = \"FR-001\"\norder = 1\n\n\
         [[member]]\nrequirement = \"REQ-999\"\nlabel = \"FR-002\"\norder = 2\n",
    );

    let out = run(root, &["coverage", "show", "PRD-050"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let body = stdout(&out);
    let lines: Vec<&str> = body.lines().collect();
    assert_eq!(lines.len(), 3, "header + healthy row + dangling row");
    // The inline load-error note carries the absolute tempdir path, which floats
    // per run AND widens the whole column — so assert on whitespace-split cells,
    // not fixed-width byte prefixes.
    assert_eq!(
        lines[0].split_whitespace().collect::<Vec<_>>(),
        [
            "label", "│", "id", "│", "status", "│", "observed", "│", "verdict"
        ]
    );
    // healthy member renders fully (fan did not abort): label/id + the typed cells.
    let healthy: Vec<&str> = lines[1].split_whitespace().collect();
    assert_eq!(
        healthy,
        [
            "FR-001", "│", "REQ-001", "│", "pending", "│", "none", "│", "Coherent"
        ]
    );
    // dangling member renders the inline load-error note in place of typed cells.
    assert!(
        lines[2].starts_with("FR-002 │ REQ-999 │ "),
        "dangling row label/id: {}",
        lines[2]
    );
    assert!(
        lines[2].contains("requirement REQ-999 not found"),
        "dangling row inline load-error note: {}",
        lines[2]
    );
}

/// `coverage REQ-NNN --columns id,verdict` selects + orders columns (byte-exact).
#[test]
fn coverage_columns_selects_and_orders() {
    let dir = tmp();
    let root = dir.path();
    seed_requirement(root, 1, "alpha", "Alpha", "pending", "functional");

    let out = run(
        root,
        &["coverage", "show", "REQ-001", "--columns", "id,verdict"],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "id      │ verdict\n\
         REQ-001 │ Coherent\n"
    );
}

/// `coverage REQ-NNN --columns bogus` → SL-037 unknown-column error + non-zero exit.
#[test]
fn coverage_columns_unknown_errors() {
    let dir = tmp();
    let root = dir.path();
    seed_requirement(root, 1, "alpha", "Alpha", "pending", "functional");

    let out = run(root, &["coverage", "show", "REQ-001", "--columns", "bogus"]);
    assert!(!out.status.success());
    assert_eq!(
        stderr(&out),
        "Error: unknown column `bogus` (available: id, label, kind, status, observed, verdict)\n"
    );
}

/// `coverage NOPE-1` → not-a-coverage-ref error + non-zero exit.
#[test]
fn coverage_bad_ref_errors() {
    let dir = tmp();
    let out = run(dir.path(), &["coverage", "show", "NOPE-1"]);
    assert!(!out.status.success());
    assert_eq!(
        stderr(&out),
        "Error: `NOPE-1` is not a coverage ref (expected REQ-/PRD-/SPEC-NNN)\n"
    );
}

// =========================================================================
// 3. spec req list — the authored roster (no observed/verdict column).
// =========================================================================

/// `spec req list SPEC-NNN` table: authored roster id/label/kind/status, member
/// order, NO observed/verdict column. Byte-exact.
#[test]
fn spec_req_list_table_roster_byte_exact() {
    let dir = tmp();
    let root = dir.path();
    seed_requirement(root, 1, "alpha", "Alpha", "pending", "functional");
    seed_requirement(root, 2, "beta", "Beta", "active", "quality");
    seed_tech_spec(
        root,
        70,
        "probe-tech",
        "Probe Tech",
        "[[member]]\nrequirement = \"REQ-002\"\nlabel = \"NF-001\"\norder = 2\n\n\
         [[member]]\nrequirement = \"REQ-001\"\nlabel = \"FR-001\"\norder = 1\n",
    );

    let out = run(root, &["spec", "req", "list", "SPEC-070"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "id      │ label  │ kind       │ status\n\
         REQ-001 │ FR-001 │ functional │ pending\n\
         REQ-002 │ NF-001 │ quality    │ active\n"
    );
}

/// `spec req list SPEC-NNN --json`: faithful rows.
#[test]
fn spec_req_list_json_faithful_rows() {
    let dir = tmp();
    let root = dir.path();
    seed_requirement(root, 1, "alpha", "Alpha", "pending", "functional");
    seed_requirement(root, 2, "beta", "Beta", "active", "quality");
    seed_tech_spec(
        root,
        70,
        "probe-tech",
        "Probe Tech",
        "[[member]]\nrequirement = \"REQ-001\"\nlabel = \"FR-001\"\norder = 1\n\n\
         [[member]]\nrequirement = \"REQ-002\"\nlabel = \"NF-001\"\norder = 2\n",
    );

    let out = run(root, &["spec", "req", "list", "SPEC-070", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v = json(&out);
    assert_eq!(v["kind"], "requirement");
    let rows = v["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), 2);
    let r0 = rows[0].as_object().unwrap();
    assert_eq!(r0.get("id").and_then(|x| x.as_str()), Some("REQ-001"));
    assert_eq!(r0.get("label").and_then(|x| x.as_str()), Some("FR-001"));
    assert_eq!(r0.get("kind").and_then(|x| x.as_str()), Some("functional"));
    assert_eq!(r0.get("status").and_then(|x| x.as_str()), Some("pending"));
    // The authored roster carries NO observed/verdict — those are coverage-only.
    assert!(
        r0.get("observed").is_none(),
        "roster row has no observed column"
    );
    assert!(
        r0.get("verdict").is_none(),
        "roster row has no verdict column"
    );
}

/// `spec req list SPEC-NNN --columns id,status` selects columns (byte-exact).
#[test]
fn spec_req_list_columns_selects() {
    let dir = tmp();
    let root = dir.path();
    seed_requirement(root, 1, "alpha", "Alpha", "pending", "functional");
    seed_tech_spec(
        root,
        70,
        "probe-tech",
        "Probe Tech",
        "[[member]]\nrequirement = \"REQ-001\"\nlabel = \"FR-001\"\norder = 1\n",
    );

    let out = run(
        root,
        &["spec", "req", "list", "SPEC-070", "--columns", "id,status"],
    );
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "id      │ status\n\
         REQ-001 │ pending\n"
    );
}

/// `spec req list SPEC-NNN --columns bogus` → unknown-column error + non-zero exit.
/// (The roster's available set is the authored four — no observed/verdict.)
#[test]
fn spec_req_list_columns_unknown_errors() {
    let dir = tmp();
    let root = dir.path();
    seed_tech_spec(
        root,
        70,
        "probe-tech",
        "Probe Tech",
        "[[member]]\nrequirement = \"REQ-001\"\nlabel = \"FR-001\"\norder = 1\n",
    );
    seed_requirement(root, 1, "alpha", "Alpha", "pending", "functional");

    let out = run(
        root,
        &["spec", "req", "list", "SPEC-070", "--columns", "bogus"],
    );
    assert!(!out.status.success());
    assert_eq!(
        stderr(&out),
        "Error: unknown column `bogus` (available: id, label, kind, status)\n"
    );
}

/// A roster over a spec with a DANGLING member renders a degraded row and does NOT
/// abort: the healthy member is fully present, the dangling member carries the
/// load-error note in its typed cells, and exit is success. (Table structural +
/// JSON forbidden-key check.)
#[test]
fn spec_req_list_dangling_member_degrades_does_not_abort() {
    let dir = tmp();
    let root = dir.path();
    seed_requirement(root, 1, "alpha", "Alpha", "pending", "functional");
    seed_product_spec(
        root,
        50,
        "probe",
        "Probe",
        "[[member]]\nrequirement = \"REQ-001\"\nlabel = \"FR-001\"\norder = 1\n\n\
         [[member]]\nrequirement = \"REQ-999\"\nlabel = \"FR-002\"\norder = 2\n",
    );

    // table: success, both rows present, dangling row notes the error inline.
    let out = run(root, &["spec", "req", "list", "PRD-050"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let lines: Vec<String> = stdout(&out).lines().map(str::to_owned).collect();
    assert_eq!(lines.len(), 3, "header + healthy + degraded");
    // The degraded row's load-error note carries (and column-widens to) the floating
    // absolute path — split on whitespace for the healthy row's typed cells.
    assert_eq!(
        lines[1].split_whitespace().collect::<Vec<_>>(),
        ["REQ-001", "│", "FR-001", "│", "functional", "│", "pending"]
    );
    assert!(lines[2].starts_with("REQ-999 │ FR-002 │ "));
    assert!(
        lines[2].contains("load error"),
        "degraded row note: {}",
        lines[2]
    );

    // json: dangling row omits the typed `kind`/`status` keys.
    let outj = run(root, &["spec", "req", "list", "PRD-050", "--json"]);
    assert!(outj.status.success(), "stderr: {}", stderr(&outj));
    let v = json(&outj);
    let rows = v["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), 2);
    let dangling = rows[1].as_object().unwrap();
    assert_eq!(dangling.get("id").and_then(|x| x.as_str()), Some("REQ-999"));
    assert_eq!(
        dangling.get("label").and_then(|x| x.as_str()),
        Some("FR-002")
    );
    assert_eq!(
        dangling.get("dangling").and_then(|x| x.as_bool()),
        Some(true)
    );
    assert!(dangling.get("load_error").is_some(), "load_error present");
    for forbidden in ["kind", "status"] {
        assert!(
            dangling.get(forbidden).is_none(),
            "degraded roster row must NOT carry `{forbidden}`"
        );
    }
}
