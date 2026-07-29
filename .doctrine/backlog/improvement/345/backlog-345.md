# IMP-345: boot snapshot SPINE lists 'reports' and 'explore' as group headings but they are not subcommands

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Diagnosis

`render_boot_map()` (`src/commands/cli.rs`) rendered family keys bare at column 0
and real commands indented two spaces:

```
change      slice revision rfc rec review reconcile coverage   ← help GROUPING
  slice       design plan phases …                             ← real command + verbs
```

The indent runs backwards from intuition — indentation reads as "child" — and the
map never stated its own grammar. Agents therefore inferred
`doctrine <family> <member>`, producing `doctrine reports next` (error:
`unrecognized subcommand 'reports'`, and clap's near-miss tip suggests `export`,
which is useless).

Two aggravators:

- **`knowledge` is both a family key and a real command.** `doctrine knowledge
  list` works, which *confirms* the false rule before the reader reaches
  `reports` / `explore`, where it breaks.
- The defect is the whole class, not the two named keys: `doctrine change slice`,
  `doctrine facets estimate`, `doctrine infra boot` all fail identically.

Recurrence: ≥8 independent captures across three dates in the RFC-011 corpus
(`.doctrine/rfc/011/2026-07-20.case-notes.md`, `2026-07-27.case-notes.md`,
`case-notes.md`) — the highest-frequency friction recorded there.

## Remedy

Bracket the family keys and state the grammar once. Two design constraints
decided the shape:

1. **The snapshot embeds the map in *unfenced* markdown** (`## Commands` in
   `boot.md`), so a `#` sigil would render as an H1 heading. Brackets are inert
   there.
2. **Value fields must stay in one column** so the surface still scans. `[{key}]`
   is padded to `pad + 2`; a command's two-space indent plus `pad` lands its verbs
   at the same column — alignment is now *better* than before, not worse.

```
SPINE: new list show paths (+status where lifecycle) — entity kinds
Grammar: [group] rows list that group's commands; indented rows list a command's verbs. Invoke bare — `doctrine <command> [verb]`, never `doctrine <group> …`.

[change]      slice revision rfc rec review reconcile coverage
  slice       design plan phases notes …
```

Cost ~45 tokens of boot prefix against ≥8 recorded failed invocations at
~150–400 tokens each (error output + retry). Pays back on first avoided miss.

**Rejected:** aliasing the family keys into real subcommands so
`doctrine reports next` works. Eight shim subcommands duplicating dispatch, two
blessed spellings for every verb — parallel implementation, and it teaches the
wrong grammar rather than correcting it.

## Live authored bugs fixed alongside

The wrong form had already propagated out of the map into authored prose:

| site | was | now |
|---|---|---|
| `install/routing-process.md` | `doctrine reports next` | `doctrine next` |
| `plugins/doctrine/skills/spec-coverage-assessment/SKILL.md` | `doctrine explore relation census` / `doctrine explore concept-map` | `doctrine relation census` / `doctrine concept-map` |
| `.doctrine/backlog/chore/042/backlog-042.md` | `doctrine reports next` / `doctrine reports explain <ID>` | `doctrine next` / `doctrine explain <ID>` |

The first is the load-bearing one: `routing-process.md` is inlined into `boot.md`,
so that unparseable invocation was shipping into **every agent's context every
session**.

Historical occurrences in `.doctrine/rfc/011/*case-notes.md` and
`.doctrine/slice/220/` are archive — deliberately left as recorded.

## Verification

`tests/e2e_boot_map_golden.rs` — five existing goldens updated to the bracketed
grammar, two added:

- `boot_map_declares_its_grammar_on_the_second_line` — the legend exists, is
  positioned, and names both tiers plus the bare-invocation rule.
- `boot_map_never_renders_a_family_key_as_a_bare_command` — asserts the *class*:
  no `FAMILY_ORDER` key ever heads a line bare. Guards `change` / `facets` /
  `infra` as well as the two in the title.

`doctrine check gate` green (clippy zero warnings, fmt clean, suites pass).
`doctrine boot` regenerated the snapshot.

## Note: the `.agents/` skill mirror lags by construction

The skill fix lands in the master (`plugins/doctrine/skills/…`); the installed
mirror at `.agents/skills/` did **not** pick it up, and that is correct
behaviour, not a failed install. `install.rs::delegate_argv` shows non-Claude
skill install is `npx skills add <github-slug>` — sourced from the **published**
repo, so a working-tree skill edit cannot appear there until it lands. Mirror is
gitignored and refreshes on publish. Two friction observations recorded (this,
plus a gitignore-respecting `grep` backend that silently skipped `.agents/` and
produced a false-clean sweep).
