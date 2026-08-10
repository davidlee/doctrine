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


---

# CORRECTION (2026-08-10, SL-252) — the conclusion above is the defect

**Do not implement the rule this memory recommends.** The two-step derivation —
top-level ancestor, minus operator regions — is `system_readable_roots` in
`crates/doctrine-control/src/conformance.rs`, and it was raised as a defect
(`ISS-341`) after the conformance suite's first off-jail runs (`ISS-339`).

## What is wrong

*"The closure of anything under a top level is inside that top level, by
construction"* is true and is the trap. On a store-managed host the top level is
`/nix`, so the derivation binds **the entire package store**: 691 store paths
measured off-jail, with `git`, `curl` (TLS-capable) and `gcc` all executing
inside the sandbox. `--ro-bind` is read-only, not `noexec`. A capsule whose
defining property is a bounded input set was given a larger executable set than
an ordinary unconfined process.

Two host `PATH` entries nobody uses — `/run/wrappers/…` and
`/var/lib/flatpak/exports/bin` — also drag in the host's whole `/run` and `/var`,
including `/run/user/1000` with its keyring, gnupg and pipewire sockets.

Doctrine's *production* profile already refuses this shape and says so:
`SL-248` `PHASE-05` `EX-8` — no package store is ever bound whole, no
host-shaped default, no fallback.

## What is still true here, and worth keeping

- **A `bin` directory is not a closure.** Correct, and it is the constraint that
  makes this hard. Bind the directory alone and every binary is `-x` and exits
  127.
- **`PATH` routinely names operator state**, so an exclusion step is mandatory.
- **Resolve through symlinks before deriving anything.** `bwrap` dereferences a
  bind's source, so the resolved path is the one that matters.
- **The discriminating fixture** — a `PATH` spanning both a lawful top level and
  the operator's — is the right test shape.

## The rule that replaces it

Bind the `PATH` entry directories that supply a declared toolset, plus each
tool's **runtime closure**, each member bound individually and never their
common parent. Get the closure from the dynamic loader —
[[mem.fact.linker.ld-trace-loaded-objects-is-the-closure]] — which needs no Nix,
no `ldd` and no ELF parser. Measured: 16 store paths where the ancestor rule
bound 691.

Decisions: `DEC-185` (the bar is representativeness, not production parity),
`DEC-186` (the resolver is a self-written loader-trace script). Trust dropped to
low and severity raised: this memory is retained for the analysis it contains,
not for the rule it names.
