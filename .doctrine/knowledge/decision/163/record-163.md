# DEC-163: Settings scope is a sticky doctrine.toml key

## Decision

The Claude settings scope [[DEC-162]] installs at is **remembered as a key in
`doctrine.toml`**, defaulting to project `.claude/settings.json`.

- **No `--scope` flag.** The config key is the only way to say it.
- **The installer announces the target early** — where it will write, and which
  key changes it.

## Selected and remembered

The key is read through the existing `load_doctrine_toml` seam (`src/dtoml.rs`),
with the `[dispatch]` table as the precedent: a kebab-case key with a serde
default, so an absent key and an absent table both yield the default. This rides
an existing seam rather than inventing a second config surface
(no-parallel-implementation).

### Why no flag

A per-invocation flag makes `R10` recurring rather than one-time. Doctrine's
install paths run routinely — `memory sync install` writes `SETTINGS_REL` today
(`src/corpus.rs:506`), and `doctrine install` is the documented ritual after any
shipped-memory edit. Under a flag-only design, a user who chose local scope has
that choice reverted by the next flagless run, which re-creates the entry in the
project file beside their local one. Every routine install becomes another
double-fire.

A flag layered *over* a sticky key is redundant surface: it buys the ability to
say the scope on the command line in a fresh project, and nothing else.

### What replaces it

The installer's early announcement is what makes dropping the flag safe —
discoverability was the flag's only remaining job. Without it the key would be
undiscoverable except by reading source or docs.

Mechanics are settled: `print_stdout` is denied (`Cargo.toml:228`) and
`src/install.rs` already threads a writer, so the line rides the existing
`writeln!(stdout, …)` pattern. No new output seam.

## Threading

Not contested, and cheap. `install_hook_to_file(root, rel_path, spec, dry_run)`
(`src/boot.rs:1595`) already takes the target file per call as a `&'static str`,
so a two-variant enum mapping to two constants satisfies the signature unchanged.
`SETTINGS_REL` (`.claude/settings.local.json`) exists; `.claude/settings.json`
needs a new named constant beside it (STD-001).

The project/local distinction is already recognised vocabulary in this file —
`MCP_REL`'s comment (`src/boot.rs:537`) calls `.mcp.json` "a committed,
team-shared file — unlike `SETTINGS_REL`".

## Consequence for `R10`

This restores `R10` to a **single cutover moment** rather than a treadmill, which
is the assumption `inq-5` needs in order to choose between documenting the
double-fire and normalizing against the scope being left.

Recorded from design run `dr-019fd692` checkpoint `cp-3` disposing `inq-4`.
