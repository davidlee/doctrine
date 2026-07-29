// SPDX-License-Identifier: GPL-3.0-only
//! SL-231 PHASE-03 — CLI end-to-end tests for `doctrine observation`.
//!
//! Black-box tests over the BUILT binary against a TEMPORARY git repository.
//! Covers record, show, list, search, supersede, retract, receipts, replay,
//! hostile content, correction chains, and strict failure paths.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "integration test: fail-fast unwrap/expect/panic/index are idiomatic"
)]

use std::path::Path;
use std::process::{Command, Output};

mod common;

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

/// Run `git -C <dir> <args>`, asserting success; returns trimmed stdout.
fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("spawn git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A doctrine-rooted git repo with one commit.
fn init_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    std::fs::create_dir_all(dir.join(".doctrine")).unwrap();
    std::fs::write(dir.join("a.txt"), "hello").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "base"]);
}

/// Spawn `doctrine <args...>` in `cwd`. `worker_env` decides the env leg of the
/// worker-mode predicate (`marker.rs` `describe_mode`): set it for the pi-arm
/// dispatched shape, clear it for everything else. Cleared explicitly rather
/// than inherited, so a `DOCTRINE_WORKER` in the runner's own environment
/// cannot silently reclassify an unrelated test.
fn spawn(cwd: &Path, args: &[&str], worker_env: bool) -> Output {
    let mut cmd = common::doctrine_cmd(cwd);
    cmd.args(args);
    if worker_env {
        cmd.env("DOCTRINE_WORKER", "1");
    } else {
        cmd.env_remove("DOCTRINE_WORKER");
    }
    cmd.output().expect("spawn doctrine")
}

/// Run `doctrine <args...>` in `cwd`, unsetting DOCTRINE_WORKER.
fn run(cwd: &Path, args: &[&str]) -> Output {
    spawn(cwd, args, false)
}

/// Run `doctrine <args...>` in `cwd` with `DOCTRINE_WORKER=1` — the env leg the
/// pi-arm dispatched worker carries alongside the marker.
fn run_as_env_worker(cwd: &Path, args: &[&str]) -> Output {
    spawn(cwd, args, true)
}

/// Run `doctrine <args...>` in `cwd` with `stdin_text` piped to standard input.
///
/// The `--input -` sentinel is only reachable through a real pipe, so this is a
/// spawn-and-write rather than a variant of [`spawn`]: a test that merely passes
/// `-` without writing would hang on an inherited terminal instead of failing.
fn run_with_stdin(cwd: &Path, args: &[&str], stdin_text: &str) -> Output {
    use std::io::Write as _;
    use std::process::Stdio;

    let mut child = common::doctrine_cmd(cwd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn doctrine");
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(stdin_text.as_bytes())
        .expect("write stdin");
    child.wait_with_output().expect("wait for doctrine")
}

/// Look up one stored facet field, `None` when the group or field is absent.
///
/// Total where TOML indexing panics, so a test can assert a field is ABSENT —
/// which is the whole point of the `enrich: false` cases.
fn facet<'a>(stored: &'a toml::Value, group: &str, field: &str) -> Option<&'a toml::Value> {
    stored
        .get("facets")
        .and_then(|f| f.get(group))
        .and_then(|g| g.get(field))
}

/// Read back the record a receipt points at, as parsed TOML.
///
/// Receipts carry `rel_path`, so a test asserting what was *stored* — facets and
/// their origins especially — reads the authored file rather than trusting the
/// receipt, which only reports the outcome.
fn stored_record(dir: &Path, receipt: &serde_json::Value) -> toml::Value {
    let rel = receipt["rel_path"]
        .as_str()
        .expect("receipt carries rel_path");
    let raw = std::fs::read_to_string(dir.join(rel)).expect("stored record is readable");
    raw.parse::<toml::Value>().expect("stored record is TOML")
}

/// Run with explicit path.
#[expect(dead_code, reason = "available for future tests")]
fn run_path(cwd: &Path, path: &Path, args: &[&str]) -> Output {
    let mut full_args: Vec<&str> = vec!["-p"];
    let path_str = path.to_string_lossy();
    full_args.push(&path_str);
    full_args.extend(args);
    spawn(cwd, &full_args, false)
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf8 stdout")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf8 stderr")
}

/// Parse JSON from stdout.
fn json_stdout(out: &Output) -> serde_json::Value {
    serde_json::from_str(&stdout(out)).expect("valid JSON stdout")
}

// ── Record tests ──────────────────────────────────────────────────────────

#[test]
fn record_friction_returns_receipt() {
    let dir = tmp();
    init_repo(dir.path());

    let out = run(
        dir.path(),
        &["observation", "record", "friction", "test friction"],
    );
    assert!(
        out.status.success(),
        "record must succeed: {}",
        stderr(&out)
    );

    let receipt = json_stdout(&out);
    assert_eq!(receipt["kind"], "friction");
    assert_eq!(receipt["outcome"], "created");
    assert!(receipt["uid"].is_string());
    assert!(receipt["recorded_at"].is_string());
    assert!(receipt["rel_path"].is_string());
    assert!(
        receipt["rel_path"].as_str().unwrap().contains("records/"),
        "rel_path must contain records/"
    );

    // Record file exists on disk.
    let rel = receipt["rel_path"].as_str().unwrap();
    let abs = dir.path().join(rel);
    assert!(abs.is_file(), "record file must exist at {abs:?}");
}

#[test]
fn record_replay_and_collision() {
    let dir = tmp();
    init_repo(dir.path());

    // First write.
    let uid = "019f1234-5678-7abc-8def-0123456789ab";
    let out1 = run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "same summary",
            "--uid",
            uid,
        ],
    );
    assert!(out1.status.success());
    let r1 = json_stdout(&out1);
    assert_eq!(r1["outcome"], "created");

    // Same intent → replay.
    let out2 = run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "same summary",
            "--uid",
            uid,
        ],
    );
    assert!(out2.status.success());
    let r2 = json_stdout(&out2);
    assert_eq!(r2["outcome"], "replayed");
    assert_eq!(r2["uid"], uid);

    // Different summary → collision.
    let out3 = run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "different summary",
            "--uid",
            uid,
        ],
    );
    assert!(!out3.status.success(), "collision must fail");
    assert!(
        stderr(&out3).contains("identity collision"),
        "must report identity collision"
    );
}

