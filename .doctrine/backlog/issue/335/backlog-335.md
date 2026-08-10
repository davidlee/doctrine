# ISS-335: Spike scripts inherit an unpinned PATH

Two shipped spike scripts carry `F-41`'s silent-false-negative defect, and one of
them is the guard's own sibling.

## The defect

`spike-deltas.sh` does not pin its sandbox `PATH` — it inherits the ambient one,
which *in this jail* is already store-rooted with `/nix` bound, so it works here
and only here.

Run off-jail verbatim, **every arm returned `exit=0` with `command not found`**
for `sleep`, `cat`, `cut`, `ls`, `grep` and `dd`. The readings were `wrote 0
bytes`, `0s` and empty proc counts: a full-table false negative **that looks like
output**.

`spike-teardown-2x2.sh` has the same shape via `PYTHON=$(command -v python3)`,
which resolves on the *host* while the sandbox binds omit `/home`. Its "`setsid`
binary on PATH" line likewise reports the **runner's** `PATH` rather than a host
capability — so as the evidence behind `F-27`'s "absent in this jail" claim, it
is weaker than it reads.

## What makes it galling

`spike-credentials.sh` already guards exactly this, at its own lines 51–61, and
names the trap. The sibling did not inherit the guard. `spike-mounts.sh` pins its
`PATH` into a bound root and does not share the defect — it is the pattern to
copy.

## Fix

Pin the sandbox `PATH` into a bound root in `spike-deltas.sh` and
`spike-teardown-2x2.sh`, and make an unresolvable payload binary **convict**
rather than return zero.

## The generalisation

`F-41`'s, and the slice's own audit walked into it: **a probe that cannot run is
indistinguishable from a probe that found nothing, unless it is built to
convict.**

## References

`SL-248` `notes.md` § *Owed* item 170 · phase-sheet finding `F-41` · `RV-352` ·
see also `ISS-336`, the environment-provenance half of the same class
