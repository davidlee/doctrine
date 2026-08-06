# A sandboxed reviewer measures its own cage

When an agent running inside a sandbox reports that some confinement mechanism
*fails*, the first hypothesis is its own sandbox — not the subject.

## The case

`RV-346` `F-20` (raiser codex, `sandbox: workspace-write`) reported that this
project's development jail could not create a network namespace:

```
bwrap --ro-bind / / --dev /dev --proc /proc --tmpfs /tmp --unshare-all -- /bin/sh -c 'echo nested-ok'
→ loopback: Failed to create NETLINK_ROUTE socket: Operation not permitted
```

It even ran discriminating controls, which isolated the failure to the
denied-network leg — good technique, and it still reached the wrong conclusion.
The identical command and all three controls succeed when run in the jail
directly. Codex's `workspace-write` sandbox denies network syscalls by seccomp,
and `socket(AF_NETLINK, …)` is what bubblewrap opens to bring up loopback in a
fresh netns.

## Why the discriminating controls did not save it

Every control ran *inside the same cage*, so they discriminated between legs of
the profile while holding the confounder constant. A control that cannot vary
the thing actually responsible is not a control for it — the same defect the
subject design was being reviewed for (see [[mem.pattern.review.control-must-remove-the-unique-mechanism]]).

## What to do

- Reviewing confinement, isolation, namespaces, or syscall availability: run
  the reviewer **outside** the sandbox, or treat every negative as unconfirmed
  until reproduced outside it.
- Receiving such a finding: reproduce it yourself before remediating. A
  positive control — the same command succeeding elsewhere — is what settles it.
- Keep the general form when the specific claim dies. `F-20`'s attribution was
  wrong and its mechanism was real: any layer denying network-namespace setup
  breaks a network-posture probe, and a seccomp-filtered CI runner is such a
  layer. That half was worth more than the claim.

The mirror image of [[mem.fact.capsule.bwrap-ro-bind-dereferences-source]],
where the same reviewer was right for the same reason — it executed a probe
instead of reading flags. Execution is necessary; knowing *where* you executed
is what makes it evidence.
