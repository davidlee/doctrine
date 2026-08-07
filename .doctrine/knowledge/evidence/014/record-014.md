# EVD-014: Bubblewrap credential confinement — what a delta can and cannot falsify

Measured while preparing `RV-346` round 6 for `SL-248`, to settle whether
`sec-7` table A row 13's one-property-removed control can fire. `bwrap` 0.11.2
throughout. **Two hosts**: first inside this project's bubblewrap jail, then
unjailed on the real NixOS host (Linux 7.1.6 x86_64) — the second run is what
closes the legs the first could not reach.

## Method

Each arm runs a reporting payload — `id`, `/proc/self/status`,
`/proc/self/ns/user`, `/proc/self/uid_map` — under a different flag set. No arm
writes, binds a real path, or acts. Script and verbatim capture:
`.doctrine/slice/248/spike-credentials.sh`, `spike-credentials-output.txt`;
brief and answers in `spike-credentials.md`.

Three controls on the method itself, all needed:

- **A no-bwrap arm** (`P`), so a field that never moves is distinguishable from
  a field the payload cannot read.
- **The user-namespace inode compared against the parent's**, because the
  credential report alone cannot say which cage produced a result (`RV-346`
  `F-20`). What is load-bearing is only that each `bwrap` arm's inode differs
  from the init namespace — **the values themselves are not stable across runs**,
  since the namespaces are torn down at arm exit and the kernel recycles the
  numbers. Every other column reproduces byte-for-byte.
- **Absolute paths for the payload's binaries.** Two unjailed runs were
  discarded before the reported one. Run 1 returned `command not found` for
  `id`, `grep`, `sed`, `readlink` and `tr` in every `bwrap` arm while `P`
  reported normally — NixOS's login `PATH` points at `/run/current-system/sw/bin`
  and `/run` is deliberately unbound. That is **`EVD-013`'s shape exactly**: five
  arms agreeing because none of them could speak. Fixed by pinning the sandbox
  `PATH` to `/nix/store` bin directories already reachable through the existing
  `/nix` bind, adding nothing to the bind set. Run 2 then broke `P` alone, that
  `PATH` having no shell; fixed by invoking `/bin/sh` absolutely so the control
  runs the identical environment to the arms.

`bwrap` is mode `555`, root-owned, **not setuid**, on both hosts, and the
unjailed host permits unprivileged user namespaces
(`max_user_namespaces=247154`). That is load-bearing for everything below and is
the first thing any repetition should re-check.

## Result — unjailed host

| arm | uid | gid | Groups | CapInh | CapPrm / CapEff | CapBnd | NoNewPrivs | new userns |
|---|---|---|---|---|---|---|---|---|
| `P` — no bwrap | 1000 | 100 | 9 real gids | `0000000800000000` | `0` | **`000001ffffffffff`** | **`0`** | — (init) |
| `--unshare-all` | 1000 | 100 | 9, unmapped | `0` | `0` | **`0`** | **`1`** | yes |
| `--unshare-user` dropped | 1000 | 100 | 9, unmapped | `0` | `0` | `0` | `1` | yes |
| `--uid 4242 --gid 4242` | **4242** | **4242** | 9, mapped entry moved | `0` | `0` | `0` | `1` | yes |
| `--cap-add ALL` | 1000 | 100 | 9, unmapped | **`000001ffffffffff`** | **`000001ffffffffff`** | **`000001ffffffffff`** | `1` | yes |

The in-jail run agrees on every column except `P`'s, which read all-zero
capabilities and `NoNewPrivs: 1` before any `bwrap` — the jail having already
established what the measurement was trying to observe.

## What it settles

1. **`--unshare-user` is a no-op, so row 13's delta as specified cannot fire.**
   The two arms are identical on every credential column on both hosts. A
   non-setuid `bwrap` must create a user namespace to obtain `CAP_SYS_ADMIN` for
   its own mounts, and does so whether or not the flag is passed — both arms
   created distinct *new* namespaces, neither inherited. The control can never
   fail, row 13 can only read `Unproven`, and `Admission::Admitted` is
   unreachable. This is `sec-7` `B4`'s class — a control whose delta cannot
   change what its payload observes — in the row added one round after `B4`.

2. **Bubblewrap sets `no_new_privs`; it does not inherit it.** `P` reads `0`,
   every arm reads `1`. `sec-7`'s claim that it is bubblewrap's default is now
   corroborated by measurement rather than assumed. The jail could not answer
   this, having read `1` before any `bwrap` ran.

3. **Bubblewrap does observably strip capability authority — but not in the two
   fields row 13 names.** `P` carries a full bounding set
   `CapBnd=000001ffffffffff` and a non-empty `CapInh=0000000800000000`
   (`CAP_WAKE_ALARM`); every `bwrap` arm reads zero on both. `CapPrm` and
   `CapEff`, by contrast, are all-zero in the unprivileged parent *and* in every
   arm — they are all-zero for any unprivileged process on any host, so they can
   never show bubblewrap removing anything. **The signal lives entirely in
   `CapBnd` and `CapInh`.** Row 13 holds when *both capability sets are empty*,
   meaning effective and permitted, which is true vacuously and discriminates
   nothing.

4. **Supplementary groups are never cleared, so row 13 cannot hold as written.**
   `P` shows nine real gids; every arm shows nine entries still, the mapped gid
   in its original position and the other eight rendered as the overflow `65534`.
   The groups are *unmapped*, not dropped — a user namespace may not `setgroups`
   — so the list has the same cardinality in every arm. Row 13 holds only when
   the list is empty and it is empty in no arm. (A prediction was falsified here,
   against the brief rather than against the jail: real gids were expected
   outside in place of the in-jail `65534`, and the unmapped entries render
   `65534` on the real host too.)

5. **`--cap-add ALL` fires, so the property is falsifiable after all.** It moved
   all four capability sets from zero to `000001ffffffffff`, exiting clean, on
   both hosts and without refusing — so it does not depend on already being
   inside a user namespace. Unjailed it moves `CapBnd` from a *measured-full*
   parent through a *measured-empty* probe arm, which is a stronger observation
   than the jail could make. These are capabilities within the capsule's own user
   namespace rather than host root, which is precisely the threat `sec-2`
   invariant 15 names: a capsule holding `CAP_SYS_ADMIN` in its namespace can
   mount what was never bound. `--uid`/`--gid` moves identity on a separate axis.

## What this leaves for the design, and does not decide

Row 13 has four holding conditions and **three are wrong**: mapped identity is
unmet (the profile passes no `--uid`/`--gid`), empty supplementary groups is
unmeetable under bubblewrap, and *both capability sets are empty* names the two
fields that carry no signal. Only `no_new_privs` survives, and it is now measured.

The candidate delta is `Widened`-shaped, not `PropertyRemoval`-shaped — it *adds*
to the capsule's posture where every other delta in `sec-7`'s table *removes* a
mechanism. Whether *one control removes exactly one mechanism* survives a control
that grants rather than withholds is `RV-346` round 6's to rule on, and is
deliberately not settled here.

Relates to [[DEC-156]] (one-property-removed controls), [[EVD-013]] (the same
question answered for teardown, and the source of the positive-control rule this
method reuses — and reused twice more here), [[SL-248]] `sec-7` row 13 and
`sec-2` invariant 15.