#[test]
fn record_does_not_touch_git_index() {
    let dir = tmp();
    init_repo(dir.path());

    // Record an observation.
    let out = run(dir.path(), &["observation", "record", "friction", "test"]);
    assert!(out.status.success());

    // The observation file must not be staged.
    let status = git(dir.path(), &["status", "--porcelain"]);
    // We expect the records directory to be untracked (??), not staged.
    // On a fresh repo, `git status --porcelain` may show nothing if .gitignore
    // covers it, or ?? for untracked. But after a commit, any new file not in
    // .gitignore shows as ??.
    // The critical assertion: no staged file.
    for line in status.lines() {
        assert!(
            !line.starts_with("A ") && !line.starts_with("M ") && !line.starts_with("D "),
            "no file should be staged, got: {line}"
        );
    }
}

#[test]
fn hostile_content_is_safely_rendered() {
    let dir = tmp();
    init_repo(dir.path());

    // Test with ANSI escape sequences (no NUL).
    let ansi_summary = "normal \x1b[31mRED\x1b[0m text \x07 bell";
    let out = run(
        dir.path(),
        &["observation", "record", "friction", ansi_summary],
    );
    assert!(out.status.success(), "ANSI summary should record ok");

    let receipt = json_stdout(&out);
    let uid = receipt["uid"].as_str().unwrap();

    // Show the record — output must escape hostile content.
    let show = run(dir.path(), &["observation", "show", uid]);
    assert!(show.status.success());

    let show_out = stdout(&show);
    // ANSI escape sequences must not appear as literal escape bytes.
    assert!(
        !show_out.contains('\x1b'),
        "ANSI escapes must not appear literally in terminal output"
    );
    // The content should be framed as untrusted.
    assert!(
        show_out.contains("untrusted input"),
        "output must frame content as untrusted"
    );
}

#[test]
fn row_injection_is_defeated_in_list_table() {
    let dir = tmp();
    init_repo(dir.path());

    // Summary with embedded newline shaped to look like a second table row.
    let summary = "legit\nFAKE-UID | 2026-01-01T00:00:00Z | friction | injected";
    let out = run(dir.path(), &["observation", "record", "friction", summary]);
    assert!(
        out.status.success(),
        "row-injection summary should record ok"
    );

    let list = run(dir.path(), &["observation", "list"]);
    assert!(list.status.success());
    let list_out = stdout(&list);

    // The literal backslash-n escape sequence must appear in the output
    // (proof the newline was escaped), and no raw newline should break the
    // summary cell into multiple visual rows.
    assert!(
        list_out.contains("\\n"),
        "literal \\n must appear in rendered summary cell\noutput:\n{list_out}"
    );
    // The injected fake fields must NOT appear outside a cell boundary;
    // they should be glued to "legit" via the escaped newline.
    assert!(
        list_out.contains("legit\\nFAKE-UID"),
        "escaped newline must keep injection text in same cell\noutput:\n{list_out}"
    );
}

#[test]
fn non_ascii_content_survives_rendering_intact() {
    let dir = tmp();
    init_repo(dir.path());

    // Accented Latin, em-dash, CJK, emoji — all must survive the escaper
    // round-trip without corruption.
    let summary = "résumé — 日本語 🎉";
    let out = run(dir.path(), &["observation", "record", "friction", summary]);
    assert!(out.status.success(), "non-ASCII summary should record ok");

    let receipt = json_stdout(&out);
    let uid = receipt["uid"].as_str().unwrap();

    // Show — summary must appear verbatim.
    let show = run(dir.path(), &["observation", "show", uid]);
    assert!(show.status.success());
    let show_out = stdout(&show);
    assert!(
        show_out.contains(summary),
        "non-ASCII summary must appear verbatim in show output\noutput:\n{show_out}"
    );

    // List table — summary must appear verbatim.
    let list = run(dir.path(), &["observation", "list"]);
    assert!(list.status.success());
    let list_out = stdout(&list);
    assert!(
        list_out.contains(summary),
        "non-ASCII summary must appear verbatim in list output\noutput:\n{list_out}"
    );
}

#[test]
fn record_with_detail_and_no_enrich() {
    let dir = tmp();
    init_repo(dir.path());

    let out = run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "summary text",
            "--detail",
            "detail text",
            "--no-enrich",
        ],
    );
    assert!(out.status.success());
    let receipt = json_stdout(&out);
    assert_eq!(receipt["outcome"], "created");
}

#[test]
fn record_with_explicit_uid() {
    let dir = tmp();
    init_repo(dir.path());

    let uid = "019faaaa-5678-7abc-8def-0123456789ab";
    let out = run(
        dir.path(),
        &["observation", "record", "friction", "test", "--uid", uid],
    );
    assert!(out.status.success());
    let receipt = json_stdout(&out);
    assert_eq!(receipt["uid"], uid);
}

// ── Explicit facet fields and complete requests (IMP-332, design §3.1) ────

#[test]
fn record_facet_flags_land_as_explicit_beside_automatic_enrichment() {
    let dir = tmp();
    init_repo(dir.path());

    let out = run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "explicit facets",
            "--facet",
            "execution.harness=claude",
            "--facet",
            "work_context.slice=SL-231",
        ],
    );
    assert!(
        out.status.success(),
        "record must succeed: {}",
        stderr(&out)
    );

    let stored = stored_record(dir.path(), &json_stdout(&out));
    let execution = &stored["facets"]["execution"];
    assert_eq!(execution["harness"].as_str(), Some("claude"));
    assert_eq!(
        execution["harness_origin"].as_str(),
        Some("explicit"),
        "a caller-supplied value is explicit by construction"
    );
    assert_eq!(
        execution["interface_origin"].as_str(),
        Some("automatic"),
        "automatic enrichment must survive alongside explicit fields, not be replaced by them"
    );
    assert_eq!(
        stored["facets"]["work_context"]["slice"].as_str(),
        Some("SL-231"),
        "a second --facet must reach a different group"
    );
}

