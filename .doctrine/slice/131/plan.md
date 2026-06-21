# SL-131 implementation plan

## Rationale

Five phases, strictly sequential — each builds on the previous and is
independently testable.

### Phase boundaries

**PHASE-01 (Writer abstraction)** is the lowest-risk, highest-blast-radius
change. It touches five functions across two files but the behavioural change
is mechanical — replace stdout writes with writer writes. The CLI passes
`&mut io::stdout()` so output is byte-identical. This phase is gated on
existing tests staying green; writer-capture smoke tests prove the abstraction
works.

**PHASE-02 (Structured helpers)** adds the data-returning functions the MCP
handlers will call. These live in `retrieve.rs` and `memory.rs`, reusing
existing private functions (`load_query`, `query`, `listing::retain`,
`json_rows`, `sort_default`). No MCP wiring yet — the helpers are inert
until PHASE-03B connects them. This phase is the most test-intensive: every
helper gets unit tests for its gate logic (partition, lifecycle, thread
expiry, holdback, pagination).

**PHASE-03 (MCP dispatch refactor)** is the narrowest, highest-risk change.
It changes `call_tool`'s return type from `ReviewOutput` to `String`, wraps
the 10 review arms, and fixes the `handle_tools_call` double-encoding trapdoor.
This is split from handler wiring to keep blast radius small — if the
refactor is wrong, ALL 14 MCP tools break. The tools() vec grows from 10 to
14 (adding memory tool schemas) but no new handler arms are connected yet.
VT-1 (byte-identical review JSON) and VT-2 (content[0].text parses as JSON
object) together prove the double-encoding fix is correct.

**PHASE-04 (Memory handler wiring)** adds the 4 handler arms. By this point
the dispatch refactor is proven correct by PHASE-03's tests, so any
regression in the review tools is a handler implementation bug, not a
dispatch-level bug. Mutual exclusivity and min_trust validation are enforced
in the handler before delegating to PHASE-02 helpers.

**PHASE-05 (Integration + skills)** exercises the full MCP request-response
cycle against a live server, reusing the existing `tests/e2e_mcp_server.rs`
infrastructure (spawn `doctrine serve --mcp --path <root>`, drive
initialize + tools/call, read responses via `tool_result_text()`). These
tests prove the trust holdback works end-to-end, confirm the
consumable/backlinks enrichment, and validate the pagination envelope shape.
Skill file updates are mechanical text additions.

### Why PHASE-03 was split

PHASE-03 as originally conceived was overloaded: dispatch refactor,
double-encoding fix, 14 tool definitions, parse helpers, AND 4 memory
handlers in one phase. The dispatch refactor affects all 14 tools — if the
return-type change or handle_tools_call fix is wrong, every review tool
breaks. Splitting into PHASE-03 (refactor + definitions) and PHASE-04
(handler wiring) lets us:

1. Prove the refactor is correct (PHASE-03) with the existing 10 review
   tools as witnesses.
2. Add memory handlers (PHASE-04) on a proven foundation, with review
   compat already guaranteed.

### Out of scope (deferred)

- `expand_graph` writer param wiring (MCP handler always passes `expand: None`)
- `memory record`/`edit`/`verify`/`tag`/`status` MCP tools (write tools)
- `inspect`/`next`/`survey`/`backlog` MCP tools (unproven per IDE-016)
- Review error mapping generalization (kept as-is)
- Backlink index caching (acceptable for <1000 memories)
