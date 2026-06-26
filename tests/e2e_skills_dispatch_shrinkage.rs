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

mod common;

/// Read a skill file from the plugins directory.
fn source_skill_text(skill: &str) -> String {
    let path = common::repo_root().join(format!("plugins/doctrine/skills/{skill}/SKILL.md"));
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
    // SL-127 PHASE-05 added the "Base freshness (mid-drive)" routing section
    // (refresh-base), raising the lean-router budget from 64 to 74.
    // SL-147 PHASE-06 added the conformance record beat (step 8, per-arm
    // boundary write), raising it from 74 to 80.
    // SL-154 PHASE-06 documented the enforced prepare-review derive+gate beat
    // (step 8 enforcement note + Conclude paragraph), raising it from 80 to 90.
    assert!(
        body.len() <= 90,
        "dispatch router body lines: {} (target ≤90)",
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
    // SL-147 PHASE-06: record-boundary now double-writes the conformance
    // registry — documenting that raised the budget from 78 to 82.
    // SL-152 PHASE-05: the WorktreeCreate-hook contract replaced the
    // placement-implicit base with the arm-spawn / cd-in-cd-back bracket and
    // the worktreePath-derived branch (post-spawn), raising it from 82 to 100.
    // SL-154 PHASE-06: documented the prepare-review derive/gate as enforced
    // machinery + record-delta as escape-hatch-only, raising it from 100 to 104.
    assert!(
        body.len() <= 104,
        "dispatch-agent body lines: {} (target ≤104)",
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
    assert!(full.contains("base-guard"), "must reference base-guard");
    assert!(full.contains("not-isolated"), "must reference not-isolated");
    assert!(
        full.contains("branch-mismatch"),
        "must reference branch-mismatch"
    );
    assert!(full.contains("worktreePath"), "must reference worktreePath");

    assert!(
        !full.contains("dispatch import"),
        "must NOT contain 'dispatch import' (verb dropped per D1)"
    );
}
