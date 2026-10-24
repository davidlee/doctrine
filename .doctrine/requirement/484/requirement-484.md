# REQ-484: An explicit `install` or `boot install` registers the doctrine MCP server in the codex user config (`$CODEX_HOME/config.toml`, else `~/.codex/config.toml`) by `codex mcp add doctrine -- <absolute path of the doctrine on PATH> serve --mcp`, only after a consent prompt (default No, asked only when the entry would change, granted by `-y`) that lists any keys the replacement drops; it never bakes a binary inside the installed project, and worktree regeneration never runs it. With consent it removes doctrine's own project-level wrapper; a project entry whose `command` contains a path separator, or a non-empty `url`, is an override left untouched and reported, while a PATH-dependent foreign entry is warned and holds install incomplete. No codex CLI, a bad `CODEX_HOME` or `HOME`, or an unreadable user config degrades to printing the cause and the command to run. The report names the file written and the path baked, carries the daemon-restart hint after any write, and never claims the harness has activated the server.

## Statement

<!-- The sister TOML's `description` field is the primary, normative statement.
     Prose here may elaborate, expand upon, or disambiguate it — never
     duplicate it. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
