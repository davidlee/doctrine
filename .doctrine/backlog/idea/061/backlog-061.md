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

## Settled (design conversation, 2026-09-29)

- **Eval first.** One code path, two drivers: rank one slice; replay many and
  report. Verdict lands as an EVD record. No workflow integration until the
  evidence justifies it. Rationale: feature-first costs ~2–4× and still needs the
  eval; its integration spend is wasted on a negative result.
- **Rust**, in a workspace crate outside `default-members` (like
  `crates/doctrine-control`). User expects to adopt Jev in some form, so the
  client, sectioner, and provenance should carry forward. Keeps the MSRV and HTTP
  dependency off shipped users; no POL-002 capability to declare yet.
- **Don't presume BM25 is the right home.** Three arms:
  A BM25 pool + BM25 rank (baseline); B BM25 pool + Jev rank; C whole snapshot +
  Jev rank (recall ceiling — is the BM25 pool the bottleneck?). Authored corpus
  ≈ 32 MB / ~8M tokens → arm C ≈ $0.40 and ~30 s per query at documented
  price/throughput.
- **Leakage guard:** each slice's corpus is rebuilt as of its pre-research commit.

- **Thin HTTP client** (`reqwest` + `serde`) over `POST /v1/systemone`, owning
  the error typing: rate-limit vs infrastructure failure vs semantic abstention
  are distinct; no failure becomes a zero score; raw probabilities retained for
  replay. `typesafe-sdk-rust` is a wire-shape reference only, not a dependency.
- **Silver labels:** entity ids cited in each slice's `research.md` +
  `design.md`; user spot-checks ~5 slices before tuning. Citations measure what
  was *found and used*, not what was *relevant* — the report lists Jev-surfaced
  uncited entities (esp. arm C) for human judgment rather than scoring them wrong.

- **Data egress:** sending the authored `.doctrine/` corpus is acceptable (user).
  Still: explicit path allow-list (default `.doctrine/` entity bodies) and a
  local log of what each run sent.
- **Credentials:** env var `JEV_API_KEY`, forwarded into the jail by the user
  (needs a harness restart to take effect). Never written to records, cache, or
  debug output.
- **Budget:** user is on the free tier for now; ~$20 acceptable if it comes to
  that. Dry-run by default (planned requests, token + dollar estimate); live runs
  take an explicit spend cap and abort before exceeding it, retries counted.
  Free-tier rate limits are likely tighter than the documented 1,200 req/min —
  the client must back off on rate-limit responses, never score through them.
  Sequence: ~10-call live probe → one slice across arms A–C → full ~25 slices.

## Open forks

None blocking. Next: promote to a slice → `/research` → `/design`; user
spot-check of silver labels is a planned human step.
