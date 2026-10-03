# Review read surface

## Context

`review show RV-NNN` renders a header, the finding index (id, severity, status,
disposition, title; `IMP-490`) and the brief. A finding's `detail`, `response`,
`route` and turn journal (`SL-268`) — the reasoning a reader needs to judge it —
are reachable only through `--json` or the raw ledger TOML. `show` has no flag
beyond `--format`/`--json`. So "see `doctrine show RV-NNN`" tells a reader
*what* a finding is, never *why*.

The JSON is no better for agents: CLI `--json` leaks the TOML array key
(`review.finding`) and dumps everything; MCP `review_show` returns a different
shape (`findings`). There is no corpus-wide view over findings at all.

This is RFC-032 (review ledger effectiveness) slice 2, "Read surface". Its
design proposal, `D5` in `.doctrine/rfc/032/decision-frontier.md`, already
specifies the shape: one serialisable `ReviewView` built in the ledger module,
and every surface a render of it. Slice 1 (`SL-268`, ledger v2) landed the
engine-tier module (`D4`) and the turn journal (`D1`) this reads. The proposal
is evidence, not canon; this slice's design locks it.

## Scope & Objectives

Designed views for humans and agents, on three orthogonal axes:

```
projection (what)          format (how)       filter (which)
────────────────────       ────────────       ─────────────────────
index        (default)  ×  table (default) ×  --open (non-terminal)
--finding F-n (repeat)     json               --status <s>
--full                                        --severity <s>
```

1. **One projection.** A serialisable `ReviewView` (finding view included) in
   the ledger tier; table, CLI JSON and MCP are renders of it.
2. **Projections on `review show`:**
   - default — header (status, await, facet, target, rounds, concluded) + one
     line per finding + the brief (as today);
   - `--finding F-n` — that finding in full: severity, status, disposition,
     route, title, detail, response, turn journal (no `anchor` until RFC-032
     `D3`, slice 3);
   - `--full` — brief, synthesis, every finding in full.
3. **Filters** shared by `show` and the census: `--open`, `--status`,
   `--severity`.
4. **JSON is the projection, not the TOML** (`IMP-476`). CLI `--json` and MCP
   `review_show` emit one shape, `findings` plural. `--json` defaults to the
   index projection, not the full dump. Breaking change to CLI JSON, accepted
   pre-1.0 (`D5`). Ledger findings have no `summary` field; IMP-476's empty
   `summary` is the design-run finding `D6` deletes (research X5).
5. **MCP parity** — `review_show` takes the same projection and filter
   arguments.
6. **Census** (`IMP-477`) — `doctrine review findings`: one row per finding
   across the corpus (review id, facet, target, finding columns), standard
   listing machinery, `--format json`, the shared filters. No aggregation verb.

Backlog fulfilled: `IMP-494` (entry point), `IMP-476`, `IMP-477`, and the
remainder of `IMP-475`.

## Non-Goals

- **`D14` — RV as a relation target (`IMP-480`).** RFC-032 places it in slice
  2, but it changes the relation engine (`RELATION_RULES`,
  `partition::status_class`), not the read surface. Carved to SL-279; DEC-318
  amended to name it owner (user direction, 2026-10-03).
- Schema or write-verb changes to the ledger. This slice reads what `SL-268`
  wrote.
- Design-run binding (`D3`/`D6`, slice 3) and identity/locus (`D7`/`D9`,
  slice 4).
- Finding-granular citation (`RV-N#F-3`) — rejected by `D14`.

## Affected surface

- `src/review/read.rs` — today's `ReviewView` and table render.
- `src/review/cli.rs` — `show` / `list` args; new `findings` verb.
- `src/review_ledger/` — home of the projection (`D4`/`D5`).
- `src/mcp_server/tools.rs` — `review_show` (and `review_list`) shapes.
- Skills and reference docs citing `review show` or its JSON shape.
- SPEC-032 (review ledger tech spec) — read-surface section.

## Risks, assumptions, open questions

- **Q1 — `--full` vs `D5`'s rejected opt-in.** `D5` rejects a `--findings`
  flag because the default should be right. `--full` is a larger opt-in render
  on top of a correct default. Assumed compatible; confirm at design.
- **Q2 — default JSON projection.** Index-by-default may break callers that
  read `detail` from today's dump. Inventory callers (skills, scripts, MCP
  consumers) at research.
- **Q3 — `detail` carrying multi-KB entity dumps** (`IMP-476`): render
  verbatim, truncate in the index, or stop writing them? Writing is out of
  scope; rendering is in.
- **Q4 — census verb name.** `review findings` vs the top-level `findings`
  report (priority findings) — check for collision or confusion.
- **R1 — two shapes forever.** Mitigated by building MCP and CLI from the same
  struct and testing the JSON once.
- **A1** — `SL-268`'s journal and `route` are reachable from the ledger module
  without schema change.

## Verification / closure intent

- Table and JSON renders of each projection tested over fixture ledgers
  (behaviour: which fields appear at which projection).
- One JSON shape test covering CLI `--json` and MCP `review_show`.
- Filters tested on `show` and the census against the same fixtures.
- Token-cost check: the default index for a ~5-finding ledger stays near the
  ~200-token budget `D5` cites.
- Skills/docs that cite the old shape updated; SPEC-032 read-surface section
  reconciled through a REV (revision) at reconcile.

## Summary

## Follow-Ups

- SL-279 — `D14` / `IMP-480` (RV and REC as relation targets).
