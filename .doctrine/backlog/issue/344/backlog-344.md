# ISS-344: Canonicalized readable roots lose the declared name

`doctrine-control` resolves every readable input before placement and then binds
it **at its resolved path**, using that one path as both the bwrap source and the
inner destination. Canonicalization is `std::fs::canonicalize`
(`host.rs:96`), so a declared entry that is a symlink arrives inside the capsule
under its *target's* name. The declared name is destroyed, and with it two things
that depend on it.

## The two consequences

**1. Shebang interpreters do not work — the case the feature exists for.**
`SL-248` `PHASE-05` `EX-9` supports a single-file readable root precisely because
*"shebang interpreters are resolved by the kernel before `PATH` exists"*, and
says the entry is *"bound exactly as declared"*. But on any ordinary host
`/bin/sh` is a symlink — `/usr/bin/dash` on Debian, a store path on NixOS — so a
declared `/bin/sh` becomes:

```
--ro-bind /usr/bin/dash /usr/bin/dash      # what happens
--ro-bind /usr/bin/dash /bin/sh            # what EX-9 describes
```

and `#!/bin/sh` still fails inside. This repository's own provisional config
(`.doctrine/doctrine.toml:59`) declares `readable-roots = ["/bin/sh",
"/usr/bin/env"]`, so the contradiction is live, not hypothetical.

**2. Multicall binaries lose their identity, not merely their path.** A program
like coreutils or busybox decides what to be from its own `argv[0]`. Binding
`/nix/store/…-coreutils/bin/cat` at the resolved `…/bin/coreutils` mounts the
right file under the wrong name, and `cat` becomes uninvokable — as do `head`,
`cut`, `ls`, `tr`, `pwd`, `env`, `true` and `sleep`. Measured in a live capsule,
2026-08-11; see `mem.fact.capsule.resolved-path-bind-breaks-multicall-dispatch`.

## The contradiction is authored, not a drift from spec

`PHASE-05` has two exit criteria that cannot both hold:

- `EX-9` — *"`readable-roots` is bound exactly as declared."*
- `EX-3` — *"a declared readable input keeps its **resolved** host path as its
  inner path, identity-mapped, which is what makes the derived inner `PATH`
  computable at all."*

The implementation follows `EX-3`. So this is not an implementation that drifted
from its spec; it is two criteria of one phase in conflict, with `EX-3`'s
rationale — computability of the derived inner `PATH` — as the reason the
collapsing was chosen. Any fix has to answer that rationale, which makes this a
governance question as much as a code one.

## Why nobody saw it

`a_single_file_readable_root_is_bound_as_declared`
(`bubblewrap.rs:1830`) models the host as `/bin/sh → /bin/sh`. That is false on
every ordinary Linux host and true inside this project's development jail, where
`/bin/sh` is a **bind mount of bash rather than a symlink** (`ls -l /bin/sh`
shows a regular file). It is the fourth instance of the pattern `ISS-339`,
`ISS-340` and `ISS-341` each recorded: a derivation that is accidentally correct
in the single environment it has ever run in.

## Candidate shape

Resolve the host source, preserve the declared inner name — `MountedPath`
already carries the two separately and provisioning simply collapses them
(`provision.rs:879`). A declaration of `…/bin/cat` resolving to
`…/bin/coreutils` would mount `host = …/bin/coreutils`,
`inner = …/bin/cat`. Security validation keeps operating on the fully resolved
host object; the capsule receives the interface the operator declared;
`argv[0]` survives; `/bin/sh` stays `/bin/sh`.

`EX-3`'s rationale then needs a replacement: derive the inner `PATH` from the
inner destinations reconstructed beneath the host's declared `PATH` entries,
rather than asking whether a canonical host `PATH` directory lies beneath a
canonical bound source. That would also fix the bootstrap problem below.

Note what this shape does **not** do on its own: `expand_closure_root`
(`bubblewrap.rs:924`) builds the bound set from resolver output only and never
adds the realised root it was given, so a closure root still is not bound unless
the resolver echoes it. Making the declared root supply the mapped executable is
an *additional* production change, not a consequence of the mapping fix.

## Adjacent, and probably the same repair

`clone_inside` (`provision.rs:949`) runs all four git operations as bare `git`,
and `BubblewrapBackend::run` derives the capsule's `PATH` from the readable
placement immediately before each execution. So any readable-set shape that
yields an empty inner `PATH` breaks *provisioning*, before a single conformance
row runs. The jail masks this exactly as it masks the rest: there every `$PATH`
entry is already an individual package `bin` directory.

## Provenance

Surfaced during `SL-252`'s design run (`inq-2`), by measurement rather than
review, and sharpened by an external reviewer reading the slice's context dump.
`SL-252` has not decided whether it expands to cover this or defers it.
