# Implementation Plan SL-008: Memory retrieval: find/retrieve + scope ranking + staleness

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

SL-008 is the **reader** over the SL-007-populated store: `find` (ranked rows)
and `retrieve` (the bounded agent-context block). The ranking algorithm is
already locked by `doc/memory-spec.md` § Retrieval and re-confirmed across the
codex review (design D1–D20), so the plan is not about *inventing* ranking — it
sequences three correctness disciplines into shippable phases: determinism that
does not leak (frozen snapshot, total `Ord`), a per-block security contract
(`render_show` with a fresh nonce each), and a clean pure/impure split for
staleness (`commits_touching` resolved at the shell, pure `staleness` over plain
`GitFacts`).

## Sequencing & Rationale

The split is **pure core first, impurity last, surfaces on top** — the project's
pure/imperative gate makes the bulk (matching, scoring, staleness, rank) testable
as plain functions before any git or CLI exists.

- **PHASE-01 (filters) and PHASE-02 (rank)** are the pure spine, separated along
  the design's load-bearing line: *filters drop, rank orders* (B1). Building them
  as distinct phases keeps that boundary honest — a predicate can never sneak in
  as an `Ord` key. PHASE-02 depends on PHASE-01 only for the `ScopeMatch`/
  `QueryContext` types.
- **PHASE-03 (git seam)** is the single new impurity. It is isolated so the
  `merge-base --is-ancestor` precheck — the one subtle correctness trap (`A..B`
  is a set-difference, not an ancestry test — F2) — gets its own focused test
  surface against real temp-repo fixtures. It rides SL-007's locked seam
  (`capture`, NORMATIVE_FLAGS) and adds **no** new SL-007 surface (F1).
- **PHASE-04 (find)** is where the impure shell is assembled and frozen — the
  `Snapshot` (today + target + partition), `collect_memories`, per-candidate
  `GitFacts` resolution, and the ordered pipeline. It deliberately precedes
  retrieve because that shell is **shared**: retrieve is then a thin surface over
  it. `find` is also the lower-risk surface (no holdback, no body rendering), so
  it shakes out the wiring before the security-sensitive command lands.
- **PHASE-05 (retrieve)** comes last because it carries the security contract:
  per-block fresh nonce (D2), pre-render suppression (quarantined/retracted +
  the `low ∧ severity≥high` holdback, D8), and the bounded `--limit`. Layering it
  on the proven PHASE-04 shell means the only new concerns are suppression and
  framing, reviewed in isolation.

Each phase ends green (tests + clippy zero) and is independently committable;
the entrance criteria encode the merge dependencies above.

## Notes

- The two trailing open questions were locked before planning: **D19** (staleness
  never hides — display + sort-key only, no `--fresh-only`; conditional on
  staleness staying *visible* on both surfaces) and **D20** (a bare `--query`
  ranks lexically over all active memories). See design.md § 6/7.
- Deferred to a future F-index follow-up (explicitly out of v1 scope): body-text
  search (`memory.md` body is rendered-as-data but not *scanned*, B15),
  glob→git-pathspec expansion for commit-staleness (paths-only in v1, B5), and any
  persistent lexical index.

## Review

Adversarial plan review (codex, gpt-5.2, read-only) — 2 red, 2 amber, 1 green,
all adjudicated into the criteria above (ids appended, never renumbered):

- **F1 (red) → PHASE-01 EX-5/VT-4.** Per-dimension matcher semantics (spec
  § Retrieval: paths exact/prefix, globs `**`-aware, commands token-prefix, tags
  set-intersection) were absent — the phase could pass with a wrong matcher.
- **F2 (red) → PHASE-04 EX-5/VT-4, PHASE-05 EX-4/VT-4.** `--type`/`--status` were
  unspecified; now explicit hard filters riding the existing `select_rows`
  AND-filter (`src/memory.rs`) — no parallel implementation.
- **F3 (amber) → PHASE-04 EX-6, PHASE-05 EX-4.** The shared-shell boundary is now
  a contract: the query stage returns `Vec<Ranked>` carrying the `Memory`, so
  `retrieve` reuses it (reading bodies per `take(limit)` hit via the existing
  `resolve_show` seam) without forcing a PHASE-04 refactor.
- **F4 (amber) → PHASE-02 EX-5/VT-4.** D4 made enforceable: pure in-memory
  tokenization (case-fold, non-alnum split), no persistent index / derived write.
- **F5 (green).** PHASE-03's git-seam isolation confirmed sound (rides the locked
  `src/git.rs` seam; impurity stays out of the pure ranker).