#[test]
fn record_facet_explicit_value_wins_over_automatic_enrichment() {
    let dir = tmp();
    init_repo(dir.path());

    // `execution.command` is one the CLI adapter enriches automatically, so
    // shadowing it is what proves design §3.1's "explicit caller values win" is
    // reachable from this surface at all — the gap IMP-332 was raised for.
    let out = run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "shadowed",
            "--facet",
            "execution.command=caller said so",
        ],
    );
    assert!(
        out.status.success(),
        "record must succeed: {}",
        stderr(&out)
    );

    let stored = stored_record(dir.path(), &json_stdout(&out));
    assert_eq!(
        stored["facets"]["execution"]["command"].as_str(),
        Some("caller said so")
    );
    assert_eq!(
        stored["facets"]["execution"]["command_origin"].as_str(),
        Some("explicit")
    );
}

#[test]
fn record_facet_cannot_forge_the_origin_of_its_own_value() {
    let dir = tmp();
    init_repo(dir.path());

    // Origin is the only provenance discriminator the corpus carries, so a
    // caller able to stamp `automatic` on its own value would make every
    // origin-partitioned statistic unsound (RV-318 F-2). Pinned at the CLI
    // entry point, not just in the merge.
    let out = run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "forged",
            "--facet",
            "execution.harness=claude",
            "--facet",
            "execution.harness_origin=automatic",
        ],
    );
    assert!(
        out.status.success(),
        "record must succeed: {}",
        stderr(&out)
    );

    let stored = stored_record(dir.path(), &json_stdout(&out));
    assert_eq!(
        stored["facets"]["execution"]["harness_origin"].as_str(),
        Some("explicit"),
        "origin is derived from reaching the merge, never read from the caller"
    );
}

#[test]
fn record_facet_typed_field_names_the_argument_and_points_at_input() {
    let dir = tmp();
    init_repo(dir.path());

    let out = run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "typed",
            "--facet",
            "usage.total_tokens=500",
        ],
    );
    assert!(!out.status.success(), "a u64 field takes no string value");

    let err = stderr(&out);
    assert!(
        err.contains("usage.total_tokens=500"),
        "serde reports no field path for a type mismatch, so the refusal must \
         name the offending argument: {err}"
    );
    assert!(
        err.contains("--input"),
        "and must name the way through: {err}"
    );
}

#[test]
fn record_facet_unknown_field_is_refused_not_dropped() {
    let dir = tmp();
    init_repo(dir.path());

    let out = run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "unknown",
            "--facet",
            "execution.nonsuch=v",
        ],
    );
    assert!(
        !out.status.success(),
        "silently dropping a facet the caller asked for would record a lie \
         about what was captured"
    );
    assert!(stderr(&out).contains("nonsuch"), "{}", stderr(&out));
}

#[test]
fn record_input_reads_a_complete_request_from_a_file() {
    let dir = tmp();
    init_repo(dir.path());

    let req = dir.path().join("req.json");
    std::fs::write(
        &req,
        r#"{"summary":"from a file","detail":"d",
            "facets":{"execution":{"harness":"pi"}},"enrich":false}"#,
    )
    .unwrap();

    let out = run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "--input",
            req.to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "record must succeed: {}",
        stderr(&out)
    );

    let stored = stored_record(dir.path(), &json_stdout(&out));
    assert_eq!(stored["summary"].as_str(), Some("from a file"));
    assert_eq!(stored["detail"].as_str(), Some("d"));
    assert_eq!(
        stored["facets"]["execution"]["harness"].as_str(),
        Some("pi")
    );
    assert!(
        facet(&stored, "execution", "interface").is_none(),
        "`enrich: false` in the request must suppress automatic enrichment"
    );
}

#[test]
fn record_input_dash_reads_a_complete_request_from_stdin() {
    let dir = tmp();
    init_repo(dir.path());

    let out = run_with_stdin(
        dir.path(),
        &["observation", "record", "friction", "--input", "-"],
        r#"{"summary":"from stdin","facets":{"work_context":{"phase":"PHASE-03"}}}"#,
    );
    assert!(
        out.status.success(),
        "record must succeed: {}",
        stderr(&out)
    );

    let stored = stored_record(dir.path(), &json_stdout(&out));
    assert_eq!(stored["summary"].as_str(), Some("from stdin"));
    assert_eq!(
        stored["facets"]["work_context"]["phase"].as_str(),
        Some("PHASE-03")
    );
    assert_eq!(
        stored["facets"]["execution"]["interface_origin"].as_str(),
        Some("automatic"),
        "an absent `enrich` must still mean enrich"
    );
}

#[test]
fn record_input_carries_the_typed_fields_the_facet_flags_cannot() {
    let dir = tmp();
    init_repo(dir.path());

    // The complement of `record_facet_typed_field_names_the_argument…`: the
    // route that refusal points at must actually work, or the diagnostic is
    // advice to a dead end.
    let out = run_with_stdin(
        dir.path(),
        &["observation", "record", "friction", "--input", "-"],
        r#"{"summary":"typed",
            "facets":{"correlation":{"related_observations":["a","b"]}}}"#,
    );
    assert!(
        out.status.success(),
        "record must succeed: {}",
        stderr(&out)
    );

    let stored = stored_record(dir.path(), &json_stdout(&out));
    assert_eq!(
        stored["facets"]["correlation"]["related_observations"]
            .as_array()
            .map(Vec::len),
        Some(2),
        "a Vec<String> field is reachable through the request but not the flags"
    );
}

