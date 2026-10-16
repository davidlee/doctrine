// SPDX-License-Identifier: GPL-3.0-only
//! SL-282 PHASE-04 VT-1 — hymn starters on demand, over the built binary.
//!
//! Install no longer projects starter copies under `.doctrine/hymns/`; a user
//! takes one with `doctrine prompt customise <slot>`. A one-time legacy removal
//! deletes the unedited copies the retired starter leg left behind, only while
//! the tracked stamp is absent. Two control rows pin the RV-421 findings:
//! customise refuses before install (F-18) and install never deletes a
//! deliberate customisation (F-16).
//!
//! Every absence assertion pairs with a positive control from the same run.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::fs;
use std::path::Path;

mod common;

use common::{INSTALL_MARKER, doctrine_cmd, doctrine_toml, repo_root, tree_digest, write_stamp};

/// The running binary's version: what a complete install records.
const B: &str = env!("CARGO_PKG_VERSION");
/// One harness, one skill — small and fast.
const BASE_ARGS: &[&str] = &["--agent", "claude", "--skill", "code-review", "--yes"];
const HYMNS: &str = ".doctrine/hymns";
const WORKER_DEF: &str = ".doctrine/agents/claude/dispatch-worker.md";
/// The framework worker contract's opening line (`install/hymns/role/worker.md`).
const WORKER_CONTRACT: &str = "You are a doctrine worker. You implement exactly ONE declared phase";
/// The framework `model/adherence/low` body's opening words.
const LOW_ADHERENCE: &str = "Self-guidance for models with low";
const PRE_STAMP_REFUSAL: &str = "Error: doctrine isn't installed here (no installed-version in \
                                 .doctrine/doctrine.toml): run `doctrine install` first";
const NOTICE: &str = "this copy replaces the framework text for role/worker; it no longer \
                      receives framework updates";

/// `doctrine <verb…> -p <root>`, output unchecked.
fn run_output(root: &Path, args: &[&str]) -> std::process::Output {
    doctrine_cmd(root)
        .args(args)
        .arg("-p")
        .arg(root)
        .output()
        .expect("spawn doctrine")
}

/// As [`run_output`]; asserts success and returns stdout.
fn run(root: &Path, args: &[&str]) -> String {
    let out = run_output(root, args);
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    assert!(
        out.status.success(),
        "doctrine {args:?} failed: {stdout}\n{}",
        String::from_utf8_lossy(&out.stderr),
    );
    stdout
}

