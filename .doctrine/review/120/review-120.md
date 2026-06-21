# Review RV-120 — design of SL-131

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject:** SL-131 design (`design.md`) — MCP memory retrieval and find tools for agent harnesses.

**Artifacts reviewed:**
- `.doctrine/slice/131/design.md` (design)
- `.doctrine/slice/131/slice-131.md` (scope)
- `src/mcp_server/tools.rs` (current MCP dispatch)
- `src/retrieve.rs` (current run_find/run_retrieve/expand_graph)
- `src/memory.rs` (current run_show/run_list/backlinks)
- `src/links.rs` (current backlinks_index)
- `SPEC-007` (tech), `PRD-004` (product), `ADR-007` (review)

**Lines of attack:**
1. Completeness: does the design specify every concrete parameter, type conversion, default value, and error code needed for implementation?
2. Fidelity to existing APIs: does the design accurately describe functions that exist, with correct signatures and module paths?
3. Scope-design coherence: does the design address every constraint, risk, and mitigation from the scope document?
4. Gap analysis: what paths/parameters/error cases are missing from the described implementation?
5. Architectural fidelity: does the design respect SPEC-007's contracts (holdback, render, deterministic ranking)?

## Synthesis

### Overall: revision-required

The design correctly identifies the core architectural seam (MCP dispatch → memory verbs via writer capture) and makes sound decisions on format baking, error mapping posture, and the `impl Write` abstraction. However, it has **three blocker-level specification errors** that make the design unimplementable as written, four major underspecification gaps that would require an implementer to reverse-engineer the codebase, and several minor omissions.

### Standing risks

1. **Backlinks enrichment (F-1, F-5):** The design's §4 describes a fictional API. The actual implementation requires a full corpus scan similar to `run_backlinks` (~100 lines in `memory.rs:2080`), not a one-liner. The backlinks output schema (`uid` + `title` + `type`) also requires per-source memory lookups that the design never specifies.

2. **JSON double-encoding (F-15):** The `call_tool → String` return type change introduces a serialization bug in `handle_tools_call` that would silently corrupt all MCP responses — both review and memory tools. The fix is a one-line change (`McpToolResult::text(out)` instead of `McpToolResult::text(serde_json::to_string(&out)?)`), but the design must explicitly state it.

3. **Scope-design divergence (F-2):** The scope document proposes `run_find_string()` variants; the design chooses `writer: &mut impl Write`. Both are defensible, but the design doesn't reconcile the contradiction. An implementer shouldn't have to choose which document to trust.

### Tradeoffs accepted (no raise)

- **`impl Write` over `dyn Write` (D4):** Sound. Static dispatch is appropriate for 2 call-side types. The design justifies it well.
- **Format hardcoding per tool:** Correct choice — MCP schemas should be narrow contracts. The security rationale for `retrieve` using Table format is well-argued.
- **No structured error enum for memory:** Sensible. With only 3 failure modes, a dedicated error type is overengineering. However, the error code mapping claim in §2 needs correction (F-6).
- **Backlink computation cost:** The design acknowledges the O(n) scan and proposes caching thresholds. Acceptable for v1.

### Haiku

*Design sketches pipes—*
*one leads nowhere; one double-wraps.*
*Ground the dream in code.*
