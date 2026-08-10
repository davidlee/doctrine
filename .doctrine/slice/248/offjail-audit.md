# SL-248 — off-jail audit of jail-contingent findings

Run off-jail on the host, 2026-08-10, against the `sl-248` clone at `34c6181eb`.
Nothing committed; nothing under `.doctrine/slice/248/**` written.

Carded-already and deliberately not re-derived (per the orchestrator's note):
items 158 (row 13 uid surfaces), 159 (row 14 capabilities), 160 (`no_new_privs`),
and `setsid`'s absence.

---

## A. Rows that are CLEAN — reproduced off-jail, verdict-for-verdict

Reported because a clean row is what shows the audit discriminates.

### `spike-deltas.sh` — all four axes identical

| axis | in-jail record | off-jail |
|---|---|---|
| `WorkingDirectory` | `/capsule` vs `/` | identical |
| `FileSizeBound` | `1048576` vs `2097152`; exit `153` vs `0` | identical |
| `WallBound` | exit `124` @2s vs exit `0` @6s | identical |
| `ProcessVisibility` | own-pid=4, host-pid visible `no` vs `yes` | identical |

The one numeric divergence is `numeric-proc-entries` on the weakened arm:
**30 in-jail, 765 off-jail** (confining arm reads 5 in both). My plan predicted
this would matter; **it does not**. The row's discriminator is
`host-pid-<N>-visible`, which reads `no`/`yes` in both environments. The entry
count is carried alongside as the `F-7` mechanism reading and the row correctly
does not rest on it. `ProcessVisibility` is not compromised.

### `spike-teardown-2x2.sh` — all six cells identical

`P` survives; `C1` 8s/no; `C2` 0s/yes; `C3` 0s/no; `C4` 0s/yes; `C1b` 0s/yes
(two pids). Cell-for-cell with the committed output. `EVD-013`'s 2×2 and the
`C1b` capture-pipe result are environment-independent.

---

## B. CONFIRMED COMPROMISED

### B1. `F-38` — the group-reader tests passed by luck, not by discrimination

The orchestrator's hypothesis, checked and confirmed:

| | live processes | with `pgrp != sess` |
|---|---|---|
| in the jail | 21 | **0** |
| off-jail host | 759 | **34** |

With zero discriminating processes, a `/proc`-based reader that slid between the
pgrp and sess field indices reads the same value either way and **cannot be
convicted**. The jail's process population is degenerate; an ordinary host with
shell job control is not. The mutation battery's green on these tests is a
property of where it ran.

Sample of discriminating processes (pid pgrp sess): `2487 2487 1564`,
`72832 72830 72379`, `72888 72830 72379`.

**Owed:** the group-reader tests need re-running against a host process
population, or a fixture that manufactures a `pgrp != sess` process rather than
relying on the ambient population. The latter is the honest fix — it makes the
test independent of where it runs, which is the whole lesson.

### B2. Row 7's ruling — the 1004 ms figure sizes the wrong arm

The 2026-08-10 ruling adopts option (b) and records: *"Measured: returned in
1004 ms with both tokens captured, and the escapee still alive afterwards in its
own session and group… That is the two-arm delta, at `D` seconds of cost rather
than `ESCAPE_SECONDS`."*

Measured off-jail, varying only the linger duration:

| arm | linger 3s | linger 8s |
|---|---|---|
| pid-ns **present** (row 7 confining) | **4012 ms** | **9012 ms** |
| pid-ns **absent** (row 7 weakened) | 1008 ms | 1008 ms |

Both arms captured both tokens in every cell.

The confining arm's cost tracks the linger exactly; the weakened arm is flat.
**Mechanism:** under `--unshare-all`, bwrap is pid 1 of the new pid namespace and
does not exit until every process in that namespace has exited. `exec 1>&-`
releases the *capture pipe*; it does not release the *namespace wait*. So option
(b) removes the stall on the weakened arm only.

This does not need a jail-vs-host argument to stand — it is checkable against the
slice's own evidence. The ruling's two clauses cannot both describe the confining
arm: `C1` and `C3` of the slice's own 2×2 both record `DESCENDANT SURVIVES: no`
under pid-ns-present, so an escapee that is *"still alive afterwards"* was
measured on a pid-ns-**absent** arm. The 1004 ms and the survival clause come
from the same arm, and it is not the one whose cost the ruling is sizing.

**Consequence:** row 7's confining arm still costs `D + ESCAPE_SECONDS` on every
suite run. The suite's cost is a floor (the ruling's own words), so this needs
re-sizing before wiring, not after.

### B3. Item 161 — independently confirmed

