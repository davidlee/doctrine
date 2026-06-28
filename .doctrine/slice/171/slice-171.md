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

3. **`unblocks` column wastes space — and is redundant.** The blocking count is
   almost always 0 (see current output), and where non-zero its signal (downstream
   leverage) is already folded into the SL-133 score's `leverage`/`optionality`
   dimensions — which `next` is *already sorted by*. The precise dependent set lives
   in `blockers <id>`, `explain <id>`, and `next --json` (`blocking`). The column
   earns no width; **drop it entirely** (design D1).

4. **No pagination / default limit.** The current `next` dumps every actionable
   item (currently ~60 rows). `next` adopts `memory`'s pagination triple
   (`--limit`/`--offset`/`--page`) with a default `--limit 20` and a truncation
   footer.

5. **Authored `tags` are invisible.** Like `estimate`/`value`, tags are authored
   facet data; `next` should surface them via the house `default_with_tags`
   convention (shown iff any surfaced row is tagged).

## Scope

### 1. Wire `--columns`

- Add `--columns` to the `Next` CLI variant in `src/commands/cli.rs`.
- Thread through `run_next` → `next_human`.
- Use the existing `listing::select_columns` with a `NEXT_DEFAULT` (already
  declared, dead-code-expected).
- Unknown column: clean error with available set (standard `select_columns` UX).

### 2. Columns & defaults

- `NEXT_COLS = [id, kind, status, score, estimate, value, tags, title]` —
  **`unblocks` removed**; `estimate`/`value`/`tags` added.
- `NEXT_DEFAULT = ["id", "status", "score", "estimate", "value", "title"]`;
  effective default = `default_with_tags(NEXT_DEFAULT, any_tagged)`.
- `kind` drops from defaults (redundant with prefix), stays selectable.
- Facet cells (compact, unitless, pure `fn`): estimate `L–U` (e.g. `3.2–4.8`),
  value `{:.1}` (e.g. `5.0`), tags joined `, ` (`paint_tag`), absent → `·`.

### 3. Facets in the priority graph

- `NodeAttr` carries `facets: EntityFacets` (the shared estimate/value/risk/tags
  projection); populated at build-3c from the scanned entity.
- `NextRow` carries the render subset `estimate`/`value`/`tags`; `surface::next()`
  projects from `NodeAttr.facets`. A future `risk` column extends `NextRow` only.

### 4. Pagination

- `--limit <N>` (default 20), `--offset <N>` (default 0), `--page <N>` (1-based
  sugar, `conflicts_with offset`) — mirroring `memory`. `--page 0` → error;
  `--limit 0 --page N` → error (`--page requires a positive --limit`).
- `--limit 0` ⇒ uncapped ⇒ no footer. Footer (table-mode, only when `shown < total`)
  via the truncation helper **lifted** from `retrieve.rs` to shared `listing`.
- `--json`: full set, no pagination, no column projection (listing precedent).

## Out of scope

- Changing `survey` or `blockers` columns.
- No `risk` column yet (future `NextRow`-only extension).
- No estimate/value/tags columns in `survey` (separate follow-up).
- No JSON schema change (`blocking` already present).

## Terrain

| File | Change |
|------|--------|
| `src/commands/cli.rs` | `Next` variant: add `--columns`, `--limit`, `--offset`, `--page`; resolve page→offset |
| `src/priority/mod.rs` | `run_next`: accept + thread columns/limit/offset |
| `src/priority/render.rs` | `NEXT_COLS` (drop unblocks; add estimate/value/tags); `NEXT_DEFAULT`; `next_human`: select_columns + default_with_tags + pagination footer; `NEXT_LIMIT_DEFAULT` |
| `src/priority/surface.rs` | `next()`: project facets into `NextRow` |
| `src/priority/view.rs` | `NextRow`: add `estimate`, `value`, `tags` |
| `src/priority/graph.rs` | `NodeAttr`: add `facets: EntityFacets`; populate at 3c |
| `src/listing.rs` | lift `format_truncation_notice` (shared) |
| `src/retrieve.rs` | re-point call sites to lifted helper |
