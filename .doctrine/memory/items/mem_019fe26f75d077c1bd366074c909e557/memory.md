## The problem

A confined payload needs *some* executables — a shell, `git`. The obvious move
is to bind the directories named on the host's `PATH`. That fails twice, in
opposite directions.

## Too narrow: a `bin` directory is not a closure

`--ro-bind /nix/store/abc-git/bin /nix/store/abc-git/bin` puts `git` in the
namespace and it still will not run: a dynamically linked executable needs its
loader (`ld-linux`) and its libraries, which on a store-based host live under
*sibling* directories of the same store. Measured on NixOS with bwrap 0.11.2:
binding the `bin` directory alone fails; binding `/nix` runs
`git version 2.54.0`.

The rule that works is the **top-level ancestor** — the first component under
`/`. `/nix/store/abc-git/bin` → `/nix`; `/usr/bin` → `/usr`. It is coarse on
purpose: the closure of anything under a top level is inside that top level, by
construction, without resolving a single `DT_NEEDED`.

A second, unrelated benefit: top-level ancestors are **pairwise
non-overlapping**, so a placement validator that refuses overlapping mount
destinations is satisfied with no deduplication pass beyond set membership.

## Too wide: `PATH` routinely names the operator's home

`$HOME/.local/bin`, `~/bin`, a language version manager's shim directory — all
common `PATH` entries. Take their top-level ancestor and you get `/home`, and
now the sandbox has read access to the operator's ssh keys, git credentials and
token stores. The coarseness that fixed the first problem causes the second.

So the derivation is two steps, not one:

1. top-level ancestor of the resolved shell and of every resolved `PATH` entry;
2. **drop any candidate that contains operator state** — `$HOME`, the working
   directory, and the harness's own scratch root.

## Testing it

The discriminating fixture is a `PATH` spanning **both** a lawful top level and
the operator's, e.g. `/nix/store/abc-git/bin:/home/operator/.local/bin` with
`HOME=/home/operator`. A `PATH` of store paths alone passes under an
implementation that has no exclusion step at all, so it proves nothing.

Resolve entries through symlinks *before* taking the ancestor: `/bin/sh` is
frequently a symlink into the store, and the unresolved form yields `/bin`,
which may be a different (and less sufficient) answer than the real one.
