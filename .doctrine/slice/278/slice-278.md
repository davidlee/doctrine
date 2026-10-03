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

Designed views for humans and agents. Settled in the design run's inquiry
(DEC-361..DEC-370):

```
depth (how much)              selector / filters (which)       format
───────────────────────       ──────────────────────────       ──────
index          (default)  ×   -f/--finding 3,5,7          ×    table (default)
--full  (+ brief body)        --open  -s/--status              json
                              -S/--severity  (ANDed)
```

1. **One view** — an owned, serialisable `ReviewView` / `FindingView` in
   `review_ledger`; table, CLI JSON and MCP render it (DEC-361).
2. **`review show` depth** — index by default; `-f` shows the selected findings
   in full; `--full` shows every finding in full plus the brief (DEC-362).
3. **Default render** — header with rounds and the concluded marker; an
   outstanding line counting findings by severity and turn; the index, with a
   route column when any finding has a route; a one-line brief hint; `BLOCKER`
   and `MAJOR` in capitals in table output (DEC-367). `doctrine show RV-N`
   prints the same (DEC-364).
4. **Filters** — one finding predicate in `review_ledger`, shared by `show` and
   the census (DEC-363).
5. **JSON** — identical inner `ReviewView` for CLI (`{kind, review}`) and MCP
   (`{"Showed": …}`), with a `depth` field; `review.finding` stops leaking
   (DEC-368). Ledger findings have no `summary` field; IMP-476's empty
   `summary` is the design-run finding `D6` deletes (research X5).
6. **MCP** — `review_show` takes the CLI's projection arguments and defaults to
   index; `view` and `format` retire (DEC-369).
7. **Census** — `doctrine review findings` plus MCP `review_findings`; the bare
   verb shows open findings across the corpus (DEC-370).
8. **`--facet`** on `review list` and the census (DEC-365). Detail is never
   truncated (DEC-366).

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

Q1-Q4 were settled by the design inquiry (DEC-362, DEC-368, DEC-366, DEC-370).

- **R1 — two shapes forever.** Mitigated by one struct and one equality test
  (DEC-368).
- **R2 — breaking CLI JSON and MCP defaults.** Callers that read
  `review.finding` or rely on MCP's full default move; research lists them
  (`research.md` § callers).
- **A1** — `SL-268`'s journal and `route` are reachable from the ledger module
  without schema change (research fact 4).

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
