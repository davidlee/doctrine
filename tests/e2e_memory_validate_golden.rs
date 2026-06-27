// SPDX-License-Identifier: GPL-3.0-only
//! Byte-exact golden for `memory validate` — guards the pure extraction (PHASE-02
//! of SL-168): the rendered output must be identical before and after the pure
//! memory-health fn was lifted out of `run_validate`.
//!
//! Hermetic by construction (RV-185 F-2): runs the built binary against a SEEDED
//! fixture corpus via `-p`, never the live repo. The earlier live-corpus byte
//! golden baked a volatile `N commits behind HEAD` count and a live `mem_*` set,
//! so it went red the moment the repo gained a commit — it could not be the
//! stable behaviour-preservation proof D12 requires. The dangling-relation check
//! exercises the same render path with no git/clock dependency (staleness needs
//! git history and is deliberately out of scope for the golden).

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic"
)]

use std::path::Path;
use std::process::Command;

mod common;

/// Seed one memory with the given outbound relations under `root`.
fn seed_memory(root: &Path, uid: &str, relations: &[(&str, &str)]) {
    let dir = root.join(format!(".doctrine/memory/items/{uid}"));
    std::fs::create_dir_all(&dir).expect("mkdir memory item");
    let relation_rows: String = relations
        .iter()
        .map(|(label, target)| {
            format!("[[relation]]\nlabel = \"{label}\"\ntarget = \"{target}\"\n")
        })
        .collect();
    std::fs::write(
        dir.join("memory.toml"),
        format!(
            "memory_uid = \"{uid}\"\n\
             schema_version = 1\n\
             memory_type = \"pattern\"\n\
             status = \"active\"\n\
             title = \"{uid}\"\n\
             summary = \"summary\"\n\
             created = \"2026-01-01\"\n\
             updated = \"2026-01-01\"\n\
             [scope]\n\
             workspace = \"default\"\n\
             [git]\n\
             repo = \"repo\"\n\
             [trust]\n\
             level = \"medium\"\n\
             [ranking]\n\
             severity = \"none\"\n\
             weight = 0\n\
             {relation_rows}"
        ),
    )
    .expect("write memory.toml");
    std::fs::write(dir.join("memory.md"), "body\n").expect("write memory.md");
}

#[test]
fn memory_validate_dangling_relation_byte_exact_golden() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    std::fs::create_dir_all(root.join(".doctrine")).expect("mkdir .doctrine");

    // One memory with a dangling relation to a target that does not exist.
    seed_memory(
        root,
        "mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        &[("related", "mem_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")],
    );

    let output = Command::new(common::doctrine_bin())
        .args(["memory", "validate"])
        .arg("-p")
        .arg(root)
        .output()
        .expect("spawn doctrine memory validate");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected = "mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: dangling: [[relation]] target \"mem_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\" not found";

    assert_eq!(stdout.trim(), expected.trim());
    // Any dangling finding is an error → non-zero exit (behaviour-preserved).
    assert!(
        !output.status.success(),
        "dangling relation must exit non-zero"
    );
}
