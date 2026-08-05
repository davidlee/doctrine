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
