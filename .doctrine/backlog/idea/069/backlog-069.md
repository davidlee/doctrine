# IDE-069: User-level MCP registration for Claude and pi

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

SL-286 moves the codex MCP entry to the user-level codex config (DEC-403,
DEC-406), under the out-of-repo write rule DEC-410. The Claude (`.mcp.json`)
and pi arms stay project-level: they do not share codex's frozen-PATH daemon
failure (EVD-032). Question: should they move to user level for uniformity, with
the project layer as the override slot? Weigh against: Claude's
`${DOCTRINE_BIN:-doctrine}` portable form works today, and a user-level write
touches every project on the machine.
