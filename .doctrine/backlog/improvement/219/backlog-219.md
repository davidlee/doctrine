## Problem

`doctrine <kind> paths <ref>` uses `fs::read_dir` (non-recursive) and explicitly
skips subdirectories in `src/paths.rs:64-67`. This means any files nested in
subdirectories under an entity directory are invisible to the `paths` command.

For example, RSK-014 has probe artifacts in subdirectories:
```
.doctrine/backlog/risk/014/probe-h1/README.md
.doctrine/backlog/risk/014/probe-h1/discriminator-prompt.md
...
```

But `doctrine backlog paths RSK-014` only shows:
```
.doctrine/backlog/risk/014/backlog-014.toml
.doctrine/backlog/risk/014/backlog-014.md
.doctrine/backlog/risk/014/probe-brief-h1-pretooluse-bwrap.md
```

## Root cause

The `scan_entity_dir` function in `src/paths.rs` uses `fs::read_dir` (one-level)
and has `if !file_type.is_file() { continue; }` on line 66, which skips
subdirectories entirely.

## Proposed solution

Replace `scan_entity_dir` with a recursive walk that descends into
subdirectories, still applying the exclusion filter (`is_excluded_name`) to
each entry. The `EntityPathSet.others` field should collect all non-identity
files from all levels, using root-relative paths throughout.

## Affected commands

- All `* paths` subcommands (backlog, slice, adr, spec, review, memory, etc.)
- Any other consumer of `scan_entity_dir`

## Considerations

- Keep paths root-relative (they already are).
- Keep the exclusion filter (editor detritus).
- Symlinks within entity dirs: decide whether to follow or skip (current
  behaviour skips them). Following would be more intuitive for probe artifacts.
- Performance: entity directories are typically small (<1000 files), so
  `walkdir` or a hand-rolled BFS is fine.