#[test]
fn record_input_cannot_be_combined_with_the_per_field_flags() {
    let dir = tmp();
    init_repo(dir.path());

    // Two sources of truth for one field would re-open "explicit caller values
    // win" at a layer with no origin to record the answer in.
    for extra in [
        vec!["a summary"],
        vec!["--detail", "d"],
        vec!["--uid", "019faaaa-5678-7abc-8def-0123456789ab"],
        vec!["--facet", "execution.harness=claude"],
        vec!["--no-enrich"],
    ] {
        let mut args = vec!["observation", "record", "friction", "--input", "-"];
        args.extend(extra.iter());
        let out = run_with_stdin(dir.path(), &args, r#"{"summary":"s"}"#);
        assert!(
            !out.status.success(),
            "--input must refuse {extra:?}: {}",
            stdout(&out)
        );
    }
}

#[test]
fn record_input_malformed_json_is_refused_on_one_line() {
    let dir = tmp();
    init_repo(dir.path());

    let out = run_with_stdin(
        dir.path(),
        &["observation", "record", "friction", "--input", "-"],
        "{not json",
    );
    assert!(!out.status.success(), "malformed input must be refused");
    assert!(
        stderr(&out).contains("invalid --input request"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn record_input_unknown_key_is_refused_rather_than_ignored() {
    let dir = tmp();
    init_repo(dir.path());

    let out = run_with_stdin(
        dir.path(),
        &["observation", "record", "friction", "--input", "-"],
        r#"{"summary":"s","path":"/etc/passwd"}"#,
    );
    assert!(
        !out.status.success(),
        "a key past the request contract must not be silently dropped"
    );
    assert!(stderr(&out).contains("path"), "{}", stderr(&out));
}

#[test]
fn record_input_missing_file_is_refused_with_the_path() {
    let dir = tmp();
    init_repo(dir.path());

    let out = run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "--input",
            "nonsuch.json",
        ],
    );
    assert!(!out.status.success(), "a missing request file must refuse");
    assert!(stderr(&out).contains("nonsuch.json"), "{}", stderr(&out));
}

#[test]
fn record_friction_valid_summary_produces_receipt() {
    let dir = tmp();
    init_repo(dir.path());

    // A valid friction record produces a valid JSON receipt.
    let out = run(dir.path(), &["observation", "record", "friction", "valid"]);
    assert!(out.status.success());
    let _r = json_stdout(&out); // ensure valid JSON receipt
}

// ── Show tests ────────────────────────────────────────────────────────────

#[test]
fn show_resolves_correction_chain() {
    let dir = tmp();
    init_repo(dir.path());

    // Create three friction records.
    let uid_a = "019faaaa-5678-7abc-8def-0123456789ab";
    let uid_b = "019fbbbb-5678-7abc-8def-0123456789cd";
    let uid_c = "019fcccc-5678-7abc-8def-0123456789ef";

    run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "original",
            "--uid",
            uid_a,
        ],
    )
    .assert_success();
    run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "replacement",
            "--uid",
            uid_b,
        ],
    )
    .assert_success();
    run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "replacement-2",
            "--uid",
            uid_c,
        ],
    )
    .assert_success();

    // Supersede a → b, then b → c.
    run(dir.path(), &["observation", "supersede", uid_a, uid_b]).assert_success();
    run(dir.path(), &["observation", "supersede", uid_b, uid_c]).assert_success();

    // Show resolved — a should show superseded by b.
    let show = run(dir.path(), &["observation", "show", uid_a]);
    assert!(show.status.success());
    let show_out = stdout(&show);
    assert!(show_out.contains("superseded by"), "must show superseded");
    assert!(show_out.contains(uid_b), "must reference replacement");
    assert!(
        show_out.contains("correction chain"),
        "must show correction chain"
    );
}

#[test]
fn show_raw_skips_resolution() {
    let dir = tmp();
    init_repo(dir.path());

    let uid = "019faaaa-5678-7abc-8def-0123456789ab";
    run(
        dir.path(),
        &["observation", "record", "friction", "test", "--uid", uid],
    )
    .assert_success();

    let show = run(dir.path(), &["observation", "show", uid, "--raw"]);
    assert!(show.status.success());
    let show_out = stdout(&show);
    // Raw output shouldn't mention resolution.
    assert!(
        !show_out.contains("superseded by"),
        "raw mode must not show resolution"
    );
}

#[test]
fn show_json_format() {
    let dir = tmp();
    init_repo(dir.path());

    let uid = "019faaaa-5678-7abc-8def-0123456789ab";
    run(
        dir.path(),
        &["observation", "record", "friction", "test", "--uid", uid],
    )
    .assert_success();

    let show = run(dir.path(), &["observation", "show", uid, "--json"]);
    assert!(show.status.success());
    let val = json_stdout(&show);
    assert_eq!(val["uid"], uid);
}

#[test]
fn show_exact_lookup_works_regardless_of_active_state() {
    let dir = tmp();
    init_repo(dir.path());

    let uid = "019faaaa-5678-7abc-8def-0123456789ab";
    run(
        dir.path(),
        &["observation", "record", "friction", "test", "--uid", uid],
    )
    .assert_success();

    // Retract it.
    run(dir.path(), &["observation", "retract", uid]).assert_success();

    // Exact lookup still works.
    let show = run(dir.path(), &["observation", "show", uid]);
    assert!(show.status.success());
    let show_out = stdout(&show);
    assert!(show_out.contains("retracted"), "must show retracted state");
}

// ── List / Search tests ───────────────────────────────────────────────────

#[test]
fn list_and_search_use_resolved_projection() {
    let dir = tmp();
    init_repo(dir.path());

    let uid_a = "019faaaa-5678-7abc-8def-0123456789ab";
    let uid_b = "019fbbbb-5678-7abc-8def-0123456789cd";

    run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "active record",
            "--uid",
            uid_a,
        ],
    )
    .assert_success();
    run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "superseded record",
            "--uid",
            uid_b,
        ],
    )
    .assert_success();

    // Retract b.
    run(dir.path(), &["observation", "retract", uid_b]).assert_success();

    // List — default active projection should exclude retracted b.
    let list = run(dir.path(), &["observation", "list"]);
    assert!(list.status.success());
    let list_out = stdout(&list);
    assert!(list_out.contains(uid_a), "active record must appear");
    assert!(
        !list_out.contains(uid_b),
        "retracted record must be excluded from active list"
    );

    // List --history should include b.
    let hist = run(dir.path(), &["observation", "list", "--history"]);
    assert!(hist.status.success());
    let hist_out = stdout(&hist);
    assert!(
        hist_out.contains(uid_b),
        "history must include retracted record"
    );

    // Search.
    let search = run(dir.path(), &["observation", "search", "active"]);
    assert!(search.status.success());
    let search_out = stdout(&search);
    assert!(search_out.contains(uid_a));
    assert!(!search_out.contains(uid_b));
}

#[test]
fn list_and_search_kind_filter() {
    let dir = tmp();
    init_repo(dir.path());

    run(
        dir.path(),
        &["observation", "record", "friction", "a friction"],
    )
    .assert_success();

    let list = run(dir.path(), &["observation", "list", "--kind", "friction"]);
    assert!(list.status.success());
    assert!(!stdout(&list).is_empty());

    let list = run(
        dir.path(),
        &["observation", "list", "--kind", "measurement"],
    );
    assert!(list.status.success());
    // No measurements — empty.
    assert!(stdout(&list).contains("(no observations)"));
}

