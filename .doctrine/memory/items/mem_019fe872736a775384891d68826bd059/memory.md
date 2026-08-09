# A mutation battery on a signalling instrument must drive synthetic targets

## The trap

When the code under mutation is something that **signals, kills, sweeps, deletes
or otherwise acts destructively**, the mutation you most need to convict is by
definition the one that *removes a containment check*. Run that mutant against a
real target and the instrument does the destructive thing to whatever is nearby
— which, during `cargo test`, is the test runner.

Measured in SL-248 PHASE-09: removing a process-group reaper's own-group refusal
made a test that had been handed the current process's **real** process group
enumerate that group and `SIGKILL` the `cargo test` process tree. Twice, on two
separate invocations.

## Why it is expensive out of proportion to the bug

The failure surfaces as a bare `Killed` with the runner's buffered output lost,
so it is indistinguishable from OOM, a harness timeout, or an infrastructure
flake. The first diagnosis was OOM; `free -m` showing 35 GB available is what
ruled it out. You lose the run, the log, and the diagnosis — and the source file
is left mutated, so the next thing you do is also wrong.

## The fix, which is a design change and not a retry

Make the instrument's notion of "self" and its notion of "target" both
**injectable**, and drive every signalling-path test with:

- a **synthetic own-identity** — a value the instrument will refuse to signal,
  distinct from the process's real one; and
- a **memberless target** — e.g. an id one past the highest live process, so
  even a fully un-contained mutant reaches nothing.

Then wire the *real* self-identity in exactly one place, and assert it with one
test that never signals anything.

After that change the same mutant reds two tests in 0.00 s instead of killing
the runner.

## The general rule

> A battery must red a **test**, not the runner. If a mutant can take out the
> harness, the fixture is wrong, not the mutant.

Corollary for the instrument itself: prefer **enumerate-and-signal** over
broadcast primitives. POSIX `killpg` reinterprets group 0 as the caller's own
group and -1 as every process on the machine, so a mutated argument to `killpg`
has a blast radius no test fixture can contain.


## Related

- [[mem.pattern.tests.mutation-needs-a-discriminating-fixture]] — the sibling failure: a
  mutant that convicts nothing usually means a non-discriminating fixture.
- A mutant convicted by the compiler is not convicted by a test; reformulate it
  so it compiles.
