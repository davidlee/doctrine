// SPDX-License-Identifier: GPL-3.0-only
//! SL-220 PHASE-04 (design §4) — the value-claim capture verbs as BLACK-BOX CLI
//! behaviours: `value set` mints a session-of-one anchor, `value pin`'s TTY gate
//! refuses piped stdin naming the posture, and `value clear` tombstones the
//! active rows.
//!
//! The suite spawns the BUILT binary with the child cwd set to the TEMPDIR
//! (`mem.pattern.testing.black-box-cli-golden`): these are WRITE / operator-gated
//! verbs, so a spawn whose cwd is the marked dispatch worktree would hit the
//! worker-mode guard's `signal: marker` refusal (masking the verb's own
//! behaviour). Rooting the child in the markerless tempdir makes the guard
//! resolve non-worker mode, so `value set|clear` write and `value pin`'s own TTY
//! gate is what refuses the piped stdin — exactly the surface under test.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    clippy::too_many_arguments,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, test fns live at crate root by construction, and the session builder mirrors the wire fields positionally"
)]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

/// A tempdir `root::find` resolves as a MARKERLESS project root (no worker
/// marker, so the guard resolves non-worker mode when the child cwd is here).
fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    write(dir.path(), ".project", "");
    write(dir.path(), "doctrine.toml", "");
    dir
}

/// Seed a resolvable, value-bearing slice entity.
fn seed_slice(root: &Path, id: u32) {
    write(
        root,
        &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
        &format!(
            "id = {id}\nslug = \"s{id}\"\ntitle = \"Slice {id}\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n"
        ),
    );
}

/// `doctrine <args...>` with the child cwd rooted in the markerless tempdir and
/// stdin left empty (so `.output()`'s null stdin is a non-TTY — the pin gate's
/// piped-stdin case).
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .current_dir(root)
        .env_remove("DOCTRINE_WORKER")
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

/// The `.toml` session files under `.doctrine/comparisons/`, sorted.
fn session_files(root: &Path) -> Vec<std::path::PathBuf> {
    let dir = root.join(".doctrine/comparisons");
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut files: Vec<_> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    files.sort();
    files
}

// =============================================================================
// value set — the mint path
// =============================================================================

/// `value set` mints a session-of-one anchor stamped `value-anchor` / `value` /
/// `anchor` with the given magnitude and no pairwise payload.
#[test]
fn value_set_mints_a_session_of_one_anchor() {
    let dir = project();
    let root = dir.path();
    seed_slice(root, 118);

    ok(&run(
        root,
        &["value", "set", "SL-118", "42", "--rater", "human"],
    ));

    let files = session_files(root);
    assert_eq!(files.len(), 1, "one session file minted");
    let body = fs::read_to_string(&files[0]).unwrap();
    assert!(body.contains("frame = \"value-anchor\""), "{body}");
    assert!(body.contains("domain = \"value\""), "{body}");
    assert!(body.contains("form = \"anchor\""), "{body}");
    assert!(body.contains("magnitude = 42"), "{body}");
    assert!(body.contains("a = \"SL-118\""), "{body}");
    // Anchor rows carry no pairwise payload.
    assert!(!body.contains("\nb = "), "no `b` on an anchor row:\n{body}");
    assert!(
        !body.contains("response ="),
        "no response on an anchor:\n{body}"
    );
}

/// `--rater` is MANDATORY — omitting it is a parse-level refusal (no default
/// fabricates provenance), and nothing is written.
#[test]
fn value_set_requires_rater() {
    let dir = project();
    let root = dir.path();
    seed_slice(root, 118);

    let out = run(root, &["value", "set", "SL-118", "42"]);
    assert!(!out.status.success(), "missing --rater must fail");
    assert!(session_files(root).is_empty(), "nothing written");
}

/// D10: every invocation mints — two identical `value set`s yield TWO session
/// files (no idempotency guard).
#[test]
fn value_set_mints_every_invocation() {
    let dir = project();
    let root = dir.path();
    seed_slice(root, 118);

    ok(&run(
        root,
        &["value", "set", "SL-118", "7", "--rater", "human"],
    ));
    ok(&run(
        root,
        &["value", "set", "SL-118", "7", "--rater", "human"],
    ));
    assert_eq!(session_files(root).len(), 2, "two files (D10)");
}

