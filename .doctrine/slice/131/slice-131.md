# SL-131: MCP memory retrieval and find tools for agent harnesses

## Context

Doctrine runs a hand-rolled MCP stdio server (`src/mcp_server/`) that currently
exposes only **review** (`review_new`, `review_list`, `review_show`,
`review_raise`, `review_dispose`, `review_verify`, `review_contest`,
`review_withdraw`, `review_status`, `review_prime`). The memory subsystem
(`SPEC-007`, `PRD-004`) provides a rich scope-aware durable-knowledge store with
`find`, `retrieve`, `show`, and `list` verbs — but these are CLI-only, unavailable
to agents via MCP.

**IDE-012** calls for a read-only doctrine memory retrieval surface for agent
harnesses. IDE-016 validates that review-adjacent MCP reads are proven valuable and
memory retrieval is the next candidate. The existing MCP infra (JSON-RPC 2.0 stdio
server, tool dispatch, error mapping) provides the seam — this slice rides it.

## Scope & Objectives

Add MCP tools that expose **read-only** memory operations to agent harnesses:

### In scope

1. **`memory_find`** MCP tool — exposes `retrieve::run_find` with scope probes
   (path, glob, command, tag), free-text query, type/status/lifespan filters,
   pagination (offset/limit). Returns ranked results as JSON with trust+severity
   visibility (the holdback-exempt `find` surface — D8).
2. **`memory_retrieve`** MCP tool — exposes `retrieve::run_retrieve` with the same
   scope + filter surface plus `--min-trust` and `--limit`/`--offset`. Returns
   security-framed data blocks (trust holdback enforced — B7/D8).
3. **`memory_show`** MCP tool — resolve a memory by uid or key and return its
   header + body-as-data (maps to `doctrine memory show`).
4. **`memory_list`** MCP tool — list memories, newest first, AND-filtered on the
   shared spine (maps to `doctrine memory list`).

### Out of scope

- **Write tools** — no `record`, `edit`, `verify`, `tag`, `status`, `sync`,
  `resolve-links`, `backlinks` MCP tools. This slice is read-only per IDE-012.
- **Other doctrine read surfaces** — `inspect`, `next`, `survey`, `backlog` MCP
  tools deferred per IDE-016 (unproven).
- **Review error mapping generalization** — the existing `map_review_error` is kept
  as-is; new memory errors map separately or through a shared error trait (deferred
  to a later slice if the pattern repeats).
- **Coordination with knowledge records** — `doctrine knowledge` MCP tools are
  separate (records are not memories).
- **`retrieve` graph expansion** (the `--expand` flag) — deferred; the basic
  retrieve surface is sufficient initially.
- **Protocol changes** — no changes to the JSON-RPC 2.0 protocol, transport, or
  initialization handshake.

## Affected surface

- `src/mcp_server/tools.rs` — add memory tool definitions, handler dispatch arms,
  and the `call_tool` match arms that call into `retrieve::run_*` and a Show/List
  wrapper.
- `src/mcp_server/mod.rs` — no change (transport/server loop is generic).
- `src/mcp_server/protocol.rs` — may need a `MemoryOutput` variant or the existing
  `McpToolResult` is reused with JSON-text via `serde_json`.
- `src/retrieve.rs` — no change (the MCP layer adapts the existing run functions,
  capturing `io::stdout()` output programmatically rather than printing it).
- `src/memory.rs` — may need a thin adapter to return output as a `String`/`Value`
  instead of writing to stdout (or capture stdout).

## Risks and assumptions

- **Assumption:** The existing `run_find`/`run_retrieve` print to stdout. The MCP
  handler captures that output via a `stdout` redirect or by adding a
  `String`-returning variant. Prefer adding a `run_find_string() /
  run_retrieve_string()` or a capture helper to avoid stdout pollution in the MCP
  loop (which shares `io::stdout()` with the JSON-RPC response).
- **Risk:** stdout capture from `run_*` functions is fragile — they use
  `write!(io::stdout())` internally. The MCP server's own `io::stdout()` is
  concurrently locked by `BufWriter` in the serve loop. The cleanest path is to
  refactor the `run_*` functions to accept an `impl Write` or return `String`, but
  that touches `src/retrieve.rs` which is out of scope unless necessary.
  **Mitigation:** add programmatic variants (`run_find_value` / `run_retrieve_value`
  returning `serde_json::Value`) that compose the same pipeline logic without
  writing to stdout, then delegate the CLI variant to them. This avoids touching
  the existing `run_*` shape.
- **Risk:** MCP output format differs from CLI table/JSON format — the MCP layer
  will always return JSON (as `McpToolResult::text`). This is fine; the CLI format
  flag is not exposed in the MCP tool schema.
- **Assumption:** The MCP `ReviewOutput` enum is not reusable for memory — a new
  `MemoryOutput` or generic `TextOutput` variant is needed. Simpler: sidestep the
  `ReviewOutput` type entirely and return `String` JSON directly from the new
  handler.

## Verification / closure intent

"Done" means:

1. An agent can call `memory_find` via MCP with scope probes and receive ranked,
   JSON-formatted results with trust+severity visibility.
2. An agent can call `memory_retrieve` via MCP with the same probes and receive
   trust-holdback-respected, security-framed results.
3. An agent can call `memory_show` with a uid and receive the memory's full
   content.
4. An agent can call `memory_list` with no arguments and receive the full memory
   index.
5. Existing review MCP tools are untouched — all pass.
6. `cargo clippy` zero warnings.
7. `just check` green.
8. A manual or integration test confirms two MCP request-response cycles (e.g.
   `memory_find` + `memory_list`) against the doctrine repo's own cache-warm corpus.