#[test]
fn list_json_format() {
    let dir = tmp();
    init_repo(dir.path());

    run(dir.path(), &["observation", "record", "friction", "test"]).assert_success();

    let list = run(dir.path(), &["observation", "list", "--json"]);
    assert!(list.status.success());
    let val = json_stdout(&list);
    assert_eq!(val["kind"], "observations");
    assert!(val["rows"].is_array());
}

#[test]
fn search_json_format() {
    let dir = tmp();
    init_repo(dir.path());

    run(
        dir.path(),
        &["observation", "record", "friction", "needle in haystack"],
    )
    .assert_success();

    let search = run(dir.path(), &["observation", "search", "needle", "--json"]);
    assert!(search.status.success());
    let val = json_stdout(&search);
    assert_eq!(val["kind"], "observations");
    assert!(val["rows"].is_array());
}

#[test]
fn list_empty_repo_graceful() {
    let dir = tmp();
    init_repo(dir.path());

    let list = run(dir.path(), &["observation", "list"]);
    assert!(list.status.success());
    let out = stdout(&list);
    assert!(
        out.contains("(no observations)"),
        "empty repo must be graceful"
    );
}

// ── Diagnostics on stderr (RV-317 E2) ────────────────────────────────────

#[test]
fn healthy_corpus_produces_empty_stderr() {
    let dir = tmp();
    init_repo(dir.path());

    // Record a valid friction with a known uid.
    let uid = "019faaaa-5678-7abc-8def-0123456789ab";
    run(
        dir.path(),
        &["observation", "record", "friction", "test", "--uid", uid],
    )
    .assert_success();

    // List — stderr must be empty.
    let list = run(dir.path(), &["observation", "list"]);
    assert!(list.status.success());
    assert!(
        stderr(&list).is_empty(),
        "healthy corpus list must produce empty stderr, got: {}",
        stderr(&list)
    );

    // Search — stderr must be empty.
    let search = run(dir.path(), &["observation", "search", "test"]);
    assert!(search.status.success());
    assert!(
        stderr(&search).is_empty(),
        "healthy corpus search must produce empty stderr, got: {}",
        stderr(&search)
    );

    // Show — stderr must be empty.
    let show = run(dir.path(), &["observation", "show", uid]);
    assert!(show.status.success());
    assert!(
        stderr(&show).is_empty(),
        "healthy corpus show must produce empty stderr, got: {}",
        stderr(&show)
    );
}

#[test]
fn malformed_record_surfaces_diagnostic_on_stderr() {
    let dir = tmp();
    init_repo(dir.path());

    // Record a valid friction.
    let uid_good = "019faaaa-5678-7abc-8def-0123456789ab";
    run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "good record",
            "--uid",
            uid_good,
        ],
    )
    .assert_success();

    // Plant a malformed file (not valid TOML).
    let shard = &uid_good[uid_good.len() - 2..];
    let bad_uid = "019fbbbb-5678-7abc-8def-0123456789cd";
    let rec_dir = dir
        .path()
        .join(".doctrine/observations/records")
        .join(shard);
    std::fs::create_dir_all(&rec_dir).unwrap();
    std::fs::write(
        rec_dir.join(format!("{bad_uid}.toml")),
        "this is not valid toml {{{{{",
    )
    .unwrap();

    // List — good record on stdout.
    let list = run(dir.path(), &["observation", "list"]);
    assert!(list.status.success());
    let out = stdout(&list);
    assert!(
        out.contains(uid_good),
        "good record must still appear on stdout"
    );

    // Bad record named on stderr.
    let err = stderr(&list);
    assert!(
        !err.is_empty(),
        "malformed record must produce stderr diagnostic"
    );
    assert!(
        err.contains(bad_uid),
        "stderr must name the bad record's path or uid, got: {err}"
    );
}

// ── Corrections tests ─────────────────────────────────────────────────────

#[test]
fn corrections_preserve_complete_history() {
    let dir = tmp();
    init_repo(dir.path());

    let uid_a = "019faaaa-5678-7abc-8def-0123456789ab";
    let uid_b = "019fbbbb-5678-7abc-8def-0123456789cd";

    run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "original",
            "--uid",
            uid_a,
        ],
    )
    .assert_success();
    run(
        dir.path(),
        &[
            "observation",
            "record",
            "friction",
            "replacement",
            "--uid",
            uid_b,
        ],
    )
    .assert_success();

    // Supersede.
    let ss_out = run(dir.path(), &["observation", "supersede", uid_a, uid_b]);
    assert!(ss_out.status.success());
    let ss = json_stdout(&ss_out);
    assert_eq!(ss["outcome"], "created");

    // History list includes superseded record.
    let hist = run(dir.path(), &["observation", "list", "--history"]);
    let hist_out = stdout(&hist);
    assert!(
        hist_out.contains(uid_a),
        "history must include superseded original"
    );
    assert!(hist_out.contains(uid_b), "history must include replacement");

    // Show shows correction chain.
    let show = run(dir.path(), &["observation", "show", uid_a]);
    let show_out = stdout(&show);
    assert!(
        show_out.contains("correction chain"),
        "must show correction chain"
    );
}

#[test]
fn supersede_missing_target_fails() {
    let dir = tmp();
    init_repo(dir.path());

    let out = run(
        dir.path(),
        &[
            "observation",
            "supersede",
            "019faaaa-5678-7abc-8def-0123456789ab",
            "019fbbbb-5678-7abc-8def-0123456789cd",
        ],
    );
    assert!(
        !out.status.success(),
        "supersede of missing target must fail"
    );
    assert!(
        stderr(&out).contains("does not exist"),
        "must report missing target"
    );
}

