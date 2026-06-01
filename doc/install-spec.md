# Installer specification

## Overview

`heresy install` embeds a set of text files into the binary at compile time and
reproduces them into a target directory within a user's project.

## Source layout

```
install/              ← embedded at compile time (§ Embedding)
  manifest.toml       ← configuration (not installed)
  <path>/<file>       ← installed as `<target>/<path>/<file>`
```

## Manifest (`install/manifest.toml`)

```toml
# Optional. Target directory relative to project root. Default: ".doctrine".
target = ".doctrine"

[dirs]
# Directories to create even if no files map into them.
create = [
  ".doctrine/agents",
  ".doctrine/templates",
]

[gitignore]
# Lines to append to .gitignore (idempotent — duplicates are suppressed).
entries = [
  ".doctrine/memory/*",
]

[root_markers]
# Files/dirs that identify the project root when walking up from CWD.
# Default: [".git", ".jj", ".project", "Cargo.toml"].
markers = [
  ".git",
  ".jj",
  ".project",
  "Cargo.toml",
]
```

All sections are optional. Omitted sections use defaults shown above.

## CLI

```
heresy install              # print plan, prompt [y/N], execute
heresy install --dry-run    # print plan, exit
heresy install --yes        # print plan, execute (no prompt)
heresy install --path <dir> # explicit project root (skip walk detection)
```

## Behaviour

### Project-root detection

1. If `--path` is given, use it directly.
2. Otherwise, start at CWD and walk up parent directories.
3. A directory is the project root if it contains any file/dir listed in
   `[root_markers].markers`.
4. Error if no root is found before reaching `/`.

### Dry-run output

Prints the project root, target path, and a table of planned actions:

| Action        | Meaning                              |
|---------------|--------------------------------------|
| `create dir`  | Create a directory (shows `(exists)` if already present) |
| `install`     | Write a new file                     |
| `skip`        | File already exists — left untouched |
| `gitignore`   | Append a line to `.gitignore`        |

### Execution

- **Directories**: `create_dir_all` (no-op if exists).
- **Files**: written only if the destination does not exist. Existing files are
  never overwritten.
- **`.gitignore`**: entries are appended only if not already present
  (line-based deduplication). The file is created if missing.

### Idempotency

Running `install` multiple times is safe:
- Files are never overwritten.
- Directories are no-ops.
- Gitignore entries are not duplicated.

## Embedding

`rust-embed` (`#[derive(RustEmbed)]`, `#[folder = "install/"]`) bakes all
files under `install/` into the binary at compile time. `manifest.toml` is
parsed at runtime but excluded from file installation.

## Testing

Unit tests cover:

- Project-root detection (explicit path, marker matching).
- Plan construction — directory creation, file skip vs install, gitignore
  deduplication.
- Execution — file creation, directory creation, gitignore append, existing-file
  preservation.
