# REV REV-076 — reconcile SL-286

Revision — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-286 moved codex MCP registration from a project-level `sh -c` wrapper to a
user-level `codex mcp add` entry with an absolute path (EVD-032: interactive codex
spawns MCP servers from a daemon whose PATH is frozen), and made `serve --mcp`
rootless (DEC-405). SPEC-011 still describes the retired wrapper. Audit RV-428
F-3 routed both to this REV.

## Reconcile narrative

### modify REQ-484 (RV-428 F-3) — primary

Before:

> `boot install` registers the doctrine MCP server by a narrow-path,
> edit-preserving write of `[mcp_servers.doctrine]` into the project
> `.codex/config.toml` — a portable `sh -c` entry carrying no host absolute path —
> owning only a shape doctrine has emitted (…), refreshing a stale owned entry,
> preserving foreign keys and tables, and printing the TOML snippet rather than
> clobbering a file or entry it cannot own; the report states what was written,
> never that the harness has activated it.

After:

> An explicit `install` or `boot install` registers the doctrine MCP server in the
> codex user config (`$CODEX_HOME/config.toml`, else `~/.codex/config.toml`) by
> `codex mcp add doctrine -- <absolute path of the doctrine on PATH> serve --mcp`,
> only after a consent prompt (default No, asked only when the entry would change,
> granted by `-y`) that lists any keys the replacement drops; it never bakes a
> binary inside the installed project, and worktree regeneration never runs it.
> With consent it removes doctrine's own project-level wrapper; a project entry
> whose `command` contains a path separator, or a non-empty `url`, is an override
> left untouched and reported, while a PATH-dependent foreign entry is warned and
> holds install incomplete. No codex CLI, a bad `CODEX_HOME` or `HOME`, or an
> unreadable user config degrades to printing the cause and the command to run.
> The report names the file written and the path baked, carries the daemon-restart
> hint after any write, and never claims the harness has activated the server.

Grounds: DEC-404, DEC-406, DEC-407, DEC-408, DEC-411, DEC-412..414; tests
`tests/e2e_codex_install.rs`, `src/codex_mcp.rs` unit suite.

### introduce FR-014 in SPEC-011 (RV-428 F-3)

Statement as recorded on the row: rootless `serve --mcp` (DEC-405). Placed in
SPEC-011 beside FR-013 because no spec governs `serve --mcp` behaviour and the
rootless mode exists for the user-level registration, which starts the server in
every folder. Tests: `tests/e2e_mcp_server.rs` rootless cases.

### modify SPEC-011 prose (RV-428 F-3)

Two passages describe the codex wrapper:

- `## Scope` summary: "register the doctrine MCP server edit-preservingly per
  harness (Claude `.mcp.json`, codex `.codex/config.toml`), owning only a shape
  doctrine emitted" → Claude `.mcp.json` edit-preservingly; codex at user level
  via `codex mcp add`, with consent.
- The "Both arms also **register the doctrine MCP server**" paragraph: replace the
  codex half with the user-level registration, consent, project-wrapper
  retirement and override slot, and the rootless server; keep the Claude half and
  the never-claims-activation clause. The POL-002 sentence becomes: the absolute
  path goes only into the user config, never a file client repos commit (DEC-410).
