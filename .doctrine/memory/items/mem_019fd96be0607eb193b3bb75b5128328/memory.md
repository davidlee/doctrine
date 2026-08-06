Measured on **bubblewrap 0.11.2** while preparing `RV-346` round 6 for `SL-248`.

Unprivileged (non-setuid) `bwrap` must create a user namespace to perform its
own mounts, and it does so **whether or not `--unshare-user` is passed**. Both
of these produced a *new* userns with an identical `uid_map`, differing only in
inode:

```bash
bwrap --unshare-all                    ... /bin/sh -c 'readlink /proc/self/ns/user; cat /proc/self/uid_map'
bwrap --unshare-pid --unshare-ipc --unshare-uts --unshare-cgroup --unshare-net \
                                       ... # same, --unshare-user dropped
```

The credential reports were byte-identical across the two: same uid and gid,
same non-empty supplementary groups, `CapPrm` and `CapEff` both zero,
`NoNewPrivs` 1.

**Why this bites.** A conformance suite in the `DEC-156` shape proves a property
by removing the one mechanism that enforces it and requiring the control arm to
fail. Dropping `--unshare-user` looks like the removal for *process credential
confinement* and is not one — it changes nothing observable, so the control can
never fail, the row reads `Unproven`, and admission is unreachable. That is the
unfalsifiable-control class, not a gap in coverage: the row exists, runs, and
carries no information.

Two adjacent facts measured at the same time, both cage-independent:

- **`--unshare-all` does not clear supplementary groups.** A userns cannot
  `setgroups`, so the list survives; unmapped gids surface as the overflow gid
  (65534). A probe holding on *supplementary groups are empty* cannot pass under
  bubblewrap.
- **`no_new_privs` is bubblewrap's default**, not a flag the profile passes, so
  it is set under every namespace combination and carries no signal for any
  namespace-shaped delta.

**Establish the cage before believing any of this** (`RV-346` `F-20`). This
project's jail is itself a userns with capabilities already stripped and
`NoNewPrivs` already 1, so a naive A/B inside it is uninformative for a
different reason. What settles it is comparing `/proc/self/ns/user` and
`/proc/self/uid_map` between arms, not comparing the credential report.

Related: [[mem.pattern.doctrine.tdd-loop]] — a control that cannot fail is the
confinement analogue of a test that cannot go red.
