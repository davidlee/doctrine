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

## PHASE-02 — pure lexical leaf (tokenize, trait, OverlapRanker, quantize)

`src/lexical.rs` landed: `tokenize` MOVED from retrieve (retrieve now
`use crate::lexical::tokenize;`), `LexDoc{id,text}`, 1-variant `LexicalCorpus::Raw`,
the `LexicalRanker` trait (A1-total contract), `OverlapRanker`, `quantize`,
`LEX_SCALE`. Module-level `#![cfg_attr(not(test), expect(dead_code, …))]` bridge
(mirrors memory.rs); self-clears at PHASE-04. `lexical_score` retained unchanged
(retires at PHASE-04). All `pub` → `pub(crate)`: repo denies `unreachable_pub`
(bin crate) — design §5.2's `pub` was illustrative.

### Decision A — `quantize` non-finite is profile-split (consult, approved)

Design §5.2 mandated `debug_assert!(is_finite)` AND §9 mandated a
`non_finite → 0` test; both cannot pass in one debug run (`debug_assert!` is live
under `cargo test`). Resolved as a **design clarification** (not deviation):
- debug build: non-finite is a scorer bug → the assert panics (A8).
- release build: `quantize` stays total → non-finite returns 0.
Tests are profile-split: `quantize_non_finite_panics_in_debug`
(`#[cfg(debug_assertions)]` `should_panic`), `quantize_non_finite_is_zero_in_release`
(`#[cfg(not(debug_assertions))]`), plus `quantize_negative_is_zero` (all builds,
the `max(0.0)` guard). Design §5.2/§9 patched.
**Gate implication:** normal `cargo test` proves bug-surfacing; a **release-profile
test pass** is required for direct proof of the non-finite→0 fallback (carry to
/audit). `cargo test --release --bin doctrine quantize_non_finite` is green.

### Decision B — `quantize` uses one guarded saturating `as`

Repo blanket-denies `as_conversions` + `cast_sign_loss` (+ truncation/precision)
— there is **no `as` anywhere else in `src`** (memory.rs:125). No safe std
float→int API exists. Rust's float→int `as` is *saturating* (≥1.45): out-of-range
→ `u32::MAX`, and `scaled` is finite & ≥0 by construction, so a single
`scaled as u32` is total + monotonic + saturating — the design's explicit
ceiling branch is unnecessary. Carries the house-style stacked
`#[expect(as_conversions, cast_possible_truncation, cast_sign_loss, reason=…)]`
(expect+reason, never bare allow). Behaviour identical to design §5.2's sketch.

### Parity proof (VT-2)

`OverlapRanker` parity vs `lexical_score` lives in **retrieve**'s test module
(`overlap_ranker_matches_lexical_score`) — it needs both `Memory`/`lexical_score`
and the leaf, which the leaf may not import (ADR-001). Concat-vs-segment
equivalence held across separator-laden fixtures (`mem.x.y`, `src/x.rs`) and the
empty query. `just check` green: clippy zero-warning, full suite + e2e pass,
`cargo fmt --check` clean.

## PHASE-03 — Bm25Ranker: corpus-relative BM25 behind the trait — DONE

`Bm25Ranker` implemented in `src/lexical.rs` on the PHASE-01 recipe + PHASE-02
trait, still a pure leaf. 9 new tests green, `just check` exit 0 (bm25 now in the
bin/lib build — the PHASE-01 "bm25 test-only" caveat is resolved; clippy compiles
bm25 usage). NOT yet wired into retrieve (PHASE-04).

### Recipe as shipped (matches `tests/bm25_probe.rs` verbatim)

`type Space = u32` (DefaultTokenEmbedder unexported); `struct LexTokenizer`
(ZST) `impl bm25::Tokenizer` delegates to the module free `tokenize` — bare
`tokenize(s)` resolves to the free fn, not the method (no receiver ⇒ no
recursion), so ONE lexer (D2). `score`: `assert_targets_subset` first; guard
`Some(q) if !tokenize(q).is_empty() && !docs.is_empty()` else `zeros(targets)`
(None / empty-after-tokenize / empty corpus — the §5.4 step-1 avgdl div-by-zero
guard); else `with_tokenizer_and_fit_to_corpus(LexTokenizer, &texts)`, upsert
every doc into `Scorer::<String,Space>`, `matches(embed(q))` → `BTreeMap<&str,
f32>`, map targets POSITIONALLY to `quantize(scored.get(t).copied()
.unwrap_or(0.0))`. bm25's descending order discarded.

### A3 STRUCTURAL — confirmed on the real lexer (VT-5, EX-3)

`with_tokenizer_and_fit_to_corpus` computes avgdl with the SAME tokenizer the
embedder uses per-doc, so the avgdl/doc_len equivalence is structural, not
hand-maintained — no `with_avgdl`, no self-computed mean (plan.toml's "manual
avgdl" prose is superseded, §5.4 reconciled at fd172fb; criterion intent met more
strongly, no id change). `bm25_avgdl_equals_multiset_mean_on_real_tokenizer`
carries PHASE-01 EX-3 to the production `lexical::tokenize`: `embedder.avgdl()`
== `mean(tokenize(t).len())`. EX-3's STOP→/consult backstop stayed unreached.

