Two git reads that look like pure identity are not, and both bit this repo as
latent defects (ISS-261, ISS-262 — fixed 2026-07-29, measured on git 2.55.0).

## `hash-object` applies filters by default

`git hash-object -- <path>` runs the `clean` filter and eol conversion. It does
**not** return the bytes on disk. Under a lossy clean filter two unrelated files
hash to the *same* oid (measured: both to `e96ee3ab528e21119bf96487a3cc4f4acd159834`),
so any comparison built on it reads `equal` where the bytes differ.

Use `--no-filters` whenever the contract is **bytes** (`git::worktree_blob_oid`).

The cost is real but confined: conversion needs an explicit opt-in — committed
`text`/`eol`/`filter` attributes, **or** `core.autocrlf`. With neither set,
filtered and `--no-filters` oids are byte-identical, so the flag is a no-op on an
ordinary repo. Where a repo *did* opt in, a raw worktree oid legitimately differs
from the stored (converted) blob, so comparing the two reads `diverged` for an
untouched file. That is fail-closed noise and is preferred over silent overwrite —
pinned by test in `src/git.rs`, don't "fix" it back.

`hash-object --stdin` *without* `--path` applies nothing (no path ⇒ no attributes),
so `hash_object_stdin` is not affected.

## The empty-tree oid is per repository

`4b825dc642cb6eb9a060e54bf8d69288fbee4904` is the **sha1** empty tree. On a
`--object-format=sha256` repo it is `fatal: not a valid object name` and the value
is `6ef19b41225c5369f1c104d45d8d85efa9b057b53b14b4b9b939dd74decc5321`. Derive it
with `git::empty_tree_oid(root)` (`hash-object -t tree --stdin` over empty stdin,
no `-w` — writes no object, `count-objects` unchanged). It must run **inside** the
target repo: outside any repo git answers with the sha1 value unconditionally
regardless of the target's algorithm.

## The generalisation

An oid in source is a magic string standing in for a derived fact (STD-001), and
the fact is relative to the repository's hash algorithm. Same class as DEC-055 /
IMP-325, where a width-discriminating shortcut on `verified_sha` was falsified for
the same reason. Never write an oid literal outside a test's expected value.

Still open at the time of writing: `untracked_fingerprint`'s plain `hash-object`
(`src/git.rs`) is a third instance, owned by SL-232 / DEC-089 because changing it
moves persisted csids.
