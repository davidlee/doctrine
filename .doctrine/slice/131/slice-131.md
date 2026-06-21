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

- `src/mcp_server/tools.rs` — add memory tool definitions and handler dispatch
  arms; change `call_tool` return type from `Result<ReviewOutput>` to
  `Result<String>`; wrap 10 review arms with `.map(|o| serde_json::to_string(&o)?)`.
- `src/mcp_server/mod.rs` — no change (transport/server loop is generic).
- `src/mcp_server/protocol.rs` — no change (McpToolResult reused as-is).
- `src/retrieve.rs` — add `writer: &mut impl Write` param to `run_find`,
  `run_retrieve`, `expand_graph`; replace `write!(io::stdout(), …)` →
  `write!(writer, …)`.
- `src/memory.rs` — add `writer: &mut impl Write` param to `run_show`, `run_list`;
  replace `write!(io::stdout(), …)` → `write!(writer, …)`.
- `src/main.rs` — 4 CLI call sites pass `&mut io::stdout()` as first arg.

## Risks and assumptions

- **Decision (design):** refactor `run_find`, `run_retrieve`, `run_show`, `run_list`
  to accept `writer: &mut impl Write` — the MCP handler passes `&mut Vec<u8>`,
  CLI passes `&mut io::stdout()`. MCP tools use structured helpers
  (`find_for_mcp`, `list_for_mcp`, `retrieve_reference`) that share the existing
  `load_query` → `query` pipeline rather than post-processing rendered strings.
- **Risk:** the `ExtractFields` helper in `tools.rs` lacks `opt_bool_field`. Must add
  it for the `include_draft` flag.
- **Risk:** `call_tool` return type changes to `String`. The `handle_tools_call`
  handler must NOT re-serialize the string (`serde_json::to_string` on an already-
  serialized JSON string produces double-encoding) — use `McpToolResult::text(out)`
  directly.
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
