`memory validate` exiting non-zero during an audit is a standing corpus signal, not
usually the slice's. Ruling it out of scope is legitimate — but the test has to be
the right one.

**Wrong test.** *Is any stale memory scoped to the subsystem I worked in?*
(`src/design_run/**`). This is a prefix match on where the slice **lives**.

**Right test.** *Does any stale memory's scope cover a path this slice actually
**edited**?* Enumerate all of them — a slice almost always edits outside its
headline subsystem, most often `tests/`.

`SL-256` ruled the drift out with the wrong test at PHASE-03 and the ruling
survived, but by luck: `RV-364` `F-8` re-ran it by enumeration and found
`mem.pattern.testing.no-root-find-walk` scoped `[src/root.rs, tests]`, and the
slice had edited `tests/e2e_design_state.rs`. The conclusion held on the merits
(that memory was already ~50 commits stale and its content was untouched by the
edit), but nothing in the original argument established that.

Enumerating is cheap — `memory validate` prints every finding, and each memory's
`scope.paths` is one `show` away. The prefix match is the shortcut that looks like
the same answer.

Same family as [[mem.pattern.harness.grep-negative-needs-positive-control]]:
a ruling that rests on *not finding* something is only as good as the search that
failed to find it.
