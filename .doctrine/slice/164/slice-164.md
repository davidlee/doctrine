# Wire memory write verbs into MCP server tools (record, verify, tag, status, edit)

## Context

The MCP server (`serve --mcp`) exposes 15 tools: 10 review (7 write + 3 read)
and 5 memory (all read: `find`, `retrieve`, `show`, `list`, `validate`). The
memory write verbs (`record`, `verify`, `tag`, `status`, `edit`) exist in the
CLI but have no MCP tool entries in `src/mcp_server/tools.rs`. Agents must fall
back to `bash` for these — fragile and not the right seam. The write-tool
pattern is already well-established on the review side.

## Scope & Objectives

### 1. Memory write tools (2 new MCP tools)
- `memory_record` — high frequency, complex args; the primary agent pain point
- `memory_edit` — structured mutation, clean JSON schema fit
- NOT included (better as CLI): `memory_status` (rare, single-flag),
  `memory_tag` (awkward positional+mixed semantics), `memory_verify`
  (clean-tree precondition makes it fragile as a tool anyway)
- Wire `call_tool()` dispatch for each, calling the existing `memory::run_*`
  functions
- Error mapping: invalid arguments → `-32602`, tool errors → `-32603`
- Update VT-2 (tools/list count: 15 → 18) and the expected name set
- E2E round-trip test for at least `memory_record`

### 2. Onboarding tool (`doctrine_onboard`)
- New MCP tool that returns:
  - **CLI→MCP mapping**: for each skill-referenced CLI command, the MCP tool
    name + parameter mapping the agent should use instead (when MCP is active)
  - **Onboarding memories**: the body of `mem.signpost.doctrine.overview` and
    `mem.signpost.project.orientation` included inline, saving 2 round-trips
- This makes MCP agents self-documenting — they discover the mapping without
  parsing skill files

### 3. Boot footer update
- Modify `.doctrine/boot-footer.md` to instruct: when MCP tools are available,
  call `doctrine_onboard` instead of running `/retrieving-memory` for the two
  signpost memories. Fall back to `/retrieving-memory` if no MCP.

### Affected surface

- `src/mcp_server/tools.rs` — 3 tool definitions + handler dispatch
- `src/memory.rs` — possibly new helper to render onboarding memory bundle
- `tests/e2e_mcp_server.rs` — E2E tests
- `.doctrine/boot-footer.md` — updated onboarding instruction

## Non-Goals

- CLI changes (the verbs already exist)
- New memory verb functionality
- Schema changes to memory TOML/MD
- Harness-side changes (pi/Claude) — these auto-discover from `tools/list`
- Mapping for non-MCP CLI verbs (e.g. `backlog`, `slice`) — only memory + review
  verbs that have MCP equivalents are covered by the onboarding mapping

## Summary

Three parts: (1) wire memory write verbs into MCP following the review write-tool
pattern, (2) add a self-describing `doctrine_onboard` tool so MCP agents discover
CLI→tool mappings and onboarding memories in one call, (3) update the boot footer
to route agents through the tool when available.

Total new MCP tools: 3 (`memory_record`, `memory_edit`, `doctrine_onboard`).
Tool count goes 15 → 18.

## Follow-Ups

- IMP-186 tracks the original discovery
