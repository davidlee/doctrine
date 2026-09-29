<!-- doctrine:section sec-1 -->
## What changes

`doctrine memory search` gets a default page, one page-size value, a retrieval
floor under free-text queries, and an explicit zero-evidence notice. The floor
lives in the shared `query()` pipeline, so `memory retrieve` and MCP
`memory_search` inherit it. The ranker (`sort_key`, BM25) is untouched.

Two request contracts, stated once and relied on everywhere:

| request | trigger | floor | order |
|---|---|---|---|
| **find** | the free-text query tokenizes to ≥ 1 token | rows without evidence are dropped | the existing 9-key order |
| **browse** | no free text (selectors only, or none) | none | the existing 9-key order — severity leads, since lexical ties at 0 |

The surface hook (`retrieve_rows`, `src/retrieve.rs:1069`) always browses: it
passes `free_query = None`. It is unaffected by construction.

Decisions: `DEC-347` page size, `DEC-348` floor, `DEC-349` zero-evidence signal,
`DEC-350` floor placement.

<!-- doctrine:section sec-2 -->
## Page size

**One constant, one resolution site.** `SEARCH_LIMIT_DEFAULT: usize = 20` sits
beside `RETRIEVE_LIMIT_DEFAULT`/`_MAX` in `src/retrieve.rs`. It is a separate
concept, not a duplicate (STD-001): search rows are one-line metadata, while
retrieve's 5 is an agent-context budget for framed bodies. The constant replaces
the bare `20` at `src/mcp_server/tools.rs:895`.

**The command layer resolves; the engine receives a concrete `usize`.** This
copies the `Retrieve` arm (`src/memory.rs:648-658`):

- `MemoryCommand::Search`: `let limit = args.limit.unwrap_or(SEARCH_LIMIT_DEFAULT);`
  one value feeds `--page` → offset and `run_search`.
- `run_search(…, offset: usize, limit: usize, …)`: the `Option` goes away. The row
  slice becomes `ranked.iter().skip(offset).take(limit)`, so the `.min(len)` clamp
  and its comment (`:887-891`) are deleted, not re-cited. The footer passes `limit`
  as `page_size`; the `unwrap_or(shown)` at `:918` goes away.
- MCP `memory_search`: `let limit = limit.unwrap_or(SEARCH_LIMIT_DEFAULT)` whether or
  not selectors are present. The `has_selectors` branch goes away. `search_for_mcp`
  takes `limit: usize`, and the handler's `limit`/`next_offset` derive from that same
  value, not from `result.total`.
- `--limit 0` is rejected at every edge: the CLI arms (as today) **and the MCP
  handler**, which gains its own check before resolving (RV-410 `F-3`).
  `search_for_mcp` and `run_search` keep a defensive `limit == 0` bail — they are
  `pub(crate)` entry points, and the check is one line.
- MCP continuation arithmetic is overflow-safe: `next_offset =
  offset.checked_add(limit).filter(|n| *n < total)` (RV-410 `F-4`; today's
  unchecked `offset + cap` panics in debug on `offset` near `usize::MAX`).

`--page` → offset arithmetic appears in both the `Search` and `Retrieve` arms.
Extract it into one pure helper (`page_offset(page, offset, page_size) ->
Result<usize>`) that both arms call. It is small, it removes a copy, and it is the
place a unit test pins the arithmetic. **Each arm passes the page size it actually
renders**: search its resolved limit, retrieve its *capped* limit
(`.min(RETRIEVE_LIMIT_MAX)`). Today retrieve computes the offset from the uncapped
request, so `--limit 30 --page 2` starts at row 31 and rows 21-30 are never shown
(RV-410 `F-2`, a live defect this fixes).

**The continuation hint names the real next row** (RV-410 `F-1`).
`format_truncation_notice` (`src/listing.rs:926`) today computes
`next_page = offset / page_size + 2`, so an unaligned `--offset 5 --limit 20`
(rows 6-25) points at `--page 2` (rows 21-40) and repeats five rows. It becomes:
`next = offset + shown`. If `next >= total` there is no next page, so the notice
reads `{shown} of {total}; end of results` with no continuation (RV-410 `F-8`:
the final partial page must not point at itself, and today's formula points past
the end). Otherwise, if `offset % page_size == 0` the hint is
`--page {next / page_size + 1}`, else `--offset {next}`. The `offset >= total`
branch is unchanged. Output for aligned non-final pages is byte-identical to today,
so the third caller (`src/priority/render.rs:291`) only gains correctness for
unaligned offsets and final pages.

**An explicit `--limit` is honoured, uncapped**, on CLI and MCP search. Retrieve
keeps `.min(RETRIEVE_LIMIT_MAX)`.

