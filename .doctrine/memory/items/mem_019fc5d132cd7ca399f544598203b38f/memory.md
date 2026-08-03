# A smoke test must exercise the CAPABILITY, not just the dependency

Found in SL-241 PHASE-06 running the first real agent inside a capsule
(`scripts/spike-capsule`).

The A2 smoke (`control/probe-smoke.sh`) certified the bwrap confinement profile
for agent work using `claude -p 'print OK'`. Both legs green: network reachable,
credential survives the nested sandbox. The profile was then built on for three
phases.

**`print OK` needs no tools.** The first probe where an agent had to *do*
something found that every shell path was dead:

```
EROFS: read-only file system, mkdir '/agent/.claude/session-env/<session-id>'
```

The harness creates a per-session working directory under `$HOME` before any
Bash tool call runs, and the profile ro-bound the agent home. The agent could
write a file but could not run the tests, the linter, or `git commit`.

## Why it scored green anyway — the second trap

The agent reasoned the implementation out from reading the test file, was
**correct**, and the suite passed when the harness ran it afterwards. The exit
criterion said "the suite reaches green" and the suite reached green. The
criterion's *intent* — a real agent executes a real phase — did not happen.

An assertion on the OUTCOME cannot distinguish "the work was done" from "the
outcome arrived by another route". The probe only caught it because it recorded
**who completed the ritual** (`agent-committed=no tree-dirty=yes`) separately
from whether a commit existed. Fold those together and it reads as a clean pass.

## What to do

- **Smoke the verb the workload needs, not the noun it needs it from.** "Can I
  reach the API" and "can the agent edit, run, and commit" are different claims
  with different fixes. Reachability is necessary and nowhere near sufficient.
- **Ask what the real workload does that the smoke does not.** Here: allocate
  writable state under `$HOME`. A one-line prompt exercises none of it.
- **Record the ritual, not just the result.** When an outcome could arrive by a
  path other than the one under test, assert on the path too.

## Why it generalises

`probe-smoke.sh` had *already* split credential from network on exactly this
reasoning — "distinct failure modes and a single test conflates them". The
argument extends one step further, to *can it work*, and stopping one step early
is the whole failure. Whenever a smoke splits into N legs, ask whether leg N+1
is the one the workload actually needs.

The cost profile is the reason this matters: a smoke is run early and cheap
precisely so failures surface early and cheap. One that skips the load-bearing
capability inverts that — the failure surfaces at the most expensive possible
moment, after everything has been built on the certification.

Compare [[mem.pattern.tests.assert-the-refused-writes-signature]] (assert the
signature, not the product) and
[[mem.pattern.shell.shebang-interpreter-is-a-mount-dependency]] (assert the
script EXECUTES, not that it resolves) — the same family: an assertion on a
property *adjacent to* the one that matters scores green while the mechanism is
wholly broken.
