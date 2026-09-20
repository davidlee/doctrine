# ISS-469: A capsule hand-back worktree is not gatable in the jail

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced at `SL-246`'s reconciliation audit as `RV-372` `F-13`.

## Symptom

`SL-246` generation 17 came home as a branch (`capsule/SL-246/c17`) plus a
worktree at `.worktrees/SL-246-c17` carrying the state-tree exhibit. Neither
`git` nor `cargo` works inside that worktree from within the bwrap jail.

**1. The git linkage points at a host path.**

```
$ cat .worktrees/SL-246-c17/.git
gitdir: /home/david/dev/doctrine/.git/worktrees/SL-246-c17
```

`/home/david/dev` does not exist in the jail; the same repository is mounted at
`/workspace/doctrine`. Consequences, all from inside the worktree:

| command | result |
|---|---|
| any plain `git` | `fatal: not a git repository: (null)` |
| `git worktree list` (from the primary tree) | the entry is marked `prunable` |
| `doctrine slice conformance SL-246` | `Error: git command failed: … diff --name-status` |
| `tests/e2e_knowledge_install_commit.rs::this_repos_knowledge_tree_is_tracked` | FAILED — `git ls-files failed` |

**2. `web/map/dist` is unprovisioned.** It is gitignored, so a bare worktree
does not get it, and the `RustEmbed` derive on `map_server::assets::Assets`
(`#[folder = "web/map/dist/"]`) then produces no `::get`. `cargo build` dies with
three `E0599`s **before any test runs** — and the error names
`icu_collections::codepointtrie::TypedCodePointTrie` as a candidate trait, which
points nowhere near the real cause.

## Why it matters

`PHASE-06`'s `EX-5` is *"`doctrine check gate` is green with NO suite red"*. The
tree in which that claim is handed back is a tree in which the gate **cannot be
run**. The claim is not falsified — it is unfalsifiable *where it lands*, which
is the property an exit criterion exists to deny. (`SL-246`'s gate does pass:
`just gate` exits 0 with 7801 tests and zero failures once the work is on a
worktree forked in-jail.)

The second half is already known — `mem_019f4c64e65574238b7026f7301c8a2c` records
"supply `web/map/dist` first" for auditing a dispatched slice's evidence ref. The
memory does not cover the git-linkage half, which is specific to a worktree
**created outside the jail** and handed to an agent inside it.

## The repair was already written down, and the workaround is a loaded gun

`mem.pattern.platform.host-created-worktree-gitdir-breaks-in-jail`
(`mem_019f255e18847da2aa383667826a314d`) already carries the correct fix, and it
is better than the obvious one: **relativize both pointers** so they resolve
under any prefix, host and jail alike.

```
<worktree>/.git                      -> gitdir: ../../.git/worktrees/<name>
.git/worktrees/<name>/gitdir         -> ../../../.worktrees/<name>/.git
```

Applied to `SL-246-c17` at the audit: `git` works natively inside it again, it
no longer reports `prunable`, and the tree verifies clean.

The workaround that was reached for instead — export `GIT_DIR` / `GIT_WORK_TREE`
— is a loaded gun: see `ISS-468`, where exactly that leaked into `cargo test` and
rewrote the shared repository config. Part of this item is therefore a
*discoverability* failure, not only a provisioning one: the memory existed and
was not retrieved before improvising.

Neither half is intrinsic. `doctrine worktree fork` already does the right thing:
it reports `provisioned … 4 copied` (which includes `web/map/dist`) and mints a
jail-local linkage, and an audit fork made that way gates green first try.

## Done when

A capsule hand-back either (a) is produced through the same provisioning path
`doctrine worktree fork` uses, or (b) states in its hand-back that the tree is
read-only evidence and names the fork command that produces a gatable one —
rather than inviting `cd` into a tree where the first command fails.

Related: `RV-372` `F-13`, `F-14`; `ISS-468`; `IMP-434` (the capsule agent defs);
`mem_019f4c64e65574238b7026f7301c8a2c`.
