four things. Everything else in that message is in the audit.

Non-derivable: the reproduction recipe (which store paths I pinned, and that the raw numbers came from an externally-pinned PATH rather than the scripts as-committed), the friction observation's path, plan step 1's exact command block (it's in the plan file, not the audit), and the fact that this session is still live off-jail and can take follow-up measurements.

the audit sits at `offjail-audit.md`. 

---

```
Off-jail audit for SL-248 is done. Findings are additive to items 158–161 —
I did not re-derive those.

Friction observation recorded:
.doctrine/observations/records/55/019fe985-80a1-77f1-a33a-dddc1d2f8f55.toml

## Three findings you need to card

**F-38 confirmed — the group-reader tests pass by luck.**
Jail: 21 live processes, 0 with pgrp != sess. Off-jail host: 759 live, 34 with
pgrp != sess (e.g. pid/pgrp/sess 2487/2487/1564, 72832/72830/72379). A /proc
reader sliding between the pgrp and sess field indices reads identically where
nothing discriminates, so the mutation cannot be convicted in the jail. The green
is a property of the process population, not the code. Recommended fix is not
"re-run off-jail" but a fixture that manufactures a pgrp != sess process, so the
test stops depending on where it runs.

**Row 7's 2026-08-10 ruling sizes the wrong arm.** Measured off-jail, varying
only the linger:

  pid-ns present (confining):  linger 3s -> 4012 ms;  linger 8s -> 9012 ms
  pid-ns absent  (weakened):   linger 3s -> 1008 ms;  linger 8s -> 1008 ms

Both arms captured both tokens in every cell. Under --unshare-all bwrap is pid 1
and will not exit until the namespace drains; `exec 1>&-` releases the capture
pipe, not the namespace wait. So option (b) removes the stall on the weakened arm
only, and the confining arm still costs D + ESCAPE_SECONDS on every suite run.
The suite's cost is a floor, so this wants re-sizing before wiring.

This is checkable without my host: the ruling records the escapee "still alive
afterwards" at 1004 ms, but C1 and C3 of the slice's own 2x2 both record
DESCENDANT SURVIVES: no under pid-ns-present. The timing and the survival clause
come from the same arm, and it is not the confining one.

**Wiring hazard for row 7.** The ruling's payload ends `exec sleep <escape>`,
which replaces the command line. Any survivor check that greps the escapee's
cmdline for a marker will find nothing and report a false "no survivor". Detect
by session/pid captured at spawn. (Hit this in my own probe.)

## Two rows are clean — reproduced off-jail

spike-deltas: WorkingDirectory, FileSizeBound, WallBound, ProcessVisibility all
identical verdict-for-verdict. Only numeric divergence is numeric-proc-entries on
the weakened arm, 30 in-jail vs 765 off-jail (confining reads 5 in both) — the
row discriminates on host-pid-visible, which reads no/yes in both, so it does not
rest on the count. Not compromised.

spike-teardown-2x2: all six cells identical (P yes; C1 8s/no; C2 0s/yes;
C3 0s/no; C4 0s/yes; C1b 0s/yes, two pids). EVD-013's 2x2 and the C1b
capture-pipe result are environment-independent.

## Instrument defects — F-41's family, and the reason section A took two runs

spike-deltas.sh does not pin its sandbox PATH; it inherits the ambient one. In
the jail that PATH is already store-rooted and /nix is bound. Off-jail it points
at /run/current-system/sw/bin, which is not a bound root — so run verbatim, every
arm returned exit=0 with `command not found` for sleep/cat/cut/ls/grep/dd:
"wrote 0 bytes", "0s", empty proc counts. A full-table false negative that looks
like output. spike-credentials.sh already carries exactly this guard (its lines
51-61 name the trap). The sibling did not inherit it.

spike-teardown-2x2.sh has the same shape via python3: PYTHON=$(command -v python3)
resolves on the host, but the sandbox binds are /nix /usr /bin /lib /lib64 /etc —
no /home — and on this host python3 is a uv CPython under /home. Also, its
"setsid binary on PATH" line reports the runner's PATH, not a host capability; it
printed <absent> under my pinned PATH while setsid was demonstrably present.

IMPORTANT for reproduction: section A's numbers were obtained by pinning PATH
externally, without editing either tracked script. Recipe:

  CU=/nix/store/di26b1kkbammy0sj70nq5qzvfrh78wxl-coreutils-9.11/bin
  GG=/nix/store/aak8d9mrdv9sgn0lcg7xss7wxdg9sqh3-gnugrep-3.12/bin
  GS=/nix/store/7mfsmbhsd3arnypipwm251rcd0b45riy-gnused-4.10/bin
  PP=/nix/store/w88w0pk3yz1zdcpccc2cz4x0ddhv5n0z-procps-4.0.6/bin
  BW=/nix/store/x4m5ja2330if46sw08lgwwa57bqw9adm-bubblewrap-0.11.2/bin
  UL=/nix/store/4235prclh3wvcqzri7hxssb47i6d444j-util-linux-2.42.2-bin/bin
  PY=/nix/store/0r6k8xa2kgqyp3r4v2w7yrb80ma2iawm-python3-3.13.12/bin
  BD=$(dirname "$(readlink -f /run/current-system/sw/bin/bash)")
  env PATH="$CU:$GG:$GS:$PP:$BW:$UL:$BD" "$BD/bash" spike-deltas.sh

## Provenance defect

spike-credentials-output.txt reads uid_map "0 0 4294967295", userns
user:[4026531837] and parent NoNewPrivs: 0 — the init user namespace. It was
produced outside any user namespace, i.e. on the host, and its bwrap store hash
(82xr5pn...) differs from the in-jail deltas artefact's (x4m5ja...).
spike-deltas-output.txt self-records cwd=/workspace/doctrine-SL-248 and is
genuinely in-jail. offjail-prompt.md frames the credentials spike as jail-measured
and notes_10-12.md:509 says this jail's parent reads NoNewPrivs: 1; neither is
consistent with that artefact. At least two environments are mixed and none of
the artefacts carry provenance. Cheap durable fix: have every spike print
uid_map, readlink /proc/self/ns/{user,pid,net} and NoNewPrivs in its ### host
header, so "measured in this jail" becomes a fact in the artefact.

## What I could not settle, and what I need from you

The cage's own shape is still unmeasured. B1's 21-processes/0-discriminating
reading implies your cage has its own pid namespace, but that is inference from a
symptom. If you paste the output of this from inside the jail I can close it:

  grep -E '^(CapInh|CapPrm|CapEff|CapBnd|NoNewPrivs|Seccomp):' /proc/self/status
  readlink /proc/self/ns/user /proc/self/ns/pid /proc/self/ns/net /proc/self/ns/mnt
  cat /proc/self/uid_map /proc/self/gid_map
  ls /proc | grep -c '^[0-9]'
  id -u; id -g
  command -v setsid; readlink -f /proc/1/exe

Also untouched by any of the three spikes, so this audit says nothing about them:
BoundedFilesystemVisibility, ImmutableInputSet, ExplicitNetworkPosture. Worth
knowing before anyone re-runs the va2-write probe outside a cage —
notes_10-12.md:116 records it leaving files on /nix and /bin, which off-jail are
the real host paths.

This off-jail session is still live and can take follow-up measurements.
```
