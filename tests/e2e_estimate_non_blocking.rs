// SPDX-License-Identifier: GPL-3.0-only
//! SL-104 PHASE-01 — REQ-275 structural non-blocking tripwire (NF-001 / VT-1).
//!
//! Source-scan allowlist: every file under `src/` that names a facet symbol
//! (EstimateFacet, ValueFacet, EstimationConfig, ValueConfig, resolve_confidence,
//! crate::estimate, crate::value, estimate::, value::) must be in the known
//! exposure surface. Any NEW file naming a facet symbol fails the test — the
//! non-blocking guarantee is structural: a gating path that reads estimate/value
//! is ruled out by construction.
//!
//! Tier-2 (in `src/slice.rs`) complements this with a compile-time Gate
//! destructure that proves the closure-gate input type is structurally
//! facet-free.
//!
//! Because `resolve_confidence` is in the symbol set, this same test also proves
//! no non-allowlist consumer of confidence exists (the display-only guarantee).
//!
//! # CHR-014 stale-shared-target-path hazard
//! `CARGO_MANIFEST_DIR` is compile-time relative to the test binary's crate root.
//! In a cargo build with shared target dir (e.g. workspace or `--target-dir`),
//! stale binaries may embed the wrong path. `cargo clean -p doctrine` before a
//! suspect run is the escape hatch.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration test: fail-fast unwrap/expect are idiomatic, and test fns live at crate root by construction"
)]

use std::path::PathBuf;

/// The set of precise symbol strings whose presence in a source file
/// constitutes a facet reference. Bare words like `estimate`/`value` are NOT
/// included — they collide with `toml::Value`, field names, and prose.
const FACET_SYMBOLS: &[&str] = &[
    "EstimateFacet",
    "ValueFacet",
    "EstimationConfig",
    "ValueConfig",
    "resolve_confidence",
    "crate::estimate",
    "crate::value",
    "estimate::",
    "value::",
];

/// Files known to name facet symbols. These are the legitimate exposure surface:
/// the facet definitions themselves, their configuration readers, the catalog
/// that hydrates them, the CLI handlers (main.rs), and the SliceDoc that
/// carries them as optional fields.
///
/// `value::` is a broad substring that collides with `serde::de::value::`,
/// `toml::value::`, and `serde_json::from_value::` — those lines are excluded
/// from matching (see `line_matches_symbol`), so false-positives in revision.rs,
/// knowledge.rs, and backlog.rs are correctly skipped.
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

fn src_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Walk `src/**/*.rs`, returning every file path relative to `src/`.
fn src_files(root: &PathBuf) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

fn walk(base: &PathBuf, dir: &PathBuf, out: &mut Vec<PathBuf>) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
            } else if p.extension().is_some_and(|ext| ext == "rs") {
                let rel = p.strip_prefix(base).unwrap().to_path_buf();
                out.push(rel);
            }
        }
    }
}

/// Does `line` name a facet symbol? For `estimate::` and `value::`, exclude
/// lines that mention `serde::`, `serde_json::`, or `toml::` — those are
/// library-level path segments, NOT references to the crate's facet modules.
fn line_matches_facet_symbol(line: &str) -> bool {
    let trimmed = line.trim();
    for sym in FACET_SYMBOLS {
        if trimmed.contains(sym) {
            // For `estimate::` and `value::`, guard against serde/toml false positives.
            if *sym == "estimate::" || *sym == "value::" {
                if trimmed.contains("serde::")
                    || trimmed.contains("serde_json::")
                    || trimmed.contains("toml::")
                {
                    continue;
                }
            }
            return true;
        }
    }
    false
}

fn file_matches_facet_symbol(path: &PathBuf) -> bool {
    let full = src_dir().join(path);
    let text =
        std::fs::read_to_string(&full).unwrap_or_else(|e| panic!("read {}: {e}", full.display()));
    text.lines().any(line_matches_facet_symbol)
}

#[test]
fn no_facet_symbol_outside_allowlist() {
    let root = src_dir();
    let files = src_files(&root);
    let mut offenders: Vec<String> = Vec::new();

    for file in &files {
        let rel = file.to_string_lossy().to_string();
        if ALLOWLIST.contains(&rel.as_str()) {
            continue;
        }
        if file_matches_facet_symbol(file) {
            offenders.push(rel);
        }
    }

    if !offenders.is_empty() {
        panic!(
            "NF-001 structural non-blocking tripwire FAILED — facet symbol(s) found outside allowlist:\n  {}\n\n\
             Every file naming EstimateFacet, ValueFacet, EstimationConfig, ValueConfig, \
             resolve_confidence, crate::estimate, crate::value, estimate::, or value:: \
             must be in the ALLOWLIST (the known exposure surface). A new gating path \
             that reads estimate/value would appear here.",
            offenders.join("\n  ")
        );
    }
}
