# IMP-166: Structured --help --commands: subcommand-native flat table

## Context

`doctrine --help --commands` currently emits a flat two-column table (`command | description`).
Each top-level command (e.g. `slice`, `review`, `backlog`) carries multiple subcommands
(`list`, `new`, `show`, ...) but the flat view gives no visibility into the verb set under
each command — the user must run `doctrine <cmd> --help` separately to discover them.

## Desired behaviour

`doctrine --help --commands` should render a three-column table (`command | verb | description`)
with subcommands nested under their parent:

```
command     verb     description
slice       list     List slices by id: id, status, phases, slug, title
            new      Allocate the next id and scaffold a new slice
            design   Scaffold a design-doc sibling into an existing slice
            ...
review      list     List reviews by id
            new      ...
            ...
```

The command column carries the parent name on the first subcommand row only; subsequent
subcommand rows leave it blank (visual grouping).

A footer line directs users to per-command help for arguments and options:
`For arguments & options: doctrine <command> <verb> --help`

## Non-goals

- Not adding or changing flag/option rendering
- Not modifying per-command `--help` output
- Not adding a third display mode; this replaces the current flat table
