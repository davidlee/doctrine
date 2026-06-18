// SPDX-License-Identifier: GPL-3.0-only
//! SL-085 PHASE-04 — verify shrunk dispatch skill files meet line-count targets
//! and content requirements (VT-1).
//!
//! Reads each skill file from `plugins/doctrine/skills/<name>/SKILL.md`. The
//! compile-time embed is verified by `cargo build` (PluginAssets embeds the
//! same `plugins/` directory). Asserts:
//! - Body line count ≤ target max (not counting YAML frontmatter)
//! - Key prose present
//! - `dispatch import` references ABSENT (D1: the verb was dropped)

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::PathBuf;

/// Read a skill file from the plugins directory.
fn source_skill_text(skill: &str) -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join(format!("plugins/doctrine/skills/{skill}/SKILL.md"));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read skill file {path:?}: {e}"))
}

/// Body lines: everything after the closing `---` of the YAML frontmatter.
fn body_lines(text: &str) -> Vec<&str> {
    let mut in_frontmatter = false;
    let mut closed = false;
    let mut body_start = 0usize;
    for (i, line) in text.lines().enumerate() {
        if line == "---" {
            if !in_frontmatter {
                in_frontmatter = true;
            } else {
                closed = true;
                body_start = i + 1;
                break;
            }
        }
    }
    assert!(closed, "frontmatter never closed");
    text.lines().skip(body_start).collect()
}

#[test]
fn dispatch_router_skill_is_shrunk() {
    let full = source_skill_text("dispatch");
    let body = body_lines(&full);
    assert!(
        body.len() <= 50,
        "dispatch router body lines: {} (target ≤50)",
        body.len()
    );

    // Key prose: the new CLI verbs must be present.
    assert!(
        full.contains("dispatch setup"),
        "must reference 'dispatch setup'"
    );
    assert!(
        full.contains("dispatch plan-next"),
        "must reference 'dispatch plan-next'"
    );
    assert!(
        full.contains("report-and-halt"),
        "must mention report-and-halt"
    );
    assert!(
        full.contains("prepare-review"),
        "must mention prepare-review conclude"
    );

    // D1: "dispatch import" must be ABSENT (the verb was dropped).
    assert!(
        !full.contains("dispatch import"),
        "must NOT contain 'dispatch import' (verb dropped per D1)"
    );
}

#[test]
fn dispatch_agent_skill_is_shrunk() {
    let full = source_skill_text("dispatch-agent");
    let body = body_lines(&full);
    assert!(
        body.len() <= 30,
        "dispatch-agent body lines: {} (target ≤30)",
        body.len()
    );

    assert!(
        full.contains("subagent_type: dispatch-worker"),
        "must contain spawn template"
    );
    assert!(
        full.contains("verify-worker"),
        "must reference verify-worker"
    );
    assert!(
        full.contains("record-boundary"),
        "must reference record-boundary"
    );

    assert!(
        !full.contains("dispatch import"),
        "must NOT contain 'dispatch import' (verb dropped per D1)"
    );
}

#[test]
fn dispatch_subprocess_skill_is_shrunk() {
    let full = source_skill_text("dispatch-subprocess");
    let body = body_lines(&full);
    assert!(
        body.len() <= 25,
        "dispatch-subprocess body lines: {} (target ≤25)",
        body.len()
    );

    assert!(
        full.contains("worktree fork"),
        "must contain fork spawn template"
    );
    assert!(full.contains("env -C"), "must reference env -C cwd binding");
    assert!(
        full.contains("DOCTRINE_WORKER=1"),
        "must reference self-arm"
    );

    assert!(
        !full.contains("dispatch import"),
        "must NOT contain 'dispatch import' (verb dropped per D1)"
    );
}
