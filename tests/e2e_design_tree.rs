// SPDX-License-Identifier: GPL-3.0-only
//! `design tree` and `design show --format tree` — the read surface for the
//! whole inquiry map (SL-266 PHASE-03, design `VT-10`) — and the relay line that
//! delivers it after a map-changing write (SL-266 PHASE-04, design `VT-12`).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::time::{Duration, SystemTime};

use serde_json::{Value, json};

mod common;
mod design_fixture;
mod runbook_fixture;

use design_fixture::{
    DesignRun, SLICE, SLICE_NUMBER, fail, run, seed_slice_record, seed_slice_record_as, start_run,
};
use runbook_fixture::{EXPLORING_STEPS, discharge_body};

const ESCAPE: char = '\u{1b}';

/// Start a run for slice `number` in `root` with its slice record at `status`,
/// its snapshot stamped `age_secs` before now; return the snapshot path.
fn open_run(root: &Path, number: &str, status: &str, age_secs: u64) -> std::path::PathBuf {
    seed_slice_record_as(root, number, status);
    let (_uid, snapshot) = start_run(root, &format!("SL-{number}"));
    stamp(&snapshot, age_secs);
    snapshot
}

fn stamp(path: &Path, age_secs: u64) {
    let when = SystemTime::now() - Duration::from_secs(age_secs);
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(when)
        .unwrap();
}

/// `design tree` with the global `--color` choice in front.
fn tree(root: &Path, color: &str, extra: &[&str]) -> String {
    let mut args = vec!["--color", color, "design", "tree", "-p", "."];
    args.extend_from_slice(extra);
    run(root, &args)
}

/// `EX-1`: the verb with a slice is the format; `--color` reaches both paths.
#[test]
fn design_tree_matches_show_format_tree() {
    let fixture = DesignRun::start();
    seed_slice_record(&fixture.root, SLICE_NUMBER);
    for color in ["never", "always"] {
        let by_verb = tree(&fixture.root, color, &[SLICE]);
        let by_format = run(
            &fixture.root,
            &[
                "--color", color, "design", "show", SLICE, "-p", ".", "--format", "tree",
            ],
        );
        assert_eq!(by_verb, by_format, "--color {color}");
        assert_eq!(
            by_verb.contains(ESCAPE),
            color == "always",
            "--color {color}"
        );
    }
    let footer = tree(&fixture.root, "never", &[SLICE]);
    assert!(
        footer
            .trim_end()
            .ends_with(&format!("doctrine design tree {SLICE}")),
        "the footer names the command: {footer}"
    );
}

/// `DEC-305`: no slice → the newest open run, disclosed; a locked-out done
/// slice and a malformed snapshot are not chosen, and the malformed one is
/// named with its cause.
#[test]
fn design_tree_without_a_slice_picks_the_newest_open_run() {
    let fixture = DesignRun::start();
    seed_slice_record(&fixture.root, SLICE_NUMBER);
    stamp(&fixture.snapshot, 300);
    open_run(&fixture.root, "234", "started", 200);
    open_run(&fixture.root, "235", "done", 10);
    let malformed = open_run(&fixture.root, "247", "started", 5);
    std::fs::write(&malformed, "not = [ toml").unwrap();

    for color in ["never", "always"] {
        let out = tree(&fixture.root, color, &[]);
        assert_eq!(out.contains(ESCAPE), color == "always", "--color {color}");
    }
    let out = tree(&fixture.root, "never", &[]);
    let lines: Vec<&str> = out.lines().collect();
    assert!(
        lines[0].starts_with("SL-234 "),
        "newest open run heads: {out}"
    );
    assert_eq!(lines[1], "chosen: newest of 2 runs open when scanned");
    assert!(
        lines[2].starts_with("skipped .doctrine/state/slice/247/design.toml: "),
        "the malformed snapshot is disclosed with its cause: {out}"
    );
    assert!(out.trim_end().ends_with("doctrine design tree SL-234"));
}

/// With nothing open, the refusal names the verb that takes a slice and still
/// lists what it could not read.
#[test]
fn design_tree_with_no_open_run_refuses_naming_the_slice_form() {
    let fixture = DesignRun::start();
    seed_slice_record_as(&fixture.root, SLICE_NUMBER, "abandoned");
    let malformed = open_run(&fixture.root, "247", "started", 5);
    std::fs::write(&malformed, "not = [ toml").unwrap();

    let err = fail(&fixture.root, &["design", "tree", "-p", "."]);
    assert!(err.contains("`doctrine design tree SL-NNN`"), "{err}");
    assert!(
        err.contains("skipped .doctrine/state/slice/247/design.toml: "),
        "{err}"
    );
}

