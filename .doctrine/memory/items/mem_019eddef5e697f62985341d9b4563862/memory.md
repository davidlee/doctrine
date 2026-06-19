# pi v0.79.6 default built-in tool set is read, write, edit, bash only

pi v0.79.6 ships exactly four built-in tools: `read`, `write`, `edit`, `bash`.
`grep`, `find`, and `ls` are NOT in the default set — they must be explicitly
enabled via `--tools grep,find,ls` (or `--tools read,bash,edit,write,grep,find,ls`
to include all).

Source: pi --help description line, pi README.md §Quick Start.

## When designing pi spawn templates

- Do NOT assume grep/find/ls are available by default.
- Pass `--tools read,bash,edit,write,grep,find,ls` if the worker needs them.
- pi's `--no-skills` does not affect tool availability — only skill loading.
- pi's `--no-extensions` does not affect built-in tools — only extension loading.

## Verification

```bash
pi --help | head -1
# pi - AI coding assistant with read, bash, edit, write tools
```

Discovered during RV-090 (inquisition of SL-108 pi dispatch worker design).
