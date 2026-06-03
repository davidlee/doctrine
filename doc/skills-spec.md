# Skills specification

## Overview

doctrine ships a curated set of **agent skills** (Anthropic `SKILL.md`
format). They reach agents through two independent channels:

1. **Published marketplace** — the repo is itself a Claude Code plugin
   marketplace. Anyone can consume it with no doctrine binary:
   `/plugin marketplace add doctrine/doctrine`, or
   `npx skills add doctrine/doctrine`.
2. **`doctrine skills`** — for users who already have the binary. It carries the
   same skills embedded at compile time and installs them per agent. For
   **Claude** it installs **directly** (file copy, no Node). For **every other
   agent** it **delegates** to `npx skills` (vercel-labs/skills), which knows
   ~71 agent layouts so doctrine does not have to.

Both channels read one canonical source tree. No skill is duplicated.

### Why this split

- **`npx skills` (vercel-labs/skills) is the universal installer.** One CLI
  (`npx skills add <source>`) installs Anthropic `SKILL.md` skills into ~71
  agents, each with its own layout, and it already understands Claude plugin
  marketplaces (`.claude-plugin/marketplace.json` / `plugin.json`). doctrine
  delegates to it rather than reimplementing per-agent install logic.
- **Claude is installed directly instead of via `npx skills`** because the
  binary already embeds the skills and owns a file-copy installer (`doctrine
  install`). The direct path reuses that machinery, needs no Node on `PATH`,
  and keeps the default agent working offline. `npx skills` is the fallback
  precisely for the agents doctrine does *not* special-case.
- **Net rule:** target is Claude ⇒ direct file copy; any other agent ⇒ shell
  out to `npx skills`. See § Routing.

## Source layout

Skills are grouped into **domain plugins**. Each domain is a self-contained
Claude Code plugin; each skill belongs to exactly one domain.

```
.claude-plugin/
  marketplace.json          ← lists the domain plugins (publishing channel)
plugins/
  <domain>/                 ← one Claude Code plugin per domain
    .claude-plugin/
      plugin.json           ← { name, version, description }
    skills/
      <skill>/
        SKILL.md            ← YAML frontmatter: name, description
        <supporting files>
```

- `plugins/` is embedded into the binary at compile time (a second
  `#[derive(RustEmbed)] #[folder = "plugins/"]`, parallel to `install/`).
- The same tree is what `npx skills` and Claude's `/plugin` discover when
  pointed at the repo. Single source, two channels.
- Domains (TBD) are coarse capability groups, e.g. `review`, `rust`, `docs`.

### `marketplace.json`

```json
{
  "name": "doctrine",
  "owner": { "name": "doctrine" },
  "plugins": [
    { "name": "review", "source": "./plugins/review", "description": "…" },
    { "name": "rust",   "source": "./plugins/rust",   "description": "…" }
  ]
}
```

### `SKILL.md` frontmatter

doctrine reads only `name` and `description` from each skill's frontmatter
for listing, via `serde_yaml`. Everything else is opaque payload copied
verbatim.

```markdown
---
name: <skill-name>
description: <one line — when to use this skill>
---

<skill body>
```

## Architecture

The CLI layer stays thin and dumb. `main`/clap only parses args and calls one
entry function; that function does IO and prompting. All decisions live in pure
library functions that take data and return data — no `std::process`, no
`stdout`, no filesystem reads inside them.

| Pure (library, unit-tested)                          | Imperative (thin shell)        |
|------------------------------------------------------|--------------------------------|
| frontmatter parse → `{name, description}`            | read embedded bytes / `SKILL.md` |
| agent detection from a probe result → agent list     | stat `.claude/`                |
| plan construction (select skills, direct vs delegate)| print plan, prompt `[y/N]`     |
| delegate argv assembly → `Vec<String>`               | spawn `npx`, copy files        |

The `npx` spawn and file copy sit behind a seam (a trait / fn pointer) so plans
are asserted without Node or disk. Same split as `doctrine install` today
(`build_plan` / `detect_project_root` pure; `run` / `execute_plan` imperative).
Domain logic does not leak into the CLI, and IO does not leak into the planner.

## CLI

`doctrine skills` is a new subcommand group, parallel to `doctrine install`.

```
doctrine skills list [--agent <a>] [--installed]
doctrine skills install [--agent <a>]... [--skill <name>]... [--domain <d>]...
                      [--global] [--dry-run] [--yes]
```

v1 scope is **list + install** only. Removal and update are out of scope
(§ Out of scope).

### `doctrine skills list`

Enumerates embedded skills grouped by domain — `name`, `description`, and
install status for the detected (or `--agent`-named) agent. `--installed`
restricts to skills already present.

```
review
  code-review    Review a diff for correctness bugs           [claude: installed]
  security       Security review of pending changes           [claude: —]
rust
  clippy-triage  Triage and fix clippy denials                [claude: —]
```

Status is authoritative for Claude (file presence under `.claude/skills/`).
For delegated agents status is best-effort and may read `not tracked`.

