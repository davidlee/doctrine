# SL-014 local POC

Working local surfaces in this repo:

- `.claude/settings.local.json`
  - `SessionStart` runs `/home/david/.cargo/bin/doctrine boot`
  - `SubagentStart` stamps the worker marker
  - `enabledMcpjsonServers` includes `doctrine`
- `.codex/hooks.json`
  - `SessionStart` matcher is `startup|resume|clear|compact`
  - hook command regenerates the snapshot then emits it into the session
- `.mcp.json`
  - registers the Doctrine MCP server on the shared project surface

POC constraint:

- The Codex project layer still needs the one-time trust step before the hook
  file is loaded. That is a separate trust axis from per-hook trust and cannot
  be automated by the install flow.

Design implications captured by the POC:

- `boot install` needs a Codex hook writer that targets `.codex/hooks.json`,
  not the Claude settings file.
- Codex hook management is separate from MCP server registration, so `IMP-111`
  stays split from the hook wiring.
- `boot --emit` is the right long-term hook target, but the current POC uses the
  existing boot-then-read shell wrapper until that verb exists.

Related backlog item:

- `IMP-111` tracks Codex MCP server registration during install. This POC keeps
  that surface separate so the hook writer can be designed and shipped without
  entangling the MCP config path.

## Audit (2026-06-21)

RV-128 reconciliation review completed. Implementation conforms to design —
zero deviation surfaced. Single finding (F-1) tolerated: VT-7/VT-8
(trust-instruction and spike-warning stdout assertions) are low-value stdout
sniffing on exercised code paths.

Evidence: `cargo clippy` zero warnings, 2235+ tests green, code review against
design.md D1–D8.

Ready for reconcile → close.
