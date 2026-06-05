# Audit SL-017: Pluggable lexical scorer — trait + BM25 backend

Mode: **conformance** (post-implementation, slice-tied). Hand-authored — no
`slice audit` scaffold yet (CLAUDE.md known-gap). Audits the implemented slice
against `design.md` (D1/D5, §5.1/§5.3/§5.4, §9), `plan.toml` EX/VT, and ADR-001.

## Gate evidence (fresh, this audit)

`just check` GREEN: **445 unit + 3 probe (bm25) + 4 e2e + 1 skills** pass; `cargo
fmt --check` clean; `cargo clippy` (bins/lib, NOT `--all-targets` — repo gate)
zero-warning. Release-profile fallback re-verified:
`quantize_non_finite_is_zero_in_release ... ok` (debug `just check` proves the
A8 panic path; release proves the non-finite→0 totality — Decision A profile-split).

### Gate contamination — NOT an SL-017 defect (resolved for measurement)

First `just check` run was RED: 14 `skills::tests::*` failures, every one a
frontmatter parse error `mapping values are not allowed … line 2 column 106` on
`plugins/doctrine/skills/spec-product/SKILL.md`. Cause: **uncommitted SL-019 WIP**
(`M install/templates/spec-product.md`, `M …/spec-product/SKILL.md`; HEAD is now
`ec7a5ae design(SL-019)`) — an unquoted YAML `description` whose value contains
`specification/prd: the…` (the `: ` opens a mapping). Zero `retrieve`/`lexical`
failures. Parked via `git stash` (`SL-019 spec-product WIP …`) → gate green.
**Disposition: out-of-scope, tolerated (external).** SL-019's frontmatter bug is
its own to fix; `/close` must restore the stash and must NOT commit those two
files. Recorded so the closer does not mistake it for SL-017 breakage.

## EX disposition (PHASE-04 — the wiring phase; P01–P03 EX discharged at their phases)

- **EX-1** `query(ranker:&dyn LexicalRanker)`; active(fit)/survivors split; one
  `lex_doc` per active; `Candidate.lexical` positional, no `unwrap_or` —
  **aligned**. `src/retrieve.rs:540-588`: `active`=base_filter survivors → `docs`
  via `lex_doc` → `survivors` (scope+thread_expiry, carrying Copy `ScopeMatch`) →
  `survivors.zip(scores)` positional. The `survivors ⊆ active = corpus` invariant
  is documented inline (L556-558) so the ranker's `targets ⊆ corpus` assert cannot
  fire from `query`.
- **EX-2** `lexical_score` deleted; `tokenize` sourced from `lexical`; exact_key +
  9-key SortKey + Key-2 `Reverse<u32>` polarity unchanged — **aligned**.
  `grep 'fn lexical_score' src/` ⇒ NONE; `use crate::lexical::tokenize` dropped from
  retrieve (now via the leaf). SortKey untouched (no diff in the rank core).
- **EX-3** impure shell builds the default `Bm25Ranker`, passes `&dyn` in; no
  CLI/env/config selector (D5); find AND retrieve use it — **aligned**.
  `run_find:717` and `run_retrieve` both `let ranker = Bm25Ranker; query(…, &ranker)`.
  `query` never constructs a ranker (purity boundary, design §5.1).
- **EX-4** behaviour-preservation: pre-existing SL-008 overlap tests pass UNCHANGED
  re-pointed through `OverlapRanker`; holdback/staleness/filters/partition untouched
  — **aligned**. `query_bare_query_keeps_all_active_ranked_lexically` re-pointed
  through `&OverlapRanker`, assertions unchanged; 441 pre-existing tests green.
- **EX-5** §9 `dead_code` bridge REMOVED (self-clearing on wiring); clippy/suite/e2e
  green; notes.md updated — **aligned**. `grep 'cfg_attr(not(test)' src/lexical.rs`
  ⇒ NONE; module doc rewritten to record the self-clear; zero-warning gate.

## VT disposition (PHASE-04, plan.toml block #4)

- **VT-1** exact_key dominates higher BM25 → `query_exact_key_dominates_higher_bm25`
  (retrieve.rs:1956). Exact-key hit with LOWER bm25 ranks first (Key-1 > Key-2
  magnitude), asserted on live scores. **aligned.**
- **VT-2** shuffle-invariance with Bm25Ranker wired →
  `query_is_shuffle_invariant_under_bm25` (1998): permuted store ⇒ identical
  `(uid, lexical)` sequence. **aligned.**