bwrap with **no `--unshare-*` flag at all**: `userns user:[4026535164]` against
the host's `user:[4026531837]`, `uid_map 1000 0 1`. bwrap here is not setuid
(`mode=555 owner=root`), so unprivileged userns creation is unconditional.
Anything reading `CredentialsConfined` off userns identity cannot discriminate
the flag. Confirmed.

---

## C. Instrument defects — silent-false-negative class (`F-41`'s family)

These bite whoever re-runs the spikes anywhere but the jail they were written in.

### C1. `spike-deltas.sh` does not pin the sandbox PATH

Run off-jail verbatim, **every arm returned `exit=0` with `command not found`**:
`sleep`, `cat`, `cut`, `ls`, `grep`, and `dd` all absent inside the sandbox. The
readings were `wrote 0 bytes`, `0s`, and empty proc counts — a full-table false
negative that *looks like output*.

Cause: the script inherits the ambient PATH into the sandbox. In the jail that
PATH was already store-rooted (and `/nix` is bound). Off-jail it points at
`/run/current-system/sw/bin`, which is not a bound root.

`spike-credentials.sh` **already guards exactly this** (its lines 51–61 explain
the trap and pin `SYSPATH` to the store bin dirs "so no arm can return a false
negative through an absent binary"). `spike-deltas.sh` did not inherit the guard.
The section A results above were obtained by pinning PATH externally, without
editing the tracked file.

### C2. `spike-teardown-2x2.sh` resolves `python3` on the host but runs it in the sandbox

`PYTHON=$(command -v python3)` resolves on the host PATH. The sandbox binds are
`/nix /usr /bin /lib /lib64 /etc` — no `/home`. On this host `python3` is a uv
CPython under `/home/david/.local/share/uv/…`, which resolves fine on the host
and **does not exist inside the sandbox**. Same silent no-op. Worked around by
pinning a `/nix/store` python3.

### C3. The `setsid binary on PATH` line reports the runner's PATH, not a capability

Under a pinned PATH the script printed `<absent — the EVD-013 trap>` while
`setsid` was demonstrably present on the host. As a diagnostic line that is
harmless; as the evidence behind `F-27`'s "absent in this jail" claim it is
weaker than it reads.

### C4. Survivor detection by command-line marker breaks under `exec`

Not a defect in the tracked scripts (their python escapee keeps the marker in
argv), but load-bearing for row 7's wiring: the ruling's payload ends
`exec sleep <escape>`, which **replaces the command line**. Any survivor check
that greps the escapee's cmdline for a marker will find nothing and report a
false "no survivor". Row 7's control must detect the survivor by session/pid
recorded at spawn, not by cmdline match. (Hit this in my own probe; recording it
so the wiring does not.)

---

## D. Provenance — the spike artefacts do not record their environment

- `spike-credentials-output.txt`: control arm reads `uid_map 0 0 4294967295` and
  `userns user:[4026531837]` — the **init** user namespace — with
  `parent NoNewPrivs: 0`. Produced **outside any user namespace**, i.e. on the
  host. Its bwrap store hash is `82xr5pn…`.
- `spike-deltas-output.txt`: self-records `cwd=/workspace/doctrine-SL-248` —
  genuinely in-jail. Store hash `x4m5ja…` (the same bwrap this host has).
- `offjail-prompt.md` frames the credentials spike as "the jail already measured
  this"; `notes_10-12.md:509` states "this jail's own parent reads
  `NoNewPrivs: 1`". Neither is consistent with the credentials artefact.

At least two environments are mixed in the corpus and no artefact carries its
own provenance. Cheap fix, and it is the thing that would have made this whole
audit unnecessary: have each spike print its `uid_map`, `readlink
/proc/self/ns/{user,pid,net}` and `NoNewPrivs` in its `### host` header. Then
"measured in this jail" is a fact in the artefact rather than a claim about it.

---

## E. Not settled here

- The cage's actual shape is still unmeasured (plan step 1). `src/worktree/jail.rs`
  is the *dispatch-worker* jail and `scripts/pi-spawn-confined.sh` the pi cage;
  neither is necessarily where the SL-248 agents sat. The `21 processes / 0
  discriminating` reading in B1 says the cage the orchestrator is in has its own
  pid namespace, but that is inference from a symptom, not a reading.
- `BoundedFilesystemVisibility`, `ImmutableInputSet`, `ExplicitNetworkPosture`:
  not exercised by any of the three spikes, so this audit says nothing about
  them. `notes_10-12.md:116` records `va2-write` leaving files on `/nix` and
  `/bin` — off-jail those are the real host paths, which is worth a look before
  anyone re-runs that probe outside a cage.
