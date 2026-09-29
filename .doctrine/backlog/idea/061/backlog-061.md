# IDE-061: Jev semantic reranking trial for research relevance

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Idea

Trial TypeSafe's Jev — a hosted "System One" classifier that returns typed,
calibrated judgments (Choice / Score / Noul) over caller-defined answer spaces,
not prose — as a cheap semantic reranker over Doctrine's existing lexical
candidates. Goal: better useful-source coverage for the `/research` round within
the reasoning agent's context budget.

Source: a local handoff brief (`jev-brief.local.md`, gitignored, 2026-09-29). It
also sketches later trials — repair propagation, citation support, timely
guidance — that should reuse this trial's machinery if it works. Out of scope
here.

## Shape (to be settled in design)

```
doctrine search / BM25 + refs + graph neighbours   (candidate pool)
  + mandatory governance (deterministic, never scored away)
  → Jev: applicability + contribution per passage   (optional, advisory)
  → dedupe, token-budget select, keep provenance
  → ranked research packet + diagnostics
```

Evaluation: ~20–30 completed slices; compare BM25, BM25+graph, and the same pool
reranked. Measure candidate recall separately from ranking quality, then
relevant-source recall at a fixed token budget.

## Preflight findings (2026-09-29)

- Seams: `LexicalRanker` (`src/lexical.rs`, SL-017) — a network, fallible
  reranker does not belong behind it; it is a post-candidate stage.
  `doctrine search` (`src/search.rs`) scores whole entities, not passages.
- Corpus: 78 slices carry `research/research.md` (226 raw thread files) —
  possible silver labels, agent-authored and fallible.
- POL-002 facet 3: a hosted-service dependency is a feature-scoped capability —
  opt-in, fails naming what is missing. STD-003: an inference failure is
  disclosed, never mapped to a zero score.
- Toolchain: workspace `rust-version = 1.85`; `typesafe-sdk-rust` 0.1.1 needs
  1.98. No HTTP client crate in the tree today (tokio only).

## Open forks

1. Standalone evaluator (outside the shipped binary) vs opt-in cargo feature.
2. Transport: Rust SDK port, thin HTTP adapter, or official Python SDK.
3. Data egress: may corpus text go to the hosted API; credential source.
4. Label adjudication: human, silver labels from research.md, or LLM-assisted.
5. Spend / request budget for the first live run.
