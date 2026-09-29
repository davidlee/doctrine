## The pattern

A read surface that ranks but does not page returns the whole surviving set, so its
**row count is invariant to the query**. Three unrelated queries, one of them literal
garbage, all report the same total. The ordering underneath varies correctly — the
size does not.

A reader who probes the surface by comparing result counts therefore concludes the
query is being ignored, and goes looking for the defect in the ranker. That is what
happened to `doctrine memory search` (`ISS-465`): five friction observation records
and one issue were filed against "BM25 ranking is broken" when the actual defect was
a missing `--limit` default (`--offset` has one; `--limit` does not). The ranker was
never at fault.

## How to apply it

- When triaging a "search ignores my query" report, **compare the top rows, not the
  counts.** Count equality across unrelated queries is expected of any unpaged
  listing and tells you nothing about scoring.
- Before believing a ranking defect, check whether the result set is capped at all:
  `--limit 5` and see whether the total changes. A surface reporting "5 of 565" only
  when asked is an unpaged surface by default.
- The inverse also holds: a *zero-evidence* query legitimately returns an
  identical result set on a ranker that falls through to severity/trust tiebreaks.
  Distinct symptom, same data, opposite root cause. Separate "no document matched"
  from "matched, ranked low" before writing either up.

Separately, the unpaged default is its own cost: a single triage call dumps the whole
corpus (565 rows) into a context window, burying the ranking it was meant to expose.

## Where

- `ISS-465` — the re-scoped defect (no default page, no relevance floor, no zero-hit
  signal).
- `src/retrieve.rs::sort_key` — the 9-key total order; BM25 is key 2, severity keys
  4–6. Keys below the tie determine the fallback order.
- `src/search.rs`, `src/observation/query.rs` — the sibling read surfaces; the
  observation matcher is Boolean and unranked, so it has the count symptom without
  even a ranker to blame.

## Status of the original case

`SL-275` fixed `memory search`: an unset `--limit` is now a 20-row page, free text
is floored to rows with lexical/exact-key evidence, and a zero-evidence query says
`no match for "<q>"` (JSON/MCP: empty `rows`). The pattern still applies to any
other ranked-but-unpaged surface.
