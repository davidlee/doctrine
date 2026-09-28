# Memory search: default page, retrieval floor, and a zero-evidence signal

## Context

`doctrine memory search` is a ranked read surface (BM25, `src/retrieve.rs`) with
two defects that compound, plus a third that makes the first two hard to see.
Promoted from `ISS-465`, re-scoped after live probing (2026-09-27).

**1. No default page — and the pagination is incoherent, not merely open-ended.**
`run_search` takes `limit: Option<usize>` and takes `usize::MAX` when it is unset
(`src/retrieve.rs:889-891`, deliberate per the comment at :884-888). The CLI
resolves `--limit` to `RETRIEVE_LIMIT_DEFAULT` (5) for the **offset** arithmetic
(`src/memory.rs:619-621`), then passes the *unresolved* `Option` through to
`run_search` (`src/memory.rs:642`), while the **footer** uses a third value,
`limit.unwrap_or(shown)` (`src/retrieve.rs:918`). Three notions of page size in one
pagination:

```
$ doctrine memory search "bwrap"              # no --limit
565 rows
$ doctrine memory search "bwrap" --page 2     # no --limit
560 rows, footer: "560 of 565; use --page 2 for next"
```

The footer tells you to go to the page you are already on. `--page N` without
`--limit` is silently wrong on any reading, independent of whether an unset
`--limit` *should* mean "everything".

**2. No retrieval floor.** `doctrine search` drops zero-score hits —
`filter(|(_, score)| *score > 0)` (`src/search.rs:383`). `memory search` has no
equivalent: every filter survivor is returned, ranked, with no evidence threshold.

**3. No zero-evidence signal.** When a query's tokens match no document, BM25 ties
at 0 for every candidate and the 9-key total order falls through to verification →
trust → severity (`src/retrieve.rs:485-500`). A zero-evidence query therefore
returns the same high-severity memories every time, looking exactly like a
successful ranked result.

**The compounding effect, and why this was misdiagnosed.** Because the row count is
query-invariant when the result set is unpaged, unrelated queries report the same
*count*. That reads as "the query was ignored", and it is how `ISS-465` and five
friction observation records came to describe a *ranker* defect that does not
exist — the ordering always varied. `mem.pattern.cli.unpaged-count-is-not-a-ranking-signal`
records the diagnostic.

## Scope & Objectives

1. **Page by default.** `memory search` (CLI and MCP `memory_search`) resolves an
   unset `--limit` to one named constant, `SEARCH_LIMIT_DEFAULT = 20`, so an unset
   `--limit` is a page, not the corpus (`DEC-347`; supersedes the "5" framing).
2. **One notion of page size.** `--offset`/`--page` arithmetic, the row slice, the
   footer's continuation hint, and MCP's `limit`/`next_offset` derive from the same
   value, resolved once in the command layer. `--page N` without `--limit` must be
   correct.
3. **A retrieval floor.** Under a free-text query, rows with no evidence (no lexical
   score, no exact-key hit) are excluded; selector-only requests are unfloored
   (`DEC-348`). The floor lives in the shared `query()` pipeline, so it reaches
   `memory retrieve` too (`DEC-350` — scope widened 2026-09-28).
4. **A zero-evidence signal.** A free-text query with no evidence says so in table
   mode; `--json`/MCP signal it by empty rows, no wire change (`DEC-349`).

## Non-Goals

- **The ranker itself.** BM25 is not at fault and does not change; `sort_key`'s key
  order is untouched except where a floor makes the fallback unreachable.
- **The observation ledger's read surface.** Unranked by `DEC-051`, a different
  corpus and a different governance question — that is `SL-274`, deliberately
  excluded here.
- **The entity search surface** (`doctrine search`, SL-141). It is the *precedent*
  for objective 3, not a target; if its floor needs changing, that is its own item.
- **`IMP-154`** (index non-entity docs) — corpus widening, not retrieval shape.
- **`memory retrieve`'s** five-row default and cap (`RETRIEVE_LIMIT_DEFAULT`/`_MAX`)
  stay unchanged. Retrieve *does* gain the retrieval floor (`DEC-350`).
- **The surface hook** (`retrieve_rows`) — it carries no free text, so the floor never
  applies to it; it remains the scope-only, severity-gated mode.

## Affected surface

- `src/retrieve.rs` — `run_search` (`:849`), `run_retrieve` (`:1288`), `query`
  (`:580`), `rank`/`sort_key` (`:503`/`:485`), `RETRIEVE_LIMIT_DEFAULT`/`_MAX`
  (`:935`/`:937`), and the pagination clamp at `:889-891`.
- `src/memory.rs` — `FindRetrieveArgs` (`:45`, `limit` at `:104`), the `Search` and
  `Retrieve` dispatch arms (`:606`, `:645`), and `search_for_mcp` (`:1187`) which
  must not diverge.

## Risks & assumptions

- **A1 (resolved, affirmative)** — consumers of an unbounded search exist, and two
  are verification artefacts. `tests/e2e_memory_sync.rs:425-431` runs an unpaged
  scoped `memory find` and asserts a uid at **index 6 of 10 rows** (measured) — it
  fails under a 5-row default. The MCP VT-1 browse test (`tools.rs:2797`) asserts
  non-empty. Also affected: `record-memory`'s §7 sanity check, `dreaming`, and the
  `using-doctrine.md` guidance. Their update is **in scope**, not fallout
  (`research.md` `F-1`).
