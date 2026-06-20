// SPDX-License-Identifier: GPL-3.0-only
//! NF-001 structural non-blocking tripwire: Tier 1 — allowlist source-scan.
//!
//! Scans all `src/**/*.rs` for estimate / value facet symbols and asserts every
//! match lands in the known exposure surface (the allowlist). Any new file naming
//! a facet symbol breaks this test, forcing explicit review of the exposure.
//!
//! Because `resolve_confidence` is in the symbol set, this same test proves no
//! non-allowlist consumer of confidence exists (the display-only guarantee).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The workspace root, via the compile-time crate directory.
fn src_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Recursively collect every `.rs` file under `dir` (skipping directories that
/// aren't Rust modules). Returns file paths relative to `dir`.
fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(collect_rs_files(&path));
        } else if path.extension().map_or(false, |e| e == "rs") {
            out.push(path);
        }
    }
    out
}

/// The known exposure surface — files that are allowed to name facet symbols.
const ALLOWLIST: &[&str] = &[
    "estimate.rs",
    "value.rs",
    "estimate/display.rs",
    "dtoml.rs",
    "catalog/scan.rs",
    "catalog/graph.rs",
    "catalog/hydrate.rs",
    "slice.rs",
    "main.rs",
];

/// Normalise a path to the `src/`-relative form used in the allowlist.
fn src_relative(path: &Path, src: &Path) -> String {
    path.strip_prefix(src)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

/// Check whether `text` contains the literal string `pat` on any line, excluding
/// lines that match any entry in `exclude_line_patterns`.
fn text_contains_pattern(text: &str, pat: &str, exclude_line_patterns: &[&str]) -> bool {
    text.lines().any(|line| {
        if !line.contains(pat) {
            return false;
        }
        if exclude_line_patterns.iter().any(|excl| line.contains(excl)) {
            return false;
        }
        true
    })
}

/// Symbol strings that indicate facet usage. Each is checked as a literal
/// substring, with the exclusion filters noted below.
const SYMBOLS: &[Symbol] = &[
    Symbol::simple("EstimateFacet"),
    Symbol::simple("ValueFacet"),
    Symbol::simple("EstimationConfig"),
    Symbol::simple("ValueConfig"),
    Symbol::simple("resolve_confidence"),
    Symbol::simple("crate::estimate"),
    Symbol::simple("crate::value"),
    Symbol::simple("estimate::"),
    // `value::` is precise only when NOT inside `toml::value::`,
    // `serde::de::value::`, or `serde_json::from_value::` — those are
    // serde/TOML library paths, not the doctrine value module.
    Symbol::with_exclusions(
        "value::",
        &[
            "toml::value::",
            "serde::de::value::",
            "serde_json::from_value::",
        ],
    ),
];

struct Symbol {
    pat: &'static str,
    exclusions: &'static [&'static str],
}

impl Symbol {
    const fn simple(pat: &'static str) -> Self {
        Self {
            pat,
            exclusions: &[],
        }
    }
    const fn with_exclusions(pat: &'static str, exclusions: &'static [&'static str]) -> Self {
        Self { pat, exclusions }
    }
}

/// Which symbols does a file match? Returns the FIRST matching symbol name (for
/// the error message); the test fails on the first offender per file.
fn offending_symbol(text: &str) -> Option<&'static str> {
    for sym in SYMBOLS {
        if text_contains_pattern(text, sym.pat, sym.exclusions) {
            return Some(sym.pat);
        }
    }
    None
}

#[test]
fn facet_symbols_are_confined_to_allowlist() {
    // CHR-014: `CARGO_MANIFEST_DIR` points at the crate root, not the workspace
    // root — this is correct here because `src/` lives beside `Cargo.toml`. A
    // stale shared target path hazard exists if the build directory changes, but
    // the manifest dir stays reliable for source discovery.
    let src = src_dir();
    let files = collect_rs_files(&src);

    let allowlist: BTreeSet<&str> = ALLOWLIST.iter().copied().collect();

    let mut offenders: Vec<(String, &str)> = Vec::new();
    for file in &files {
        let rel = src_relative(file, &src);
        let text = std::fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("read {}: {e}", file.display()));
        if let Some(sym) = offending_symbol(&text) {
            if !allowlist.contains(rel.as_str()) {
                offenders.push((rel, sym));
            }
        }
    }

    if !offenders.is_empty() {
        let mut msg = String::from(
            "NF-001 TRIPWIRE: facet symbols found outside the allowlist exposure surface.\n",
        );
        for (file, sym) in &offenders {
            msg.push_str(&format!("  {} — found `{sym}`\n", file));
        }
        msg.push_str("The closure-gate path must not read estimate/value facets.\n");
        msg.push_str("Either remove the reference or add the file to the ALLOWLIST\n");
        msg.push_str("(tests/e2e_estimate_non_blocking.rs) after confirming it is not\n");
        msg.push_str("a gating-path consumer.\n");
        panic!("{msg}");
    }
}