/// A non-finite magnitude is refused at capture (mirrors `value::validate`).
#[test]
fn value_set_rejects_non_finite() {
    let dir = project();
    let root = dir.path();
    seed_slice(root, 118);

    let out = run(root, &["value", "set", "SL-118", "inf", "--rater", "human"]);
    assert!(!out.status.success(), "inf magnitude must fail");
    assert!(session_files(root).is_empty(), "nothing written");
}

// =============================================================================
// value pin — the D13 TTY gate
// =============================================================================

/// `value pin` refuses when stdin is not a TTY (the spawned child's stdin is
/// piped-empty), naming the interactive-operator posture — and writes nothing.
#[test]
fn value_pin_refuses_piped_stdin_naming_the_posture() {
    let dir = project();
    let root = dir.path();
    seed_slice(root, 118);

    let out = run(root, &["value", "pin", "SL-118", "6.5", "--by", "david"]);
    assert!(!out.status.success(), "piped-stdin pin must refuse");
    let err = stderr(&out);
    assert!(
        err.contains("interactive operator"),
        "names the posture: {err}"
    );
    assert!(err.contains("not a TTY"), "names the cause: {err}");
    assert!(session_files(root).is_empty(), "no row written on refusal");
}

// =============================================================================
// value clear — tombstone the active rows
// =============================================================================

/// `value clear` tombstones the active unlensed anchor rows on the subject
/// (append-only: a fresh session carrying the tombstone).
#[test]
fn value_clear_tombstones_active_rows() {
    let dir = project();
    let root = dir.path();
    seed_slice(root, 118);

    ok(&run(
        root,
        &["value", "set", "SL-118", "5", "--rater", "human"],
    ));
    let cleared = ok(&run(root, &["value", "clear", "SL-118"]));
    assert!(cleared.contains("value cleared: SL-118"), "{cleared}");

    // Append-only: the set file plus a new tombstone-carrying session.
    let files = session_files(root);
    assert_eq!(files.len(), 2, "set + clear sessions");
    let has_tombstone = files
        .iter()
        .any(|p| fs::read_to_string(p).unwrap().contains("[[tombstone]]"));
    assert!(has_tombstone, "a tombstone session was written");

    // A second clear is a no-op (nothing active remains).
    let again = ok(&run(root, &["value", "clear", "SL-118"]));
    assert!(again.contains("no value anchor to clear"), "{again}");
}

// =============================================================================
// SL-220 PHASE-06 — provenance rendering, end to end (§8.8)
// =============================================================================

/// `explain` needs the scope `.md` companion to resolve the slice into the
/// priority graph, so seed both tiers.
fn seed_explainable_slice(root: &Path, id: u32) {
    seed_slice(root, id);
    write(
        root,
        &format!(".doctrine/slice/{id:03}/slice-{id:03}.md"),
        "# Scope\n",
    );
}

/// A hand-authored value-anchor session (the wire shape `value set` mints),
/// optionally superseding a prior row by uid — the append-only supersession
/// path made explicit for the resolution round-trip tests.
fn write_value_session(
    root: &Path,
    file: &str,
    session_uid: &str,
    row_uid: &str,
    item: &str,
    magnitude: f64,
    by: &str,
    date: &str,
    supersedes: Option<&str>,
) {
    let supersedes_line =
        supersedes.map_or(String::new(), |uid| format!("supersedes = \"{uid}\"\n"));
    write(
        root,
        &format!(".doctrine/comparisons/{file}"),
        &format!(
            "schema = \"doctrine.comparison-session\"\nversion = 2\ntombstone = []\n\n\
             [session]\nuid = \"{session_uid}\"\ndate = \"{date}\"\n\n\
             [[judgement]]\nuid = \"{row_uid}\"\nseq = 0\na = \"{item}\"\n\
             domain = \"value\"\nframe = \"value-anchor\"\nform = \"anchor\"\n\
             magnitude = {magnitude}\n{supersedes_line}rater = \"human\"\nby = \"{by}\"\ndate = \"{date}\"\n"
        ),
    );
}

