# DEC-405: serve --mcp starts without a project root

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Amended (codex review, 2026-10-23)

User assent: "yes, agreed" to the SL-286 codex-review dispositions.

F1: the condition widened from "no project root" to "no doctrine project". Research Thread 3 showed a git repo without `.doctrine/` serves all 32 tools, each returning empty results; a user-level entry would advertise them in every repo. The server is only useful after `install`, which creates `.doctrine/`.

F4: whether codex displays initialize instructions to the user is unverified (ASM, see SL-286). An empty tool list outside a doctrine project is accepted as self-explanatory; no status tool is added.