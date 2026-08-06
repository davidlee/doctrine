`bwrap` **dereferences the source path** of a `--ro-bind` / `--bind`. A mount
entry whose declared source is a symlink binds the symlink's *target*, not the
link.

So an allowlist validated **lexically** — normalise the string, check it is not
at or under a forbidden prefix — does not confine anything. A declared readable
root of `/opt/tools` that happens to be a symlink to the canonical repository
passes every string check and makes the canonical repository readable inside the
sandbox.

## The rule

Resolve first, validate the **resolved** path, and bind the resolved path. All
three, in that order, so no window exists in which the validated path and the
bound path differ.

## How this was found, and the transferable part

`SL-248`'s design asserted the opposite — that symlinks were deliberately not
followed, framed as a usability cost rather than an escape. Nothing in reading
the design or the `bwrap` flags contradicted it. It was refuted by an external
reviewer *executing* a probe: a declared directory symlinked to an out-of-tree
target, whose marker file was readable inside the sandbox (`RV-346` `F-1`).

The transferable lesson is the one the `SL-241` spike already recorded twice
under different names (its `F-P02-2` and `F-P05-17` shebang findings): **a claim
about what a confinement mechanism does or does not expose is not established by
reading its flags.** Sandbox APIs resolve, inherit, and default in ways their
option names do not state. Any such claim entering a design owes an executed
probe, and a probe paired with a one-property-removed control if it is a denial
claim.

The same class, second instance, recorded in `DEC-155` rather than here: `bwrap`
network is default-**open** and only `--unshare-net` closes it (`jail.rs:806`),
so a capsule profile must *invert* the worktree jail's default rather than
inherit it. Both instances are defaults you cannot read off an option name.
