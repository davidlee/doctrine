# EVD-018: Firecracker boots with its tap inside a network namespace

The last unknown in `PLAN_C`'s netns shape, and the one everything downstream
was waiting on. `probe/netns.sh` established the *namespace* behaviour with
veths and a simulated guest; the host module was verified against microvm.nix's
pinned source. Neither of those is a boot. This is the boot.

## What ran

`sudo probe-netns-boot` in `/workspace/microvm-spike`, on Sleipnir, off-jail.
The probe deliberately breaks this repo's own rule about never borrowing live
addressing — it uses the real tap name, the real `/30` and the real volume,
because the thing under test is the real guest image, which has `net.nix` baked
into it. A probe on other addressing would boot a different guest and answer a
different question. It refuses to start beside the devshell tap or a running VM.

```
PASS  the VMM starts with its tap inside the namespace
PASS  the guest boots and answers ssh from inside the namespace
PASS  the guest's NIC is live on the namespaced tap
PASS  the root namespace cannot see the tap
PASS  the root namespace cannot reach the guest
PASS  the root namespace cannot reach the guest's ssh port
PASS  the relay's socket appears in the root namespace
PASS  ssh crosses it as the human, unprivileged
PASS  git speaks over the same socket

9 passed, 0 failed
```

Both directions are asserted, which is the methodology `probe/netns.sh` paid for
twice: reachable *inside* the namespace and unreachable *outside* it. A
denial-only network test passes for the wrong reason.

## What it settles

**The tap resolves in firecracker's own namespace.** It opens `/dev/net/tun` and
`TUNSETIFF`s by interface name, both resolved where the process runs — reasoned
from source, now run. A tap created *inside* the namespace works, so the
fallback (create in root, move in) is not needed.

**The way in costs no privilege.** A `socat` relay on a unix socket in
`/run/capsule/<name>/ssh.sock`, reached by an `ssh` `ProxyCommand`. The
filesystem is not namespaced, so this needs no port allocation, no
socket-activation fd passing, and no root — and identical guest addresses never
reach `known_hosts`, because the socket path is the identity.

**It retires `EVD-016`'s one open limit.** That record's "settled since capture"
noted git over the netns `ProxyCommand` was unproven — `probe/netns.sh` had
carried only raw bytes across that bridge, via `socat`. Git has now crossed it as
git. `NOTES` item 18's "not measured: git over the netns unix-socket
`ProxyCommand`" is closed.

## The correction that must travel with this datum

**`RFC-025` asserted this result before it existed, and the overstatement ran in
the direction that is hardest to catch — doctrine's record was ahead of the
spike's.** § *The microVM turn* → *Where this stands* said *"The namespace boot
worked (2026-08-11) — firecracker comes up with its tap inside a capsule
namespace **on the host-module path**"*, and *"The spike's `NOTES.md` is
updated."* The spike's own working tree, staged roughly six hours later, said the
opposite in three places: `NOTES` item 17 *"Unrun as of this commit"*, `PLAN_C`
*"the last unverified thing in the shape"*, and `HANDOVER.md` *"Not verified …
That is the one thing still blocking `PLAN_C`."*

The proposition is now true. It became true from this run, not from the earlier
claim, and one clause of the earlier claim is still false: **this was not the
host-module path.** See the limits below. Same family as `EVD-016`'s "a
correction that was itself wrong" — the failure is asserting a result from the
reasoning that predicted it.

## Limits — n = 1, and they are not optional

* **Host: Sleipnir**, the owner's NixOS host, off-jail, one operator. **Run
  twice**, 9/9 both times — the second as the regression on extracting the boot
  sequence into `probe/harness.sh`, which is why n went up for free rather than
  by spending a run on it. Still one host and one operator, which is the limit
  that matters.
* **Not the host-module path.** The namespace and the tap were created by the
  probe and the runner was started by hand as the human. No systemd units, no
  `NetworkNamespacePath` drop-ins on `microvm@` / `microvm-tap-interfaces@`, no
  host rebuild — which is the point of the probe (the unknown was firecracker's,
  not systemd's), but it means the host-module wiring is still bookkeeping
  against a known-good result rather than itself demonstrated.
* **Egress was not tested, on purpose.** There is no upstream in the namespace at
  all, so "the guest cannot reach the internet" would pass for the wrong reason.
  Egress under netns is stage 2 of `probe/netns.sh` plus a proxy joined to the
  namespace, and belongs with the module that creates the namespace for real.
* **One namespace, one guest.** Identical addressing across two *concurrent*
  capsules is modelled by `probe/netns.sh` with veths and no VM; it is not
  demonstrated with two real guests. That is the round's P1b (`REQ-454`).
* **Nothing here bears on freshness.** The probe boots the *existing* volume by
  design. No `capsule-provision` ran, so `REQ-450`'s five axes are untouched —
  that is P1a.
* **Timing is bounded, not measured.** Firecracker logged its start at
  `08:09:15.186` and the guest halted at `13.706` s uptime, so boot, ssh, three
  assertion stages, a git operation and shutdown all fit inside **13.7 s** of VM
  lifetime. Boot-to-ssh-ready is strictly less than that and was not separately
  timed; the runner build ran before the clock started and is excluded. Treat the
  figure as a ceiling on the whole cycle, not as a boot time.
* The console shows `reboot: Power off not available: System halted instead` —
  the known firecracker-does-not-exit edge, handled by shutting the guest down
  over ssh before terminating the VMM. Any freshness claim that ignores it is
  measuring the wrong thing.

## Related

`EVD-016` (whose netns limit this retires), `IMP-426` (the round, whose P1a and
P1b this unblocks), `RFC-025` (the programme, whose claim this both confirms and
corrects), `EVD-015`.
