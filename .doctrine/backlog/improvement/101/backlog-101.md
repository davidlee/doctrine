# IMP-101: dispatch: deliver_to config field in doctrine.toml [dispatch] section

## Problem

The dispatch integration path hardcodes the landing branch as `refs/heads/main`.
A slice that should land on a different branch (e.g. an LTS branch, a release
train, or a project with a non-standard trunk) must either rename branches
post-integration or hand-edit the integration target. The landing branch is a
project-level configuration concern, not a per-slice override, and should live
in `doctrine.toml`.

## Fix direction

- **`[dispatch] deliver_to`** field in `doctrine.toml`: a string naming the
  trunk ref (e.g. `"refs/heads/main"`, `"refs/heads/release/2.x"`).
- Default: `"refs/heads/main"` (backward-compatible).
- `dispatch sync --integrate` and the close candidate-admit path read this
  config field instead of hardcoding `main`.
- Validation: the named ref must exist and be a branch (not a tag).

## Context

Surfaced during SL-121 (dispatch integration hardening). The `[dispatch]`
section already exists in `doctrine.toml`; this adds one field.

## Related

- SL-121 (originating slice)
- IMP-098, ISS-024 (sequencing dependencies)
- IMP-126 (`trunk_preference` config — adjacent config surface)