/// The tree is an envelope rendering, so the document's `--json` is refused.
#[test]
fn format_tree_refuses_json() {
    let fixture = DesignRun::start();
    let err = fail(
        &fixture.root,
        &[
            "design", "show", SLICE, "-p", ".", "--format", "tree", "--json",
        ],
    );
    assert!(err.contains("--format tree"), "{err}");
}

/// RV-392 `F-1`: a snapshot filed under one slice's directory but naming
/// another is not judged by either slice's status — it is skipped, with the
/// mismatch as its cause.
#[test]
fn design_tree_skips_a_snapshot_filed_under_another_slice() {
    let fixture = DesignRun::start();
    seed_slice_record(&fixture.root, SLICE_NUMBER);
    let done = open_run(&fixture.root, "234", "done", 5);
    std::fs::copy(&done, &fixture.snapshot).unwrap();

    let err = fail(&fixture.root, &["design", "tree", "-p", "."]);
    assert!(
        err.contains("skipped .doctrine/state/slice/233/design.toml: ") && err.contains("SL-234"),
        "the misfiled snapshot is disclosed, naming the slice it claims: {err}"
    );
}

/// RV-392 `F-2`: a state directory that only parses as a slice number
/// (`0233`) is not a second copy of slice 233's run.
#[test]
fn design_tree_counts_each_run_once() {
    let fixture = DesignRun::start();
    seed_slice_record(&fixture.root, SLICE_NUMBER);
    let slice_dir = fixture.snapshot.parent().unwrap();
    std::fs::create_dir(slice_dir.with_file_name(format!("0{SLICE_NUMBER}"))).unwrap();

    let out = tree(&fixture.root, "never", &[]);
    assert_eq!(
        out.lines().nth(1),
        Some("chosen: the only run open when scanned"),
        "{out}"
    );
}

// ── the relay line (SL-266 PHASE-04, `DEC-309` / `DEC-310`) ────────────────

/// The line a map-changing write prints last in relay mode, for [`SLICE`].
fn relay_line() -> String {
    format!(
        "map changed — before ending this turn, show the user the output of \
         `doctrine design tree {SLICE}` verbatim"
    )
}

/// The last line of a command's stdout.
fn last_line(out: &str) -> &str {
    out.lines().last().unwrap_or_default()
}

/// A payload for `fixture`'s run at its current revision: the envelope plus
/// `body`'s keys.
fn payload(fixture: &DesignRun, submission: &str, body: &Value) -> String {
    let text = std::fs::read_to_string(&fixture.snapshot).unwrap();
    let snapshot: toml::Table = toml::from_str(&text).unwrap();
    let revision = snapshot["run"]["revision"].as_integer().unwrap();
    let mut object = json!({
        "run_uid": fixture.uid,
        "known_revision": revision,
        "submission_id": submission,
    });
    for (key, value) in body.as_object().unwrap() {
        object[key] = value.clone();
    }
    object.to_string()
}

/// `design apply` with an already-built payload, expecting success.
fn apply(fixture: &DesignRun, payload: &str) -> String {
    run(
        &fixture.root,
        &["design", "apply", SLICE, "-p", ".", "--input", payload],
    )
}

/// One new, open, non-blocking inquiry node.
fn create_node(subject: &str) -> Value {
    json!({"declare": [{"subject": subject, "question": "what now?", "blocking": false}]})
}

/// A legacy `design.md` whose Open Questions seed two inquiry nodes on import.
const LEGACY_DESIGN: &str = "# Design SL-233: Relay fixture

## 1. Open Questions

- **OQ-1:** whether the relay fires on import.
- **OQ-2:** whether it fires last.
";

/// A throwaway tree holding a legacy `design.md` for [`SLICE`] and, when given,
/// a `doctrine.toml` body. No run is started.
fn legacy_tree(doctrine_toml: Option<&str>) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let slice_dir = tmp.path().join(common::SLICE_DIR).join(SLICE_NUMBER);
    std::fs::create_dir_all(&slice_dir).unwrap();
    std::fs::write(slice_dir.join("design.md"), LEGACY_DESIGN).unwrap();
    if let Some(body) = doctrine_toml {
        write_config(tmp.path(), body);
    }
    tmp
}

