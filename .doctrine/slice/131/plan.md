# SL-131 implementation plan

## Rationale

Four phases, strictly sequential — each builds on the previous and is
independently testable. The ordering is dictated by dependency: MCP handlers
(PHASE-03) call structured helpers (PHASE-02) which call writer-parametrized
functions (PHASE-01). Integration tests (PHASE-04) need the full MCP server
wired.

### Phase boundaries

**PHASE-01 (Writer abstraction)** is the lowest-risk, highest-blast-radius
change. It touches five functions across two files but the behavioural change
is mechanical — replace stdout writes with writer writes. The CLI passes
`&mut io::stdout()` so output is byte-identical. This phase is gated on
existing tests staying green; no new tests are required beyond writer-capture
smoke tests to prove the abstraction works.

**PHASE-02 (Structured helpers)** adds the data-returning functions the MCP
handlers will call. These live in `retrieve.rs` and `memory.rs`, reusing
existing private functions (`load_query`, `query`, `listing::retain`,
`json_rows`, `sort_default`). No MCP wiring yet — the helpers are inert
until PHASE-03 connects them. This phase is the most test-intensive: every
helper gets unit tests for its gate logic (partition, lifecycle, thread
expiry, holdback, pagination).

**PHASE-03 (MCP dispatch)** wires everything together. It changes `call_tool`'s
return type, adds tool definitions and handler arms, and connects them to
PHASE-02 helpers. The double-encoding fix in `handle_tools_call` is a one-line
change but affects all 14 tools — the compat tests prove no regression.
Parse helpers (`parse_min_trust`, `parse_memory_type`, etc.) wrap errors with
the `"invalid arguments: "` prefix for correct `-32602` mapping.

**PHASE-04 (Integration + skills)** exercises the full MCP request-response
cycle against a live server. These tests prove the trust holdback works
end-to-end (a low-trust high-severity memory is suppressed by
`memory_retrieve`), confirm the consumable/backlinks enrichment in
`memory_show`, and validate the pagination envelope shape. Skill file
updates are mechanical text additions.

### Out of scope (deferred)

- `expand_graph` writer param wiring (MCP handler always passes `expand: None`)
- `memory record`/`edit`/`verify`/`tag`/`status` MCP tools (write tools)
- `inspect`/`next`/`survey`/`backlog` MCP tools (unproven per IDE-016)
- Review error mapping generalization (kept as-is)
- Backlink index caching (acceptable for <1000 memories)
