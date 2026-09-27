# DEC-332: Codex MCP ownership narrows to the wrapper shape

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Ownership narrows to the wrapper shape alone. The installer owns the key
`doctrine` under `mcp_servers` only when the entry is `command = "sh"` with
`args = ["-c", <line matching the doctrine wrapper pattern>]`; `current`
additionally requires `env_vars` to contain `DOCTRINE_BIN` and the line to equal
the emitted constant. A wrapper missing the whitelist, or carrying an earlier
doctrine wording, is owned-but-stale and refreshes.

## Why

Supersedes DEC-324's second half. DEC-324 also owned a plain `command =
"doctrine"` entry as a migratable stale form; the adversarial pass (RV-399 F-7)
showed that form has never been emitted by this leg, so a user who hand-wrote it
may have chosen it deliberately — rewriting it would silently introduce a shell
wrapper, against the no-clobber posture and POL-002 facet 2's refusal to bake
leniency for local state. The plain literal, `/bin/sh` and baked abspaths are
therefore FOREIGN: left untouched, disclosed by the printed snippet.

That also closes IMP-111 D3 by enumerating the emitted-form set explicitly
instead of passing over two of its members.
