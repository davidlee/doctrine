# QUE-212: How does a result leave a shareless capsule?

## The collision

`DEC-135` chose **Git bundle** ingestion for v0 result transport, under a
structural rule: *trusted control-plane code never runs Git in a
capsule-authored repository.* `REQ-451` binds it — the control plane snapshots
hostile bytes **once** under path, symlink, quiescence, time, byte and object
bounds — and `REQ-452` completes it: trusted Git reads only that parent-owned
snapshot, through a fresh quarantine repository.

The microVM spike does something else. Having **no host filesystem shares** —
which is most of why the boundary is strong — the result has to leave over a
channel, and the channel is a live `receive-pack`: the guest pushes to a host
git-daemon into a bare mirror, ref-restricted to `refs/heads/capsule/*` by an
update hook (`perimeter/default.nix`, `vm/capsule.nix:40`), and the owner fetches
`capsule/*` back out.

That is a **trusted host process inserting capsule-authored objects into a
repository the owner then fetches from**. The spike's mitigations — dedicated
system uid, ref-namespace restriction, `IPAddressAllow`/`Deny`, cgroup ceilings —
bound the *blast radius* of a `receive-pack` compromise. They do not reproduce
snapshot-once-under-bounds, and there is no quarantine step.

So the question is not "which is safer in the abstract" but: **does the
hypervisor boundary change the transport calculus enough to revise `DEC-135`, or
does the bundle discipline survive into the VM by a different channel?**

## Candidates to price

1. **Keep push-to-mirror; revise `DEC-135`.** Argue that no-shares plus uid split
   plus ref restriction bounds the hazard at least as well. Cost: `REQ-451`'s six
   bounds have no analogue and `REQ-452` is abandoned rather than satisfied.
2. **Bundle over the existing ssh channel.** The guest writes a bundle; the host
   reads it as bytes and snapshots. `DEC-135` survives intact. The spike already
   has host→guest ssh, so the channel exists.
3. **Bundle on an outbox block device.** A small dedicated virtio-block volume:
   guest writes the bundle, VM powers off, host mounts read-only and snapshots.
   No live process ever receives hostile bytes, the snapshot boundary is physical
   rather than procedural, and `receive-pack` leaves the perimeter entirely —
   shrinking git-daemon to read-only clone-in, which `NOTES.md` item 8 flags as
   the surface worth shrinking. This is the natural VM idiom and the spike
   already has volumes.

Candidate 3 is the one this record expects to win and the one most worth
measuring first; it is stated as an expectation, not a finding.

## Why it gates the import

`IMP-426` imports unproven requirements into the spike to shape and price them.
`REQ-451` and `REQ-452` cannot be imported until this is answered, because their
subject — the snapshot — may not exist in the VM's transport at all.

## Related

`DEC-135`, `REQ-451`, `REQ-452`, `IMP-426`, `RFC-025`, `red-team.md` `RT-1`,
`CPT-002` (the ingestion path is anti-heresy machinery, not confinement, so it
carries more of the residual risk once the wall improves).
