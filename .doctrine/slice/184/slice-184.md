# SL-184: Rename memory find → search + adopt shared listing spine

Two-in-one: (1) rename `memory find` to `memory search` (better verb for
BM25-ranked free-text discovery), and (2) adopt the shared listing spine so
it matches every other list surface (headers, colours, pagination, `--columns`).

Originates from IMP-220.

## Scope & Objectives

### 1. Rename `memory find` → `memory search`

- CLI: `MemoryCommand::Find` → `MemoryCommand::Search` with `alias = "find"` (silent hidden alias, no stderr notice).
- MCP server tool: `memory_find` → `memory_search` (tool name, handler,
  documentation comment, onboard mapping table).
- Internal: `run_find` → `run_search`, `format_find_table`/`format_find_json` →
  `format_search_table`/`format_search_json`, `MemoryFindRow` → `MemorySearchRow`.
- Tests: update all references to the old verb.

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
- JSON output keeps typed `MemorySearchRow` / `format_search_json` (same schema, renamed struct).
- Pagination: use shared footer rendering from listing spine.

### 3. MCP tool mapping

- Update `doctrine_onboard` mapping table: `memory_find` → `memory_search`.
- Update boot footer if it references `memory_find`.

## Non-Goals

- No changes to `memory retrieve` (stays as-is, distinct surface).
- No schema or TOML changes.
- No changes to the BM25 ranking or query pipeline.

## Affected Surface

- `src/memory.rs` — rename variant + args + CLI wiring
- `src/retrieve.rs` — rename functions + column definitions + rendering
- `src/mcp_server/tools.rs` — rename tool + handler + onboard mapping
- `tests/e2e_mcp_server.rs` — update tool names, golden values
- `src/retrieve.rs` tests — rename + new column-projection tests

## Risks & Assumptions

- The shared listing spine (`listing.rs`) already has a column model that
  Candidate can adopt. Verify `listing::Column<Candidate>` pre-materialises
  the row (see memory `mem_019eb21e007f7d10838e2166c7a0fa2d`).
- comfy-table custom_styling requires `force_no_tty` (see
  `mem_019ebc3cfa0a7572992f761cfab49885`).
- Each list surface must call `listing::validate_statuses` itself
  (`mem_019ebb51fd8478d09a7bdca8797e25ed`).
- MCP handler test patterns for seeded memories documented in
  `mem_019ee83c84487d51b9d584a8f13ddb92`.

## Open Questions

- Should JSON output adopt `listing::json_envelope` or stay as typed rows?
  (Design-time decision — likely stay typed for backward compat.)
- Deprecation period for `memory find` alias: how many releases before removal?

## Verification / Closure Intent

- CLI: `doctrine memory search --help` shows the new verb.
- `doctrine memory find` prints deprecation notice and delegates.
- Table output uses shared spine (headers, colours, comfy-table).
- `--columns` flag filters visible columns.
- Pagination footer matches other list surfaces.
- MCP tool `memory_search` works; `memory_find` removed from onboard.
- All existing tests pass (renamed).
- E2E MCP tests updated.
