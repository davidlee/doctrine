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
active    = mems.filter(base_filter)                    // fit corpus (searchable set; honours include_draft)
survivors = active.filter(scope_match? + thread_expiry) // score targets + Candidates
corpus    = LexicalCorpus::Raw(&active_lexdocs)
targets   = survivor uids
scores    = ranker.score(q.query, &corpus, &targets)    // id -> u32
Candidate.lexical = scores.get(uid).copied().unwrap_or(0)
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
    /// Fit corpus-level statistics over `corpus`, score only `targets`, and
    /// return id -> quantized lexical score (Key-2 magnitude).
    ///
    /// Contract:
    /// - Every `target` is scorable; a target absent from the result means a
    ///   *legitimate* zero (query None / empty after tokenize / no term matched),
    ///   assembled as 0 by the caller — never a silent lookup miss.
    /// - `debug_assert!` every target id is present in `corpus` (targets ⊆ corpus).
    /// - Duplicate ids in `corpus` violate the uniqueness contract
    ///   (`debug_assert!` rejects; release behaviour is last-wins, undefined-but-safe).
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
2. Fit `Embedder` to the active corpus; upsert every active `LexDoc` into a
   `Scorer` (keyed by uid). df/IDF and avgdl now describe the searchable set.
3. `matches(embed(query))` → ranked `(uid, f32)` for docs a query term touched.
4. Filter to `targets`, `quantize` each f32, assemble `Vec<(uid, u32)>`; targets
   not in `matches` are left for the caller's `unwrap_or(0)`.

`OverlapRanker::score`: per-target distinct-query-token set-membership over its
`LexDoc.text` (re-tokenized), returning the `u32` count directly — **no quantize**,
byte-identical to the retired `lexical_score`. Corpus is ignored (per-document).

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
- **Targets ⊆ corpus** (`debug_assert`); **unique corpus ids** (`debug_assert`).
- **`Memory` serialization unchanged** (asserted) — no float-bearing payload.

## 6. Open Questions & Unknowns

- **OQ-3 (build) — partially resolved, gated on PHASE-01 probe.** `bm25 = 2.3.2`
  (MIT) fetches over the live network. Unconfirmed until built: that
  `Tokenizer` / `EmbedderBuilder` (fit-to-corpus) / `Scorer` (`upsert`/`matches`)
  remain exposed under `default-features = false`. If any core scoring API is
  feature-gated behind `default`/`default_tokenizer`, **do not silently broaden
  dependencies — stop and `/consult`.**
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
  slice — would force a signature change later.

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
  scores); empty query / empty-after-tokenize / empty corpus ⇒ all-zero; non-matching
  survivor ⇒ 0; IDF drawn from full corpus not just targets (a target's score
  reflects df over active, not over the target subset).
- Contract invariants: `targets ⊄ corpus` trips the debug assert; duplicate corpus
  ids trip the debug assert.

**`retrieve` integration:**
- `exact_key` still dominates Key 2 under BM25 (an exact-key hit outranks a
  higher-BM25 non-key hit).
- shuffle-invariance property test green with `Bm25Ranker` wired.
- one crafted case where BM25 reorders vs overlap (the intended quality change).
- `Memory` serialization unchanged (no float payload / new field).

**e2e/VT:** `doctrine memory find --query …` default path uses BM25, deterministic.

**Lint:** guarded `as` casts in `quantize` carry a narrow
`#[expect(clippy::cast_possible_truncation, reason = "…")]` (and
`cast_precision_loss` for `u32::MAX as f32`) — `expect` + `reason`, never bare
`allow` (`mem.pattern.lint.expect-not-allow`). `just check` / `cargo clippy` zero
warnings.

## 10. Review Notes

(Adversarial pass pending — to be recorded here.)
