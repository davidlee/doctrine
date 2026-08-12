# EVD-019: Fresh capsule cost — freshness is free at the boundary

`IMP-426` P1a, the round's first import of `REQ-450`. The bar was a price and
this is the price. `sudo probe-freshness` on Sleipnir, off-jail. **Run 1: 20
passed, 1 failed**, the failure a defect in the probe rather than in the capsule.
**Run 2, after both corrections: 22 passed, 0 failed** — the figures below are
run 2's, with run 1 beside them.

## The figures

| | run 2 | run 1 |
|---|---|---|
| runner closure, shared by every capsule | **12,175 MiB** | 12,175 |
| cold boot to ssh — volume created, mkfs, seed | **6.41 s** | 6.34 |
| provision — 32 MiB, full history, first push | **1.90 s** | 2.26 |
| **time to a usable fresh capsule** | **8.31 s** | 8.60 |
| warm boot to ssh — volume already made and provisioned | **6.34 s** | 6.36 |
| the price of freshness — cold minus warm | **+0.07 s** | −0.02 |
| teardown — guest halts, then the VMM is terminated | **3.63 s** | void |
| volume after boot, before provision | **260 MiB** | 260 |
| volume after provision | **296 MiB** | 296 |

## What they mean

**Freshness costs nothing at the boundary, and the second run is what makes that
a finding rather than a coincidence.** Cold minus warm was −0.02 s, then +0.07 s.
It **changed sign**: the quantity is not small-and-positive, it is
indistinguishable from zero, which is the strongest form the claim can take
without more samples — a single negative reading is noise, but two readings
straddling zero locate it. Creating a 32 GiB sparse volume, making a filesystem
on it and running the seed are all unmeasurable against a 6.4 s boot. `REQ-450`'s
five axes therefore have no wall-clock price at the capsule boundary — whatever
freshness costs, it is not here. That is a stronger result than the round
expected, and it moves the question entirely onto what a fresh volume *discards*
(see the limits).

**Teardown is 3.63 s**, and a capsule's full cycle — cold boot, provision, use,
halt — is under 12 s of overhead. The figure exists only because run 1's version
of it was discarded; see the limits for what it replaced.

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

**Provisioning is not the cost either.** 32 MiB in at 1.90 s (2.26 s in run 1 —
the noisiest term measured, ±16%), against `EVD-016`'s 32 MiB out at 117 MiB/s.
The link was never the expensive part and now neither is the transaction around
it.

## Four axes rowed and green; the fifth is a deliberate absence

Asserted on a capsule nothing has used, before provisioning — so a green row
means the state is absent *by construction*, not cleaned up afterwards.

* **checkout** — the repository exists, `HEAD` is unborn, the worktree is empty.
  This is the axis the transport inversion bought: with the base commit an
  argument to a host command, a fresh capsule genuinely has no history.
* **repository** — no remote, and no `objects/info/alternates`. `REQ-448`'s "no
  writable shared object store" holds *by absence* here rather than by
  permission, which is the stronger form.
* **temporary state** — `/work/tmp`, `.cargo` and `.bun-cache` all empty.
* **runtime** — the current boot has a journal, and no previous boot survives in
  it. This is the axis run 1 got wrong: asked as a line count against
  `journalctl --list-boots`, which prints a header, so the count was 2 where the
  probe expected 1. Re-asked in run 2 as a parse-free pair (`-b 0` present,
  `-b -1` absent), it holds — and the raw count, carried along as a *figure*
  rather than an assertion, printed **2**, confirming the header explanation from
  the run after the one it explains. The property was always likely (the guest's
  root is tmpfs, so the journal is in guest RAM and cannot carry a previous
  capsule's boots whatever the volume holds), but likely is not measured, and
  run 1's record did not claim it.

The pair matters more than the fix. Asserted in one direction, "no previous boot"
passes just as well against a `journalctl` that cannot run at all — the same
wrong-reason failure `EVD-018` guards against on the network side, met again on a
different front. That the correction is a *second* assertion rather than a better
parse is the transferable part.

**Process** is deliberately **not a row**. A capsule is a separate kernel, so no
state of the volume can put a previous capsule's processes in this process
table: there is no delta capable of falsifying the reading, and a permanently
green row is misleading evidence rather than extra assurance (`DEC-189`). It is
recorded as an unrowed observation, which is that decision applied rather than
cited.

## Limits — n = 2, and they are not optional

* **Host: Sleipnir**, off-jail, one operator, two runs. Two samples bound the
  noise on the one figure that needed it (cold minus warm, which straddles zero)
  and show provision to be the loose term at ±16%. They do not make this a
  distribution, and nothing here has been run on another host.
* **Run 1's teardown figure is void — 22.68 s.** The harness waited out a VMM
  exit that firecracker is documented never to produce on guest poweroff, then
  timed its own patience. Fixed to wait on the guest going quiet; run 2's 3.63 s
  is the figure. **Do not carry 22.68 s anywhere**, including as an upper bound —
  it measured the harness, so it bounds nothing about the capsule.
* **"Cold" means a fresh volume, not a fresh store.** The runner was already
  built. Building the guest image is a separate cost and is not in the 6.41 s.
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
* **One capsule, not two.** Nothing here bears on `REQ-454`. The one figure that
  points at it is not a measurement: `target.nix`'s 16 GiB reservation.

## Related

`EVD-018` (the namespace boot this rides on), `EVD-016` (the transport, whose
inversion is why the checkout axis holds), `DEC-189` (why process is unrowed),
`DEC-192`, `IMP-426` (the round), `RFC-025`.
