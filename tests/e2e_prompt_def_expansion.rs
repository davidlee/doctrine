// SPDX-License-Identifier: GPL-3.0-only
//! E2E coverage for dispatch-worker def expansion during install.
//!
//! Keywords pinned for SL-186 PHASE-04: `dispatch-worker`, `resolve`,
//! `--role worker`.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic"
)]

use std::fs;
use std::path::Path;
use std::process::Command;

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

fn install(root: &Path) -> std::process::Output {
    Command::new(bin())
        .current_dir(root)
        .env_remove("DOCTRINE_WORKER")
        .args(["install", "--agent", "claude", "--yes", "-p"])
        .arg(root)
        .output()
        .expect("spawn doctrine install")
}

#[test]
fn dispatch_worker_install_expands_worker_role_marker() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    let hymns = root.join(".doctrine/hymns/role");
    fs::create_dir_all(&hymns).unwrap();
    fs::write(
        hymns.join("worker.md"),
        "Role body from prompt resolve --role worker",
    )
    .unwrap();

    let out = install(root);
    assert!(
        out.status.success(),
        "install failed: {}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    let written = fs::read_to_string(root.join(".doctrine/agents/dispatch-worker.md")).unwrap();
    assert!(
        written.contains("Role body from prompt resolve --role worker"),
        "{written}"
    );
    assert!(
        !written.contains("{{ prompt resolve --role worker }}"),
        "{written}"
    );
}

#[test]
fn dispatch_worker_install_refresh_is_stable_when_role_body_is_unchanged() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    let hymns = root.join(".doctrine/hymns/role");
    fs::create_dir_all(&hymns).unwrap();
    fs::write(hymns.join("worker.md"), "stable dispatch-worker body").unwrap();

    let first = install(root);
    assert!(
        first.status.success(),
        "first install failed: {}",
        String::from_utf8_lossy(&first.stderr),
    );
    let before = fs::read(root.join(".doctrine/agents/dispatch-worker.md")).unwrap();

    let second = install(root);
    assert!(
        second.status.success(),
        "second install failed: {}",
        String::from_utf8_lossy(&second.stderr),
    );
    let after = fs::read(root.join(".doctrine/agents/dispatch-worker.md")).unwrap();
    assert_eq!(before, after);
}
