# Review RV-412 — code-review of SL-275

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Subject: SL-275 PHASE-01 (`7edab1dda`) + PHASE-02 (`482337367`) — memory search
page size, retrieval floor, zero-evidence notice. Reviewer: `./scripts/pi-research`
(deepseek-v4-pro, read-only); findings verified and raised by the orchestrator.

Lines of attack:
- Page-size contract: one resolution site per surface; any residual page-size
  notion (run_retrieve's internal `.min(MAX)`, MCP `limit` echo); overflow paths
  (`page_offset`, `next_offset`, `offset + shown`).
- Continuation hint: byte-identity for aligned non-final pages across all
  `format_truncation_notice` callers (search, retrieve, priority `paginated`).
- Floor: `has_free_text` / BM25 tokenizer agreement; exact-key path; floor
  placement vs `git_facts` cost; surface hook untouched.
- Notice: pre-holdback keying (F-7); scrub; json byte-identity; MCP
  `memory_retrieve` table path.
- Tests: behaviour vs theatre; fixture helpers (duplication with MCP seeders);
  the one re-pinned keep-all test.
- Layering (ADR-001), STD-001 constants, STD-003 no-silent-skip.

## Synthesis

- **Overall**: solid
- **Synopsis**: The reviewer (`pi-research`, deepseek-v4-pro, high thinking) read
  both commits against the design and plan and ran the focused suites, clippy and
  live probes. It found no blocker or major. The page-size contract holds with
  one resolution per surface. Retrieve keeps a deliberate second `.min(MAX)` per
  design sec-2. The continuation hint is byte-identical on aligned non-final
  pages, and every overflow path is checked or saturating. The floor uses the
  same `lexical::tokenize` as BM25, so `has_free_text` and the ranker cannot
  disagree. The notice is keyed before holdback, scrubbed, and table-only.
  Findings were doc drift (F-1, F-2, F-5), test pinning (F-3, F-4) and a wasted
  pre-guard load (F-6), all fixed in `a2bd4d332`. F-7 (floor after `git_facts`)
  is tolerated as design-mandated placement with no regression.
  Accepted without a raise: the MCP handler's inline limit resolution differs
  from `resolve_limit` only in error wording, and all-held-back retrieve stays
  silent as governed by DEC-349 / RV-410 F-7. Out of scope: the unchecked
  `priority` page arithmetic, captured as ISS-503.
- **Haiku**:
  Twenty rows, then more —
  the query that finds nothing
  now says so, and stops.
