// SPDX-License-Identifier: GPL-3.0-only
//! Pure lexical leaf (SL-017, design §5). The pluggable lexical axis behind the
//! SL-008 9-key `sort_key`: a tokenizer, the `LexicalRanker` trait, today's
//! set-membership `OverlapRanker`, and the `f32 → u32` `quantize`. Imports
//! nothing of `retrieve` or `memory` (ADR-001 layering, design D6) — `retrieve`
//! adapts `Memory` into the leaf's `LexDoc`, never the reverse.
//!
//! Staged `dead_code` bridge (design §9): through PHASE-02–03 every item except
//! the moved `tokenize` (still called by `retrieve::lexical_score`) is reachable
//! only from tests; `retrieve` wires the ranker at PHASE-04, the self-clearing
//! condition under which this attribute is removed.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "lexical consumers wired into retrieve at PHASE-04"
    )
)]

use std::collections::{BTreeMap, BTreeSet};

/// Case-fold + split on non-alphanumeric, dropping empties. The shared lexer for
/// the lexical axis — `mem.foo.bar` / `src/x.rs` split on their separators too.
/// General-purpose: `retrieve` imports it; this leaf imports nothing of retrieval.
pub(crate) fn tokenize(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// A memory-agnostic scoring document. `id` is the memory uid (caller-canonical,
/// assumed unique). `text` is the pre-concatenated doc bag (design §5.3).
pub(crate) struct LexDoc {
    pub(crate) id: String,
    pub(crate) text: String,
}

/// The fit-corpus view. One variant in SL-017; `Indexed(&LexicalIndex)` is the
/// non-breaking evolution when an active-set index is precomputed (follow-up, D7).
pub(crate) enum LexicalCorpus<'a> {
    Raw(&'a [LexDoc]),
}

impl LexicalCorpus<'_> {
    fn docs(&self) -> &[LexDoc] {
        match self {
            LexicalCorpus::Raw(docs) => docs,
        }
    }
}

pub(crate) trait LexicalRanker {
    /// Fit corpus-level statistics over `corpus`, score `targets`, and return one
    /// `(id, u32)` entry **per target, in `targets` order** (Key-2 magnitude).
    ///
    /// Contract (A1 — completeness is total, not best-effort):
    /// - The result has **exactly `targets.len()` entries, one per target, in
    ///   order.** A no-evidence target (query `None` / empty after tokenize / no
    ///   term matched) is filled with `0` *inside the ranker*. Candidate assembly
    ///   indexes positionally — it never uses `unwrap_or(0)`; an absent entry is a
    ///   bug.
    /// - **Hard precondition (all builds):** `assert!` every target id is present
    ///   in `corpus` (targets ⊆ corpus). A target outside the fit corpus is an
    ///   internal invariant violation (survivors ⊆ active = corpus) — fail loud,
    ///   never silently demote to 0.
    /// - Duplicate ids in `corpus` violate uniqueness (`debug_assert!`; release is
    ///   last-wins, undefined-but-safe).
    /// - Future (A4): an `Indexed` corpus may fail at *construction* time; `Raw`
    ///   never does.
    fn score(
        &self,
        query: Option<&str>,
        corpus: &LexicalCorpus<'_>,
        targets: &[&str],
    ) -> Vec<(String, u32)>;
}

/// Distinct-query-token set-membership over `LexDoc.text` (re-tokenized). Returns
/// the `u32` count directly — **no quantize** — byte-identical to the SL-008
/// `lexical_score` it will retire (design §5.4). Corpus statistics are ignored:
/// this is a per-document signal.
pub(crate) struct OverlapRanker;

/// BM25 f32 (>= 0 under Lucene IDF) -> Key-2 u32 (design §5.2). Monotonic
/// non-decreasing, saturating, total (non-finite -> 0: invalid evidence, never
/// maximal).
pub(crate) const LEX_SCALE: f32 = 1_000_000.0;

