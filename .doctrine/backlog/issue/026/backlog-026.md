# ISS-026: worktree import: piped diff to git apply drops trailing newline -> corrupt patch at <stdin>:N

`doctrine worktree import --base <B> --fork <S>` generates the `B..S` diff and
pipes it to `git apply --3way --index`, but the captured patch reaches `git
apply` **without a terminating newline**. When the patch's final line is a real
context line (no trailing blank / no `\ No newline` marker), `git apply` rejects
it with `corrupt patch at <stdin>:N` where N is the last line.

## Repro (SL-103 PHASE-01 funnel, claude arm)
- Fork `worktree-agent-acd07dcb4e1e49226` @ `dd206cf6`, base `B=fbfd8e73`.
- `doctrine worktree import --base fbfd8e73 --fork worktree-agent-acd07dcb4e1e49226`
  → `error: git apply --3way --index … corrupt patch at <stdin>:383` (383 = last line).
- The identical patch applies cleanly when fed **with** a trailing newline:
  - `git diff B..S > p.patch; git apply --3way --index --check p.patch` → exit 0.
  - `printf '%s' "$(cat p.patch)" | git apply --3way --index --check` → reproduces the corrupt-patch error.

## Root cause
The patch byte stream handed to `git apply` is missing its final `\n` (likely a
trailing-newline trim when the diff is captured into a buffer/String before the
pipe). `git apply` requires the patch to end in a newline.

## Fix
Ensure exactly one trailing `\n` terminates the patch stream before `git apply`
(append if absent). Belt-and-suspenders: a regression test importing a fork
whose delta ends on a context line.

## Workaround used
Faithful manual equivalent — `git apply --3way --index <patch-with-newline>` into
the coordination index after independently confirming all five import belts
(HEAD==B, tree-clean, single non-merge commit, no `.doctrine/`/`.claude/` touch).

## Related
- IMP-043 (import re-anchor on moved HEAD) — same verb, different concern.
