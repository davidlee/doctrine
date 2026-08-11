# EVD-019: Fresh capsule cost — freshness is free at the boundary

`IMP-426` P1a, the round's first import of `REQ-450`. The bar was a price and
this is the price. `sudo probe-freshness` on Sleipnir, off-jail, run 1:
**20 passed, 1 failed**, the failure a defect in the probe rather than in the
capsule (below).

## The figures

| | |
|---|---|
| runner closure, shared by every capsule | **12,175 MiB** |
| cold boot to ssh — volume created, mkfs, seed | **6.34 s** |
| provision — 32 MiB, full history, first push | **2.26 s** |
| **time to a usable fresh capsule** | **8.60 s** |
| warm boot to ssh — volume already made and provisioned | **6.36 s** |
| volume after boot, before provision | **260 MiB** |
| volume after provision | **296 MiB** |

## What they mean

**Freshness costs nothing at the boundary.** Cold and warm boots differ by
−0.02 s, which at n = 1 means *indistinguishable*, not *faster*. Creating a
32 GiB sparse volume, making a filesystem on it and running the seed are all
unmeasurable against a 6.3 s boot. `REQ-450`'s five axes therefore have no
wall-clock price at the capsule boundary — whatever freshness costs, it is not
here. That is a stronger result than the round expected, and it moves the
question entirely onto what a fresh volume *discards* (see the limits).

**The one-image lever has a number, and it is the whole argument.** `NOTES`
item 17 reasoned that the deciding cost of N capsules is the guest image, and
that getting the per-instance values out of the closure buys one image and N
small runners instead of N blobs. The blob is **11.9 GiB**. At the 3–4 capsules
this design is scoped for, the netns shape avoids 24–36 GiB of duplicated store,
and every `dev-tools` bump costs one pack instead of N. Both per-instance values
are now out: the base commit left with the transport inversion (`DEC-192`), the
address with the namespace (`EVD-018`).

**Marginal disk per capsule is ~296 MiB, and most of it is nothing.** 260 MiB
of that exists before any content does — ext4 metadata for a 32 GiB declaration.
The actual work costs ~36 MiB, consistent with a 32 MiB repository. So the
volume *size declaration* is the tunable, not the content, and a capsule is
cheap on disk in a way the sparse `ls -l` figure (32 GiB) actively hides.

**The binding constraint at N is RAM, not disk or time.** `target.nix` reserves
16 GiB per capsule. Two concurrent capsules is 32 GiB of the host's memory
before anything else runs, against ~600 MiB of disk and one shared image. That
is a configuration fact rather than a measurement, and it is what `REQ-454`
(P1b) will run into first.

**Provisioning is not the cost either.** 32 MiB in at 2.26 s, against
`EVD-016`'s 32 MiB out at 117 MiB/s. The link was never the expensive part and
now neither is the transaction around it.

## The four axes that hold, and the two that are not rows

Checked on a capsule nothing has used, before provisioning:

* **checkout** — the repository exists, `HEAD` is unborn, the worktree is empty.
  This is the axis the transport inversion bought: with the base commit an
  argument to a host command, a fresh capsule genuinely has no history.
* **repository** — no remote, and no `objects/info/alternates`. `REQ-448`'s "no
  writable shared object store" holds *by absence* here rather than by
  permission, which is the stronger form.
* **temporary state** — `/work/tmp`, `.cargo` and `.bun-cache` all empty.

**Runtime** was asked as a line count against `journalctl --list-boots` and came
back red. That listing prints a header, so the count was 2 where the probe
expected 1. The property almost certainly held — the guest's root is tmpfs, so
the journal is in guest RAM and cannot carry a previous capsule's boots whatever
the volume holds — but *almost certainly* is not a measurement, and this record
does not claim it. Re-asked as a parse-free pair (`-b 0` present, `-b -1`
absent); unproven until that runs.

**Process** is deliberately **not a row**. A capsule is a separate kernel, so no
state of the volume can put a previous capsule's processes in this process
table: there is no delta capable of falsifying the reading, and a permanently
green row is misleading evidence rather than extra assurance (`DEC-189`). It is
recorded as an unrowed observation, which is that decision applied rather than
cited.

## Limits — n = 1, and they are not optional

* **Host: Sleipnir**, off-jail, one run, one operator. Every figure is a single
  sample; the cold-minus-warm difference is well inside what a second run would
  move.
* **The teardown figure from this run is void.** It reported 22.68 s, which was
  the harness waiting out a VMM exit that firecracker is documented never to
  produce on guest poweroff, and then timing its own patience. Fixed to wait on
  the guest going quiet. **Do not carry 22.68 s anywhere** — the real teardown
  cost is unmeasured.
* **"Cold" means a fresh volume, not a fresh store.** The runner was already
  built. Building the guest image is a separate cost and is not in the 6.34 s.
* **The closure figure is the runner's**, via `nix path-info -S` — the guest
  image plus what the runner needs to start it. That is the right subject for
  the one-image question, because the runner is what an N-blob design would
  duplicate, but it is not "the guest image" on its own.
* **The largest cost freshness imposes was not measured and could not be here.**
  A fresh volume discards the build cache — that is the axis holding, and it is
  the price. The first `cargo build` on a fresh volume needs egress, and this
  probe's namespace has no upstream at all, so nothing in the guest can fetch a
  crate. Measuring it needs the proxy joined to the namespace, i.e. the host
  module. Recorded as not-measured rather than estimated.
* **One capsule, not two.** Nothing here bears on `REQ-454`.

## Related

`EVD-018` (the namespace boot this rides on), `EVD-016` (the transport, whose
inversion is why the checkout axis holds), `DEC-189` (why process is unrowed),
`DEC-192`, `IMP-426` (the round), `RFC-025`.