- **VT-3** cross-process byte-identical → e2e
  `find_bm25_ranking_is_cross_process_deterministic`
  (tests/e2e_memory_anchoring.rs:313): two separate `memory find --query` processes
  over one seeded store ⇒ byte-identical stdout. R7 LEX_SCALE-coarsen rung UNREACHED
  (determinism held first run). **aligned.**
- **VT-4** BM25 reorders vs overlap → `query_bm25_and_overlap_order_oppositely`
  (2038): same query, `&Bm25Ranker` puts the rare-term match first while
  `&OverlapRanker` puts the higher raw-overlap match first — the intended quality
  change, made visible. **aligned.**
- **VT-5** storage model unchanged → `query_bm25_score_is_derived_not_persisted_on_memory`
  (2096): `Memory` Debug carries no `lexical`/float field; scoring takes an immutable
  borrow. No persisted float, by construction (R3). **aligned.**

## Harvested seam decisions (from the gitignored phase sheets / notes.md)

1. **OverlapRanker → `#[cfg(test)]` — deliberate §5.4 deviation, ACCEPTED.**
   §5.4 says "retire `lexical_score`". Post-D5 (BM25 the only runtime ranker, no
   selector) `OverlapRanker` has no production caller; removing the module bridge
   (EX-5) would otherwise leave it dead in the bins/lib build. It retires INTO the
   test harness — struct+impl both `#[cfg(test)]` (lexical.rs:79-80, 129-130) — as
   the behaviour-preservation instrument the SL-008 overlap tests re-point through.
   Faithful to §5.4's intent (retire-vs-delete); a future selector un-gates it in
   its own slice (YAGNI). **Disposition: aligned** — conscious, documented, no
   external behaviour change.

2. **Parity-vs-fn → parity-vs-frozen-vector — ACCEPTED.** `lexical_score` deleted,
   so `overlap_ranker_preserves_retired_overlap` (retrieve.rs:1299) asserts FROZEN
   overlap counts lifted verbatim from the deleted unit tests, scored through the
   production `lex_doc`. Literals hand-verified against the §5.3 bag semantics
   (title summary tags key, distinct-token set membership): fixture-2 bag =
   `{src,memory,rs,clippy,expiry,token,lint,rust,mem,pattern}`; query
   "src memory rs lint clippy" ⇒ 5 ✓, "token middleware rust auth" ⇒ token+rust ⇒
   2 ✓. No recompute-by-eye drift. **Disposition: aligned.**

## Code review — `git diff 92f498a..b75e6bb` (the two SL-017 PHASE-04 commits)

`984165f` (wiring) + `b75e6bb` (VT) on parent `92f498a`. Scope: retrieve.rs +413,
lexical.rs ±25, e2e +51, notes +45.

- `query()` restructure — survivors⊆active invariant holds by construction;
  positional `zip(scores)` with destructured `(_, lexical)`, no `unwrap_or`. Clean.
- Purity boundary — both shells construct `Bm25Ranker`; `query` takes `&dyn` only.
- `lex_doc` — the SINGLE `Memory→LexDoc` adapter (retrieve.rs:235); body excluded
  (Q1/B15). Both the fit corpus and the parity test score through it — no parallel
  projection (DRY, design D6).
- `cfg(test)` gating — struct AND impl gated (lexical.rs:79, 129); module
  `dead_code` bridge fully removed, doc rewritten to record the self-clear.
- **No defects found.** No fix-now, no follow-up, no tolerated drift inside SL-017.

## Known/owned follow-ups (out of scope — NOT closure blockers)

- `Indexed(&LexicalIndex)` precompute — `lex_doc` clone + full per-query embed is
  O(active) (design §6 corpus cost, R5). Non-breaking (`LexicalCorpus` enum, D7).
  SL-018's shipped corpus bumps it "later → soon" (notes.md forward-link). Own slice.
- `record` has no `--trust`/`--severity` flag (producer audit A-3, CLAUDE.md
  known-gaps) — risk axes TOML-only. Own slice, not SL-017.

## Closure readiness

All 5 EX + 5 VT (PHASE-04) **aligned**; P01–P03 discharged at their phases. Gate
green (contamination isolated to parked SL-019 WIP). Both seam decisions harvested
and accepted. No undispositioned findings. **Audit-ready → `/close`.**

Closer checklist: confirm rollup 4/4 · reconcile `slice-017.toml` status
proposed→done (clears the `⚠`) · `git stash pop` to restore SL-019 WIP · final
commit touches ONLY SL-017 artifacts (audit.md, slice-017.toml), never the
spec-product files.