<!-- doctrine:section sec-3 -->
## Retrieval floor

**Evidence** = `lexical > 0 || exact_key`. A scope match is not evidence: it
narrows candidates, it does not answer the text. `exact_key` is computed
independently of BM25 (`exact_key_match`, `src/retrieve.rs:293`), so a
lexical-only test would drop an exact-key hit.

**Trigger** = the free-text query tokenizes (with `lexical::tokenize`, the one lexer
BM25 also uses) to at least one token. `"!!!"` or `"  "` therefore browses rather
than returning nothing. This is a pure predicate on `QueryContext`
(`has_free_text()`), sibling to `has_scope_constraints()`.

**Placement.** In `query()` (`src/retrieve.rs:580`), after candidate assembly and
before `rank`:

```rust
let cands = if q.has_free_text() {
  cands.into_iter().filter(Candidate::has_evidence).collect()
} else {
  cands
};
rank(cands, &snap.today)
```

`Candidate::has_evidence` is a pure method. BM25 still fits its corpus statistics
over the full active set (§5.3 unchanged). The floor filters scored survivors, and
never the fit corpus.

**Effects.** `total` on every surface counts evidence rows only. Search, MCP search
and retrieve all return nothing for a zero-evidence free-text query. Retrieve's
holdback still runs after the floor, unchanged. Under the find contract, `sort_key`
keys 4-6 only break ties among rows with evidence; for browse they remain the
primary order.

**REQ-152 holds.** Held-back memories stay inspectable through browse (search with
selectors only) and `memory show`, and the floor touches neither.

<!-- doctrine:section sec-4 -->
## Zero-evidence signal

