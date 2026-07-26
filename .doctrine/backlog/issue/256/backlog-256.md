# ISS-256: Hook-fixture test is not hermetic: an ambient core.hooksPath bypasses it and panics on a bare unwrap

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

```
---- dispatch::tests::run_commit_declared_deletion_lands_and_scopes_allowed_deletions_to_child stdout ----
thread '…' panicked at src/dispatch.rs:11283:55:
called `Result::unwrap()` on an `Err` value:
  Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

Reported by the User 2026-07-27 against a tree where `doctrine check gate` was
otherwise green (4903 passed / 0 failed in the jail). **Environment-dependent, not
a code regression** — the same test passes in an environment with no ambient
`core.hooksPath`.

## Root cause

`install_recording_hook` (`src/dispatch.rs:11237`) installs the fixture hook at the
**default** location:

```rust
let git_dir = git(coord, &["rev-parse", "--absolute-git-dir"]);
let hooks = Path::new(&git_dir).join("hooks");
…write(hooks.join("pre-commit"), …)
```

It never pins `core.hooksPath` on the fixture repo. Git resolves hooks through
`core.hooksPath` when set at **any** scope, so a global or system value — common in
real dev setups (husky, dotfiles hook dirs, corporate templates) — silently
redirects git away from the fixture's hook. The hook never runs, the sentinel file
is never written, and line 11283's

```rust
assert_eq!(std::fs::read_to_string(&sentinel).unwrap(), "del.txt");
```

panics with a bare `NotFound` naming neither the file nor the cause.

## Reproduction (confirmed)

```bash
git config --global core.hooksPath /some/empty/dir
cargo test --bin doctrine \
  dispatch::tests::run_commit_declared_deletion_lands_and_scopes_allowed_deletions_to_child
# → FAILED, exactly the reported panic
git config --global --unset core.hooksPath
# → passes
```

Verified 2026-07-27 in this repo. Scope: **one** call site (`:11268`), so one
affected test.

## The irony worth recording

SL-228 design §11 lists, as a REQ-389 verification criterion, precisely this
scenario for the *product*:

> **chained global hook still fires** (the operator-gotcha case: global
> `core.hooksPath` set → both run)

The shipped hook chains to the effective non-worktree `hooksPath` and handles the
override correctly (D5). It is the **test fixture** that does not — it breaks under
exactly the condition the feature was designed to survive. The product is hermetic
about hooks; its test is not.

## Fix

One line, with an existing in-file precedent — the neighbouring fixture at
`src/dispatch.rs:11500` already does this:

```rust
git(coord, &["config", "--local", "core.hooksPath", hooks.to_str().unwrap()]);
```

Pin the fixture repo's `core.hooksPath` to its own hooks dir inside
`install_recording_hook`, making the test independent of ambient config.

Worth doing at the same time: replace the bare `.unwrap()` at `:11283` with an
`.expect()` naming the sentinel and the likely cause, so the next occurrence
diagnoses itself instead of costing a bisect. That is the same class of defect
IMP-321 tracks for refusal text — an error that does not name what to do about it.

## Acceptance

- `install_recording_hook` pins `--local core.hooksPath`; the test passes with a
  global `core.hooksPath` set to an empty directory.
- The sentinel read reports the missing file and the hooksPath cause on failure.
- Optional sweep: any other test fixture depending on default hook resolution.

Related: SL-228 (REQ-389, D5 chained-hook behaviour), IMP-321 (self-diagnosing
failure text).