### VT evidence

VT-1 IDF (`bm25_idf_rare_outranks_common`): "rare" df=1 target > "common" df=3
target at equal length. VT-2 length-norm (`..shorter_outranks_longer`): equal TF,
short(len1) > long(len5), avgdl=3. VT-3 determinism
(`..shuffle_invariant_and_repeatable`): permuted corpus ⇒ identical per-target
scores (positional over targets, corpus order cannot leak) + repeat-call
identical — OQ-5/R7 by construction, no LEX_SCALE coarsening needed. VT-4 edges
(`..all_zero_with_exact_arity`, `..empty_corpus_returns_empty_vec`,
`..survivor_untouched..`, `..df_reflects_full_corpus_not_targets`): None/""/seps
/empty ⇒ all-zero exact arity; untouched survivor ⇒ 0; df over FULL corpus (D3).

### Notes for PHASE-04

- `zeros(targets)` is a private module helper (one `(id,0)` per target) — reused
  by the wiring if a no-evidence path needs it; assembly must stay positional, no
  `unwrap_or` (A1).
- The §9 staged `dead_code` bridge still covers `Bm25Ranker`/`LexTokenizer`/
  `zeros`/`Space` — its self-clearing condition is the PHASE-04 wiring (remove the
  `#![cfg_attr(not(test), expect(dead_code…))]` then; a now-unfulfilled
  expectation would warn).
- **PHASE-04 seam decision (planned, flag for audit): `OverlapRanker` →
  `#[cfg(test)]`.** Post-wiring it has NO production caller — BM25 is the hard
  default, no selector (D5). Removing the module `dead_code` bridge (EX-5) would
  otherwise leave `OverlapRanker` dead in the bins/lib build. Gating its struct +
  impl behind `cfg(test)` makes it a pure behaviour-preservation test instrument
  (the SL-008 overlap tests re-point through it) and lets the bridge vanish
  cleanly. Faithful to §5.4 "retiring `lexical_score`" — it retires INTO the test
  harness; a future selector would un-gate in its own slice (YAGNI). No external
  behaviour change. Captured here so it survives the gitignored phase sheet.

## PHASE-04 outcome — BM25 wired as the hard default (commit 984165f + this)

`query()` restructured (design §5.1/§5.3): build `active` = `base_filter`
survivors (the BM25 fit corpus, honours `include_draft`), project once through
the new production `lex_doc` (§5.3 bag: `title summary tags key`, space-joined),
narrow to `survivors` (scope + thread-expiry, carrying the Copy `ScopeMatch`),
score via an injected `&dyn LexicalRanker`, and assign `Candidate.lexical`
POSITIONALLY from `scores[i].1` (A1 — no `unwrap_or`). The impure shell
(`run_find`/`run_retrieve`) constructs the default `Bm25Ranker` and passes `&dyn`
in — the purity boundary: `query` never picks the ranker (D5, no selector on
either surface).

`survivors ⊆ active = corpus` holds by construction, so the ranker's hard
`targets ⊆ corpus` assert cannot fire from `query`.

Retirement landed: `lexical_score` + its 3 unit tests DELETED; `tokenize` import
dropped from `retrieve`. `OverlapRanker` struct+impl now `#[cfg(test)]`; the
module `#![cfg_attr(not(test), expect(dead_code…))]` bridge REMOVED and clippy
stays zero-warning — the §9 self-clear realised (EX-5). The seam decision
(OverlapRanker → cfg(test)) is flagged above for audit harvest.

### Behaviour-preservation (EX-4)
- Parity test re-based: `overlap_ranker_preserves_retired_overlap` scores through
  the production `lex_doc` and asserts FROZEN overlap counts lifted verbatim from
  the deleted `lexical_score` unit tests (fixture-2 "src memory rs lint clippy" ⇒
  5, etc). `lexical_score` is gone, so parity-vs-fn became parity-vs-frozen-vector.
- `query_bare_query_keeps_all_active_ranked_lexically` re-pointed through
  `&OverlapRanker`, assertions UNCHANGED — the witness the seam did not alter
  overlap ordering.

### VT evidence (PHASE-04)
- VT-1 `query_exact_key_dominates_higher_bm25`: exact-key hit with LOWER bm25
  still ranks first (Key-1 over Key-2 magnitude), asserted on the live scores.
- VT-2 `query_is_shuffle_invariant_under_bm25`: permuted store ⇒ identical
  `(uid, lexical)` sequence.
- VT-3 `find_bm25_ranking_is_cross_process_deterministic` (e2e): two separate
  `doctrine memory find --query …` processes over one seeded store ⇒ byte-
  identical stdout. R7 coarsen rung stayed unreached.
- VT-4 `query_bm25_and_overlap_order_oppositely`: same query, `&Bm25Ranker` puts
  the rare-term match first while `&OverlapRanker` puts the higher raw-overlap
  match first — the intended quality change, made visible.
- VT-5 `query_bm25_score_is_derived_not_persisted_on_memory`: `Memory` Debug
  carries no `lexical` field; a nonzero-scored memory's representation is
  unchanged by scoring (immutable borrow). No persisted float, by construction (R3).
