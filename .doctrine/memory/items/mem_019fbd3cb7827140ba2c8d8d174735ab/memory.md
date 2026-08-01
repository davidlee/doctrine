# A script's shebang interpreter is a mount dependency of the sandbox

Found in SL-241 PHASE-02 building the capsule confinement profile
(`scripts/spike-capsule/capsule/sandbox.sh`).

Under an **allowlist** bwrap floor — explicit `--ro-bind`s rather than
`--ro-bind / /` — a script bound into the sandbox is readable, executable by
mode bits, and **still cannot run**, because the kernel resolves the `#!` line
*before* PATH exists and `/usr/bin/env` is not in the namespace.

The failure is actively misleading:

```
bwrap: execvp /rig/provision.sh: No such file or directory
```

It names the **script**, which is present. The file that is missing is the
**interpreter**. Three runners were bound ro and all three failed this way
while assertions that they *resolved* and were *readable* were green.

## What to do

- Bind the interpreter: `--ro-bind /usr/bin/env /usr/bin/env` (or invoke as
  `bash /path/script.sh`, which sidesteps the shebang entirely).
- **Assert that a bound script EXECUTES, not that it resolves.** `test -r` and
  `test -x` both pass on an unrunnable script. The distinguishing observable is
  the exit status: exec failure surfaces as **127**, a script's own refusal as
  its own code.
- Map 127 to a distinct rig status so "the runner refused" and "the runner
  never ran" cannot read identically in a results table.

## Why it generalises

This is the confinement-specific member of a family: an assertion on a
*property adjacent to* the one that matters scores green while the mechanism is
wholly broken. Compare [[mem.pattern.shell.guard-exit-swallowed-by-command-substitution]]
(a guard's refusal cannot reach the entry point) and the rig's F-P01-3
("declares the script" is not "the script works").

The `--ro-bind / /` posture in `scripts/pi-spawn-confined.sh` never meets this,
because everything on the host is already visible there. It appears the moment
a profile switches to an allowlist — which is what any profile needing a path
to be **ABSENT rather than read-only** must do.