#[test]
fn supersede_kind_incompatible_target_fails() {
    let dir = tmp();
    init_repo(dir.path());

    let uid_a = "019faaaa-5678-7abc-8def-0123456789ab";
    let uid_b = "019fbbbb-5678-7abc-8def-0123456789cd";

    // We can't easily create a measurement via CLI (gated), but we can try to
    // supersede a friction with a nonexistent replacement and check the error.
    run(
        dir.path(),
        &["observation", "record", "friction", "test", "--uid", uid_a],
    )
    .assert_success();

    // Replace with nonexistent uid — fails.
    let out = run(dir.path(), &["observation", "supersede", uid_a, uid_b]);
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("does not exist"),
        "must report missing replacement"
    );
}

#[test]
fn supersede_control_kind_target_fails() {
    let dir = tmp();
    init_repo(dir.path());

    let uid_a = "019faaaa-5678-7abc-8def-0123456789ab";
    let uid_b = "019fbbbb-5678-7abc-8def-0123456789cd";

    run(
        dir.path(),
        &["observation", "record", "friction", "a", "--uid", uid_a],
    )
    .assert_success();
    run(
        dir.path(),
        &["observation", "record", "friction", "b", "--uid", uid_b],
    )
    .assert_success();

    // Create a supersession control a → b.
    run(dir.path(), &["observation", "supersede", uid_a, uid_b]).assert_success();

    // Find the supersession control UUID via JSON list.
    let list = run(dir.path(), &["observation", "list", "--history", "--json"]);
    let val = json_stdout(&list);
    let rows = val["rows"].as_array().expect("rows array");
    let ss_uid: String = rows
        .iter()
        .filter(|r| r["kind"] == "supersession")
        .map(|r| r["uid"].as_str().unwrap().to_string())
        .next()
        .expect("must find supersession control UUID");

    // Now try to supersede the control.
    let out = run(dir.path(), &["observation", "supersede", &ss_uid, uid_b]);
    assert!(!out.status.success(), "supersede of control must fail");
    assert!(
        stderr(&out).contains("control"),
        "must report control-kind rejection"
    );
}

#[test]
fn retract_missing_target_fails() {
    let dir = tmp();
    init_repo(dir.path());

    let out = run(
        dir.path(),
        &[
            "observation",
            "retract",
            "019faaaa-5678-7abc-8def-0123456789ab",
        ],
    );
    assert!(!out.status.success(), "retract of missing target must fail");
    assert!(
        stderr(&out).contains("does not exist"),
        "must report missing target"
    );
}

#[test]
fn retract_succeeds() {
    let dir = tmp();
    init_repo(dir.path());

    let uid = "019faaaa-5678-7abc-8def-0123456789ab";
    run(
        dir.path(),
        &["observation", "record", "friction", "test", "--uid", uid],
    )
    .assert_success();

    let out = run(dir.path(), &["observation", "retract", uid]);
    assert!(out.status.success());
    let receipt = json_stdout(&out);
    assert_eq!(receipt["outcome"], "created");
}

#[test]
fn retract_control_kind_target_fails() {
    let dir = tmp();
    init_repo(dir.path());

    let uid_a = "019faaaa-5678-7abc-8def-0123456789ab";
    run(
        dir.path(),
        &["observation", "record", "friction", "test", "--uid", uid_a],
    )
    .assert_success();

    // Retract it.
    run(dir.path(), &["observation", "retract", uid_a]).assert_success();

    // Find the retraction control UUID via JSON list.
    let list = run(dir.path(), &["observation", "list", "--history", "--json"]);
    let val = json_stdout(&list);
    let rows = val["rows"].as_array().expect("rows array");
    let ret_uid: String = rows
        .iter()
        .filter(|r| r["kind"] == "retraction")
        .map(|r| r["uid"].as_str().unwrap().to_string())
        .next()
        .expect("must find retraction control UUID");

    // Try to retract the retraction control itself.
    let out = run(dir.path(), &["observation", "retract", &ret_uid]);
    assert!(!out.status.success(), "retract of control must fail");
    assert!(
        stderr(&out).contains("control"),
        "must report control-kind rejection"
    );
}

// ── D1: Hostile metadata rendering (RV-317) ──────────────────────────

/// Plant a record straight onto disk with an arbitrary `recorded_at`.
///
/// Writing through the CLI cannot reach this state by design — the write
/// path validates. Bypassing it is the threat model itself: `load_one`
/// re-parses without re-validating, so the render path is the only
/// remaining boundary.
fn plant_record(root: &Path, uid: &str, recorded_at: &str) {
    let shard = &uid[uid.len() - 2..];
    let rec_dir = root.join(".doctrine/observations/records").join(shard);
    std::fs::create_dir_all(&rec_dir).unwrap();
    // TOML-escape the hostile timestamp for a basic string value:
    // backslash, newline, CR, tab must be TOML-escaped; ESC as \u001B.
    let toml_ts = recorded_at
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace('\x1b', "\\u001B");
    let record_toml = format!(
        r#"schema = "doctrine.observation"
schema_version = 1
uid = "{uid}"
recorded_at = "{toml_ts}"
kind = "friction"
summary = "legit"
"#
    );
    std::fs::write(rec_dir.join(format!("{uid}.toml")), record_toml).unwrap();
}

/// Write a record with hostile content in `recorded_at` and verify
/// that neither the table nor the detail view leaks raw escapes.
#[test]
fn hostile_recorded_at_is_escaped_in_list_and_show() {
    let dir = tmp();
    init_repo(dir.path());

    let uid = "019faaaa-5678-7abc-8def-0123456789ab";
    plant_record(
        dir.path(),
        uid,
        "2026-01-01T00:00:00Z\n\x1b[31mINJECTED\x1b[0m",
    );

    // List table — no raw ESC, escaped newline keeps row count=1.
    let list = run(dir.path(), &["observation", "list", "--history"]);
    assert!(list.status.success());
    let list_out = stdout(&list);
    assert!(
        !list_out.contains('\x1b'),
        "raw ANSI escapes must not appear literally in list output\noutput:\n{list_out}"
    );
    // The escape sequence must appear in escaped form.
    assert!(
        list_out.contains("\\x1b"),
        "ESC must be rendered as \\x1b in list output\noutput:\n{list_out}"
    );
    // The embedded newline in recorded_at must be escaped to \\n
    // (Inline context for table cells) — it must not forge an extra row.
    assert!(
        list_out.contains("\\n"),
        "embedded newline in recorded_at must be \\n in list table\noutput:\n{list_out}"
    );

    // Detail view — no raw ESC.
    let show = run(dir.path(), &["observation", "show", uid]);
    assert!(show.status.success());
    let show_out = stdout(&show);
    assert!(
        !show_out.contains('\x1b'),
        "ANSI escapes must not appear literally in show output\noutput:\n{show_out}"
    );
    assert!(
        show_out.contains("\\x1b"),
        "ESC must be rendered as \\x1b in show output\noutput:\n{show_out}"
    );
}

