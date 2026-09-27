Codex splits the two activation surfaces:

- **Project-local MCP config** (`.codex/config.toml`, `[mcp_servers.<id>]`) takes
  effect with **no trust/approval step** at startup. Verified by operator testing
  (2026-09-27) adding the doctrine server manually.
- **Hooks** (`.codex/hooks.json`) need the explicit pass: `[features] hooks = true`
  in `.codex/config.toml`, accept the project trust prompt, then `/hooks` to trust
  each hook — which is exactly the three-step disclosure doctrine's
  `write_codex_activation` prints.

Consequence for installers: a project-local MCP registration can be written
silently with no follow-up instruction; a hook install cannot, and must keep
disclosing its manual steps.

Related: `codex mcp add` has **no scope flag** (checked against codex 0.155.1
`mcp add --help`) — it only ever writes `~/.codex/config.toml`, so a
project-local entry must be merged into `.codex/config.toml` directly.

See IMP-111 (codex MCP registration during install).

## Nuance (probe, 2026-09-27): the project layer is still trust-gated

"Just works" holds *within a trusted project*. In a scratch project codex skipped
`.codex/config.toml` entirely — `codex mcp list` reported no servers, and the
configured servers never spawned — until an isolated `CODEX_HOME` config declared
`[projects."<path>"] trust_level = "trusted"`. Under non-interactive `codex exec`
the skip is **silent**: no prompt, no error, no warning. So an installer can
write a correct project-local MCP entry that a fresh, untrusted project simply
ignores.
