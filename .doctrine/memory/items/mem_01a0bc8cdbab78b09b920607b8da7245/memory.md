# Never export `GIT_DIR`/`GIT_WORK_TREE` around a test run

**The temptation.** A worktree created outside the jail has a host-absolute
gitdir pointer, so every plain `git` inside it fails `not a git repository:
(null)`. The obvious workaround is to export `GIT_DIR` and `GIT_WORK_TREE` and
carry on.

**Do not.** Use the documented repair instead —
[[mem.pattern.platform.host-created-worktree-gitdir-breaks-in-jail]] relativizes
both pointers and works from host *and* jail. It is strictly better and it is
already written down.

## What the export actually does

`GIT_DIR` is inherited by every child process, including `cargo test`. Doctrine's
own git fixtures set `.current_dir(tmpdir)` and scrub **nothing**:

```rust
// tests/e2e_knowledge_install_commit.rs
git(root, &["init", "-q"]);
git(root, &["config", "user.email", "t@t"]);
git(root, &["config", "user.name", "t"]);
```

`current_dir` does not beat `GIT_DIR`. Under an exported one, all three run
against the **real repository**, and two of them are writes:

- `git init` with an ambient `GIT_WORK_TREE` writes `core.worktree = <that dir>`
  into the shared `.git/config`;
- the two `config` calls write `[user] email = t@t / name = t`.

## Why `core.worktree` in the shared config is the bad one

It repoints **every git invocation in the repository** — from every worktree,
including the primary tree — at that one directory. And it is *silent*: `git
status` and `git branch --show-current` keep answering plausibly, because the
gitdir is still resolved from `cwd` and only the **work tree** moves. A `git
merge` run from one worktree writes its results into another.

## If you think it may have happened

```sh
git config --file .git/config --get core.worktree   # must print nothing
git config --file .git/config --get user.email      # `t@t` is the fixture's value
```

Repair: `git config --file .git/config --unset core.worktree`, then restore the
tree that got written into (`git restore --source=HEAD -- <paths>` plus removal
of the untracked files the stray checkout left).

Observed 2026-09-20 during `SL-246`'s audit (`RV-372` `F-14`). The fixture-hygiene
class is `ISS-468`; `ISS-256` is its resolved sibling (ambient `core.hooksPath`).
Related: [[mem.pattern.doctrine.audit-dispatched-slice-fresh-worktree]].
