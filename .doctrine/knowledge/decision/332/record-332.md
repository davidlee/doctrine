# DEC-332: Codex MCP ownership narrows to the wrapper shape

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Ownership narrows to shapes doctrine has emitted. The installer owns the key
`doctrine` under `mcp_servers` only when the entry's keys are a subset of
`{command, args, env_vars}`, `command = "sh"`, `args` is exactly `["-c", <line>]`,
`<line>` normalises into the emitted-forms set (`CODEX_MCP_WRAPPER_FORMS` — every
wrapper wording doctrine has emitted, newest first), and `env_vars` is either
absent or exactly the emitted whitelist `["DOCTRINE_BIN"]`. `current` requires the
line to equal `CODEX_MCP_WRAPPER` byte for byte and `env_vars` to be exactly the
whitelist. A wrapper whose `env_vars` is absent, whose line is a spacing variant,
or whose line is an earlier emitted wording is owned-but-stale and refreshes to
canonical.

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

## Amended (RV-399 F-32, F-33)

The third adversarial pass found the predicate as first written could not express
what this decision says. Three corrections, none changing the narrowing:

- **The emitted-forms set is now the mechanism.** "An earlier doctrine wording is
  owned-and-stale" was inexpressible as literal equality against one constant:
  every other wording failed the owned test and became foreign, so the first time
  the wrapper wording changed, every installed entry would have nagged forever.
  Ownership is membership in `CODEX_MCP_WRAPPER_FORMS`; a wording change APPENDS
  its predecessor. Today the set holds exactly one element.
- **`owned` normalises, `current` is byte-exact.** Normalising for ownership
  (whitespace) while comparing byte-exactly for `current` is what makes a
  reformatted wrapper an owned-but-stale refresh rather than a silent no-op.
- **`env_vars` is strict in both directions.** The first wording admitted ANY
  `env_vars` value to ownership and required only membership for `current`, so
  `["FOO"]` (a user's list) was owned-and-stale and would have been rewritten to
  the whitelist, dropping their variable, while `["DOCTRINE_BIN", "FOO"]` read as
  current. Ownership now requires `env_vars` absent or exactly the emitted array;
  anything else is foreign. Doctrine has only ever emitted absent or exactly the
  whitelist, so no user value can be clobbered.

The reasoning above (the narrowing itself) is unchanged and unreopened.
