# EVD-015: Effort-to-confidence: bwrap week versus microVM afternoon

Observed 2026-08-11, in the `RFC-025` capsule-rescue round. Recorded because
`RSK-231`'s claim is about a **rate × cost** curve, and a claim about a curve
wants a datum rather than a feeling.

## The two figures

**Bubblewrap.** `SL-248` (*Capsule provisioning and Linux backend*) created
2026-08-05, `done` 2026-08-10 — ten phases, executed and audited. `SL-252`
(*Conformance fixture readable-input posture*) opened and **abandoned at design**
on 2026-08-10/11 without reaching implementation. Six days of intense agent work.
What it has *not* produced: a backend meeting doctrine's own criteria on the
owner's NixOS host. `REQ-459` criterion 2 stands `partial` in `REV-051`.
`SL-248` threw off **24 open follow-up items** plus `ADR-021` `proposed`.

Four of those are one pattern — `ISS-339`, `ISS-340`, `ISS-341`, `ISS-344`: *a
derivation that is accidentally correct in the single environment it has ever run
in*. The suite had never run off-jail until 2026-08-10; its first off-jail run
was red.

**microVM spike** (`/workspace/microvm-spike`). 19 commits, first
2026-08-11 13:26 +1000, last 2026-08-11 23:46 +1000 — **~10.3 hours elapsed on
one calendar day**. (The owner recalled "6 hours"; elapsed is the measured figure
and working time is unknown and lower. Elapsed is not effort.) In that window: a
booting Firecracker guest holding a real clone of the target repo, a
point-to-point tap with no default route and no guest resolver, a host perimeter
whose forward-drop is **verified rather than assumed** with a fail-closed
three-state check that refuses to start on `open`, dedicated system uids and
cgroup ceilings for the proxy and git-daemon via an exported NixOS module, and a
netns-per-capsule design spiked and holding (`spike/netns.sh`).

## What this supports

`RSK-231`'s thesis: the bubblewrap surface produces host-sensitive defects at a
rate, and costs a lot to engage with per defect, and the product of those two
exceeds what the programme can sustain. Six days against ten hours is consistent
with that, and with the owner's judgement that patching forward keeps the cost
curve where it is rather than bending it.

## What this does not support, and the limits are not optional

1. **Not like-for-like.** The spike has not attempted `REQ-450`'s
   fresh-mutable-state requirement — the five freshness axes — at all. It is one
   persistent VM with a persistent `/work` volume, `vm` is build-then-exec in the
   foreground rather than a daemon, and more than one capsule at a time is
   *"scoped, not started"* (`PLAN_C.md`). A large share of what `SL-248`'s six
   days bought is exactly the cost the spike has not yet paid.
2. **The spike has no conformance suite.** Its claims are design reasoning and
   manual checks. `SL-248`'s are fourteen properties and five axes with
   one-property-removed controls, two independent gate runs at `297 passed;
   0 failed`, and `backend verify` exit 0 (`RV-352` § Synthesis). These are not
   the same kind of claim and the comparison must not read as though they were.
3. **The spike carries known holes.** microvm.nix has no jailer support, so
   firecracker itself runs unwrapped as uid 1000 with no chroot, no netns and no
   uid drop (its in-process seccomp filter *is* on); and there is no
   `MemoryMax`/`CPUQuota`/`TasksMax` on the VM (`NOTES.md` items 11–12).
4. **n = 1, and not independent.** One host, one operator, and the same operator
   authored both systems. Nothing here is replicated.

## The claim it does *not* license

That a microVM backend is cheaper overall. That is unmeasured, and the expensive
part — determinism and per-transaction freshness — is precisely the part not yet
attempted. This datum bears on *where the difficulty was concentrated*, not on
total cost.

## Related

`RSK-231` (supports), `RFC-025`, `DEC-188`, `QUE-211`.
