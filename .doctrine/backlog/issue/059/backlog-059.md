# ISS-059: review prime fails on a directory/symlink-to-dir slice selector — cannot hash a non-file fileset member

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

**Source:** SL-178 reconcile (RV-196 F-5 follow-on).

## Problem

`doctrine review prime RV-NNN` hashes the target slice's selector fileset. When a
selector resolves to a **directory** (or a symlink to one), hashing aborts:

```
Error: hash the slice selector fileset
Caused by: Is a directory (os error 21)
```

Observed on SL-178: selector `memory/mem.pattern.doctrine.close-drift-discharge-rec`
is a key-alias symlink → the master uid dir `mem_019f176f…/`. Legitimate
design-target (the shipped master), but `review prime` cannot fingerprint it.

## Why it surfaced now

RV-196 F-5 fixed a *different* prime break (the `.agents/skills/close/SKILL.md`
selector resolved to no tracked file). Clearing that unmasked this second cause —
the directory selector. Two independent prime defects on the same slice.

## Fix direction

`review prime` fileset hashing must handle non-file selector members: either
recurse into a directory (hash its tracked contents) or resolve a symlink to its
target dir and hash that subtree — mirroring how conformance/`record-delta`
already treat the master selector. Low stakes (prime is re-prime convenience), but
it makes RV re-priming impossible for any slice shipping a memory master.
