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
