# Design SL-017: Pluggable lexical scorer: trait + BM25 backend for memory retrieval

## 1. Design Problem

Memory retrieval (SL-008) ranks filter survivors with a fixed 9-key `Ord` tuple.
Key 2 — the **lexical axis** — is today a single concrete function,
`lexical_score` (`src/retrieve.rs`), returning a bounded `u32` count of *distinct
query tokens that hit* a memory's `title + summary + tags + memory_key` bag
(set-membership, body excluded — SL-008 Q1/B15). It is a per-document signal with
no term weighting, no length normalisation, and no awareness of how common a term
is across the searchable set: a rare, discriminating term counts the same as a
ubiquitous one.

The driver for this slice is **ranking quality** (not merely the abstraction).
The design problem is three disciplines:

1. **Lift the lexical axis behind a trait** so the scoring strategy is pluggable —
   without duplicating the ranking machinery (no parallel implementation; ride the
   existing `Candidate`/`sort_key` seam) and without disturbing the other 8 keys,
   `exact_key_match`, staleness, filters, or the trust holdback.
2. **Add a corpus-relative BM25 backend** (the `bm25` crate) as the hard default,
   reconciling two structural mismatches: BM25 is corpus-relative (needs IDF/avgdl
   over a fit corpus) where the old signal was per-document; and BM25 emits `f32`
   where Key 2 is an `Ord` `u32`.
3. **Preserve determinism and the storage model.** Same query + store + clock +
   git ⇒ identical order (shuffle-invariance holds); lexical scores stay
   **derived, never stored** — no `f32` reaches the canonical payload, export, or
   event-store backend ([memory-spec](../../../doc/memory-spec.md) §584-585 float
   ban targets the payload/backend, not in-process scoring).

## 2. Current State

