# SL-171: `doctrine next` read-surface improvements

## Problem

`doctrine next` is the primary "what should I work on?" advisory surface, but its
column selection is hardcoded and skips authored facet data that would help a user
triage.

1. **No `--columns` flag.** Unlike `memory list` and `backlog list`, `next`'s
   columns are immutable constants (`NEXT_COLS` in `src/priority/render.rs`). The
   `listing` crate already supports `select_columns` + `RenderOpts`; the CLI arg
   just isn't wired.

2. **Missing facet columns.** `kind` takes a column but is redundant with the
   `id` prefix. `estimate` and `value` are authored on entities and loaded in
   the priority graph, but never surfaced in `next`. Surprise: IMP-120 (scored
   1.9) looks identical to IMP-195 (scored 1.3) — the value/estimate that drove
   the score is invisible.

3. **`unblocks` column wastes space.** The blocking count is almost always 0
   (see current output). When non-zero it's instructional, but a whole column
   for a number that's zero 90%+ of the time is poor use of width. It should be
   an inline annotation (e.g. `⛓ 2` suffixed to `id` or shown only when > 0).

4. **No pagination / default limit.** The current `next` dumps every actionable
   item (currently ~60 rows). `memory retrieve` has `--limit`/`--offset`/`--page`;
   `next` should have at least `--limit` with a sensible default (20?) and a
   footer hint when rows were truncated.

## Scope

### 1. Wire `--columns`

- Add `--columns` to the `Next` CLI variant in `src/commands/cli.rs`.
- Thread through `run_next` → `next_human`.
- Use the existing `listing::select_columns` with a `NEXT_DEFAULT` (already
  declared, dead-code-expected).
- Unknown column: clean error with available set (standard `select_columns` UX).

### 2. Default columns: `id status score estimate value unblocks title`

- Add `estimate` and `value` columns to `NEXT_COLS`.
- Default set changes from `["id", "kind", "status", "score", "unblocks", "title"]`
  to `["id", "status", "score", "estimate", "value", "unblocks", "title"]`.
- `kind` drops from defaults (redundant with prefix).

### 3. Raw facets in the priority graph

- Add `estimate: Option<EstimateFacet>` and `value: Option<ValueFacet>` to
  `NodeAttr` (currently only carries `base_score`).
- Populate them during graph build from the `ScannedEntity` (already carries
  `estimate` and `value`).
- Render: estimate shows `L–U` (e.g. `1–3`), value shows the magnitude (e.g.
  `5.0`). Absent → `·` (middle dot, the listing convention).

### 4. Unblocks as inline annotation

- Remove the standalone `unblocks` column.
- Annotate the `id` cell: `IMP-120 ⛓2` when blocking > 0, bare `IMP-120` when 0.
- Candidate approach: a `paint` closure on the `id` column that appends the
  annotation; or fold it into the cell closure.
- The `blocking` field on `NextRow` is already populated — just render it
  differently.

### 5. Default limit + pagination

- Add `--limit <N>` (default: 20) and `--offset <N>` (default: 0) to the CLI.
- When the actual row count exceeds the displayed count, print a footer:
  `showing 20 of 63; use --offset 20 for next page`.
- `--limit 0` means "no cap" (matches the current behaviour when limit is
  absent — the `all` sentinel).
- On `--json`: no pagination — JSON always returns the full set (the listing
  precedent: `--columns` is no-op under `--json`).

## Out of scope

- `--page` sugar (MVP: `--offset` is sufficient).
- Changing `survey` or `blockers` columns.
- `next --format json` already includes `blocking` in the JSON — no JSON
  schema change needed.
- No estimate/value columns in `survey` (separate follow-up).

## Terrain

| File | Change |
|------|--------|
| `src/commands/cli.rs` | `Next` variant: add `--columns`, `--limit`, `--offset` |
| `src/priority/mod.rs` | `run_next`: accept + thread new args |
| `src/priority/render.rs` | `NEXT_COLS`: add estimate/value; default set; `next_human`: column selection + pagination footer |
| `src/priority/surface.rs` | `next()`: carry estimate/value facets into `NextRow` |
| `src/priority/view.rs` | `NextRow`: add `estimate`, `value` fields |
| `src/priority/graph.rs` | `NodeAttr`: add `estimate`, `value` fields; populate in build |
