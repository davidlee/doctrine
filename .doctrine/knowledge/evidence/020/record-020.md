# EVD-020: Two concurrent capsules — declared RAM is a ceiling, not a reservation

`IMP-426` P1b, the round's import of `REQ-454` — a candidate verified in a
*separate* fresh capsule from the one that produced it, ranked highest in the
round by `CPT-002` because it is the substrate for the defence against the
threat that actually arrives. `sudo probe-two-capsules` on Sleipnir, off-jail:
**28 passed, 0 failed**, first run.

## The figures

| | |
|---|---|
| capsule A answers ssh, from launch | **6.94 s** |
| **both** capsules answer ssh, from launch | **7.12 s** |
| declared guest RAM, per capsule (`target.nix`) | 16,384 MiB |
| **host `MemAvailable` consumed by both, booted and idle** | **1,488 MiB** |
| both provisioned from different refs, in series | 3.51 s |
| volume, capsule A / capsule B | 295 / 295 MiB |
| teardown of one capsule with its sibling up | 3.56 s |
| processes matching `microvm@capsule`, host-wide | **2** |
| ...and in each capsule's own namespace | **1 / 1** |

## What they mean

**The 16 GiB reservation is not a reservation.** Two capsules declaring 32 GiB
between them cost **1,488 MiB** of `MemAvailable` once booted — roughly 744 MiB
each. Firecracker does not preallocate, and the guest root is tmpfs, so a capsule
occupies what it has touched and nothing more. This **corrects `EVD-019`**, which
inferred from `target.nix` that "the binding constraint at N is the 16 GiB RAM
reservation per capsule". It is not: the declaration is a *ceiling* on what one
capsule may reach, not a charge levied at boot. The correction is exactly the
kind `DEC-189` predicts — a row reasoned from configuration rather than measured
reads as evidence and is not.

What replaces it is worse-behaved and honest: the real constraint at N is the sum
of what the capsules actually *touch*, which is workload-dependent and unmeasured
here. Two idle capsules are cheap. Two capsules each running a `cargo build`
against a 16 GiB ceiling and a tmpfs root is the question, and this probe does not
ask it.

**The second capsule is free in wall-clock.** Both answer ssh 7.12 s after
launch, against 6.94 s for the first and `EVD-019`'s 6.41 s for a single cold
boot. The marginal cost of concurrency is ~0.2 s on a 32-core host: they boot
alongside each other rather than behind each other. Teardown likewise — 3.56 s
with a sibling running, against `EVD-019`'s 3.63 s alone.

**Two capsules, one image, two histories.** Both booted from a single runner
store path and were provisioned from *different* base commits, each landing at
the commit it was pinned to. That is the transport inversion's premise
(`DEC-192`) discharged rather than argued: with the base commit an argument to a
host command, N capsules need one image. `EVD-019` priced the image at
12,175 MiB; this is the evidence that N of them share it.

**Results come back attributable.** Each capsule collected into its own
quarantine, and what came out of each is what went into it. `REQ-454` needs the
verifier's verdict distinguishable from the worker's claim, and the collect side
already carries the name that distinguishes them.

## The independence that has nothing to configure

Both guests are the same image at the same address in their own namespace. A
marker file written on each volume and read back through that shared address
returns *A* in A's namespace and *B* in B's.

So a capsule cannot reach its sibling **because it cannot name it** — the address
it would use is its own. This is stronger than the property the design started
out wanting. A dropped route is a control: it can be misconfigured, and something
has to keep configuring it. Identical addressing is not a control; there is
nothing to get wrong, and the failure mode "the forward drop did not stop
cross-capsule reach" (which the spike already paid for once) has no analogue
here. The root namespace, meanwhile, cannot reach that address at all.

## The identity finding, and what it cost before the run

**One name matches every capsule; the namespace matches one.** Host-wide, two
processes answer to `microvm@capsule`; each namespace holds exactly one. That is
a direct consequence of the one-image lever — N capsules run the same runner from
the same store path — and it means a VMM cannot be identified by name once N > 1.

This surfaced as a live hazard while the probe was being written, not as a
result of running it. The shared harness tore a capsule down with
`pkill -f microvm@capsule` and then asked whether a microvm was running: with a
sibling up, that kills both and **reports success**, because the second question
is satisfied by a host that has neither. A teardown that passes by destroying
more than it was asked to is the failure mode worth naming.

The fix needs no new machinery: `ip netns pids` scopes both the query and the
signal, so **the thing that isolates a capsule is the same thing that names it**
— no pidfile, no registry, no allocation scheme. Both counts are now reported on
every run, so the day they agree is visible rather than silent.

One asymmetry is left standing and is a cost at N: `capsule-collect` takes its
capsule's name as an argument, `capsule-provision` bakes its socket path into a
store path. Two capsules therefore need two provision programs.

## Limits — n = 1, and they are not optional

* **Host: Sleipnir**, off-jail, one operator, **one run**. Unlike `EVD-019`
  nothing here has a second sample.
* **Two capsules, not N.** 2 is the number `REQ-454` needs (worker and verifier)
  and it is the number that was run. Nothing here establishes 3 or 4, and the
  memory figure in particular is the one most likely to stop being linear.
* **Idle, not working.** The memory figure is two booted capsules that have run a
  provision and nothing else. It is a floor, and the interesting number — two
  concurrent builds against a 16 GiB ceiling each — is not measured.
* **Egress is still untested under netns**, as in `EVD-018`: there is no upstream
  in either namespace, so nothing here says anything about two capsules sharing
  the perimeter. The proxy-to-proxy interface-pair drop that `probe/netns.sh`
  found remains the open question there.
* **Not the host-module path**, for the same reason as `EVD-018`: the namespaces,
  the taps and the sockets were made by the probe. No systemd units, no
  `capsule-netns@`, no host rebuild.
* **`MemAvailable` is a kernel estimate**, sampled before boot and after both
  guests answered ssh, on a host doing other things. Treat 1,488 MiB as an order
  of magnitude — the load-bearing claim is that it is ~1.5 GiB and not ~32 GiB.

## Related

`EVD-019` (whose RAM inference this corrects, and whose image figure this shows
is shared), `EVD-018` (the namespace boot both ride on), `DEC-192` (the inversion
that puts the base commit in an argument), `DEC-189` (a row reasoned from
configuration is not evidence — met here as the thing corrected), `CPT-002`,
`REQ-454`, `REQ-448`, `IMP-426`, `RFC-025`.
