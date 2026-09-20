# ISS-468: Git test fixtures are not hermetic against an inherited GIT_DIR

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced at `SL-246`'s reconciliation audit as `RV-372` `F-14`, where it did real
damage to the working repository before being caught.

## The trap

`tests/e2e_knowledge_install_commit.rs:28-35`:

```rust
fn git_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.email", "t@t"]);
    git(root, &["config", "user.name", "t"]);
    dir
}
```

`git()` sets `.current_dir(root)` and **nothing else**. `current_dir` does not
beat `GIT_DIR`: if the environment carries one, all three commands run against
*that* repository instead of the temp dir, and two of them are writes.

## What it did

An agent auditing a worktree whose git linkage was broken (`ISS-469`) exported
`GIT_DIR` and `GIT_WORK_TREE` as a workaround, then ran `cargo test -p doctrine
--test e2e_knowledge_install_commit`. The fixture wrote into the **real**
`/workspace/doctrine/.git/config`:

- `core.worktree = /workspace/doctrine/.worktrees/SL-246-c17` — from `git init`
  seeing an ambient `GIT_WORK_TREE`;
- `[user] email = t@t / name = t` — the fixture's literal values.

`core.worktree` in the *shared* config repoints **every git invocation in the
repository**, from every worktree including the primary tree on `edge`, at that
one directory. A `git merge` run from a different worktree wrote its results
into `SL-246-c17`. It is silent: `git status` and `git branch --show-current`
both keep answering plausibly, because the gitdir is still resolved from `cwd`
and only the *work tree* moves.

## Why the class matters more than the instance

`ISS-256` is the same species, resolved: *"Hook-fixture test is not hermetic: an
ambient `core.hooksPath` bypasses it and panics on a bare unwrap."* That one was
fixed at its own site. This says the repo has a **fixture-hygiene class**, not
two unlucky tests.

The asymmetry is what makes it worth fixing centrally: an ambient
`core.hooksPath` makes a test *lie*; an ambient `GIT_DIR` makes a test *write to
your repository*.

## Done when

Every `Command::new("git")` in `tests/` scrubs the ambient git environment —
at minimum `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`,
`GIT_COMMON_DIR`, `GIT_CEILING_DIRECTORIES` — through **one** shared helper
rather than per call site (`STD-001`), and a test asserts the scrub holds by
running a fixture under a deliberately poisoned `GIT_DIR`.

Related: `RV-372` `F-14`, `F-13`; `ISS-256` (resolved sibling); `ISS-469`.
