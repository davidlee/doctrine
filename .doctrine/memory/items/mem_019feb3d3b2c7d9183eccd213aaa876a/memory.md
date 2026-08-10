## The pattern

A function that derives a value **from its environment** — a search path, a
mount set, a set of roots, a discovered toolchain — is not verified by a green
suite if that suite has only ever run in one environment. The environment's
accidents silently stand in for the logic, and the derivation reads as correct
because nothing has ever disagreed with it.

The tell is structural, and you can look for it without a second host: **does
this function read the world, and has the world it read ever varied?** If the
answer is "reads the world" and "no", the green result is evidence about the
environment, not about the function.

## What made it visible

`SL-248` shipped a conformance suite proving properties of confined capsules.
Nineteen rows, green in the development jail for the whole slice. `ISS-339` was
opened on the observation that it had never run anywhere else. Discharging it —
three runs on a real NixOS host, one day — found **three defects in three runs**,
each hidden behind the one before it:

1. **0/19, every row `NoLiveness`.** The bind set was derived from the *resolved*
   shell, but every payload execs the *literal* `/bin/sh`. On a store-based host
   `/bin/sh` is a symlink into `/nix/store` and `/bin` is not on `PATH`, so the
   capsule got the libraries and not the entry point.
2. **17/19.** Fixing (1) broke the invariant the *next* derivation rested on —
   row 4's permitted-`/` set was computed as "the first component of some `PATH`
   entry", so the newly-bound `/bin` came back as unexpected.
3. **18/19**, and the serious one: the fixture binds whole host top-level roots,
   including `/nix`. `--ro-bind` is read-only, **not `noexec`** — measured, such
   a capsule sees 691 store paths and runs `git`, `curl` and `gcc`. Carried as
   `ISS-341`; `SL-252` exists to fix it.

The jail supplied `/bin` twice over by coincidence — as a bind-mounted real file
*and* as a `PATH` entry — and either accident alone was enough to mask each
defect in turn.

## Why the green rows were not lies

This is the part worth internalising. None of the nineteen rows was false. They
were **evidence about the properties, in an environment that happened to satisfy
every unstated precondition**. No shipped row could convict the defect, because
each row's own permitted set was derived from the same environment reading that
was wrong: row 4 permits every root by the very fact that put it there. A check
whose expectation is derived from the thing it checks cannot fail.

So the failure mode is not "the tests were wrong". It is **the verdicts are worth
much less than they read, and nothing in the transcript says so.**

## What to do

- **Treat "has only run in one environment" as an open finding**, not a
  background condition. It cost nothing to write down as `ISS-339` and it paid
  out three times in one day.
- **Derive expectations from the specification, not from the environment.** Where
  a check's permitted set is computed the same way as the thing under test, the
  check is a tautology.
- **Expect each fix to expose the next precondition.** Three runs, three defects,
  each invisible until the previous was repaired. Budget for the sequence.
- **Watch for the cheap green.** `ISS-340` (the last red row) has a fix that
  turns the transcript green while leaving `/nix` bound whole — and the red row
  is currently the only thing pointing at the real defect. A green transcript
  with the serious half unfixed is worse than the red one.
- **The gain runs the same way.** Off-jail, `sec-9`'s `no_new_privs` observation
  came back caveat-free, because the trusted side does not already hold the bit
  there — a residual closed *by measurement* that no amount of in-jail running
  could close.

Related: [[mem.pattern.doctrine.conformance-measure-must-match-the-claim]] — the
sibling failure where the measure is weaker than the claim, rather than derived
from the subject. And [[mem.pattern.tests.caveats-must-be-measured-not-hard-coded]]
— the same root in the reporting layer: a caveat asserted from one environment's
shape rather than computed from the environment at hand.
