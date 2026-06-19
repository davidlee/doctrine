# ISS-032: doctrine worktree import strips trailing newline before git apply — valid single-commit patch rejected as corrupt

Discovered during SL-111 dispatch session (2026-06-19), funnelling PHASE-03's
worker delta into the coordination index.

## Symptom

`doctrine worktree import --base <B> --fork <fork>` failed:

```
Error: git apply --3way --index 0ff72b6f..worktree-agent-…
Caused by:
    git command failed: apply --3way --index: error: corrupt patch at <stdin>:247
```

The delta was a clean, well-formed 247-line patch (23 one-line edits across 12
files; `S^ == B`, single non-merge commit, no `.doctrine/`/`.claude/` touch).

## Root cause

`import` pipes the generated diff to `git apply` via **stdin with the trailing
newline stripped** (a `.trim()` / `.lines()`-rejoin or equivalent on the captured
diff). `git apply` requires the patch stream to be newline-terminated; without it
the final hunk is "corrupt at <stdin>:<last-line>".

Reproduced deterministically:

```
git apply --3way --index --check < patch          # exit 0 — applies cleanly
printf '%s' "$(cat patch)" | git apply --check     # error: corrupt patch at <stdin>:247
```

The `$()` strips the trailing newline, mimicking `import`'s pipe — and reproduces
the exact error. The patch itself is valid (file-based `--check` applies all 12
files cleanly).

## Workaround used (this session)

Bypassed `import` with its exact semantic equivalent — staged the validated patch
into the coordination index, non-committing:

```
git diff <B>..<fork> > p.patch
git apply --3way --index p.patch     # newline preserved → clean
```

Then continued the funnel normally (verify → branch-point-check → one commit →
record-boundary).

## Proposed fix

In `worktree import`, **do not strip the trailing newline** before handing the
diff to `git apply` — pipe the raw diff bytes (or append a `\n`). Add a
regression test: a single-commit fork whose patch ends exactly on a final-hunk
context line must import cleanly.

## Severity

High for the claude arm: every `import` of a delta whose last hunk lands on the
final line corrupts. The funnel cannot complete via the CLI verb without the
manual `git apply` workaround.

## Resolved

`worktree import` now captures the delta via `git::git_bytes` (raw, untrimmed)
and `git_apply_index` streams `&[u8]` verbatim — the trailing newline survives.
`git_text`'s `.trim()` is untouched (its many other callers want trimmed output).
Regression test `import_applies_patch_ending_at_eof` (a fork rewriting a
newline-terminated file's final line) is red on the old path, green now.

## Related

- ISS-029, ISS-031 — the other two claude-arm dispatch findings from the
  SL-111 session (base-selection cd instruction; coord-dir placement). Three
  distinct gaps surfaced funnelling one slice through the claude arm.
- `verify-worker` `unstamped` refusal (Agent worktrees carry no worker marker;
  IMP-072) is a *separate* claude-arm gap hit in the same funnel.
