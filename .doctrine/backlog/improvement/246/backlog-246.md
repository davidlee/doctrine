# IMP-246: Surface estimate/value/risk columns in backlog list table and JSON output

## Context

`doctrine backlog show` has a `facet` field that is always `null` — the TOML
`[estimate]`/`[value]` sections are not surfaced. IMP-183 covers wiring those
into the `show` and `inspect` paths for all estimable kinds.

This item is the **list surface** complement: `doctrine backlog list` columns
and its `--json` output should also carry estimate/value (and risk, where
applicable).

## What's missing

- `doctrine backlog list --columns` does not offer `estimate`, `value`, or
  `risk` in the available column set (only id, kind, status, slug, tags, title).
- `doctrine backlog list --json` omits facet data entirely from its rows.

## Ask

1. Add `estimate`, `value`, `risk` to the `backlog list --columns` available set.
2. Include estimate/value/risk in `backlog list --json` row output.
3. Ensure the columns and JSON fields are present but null/empty when unset
   (mirroring how `slice show` handles null estimate/value).

## Dependencies

- IMP-183 (surface facets in `show`/`inspect`) — the `show` path should land
  first; `list` columns read from the same deserialisation.
