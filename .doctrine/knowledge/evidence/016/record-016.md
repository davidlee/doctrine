# EVD-016: Host-initiated git works in both directions over the capsule ssh channel

Measured 2026-08-11 in the `RFC-025` capsule round, against the already-booted
spike guest. Recorded because `QUE-212`'s lead candidate was reasoned, not run,
and `IMP-426`'s reporting bar is a price rather than a proof.

## The two probes

| | command | result |
|---|---|---|
| **result out** — host-initiated fetch | `git init --bare q.git` then `git -C q.git fetch ssh://agent@10.99.0.2/work/doctrine '+refs/heads/*:refs/capsule/x/*'` | 66,436 objects / 32.03 MiB at 96 MiB/s into `refs/capsule/x/*` |
| **provisioning in** — host-initiated push | guest: `git init /work/scratch` + `receive.denyCurrentBranch=updateInstead`; host: `git push ssh://agent@10.99.0.2/work/scratch edge` | 66,537 objects / 32.05 MiB, `new branch edge -> edge`; unborn `HEAD` accepted |

## What it establishes

1. **The inversion is available on the channel that already exists.** No new host
   service, no new guest program, no bundle. `vm/capsule.nix` already provisions
   host→guest ssh with key auth and ships `pkgs.git` in the guest, so
   `upload-pack` and `receive-pack` are both present guest-side.
2. **Transfer cost is not a discriminator.** ~32 MiB each way for a
   doctrine-sized repo at ~100 MiB/s: the tap is not the bottleneck, and no
   direction is preferable on throughput.
3. **`updateInstead` handles the unborn-`HEAD` provisioning case.** Pushing
   straight into the guest's working repo is viable; the bare-intermediary
   fallback (`/work/origin.git` + guest-side clone) is not needed to seed an
   empty repo. It remains the answer for re-provisioning over a dirty worktree,
   which was not tested.

## The correction that must travel with this datum

*"The host's refspec fully decides the destination namespace"* is **false as
stated**. The fetch also wrote `refs/tags/*`, outside `refs/capsule/x/*`, via
git's automatic tag following. Harmless in a disposable quarantine repo, fatal to
the claim: `--no-tags` (or `remote.<name>.tagOpt`) is required before the
unqualified form holds. Do not write the unqualified version into a requirement,
an acceptance row or a downstream `EVD`.

## Limits — n = 1, and they are not optional

* **Host: Sleipnir**, the owner's NixOS host. Hand-run commands, not scripted,
  not repeated, not run off-jail on any other machine.
* **Current tap shape, not the netns design.** git over the netns unix-socket
  `ProxyCommand` is *separately unproven*: `probe/netns.sh` has carried a TCP
  session to guest:22 across that bridge with `socat` — raw bytes — never a git
  session.
* **Not checked:** whether `/work/scratch` ends with a populated worktree or only
  a moved ref. `ls /work/scratch` settles it.
* **Not measured:** cost under `transfer.fsckObjects`; cost of a
  dev-tools-sized rather than repo-sized transfer; whether fsck rejects anything
  the present guest-push path accepts today.

## Related

`QUE-212` (the question this settles), `DEC-192` (the decision it supports),
`EVD-017` (the escalation the same session turned up), `ASM-010` (which removed
the third candidate before it could be probed), `IMP-426` (the round).
