This repo's TDD discipline demands a negative control for almost every
assertion: break the thing, watch the test go red, put it back. The
put-it-back step is where the loss happens.

`git checkout -- <path>` and `git restore <path>` both reset the file to
**HEAD**. They do not undo "the edit you just made" — they undo *every*
uncommitted change in that file. When the control was applied to the same file
as the work being tested (the normal case: you break the function your new
test covers), the revert discards the function too, silently and with a
zero exit code.

Observed twice in one session (SL-244 PHASE-06):
- `sed` flipped one enum arm as a control; `git checkout src/design_run/gate.rs`
  took the control **and** the three new `as_str` functions with it.
- A python edit deleted one manifest row as a control; `git restore
  publication/manifest.toml` took the control **and** the nine rows plus the
  header paragraph.

Neither was permanent — the content was still in conversation context and was
re-applied — but each cost a rebuild cycle, and the class is worse than the
instances: had the file held work that was not re-derivable, it would be gone,
with nothing in the output saying so.

**The rule.** Apply the control with an editor and revert it with the editor —
the exact inverse edit. Or commit before controlling, so the file's HEAD *is*
the state you want back. Never a path-scoped `checkout`/`restore` while the
file holds uncommitted work you want to keep.

Distinct from `AGENTS.md`'s `git checkout <ref> --` warning, which is about an
empty pathspec falling back to a whole-worktree branch switch. Same family
(git verbs that quietly do more than the change you meant to undo), different
mechanism: that one moves the branch, this one widens the revert.

Related: [[mem.pattern.doctrine.tdd-loop]], and the positive-control rule in
[[mem.pattern.grep.negative-result-needs-positive-control]].

**Third instance, SL-259 PHASE-04 (2026-09-15), and it sharpens the rule.**
The revert was the last line of a reusable shell *probe function*:

```bash
probe() { ...mutate...; cargo test | grep ...; git checkout -- "$2"; }
```

The first invocation's `grep` matched nothing (wrong filter), so the run looked
inert — and the revert fired anyway, taking every uncommitted edit in
`src/design_run/submission.rs`: a new enum, a widened const table, a new method
and a changed signature. Nothing in the output said so; it surfaced two commands
later as `no KeyWhen in design_run::submission`.

A harness makes this worse than the inline case, in two ways. The revert runs
**unconditionally**, including on the probe that failed to apply or failed to
match, so a mistake in the probe still costs the work. And it runs **silently**
each iteration, so the loss is invisible until a build.

The rule holds and the fix is the same shape, one level up: a probe harness
restores from a **file copy** it made itself, never from git.

```bash
probe() { cp "$f" /tmp/x.bak; mutate; test; cp /tmp/x.bak "$f"; }
```

That is correct on a dirty tree, correct when the mutation fails to apply, and
has no HEAD in it to be wrong about.
