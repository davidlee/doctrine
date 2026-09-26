# `memory edit --key` produces a key that cannot be resolved

## Symptom

`doctrine memory edit <uid> --key <key>` reports success and writes
`memory_key = "<key>"` into `memory.toml`, but the key is then **unresolvable**:

```
$ doctrine memory edit mem_019ed9f59a8f7f6398145b4a99c59f62 --key mem.example.key
Edited memory mem_019ed9f59a8f7f6398145b4a99c59f62

$ doctrine memory show mem.example.key
Error: memory not found: mem.example.key

$ doctrine memory validate
mem_019faca1f05277729cb407f8d4487206: dangling: [[relation]] target "mem.example.key" not found
```

## Cause

Key resolution is **symlink-only** — `resolve_show`'s own doc (`src/memory.rs`):

> a uid hits the real dir, a key hits the slug symlink the filesystem resolves —
> there is **no** `memory_key` scan fallback, so a stale hand-edited key with no
> live symlink is a not-found.

Keyed memories live in a `mem_<uid>/` directory with a sibling symlink named by
the key (`592` such symlinks under `.doctrine/memory/items/` at the time of
writing). `apply_edit`'s `--key` leg inserts the toml field and nothing else, so
the late-bind manufactures precisely the "stale hand-edited key with no live
symlink" state the resolver doc calls a not-found. `memory list` meanwhile reads
`memory_key` from the toml, so list and show disagree.

Note the resolver is correct; the **writer** is incomplete.

## Remedy

Either mint the symlink in `apply_edit`'s `--key` leg (matching what
`memory record --key` does at creation), or refuse the late-bind with a message
naming the missing step. Minting is preferable — `--key` exists to be usable.

## Workaround (used in the 2026-09-26 memory review)

`ln -s mem_<uid> <key>` inside `.doctrine/memory/items/`. This heals the dangling
relation and makes the key resolvable, but it should not be a hand procedure.

## Adjacent

Not the same defect as [[IMP-335]] — that one is about *creation* silently
minting a tracked symlink sibling and not saying so. This is about a write verb
failing to mint the one artifact its own resolver requires.
