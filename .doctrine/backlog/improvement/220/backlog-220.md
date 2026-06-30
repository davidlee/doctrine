# IMP-220: rename memory find → search + adopt shared listing spine (--columns, header, colours, pagination)

Two-in-one: (1) rename `memory find` to `memory search` (better verb for
BM25-ranked free-text + scope discovery — names the read-path search idiom,
distinct from the `find`/`retrieve` agent-tool distinction), and (2) adopt the
shared listing spine so it matches every other list surface.

## Current state

- Verb is `memory find`. The agent-facing MCP tool is also `memory_find`.
- `format_find_table` in `src/retrieve.rs` builds a fixed 8-column table
  (uid, type, status, staleness, trust, severity, spec, title) with manual
  width alignment — no headers, no colours, no `comfy-table`.
- The `FindRetrieveArgs` struct has no `--columns` flag.
- Pagination exists (offset/page/limit) but renders a truncation notice via
  `format_truncation_notice` rather than a proper footer with header.
- The candidate type carries enough fields for a rich column set.

## Scope

### 1. Rename `memory find` → `memory search`

- CLI: `MemoryCommand::Find` → `MemoryCommand::Search`, `FindRetrieveArgs` stays
  shared with retrieve (the struct is just scope/filter args, the verb is in the
  variant name). Update help text.
- MCP server tool: `memory_find` → `memory_search` (tool name, handler dispatch,
  documentation comment).
- Internal: `run_find` → `run_search`, `format_find_table`/`format_find_json` →
  `format_search_table`/`format_search_json`, `MemoryFindRow` → `MemorySearchRow`.
- Tests: update all references to the old verb (`find_for_mcp` → `search_for_mcp`,
  test function names, golden strings).
- Keep `memory find` as a hidden alias (deprecated, prints a redirect notice to
  stderr).

### 2. Shared listing spine

- Define `listing::Column<Candidate>` array with columns: uid, type, status,
  staleness, trust, severity, spec, title, key, created, updated, weight,
  verification, lifespan, reviewed.
- Default visible set: `uid,type,status,staleness,trust,severity,spec,title`.
- Add `--columns` flag to the find/search args.
- Replace `format_find_table` with `listing::render_columns` (headers, comfy-table
  layout, colour).
- Colour: uid cyan (Fixed), status by value (status_hue), type by value, title
  zebra (Alternate).
- Replace `format_find_json` with `listing::json_envelope` or keep typed rows.
- Pagination: use shared footer rendering.

### 3. MCP tool mapping

- Update `doctrine_onboard` mapping table: `memory_find` → `memory_search`.
- Update boot footer if it references `memory_find`.

## Non-Goals

- No changes to `memory retrieve` (stays as-is, distinct surface).
- No schema or TOML changes.
- No changes to the BM25 ranking or query pipeline.

## Affected surface

- `src/memory.rs` — rename variant + args + CLI wiring
- `src/retrieve.rs` — rename functions + column definitions + rendering
- `src/mcp_server/tools.rs` — rename tool + handler + onboard mapping
- `tests/e2e_mcp_server.rs` — update tool names, golden values
- `src/retrieve.rs` tests — rename + new column-projection tests
