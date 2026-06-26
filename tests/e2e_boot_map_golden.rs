// SPDX-License-Identifier: GPL-3.0-only
//! SL-150 PHASE-02 — black-box goldens for the dense boot-map projection.
//!
//! `doctrine --help --boot-map` is intercepted in `main` (before `--commands`)
//! and rendered by `cli::render_boot_map` — a PURE plain-text projection of the
//! compiled clap tree through the FAMILIES/SPINE taxonomy. No comfy-table, no
//! colour, no width: byte-stable regardless of tty.
//!
//! These pin the load-bearing RULES (design §5.4): a single spine legend line;
//! one header line per family (all members, bare, FAMILIES order); a per-command
//! sub-line IFF the command has distinctive verbs (verbs − SPINE) AND its family
//! is not verb-suppressed; infra is header-only (D7); a leaf command never
//! sub-lines. Verb copy itself tracks clap derive order (INV-4), not pinned byte
//! for byte here. VT-1.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::process::Command;

mod common;

fn bin() -> std::path::PathBuf {
    common::doctrine_bin()
}

const FAMILY_ORDER: &[&str] = &[
    "change",
    "governance",
    "knowledge",
    "relations",
    "facets",
    "reports",
    "explore",
    "infra",
];

fn boot_map_stdout() -> String {
    let out = Command::new(bin())
        .args(["--help", "--boot-map"])
        .output()
        .expect("spawn doctrine --help --boot-map");
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

/// Plain text: no ANSI escapes, ever (the boot snapshot is colour-free).
#[test]
fn boot_map_is_plain_text() {
    let out = boot_map_stdout();
    assert!(
        !out.contains('\u{1b}'),
        "boot map must be byte-clean (no ANSI escapes)"
    );
}

/// The spine legend renders ONCE, at the very top, naming the spine verbs.
#[test]
fn boot_map_declares_the_spine_once_at_top() {
    let out = boot_map_stdout();
    let first = out.lines().next().expect("non-empty output");
    assert_eq!(
        first, "SPINE: new list show paths (+status where lifecycle) \u{2014} entity kinds",
        "the spine legend is the first line, verbatim"
    );
    assert_eq!(
        out.matches("SPINE:").count(),
        1,
        "the spine legend appears exactly once"
    );
}

/// Family header lines appear in FAMILIES-declared order. A header line is one
/// that starts at column 0 (not indented) and whose first token is a family key.
#[test]
fn boot_map_headers_render_in_declared_order() {
    let out = boot_map_stdout();
    let headers: Vec<&str> = out
        .lines()
        .filter(|l| !l.starts_with(' ') && !l.is_empty() && !l.starts_with("SPINE:"))
        .map(|l| l.split_whitespace().next().unwrap_or(""))
        .collect();
    assert_eq!(
        headers, FAMILY_ORDER,
        "family headers must render in FAMILIES-declared order, one per family"
    );
}

/// Each family header names ALL its members, bare, on the header line.
#[test]
fn boot_map_header_names_all_members_bare() {
    let out = boot_map_stdout();
    // `governance  adr policy standard spec` — the whole member set on one line.
    let gov = out
        .lines()
        .find(|l| l.starts_with("governance "))
        .expect("governance header present");
    for member in ["adr", "policy", "standard", "spec"] {
        assert!(
            gov.split_whitespace().any(|t| t == member),
            "governance header must name member `{member}` bare"
        );
    }
}

/// A command WITH distinctive verbs in a non-suppressed family gets an indented
/// sub-line (e.g. `slice` carries design/plan/phases…).
#[test]
fn boot_map_sublines_a_command_with_distinctive_verbs() {
    let out = boot_map_stdout();
    let sub = out
        .lines()
        .find(|l| l.starts_with("  slice "))
        .expect("slice sub-line present");
    for verb in ["design", "plan", "phases"] {
        assert!(
            sub.split_whitespace().any(|t| t == verb),
            "slice sub-line must carry distinctive verb `{verb}`"
        );
    }
    // SPINE verbs are factored OUT of the distinctive set.
    for spine in ["new", "list", "show", "paths"] {
        assert!(
            !sub.split_whitespace().any(|t| t == spine),
            "spine verb `{spine}` must not appear in a distinctive sub-line"
        );
    }
}

/// D7 — the infra family is verb-suppressed: header only, NO sub-lines for any
/// of its members (e.g. `config`, `worktree`, `dispatch` have subcommands but
/// are not projected).
#[test]
fn boot_map_suppresses_infra_verbs() {
    let out = boot_map_stdout();
    for infra_member in ["config", "worktree", "dispatch", "boot"] {
        assert!(
            !out.lines()
                .any(|l| l.starts_with(&format!("  {infra_member} "))),
            "infra member `{infra_member}` must NOT get a verb sub-line (D7)"
        );
    }
    // infra is also the LAST header.
    assert!(
        out.lines().any(|l| l.starts_with("infra ")),
        "infra header present"
    );
}

/// A leaf command (no subcommands) appears in its family header but NEVER sub-lines.
#[test]
fn boot_map_leaf_command_has_no_subline() {
    let out = boot_map_stdout();
    // `search`, `link`, `reconcile` are leaves — named in headers, no sub-line.
    for leaf in ["search", "link", "reconcile"] {
        assert!(
            out.split_whitespace().any(|t| t == leaf),
            "leaf `{leaf}` must be named in a family header"
        );
        assert!(
            !out.lines().any(|l| l.starts_with(&format!("  {leaf} "))),
            "leaf `{leaf}` must NOT have a sub-line (no distinctive verbs)"
        );
    }
}

/// `--boot-map` wins over `--commands` when both are passed (documented precedence).
#[test]
fn boot_map_takes_precedence_over_commands() {
    let out = Command::new(bin())
        .args(["--help", "--commands", "--boot-map"])
        .output()
        .expect("spawn with both flags");
    let text = String::from_utf8(out.stdout).expect("utf8");
    assert!(
        text.starts_with("SPINE:"),
        "with both flags, --boot-map output (spine legend) must win"
    );
    assert!(
        !text.contains("For arguments & options:"),
        "the --commands table footer must NOT appear when --boot-map wins"
    );
}