/// Write the project `doctrine.toml` in `root`.
fn write_config(root: &Path, body: &str) {
    let path = root.join(common::DOCTRINE_TOML);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

fn start_from_design(root: &Path) -> String {
    run(
        root,
        &["design", "start", SLICE, "--from-design", "-p", "."],
    )
}

/// `VT-12`, default config: a map-changing apply ends with the relay line; a
/// replay of it, and a step discharge, do not.
#[test]
fn relay_line_closes_a_map_changing_apply_only() {
    let fixture = DesignRun::start();

    let created = payload(&fixture, "create", &create_node("inq-1"));
    let out = apply(&fixture, &created);
    assert_eq!(last_line(&out), relay_line(), "{out}");
    assert_eq!(out.matches("map changed").count(), 1, "{out}");

    let replay = apply(&fixture, &created);
    assert!(replay.starts_with("resumed submission"), "{replay}");
    assert!(
        !replay.contains("map changed"),
        "a replay owes nothing: {replay}"
    );

    let discharge = payload(&fixture, "discharge", &discharge_body(EXPLORING_STEPS[0]));
    let out = apply(&fixture, &discharge);
    assert!(
        !out.contains("map changed"),
        "a discharge is not a map change: {out}"
    );
}

/// `VT-12`: `start --from-design` that imports nodes ends with the relay line.
#[test]
fn relay_line_closes_a_node_importing_start() {
    let tmp = legacy_tree(None);
    let out = start_from_design(tmp.path());
    assert_eq!(last_line(&out), relay_line(), "{out}");
}

/// `VT-12`: `design adopt` re-seats sections and never touches the map, so a
/// written adopt carries no relay line.
#[test]
fn a_written_adopt_carries_no_relay_line() {
    let tmp = legacy_tree(None);
    let root = tmp.path();
    start_from_design(root);
    run(root, &["design", "materialise", SLICE, "-p", "."]);
    let design = root
        .join(common::SLICE_DIR)
        .join(SLICE_NUMBER)
        .join("design.md");
    let materialised = std::fs::read_to_string(&design).unwrap();
    let edited = materialised.replace("fires last", "fires last, edited by hand");
    assert_ne!(edited, materialised, "the edit lands in a section body");
    std::fs::write(&design, edited).unwrap();

    let out = run(root, &["design", "adopt", SLICE, "-p", "."]);
    assert!(!out.contains("nothing to adopt"), "the adopt wrote: {out}");
    assert!(!out.contains("map changed"), "{out}");
}

/// `VT-12`: in sidecar mode no write prints the relay line.
#[test]
fn sidecar_mode_prints_no_relay_line() {
    let sidecar = "[design]\nmap_delivery = \"sidecar\"\n";
    let tmp = legacy_tree(Some(sidecar));
    let out = start_from_design(tmp.path());
    assert!(!out.contains("map changed"), "start: {out}");

    let fixture = DesignRun::start();
    write_config(&fixture.root, sidecar);
    let out = apply(
        &fixture,
        &payload(&fixture, "create", &create_node("inq-1")),
    );
    assert!(!out.contains("map changed"), "apply: {out}");
}

/// `VT-12`: every malformed `[design]` entry refuses `design apply` and `design
/// start` before writing, naming what is accepted — while an unrelated reader of
/// the same `doctrine.toml` (the `[conduct]` load) still succeeds.
#[test]
fn a_malformed_design_entry_refuses_the_writes_before_writing() {
    for entry in [
        "[design]\nmap_delivery = \"tree\"\n",
        "[design]\nmap_delivery = 42\n",
        "[design]\nmap_delivry = \"relay\"\n",
        "design = 1\n",
    ] {
        let fixture = DesignRun::start();
        seed_slice_record_as(&fixture.root, SLICE_NUMBER, "proposed");
        write_config(&fixture.root, entry);
        let before = std::fs::read(&fixture.snapshot).unwrap();

        let body = payload(&fixture, "create", &create_node("inq-1"));
        let err = fail(
            &fixture.root,
            &["design", "apply", SLICE, "-p", ".", "--input", &body],
        );
        assert!(
            err.contains("map_delivery = \"relay\" | \"sidecar\""),
            "{entry:?}: {err}"
        );
        assert_eq!(
            std::fs::read(&fixture.snapshot).unwrap(),
            before,
            "{entry:?}"
        );

        let err = fail(&fixture.root, &["design", "start", "SL-234", "-p", "."]);
        assert!(
            err.contains("map_delivery = \"relay\" | \"sidecar\""),
            "{entry:?}: {err}"
        );
        let other = fixture.snapshot.parent().unwrap().with_file_name("234");
        assert!(!other.exists(), "{entry:?}: start wrote nothing");

        run(
            &fixture.root,
            &["slice", "status", SLICE_NUMBER, "design", "-p", "."],
        );
    }
}
