# SL-248 spike — can a credential posture be falsified under bubblewrap?

**For an agent running outside this project's bubblewrap jail.** Everything
below is measurement. Do not edit the design, do not choose a remedy, do not
reason past what the arms report. Fill the results table, answer the six
decisions, hand it back.

`RV-346` round 6 is briefed and held pending this. Its first line of attack asks
codex to rule on a mechanism for `sec-7` table A row 13, and ruling before this
runs would be the exact failure the design has just been caught committing —
`R1`, a claim about evidence made without checking the evidence reaches.

## Why this cannot be run where the design was written

The design's conformance suite proves each property by **removing the one
mechanism that enforces it** and requiring the control arm to fail
(`DEC-156`). Row 13's property is process-credential confinement — mapped uid
and gid, no supplementary groups, empty capability sets, `no_new_privs` set.

Measured in-jail on bubblewrap 0.11.2, the chosen removal changes nothing: both
arms return a byte-identical credential report. That much is already settled and
is **not** what this spike is for — the cause is that a non-setuid `bwrap` must
create a user namespace to perform its own mounts and does so whether or not
`--unshare-user` is passed, which the arms confirmed by both creating a *new*
userns with an identical `uid_map`.

What the jail could not test is anything about **what bubblewrap itself
establishes**, because the jail had already established it. `NoNewPrivs` was
already `1` and both capability sets already empty *before* any `bwrap` ran, so
no arm could distinguish *bubblewrap set this* from *it was already so*. That is
the gap this spike fills, plus the search for a delta that fires.

## Hard constraints

- **Payloads report; they never act.** `id`, `/proc/self/status`,
  `/proc/self/ns/*`, `/proc/self/uid_map`. No writes anywhere, no real paths
  bound writable, no network. This mirrors row 13's own stated hazard
  containment and is the reason the spike is safe to run on a real host.
- **Bind nothing sensitive.** The helper below binds only what a shell needs to
  start. Do not add the home directory, credential paths, or the repository.
- **Every negative needs a positive control.** A probe that could not have
  returned a different answer has said nothing. `EVD-013` had to be re-run
  because an earlier attempt used a binary absent from `PATH`, so every arm
  returned the same false negative. Arm `P` exists for this and is not optional.
- **Report what happened, including arms that error.** A `bwrap` invocation that
  refuses to start is a result, not a failed measurement — say so and quote the
  message.

## The script

Self-contained. Run it, capture stdout verbatim.

```bash
#!/usr/bin/env bash
# SL-248 credential-confinement spike. Read-only: every payload reports.
set -uo pipefail

REPORT='echo "  uid=$(id -u) gid=$(id -g) groups=$(id -G)";
        grep -E "^(Groups|CapInh|CapPrm|CapEff|CapBnd|NoNewPrivs):" /proc/self/status | sed "s/^/  /";
        echo "  userns=$(readlink /proc/self/ns/user)";
        echo "  uid_map=$(tr -s " " < /proc/self/uid_map | tr "\n" "|")"'

# Bind only what a shell needs, and only what exists on this host.
binds=(); for d in /nix /usr /bin /lib /lib64 /etc; do
  [ -e "$d" ] && binds+=(--ro-bind "$d" "$d")
done
COMMON=("${binds[@]}" --proc /proc --dev /dev)

arm() { # arm <label> <bwrap flags...>
  local label="$1"; shift
  echo "=== $label ==="
  if [ "$#" -eq 0 ]; then
    sh -c "$REPORT" 2>&1
  else
    bwrap "$@" "${COMMON[@]}" /bin/sh -c "$REPORT" 2>&1
  fi
  echo "  exit=$?"
  echo
}

echo "### host"
uname -srm; echo "bwrap: $(command -v bwrap)"
bwrap --version
stat -Lc 'mode=%a owner=%U setuid=%A path=%n' "$(command -v bwrap)"
echo "max_user_namespaces=$(cat /proc/sys/user/max_user_namespaces 2>&1)"
echo

arm "P  — positive control, no bwrap at all"
arm "A1 — probe arm posture (--unshare-all)"                 --unshare-all
arm "A2 — CredentialsConfined as designed (--unshare-user dropped)" \
        --unshare-pid --unshare-ipc --unshare-uts --unshare-cgroup --unshare-net
arm "A3 — candidate: explicit identity"                      --unshare-all --uid 4242 --gid 4242
arm "A4 — candidate: identity delta removed"                 --unshare-all
arm "A5 — candidate: --cap-add ALL under unshare-all"        --unshare-all --cap-add ALL

echo "### A6 — does bwrap SET no_new_privs, or inherit it?"
echo "parent NoNewPrivs: $(grep NoNewPrivs /proc/self/status)"
echo "(If the parent already reads 1, this host cannot answer it either —"
echo " say so rather than reporting bwrap's 1 as evidence.)"
```

