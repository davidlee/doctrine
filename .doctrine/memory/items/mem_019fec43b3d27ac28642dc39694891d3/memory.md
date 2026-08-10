Binding an executable **at its resolved path** destroys `argv[0]` dispatch for
multicall binaries. Measured in a live `bwrap` capsule in the development jail,
2026-08-11.

On NixOS most of coreutils is one file. `/nix/store/…-coreutils-9.11/bin/cat` is
a symlink to `…/bin/coreutils`, and the program decides what to be from its own
`argv[0]`:

```
$ bwrap … --ro-bind /nix/store/…/bin/coreutils /nix/store/…/bin/coreutils \
      /bin/sh -c '/nix/store/…/bin/coreutils /proc/self/mountinfo'
Try '/nix/store/…/bin/coreutils --help' for more information.   # not cat

$ bwrap … --ro-bind /nix/store/…/bin/cat /nix/store/…/bin/cat \
      /bin/sh -c '/nix/store/…/bin/cat /proc/self/mountinfo'
4447 588 0:155 /newroot / rw,…                                  # cat
```

`bwrap` dereferences a bind's **source**, so both commands mount the same
multicall file. Only the **destination** basename differs, and that is what the
kernel hands the program as `argv[0]`.

## Why this bites doctrine-control

`provision.rs` step 10 binds "every bound readable path read-only **at its
resolved host path**" — source and destination are both the canonicalized path,
because `readable_set` resolves every entry before placement sees it. So a
capsule whose readable set is *individual files* can bind coreutils but cannot
invoke `cat`, `head`, `cut`, `ls`, `tr`, `pwd`, `env`, `true` or `sleep` by
name. Binding the containing **directory** keeps both the symlink and its target
inside, and dispatch works.

The consequence: a file-granular readable set is not viable on a store-managed
host with multicall binaries, however complete its closure. Directory-granular
binding is not merely more convenient there — for these tools it is the only
shape that runs.

The same reasoning covers busybox, toybox and any other multicall program, so
this is not Nix-specific; Nix is only where it shows up first, because the store
makes per-package `bin` directories the natural unit.

Related: [[mem.fact.linker.ld-trace-loaded-objects-is-the-closure]] already
records that the coreutils names are symlinks into one multicall binary — this
is the consequence that fact has for binding granularity.