- **Table (`memory search`, `memory retrieve` text output):** when
  `has_free_text()` and the floor left **no candidates** (`ranked.is_empty()` straight
  out of `query()`, before retrieve's holdback), print one stdout line where the
  truncation notice goes:
  `no match for "<query>"; drop the query to browse by scope`. The rendering is a
  pure function in `src/listing.rs`, beside `format_truncation_notice`; the query
  text is passed through `scrub_line`.
- **`--json` and MCP:** no wire change. `rows: []`, with `total: 0` on MCP, under a
  free-text query *is* the signal (`DEC-349`).
  Evidence that exists but is entirely held back on retrieve is *not* "no match":
  that path keeps today's output, with no notice (RV-410 `F-7`; REQ-152 unchanged).
- **MCP descriptions:** the tool description (`tools.rs:295`) and the `limit`
  input-schema field (`tools.rs:309`, "no-selector default: 20", RV-410 `F-5`).
  In the tool description, replace "Requires at least one selector
  or defaults to 20-row cap" with the find/browse contract: a free-text query
  returns only rows with lexical or exact-key evidence; selectors alone browse
  in severity order; the default page is 20.

<!-- doctrine:section sec-5 -->
## Supersession and guidance

**Superseded records.** None of these can be linked, since they are doc-local
decisions inside closed designs, so this section is the record:

| record | said | now |
|---|---|---|
| `SL-008` D17 | `find` `--limit` default 5 / max 20 | search default 20, uncapped explicit (`DEC-347`); retrieve's 5/20 unchanged |
| `SL-086` D14 | unset `--limit` on find = unbounded | unset = one page (`DEC-347`) |
| `SL-131` §2 | `run_find` stays unbounded via CLI | superseded, as above |
| `SL-184` §2 | carries the unbounded clamp | the clamp is deleted |
| `SL-008` D17/D18 (reading) | a free-text query falls back to severity order | only browse does; find is floored (`DEC-348`) |

The `RV-206` `F-5` / `RV-207` `F-2` comment at `src/retrieve.rs:887-890` is
deleted together with the clamp it defended. Nothing is left to cite.

**Verification artefacts:**
- `tests/e2e_memory_sync.rs:425`: the assertion is "a shipped memory is reachable
  by its own scope". Add an explicit `--limit` above the scope's candidate count,
  with a comment, so the test asserts reachability and does not depend on the
  default page size. (The migrated `memory find` alias stays as-is.)
- MCP VT-1 (`tools.rs:2797`, no-args browse, non-empty and paginated): expected
  green unchanged. Browse is unfloored, and the no-selector default is still 20.

**Shipped guidance:**
- `retrieve-memory` SKILL §progressive disclosure: state the find/browse contract,
  and that an empty free-text result means "no match", not "empty corpus".
- `record-memory` §7: confirm surfaceability with a query that uses the memory's own
  terms (find), or with `--page` (browse). A scoped browse can now page it out.
- `dreaming` (`memory search <keyword>`): add `--limit` / `--page` when sweeping for
  link candidates.
- `using-doctrine.md` rows 23/33/48: one clause, "free text is floored; selectors
  alone browse".

**Deferred:** `CHR-171`, where `SPEC-007`/`REQ-378` still say `memory find` and
could record `DEC-347`/`DEC-348`.

<!-- doctrine:section sec-6 -->
## Code impact

| path | change |
|---|---|
| `src/retrieve.rs` | `SEARCH_LIMIT_DEFAULT`; `QueryContext::has_free_text`; `Candidate::has_evidence`; floor in `query()`; `run_search`/`search_for_mcp` take `limit: usize`; clamp + comment deleted; retrieve text output gains the zero-evidence notice |
| `src/memory.rs` | `Search` arm resolves the limit once; `page_offset` helper shared by `Search`/`Retrieve` |
| `src/listing.rs` | `format_no_match_notice` (pure) |
| `src/mcp_server/tools.rs` | `memory_search` resolves via `SEARCH_LIMIT_DEFAULT`; `has_selectors` branch deleted; `limit`/`next_offset` from the resolved value; tool description |
| `tests/e2e_memory_sync.rs` | explicit `--limit` on the reachability assertion |
| `plugins/doctrine/skills/retrieve-memory/SKILL.md`, `plugins/doctrine/skills/record-memory/SKILL.md`, `plugins/doctrine/skills/dreaming/SKILL.md`, `install/using-doctrine.md` | guidance per §Supersession and guidance |

Layering (ADR-001): the constant and predicates live in the engine module; the
command layer only resolves and forwards. No new edges.

<!-- doctrine:section sec-7 -->
## Verification

TDD, red first, behaviour-level:

- **VT pagination:** CLI search with no `--limit` returns ≤ 20 rows. `--page 2`
  without `--limit` returns rows 21-40, and the footer names page 3. The same holds
  for MCP (`limit` = 20, `next_offset` = 20 on a > 20 set, with or without
  selectors). `page_offset` gets a unit table (page none/1/N, page 0 rejected).
  Retrieve `--limit 30 --page 2` starts at row 21, the capped page size (`F-2`).
  `format_truncation_notice` gets its first unit table: aligned offsets give
  byte-identical output to today, and an unaligned `--offset 5 --limit 20` hints
  `--offset 25` (`F-1`); 32 rows at `--offset 20 --limit 20` show 12 with
  `end of results` and no hint (`F-8`). MCP: `limit: 0` is rejected at the handler (`F-3`), and
  `offset = usize::MAX` gives an empty page with `next_offset: null` and no panic
  (`F-4`).
- **VT floor:** fixtures where the query matches a subset: only matching rows are
  returned, and `total` counts them. `Candidate::has_evidence` is unit-tested
  directly (exact-key with `lexical == 0` → kept). A real exact-key hit always has
  lexical overlap, because `lex_doc` indexes the key, so the BM25 integration case
  asserts only that an exact-key query returns its memory first (`F-6`). A no-match query returns 0 rows on search, retrieve and
  MCP. A punctuation-only query browses. A selector-only query is unchanged (same
  rows, same order as today).
- **VT hook:** `retrieve_rows` output is unchanged for a path probe (the existing
  surface suite stays green unchanged).
- **VT notice:** table output for a no-match query carries the notice. A retrieve
  query whose only evidence is held back carries no notice (`F-7`); `--json`
  carries `rows: []`, byte-identical to an empty result today.
- **Behaviour preservation:** a query with lexical hits returns the same ordered
  top-N as today for the same `--limit`. The existing ranking and `sort_key`
  suites stay green unchanged.
- **VA:** the e2e reachability test and MCP VT-1 pass. The shipped-skill text is
  reviewed against §Supersession and guidance.

Gate: `doctrine check gate`.

<!-- doctrine:section sec-8 -->
## Risks and residuals

- **Silent loss of a real hit to quantization.** A positive BM25 score below
  `1/LEX_SCALE` quantizes to 0 and would be floored. Under Lucene IDF, even a term
  present in every document scores ≈ 1e-3 on this corpus (~565 docs), three orders
  above the threshold. Accepted and not engineered around. If a hit is observed
  lost, the fix is to floor on the pre-quantize score.
- **Retrieve behaviour change.** `memory retrieve --query <no-match>` goes from
  "top-N by severity" to nothing. That is the intent (`DEC-350`), but a caller that
  used free text as a soft hint over a scope loses the scope fallback. The remedy
  is to drop the query, which the notice says.
- **Scope + free text.** `--path-scope X "foo"` with no lexical match now returns
  nothing, even though every candidate matches `X`. This is deliberate
  (`DEC-348`: scope is not evidence), and the notice names the remedy.
- **Residual:** `CHR-171` (spec text drift).

