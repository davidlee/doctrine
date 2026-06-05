# Pluggable lexical scorer: trait + BM25 backend for memory retrieval

## Context

Memory retrieval (SL-008) ranks candidates with a fixed 9-key `Ord` tuple. Key 2
is the **lexical axis**: today a single concrete function, `lexical_score`
(`src/retrieve.rs`), returning a bounded `u32` token-overlap count of the
free-text query against each memory's `title + summary + tags + memory_key`
segments. It is a per-document set-membership count — distinct query tokens that
hit the doc bag — with no term weighting, no length normalisation, and no
awareness of how common a term is across the candidate set. Common tokens count
the same as rare, discriminating ones, so overlap is a coarse relevance signal.

The intent is to (a) lift the lexical axis behind a **trait** so the scoring
strategy is pluggable rather than hard-wired, (b) provide a backend built on the
`bm25` crate (github.com/Michael-JB/bm25) that ranks by term frequency × inverse
document frequency with length normalisation, and (c) make a chosen scorer the
**default** for memory retrieval — preserving the existing token-overlap scorer
as a selectable/ fallback implementation behind the same trait.

## Scope & Objectives

- Introduce a lexical-scorer trait abstraction over the lexical axis (Key 2),
  with the current token-overlap logic refactored into one implementation
  behind it (no-parallel-implementation: it rides the existing seam).
- Add a `bm25`-backed implementation behind the same trait.
- Select the default scorer for `doctrine memory find` / `doctrine memory
  retrieve`; the other scorer remains available behind the abstraction.
- Preserve the surrounding contract: the other 8 sort keys, their polarity, the
  `exact_key_match` dominance within Key 2, hard filters, scope specificity,
  staleness, trust holdback. The lexical axis feeds the same slot in the tuple.

The behaviour-preservation gate applies: changing the shared ranking machinery
must keep the existing retrieval suites green except where a test pins the *old*
lexical ordering and is intentionally re-baselined to the new scorer.

## Non-Goals

- No change to the canonical payload, event-store format, or export. Lexical
  scores remain **derived, never stored** — they never cross to the backend.
- No embeddings / semantic / vector retrieval (memory-spec keeps those
  out-of-band, sidecar; explicitly out of scope here).
- No change to the other 8 sort keys, the filter cascade, scope matching,
  staleness classification, or the trust holdback.
- No new persisted fields on the memory schema.
- No query-language or CLI-surface redesign beyond (at most) a scorer-selection
  affordance if design deems one necessary.

## Affected Surface

- `src/retrieve.rs` — `lexical_score`, `tokenize`, `SortKey` / `sort_key`,
  `rank`, `query` (the lexical axis and its consumers).
- `Cargo.toml` / `Cargo.lock` (workspace + crate) — new `bm25` dependency.
- Retrieval test suites that pin lexical ordering.
- Possibly a new module for the trait + implementations if `retrieve.rs` grows
  too large (decided in design; must respect ADR-001 module layering — leaf ←
  engine ← command, no cycles).

## Risks, Assumptions, Open Questions

Design (`design.md`) resolves the original open questions; status below.

- **OQ-1 — float → `Ord` quantization. RESOLVED (design D4).** BM25 emits `f32`
  (>= 0 under Lucene IDF). Quantize per-score to a Key-2 `u32` (`quantize`, scale
  1e6, monotonic non-decreasing, saturating, non-finite→0). Key 2 stays
  `Reverse<u32>`. Float ban (`doc/memory-spec.md` §584-585) targets
  payload/export/backend only; lexical score is derived/never-stored.
- **OQ-2 — corpus-relative scoring. RESOLVED (design D3).** Batch trait
  `LexicalRanker::score(query, &LexicalCorpus, targets)`. Fit IDF/avgdl over **all
  active memories** (the searchable set); score only survivors (bare `--query`:
  active = corpus = targets, SL-008 D20). Scoring stays pure.
- **OQ-3 — `bm25` crate fit & jail build. PARTIALLY RESOLVED.** `bm25 = 2.3.2`
  (MIT) fetches over the live network. **Gated on a PHASE-01 build probe:** that
  `Tokenizer`/`EmbedderBuilder`/`Scorer` remain exposed under
  `default-features = false`. If core scoring is feature-gated, STOP and
  `/consult` — never silently enable the default tokenizer deps.
- **OQ-4 — default selection. RESOLVED (design D5).** `Bm25Ranker` is the hard
  default; `OverlapRanker` retained only behind the trait (parity/fallback/future
  measurement). No CLI/env/config switch in SL-017. Determinism preserved
  (shuffle-invariance — IDF/avgdl are set-, not order-, dependent).
- **Assumption (held)** — `exact_key_match` stays a separate Key-1 component
  dominating Key 2; the new scorer changes only the lexical sub-signal.
- **Tokenizer policy (design D2):** reuse doctrine `tokenize()` via a custom bm25
  `Tokenizer`; no stemming/stopwords; preserve technical tokens. Stemming and
  technical-token expansion are deferred, measured experiments — out of scope.

## Verification / Closure Intent

- Trait abstraction in place; token-overlap scorer refactored behind it with the
  existing lexical tests re-expressed against the trait (behaviour preserved).
- BM25 backend implemented, unit-tested for term-weighting / length-norm /
  determinism, and wired as the resolved default.
- Ranking stays a total deterministic order (shuffle-invariance holds); the
  other 8 keys and the holdback are provably unchanged.
- `cargo clippy` zero warnings, `just check` green, no float reaches the
  canonical payload (grep/assert the boundary).
- Design (`/design` → `/inquisition`) resolves OQ-1..OQ-4 before any plan.

## Follow-Ups

- Potential generalisation: expose the lexical-scorer choice to other ranking
  consumers if/when they appear (out of scope now).
