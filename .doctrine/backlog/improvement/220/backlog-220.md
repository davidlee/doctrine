# IMP-220: memory find: adopt shared listing spine (--columns, header, colours, pagination)

`doctrine memory find` currently renders a hand-rolled plain-text table without
column projection, headers, colour, or a truncation footer — unlike every other
list surface (`backlog list`, `slice list`, `spec list`, etc.) which use the
shared listing spine (`render_columns` + `--columns` + `select_columns`).

## Current state

- `format_find_table` in `src/retrieve.rs` builds a fixed 8-column table
  (uid, type, status, staleness, trust, severity, spec, title) with manual
  width alignment — no headers, no colours, no `comfy-table`.
- The `FindRetrieveArgs` struct has no `--columns` flag.
- Pagination exists (offset/page/limit) but renders a truncation notice via
  `format_truncation_notice` rather than a proper footer with header.
- The truncation notice only appears in table mode.
- The candidate type carries `memory` (a `&Memory`), `scope_match`, `staleness`,
  `lexical`, and `exact_key` — enough fields for a rich column set.

## Desired behaviour

- `memory find` uses `listing::Column<Candidate>` definitions and
  `listing::render_columns` for table output, matching the shared pattern.
- `--columns` flag: `--columns uid,type,status,staleness,trust,severity,spec,
  title,key,created,updated,weight,relations,verification,lifespan,reviewed`
  (sensible default).
- Table headers (bold when colour enabled).
- ANSI colour: uid in cyan (Fixed), status by value (status_hue), type by value,
  title with zebra stripes (Alternate).
- Pagination footer with header re-rendered on continuation.
- Rename to `memory search` for consistency with the `search` idiom? (Discuss.)

## Affected surface

- `src/retrieve.rs`: replace `format_find_table` + `format_find_json` with
  column-based rendering, define `CandidateColumn` or similar.
- `src/memory.rs`: add `--columns` to `FindRetrieveArgs`, wire into the caller.
- Tests in `src/retrieve.rs`: update golden outputs, add column-projection tests.
