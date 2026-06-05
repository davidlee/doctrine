//! SL-017 PHASE-01 T5 — cross-process determinism probe (OQ-5 / R7).
//!
//! Prints the BM25 score of each doc for a fixed corpus+query, as raw f32 BITS
//! (`to_bits`) so a byte-diff across two separate OS processes is an EXACT
//! bitwise-identity check. bm25 uses std `HashMap`/`HashSet` internally; doctrine
//! consumes only the per-doc score value (never the crate's ordering), so the
//! sole determinism exposure is whether that value drifts across runs.
//!
//! Source analysis (notes.md): `Scorer::score_` sums `idf * value` over the
//! query embedding's `Vec` (tokenizer order — deterministic); no float reduction
//! iterates a HashMap. So determinism is expected by construction; this is the
//! empirical confirmation. Run twice, diff stdout (see notes.md / phase sheet T5).
//!
//! Spawned directly (not via `CARGO_BIN_EXE`), so the stale-mount gotcha
//! (`mem.pattern.testing.stale-cargo-bin-exe`) does not apply here.
#![expect(
    clippy::print_stdout,
    reason = "probe example: stdout IS the determinism artifact under diff"
)]

use bm25::{EmbedderBuilder, Scorer, Tokenizer};

type Space = u32;

struct WhitespaceTokenizer;

impl Tokenizer for WhitespaceTokenizer {
    fn tokenize(&self, input_text: &str) -> Vec<String> {
        input_text.split_whitespace().map(str::to_string).collect()
    }
}

fn main() {
    // A non-trivial fixed corpus: varied lengths, repeated tokens, shared terms —
    // enough postings/IDF spread that any summation-order drift would surface.
    let corpus = [
        ("m1", "alpha beta gamma alpha"),
        ("m2", "beta gamma delta"),
        ("m3", "alpha epsilon"),
        ("m4", "gamma gamma gamma zeta"),
        ("m5", "delta epsilon zeta eta theta"),
    ];
    let query = "alpha gamma delta";

    let texts: Vec<&str> = corpus.iter().map(|(_, t)| *t).collect();
    let embedder =
        EmbedderBuilder::<Space, _>::with_tokenizer_and_fit_to_corpus(WhitespaceTokenizer, &texts)
            .build();
    let mut scorer = Scorer::<String, Space>::new();
    for (id, text) in &corpus {
        scorer.upsert(&(*id).to_string(), embedder.embed(text));
    }

    let mut rows: Vec<(String, u32)> = scorer
        .matches(&embedder.embed(query))
        .into_iter()
        .map(|m| (m.id, m.score.to_bits()))
        .collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0)); // our order; bm25's HashSet order discarded

    for (id, bits) in rows {
        println!("{id} {bits:#010x}");
    }
}
