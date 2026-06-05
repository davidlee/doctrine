//! SL-017 PHASE-01 — bm25 2.3.2 characterization probe (the risk gate).
//!
//! A KEEPER characterization test (not a throwaway spike): it pins the exact
//! integration recipe doctrine will build `Bm25Ranker` on (design OQ-3, §5.4),
//! under `--no-default-features` (the permanent build mode — the crate dep sets
//! it; `Language`/`DefaultTokenizer` stemming stack is absent here).
//!
//! It deliberately uses a TRIVIAL LOCAL whitespace tokenizer, NOT doctrine's
//! `lexical::tokenize` (which does not exist until PHASE-02): it characterizes
//! bm25 MECHANICS, independent of doctrine's lexer. PHASE-03 VT-5 re-checks the
//! avgdl invariant with the real lexer.
//!
//! Pinned facts (recipe — also harvested to notes.md):
//! - `Tokenizer::tokenize(&self, &str) -> Vec<String>` (OQ-3.1).
//! - `EmbedderBuilder::with_tokenizer_and_fit_to_corpus(tokenizer, &[&str])` —
//!   the NON-`Language` custom-tokenizer corpus-fit path (OQ-3.2). It computes
//!   `avgdl = mean(tokenizer.tokenize(doc).len())` over the corpus using the SAME
//!   tokenizer instance the embedder uses for per-doc length — so the A3
//!   avgdl/doc_len equivalence is STRUCTURAL, not hand-maintained (stronger than
//!   the design's `with_avgdl` + self-computed-mean plan; see notes.md).
//! - `Scorer::<String>` (`K: Eq + Hash + Clone`) keyed by uid; `matches(&embed(q))`
//!   returns `Vec<ScoredDocument<K>>` (OQ-3.3). We consume id→score, discard the
//!   crate's descending sort (HashSet-derived, not cross-process stable).

use bm25::{EmbedderBuilder, Scorer, Tokenizer};

/// The probe's local lexer — whitespace split, MULTISET (repeats preserved).
/// Stands in for doctrine's future `lexical::tokenize` for bm25 characterization.
struct WhitespaceTokenizer;

impl Tokenizer for WhitespaceTokenizer {
    fn tokenize(&self, input_text: &str) -> Vec<String> {
        input_text.split_whitespace().map(str::to_string).collect()
    }
}

/// A deliberately-miscounting tokenizer (dedups), used only to prove the avgdl
/// differential can DETECT divergence (design §9 / VT-2). Not a real recipe.
struct DedupTokenizer;

impl Tokenizer for DedupTokenizer {
    fn tokenize(&self, input_text: &str) -> Vec<String> {
        let mut seen = std::collections::BTreeSet::new();
        input_text
            .split_whitespace()
            .filter(|t| seen.insert(t.to_string()))
            .map(str::to_string)
            .collect()
    }
}

// `DefaultTokenEmbedder`/`DefaultEmbeddingSpace` are unexported, so we name a
// concrete `TokenEmbedder` for the embedding space `D`: `u32` (as the crate's own
// tests do). The hash space is irrelevant to the BM25 maths we exercise.
type Space = u32;

/// Build the embedder over `corpus` with a custom tokenizer (the pinned recipe),
/// upsert every doc into a `Scorer<String>`, and return (scorer, embedder).
fn fit(
    corpus: &[(&str, &str)],
) -> (
    Scorer<String, Space>,
    bm25::Embedder<Space, WhitespaceTokenizer>,
) {
    let texts: Vec<&str> = corpus.iter().map(|(_, t)| *t).collect();
    let embedder =
        EmbedderBuilder::<Space, _>::with_tokenizer_and_fit_to_corpus(WhitespaceTokenizer, &texts)
            .build();
    let mut scorer = Scorer::<String, Space>::new();
    for (id, text) in corpus {
        scorer.upsert(&(*id).to_string(), embedder.embed(text));
    }
    (scorer, embedder)
}

/// T3 / VT-1 — the recipe compiles + runs under `--no-default-features`, and the
/// IDF effect propagates end-to-end: a rare-term doc outscores a common-term doc.
#[test]
fn recipe_runs_and_idf_ranks_rare_over_common() {
    let corpus = [
        ("d0", "common alpha"),
        ("d1", "common beta"),
        ("d2", "common gamma"),
        ("d3", "rare delta"),
    ];
    let (scorer, embedder) = fit(&corpus);

    let matches = scorer.matches(&embedder.embed("common rare"));
    let score = |id: &str| {
        matches
            .iter()
            .find(|m| m.id == id)
            .map(|m| m.score)
            .unwrap_or(0.0)
    };

    // "rare" (df=1) carries far more IDF than "common" (df=3): d3 outranks d0.
    assert!(score("d3") > 0.0, "rare-term doc must match");
    assert!(score("d0") > 0.0, "common-term doc must match");
    assert!(
        score("d3") > score("d0"),
        "rare-term doc must outscore common-term doc: d3={} d0={}",
        score("d3"),
        score("d0")
    );
}

/// T4 / VT-2 — A3 avgdl/doc_len equivalence. The crate's fitted `avgdl` equals the
/// MULTISET mean token length under the SAME tokenizer; a miscounting tokenizer
/// diverges (proving the differential detects divergence).
#[test]
fn avgdl_equals_multiset_mean_token_length() {
    // Known token lengths, INCLUDING a repeated-token doc ("a a a" = 3, multiset).
    let texts = ["a a a", "b c", "d"];
    let tok = WhitespaceTokenizer;
    let reference_mean = {
        let total: usize = texts.iter().map(|t| tok.tokenize(t).len()).sum();
        total as f32 / texts.len() as f32
    };
    assert_eq!(reference_mean, 2.0, "guard: (3+2+1)/3 == 2.0");

    let embedder =
        EmbedderBuilder::<Space, _>::with_tokenizer_and_fit_to_corpus(WhitespaceTokenizer, &texts)
            .build();
    assert_eq!(
        embedder.avgdl(),
        reference_mean,
        "crate avgdl must equal multiset mean token length (A3, multiset not deduped)"
    );

    // Divergence detector: dedup tokenizer counts "a a a" as 1 → mean (1+2+1)/3.
    let dedup_embedder =
        EmbedderBuilder::<Space, _>::with_tokenizer_and_fit_to_corpus(DedupTokenizer, &texts)
            .build();
    assert_ne!(
        dedup_embedder.avgdl(),
        reference_mean,
        "a miscounting tokenizer MUST diverge (the test can detect divergence)"
    );
}

/// VT-3 (same-process half) — repeat calls on identical inputs yield identical
/// score values. The cross-process half is `examples/bm25_probe.rs` (T5).
#[test]
fn same_process_repeat_call_is_identical() {
    let corpus = [
        ("d0", "alpha beta"),
        ("d1", "beta gamma gamma"),
        ("d2", "alpha"),
    ];
    let (scorer, embedder) = fit(&corpus);
    let q = embedder.embed("beta gamma");

    let run = || {
        let mut v: Vec<(String, f32)> = scorer
            .matches(&q)
            .into_iter()
            .map(|m| (m.id, m.score))
            .collect();
        v.sort_by(|a, b| a.0.cmp(&b.0)); // we impose our own order; bm25's is discarded
        v
    };
    assert_eq!(run(), run(), "repeat scoring must be bit-identical");
}
