# IMP-186: Missing MCP tool endpoint for doctrine memory record (both Claude and pi agents)

## Context

Agents (both Claude via MCP and pi via tool functions) have read-only Doctrine
memory tools: `find`, `retrieve`, `show`, `list`, `validate`. The write-side —
`doctrine memory record` — has no MCP endpoint. Agents must fall back to running
`doctrine memory record …` through a shell, which is fragile (escaping, exit
code handling, reservation errors surfaced as raw stderr).

## Expected behaviour

A `doctrine_memory_record` tool (or equivalent) exposed via the MCP server that
accepts the same parameters as the CLI verb: `--type`, `--title`, `--trust`,
`--severity`, `--lifespan`, scope axes, and a body. The `/record-memory` skill
would then route through it rather than bash.

## Root cause

Checked `src/mcp_server/tools.rs` — the write-tool pattern is well-established:
review has 7 write tools (`new`, `prime`, `raise`, `dispose`, `verify`,
`contest`, `withdraw`) alongside 3 read tools. Memory has 5 tools, all read-side
(`find`, `retrieve`, `show`, `list`, `validate`). The CLI write verbs
(`record`/`new`, `verify`, `tag`, `status`, `edit`) were simply never wired into
the MCP server. This is **not** a harness wiring issue — both pi and Claude
correctly surface exactly what `tools/list` returns.

### MCP tool count by kind

| Kind | Read | Write |
|---|---|---|
| Review | 3 (`list`, `show`, `status`) | 7 (`new`, `prime`, `raise`, `dispose`, `verify`, `contest`, `withdraw`) |
| Memory | 5 (`find`, `retrieve`, `show`, `list`, `validate`) | 0 |

### Missing memory write tools

| CLI verb | MCP tool needed |
|---|---|
| `memory record` / `new` | `memory_record` |
| `memory verify` | `memory_verify` |
| `memory tag` | `memory_tag` |
| `memory status` | `memory_status` |
| `memory edit` | `memory_edit` |

## Notes

- `record` is the highest-value write verb to wire first, but `verify` is a
  close second (threads can't surface without it).
- The `/record-memory` skill currently routes through bash as a workaround.
