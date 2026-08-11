# IMP-426: Evidence spike: fresh-per-transaction microVM capsule

Evidence, not product — the `SL-241` shape, and its go/no-go discipline. This is
the **first** move of the capsule rescue: `RSK-231` calls for an architectural
revision rather than another remediation slice, and this is the measurement that
revision needs before anything is designed.

## The question

Can a doctrine-driven Firecracker microVM capsule be **fresh per transaction** at
tolerable cost on this project's hardware?

`SPEC-030` `REQ-450` (FR-003) requires that every phase worker *and every
verifier* launch headlessly in fresh mutable capsule state, across `Axis`'s five
freshness axes — checkout, repository, runtime, temporary state, process.
`DEC-134` fixes fresh-context-per-phase as the v0 topology.

## Why it comes first

Conformance measures a capsule. **There is no per-transaction capsule yet.**
`/workspace/microvm-spike` is the opposite shape by design:

- one persistent VM with a persistent `/work` volume holding a real clone;
- `vm` is build-then-exec in the **foreground**, not a daemon — `pgrep -af
  'microvm@'` is "the entire inventory; nothing is registered anywhere";
- driven interactively over ssh;
- more than one capsule at a time is *"scoped, not started"* (`PLAN_C.md`).

If the answer is no, the conformance work has nothing to measure and the rescue
is moot. If yes, `DEC-190`'s kernel and `DEC-189`'s re-derived row set have a
subject.

## What the spike already gives it for free

`NOTES.md` item 17 has done the scoping and names the deciding cost — **the guest
image, not the plumbing**: the obvious design pays N image blobs (N × disk, N ×
*pack* time on every `dev-tools` bump, though not N × build, since the closure is
almost entirely shared). Getting the two per-instance values out of the closure —
the guest's address and the **base commit** — buys one image and N small runners.
A kernel param does not work for either: it lands in `toplevel`, so in the
closure.

And **netns-per-capsule has been spiked and it holds** (`spike/netns.sh`): two
capsules, a guest that already has root, no path to the upstream, no path to the
sibling, unreachable from outside even by something holding a route to it, while
a process in the namespace reaches the internet normally. Flipping the
namespace's `ip_forward` proves the switch is what does the work. It makes the
guest bit-identical across instances — one image, no DHCP, no boot-time step —
and gives a forward control that is *ours* rather than one shared with docker and
tailscale.

## What to measure

1. Wall-clock to a usable fresh capsule, cold and warm.
2. Disk and pack cost per instance under the one-image-plus-runners shape versus
   N blobs.
3. Whether all five freshness axes are actually satisfied, or only some.
4. Teardown: does the VMM exit, is the tap released, is state genuinely gone.
   (Known sharp edge: firecracker does **not** exit on guest poweroff — it halts
   the vCPU and keeps the tap, so the next start fails `Device or resource busy`.)
5. Two concurrent capsules, since `red-team.md` `RT-1` requires verification in a
   *separate* capsule from the worker.

## Scope discipline

No conformance suite, no doctrine integration, no production code. Bounded
measurement with a written go/no-go, on the `SL-241` precedent — whose verdict
was explicitly scoped and did **not** claim production readiness or portability.

## Related

`RSK-231`, `RFC-025`, `SPEC-030` `REQ-450`, `DEC-134`, `EVD-015`, `DEC-189`,
`DEC-190`, `DEC-191`, `IMP-397` (capsule egress allowlist and build-input
provisioning, which this will brush against).
