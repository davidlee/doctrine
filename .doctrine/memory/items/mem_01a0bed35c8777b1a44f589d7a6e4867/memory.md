# Path tripwire criteria on `edge` need a commit-scoped `git log`, not a range diff

A slice whose design forbids touching some path (SL-260's *no change under
`src/`* tripwire is the worked example) wants a criterion that checks it. The
obvious form is wrong on a shared branch:

```sh
git diff --stat <base>..HEAD -- src/       # WRONG on edge
```

The primary worktree stays on `edge` and several slices are authored there
concurrently, so `<base>..HEAD` spans *their* commits too. The criterion
red-lights on another slice's work and says nothing about yours.

Use commit attribution instead — conventional commit scopes are the join:

```sh
git log --oneline <base>..HEAD --grep='SL-NNN' -- src/     # must be empty
git log --oneline <base>..HEAD --grep='SL-NNN'             # positive control
```

The control is not optional: a negative grep is untrustworthy without one, and
a typo'd `--grep` produces the same empty output as a clean tripwire. `<base>`
is the parent of the slice's first commit (`git log --reverse --grep='SL-NNN'`).

**Recording it:** a negative assertion has no file for keywords to live in, so
the structured VT mandate cannot gate it. `{ id = "VT-n", expects = "…",
waived = true, waived_reason = "…" }` is the sanctioned shape — it keeps the
declared `VT` mode rather than demoting the tripwire to agent judgement, and
`verify-vt` reports `WAIVED` rather than `UNCHECKABLE`.

Same root cause as
[[mem.pattern.audit.conformance-undeclared-shared-branch-interleave]] — a commit
range on `edge` is not a slice — but it bites at plan time, where the fix is
free, rather than at audit, where it is a disposition.
