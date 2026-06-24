# ISS-048: Memory validator reports false dangling warnings for relations targeting shipped memories

## Symptoms

`doctrine memory validate` reports `dangling: [[relation]] target "<uid>" not found`
for every relation that targets a **shipped** memory (uids like
`mem_019e9a...`, `mem_019ec92b...`). The relations are real — `doctrine link`
succeeds against the same targets, and `memory show <shipped-uid>` resolves
correctly. Only the validator's relation-resolution path is blind to `shipped/`.

Observed 2026-06-24 during a dreaming pass: 14 false dangling warnings across
30 newly-created links.

## Root cause

`resolve_show` (`src/memory.rs`) was fixed (IMP-148 Gap 8) to fall back to
`shipped/`, and `collect_all` has always unioned `items/` ∪ `shipped/`. But the
**memory validator**'s relation-target resolution path does not use either — it
resolves only against `items/`, producing false dangling reports for shipped
targets.

## Fix

Make the validator's relation-target resolution check `shipped/` as a fallback,
mirroring the pattern from `resolve_show` (Gap 8 fix) or `collect_all`.

## Impact

- False positives in `doctrine memory validate` erode trust in the validator.
- Dreaming passes cannot distinguish real dangling relations from shipped-target
  noise without manual inspection.
- Agents will waste time investigating phantom issues.