- **`src/retrieve.rs`** owns the lexical axis:
  - `tokenize(&str) -> Vec<String>` — case-fold + split on non-alphanumeric,
    drop empties (the shared lexer; splits `mem.foo.bar` / `src/x.rs` on their
    separators).
  - `lexical_score(m, q) -> u32` — distinct-query-token set-membership over
    `title + summary + tags + key`. No query ⇒ 0.
  - `exact_key_match(m, q) -> bool` — FULL `memory_key` equality (separate axis,
    dominates within Key 2; segment overlap is `lexical_score`'s job).
  - `Candidate<'a>` carries the per-query derived signals (`lexical: u32`,
    `exact_key: bool`, `staleness`, `scope_match`); `Candidate::new` computes
    `lexical_score` inline, per-document.
  - `SortKey` = 9-tuple; Key 2 is `Reverse<u32>` (lexical, descending). Polarity
    is load-bearing and asserted per-key in tests.
  - `query()` runs the filter cascade `base_filter → match_scope → thread_expiry`
    over `&[Memory]` the shell loaded, builds `Candidate`s, and `rank()`s. The
    pure layer receives git/clock pre-resolved as data (`Snapshot`/`GitFacts`).
- **No trait abstraction** over scoring — `sort_key`/`lexical_score` are concrete.
- **Dependencies** carry no lexical/IR crate.

## 3. Forces & Constraints

- **ADR-001 module layering** (leaf ← engine ← command, no cycles). The new
  scoring module must be a pure leaf, depending on neither `retrieve` nor
  `memory`.
- **Pure/impure split** (slices-spec §Architecture): no clock/rng/git/disk in the
  pure layer. Scoring is pure (deterministic text→score); the impure shell
  constructs the ranker and loads memories.
- **Float ban** ([memory-spec](../../../doc/memory-spec.md) §86, §584-585): no
  `f32`/`f64` in the canonical payload, export, or event-store backend. Lexical
  scores are derived/never-stored, so in-process `f32` is legal **provided** it
  never crosses to a persisted field and Key 2 stays integer/`Ord`.
- **Behaviour-preservation gate** (project): changing shared ranking machinery
  must keep the existing SL-008 suites green — the seam extraction of the
  token-overlap path must be provably behaviour-identical.
- **Determinism / shuffle-invariance** (SL-008 property): a shuffled input yields
  identical output. BM25 IDF/avgdl depend on the *set*, not insertion order — the
  property survives.
- **Jail** (CLAUDE.md): bubblewrap, read-only `~/.cargo`. The `bm25` crate must be
  fetchable/buildable; under `--no-default-features` core scoring APIs must remain
  available, else **stop and `/consult`** rather than broadening dependencies.

## 4. Guiding Principles

- **Change one variable.** Replace the *scoring model* (overlap → BM25) without
  simultaneously changing tokenization semantics. Reuse doctrine's `tokenize()`;
  treat stemming / stopwords / a technical tokenizer as a *later, measured*
  experiment, not an SL-017 default.
- **Quality over respectability.** BM25 IDF must describe the *searchable corpus*
  (all active memories), not the already-filtered survivor subset — fitting over
  3–5 survivors makes IDF query-local noise.
- **Minimal scope.** No ranking-policy/config surface, no CLI/env switch, no
  persistent index/cache. The trait is the seam; BM25 is the default; the overlap
  path survives only behind the trait for parity/measurement.
- **Memory-agnostic leaf.** The lexical layer names no memory-layer concept; the
  engine adapts `Memory` into the leaf's `LexDoc`.

## 5. Proposed Design

### 5.1 System Model

A new pure leaf module `src/lexical.rs` owns tokenization, the scorer trait, its
two implementations, and the `f32→u32` quantizer. `retrieve` adapts `Memory` into
the leaf's `LexDoc`, constructs the corpus, and consumes scores into `Candidate`.

```
  src/lexical.rs  (pure leaf — imports neither retrieve nor memory)
    tokenize()                      the shared lexer (moved here)
    LexDoc { id, text }             memory-agnostic doc
    LexicalCorpus::Raw(&[LexDoc])   the fit-corpus view (Indexed(..) later)
    LexicalRanker (trait)           score(query, corpus, targets) -> Vec<(id,u32)>
      OverlapRanker                 today's set-membership count (no quantize)
      Bm25Ranker                    bm25 crate, Tokenizer=lexical::tokenize, quantized
    quantize(f32) -> u32            monotonic non-decreasing, saturating, total

  src/retrieve.rs  (engine/command — depends on lexical)
    builds LexDoc{ id: uid, text: title+summary+tags+key } from each Memory
    query(.., ranker: &dyn LexicalRanker) — active=fit corpus, survivors=targets
    Candidate.lexical = scores.get(uid).copied().unwrap_or(0)
```

**Data flow in `query()`** (the single chained filter splits into two sets):

```
active    = mems.filter(base_filter)                    // fit corpus = partition-scoped active set (base_filter applies snap.part + include_draft)
survivors = active.filter(scope_match? + thread_expiry) // score targets + Candidates
corpus    = LexicalCorpus::Raw(&active_lexdocs)
targets   = survivor uids
scores    = ranker.score(q.query, &corpus, &targets)    // one (id,u32) per target, in order
Candidate.lexical = scores[i].1   // positional — no unwrap_or; absent entry is a bug (A1)
```

**Purity boundary (explicit):** the impure shell (`run_find`/`run_retrieve`)
*chooses and constructs* the ranker (`Bm25Ranker` default) and passes `&dyn
LexicalRanker` into the pure `query()`. The default choice never lives inside
`query()`; the pure layer stays ranker-agnostic.

### 5.2 Interfaces & Contracts

```rust
// src/lexical.rs — pure leaf

/// Case-fold + split on non-alphanumeric, dropping empties. General-purpose
/// lexer; retrieval imports it, this leaf imports nothing of retrieval.
pub fn tokenize(s: &str) -> Vec<String>;

/// A memory-agnostic scoring document. `id` is the memory uid (caller-canonical,
/// assumed unique). `text` is the pre-concatenated doc bag.
pub struct LexDoc { pub id: String, pub text: String }

/// The fit-corpus view. One variant in SL-017; `Indexed(&LexicalIndex)` is the
/// non-breaking evolution when an active-set index is precomputed (follow-up).
pub enum LexicalCorpus<'a> { Raw(&'a [LexDoc]) }

pub trait LexicalRanker {
    /// Fit corpus-level statistics over `corpus`, score `targets`, and return one
    /// `(id, u32)` entry **per target, in `targets` order** (Key-2 magnitude).
    ///
    /// Contract (A1 — completeness is total, not best-effort):
    /// - The result has **exactly `targets.len()` entries, one per target, in
    ///   order.** A no-evidence target (query None / empty after tokenize / no term
    ///   matched) is filled with `0` *inside the ranker*. Candidate assembly indexes
    ///   positionally — it never uses `unwrap_or(0)`; an absent entry is a bug.
    /// - **Hard precondition (all builds):** `assert!` every target id is present in
    ///   `corpus` (targets ⊆ corpus). A target outside the fit corpus is an
    ///   internal invariant violation (survivors ⊆ active = corpus) — fail loud,
    ///   never silently demote to 0. Cost is one membership pass per query.
    /// - Duplicate ids in `corpus` violate uniqueness (`debug_assert!`; release is
    ///   last-wins, undefined-but-safe).
    fn score(
        &self,
        query: Option<&str>,
        corpus: &LexicalCorpus<'_>,
        targets: &[&str],
    ) -> Vec<(String, u32)>;
}

pub struct OverlapRanker;   // distinct-query-token set-membership; returns u32 directly (no quantize)
pub struct Bm25Ranker;      // bm25 --no-default-features; Tokenizer = lexical::tokenize

pub const LEX_SCALE: f32 = 1_000_000.0;
/// BM25 f32 (>= 0 under Lucene IDF) -> Key-2 u32. Monotonic non-decreasing,
/// saturating, total (non-finite -> 0: invalid evidence, never maximal).
pub fn quantize(score: f32) -> u32 {
    debug_assert!(score.is_finite(), "non-finite lexical score: scorer bug");  // A8: surface upstream bugs
    if !score.is_finite() { return 0; }
    let scaled = (score.max(0.0) * LEX_SCALE).round();
    if scaled >= u32::MAX as f32 { u32::MAX } else { scaled as u32 }
}
```

`SortKey` / `sort_key` / `rank` / `exact_key_match` are **unchanged**: Key 2 stays
`Reverse<u32>`, fed from `Candidate.lexical`. `exact_key` remains a separate Key-1
component dominating Key 2.

### 5.3 Data, State & Ownership

- **`LexDoc.text`** = `title + " " + summary + " " + tags.join(" ") + " " + key`
  (body excluded — preserves SL-008 Q1/B15). Built by `retrieve` (the only layer
  that knows `Memory`'s shape); `lexical` never sees a `Memory`.
- **No new persisted state.** Lexical scores exist only in the transient
  `Candidate` (never serialized). SL-017 adds **zero** fields to `Memory`, the
  payload, export, or any on-disk artifact. No index/cache is persisted.
- **`Bm25Ranker` internals** (per call, owned, dropped): an `Embedder` fit to the
  active corpus (`avgdl`), a `Scorer` with all active `LexDoc`s upserted (`df`/IDF
  over the searchable set), then `matches(query)` filtered to `targets`.

### 5.4 Lifecycle, Operations & Dynamics

`Bm25Ranker::score`:
1. `query` is `None`, or tokenizes to empty, or `corpus` is empty ⇒ return
   `targets.iter().map(|id| (id.to_string(), 0)).collect()` **without** invoking
   bm25 (avoids `avgdl` div-by-zero on an empty fit).
2. Build the `Embedder` with the **custom tokenizer** (`lexical::tokenize`) and a
   **self-computed `avgdl`** = mean `tokenize(doc.text).len()` over the active
   corpus. NB the README's `EmbedderBuilder::with_fit_to_corpus(Language, &corpus)`
   path is **unavailable** here: `Language`/`LanguageMode` are gated behind the
   `default_tokenizer` feature, which `--no-default-features` removes. We supply
   `avgdl` directly (`with_avgdl` or equivalent — exact method pinned by the
   PHASE-01 probe, OQ-3).
3. Upsert every active `LexDoc` into a `Scorer` keyed by uid; df/IDF describe the
   searchable set.
4. `matches(embed(query))` → `(uid, f32)` for docs a query term touched. **We use
   `matches()` purely as a score map — bm25's own descending-by-score ordering is
   discarded** (its tie-order derives from `HashSet` iteration and is not
   cross-process stable; doctrine's `sort_key` re-imposes the total order).
5. Build the result **positionally over `targets`**: for each target, `quantize`
   its `matches()` f32 if present, else `0` (no-evidence). Returns exactly one
   `(uid, u32)` per target, in order (A1). The `assert!(targets ⊆ corpus)`
   precondition runs first.

**avgdl/doc-len equivalence (A3 — load-bearing for length normalisation):** the
self-computed `avgdl` denominator MUST equal the token count the crate's scorer
attributes to each document — i.e. `avgdl = mean(custom_tokenizer.tokenize(text).len())`
over the active corpus, using the *same* `Tokenizer` instance the `Embedder`/`Scorer`
use. If `avgdl`'s token-stream semantics diverge from the scorer's internal
`doc_len` (borrowed vs owned, normalised vs raw, deduped vs multiset), length
normalisation is silently wrong while design-level tests still pass. PHASE-01 pins
this by a differential check (see §9).

`OverlapRanker::score`: per-target distinct-query-token set-membership over its
`LexDoc.text` (re-tokenized), returning the `u32` count directly — **no quantize**,
byte-identical to the retired `lexical_score`. Corpus is ignored (per-document).
*Equivalence note:* `lexical_score` tokenized title/summary/tags/key separately
then unioned into a `BTreeSet`; `LexDoc.text` is those segments space-joined, and
`tokenize` splits on the joins, so the resulting token *set* — and the distinct-hit
count — is identical. The parity test pins this.

### 5.5 Invariants, Assumptions & Edge Cases

- **Determinism / shuffle-invariance.** BM25 `f32` is fixed by `{df, avgdl, doc
  len, query}`; none depend on upsert order ⇒ identical scores under a shuffled
  corpus ⇒ identical quantized Key 2 ⇒ identical rank (the `uid` tiebreak already
  guarantees totality). Property test retained, now exercising `Bm25Ranker`.
- **Quantization.** Monotonic *non-decreasing* (rounding may collapse adjacent
  f32 into one bucket — acceptable: ties fall through to deterministic keys 3–9).
  Saturating; total over all f32 including non-finite.
- **Non-negativity.** bm25 uses Lucene IDF `(1 + num/den).ln() >= 0` with
  non-negative TF weights ⇒ scores never negative. `max(0.0)` is defensive only.
- **No query ⇒ 0** preserved (contract parity with `lexical_score`).
- **`exact_key_match` unchanged** — still FULL key equality, still dominates Key 2.
- **Targets ⊆ corpus** — **hard `assert!`, all builds** (A1: fail loud, never
  silently demote a mis-sliced target to 0); **unique corpus ids** (`debug_assert`).
- **Per-target completeness** — `score` returns exactly one entry per target, in
  order; assembly is positional, never `unwrap_or` (A1).
- **`Memory` serialization unchanged** (asserted) — no float-bearing payload.

## 6. Open Questions & Unknowns

- **OQ-3 (build + API surface) — partially resolved, gated on PHASE-01 probe.**
  `bm25 = 2.3.2` (MIT) fetches over the live network. **Confirmed feature-gated:**
  `Language`/`LanguageMode` (and thus the `with_fit_to_corpus(Language, …)` fit
  path) live behind `default_tokenizer` — removed by `--no-default-features`. The
  probe must pin, under `--no-default-features`:
  1. the `Tokenizer` trait signature (method + return type) — the adapter shape;
  2. the exact builder method to set a custom tokenizer **and** a precomputed
     `avgdl` (e.g. `with_tokenizer` + `with_avgdl`) — the corpus-fit path that does
     **not** route through `Language`;
  3. `Scorer`'s generic bound (must admit a `String`/uid key) and `matches()`'s
     return type.
  4. the scorer's internal per-document token-count semantics, to confirm
     `avgdl` equivalence (A3) — see §9.
  If the core scoring path (custom tokenizer + manual `avgdl` + `Scorer`) is not
  reachable without `default`, **stop and `/consult`** — never silently enable the
  default tokenizer deps.
- **OQ-5 (determinism) — gated on PHASE-01 probe.** bm25's inverted index and
  `matches()` use std `HashMap`/`HashSet` (per-process randomized order). doctrine
  discards bm25's ordering, so the only exposure is whether a per-document score
  *value* (`Σ idf·doc_value` over query terms) is **bitwise-identical across
  process runs**. Summation likely follows deterministic query-token order, but
  this is unverified; a 1-ULP drift at 1e6 scale could flip a quantize bucket.
  Resolve empirically (R7).
- **Corpus cost (deferred).** Embedding all active memories per query is O(active).
  Acceptable at current store size; a precomputed `Indexed` corpus is the
  non-breaking follow-up if the active set grows large.
- **Tokenizer recall (deferred, by principle).** Plain `tokenize()` has no
  stemming/stopwords; `mem.pattern.lint` already splits to `mem`/`pattern`/`lint`
  but the compound is not retained. *Future* technical-token expansion (emit
  compound **and** fragments) only if measured misses prove the need — **out of
  SL-017 scope.**

## 7. Decisions, Rationale & Alternatives

- **D1 — Driver is ranking quality** (not just the abstraction). BM25 is the hard
  default. *Alternative rejected:* keep overlap default, BM25 opt-in — fails the
  driver.
- **D2 — Reuse doctrine `tokenize()` via a custom bm25 `Tokenizer`,
  `--no-default-features`.** Zero new tokenizer deps; one tokenization regime
  across scope-match and lexical; safe on code/identifiers. *Alternative
  rejected:* bm25 `DefaultTokenizer` — English stemming/stopwords/deunicode mangle
  identifiers (`lint`, `src`, `rs`), add 5 deps, introduce a second regime, and
  change two variables at once.
- **D3 — Fit IDF/avgdl over all active memories; score only survivors.** BM25
  statistics describe the searchable universe, not the eligibility subset; bare
  `--query` uses active as both corpus and targets (consistent with SL-008 D20).
  *Alternative rejected:* fit over survivors — IDF degenerate on tiny sets, scores
  shift with unrelated scope/filter decisions.
- **D4 — Quantize `f32→u32` per-score (scale 1e6, saturating, non-finite→0).**
  Keeps Key 2 a true `Ord` integer score; order-preserving (non-decreasing).
  *Alternatives rejected:* rank-position encoding (collapses magnitude, makes a
  doc's key depend on the whole target set); an ordered-float Key 2 (breaks the
  integer-Key-2 contract for no gain).
- **D5 — No runtime/CLI/env/config switch in SL-017.** BM25 hard default;
  `OverlapRanker` retained **only** behind the trait for unit parity, fallback,
  and a future measurement harness. *Alternative rejected:* a selector flag —
  expands the slice into ranking-policy/config design.
- **D6 — `LexDoc` (memory-agnostic name) in the leaf;** `retrieve` adapts
  `Memory`. Keeps the layering claim honest (no `Memory` concept in the leaf type
  name). *Alternative rejected:* `MemoryLexDoc` — leaks the memory layer.
- **D7 — One-variant `LexicalCorpus::Raw` enum now.** Small ceremony; makes the
  `Indexed(&LexicalIndex)` follow-up non-breaking. *Alternative rejected:* a bare
  slice — would force a signature change later. **A4 future-seam constraint:** a
  BM25 index is not meaningful to `OverlapRanker` (and vice versa), so when
  `Indexed` lands, an unsupported ranker/corpus pairing MUST fail at construction
  time (or be adapted before `score`) — `score` must never silently fall back to
  incorrect semantics. No SL-017 code impact; recorded so the seam does not pretend
  all indexes are universal.

## 8. Risks & Mitigations

- **R1 — bm25 core APIs feature-gated under `default`.** *Mitigation:* PHASE-01
  build probe; if gated, `/consult` — never silently enable `default_tokenizer`.
- **R2 — Ranking regression vs the old overlap signal.** *Mitigation:* the
  behaviour-preservation gate proves the overlap path unchanged (`OverlapRanker`
  parity); new BM25 tests pin the *intended* quality changes (rare-term/length
  effects); the default flip is asserted only by re-baselined tests, never assumed.
- **R3 — `f32` leaks to a persisted field.** *Mitigation:* score lives only in the
  transient `Candidate`; `Memory`-serialization-unchanged test; no new payload
  field. Structurally impossible by construction.
- **R4 — Quantization collisions reorder results unexpectedly.** *Mitigation:*
  1e6 scale + non-decreasing guarantee; collisions resolve via deterministic keys
  3–9; monotonicity unit-tested.
- **R5 — Corpus cost on a large active set.** *Mitigation:* accepted for SL-017
  (current scale); `Indexed` follow-up reserved by D7.
- **R6 — Two tokenization regimes drift.** *Mitigation:* avoided — bm25 `Tokenizer`
  delegates to the same `lexical::tokenize`; one lexer.
- **R7 — Cross-process non-determinism from bm25's internal `HashMap`.** A
  per-document score differing by 1 ULP across runs could flip a quantize bucket
  and reorder results, breaking SL-008's determinism contract. *Mitigation —
  fallback ladder (A2), determinism wins over resolution:*
  1. Never consume bm25's ordering — only the per-doc `f32`, re-sorted by
     `sort_key` (always in force).
  2. PHASE-01 empirically asserts score-value stability across two separate process
     runs (OQ-5) + a same-process repeat-call test.
  3. If values vary: prefer a *deterministic-summation* fix (sorted query tokens /
     stable postings iteration) if the crate exposes enough internals.
  4. Else coarsen `LEX_SCALE` — **allowed only if the cross-process VT passes
     *after* coarsening** (stress-run the corpus/query for byte-identical output).
     Coarsening reduces risk; it does not *prove* no boundary crossing by
     construction.
  5. Still unstable ⇒ **stop and `/consult`; do not ship BM25 as default.**

## 9. Quality Engineering & Validation

**Behaviour-preservation (gate):** existing SL-008 overlap-ordering tests
re-point through `OverlapRanker` explicitly and stay **green unchanged** — proof
the seam extraction did not alter overlap behaviour. The BM25 default path is
covered by *new* tests + re-baselined integration/e2e.

**`lexical.rs` units:**
- `quantize`: `quantize_zero_is_zero`, `quantize_is_monotonic_non_decreasing`,
  `quantize_saturates` (`f32::MAX → u32::MAX`), `quantize_non_finite_is_zero`
  (`NaN`, `INFINITY` → 0).
- `tokenize`: moved cases (separator splitting, case-fold, empty drop).
- `OverlapRanker`: parity vs retired `lexical_score` (set-membership over
  title+summary+tags+key; no-query ⇒ 0).
- `Bm25Ranker`: rare-term-outranks-common (IDF effect); shorter-doc-outranks-longer
  at equal TF (length norm, `b`); shuffle-invariance (permuted upsert ⇒ identical
  scores); **same-process repeat-call** (two `score` calls on identical inputs ⇒
  identical `u32`s — guards summation-order noise, OQ-5/R7); empty query /
  empty-after-tokenize / empty corpus ⇒ all-zero; non-matching survivor ⇒ 0; IDF
  drawn from full corpus not just targets (a target's score reflects df over
  active, not over the target subset); **per-target completeness** (result length ==
  `targets.len()`, positional, no-evidence target == 0 — A1).
- **avgdl equivalence (A3, PHASE-01 differential):** assert the self-computed
  `avgdl` denominator equals the crate scorer's internal per-doc token count. Pin
  by a differential run against the crate's *default-features* fit path on an
  equivalent simple tokenizer + corpus (or by source inspection if the internal
  `doc_len` is reachable). Proves length-normalisation correctness, not just "our
  arithmetic is self-consistent."
- **Query-edge cases (A7):** `None`, `Some("")`, and `Some("…only separators…")`
  all ⇒ all-zero, exactly one entry per target.
- Contract invariants: `targets ⊄ corpus` trips the **hard `assert!`** (all builds,
  A1); duplicate corpus ids trip the `debug_assert!`.

**`retrieve` integration:**
- `exact_key` still dominates Key 2 under BM25 (an exact-key hit outranks a
  higher-BM25 non-key hit).
- shuffle-invariance property test green with `Bm25Ranker` wired.
- one crafted case where BM25 reorders vs overlap (the intended quality change).
- `Memory` serialization unchanged (no float payload / new field).

**e2e/VT:** `doctrine memory find --query …` default path uses BM25. **Cross-process
determinism VT:** run the same `find --query` invocation twice in *separate
processes* against a fixed store and assert byte-identical output — the empirical
guard for OQ-5/R7 (no reliance on bm25 internals). A failure here triggers the R7
coarsen-scale fallback before close.

**Lint:** guarded `as` casts in `quantize` carry a narrow
`#[expect(clippy::cast_possible_truncation, reason = "…")]` (and
`cast_precision_loss` for `u32::MAX as f32`) — `expect` + `reason`, never bare
`allow` (`mem.pattern.lint.expect-not-allow`). `just check` / `cargo clippy` zero
warnings.

## 10. Review Notes

### Adversarial self-review (round 1)

- **A1 (critical) — wrong corpus-fit API under `--no-default-features`.** The first
  draft fit via `EmbedderBuilder::with_fit_to_corpus(Language::English, …)`, but
  `Language`/`LanguageMode` are gated behind `default_tokenizer` (confirmed in the
  crate `lib.rs`), which `--no-default-features` removes. *Disposition: fixed* —
  §5.4 now self-computes `avgdl` and builds via the custom-tokenizer + `with_avgdl`
  path; OQ-3 enumerates the exact builder methods to pin in the PHASE-01 probe.
- **A2 (high) — over-claimed determinism.** §5.5 asserted the f32 "fixed by inputs"
  while bm25 uses std `HashMap`/`HashSet` internally (confirmed in `scorer.rs`).
  *Disposition: bounded* — doctrine discards bm25's ordering (consumes only the
  per-doc score, re-sorts via `sort_key`), so the residual exposure is score-*value*
  stability across processes. Captured as OQ-5 + R7 with an empirical cross-process
  VT and a coarsen-`LEX_SCALE` fallback (determinism wins over resolution).
- **A3 (medium) — `Tokenizer` trait signature unverified.** lib.rs only re-exports
  it; the adapter shape (`fn tokenize(&self, &str) -> ?`) is assumed. *Disposition:
  PHASE-01 probe item (OQ-3.1); a non-`Vec<String>` return changes only the thin
  adapter, not the design.*
- **A4 (low) — "all active" imprecise.** It is the *partition-scoped* active set
  (`base_filter` applies `snap.part` + `include_draft`), not a global active set.
  *Disposition: fixed* in §5.1/§5.3 wording.
- **A5 (low) — OverlapRanker concat-vs-segment equivalence unstated.** A reviewer
  could doubt that space-joining segments then tokenizing equals the old
  per-segment union. *Disposition: fixed* — equivalence note added to §5.4; the
  parity test pins it.
- **A6 (low) — moving `tokenize()` may orphan callers.** *Disposition: PHASE-01
  must `grep` every `tokenize(` use in `retrieve` before the move; only
  `lexical_score` is expected, but verify.*
- **A7 (low) — "Memory serialization unchanged" is a claim, not a test.**
  *Disposition: it is structurally true (SL-017 edits no serialization path); the
  assertion is belt-and-suspenders, not the primary guarantee. Wording in §5.5/R3
  reflects "by construction."*

### Doctrinal alignment

- **New runtime dependency (`bm25`) — ADR needed?** *Judged no.* ADR-001 governs
  module layering, not dependency admission; the project adds libraries
  (`clap`/`serde`/`glob`/…) at slice altitude without ADRs. `bm25` is a scoped
  scoring implementation detail, not a project-global architecture decision (unlike
  the forgettable event-store backend, which is an ADR). If the user disagrees,
  promote to `doctrine adr new` before PHASE-01.
- **Storage rule / float ban — respected.** Score is derived, never stored; no
  `f32` in payload/export/backend; no new persisted field (R3, by construction).
- **Pure/impure split — respected.** Scoring is pure; the shell constructs the
  ranker and loads memories. `avgdl` is computed from in-memory `LexDoc` text, no
  I/O in the leaf.
- **Behaviour-preservation gate — honoured.** Existing SL-008 suites stay green via
  `OverlapRanker` parity; the BM25 default is proven by new + re-baselined tests.

Round-1 findings integrated. No unresolved governance conflict — the two PHASE-01
probes (OQ-3 API surface, OQ-5 determinism) are the gating unknowns; both have a
defined `/consult`/fallback path if they fail.

### Inquisition (round 2) — no architectural veto; 4 required fixes applied

- **A1 (critical) — absent-vs-zero ambiguity.** `score` was best-effort + caller
  `unwrap_or(0)`, so a mis-sliced target silently demoted to 0 in release.
  *Applied:* trait now returns exactly one entry per target in order (ranker fills
  no-evidence zeros); assembly is positional; `targets ⊆ corpus` promoted to a
  **hard `assert!` (all builds)**. §5.1/§5.2/§5.4/§5.5.
- **A2 (high) — determinism fallback underspecified.** Coarsening `LEX_SCALE` was
  implied sufficient by construction. *Applied:* R7 is now a 5-rung ladder
  (discard ordering → empirical VT → deterministic-summation → coarsen *only if
  the VT passes after* → `/consult`/don't-ship). §8 R7.
- **A3 (high) — `avgdl`/doc-len equivalence unproven.** Self-computed `avgdl` could
  diverge from the scorer's internal `doc_len`, corrupting length normalisation
  while tests pass. *Applied:* §5.4 pins the equivalence invariant; §9 + OQ-3.4 add
  a PHASE-01 differential probe.
- **A4 (medium) — future `Indexed` seam not universal.** *Applied:* D7 records the
  construction-time-failure constraint for unsupported ranker/corpus pairings.
- **A5 (medium) — "all active" overloaded in the slice.** *Applied:* slice OQ-2
  reworded to "partition-scoped `base_filter` survivors, drafts only when
  `include_draft`."
- **A6 (medium) — "selectable/fallback" scope-leak in the slice.** *Applied:* slice
  reworded to "internal parity/fallback behind the trait; no user-facing selector."
- **A7 (low) — query edge cases.** *Applied:* §9 explicitly tests `None`,
  `Some("")`, `Some("…separators…")`.
- **A8 (low) — silent `quantize(∞)==0` hides scorer bugs.** *Applied:* `quantize`
  gains `debug_assert!(score.is_finite())` before the defensive guard.

Round-2 required fixes (A1–A3, A5–A6) integrated; A4/A7/A8 also applied. Gating
unknowns unchanged (OQ-3 API surface incl. avgdl equivalence; OQ-5 determinism) —
both PHASE-01 probes with explicit escape hatches. Design cleared to plan.