fn explain(root: &Path, id: &str) -> String {
    ok(&run(root, &["explain", id]))
}

/// §8.8 capture-to-scoring round trip: `value set` mints an anchor that the
/// resolver consumes, and `explain` renders the SL-220 PHASE-06 provenance line
/// — the human-claim shape with its attribution.
#[test]
fn value_set_then_explain_renders_human_claim_provenance() {
    let dir = project();
    let root = dir.path();
    seed_explainable_slice(root, 118);

    ok(&run(
        root,
        &[
            "value", "set", "SL-118", "6", "--rater", "human", "--by", "ada",
        ],
    ));

    let out = explain(root, "SL-118");
    assert!(
        out.contains("value 6.0 — human claim (ada,"),
        "explain renders the human-claim provenance line: {out}"
    );
}

/// §8.8 pin/claim wins + supersession resolves: a superseding anchor row (by
/// uid) retires the earlier claim, so `explain` re-sources the LATER magnitude
/// and attribution — the append-only demotion path.
#[test]
fn superseding_anchor_row_resolves_the_ladder() {
    let dir = project();
    let root = dir.path();
    seed_explainable_slice(root, 118);

    write_value_session(
        root,
        "a.toml",
        "00000000-0000-7000-8000-000000000001",
        "00000000-0000-7000-8000-0000000000a1",
        "SL-118",
        3.0,
        "ada",
        "2026-07-10",
        None,
    );
    write_value_session(
        root,
        "b.toml",
        "00000000-0000-7000-8000-000000000002",
        "00000000-0000-7000-8000-0000000000b1",
        "SL-118",
        8.0,
        "bo",
        "2026-07-11",
        Some("00000000-0000-7000-8000-0000000000a1"),
    );

    let out = explain(root, "SL-118");
    assert!(
        out.contains("value 8.0 — human claim (bo,"),
        "the superseding row wins the ladder: {out}"
    );
    assert!(
        !out.contains("value 3.0"),
        "the superseded claim is gone: {out}"
    );
}

/// §8.8 conflict surfaces: two CONCURRENT same-tier anchors that disagree
/// resolve to the multiset mean rendered as a CONTESTED claim (no single
/// author), never a silent latest-wins.
#[test]
fn concurrent_conflicting_claims_render_contested() {
    let dir = project();
    let root = dir.path();
    seed_explainable_slice(root, 118);

    write_value_session(
        root,
        "a.toml",
        "00000000-0000-7000-8000-000000000001",
        "00000000-0000-7000-8000-0000000000a1",
        "SL-118",
        4.0,
        "ada",
        "2026-07-10",
        None,
    );
    write_value_session(
        root,
        "b.toml",
        "00000000-0000-7000-8000-000000000002",
        "00000000-0000-7000-8000-0000000000b1",
        "SL-118",
        8.0,
        "bo",
        "2026-07-11",
        None,
    );

    let out = explain(root, "SL-118");
    assert!(
        out.contains("contested human claim") && out.contains("interval (4.0 ‥ 8.0)"),
        "concurrent disagreement renders the contested interval: {out}"
    );
}

/// §8.8 clear → tombstone → ladder falls through: once the sole anchor is
/// tombstoned, no claim evidence remains, so the value-source line is OMITTED
/// (never the scoring floor).
#[test]
fn clear_tombstones_then_ladder_falls_through_to_no_value_line() {
    let dir = project();
    let root = dir.path();
    seed_explainable_slice(root, 118);

    ok(&run(
        root,
        &[
            "value", "set", "SL-118", "5", "--rater", "human", "--by", "ada",
        ],
    ));
    ok(&run(root, &["value", "clear", "SL-118"]));

    let out = explain(root, "SL-118");
    assert!(
        !out.contains("value 5.0") && !out.contains("— human claim"),
        "the tombstoned claim leaves no value-source line: {out}"
    );
}
