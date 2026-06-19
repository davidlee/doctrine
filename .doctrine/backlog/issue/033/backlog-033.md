# ISS-033: review_list MCP tool 100% broken — reuses serde-defaultless ListArgs + schema advertises nonexistent facet/target filters

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

Every `mcp__doctrine__review_list` call returns `MCP error -32602: Invalid
params`, for every argument shape — empty `{}`, `{"status":["active"]}`,
`{"facet":"reconciliation"}`, `{"target":"SL-066"}`. The tool has never
succeeded. The sibling read verbs (`review_show`, `review_status`) work clean.
The CLI `doctrine review list` works clean — this is MCP-surface-only.

## Root cause (two bugs, `src/mcp_server/tools.rs:271`)

1. **Wrong arg struct across the serde seam.** The handler does
   `serde_json::from_value::<crate::listing::ListArgs>(arguments)`. `ListArgs`
   (`src/listing.rs:124`) is the clap-mirror bundle; it derives `Deserialize`
   with **no `#[serde(default)]`**, so its non-`Option` fields
   (`case_insensitive`, `status`, `tags`, `all`, `format`, `json`, `render`)
   are all *required* by serde. No JSON the schema permits carries them, so
   deserialize fails with `missing field \`case_insensitive\`` before any work
   runs. `{}` can never deserialize.

2. **Schema advertises filters that do not exist.** The MCP input_schema
   (`tools.rs:62-68`) declares `facet` / `target` / `status`. `ListArgs` has no
   `facet` or `target` field at all — the real filter axes are `substr` /
   `regexp` / `status` / `tags`. Even past bug 1, `facet`/`target` are silently
   dropped.

The contrast is the diagnosis: every other read verb hand-extracts fields
(`ExtractFields::from_value`, missing-tolerant). `review_list` is the lone verb
that reuses the clap struct across the serde seam — clap-free (A-3) but not
serde-defaulted.

## Sketch (fix)

Drop `ListArgs`-via-serde from the `review_list` arm; hand-extract like
`review_show`:

- Pull `status` as an optional `Vec<String>` (default `[]`), `substr`/`regexp`
  as optional strings, `tags` as optional `Vec<String>`.
- Build `ListArgs { status, substr, tags, ..Default::default() }` and pass to
  `review::run_list`.
- Reconcile the input_schema to the **actual** filter axes: replace
  `facet`/`target` with `substr` (substring on slug+title), `regexp`, `tags`.
  Keep `status` (array of `active` | `done`). `facet`/`target` are not
  `run_list` filters — `list_rows`/`Filter` carries no such axis — so they must
  leave the schema, not be wired in.
- Optional hardening: add `#[serde(default)]` to `ListArgs` so the struct is
  serde-safe regardless of caller. Cheap insurance, but the hand-extract is the
  primary fix (matches the established verb pattern).

## Verification

- Red: a `tools.rs` test that round-trips `review_list` `{}` through
  `call_tool` and asserts `Ok(ReviewOutput::Listed { .. })`, plus one with
  `{"status":["done"]}`. Currently both error.
- Green: the above pass; MCP `review_list` returns the table for `{}` and for a
  `status` filter.
- Regression: `review_show` / `review_status` untouched and still green.
