# ISS-339: The conformance suite has never run off-jail

Nobody has run the shipped conformance suite on a real host.

## What was actually confirmed off-jail

The off-jail work ran **spikes** — shell reproductions of the backend's argv
shape. So what is confirmed on a real host is the *mechanism* rows 13 and 14 rest
on, not those rows executing there. `VA-1` asked for the credential measurements
and got them; this is not a shortfall against it.

## Why now

It is unusually cheap. `PHASE-10` `T13` made `doctrine-control backend verify`
exit **0** in-jail with all nineteen rows `Proven`, so the confirmation is a
single command on the owner's own shell.

And it is the only thing that would close `sec-9` residual 3's neighbourhood by
**measurement** rather than by argument.

## The audit's own evidence that this is not theoretical

`RV-352` ran the suite from a jail built without this slice's `flake.nix`
addition. `setsid` did not resolve, seven `conformance::tests` convicted, and
`backend verify` returned `not-admitted` with `Property(ProcessTreeTeardown)`
`Unproven`. The guard behaved correctly — but the episode is a live demonstration
that the suite's host preconditions are real, undocumented outside `flake.nix`,
and unexercised anywhere but this one jail.

## First off-jail run, 2026-08-10 — it found a real defect

The owner ran `just capsule-verify` on a NixOS host outside the jail. Every one
of the nineteen rows came back `Indeterminate { NoLiveness }`, both `sec-3` and
`sec-4` claims `Failed`, both `sec-9` observations `Unread`, all with one cause:

```
bwrap: execvp /bin/sh: No such file or directory
```

**Root cause.** `system_readable_roots` derived the bind set from the shell's
*resolved* path, but every payload execs the *literal* `SHELL` — `/bin/sh`.
Readable roots are identity-mapped, so on a store-based host, where `/bin/sh` is
a symlink into `/nix/store` and `/bin` is not on `PATH`, the capsule was given
`/nix` and no `/bin`: the libraries without the entry point. Fixed on `sl-248`
(`8f837c312`) by making the literal and the resolved shell two candidates.

**Why the in-jail runs never saw it.** In the development jail `/bin/sh` is a
bind-mounted *real file*, so the resolved path is its own top level; and `/bin`
is separately a `PATH` entry. Either accident alone puts `/bin` in the roots.
The suite has only ever run where both held. This is exactly the class of thing
this item was opened to find, and it argues the item's premise was
under-stated — the preconditions were not merely undocumented, one of them was
wrong.

**Still owed, which is why this stays open:** a *green* off-jail run. The fix is
verified by unit test (`a_readable_root_covers_the_literal_shell_the_payloads_exec`)
and by a bwrap positive control, but no real host has yet been seen to admit the
backend, and a second precondition may be sitting behind the first.

**What it hands `IMP-417`.** The failure was mute: a host structurally unable to
exec a payload reported nineteen indeterminate rows rather than naming the
missing mount. `admission`'s existing shell guard could not catch it — it asks
`path_exists("/bin/sh")`, which follows the symlink and passes. So the first
concrete member of `IMP-417`'s precondition probe is known and recorded there.

## Relationship to `EX-14` in CI

Closing this by measurement does not settle `EX-14`'s CI question (`RV-352`
`F-8`) — that is a criterion-level decision about what a runner unable to satisfy
the backend should report. But an off-jail run is the cheapest way to learn what
the suite's real host requirements are before that decision is taken.

## References

`SL-248` `notes.md` § *Owed* item 175(b) · `sec-9` residual 3 · `VA-1` ·
`RV-352` `F-8` · sibling of `ISS-338`
