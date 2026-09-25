// SPDX-License-Identifier: GPL-3.0-only
//! `design tree` and `design show --format tree` — the read surface for the
//! whole inquiry map (SL-266 PHASE-03, design `VT-10`).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::Path;
use std::time::{Duration, SystemTime};

mod common;
mod design_fixture;

use design_fixture::{
    DesignRun, SLICE, SLICE_NUMBER, fail, run, seed_slice_record, seed_slice_record_as, start_run,
};

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
