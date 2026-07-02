# CHR-033: Integrate dynamic boot resolver into shipped pi extension

Replace the body of `generate_pi_extension()` in `src/boot.rs` (~L1673) with the
dynamic resolver prototype sketched at `.pi/extensions/doctrine-boot.ts`.

**What changes:**

- Swap `execSync("doctrine boot")` (writes snapshot to disk, relies on
  APPEND_SYSTEM.md symlink) for `execSync("doctrine boot --emit")` (stdout).
- Add `before_agent_start` handler that appends the cached stdout to
  `event.systemPrompt`.
- Remove the APPEND_SYSTEM.md symlink install step (`install_append_system`)
  — the extension self-injects, no symlink needed.
- Port from CommonJS (`require`/`module.exports`) to ESM (`import`/`export
  default`) to match pi's preferred extension format.
- Preserve the MCP bridge import (`./mcp.ts`).

**Caching:** run once per session (`session_start`), inject byte-identical
string every turn → Anthropic prefix cache holds across turns.

**Swap point for SL-186:** once the resolver lands, change the exec'd command
from `doctrine boot --emit` to `doctrine prompt resolve <context>`.