pub(crate) fn quantize(score: f32) -> u32 {
    debug_assert!(score.is_finite(), "non-finite lexical score: scorer bug"); // A8: surface upstream bugs
    if !score.is_finite() {
        return 0;
    }
    let scaled = (score.max(0.0) * LEX_SCALE).round();
    // Rust's float→int `as` is *saturating* (since 1.45): out-of-range → u32::MAX,
    // and `scaled` is finite and >= 0 here, so this is total, monotonic, and the
    // saturation the design specifies — no separate ceiling branch needed. There
    // is no safe std float→int API; this guarded `as` carries the house-style
    // expect+reason (the only `as` in `src`, mem.pattern.lint.expect-not-allow).
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "saturating float→u32 (Rust >= 1.45); scaled is finite and >= 0; no safe std API"
    )]
    let q = scaled as u32;
    q
}

/// Shared hard precondition (A1): every target must be present in the fit corpus,
/// and corpus ids must be unique. Used by every `LexicalRanker` impl.
fn assert_targets_subset(corpus: &LexicalCorpus<'_>, targets: &[&str]) {
    let docs = corpus.docs();
    debug_assert!(
        docs.iter()
            .map(|d| d.id.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            == docs.len(),
        "duplicate corpus ids violate uniqueness"
    );
    let ids: BTreeSet<&str> = docs.iter().map(|d| d.id.as_str()).collect();
    for t in targets {
        assert!(
            ids.contains(t),
            "target id not in fit corpus (targets ⊆ corpus violated)"
        );
    }
}

impl LexicalRanker for OverlapRanker {
    fn score(
        &self,
        query: Option<&str>,
        corpus: &LexicalCorpus<'_>,
        targets: &[&str],
    ) -> Vec<(String, u32)> {
        assert_targets_subset(corpus, targets);
        let q_tokens: BTreeSet<String> = match query {
            Some(q) => tokenize(q).into_iter().collect(),
            None => BTreeSet::new(),
        };
        let by_id: BTreeMap<&str, &str> = corpus
            .docs()
            .iter()
            .map(|d| (d.id.as_str(), d.text.as_str()))
            .collect();
        targets
            .iter()
            .map(|t| {
                let hits = match (q_tokens.is_empty(), by_id.get(t)) {
                    (false, Some(text)) => {
                        let bag: BTreeSet<String> = tokenize(text).into_iter().collect();
                        q_tokens.iter().filter(|qt| bag.contains(*qt)).count()
                    }
                    _ => 0,
                };
                ((*t).to_string(), u32::try_from(hits).unwrap_or(u32::MAX))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- tokenize (the shared lexical lexer, moved from retrieve) ------------

    #[test]
    fn tokenize_casefolds_and_splits_on_non_alphanumeric() {
        assert_eq!(tokenize("Auth Bug"), vec!["auth", "bug"]);
        // punctuation, underscore and slash all split; empties dropped
        assert_eq!(
            tokenize("src/memory.rs__OK"),
            vec!["src", "memory", "rs", "ok"]
        );
        // key segments split on the dot
        assert_eq!(tokenize("mem.auth.token"), vec!["mem", "auth", "token"]);
        assert!(tokenize("   ").is_empty());
    }

    // -- quantize (design §5.2 / §9, VT-3) ----------------------------------

    #[test]
    fn quantize_zero_is_zero() {
        assert_eq!(quantize(0.0), 0);
    }

    #[test]
    fn quantize_is_monotonic_non_decreasing() {
        let xs = [0.0_f32, 1e-6, 0.1, 1.0, 2.5, 30.0, 1000.0];
        for w in xs.windows(2) {
            assert!(
                quantize(w[0]) <= quantize(w[1]),
                "quantize not monotonic at {:?}",
                w
            );
        }
    }

    #[test]
    fn quantize_saturates() {
        assert_eq!(quantize(f32::MAX), u32::MAX);
        // anything scaling past the ceiling saturates, not truncates/wraps
        assert_eq!(quantize(1e30), u32::MAX);
    }

    // Non-finite is profile-split (design §5.2 clarification): a non-finite BM25
    // score is a scorer bug, not ordinary input — debug explodes (A8); release
    // stays total and degrades to 0 so invalid evidence is never maximal.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "non-finite lexical score")]
    fn quantize_non_finite_panics_in_debug() {
        let _ = quantize(f32::NAN);
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn quantize_non_finite_is_zero_in_release() {
        assert_eq!(quantize(f32::NAN), 0);
        assert_eq!(quantize(f32::INFINITY), 0);
    }

    #[test]
    fn quantize_negative_is_zero() {
        // the defensive `max(0.0)` — a finite negative, so no debug_assert conflict
        assert_eq!(quantize(-1.0), 0);
    }

    // -- OverlapRanker (design §5.4, VT-2) ----------------------------------

    fn doc(id: &str, text: &str) -> LexDoc {
        LexDoc {
            id: id.to_string(),
            text: text.to_string(),
        }
    }

    #[test]
    fn overlap_counts_distinct_query_tokens_over_text() {
        let docs = vec![doc("a", "token expiry middleware check rust mem auth flow")];
        let corpus = LexicalCorpus::Raw(&docs);
        // 4 distinct hits
        assert_eq!(
            OverlapRanker.score(Some("token middleware rust auth"), &corpus, &["a"]),
            vec![("a".to_string(), 4)]
        );
        // a repeated query token counts once (SET semantics)
        assert_eq!(
            OverlapRanker.score(Some("token token token"), &corpus, &["a"]),
            vec![("a".to_string(), 1)]
        );
        // no overlap ⇒ 0
        assert_eq!(
            OverlapRanker.score(Some("python django"), &corpus, &["a"]),
            vec![("a".to_string(), 0)]
        );
    }

    #[test]
    fn overlap_no_query_is_all_zero() {
        let docs = vec![doc("a", "token"), doc("b", "auth")];
        let corpus = LexicalCorpus::Raw(&docs);
        assert_eq!(
            OverlapRanker.score(None, &corpus, &["a", "b"]),
            vec![("a".to_string(), 0), ("b".to_string(), 0)]
        );
        // empty / separators-only query tokenizes to nothing ⇒ 0
        assert_eq!(
            OverlapRanker.score(Some(""), &corpus, &["a"]),
            vec![("a".to_string(), 0)]
        );
        assert_eq!(
            OverlapRanker.score(Some("   ... "), &corpus, &["a"]),
            vec![("a".to_string(), 0)]
        );
    }

    #[test]
    fn overlap_is_positional_over_targets() {
        let docs = vec![doc("a", "token"), doc("b", "auth"), doc("c", "rust")];
        let corpus = LexicalCorpus::Raw(&docs);
        // result follows targets order, one entry each, corpus order ignored
        let got = OverlapRanker.score(Some("rust token"), &corpus, &["c", "a"]);
        assert_eq!(got, vec![("c".to_string(), 1), ("a".to_string(), 1)]);
        assert_eq!(got.len(), 2);
    }

    #[test]
    fn overlap_empty_targets_is_empty_vec() {
        let docs = vec![doc("a", "token")];
        let corpus = LexicalCorpus::Raw(&docs);
        assert!(OverlapRanker.score(Some("token"), &corpus, &[]).is_empty());
    }

    // -- contract invariants (A1, VT-4) -------------------------------------

    #[test]
    #[should_panic(expected = "targets ⊆ corpus")]
    fn target_outside_corpus_panics_in_all_builds() {
        let docs = vec![doc("a", "token")];
        let corpus = LexicalCorpus::Raw(&docs);
        let _ = OverlapRanker.score(Some("token"), &corpus, &["ghost"]);
    }

    #[test]
    #[should_panic(expected = "duplicate corpus ids")]
    fn duplicate_corpus_ids_trip_debug_assert() {
        let docs = vec![doc("a", "token"), doc("a", "other")];
        let corpus = LexicalCorpus::Raw(&docs);
        let _ = OverlapRanker.score(Some("token"), &corpus, &["a"]);
    }
}