fn stderr(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn install(root: &Path) -> String {
    let args: Vec<&str> = ["install"].iter().chain(BASE_ARGS).copied().collect();
    run(root, &args)
}

/// The shipped framework bytes of `<rel>` under `install/hymns/`.
fn framework(rel: &str) -> Vec<u8> {
    fs::read(repo_root().join("install/hymns").join(rel)).unwrap()
}

/// The sidecar a starter copy carries: its self-`replaces`.
fn self_replaces(slot: &str) -> String {
    format!("replaces = \"{slot}\"\n")
}

fn hymn(root: &Path, rel: &str) -> std::path::PathBuf {
    root.join(HYMNS).join(rel)
}

fn plant(root: &Path, rel: &str, bytes: &[u8]) {
    let path = hymn(root, rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

/// Plant an unedited legacy starter: the framework body, plus its default
/// sidecar when `sidecar`.
fn plant_starter(root: &Path, slot: &str, sidecar: bool) {
    plant(
        root,
        &format!("{slot}.md"),
        &framework(&format!("{slot}.md")),
    );
    if sidecar {
        plant(
            root,
            &format!("{slot}.toml"),
            self_replaces(slot).as_bytes(),
        );
    }
}

/// `prompt resolve` stdout for `args`.
fn resolve(root: &Path, args: &[&str]) -> String {
    let full: Vec<&str> = ["prompt", "resolve"].iter().chain(args).copied().collect();
    run(root, &full)
}

/// The trimmed stderr lines of a failed run (asserting it failed).
fn refusal_lines(out: &std::process::Output) -> Vec<String> {
    assert!(!out.status.success(), "expected a refusal: {}", stderr(out));
    stderr(out).lines().map(|l| l.trim().to_string()).collect()
}

fn removed_line(slot: &str) -> String {
    format!("removed unedited starter {HYMNS}/{slot}.md (framework text applies again)")
}

fn trimmed_lines(out: &str) -> Vec<&str> {
    out.lines().map(str::trim).collect()
}

/// EX-1 (ISS-210). The discriminating half is "no files under
/// `.doctrine/hymns/`"; the resolve half also held on the old code (the starter
/// body *was* the framework body on day one), so it only proves the worker still
/// receives its contract with no disk copy.
#[test]
fn fresh_install_writes_no_hymns_and_worker_gets_the_framework_contract() {
    if common::under_worker_marker() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    install(root);

    assert!(!root.join(HYMNS).exists(), "no hymn files written");
    // Positive control: the install ran its legs.
    let tracked = fs::read_to_string(doctrine_toml(root)).unwrap();
    assert!(tracked.contains(&format!("installed-version = \"{B}\"")));
    assert!(root.join(INSTALL_MARKER).is_file());
    assert!(root.join(WORKER_DEF).is_file(), "worker def baked");

    let worker = resolve(root, &["--role", "worker"]);
    assert_eq!(worker.matches(WORKER_CONTRACT).count(), 1, "{worker}");
}

/// EX-3 — RV-421 F-18 control: customise refuses in a repo with no
/// installed-version and writes nothing.
#[test]
fn customise_refuses_before_install() {
    if common::under_worker_marker() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let before = tree_digest(root);
    let out = run_output(root, &["prompt", "customise", "role/worker"]);
    assert!(
        refusal_lines(&out).contains(&PRE_STAMP_REFUSAL.to_string()),
        "{}",
        stderr(&out)
    );
    assert_eq!(
        tree_digest(root),
        before,
        "no doctrine.toml: nothing written"
    );

    // A doctrine.toml without `[install]` is not installed either.
    let toml = doctrine_toml(root);
    fs::create_dir_all(toml.parent().unwrap()).unwrap();
    fs::write(&toml, "[verification]\n").unwrap();
    let before = tree_digest(root);
    let out = run_output(root, &["prompt", "customise", "role/worker"]);
    assert!(
        refusal_lines(&out).contains(&PRE_STAMP_REFUSAL.to_string()),
        "{}",
        stderr(&out)
    );
    assert_eq!(tree_digest(root), before, "no [install]: nothing written");
}

/// EX-2: the starter carries the framework body and a self-`replaces` sidecar,
/// so the slot single-emits. `customize` is the same verb.
#[test]
fn customise_writes_starter_and_self_replaces_sidecar() {
    if common::under_worker_marker() {
        return;
    }
    for verb in ["customise", "customize"] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_stamp(root, B);

        let out = run(root, &["prompt", verb, "role/worker"]);

        assert_eq!(
            fs::read(hymn(root, "role/worker.md")).unwrap(),
            framework("role/worker.md")
        );
        assert_eq!(
            fs::read_to_string(hymn(root, "role/worker.toml")).unwrap(),
            self_replaces("role/worker")
        );
        let lines = trimmed_lines(&out);
        assert_eq!(
            lines,
            vec![
                ".doctrine/hymns/role/worker.md",
                ".doctrine/hymns/role/worker.toml",
                NOTICE
            ],
            "{verb}: {out}"
        );
        assert!(run(root, &["prompt", "check"]).contains("check: corpus OK"));
        let worker = resolve(root, &["--role", "worker"]);
        assert_eq!(
            worker.matches(WORKER_CONTRACT).count(),
            1,
            "{verb}: {worker}"
        );
    }
}

/// EX-2: an existing file is left byte-identical and reported as kept.
#[test]
fn customise_leaves_an_existing_file_alone() {
    if common::under_worker_marker() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_stamp(root, B);
    plant(root, "role/worker.md", b"MY WORKER\n");

    let out = run(root, &["prompt", "customise", "role/worker"]);

    assert_eq!(
        fs::read(hymn(root, "role/worker.md")).unwrap(),
        b"MY WORKER\n"
    );
    assert_eq!(
        fs::read_to_string(hymn(root, "role/worker.toml")).unwrap(),
        self_replaces("role/worker")
    );
    assert_eq!(
        trimmed_lines(&out),
        vec![
            ".doctrine/hymns/role/worker.toml",
            "kept .doctrine/hymns/role/worker.md (already exists)",
            NOTICE,
        ],
        "{out}"
    );
}

/// EX-2: a sealed slot is refused, naming the seal; nothing is written.
#[test]
fn customise_refuses_a_sealed_slot() {
    if common::under_worker_marker() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_stamp(root, B);
    let before = tree_digest(root);

    let out = run_output(root, &["prompt", "customise", "preamble/core"]);

    assert_eq!(
        refusal_lines(&out),
        vec![
            "Error: preamble/core is sealed ([hymns].seal): its framework text cannot be replaced"
        ]
    );
    assert_eq!(tree_digest(root), before, "nothing written");
}

/// EX-2: an unknown slot lists the customisable framework slots (sealed ones
/// excluded); nothing is written.
#[test]
fn customise_lists_framework_slots_on_an_unknown_one() {
    if common::under_worker_marker() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_stamp(root, B);
    let before = tree_digest(root);

    for unknown in ["role/nope", "nope/worker", "worker"] {
        let out = run_output(root, &["prompt", "customise", unknown]);
        let lines = refusal_lines(&out);
        assert_eq!(
            lines[0],
            format!("Error: {unknown} is not a framework slot; customisable slots:")
        );
        assert!(lines.contains(&"role/worker".to_string()), "{lines:?}");
        assert!(lines.contains(&"harness/claude".to_string()), "{lines:?}");
        assert!(!lines.contains(&"preamble/core".to_string()), "{lines:?}");
    }
    assert_eq!(tree_digest(root), before, "nothing written");
}

/// EX-4: legacy removal takes unedited copies only; adherence/low single-emits
/// afterwards (ISS-521).
#[test]
fn install_removes_unedited_legacy_starters() {
    if common::under_worker_marker() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    plant_starter(root, "role/worker", true);
    plant_starter(root, "model/adherence/low", false);
    plant(root, "harness/claude.md", b"EDITED CLAUDE\n");
    plant(
        root,
        "harness/claude.toml",
        self_replaces("harness/claude").as_bytes(),
    );
    plant(root, "harness/cursor.md", &framework("harness/cursor.md"));
    plant(
        root,
        "harness/cursor.toml",
        b"replaces = \"harness/cursor\"\nharness = \"cursor\"\n",
    );
    plant(root, "README.md", &framework("README.md"));
    plant(root, "project/x.md", b"PROJECT X\n");
    let kept = [
        "harness/claude.md",
        "harness/claude.toml",
        "harness/cursor.md",
        "harness/cursor.toml",
        "README.md",
        "project/x.md",
    ];
    let kept_before: Vec<Vec<u8>> = kept
        .iter()
        .map(|r| fs::read(hymn(root, r)).unwrap())
        .collect();
    let low = [
        "--role",
        "worker",
        "--band",
        "model",
        "--model",
        "adherence/low",
    ];
    // Positive control: the sidecar-less twin double-emits before removal.
    assert_eq!(resolve(root, &low).matches(LOW_ADHERENCE).count(), 2);

    let out = install(root);

    let lines = trimmed_lines(&out);
    for slot in ["role/worker", "model/adherence/low"] {
        assert!(
            lines.contains(&removed_line(slot).as_str()),
            "{slot}: {out}"
        );
    }
    assert_eq!(out.matches("removed unedited starter").count(), 2, "{out}");
    for gone in [
        "role/worker.md",
        "role/worker.toml",
        "model/adherence/low.md",
    ] {
        assert!(!hymn(root, gone).exists(), "{gone} removed");
    }
    let kept_after: Vec<Vec<u8>> = kept
        .iter()
        .map(|r| fs::read(hymn(root, r)).unwrap())
        .collect();
    assert_eq!(
        kept_after, kept_before,
        "edited and non-snippet files byte-identical"
    );
    assert_eq!(resolve(root, &low).matches(LOW_ADHERENCE).count(), 1);
}

/// EX-4: once stamped, an identical copy is a deliberate customisation.
#[test]
fn legacy_removal_does_not_run_once_stamped() {
    if common::under_worker_marker() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_stamp(root, B);
    plant_starter(root, "role/worker", false);

    let out = install(root);

    assert!(!out.contains("removed unedited starter"), "{out}");
    assert_eq!(
        fs::read(hymn(root, "role/worker.md")).unwrap(),
        framework("role/worker.md")
    );
}

/// EX-5 — RV-421 F-16 control: install → customise → install keeps both files.
#[test]
fn install_customise_install_keeps_both_files() {
    if common::under_worker_marker() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    install(root);
    run(root, &["prompt", "customise", "role/worker"]);
    let md = fs::read(hymn(root, "role/worker.md")).unwrap();
    let toml = fs::read(hymn(root, "role/worker.toml")).unwrap();

    let out = install(root);

    assert!(!out.contains("removed unedited starter"), "{out}");
    assert_eq!(fs::read(hymn(root, "role/worker.md")).unwrap(), md);
    assert_eq!(fs::read(hymn(root, "role/worker.toml")).unwrap(), toml);
}