### `doctrine skills install`

```
doctrine skills install                       # detect agent, plan, prompt [y/N], execute
doctrine skills install --dry-run             # plan only, exit
doctrine skills install --yes                 # plan, execute, no prompt
doctrine skills install --agent codex         # explicit target (delegated)
doctrine skills install --agent claude --agent cursor
doctrine skills install --skill code-review   # subset by skill name
doctrine skills install --domain review       # subset by domain
doctrine skills install --global              # user dir instead of project
```

- `--agent` is repeatable. Default: auto-detect (§ Agent detection).
- `--skill` / `--domain` select a subset; repeatable; omitted ⇒ all skills.
- `--global` installs to the user directory rather than the project.

## Behaviour

### Agent detection

When no `--agent` is given:

1. If `.claude/` exists in the project root → target `claude`.
2. Otherwise error, listing supported agents and asking for explicit
   `--agent`. (doctrine does not guess non-Claude agents.)

Project root is resolved with the same walk-up logic and `root_markers` as
`doctrine install` (see install-spec § Project-root detection). Shared code.

### Routing

Each target agent takes one of two paths:

| Agent      | Path     | Mechanism                                            |
|------------|----------|------------------------------------------------------|
| `claude`   | direct   | copy embedded skill dirs into the Claude skills dir  |
| everything else | delegate | shell out to `npx skills add …`                 |

#### Direct (Claude)

Copy each selected `plugins/<domain>/skills/<skill>/` tree into the Claude
skills directory, flattened by skill name (Claude skills are flat):

- project: `<root>/.claude/skills/<skill>/`
- `--global`: `~/.claude/skills/<skill>/`

Reuses the `doctrine install` file-copy machinery. **Skip, never overwrite** an
existing `<skill>/` directory (idempotent, like the installer). Node is not
required on this path.

Note: direct install copies **skills only**. A domain plugin's other
components (commands, agents, hooks) are delivered only through the published
marketplace channel, not by `doctrine skills install`.

#### Delegate (other agents)

Shell out once per agent:

```
npx skills add doctrine/doctrine --agent <agent> [--global] \
    [--skill <name>]... --yes
```

- Source is the **published repo shorthand** `doctrine/doctrine`.
- `--skill` / `--domain` selections map to `skills`' `-s <skill>...`.
- `--global` maps to `-g`; default is project-local.
- `--yes` is always passed (doctrine already confirmed the plan).

Prerequisite: `npx` (Node) on `PATH`. If absent, doctrine errors with
install guidance and does **not** fall back. The delegated command, verbatim,
appears in the dry-run plan so the user can run it by hand.

### Dry-run output

Prints project root, each target agent, its path (direct/delegate), and the
planned actions:

| Action     | Meaning                                              |
|------------|------------------------------------------------------|
| `install`  | copy a skill into the Claude skills dir (direct)     |
| `skip`     | skill dir already exists — left untouched (direct)   |
| `delegate` | the exact `npx skills add …` command to be run       |

### Execution

- Direct: `create_dir_all` + copy tree; skip existing skill dirs.
- Delegate: run the `npx` command; surface its exit status. A non-zero exit
  aborts that agent and is reported; other agents still proceed.

### Idempotency

- Direct installs skip existing skill directories — re-running is safe and
  changes nothing already present.
- Delegation is as idempotent as `npx skills add` (its own concern).

## Out of scope (v1)

- **Remove / update.** For Claude, delete `.claude/skills/<skill>/`. For other
  agents, `npx skills remove|update`. doctrine does not wrap these yet.
- **Authoring.** Use `npx skills init` to scaffold a new `SKILL.md`.
- **Publishing.** `marketplace.json` / `plugin.json` are maintained in-repo by
  hand (or a later `doctrine` command); `doctrine skills` only consumes them.

## Known risks

- **Version skew (accepted).** Delegated installs pull repo `HEAD`, not the
  embedded snapshot in the running binary, so a non-Claude agent can receive
  newer or older skills than the binary carries. v1 accepts this and tracks
  `HEAD`. Backlogged: pin `owner/repo@<ref>` to the binary's build tag once
  `npx skills` is confirmed to accept a ref.

## Open questions

1. **Multi-agent status in `list`.** Reliable only for Claude. Define how much
   effort to spend probing delegated agents' directories vs. reporting
   `not tracked`.

## Testing

Unit tests cover:

- Frontmatter extraction — `name` / `description` from `SKILL.md`.
- Plan construction — skill selection by `--skill` / `--domain`; direct
  install vs. skip-existing; delegate command-line assembly per agent.
- Agent detection — `.claude/` present ⇒ claude; absent ⇒ error.
- Routing — claude ⇒ direct steps; other agent ⇒ a single `delegate` step
  with the expected argv.
- Execution (direct) — skill dir created; pre-existing skill dir preserved.

Delegation execution (the actual `npx` call) is integration-tested behind a
seam that lets tests assert the assembled command without spawning Node.
