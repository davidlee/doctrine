When a test measures that a sandbox *removed* something, and the test itself
runs inside another sandbox that already removed it, two claims come apart and
are routinely conflated:

1. **The probe is not vacuous** — the observed field can still vary between the
   confining arm and a weakened one, so the test can fail.
2. **The confinement is attributable to the backend under test** — the zero the
   probe reads was produced by *this* backend rather than inherited.

(1) can hold while (2) fails. SL-248: inside a bubblewrap jail, all four
capability sets read zero on the **trusted** side, yet `--unshare-all` reads
zero and `--cap-add ALL` reads `000001ffffffffff` — so the probe discriminates
(1) while saying nothing about who produced the zero (2). The mechanism that
rescues (1) is `create_user_ns()` setting `cap_bset = CAP_FULL_SET` in a nested
user namespace, so the outer cage's stripped set is not inherited.

**What settles (2) is an unconfined positive control** — the same payload with
no sandbox at all, run somewhere the property is present. Off-jail that read
`CapBnd 000001ffffffffff` and `CapInh 0000000800000000`, which is what turns the
mechanism argument into a measurement. Note the second field: it is *zero* in
the cage and *non-zero* on the host, so a cage-only run can never demonstrate it
discriminates.

**Practice.**
- Design the arm set as probe / weakened-control / **unconfined control**, not
  just the first two. The third is the one a cage cannot supply.
- When you cannot leave the cage, say which of (1) and (2) you established, and
  do not report them as the same evidence.
- Some surfaces the cage defeats entirely: `no_new_privs` reads 1 on the trusted
  side, so provenance is unestablishable in there at any arm count. Compute such
  a caveat from the trusted side's own reading rather than hard-coding it, so it
  is correctly *absent* where the attribution is real — and test both branches.
- An artefact should print its own environment (`uid_map`,
  `readlink /proc/self/ns/{user,pid,net}`, `NoNewPrivs`) in its header, or
  "measured on the host" is a claim about the file rather than a fact in it.