#[test]
fn hostile_recorded_at_cannot_forge_detail_header_lines() {
    let dir = tmp();
    init_repo(dir.path());

    // The header block is rendered ABOVE the "untrusted input" framing, so
    // a newline that survives into it forges lines a reader takes as
    // trustworthy envelope metadata. Single-line fields must stay single-line.
    let uid = "019fcccc-5678-7abc-8def-0123456789ef";
    plant_record(
        dir.path(),
        uid,
        "2026-01-01T00:00:00Z\nkind: policy-approval\nverified: true",
    );

    let show = run(dir.path(), &["observation", "show", uid]);
    assert!(show.status.success());
    let show_out = stdout(&show);

    for forged in ["kind: policy-approval", "verified: true"] {
        assert!(
            !show_out.lines().any(|l| l.trim() == forged),
            "a newline in recorded_at forged the header line {forged:?}\noutput:\n{show_out}"
        );
    }
    assert!(
        show_out.contains("recorded_at: 2026-01-01T00:00:00Z\\n"),
        "recorded_at must render on one line with its newline escaped\noutput:\n{show_out}"
    );
}

// ── D2: Non-ASCII uid diagnostics (RV-317) ───────────────────────────

#[test]
fn hostile_uid_is_escaped_in_the_rejection_diagnostic() {
    let dir = tmp();
    init_repo(dir.path());

    // The rejection echoes the uid back. That echo is a terminal-boundary
    // render like any other — PHASE-04's MCP adapter makes the uid
    // caller-supplied rather than typed by the invoker.
    let out = run(
        dir.path(),
        &["observation", "show", "X\x1b[31mRED\x1b[0m\nforged: line"],
    );
    assert!(!out.status.success());
    let err = stderr(&out);
    assert!(
        !err.contains('\x1b'),
        "raw ESC must not reach the terminal via the diagnostic: {err:?}"
    );
    assert!(
        !err.lines().any(|l| l.trim() == "forged: line"),
        "a newline in the uid must not forge a diagnostic line: {err:?}"
    );
}

#[test]
fn non_ascii_uid_shows_clean_diagnostic() {
    let dir = tmp();
    init_repo(dir.path());

    // 'éa' — multi-byte, not a valid UUID.
    let out = run(dir.path(), &["observation", "show", "éa"]);
    assert!(
        !out.status.success(),
        "non-ASCII uid must fail with diagnostic"
    );
    let err = stderr(&out);
    assert!(
        err.contains("invalid uid"),
        "must report invalid uid, got: {err}"
    );
    assert!(!err.contains("panicked"), "must not panic, got: {err}");
}

#[test]
fn non_ascii_uid_supersede_shows_clean_diagnostic() {
    let dir = tmp();
    init_repo(dir.path());

    let out = run(
        dir.path(),
        &[
            "observation",
            "supersede",
            "éa",
            "019fbbbb-5678-7abc-8def-0123456789cd",
        ],
    );
    assert!(
        !out.status.success(),
        "non-ASCII old_uid in supersede must fail with diagnostic"
    );
    let err = stderr(&out);
    assert!(
        err.contains("invalid"),
        "must report invalid uid, got: {err}"
    );
    assert!(!err.contains("panicked"), "must not panic, got: {err}");
}

#[test]
fn non_ascii_uid_retract_shows_clean_diagnostic() {
    let dir = tmp();
    init_repo(dir.path());

    let out = run(dir.path(), &["observation", "retract", "éa"]);
    assert!(
        !out.status.success(),
        "non-ASCII uid in retract must fail with diagnostic"
    );
    let err = stderr(&out);
    assert!(
        err.contains("invalid"),
        "must report invalid uid, got: {err}"
    );
    assert!(!err.contains("panicked"), "must not panic, got: {err}");
}

// ── Worker-fork refusal (RV-317 F-4.3) ───────────────────────────────────

/// Build a linked worktree off `main_repo` carrying the dispatch worker marker
/// — the shape the solo and dispatched refusal cases share. The marker leg only
/// trips in a LINKED worktree (`marker.rs` `describe_mode`: `is_linked &&
/// marker_present`), so the `git worktree add` is load-bearing, not scenery: the
/// same marker on a non-linked tree is inert.
fn marked_linked_fork(main_repo: &Path, fork: &Path) {
    git(
        main_repo,
        &[
            "worktree",
            "add",
            "-b",
            "fork-branch",
            fork.to_str().expect("utf-8 path"),
            "main",
        ],
    );
    let marker = fork.join(".doctrine/state/dispatch/worker");
    std::fs::create_dir_all(marker.parent().expect("marker parent")).unwrap();
    std::fs::write(&marker, "").unwrap();
}

/// Design §3.4: an observation write inside a marked worker fork is refused
/// with a diagnostic directing a confined worker to the `observation_record`
/// broker, rather than the generic orchestrator-funnel text.
///
/// The SOLO half of PHASE-03 VT-4: a solo agent in a marked worktree carries no
/// `DOCTRINE_WORKER`, so the marker leg alone trips and the signal token is
/// `marker`. (The claude-arm confined worker has this same shape.)
///
/// This lives in the e2e suite deliberately. `worker_guard` resolves the root
/// from the process CWD, so a unit test would have to mutate process-global
/// state that every concurrently-running test shares — `root::find` walks the
/// CWD upward. A subprocess gets its own CWD, so there is nothing to race.
#[test]
fn solo_marked_fork_refusal_points_to_observation_record_broker() {
    let dir = tmp();
    let main_repo = dir.path().join("main");
    init_repo(&main_repo);

    let fork = dir.path().join("fork");
    marked_linked_fork(&main_repo, &fork);

    let out = run(&fork, &["observation", "record", "friction", "from a fork"]);
    assert!(
        !out.status.success(),
        "an observation write in a marked worker fork must be refused"
    );
    let err = stderr(&out);
    assert!(
        err.contains("observation_record"),
        "refusal must name the MCP capture broker, got: {err}"
    );
    assert!(
        err.contains("signal: marker"),
        "a marker-only fork must report the marker signal, got: {err}"
    );
}

