# Notes SL-017: Pluggable lexical scorer: trait + BM25 backend for memory retrieval

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Forward-link: SL-017 ↔ SL-018 (ship a guidance-memory corpus)

SL-018 (shipping a corpus of guidance memories) is upstream **data**, not a
code-path change — it does **not** alter the SL-017 plan or its 4 phases. SL-017
treats the active set as "whatever `base_filter` admits," uniformly. Two couplings
to carry forward (neither a plan edit):

1. **Strengthens the BM25 premise.** D3 fits IDF/avgdl over all active memories;
   that signal was weak when the active set is 3–5 docs. A shipped corpus makes
   term-rarity genuinely meaningful — SL-018 is the corpus BM25 wanted.
2. **Bumps the `Indexed`-corpus follow-up from "later" to "soon."** SL-017 embeds
   *all active* per query (O(active); design §6 "corpus cost" + R5). A large
   shipped corpus makes that per-query full-embed cost real. The trait is already
   index-compatible (`LexicalCorpus` enum, D7) → the `Indexed(&LexicalIndex)`
   precompute path is **non-breaking**, so this stays a follow-up, not a re-plan.
   If SL-018 lands a big corpus, prioritise `Indexed` next.

**SL-018's job, not SL-017's** (flagged so SL-017 needs no special-casing):
- **Partition/trust scoping.** Guidance shipped as `workspace=default` + repo-empty
  is admitted in every repo (B20) → it enters both the BM25 fit corpus *and* the
  result set everywhere. SL-018 must set scope / trust_level / type so authored
  guidance does not drown real captured memories in ranking. SL-017 ranks them as
  ordinary active memories.
- **Altitude smell (User-raised):** authored guidance ≠ captured durable facts —
  whether it belongs in the memory store at all (vs `doc/*` or a distinct authored
  entity) is an SL-018 *design* question, out of SL-017 scope.

**Sequencing:** SL-017 does not depend on SL-018 — its BM25 quality VTs use crafted
corpora (rare-term, length-norm), not the shipped one. Either order works; SL-018
would give SL-017 a real-world validation surface.

## PHASE-01 — bm25 probe (the risk gate) — GO

Both gating unknowns (OQ-3 API surface, OQ-5 determinism) resolved against bm25
2.3.2 source + an empirical probe. No STOP condition tripped. Cleared to PHASE-02.
Evidence: `tests/bm25_probe.rs` (keeper) + `examples/bm25_probe.rs` (cross-process
determinism).

### Confirmed recipe (`--no-default-features`)

`Bm25Ranker` (PHASE-03) builds on:

- **Custom tokenizer.** `impl bm25::Tokenizer for T { fn tokenize(&self, &str) -> Vec<String> }`
  (OQ-3.1 — `Vec<String>` return, so the adapter over `lexical::tokenize` is a
  one-liner; the only adapter risk is ruled out).
- **Fit path.** `EmbedderBuilder::<D, T>::with_tokenizer_and_fit_to_corpus(tokenizer, &[&str]).build()`
  (OQ-3.2) — the **non-`Language` custom-tokenizer corpus-fit path**. `Language`/
  `LanguageMode` are gated behind `default_tokenizer` (absent, confirmed); this
  method does NOT route through them. `where T: Tokenizer + Sync` (a ZST tokenizer
  is `Sync`).
- **Embedding space `D`.** `DefaultTokenEmbedder`/`DefaultEmbeddingSpace` are **not
  exported** — name a concrete `TokenEmbedder` for `D` (`u32`, as the crate's own
  tests do). Irrelevant to the BM25 maths.
- **Scorer.** `Scorer::<String, D>::new()` — `K: Eq + Hash + Clone`, so a `String`
  uid key works (OQ-3.3). `upsert(&K, embedder.embed(text))` per active doc;
  `matches(&embedder.embed(query)) -> Vec<ScoredDocument<K>{id, score: f32}>`
  (only query-touched docs, sorted desc). **Consume id→score, discard the order**
  (HashSet-derived, not cross-process stable; `sort_key` re-imposes totality).

### A3 avgdl/doc_len equivalence — STRUCTURAL (stronger than design assumed)

Design §5.4 step 2 planned to **self-compute** `avgdl` + pass via `with_avgdl`,
then prove it equals the scorer's internal `doc_len` (A3 risk). Source shows a
cleaner path:

- `Embedder::embed` (embedder.rs:144,161): per-doc length = `tokenizer.tokenize(text).len()`
  (**multiset**).
- `with_tokenizer_and_fit_to_corpus` (embedder.rs:226-231): `avgdl =
  mean(tokenizer.tokenize(doc).len())` over the corpus, **same tokenizer instance**.

avgdl and doc_len are the same `tokenize().len()` call path ⇒ **A3 divergence is
impossible by construction**. Probe confirms: fitted `avgdl` equals an independent
multiset mean (incl. a repeated-token doc `"a a a"`=3), and a deduping tokenizer
diverges (the differential detects divergence).

**Design-feedback for PHASE-03:** use `with_tokenizer_and_fit_to_corpus(custom,
corpus_texts)`, NOT `with_avgdl` + hand-computed mean — it eliminates the A3 risk
rather than merely testing it. Reconcile design §5.4 wording at PHASE-03 entry to
name this path. The PHASE-03 EX-3 STOP→/consult (Finding B) stays as a backstop,
now expected unreachable.

### OQ-5 / R7 determinism — resolved at rung 2; LEX_SCALE stays 1e6

- **Source.** `Scorer::score_` sums `idf * value` over the query embedding's `Vec`
  (tokenizer order — deterministic); `idf` is HashMap *lookups* (value-stable).
  No float reduction iterates a HashMap. The only HashMap-*iteration* is
  `matches()`'s candidate gather → output ordering, which we discard. Deterministic
  by construction.
- **Empirical.** `examples/bm25_probe.rs` prints each score as raw f32 bits
  (`to_bits`); 10 separate processes → 1 distinct output (byte-identical).
  Same-process repeat-call also identical.
- **Decision.** `LEX_SCALE = 1e6` (D4) holds; no coarsening. R7 ladder settled at
  rung 2.

### Gotchas

- bm25 is reached only from test/example — outside the bins/lib `just check` gate;
  `unused_crate_dependencies` paused, so no failure, but `just check` alone does
  not compile the bm25 usage (`cargo test`/`--example` does). Design §9 records it.
- `default-features = false` is permanent — verified no stemmer/deunicode/stopword
  deps entered `Cargo.lock`.
- Determinism probe spawned directly (not `CARGO_BIN_EXE`), sidestepping the
  stale-mount gotcha (`mem.pattern.testing.stale-cargo-bin-exe`); it still bites
  the `tests/e2e_*` spawn suites — `touch tests/*.rs` before a cold `cargo test`.
