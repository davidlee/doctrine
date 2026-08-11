# EVD-016: Host-initiated git works in both directions over the capsule ssh channel

Measured 2026-08-11 in the `RFC-025` capsule round, against the already-booted
spike guest. Recorded because `QUE-212`'s lead candidate was reasoned, not run,
and `IMP-426`'s reporting bar is a price rather than a proof.

## The two probes

| | command | result |
|---|---|---|
| **result out** — host-initiated fetch | `git init --bare q.git` then `git -C q.git fetch ssh://agent@10.99.0.2/work/doctrine '+refs/heads/*:refs/capsule/x/*'` | 66,436 objects / 32.03 MiB at 96 MiB/s into `refs/capsule/x/*` |
| **provisioning in** — host-initiated push | guest: `git init /work/scratch` + `receive.denyCurrentBranch=updateInstead`; host: `git push ssh://agent@10.99.0.2/work/scratch edge` | 66,537 objects / 32.05 MiB, `new branch edge -> edge`; unborn `HEAD` on `edge` checked out |

## What it establishes

1. **The inversion is available on the channel that already exists.** No new host
   service, no new guest program, no bundle. `vm/capsule.nix` already provisions
   host→guest ssh with key auth and ships `pkgs.git` in the guest, so
   `upload-pack` and `receive-pack` are both present guest-side.
2. **Transfer cost is not a discriminator.** ~32 MiB each way for a
   doctrine-sized repo at ~100 MiB/s: the tap is not the bottleneck, and no
   direction is preferable on throughput.
3. **`updateInstead` handles the unborn-`HEAD` provisioning case**, and it
   genuinely checked the tree out: `/work/scratch`'s `HEAD` is
   `refs/heads/edge` and its worktree is populated. Pushing straight into the
   guest's working repo is viable; the bare-intermediary fallback
   (`/work/origin.git` + guest-side clone) is not needed to seed an empty repo.
   It remains the answer for re-provisioning over a dirty worktree, which was
   not tested.

## The coupling this depends on, which nothing previously stated

`receive.denyCurrentBranch=updateInstead` governs **only** pushes to the branch
`HEAD` points at. Provisioning therefore works because the guest's
`/etc/gitconfig` sets `init.defaultBranch = edge`, so a bare `git init` in the
guest leaves `HEAD` on `refs/heads/edge` — the ref being provisioned. Push a ref
that is *not* the current branch and the guard never applies: the push succeeds,
the ref is created, and the worktree is left **empty**, silently.

So the guest's default-branch config is load-bearing transport machinery, not a
convenience. A `git checkout -b feature` inside the guest breaks the next
provision with no error. Anything that seeds a capsule should point `HEAD` at the
ref it is about to push (`git symbolic-ref HEAD refs/heads/<ref>`) rather than
inherit it.

**Where this was nearly recorded wrongly.** A reproduction run *outside* the
guest — without that `/etc/gitconfig` — lands `HEAD` on `master`, observes an
empty worktree, and concludes the original measurement misread its own result. It
did not. The mechanism is real and worth the explicit fix; the retraction would
have been an artefact of reproducing the commands without the environment.

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
* **Settled since capture:** `/work/scratch` ends with a populated worktree, on
  `refs/heads/edge` — see the coupling above.
* **Not measured:** cost under `transfer.fsckObjects`; cost of a
  dev-tools-sized rather than repo-sized transfer; whether fsck rejects anything
  the present guest-push path accepts today.

## Related

`QUE-212` (the question this settles), `DEC-192` (the decision it supports),
`EVD-017` (the escalation the same session turned up), `ASM-010` (which removed
the third candidate before it could be probed), `IMP-426` (the round).