/// The DISPATCHED half of PHASE-03 VT-4: the pi-arm worker carries the marker
/// AND `DOCTRINE_WORKER`, so `describe_mode` resolves `Cause::Both`. Because the
/// tree IS linked, `is_env_on_nonlinked()` is false and the refusal must still
/// route to the observation branch — the broker advice, not the dual-cause text.
/// This is the leg that distinguishes a dispatched worker from a leaked env.
#[test]
fn dispatched_marked_fork_refusal_points_to_observation_record_broker() {
    let dir = tmp();
    let main_repo = dir.path().join("main");
    init_repo(&main_repo);

    let fork = dir.path().join("fork");
    marked_linked_fork(&main_repo, &fork);

    let out = run_as_env_worker(
        &fork,
        &[
            "observation",
            "record",
            "friction",
            "from a dispatched fork",
        ],
    );
    assert!(
        !out.status.success(),
        "an observation write in a dispatched worker fork must be refused"
    );
    let err = stderr(&out);
    assert!(
        err.contains("observation_record"),
        "a dispatched fork must still be pointed at the MCP capture broker, got: {err}"
    );
    assert!(
        err.contains("signal: both"),
        "marker plus env must report the dual signal, got: {err}"
    );
}

/// The negative that fences the two above: `DOCTRINE_WORKER` on a NON-linked
/// tree is an operator hazard (a worker dropped on the coordination root, or a
/// leaked env), not a confined worker. It takes the `is_env_on_nonlinked()`
/// branch, which deliberately WITHHOLDS the broker advice — directing an
/// operator to an MCP tool they are not running would be wrong guidance.
#[test]
fn leaked_env_on_nonlinked_tree_refuses_without_broker_advice() {
    let dir = tmp();
    let repo = dir.path().join("main");
    init_repo(&repo);

    let out = run_as_env_worker(
        &repo,
        &["observation", "record", "friction", "from a leaked env"],
    );
    assert!(
        !out.status.success(),
        "a leaked worker env must still refuse an authored write"
    );
    let err = stderr(&out);
    assert!(
        err.contains("DOCTRINE_WORKER"),
        "must carry the named dual-cause diagnostic, got: {err}"
    );
    assert!(
        !err.contains("observation_record"),
        "the dual-cause leg must NOT direct an operator to the MCP broker, got: {err}"
    );
}

// ── Git disposition of the corpus (PHASE-05, VT-2) ────────────────────────
//
// These two run against the ignore rule `doctrine install` actually projects
// — not a hand-written fixture — so they fail if the manifest entry regresses,
// is broadened, or never reaches a client.

/// `git check-ignore <rel>`: true when some rule matches. Exit 1 means "not
/// ignored", so this cannot go through `git()`, which asserts success.
fn is_ignored(root: &Path, rel: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["check-ignore", rel])
        .output()
        .expect("spawn git")
        .status
        .success()
}

/// Authoritative records are authored collection data: committed, diffable,
/// and visible in review by default (design § 2.1). Capture itself never
/// stages or commits, so the proof is that a fresh record is *addable* —
/// untracked, unignored, and reaching the index.
#[test]
fn authored_records_are_visible_to_git() {
    if common::under_worker_marker() {
        return; // SL-225 #2: `install` is refused under worker-mode
    }
    let dir = tmp();
    init_repo(dir.path());
    run(dir.path(), &["install", "-y"]).assert_success();

    let out = run(
        dir.path(),
        &["observation", "record", "friction", "visible to review"],
    );
    out.assert_success();
    let rel = json_stdout(&out)["rel_path"]
        .as_str()
        .expect("receipt carries rel_path")
        .to_string();

    assert!(
        !is_ignored(dir.path(), &rel),
        "{rel} must not be ignored — records are authored by default, and a \
         broadened rule would silently make the corpus local-only"
    );

    // Untracked before, staged after: it genuinely reaches the index.
    assert!(
        git(dir.path(), &["status", "--porcelain", "--", &rel]).starts_with("??"),
        "capture must leave {rel} untracked — it never stages or commits"
    );
    git(dir.path(), &["add", "--", &rel]);
    assert!(
        git(dir.path(), &["status", "--porcelain", "--", &rel]).starts_with('A'),
        "{rel} must be stageable"
    );
}

/// The publisher writes a complete sibling temp then hard-links it into place,
/// so an interrupted capture can leave a reserved `.tmp.`-prefixed name behind.
/// Those — and only those — are ignored.
#[test]
fn reserved_publication_temp_is_ignored() {
    if common::under_worker_marker() {
        return; // SL-225 #2: `install` is refused under worker-mode
    }
    let dir = tmp();
    init_repo(dir.path());
    run(dir.path(), &["install", "-y"]).assert_success();

    // Capture one record so the shard directory exists, then plant a leftover
    // temp beside it — the exact shape a crash mid-publication leaves.
    let out = run(dir.path(), &["observation", "record", "friction", "seed"]);
    out.assert_success();
    let rel = json_stdout(&out)["rel_path"]
        .as_str()
        .expect("receipt carries rel_path")
        .to_string();
    let shard = Path::new(&rel)
        .parent()
        .expect("record lives in a shard dir");
    let temp_rel = shard.join(".tmp.1234.0.pub");
    std::fs::write(dir.path().join(&temp_rel), "partial").unwrap();

    let temp_rel = temp_rel.to_string_lossy().to_string();
    assert!(
        is_ignored(dir.path(), &temp_rel),
        "{temp_rel} is a reserved publication temporary and must be ignored"
    );
    assert!(
        git(dir.path(), &["status", "--porcelain"])
            .lines()
            .all(|l| !l.contains(".tmp.")),
        "a leftover temporary must not surface as repository noise"
    );
}

// ── Helper trait ──────────────────────────────────────────────────────────

trait AssertSuccess {
    fn assert_success(&self);
}

impl AssertSuccess for Output {
    fn assert_success(&self) {
        assert!(
            self.status.success(),
            "command must succeed: {}",
            String::from_utf8_lossy(&self.stderr)
        );
    }
}