## What the script already reports in-jail, so you are confirming not searching

The script was smoke-tested inside the jail before being handed over. Two arms
**already move**, which narrows this spike from a search to a confirmation.

- **`A5` fires.** `--cap-add ALL` under `--unshare-all` returned
  `CapInh`/`CapPrm`/`CapEff`/`CapBnd` all `000001ffffffffff` against `A1`'s
  all-zero, exiting clean. It did not refuse. These are capabilities *within the
  capsule's own user namespace* rather than host root, which is precisely the
  threat invariant 15 names — a capsule holding `CAP_SYS_ADMIN` in its namespace
  can mount what was never bound. So there **is** a delta that produces an
  observable credential difference, and it is an addition to the capsule's
  posture rather than the removal of a namespace.
- **`A3` vs `A4` fires.** `--uid 4242 --gid 4242` moved uid, gid, the `Groups`
  line and `uid_map`. Explicit identity is separately observable.

Neither was reachable by the delta the design chose. Your job on both is to
confirm the same behaviour on a real host and report anything that differs —
especially `CapBnd`, which is the column most likely to behave differently
outside, since this jail already reads `0` there and an ordinary unprivileged
user normally retains a **full** bounding set. If `P` shows a full `CapBnd`
outside and `A1` shows it emptied, that is bubblewrap observably stripping
something, and it is the one credential leg the jail genuinely could not see.

## Results

Fill one row per arm. `userns` records the inode, not just presence — two arms
that both created a namespace are only distinguishable by it.

| arm | uid | gid | Groups | CapPrm | CapEff | NoNewPrivs | userns | uid_map | exit |
|---|---|---|---|---|---|---|---|---|---|
| P | | | | | | | | | |
| A1 | | | | | | | | | |
| A2 | | | | | | | | | |
| A3 | | | | | | | | | |
| A4 | | | | | | | | | |
| A5 | | | | | | | | | |

## The six decisions

Answer each in one or two sentences, citing the arms.

1. **Is `bwrap` setuid on this host?** If it is, the in-jail conclusion does not
   transfer and everything below is re-opened — say so first and loudly.

2. **Does `--unshare-user` do anything here?** Compare `A1` and `A2` on every
   column, and specifically on `userns`. Identical userns *inodes* would mean
   something different from two distinct new ones — record which.

3. **Does bubblewrap set `no_new_privs`, or inherit it?** Only answerable if the
   parent (`P`) reads `0`. If `P` reads `1`, the honest answer is *this host
   cannot distinguish either*, and the design's claim that it is bubblewrap's
   default remains uncorroborated by measurement.

4. **Are supplementary groups clearable?** `A1`'s `Groups` against `P`'s. Real
   gids are expected rather than the overflow `65534` seen in-jail; the question
   is whether the list is ever *empty*, since row 13 holds only when it is.

5. **Can a capability set be observed being stripped?** Compare `P` and `A1` on
   `CapPrm`/`CapEff`. If the unprivileged parent already reads all-zero, then no
   arm on this host can show bubblewrap removing anything, and the finding is
   that the weakness is the test user's privilege level rather than the jail —
   which no change of host repairs. State it plainly if so.

6. **Do the two candidate deltas behave the same here as in-jail?** `A5` for
   capabilities, `A3` vs `A4` for identity. Both moved in-jail (above). Name
   every column that moved on this host, and any that moved here and did not
   there, or vice versa. If `A5` refuses on this host where it succeeded in the
   jail, quote the refusal — that would mean the candidate depends on already
   being inside a user namespace, which is a material constraint on any row
   built from it.

## What to hand back

The verbatim script output, the filled table, and the six answers. Nothing else
— in particular, no proposed rewording of row 13 and no design edit. The remedy
is `RV-346` round 6's to rule on, and it will rule better against measurements
than against a suggestion.

If an arm surprises you, prefer running it again over explaining it.