- **A2** — no `REQ` fixes this read surface (verified negative, with a positive
  control); the convention being reversed lives in three closed slice designs and one
  review finding. **No Revision is required** — but the design must name the records
  it supersedes (`R5`).
- **R1** — **Three call sites, one contract.** `memory.rs::Search`, `memory.rs::Retrieve`
  and `retrieve.rs::search_for_mcp` each resolve limit/pagination independently;
  `Retrieve` is the model to copy (one resolved value, forwarded concretely).
  Fixing one and missing another reproduces the divergence this slice exists to
  remove; the golden suite must pin all three.
- **R2** — `--limit` is shared by two verbs through one args struct
  (`FindRetrieveArgs`), so a default change lands on both. `search` never applies
  `RETRIEVE_LIMIT_MAX`, so an explicit `--limit 9999` is uncapped there while
  `retrieve` caps at 20.
- **R3** — A retrieval floor can turn a *successful* query into zero rows. The
  floor must be evidence-based (BM25 score), not result-count-based, or a legitimately
  sparse query loses its only hits. See `OQ-7` on conditionality.
- **R4** — Which layer owns the default (command vs engine) is *not* settled by
  ADR-001, which fixes dependency direction, not where a numeric default lives.
  Precedent runs both ways: `doctrine search` defaults at the clap boundary
  (`src/search.rs:220-222`), the memory surface stores an engine constant it resolves
  in the command layer. `STD-001` warns against a second `SEARCH_LIMIT_DEFAULT = 5`.
- **R5 — supersession duty.** The unpaged default is a deliberate locked decision:
  `SL-086` `D14` removed the find default that `SL-008` `D17` had set to 5,
  "preserving the current `memory find` behaviour", and `SL-131` §2 / `SL-184` §2
  reaffirm it. The design must name each superseded record explicitly, or the corpus
  carries two contradictory accounts of one surface. Also correct the
  `src/retrieve.rs:887-890` comment, whose revert authority is `RV-207` `F-2` and is
  unstated.
- **R6 — the MCP selector branch.** With any selector and no explicit limit the MCP
  handler is **uncapped** (`tools.rs:889-891` → `cap = usize::MAX`), and the shipped
  `retrieve-memory` skill tells agents to "Always supply at least one selector" — so
  the agent-facing path hits the unbounded branch by design. In scope or an explicit
  follow-up; leaving it diverges the two surfaces.

## Open questions

- **OQ-1** — *Resolved.* Something does consume an unbounded search — five things,
  named under `A1`. Their update is in scope.
- **OQ-2** — Floor semantics: hard exclusion of score-0 hits, or a *marked* boundary
  ("no lexical match — showing by severity")? Research **favours the marked
  boundary**, and the grounds are not consumer convenience: `SL-008` `D17`/`D18` and
  `REQ-152` construct the severity fallback deliberately to keep risk visible on the
  holdback-exempt surface, and `SL-141` `D8`'s stated rationale ("entity search has
  no scope-filter pre-pass") does not transfer to memory, which has one. Hard
  exclusion fixes a *signalling* defect by deleting a capability.
- **OQ-3** — Should `search` also adopt `RETRIEVE_LIMIT_MAX`, or is an explicit
  `--limit` unbounded by design for search (the `SL-086`/`SL-131` position)?
- **OQ-4** — Where does the zero-evidence signal surface — a footer line, an empty
  result, or a distinct status in JSON (a small wire change with golden cost)?
- **OQ-5** — Are the constants correctly homed, or should the *default* be resolved
  at the command layer while the engine keeps only the cap? Precedent runs both ways
  (`R4`); `STD-001` argues against a second constant for the same value.
- **OQ-7 — The floor's conditionality.** Unconditional, the floor empties the
  selector-only browse path and objective 1 becomes moot for browse. Conditional on a
  free-text query being present, "find this" and "show me what's here" become
  different requests — defensible, and research's recommendation, but it must be
  *stated as the contract*, because the MCP wire makes the two cases look alike.

## Verification / closure intent

- `memory search "<query>"` with no `--limit` returns exactly
  `RETRIEVE_LIMIT_DEFAULT` rows and a self-consistent continuation hint.
- `memory search "<query>" --page 2` (no `--limit`) returns the second page, not
  page 2-onward; the footer names the correct next page. All three page-size notions
  (`src/memory.rs:619-621`, `src/retrieve.rs:891`, `src/retrieve.rs:918`) collapse to
  one resolved value.
- A query whose tokens match no document reports zero evidence, and does not present
  the severity-ordered corpus as though it had ranked (shape per `OQ-2`/`OQ-4`).
- The two affected verification artefacts are updated **deliberately and in the
  open**: `tests/e2e_memory_sync.rs:425` re-expresses "this memory is findable"
  rather than "this uid is present in an unpaged dump", and the MCP VT-1 browse
  expectation is restated under whatever `OQ-7` decides.
- Behaviour preservation: a query with genuine lexical hits returns the same ordered
  top-N as today for the same `--limit`; the existing ranking suites stay green
  unchanged (the behaviour-preservation proof for the shared machinery).

## Follow-Ups

Pre-design research round is **complete** — `research/` (`research.md` + `raw/`),
distilled from two threads and independently verified. Design consumes the ✓ rows of
`research.md` before this card's own framing, which research partly supersedes
(`F-1`–`F-5`; `A1` resolved, `R5`/`R6` added, `OQ-2`'s default reversed, `OQ-7` new).
