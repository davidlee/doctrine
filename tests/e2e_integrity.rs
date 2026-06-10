//! SL-032 PHASE-03 — `validate` + `reseat` as BLACK-BOX goldens over the built
//! binary (ADR-006 D3 detect + repair backstop).
//!
//! `validate` scans every numbered kind for the three per-kind rules (dir==id,
//! no intra-kind duplicate, alias target equality) and exits non-zero on any
//! violation (VT-1..4). `reseat` renumbers an entity's canonical-id quad to a
//! free id, refuses an occupied target (VT-6) or live runtime phase state (VT-7),
//! and reports inbound prose citations as danglers with a non-zero exit (VT-5).
//! No trunk is reachable in a throwaway tempdir, so reseat's id pick degrades to
//! the local-only scan — exactly the EDGE path (R-2).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_doctrine");

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

/// Seed one slice: `.doctrine/slice/<dir_id>/slice-<dir_id>.{toml,md}`. `toml_id`
/// is the id the toml *declares* — usually equal to `dir_id`, deliberately
/// divergent for the rule-(a) fixture.
fn seed_slice(root: &Path, dir_id: u32, toml_id: u32, slug: &str) {
    let dir = root.join(format!(".doctrine/slice/{dir_id:03}"));
    fs::create_dir_all(&dir).unwrap();
    let toml = format!(
        "id = {toml_id}\n\
         slug = \"{slug}\"\n\
         title = \"fixture\"\n\
         status = \"proposed\"\n\
         created = \"2026-01-01\"\n\
         updated = \"2026-01-01\"\n\
         \n\
         [relationships]\n"
    );
    fs::write(dir.join(format!("slice-{dir_id:03}.toml")), toml).unwrap();
    fs::write(
        dir.join(format!("slice-{dir_id:03}.md")),
        "# fixture\n\nbody.\n",
    )
    .unwrap();
}

/// Plant an `<name> -> <target>` alias symlink under the slice tree.
fn alias(root: &Path, name: &str, target: &str) {
    symlink(target, root.join(".doctrine/slice").join(name)).unwrap();
}

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

// --- validate (VT-1..4) ---------------------------------------------------

#[test]
fn validate_clean_corpus_exits_zero() {
    let t = tmp();
    seed_slice(t.path(), 1, 1, "alpha");
    seed_slice(t.path(), 2, 2, "beta");
    alias(t.path(), "001-alpha", "001");
    alias(t.path(), "002-beta", "002");

    let out = run(t.path(), &["validate"]);
    assert!(
        out.status.success(),
        "clean corpus must exit 0: {}",
        stderr(&out)
    );
    assert!(stdout(&out).contains("corpus clean"), "{}", stdout(&out));
    // D-A visibility: the scanned kinds (incl. the memory omission) are named.
    assert!(stdout(&out).contains("scanned SL"), "{}", stdout(&out));
}

#[test]
fn validate_flags_dir_id_mismatch() {
    let t = tmp();
    // dir 003 declares id 045 (rule a).
    seed_slice(t.path(), 3, 45, "drifted");

    let out = run(t.path(), &["validate"]);
    assert!(!out.status.success(), "a violation must exit non-zero");
    assert!(
        stdout(&out).contains("dir 003 declares id 045"),
        "stdout={} stderr={}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn validate_flags_intra_kind_duplicate_id() {
    let t = tmp();
    // two dirs both declaring id 7 (rule b); dir 008 also trips rule (a).
    seed_slice(t.path(), 7, 7, "first");
    seed_slice(t.path(), 8, 7, "second");

    let out = run(t.path(), &["validate"]);
    assert!(!out.status.success());
    assert!(
        stdout(&out).contains("id 007 declared by dirs 007, 008"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn validate_flags_mis_targeted_alias() {
    let t = tmp();
    seed_slice(t.path(), 1, 1, "alpha");
    seed_slice(t.path(), 2, 2, "beta");
    // alias encodes 001 but resolves to dir 002 (declares id 2) — X7.
    alias(t.path(), "001-alpha", "002");
    alias(t.path(), "002-beta", "002");

    let out = run(t.path(), &["validate"]);
    assert!(!out.status.success());
    assert!(
        stdout(&out).contains("alias 001-* targets id 002"),
        "{}",
        stdout(&out)
    );
}

// --- reseat (VT-5..7) -----------------------------------------------------

#[test]
fn reseat_renumbers_quad_and_reports_danglers() {
    let t = tmp();
    seed_slice(t.path(), 31, 31, "worker-guard");
    alias(t.path(), "031-worker-guard", "031");
    // A prose citation elsewhere in the corpus — the dangler reseat must surface.
    let note = t.path().join(".doctrine/notes/x.md");
    fs::create_dir_all(note.parent().unwrap()).unwrap();
    fs::write(&note, "Depends on SL-031 landing first.\n").unwrap();

    let out = run(t.path(), &["reseat", "SL-031", "--to", "045"]);

    // The quad moved: dir, inner files, toml id, alias.
    let dst = t.path().join(".doctrine/slice/045");
    assert!(dst.join("slice-045.toml").is_file(), "inner toml renamed");
    assert!(dst.join("slice-045.md").is_file(), "inner md renamed");
    assert!(
        !t.path().join(".doctrine/slice/031").exists(),
        "src dir gone"
    );
    let toml = fs::read_to_string(dst.join("slice-045.toml")).unwrap();
    assert!(toml.contains("id = 45"), "toml id rewritten: {toml}");
    assert!(
        toml.contains("[relationships]"),
        "sections preserved: {toml}"
    );
    let link = fs::read_link(t.path().join(".doctrine/slice/045-worker-guard")).unwrap();
    assert_eq!(link.to_str(), Some("045"), "alias repointed");
    assert!(
        !t.path().join(".doctrine/slice/031-worker-guard").exists(),
        "stale alias removed"
    );

    // Danglers reported, non-zero exit, prose left untouched (D4/R-3).
    assert!(!out.status.success(), "danglers force non-zero exit");
    assert!(
        stdout(&out).contains("reseated SL-031 → SL-045"),
        "{}",
        stdout(&out)
    );
    assert!(
        stdout(&out).contains("notes/x.md:1"),
        "dangler located: {}",
        stdout(&out)
    );
    assert!(
        fs::read_to_string(&note).unwrap().contains("SL-031"),
        "prose citation NOT rewritten"
    );
}

#[test]
fn reseat_refuses_occupied_target() {
    let t = tmp();
    seed_slice(t.path(), 31, 31, "src");
    seed_slice(t.path(), 45, 45, "occupant");

    let out = run(t.path(), &["reseat", "SL-031", "--to", "045"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("occupied"), "{}", stderr(&out));
    // No mutation: both dirs intact.
    assert!(t.path().join(".doctrine/slice/031").is_dir());
    assert!(
        t.path()
            .join(".doctrine/slice/045/slice-045.toml")
            .is_file()
    );
}

#[test]
fn reseat_refuses_live_runtime_phase_state() {
    let t = tmp();
    seed_slice(t.path(), 31, 31, "src");
    // Live gitignored phase state keyed by the source id (F3).
    fs::create_dir_all(t.path().join(".doctrine/state/slice/031/phases")).unwrap();

    let out = run(t.path(), &["reseat", "SL-031", "--to", "045"]);
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("runtime phase state"),
        "{}",
        stderr(&out)
    );
    assert!(
        t.path().join(".doctrine/slice/031").is_dir(),
        "src untouched"
    );
    assert!(
        !t.path().join(".doctrine/slice/045").exists(),
        "no dst created"
    );
}
