# ASM-010: The host never mounts guest-authored filesystems

Owner's standing ruling, 2026-08-11:

> I'd take it as a standing assumption that mounting the guest filesystem is a
> Dumb Idea, and that any evidence it's likely to provide is not worth the risk.

## The assumption

**The host never hands guest-authored filesystem metadata to the host kernel.**
No `mount` of any volume a capsule could have written — work volume, outbox,
snapshot, or a disk image pulled aside for troubleshooting.

Held as an assumption rather than a measured constraint **deliberately**: the
second clause is the operative one. This is not "we have established the risk
exceeds the benefit"; it is "we decline to spend the effort establishing it,
because no plausible finding would change the answer." An `ASM` is the honest
kind for a premise we are choosing not to test.

## Why mounting is the wrong shape, specifically

The VM boundary exists to keep guest bytes away from the host kernel. A kernel
mount hands guest-controlled metadata **directly to a host kernel filesystem
parser** — which is a large, historically fertile attack surface, and one reached
before any policy applies. Mounting therefore *inverts the value of the
boundary*: it opens the one channel the hypervisor was chosen to close.

**`ro`, `nosuid`, `nodev` and `noexec` do not mitigate this**, and believing they
do is the trap. Those options constrain what the *mounted content* may do once
mounted. The parse of the superblock, journal and directory structures happens
**before** any of them constrain anything. An earlier draft of this round's
packet advised pricing exactly that combination; it was wrong and this record
supersedes it.

## Where the line actually falls — userspace parsers are fine

This bans the **kernel** parser, not inspection. `fuse2fs` and `debugfs` parse
ext4 in an unprivileged userspace process, where a parser bug yields a
compromised userspace process rather than a kernel privilege escalation. That is
a different and acceptable risk class, and it is the same class already accepted
for `git index-pack` over hostile pack data.

The spike already holds this line, in `vm/capsule.nix`: *"with fuse2fs or
debugfs, never `mount`: it is guest-written ext4 and mounting it hands the
metadata to the host kernel."* This record elevates that from one repo's comment
to a doctrine-side standing assumption, so it governs the capsule design and not
only this spike.

## Where it will actually be violated

Not in a design decision — in a **debugging session**. "Just let me look at the
disk" during a stuck boot is the realistic breach, and it will be made by someone
who has not read this record, at the moment they are least inclined to. Any
tooling that pulls a capsule volume aside for inspection should reach for
`fuse2fs`/`debugfs` by construction rather than by discipline.

## What it costs

1. **The outbox-block-device transport is dead** as a design (`QUE-212`
   candidate C), unless read without a filesystem parse at all. It was already
   the fallback after the transport was reframed as direction rather than format;
   this removes it outright.
2. **No host-side disk forensics.** Any exhibit `DEC-133` wants — forensic
   artefact, deterministic replay input — must leave through the same narrow
   channel as the result, or be produced host-side after ingestion. It cannot be
   recovered by opening the guest's disk afterwards.
3. **A raw-offset byte read is not a sanctioned workaround.** Technically it
   avoids the kernel parser, but reaching for it to serve the same goal is the
   loophole this assumption exists to close. If bytes need to come out, use the
   channel.

## What would invalidate it

Effectively nothing we intend to look for — which is the point. Recorded so that
a future agent proposing a host mount is arguing against a standing position
rather than filling a silence, and so that the position is visible if it is ever
deliberately revisited.

## Related

`QUE-212` (transport; candidate C removed by this), `IMP-426`, `CPT-002`,
`DEC-133`, `DEC-191` (kernel attack surface is one of its escape fronts),
`RFC-025`, `ADR-020`.
