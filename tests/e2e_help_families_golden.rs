// SPDX-License-Identifier: GPL-3.0-only
//! SL-150 PHASE-01 — black-box goldens for the family-grouped human `--help`.
//!
//! `doctrine --help` is intercepted (not clap's built-in) and rendered by
//! `render_top_level_help` → `listing::render_grouped`: ONE underlying comfy-table of
//! all command rows (shared column widths), with a full-width family-heading BAND
//! injected into the line stream at each group boundary. Spawned with piped stdout the
//! binary resolves colour OFF + width None (`force_no_tty` path), so the bands degrade
//! to plain `  {key}` + blank lines and the output is byte-stable.
//!
//! These pin the load-bearing STRUCTURE — family order, banded grouping (blank/key/blank),
//! shared column alignment, no column header (A2) — rather than every command's
//! description byte (those track clap `about` copy, out of this slice's scope, R4). VT-2.

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

/// The 8 family keys, in their canonical render order (FAMILIES declaration order).
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

fn help_stdout() -> String {
    let out = Command::new(bin())
        .arg("--help")
        .output()
        .expect("spawn doctrine --help");
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

/// Colour OFF (piped) ⇒ NO ANSI escapes anywhere — the byte-golden path.
#[test]
fn help_piped_emits_no_ansi() {
    let out = help_stdout();
    assert!(
        !out.contains('\u{1b}'),
        "piped --help must be byte-clean (no ANSI escapes)"
    );
}

/// Family heading bands appear in FAMILIES-declared order (INV-4), each as a
/// `  {key}` line — the navigational spine of the grouped help.
#[test]
fn help_renders_families_in_declared_order() {
    let out = help_stdout();
    let headings: Vec<&str> = out
        .lines()
        .filter_map(|l| l.strip_prefix("  "))
        // family heading lines carry a single bare token (no separator); command rows
        // carry the ` │ ` separator and are NOT two-space-indented.
        .filter(|rest| !rest.contains('\u{2502}') && !rest.is_empty())
        .collect();
    assert_eq!(
        headings, FAMILY_ORDER,
        "family headings must render in declared order, banded as `  key`"
    );
}

/// Each family heading is a 3-line band: a blank line, `  {key}`, a blank line
/// (colour-off degradation). Assert the blank-above/blank-below structure.
#[test]
fn help_bands_each_family_with_blank_lines() {
    let out = help_stdout();
    let lines: Vec<&str> = out.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let Some(rest) = line.strip_prefix("  ") else {
            continue;
        };
        if rest.contains('\u{2502}') || rest.is_empty() {
            continue; // command row, or the band's own blank line
        }
        // `rest` is a family key. The line above and below must be blank.
        assert!(
            i > 0 && lines[i - 1].is_empty(),
            "family band `{rest}` must have a blank line above"
        );
        assert!(
            lines.get(i + 1).is_some_and(|l| l.is_empty()),
            "family band `{rest}` must have a blank line below"
        );
    }
}

/// Shared column widths across ALL families (D8): every command row's `│`
/// separator sits at the SAME column — one underlying table, not 8.
#[test]
fn help_shares_one_column_width_across_all_families() {
    let out = help_stdout();
    let sep_cols: std::collections::BTreeSet<usize> =
        out.lines().filter_map(|l| l.find('\u{2502}')).collect();
    assert_eq!(
        sep_cols.len(),
        1,
        "all command rows must share one separator column (shared-width table); got {sep_cols:?}"
    );
}

/// No column-header row (A2): the help omits comfy-table's `command │ description`
/// header — the families ARE the structure.
#[test]
fn help_omits_the_column_header() {
    let out = help_stdout();
    assert!(
        !out.contains("command \u{2502} description"),
        "grouped help must NOT carry a `command │ description` header row"
    );
}

/// A representative command from each family appears in its expected family block,
/// pinning the classification end-to-end (slice → change, adr → governance, …).
#[test]
fn help_places_commands_under_their_family() {
    let out = help_stdout();
    let block_of = |key: &str| -> String {
        let start = out
            .find(&format!("  {key}\n"))
            .unwrap_or_else(|| panic!("family `{key}` heading not found"));
        let rest = &out[start..];
        // up to the next blank-blank-key band or end.
        rest.to_string()
    };
    // Each sample must precede the NEXT family heading (i.e. live in its own block).
    let samples = [
        ("change", "slice"),
        ("governance", "adr"),
        ("knowledge", "memory"),
        ("relations", "link"),
        ("facets", "estimate"),
        ("reports", "status"),
        ("explore", "search"),
        ("infra", "install"),
    ];
    for (family, cmd) in samples {
        let block = block_of(family);
        let cmd_pos = block
            .find(&format!("{cmd} "))
            .or_else(|| block.find(&format!("{cmd}\u{2502}")))
            .unwrap_or_else(|| panic!("command `{cmd}` not found after family `{family}`"));
        // The next family heading (if any) must come AFTER this command.
        let next_heading = FAMILY_ORDER
            .iter()
            .skip_while(|k| **k != family)
            .nth(1)
            .and_then(|nk| block.find(&format!("  {nk}\n")));
        if let Some(nh) = next_heading {
            assert!(
                cmd_pos < nh,
                "command `{cmd}` must fall inside family `{family}`'s block"
            );
        }
    }
}
