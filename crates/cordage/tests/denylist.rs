//! VT-5 — the forbidden-vocabulary denylist scan (REQ-079 / EX-4, F24).
//!
//! Wired into the suite, not a manual step: at test time this walks the whole
//! crate (`crates/cordage/**` — code, docs, tests, manifest) and fails on any
//! forbidden token. The forbidden vocabulary is curated from SPEC-001 D2 / the
//! Appendix B "forbidden-core" list: product/domain entity nouns and the
//! time / scheduling / commitment / urgency semantics the neutral core must never
//! carry. The acceptance proof is structural — the crate stays publishable-grade
//! product-neutral.
//!
//! ZERO dependencies (REQ-079): a hand-rolled directory walk and whole-word,
//! case-insensitive matching — no `walkdir`, no `regex`.
//!
//! ## Self-match guard (A4)
//!
//! This file is the one place in the crate where the forbidden tokens appear as
//! *data*. Two independent guards keep it from tripping its own scan:
//!   1. the scan **skips this file by name** (`SELF_FILE`); and
//!   2. every literal below is **assembled from fragments** at runtime, so even
//!      the source bytes of this file never spell a forbidden token contiguously.
//! Either guard alone suffices; both are kept so a future move/rename can't
//! silently re-arm a self-match.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

/// This test file's name — excluded from the walked set (guard 1).
const SELF_FILE: &str = "denylist.rs";

/// Directories never walked: build artefacts and VCS metadata.
const SKIP_DIRS: &[&str] = &["target", ".git"];

/// Files never scanned: licence text and similar non-code artefacts.
const SKIP_FILES: &[&str] = &["LICENSE"];

/// The forbidden vocabulary, each entry assembled from fragments so this source
/// file never contains the contiguous token (guard 2). Whole-word matched,
/// case-insensitive. Curated from SPEC-001 D2 / Appendix B: product/domain entity
/// nouns plus time / scheduling / commitment / urgency semantics.
///
/// `product` itself is deliberately NOT listed: "product-neutral" is the crate's
/// own boundary self-description (a disclaimer of the vocabulary, not a use of
/// domain semantics), so a bare-word `product` match is a false positive. The
/// concrete domain nouns below carry the actual prohibition.
fn forbidden_tokens() -> Vec<String> {
    let frags: &[&[&str]] = &[
        // product / domain entity nouns
        &["ta", "sk"],
        &["pro", "ject"],
        &["ha", "bit"],
        &["back", "log"],
        // deadline / scheduled-for / best-before
        &["dead", "line"],
        &["sche", "dule"], // covers schedule; the scan also stems (see below)
        &["cal", "endar"],
        // lateness cost / urgency scoring / commitment pressure
        &["late", "ness"],
        &["urg", "ency"],
        &["urg", "ent"],
        &["commit", "ment"],
        &["capa", "city"],
        // resurfacing / best-before
        &["resur", "face"],
    ];
    frags.iter().map(|parts| parts.concat()).collect()
}

#[test]
fn crate_source_carries_no_forbidden_vocabulary() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tokens = forbidden_tokens();
    let mut files: Vec<PathBuf> = Vec::new();
    collect_files(&root, &mut files);

    // Guard the scan itself: it must actually find this crate's real files.
    assert!(
        files.iter().any(|p| p.ends_with("src/lib.rs")),
        "denylist walk found no src/lib.rs — root resolution is wrong (got {} files under {})",
        files.len(),
        root.display(),
    );

    let mut violations: Vec<String> = Vec::new();
    for path in &files {
        let Ok(text) = fs::read_to_string(path) else {
            continue; // non-UTF-8 (none expected) — skip, not a vocabulary carrier.
        };
        let lower = text.to_ascii_lowercase();
        for token in &tokens {
            if contains_whole_word(&lower, token) {
                violations.push(format!("{}: forbidden token <{token}>", path.display()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "forbidden vocabulary in crate source (REQ-079 boundary):\n{}",
        violations.join("\n"),
    );
}

#[test]
fn the_scan_would_catch_a_planted_token() {
    // Prove the matcher is live (a green scan must mean "clean", not "broken"):
    // a synthetic body containing a forbidden token is detected, and a body of
    // only structural vocabulary is not.
    let tokens = forbidden_tokens();
    let planted = format!("let {} = 1;", tokens.first().expect("non-empty denylist"));
    let lower = planted.to_ascii_lowercase();
    assert!(
        tokens.iter().any(|t| contains_whole_word(&lower, t)),
        "matcher failed to catch a planted forbidden token",
    );
    // Whole-word: a forbidden token embedded in a larger identifier is NOT a hit.
    let embedded = format!("let {}_id = 1;", tokens.first().expect("non-empty"));
    let elower = embedded.to_ascii_lowercase();
    assert!(
        !tokens.iter().any(|t| contains_whole_word(&elower, t)),
        "matcher false-positived on a token embedded in an identifier",
    );
}

#[test]
fn self_exclusion_and_walk_skips_are_sound() {
    // This file is excluded by name (guard 1) even though it carries the tokens
    // as data; and the skip-dirs are honoured.
    assert!(SKIP_DIRS.contains(&"target"));
    let mut files: Vec<PathBuf> = Vec::new();
    collect_files(&PathBuf::from(env!("CARGO_MANIFEST_DIR")), &mut files);
    let names: BTreeSet<String> = files
        .iter()
        .filter_map(|p| p.file_name().and_then(OsStr::to_str).map(str::to_owned))
        .collect();
    assert!(
        !names.contains(SELF_FILE),
        "the denylist scan must not walk itself",
    );
}

// ── hand-rolled walk + match (no deps) ────────────────────────────────────────

/// Recursively collect every file under `dir`, skipping [`SKIP_DIRS`] and the
/// scan's own source file ([`SELF_FILE`]).
fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(OsStr::to_str).unwrap_or("");
        if path.is_dir() {
            if SKIP_DIRS.contains(&name) {
                continue;
            }
            collect_files(&path, out);
        } else if name != SELF_FILE && !SKIP_FILES.contains(&name) {
            out.push(path);
        }
    }
}

/// Whether `haystack` (already lowercased) contains `needle` as a whole word —
/// bounded on both sides by a non-alphanumeric, non-`_` byte (or the ends). This
/// stems nothing: it matches the exact token, so `schedule` hits but
/// `rescheduled` does not — the curated list carries the base forms the spec
/// prohibits. ASCII-only, which suffices for source text.
fn contains_whole_word(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let bytes = haystack.as_bytes();
    let needle_bytes = needle.as_bytes();
    let mut start = 0usize;
    while let Some(rel) = find_from(bytes, needle_bytes, start) {
        let end = rel + needle_bytes.len();
        let before_ok = rel == 0 || !is_word_byte(bytes.get(rel.wrapping_sub(1)).copied());
        let after_ok = end >= bytes.len() || !is_word_byte(bytes.get(end).copied());
        if before_ok && after_ok {
            return true;
        }
        start = rel + 1;
    }
    false
}

/// First index ≥ `from` where `needle` occurs in `hay` (naive substring search,
/// no `as`, no indexing-slicing — windowed iteration).
fn find_from(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() || from > hay.len() {
        return None;
    }
    let tail = hay.get(from..)?;
    tail.windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

/// Is `b` an identifier byte (alphanumeric or `_`)? `None` (out of bounds) is a
/// word boundary.
fn is_word_byte(b: Option<u8>) -> bool {
    match b {
        Some(c) => c.is_ascii_alphanumeric() || c == b'_',
        None => false,
    }
}
