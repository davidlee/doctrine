# ISS-495: install_mcp treats an unreadable .mcp.json as absent and overwrites it

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`install_mcp` reads `.mcp.json` with `fs::read_to_string(&path).ok()`
(`src/boot.rs:2089`), so ANY read error — not just a missing file — becomes
`None`, i.e. "file absent". `plan_mcp` then plans `Wired` and `write_atomic`
replaces the whole file with a single `mcpServers.doctrine` entry.

Reproduce: write a `.mcp.json` containing a non-UTF-8 byte (or make it
unreadable by permissions) alongside other `mcpServers` entries, run
`doctrine boot install`, and observe the file replaced rather than left alone.

A non-UTF-8 byte is not valid JSON, so this is the "a file doctrine cannot
interpret is never rewritten" invariant (PRD-006 / SPEC-011 REQ-186; the SL-271
design's Invariant 4) being violated for the Claude arm.

Surfaced by RV-399 F-35 on the SL-271 design, which fixes the same class in the
new codex leg (only `ErrorKind::NotFound` maps to `None`; every other read error
is malformed and disclosed). Fixing the Claude leg is out of SL-271's declared
scope and is recorded here rather than done silently.

Fix: in `install_mcp`, distinguish `ErrorKind::NotFound` from every other error,
treating the latter as the malformed/`PrintedFallback` path. Add a case with an
invalid-UTF-8 `.mcp.json` asserting the bytes are unchanged.