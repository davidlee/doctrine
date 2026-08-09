## The problem

Some properties are only observable by *varying* something the whole process
shares — the current working directory, an environment variable, a umask. A
multi-threaded test binary (`cargo test`'s default) cannot mutate those safely:
`std::env::set_current_dir` and `std::env::set_var` are process-wide, and the
races they cause are whole-suite-only reds that cost a full cycle to attribute.

The tempting repair is a mutex. It does not work, because the *readers* are
usually not yours: in `SL-248` the cwd is read by `operator_regions`, which
decides which host roots a sandbox fixture may bind — so mutating the cwd would
change what every other concurrently running test's sandbox can see.

## The pattern

Vary it **one process up**. `Command::current_dir` / `Command::env` are
per-spawn and therefore thread-safe; a child process has its own copy of every
process-wide thing.

When the code under test lives in the test binary (a bin-only crate, or a
`#[cfg(test)]` harness), the child *is* the test binary:

```rust
const HELPER: &str = "module::tests::the_helper";   // ← see the gotcha below

#[test]
#[ignore = "instrument: re-executed with a chosen cwd by the_real_test"]
fn the_helper() {
    // do the work in whatever cwd/env we were started in
    println!("RESULT={value}");
}

fn measured_from(cwd: &str) -> String {
    let exe = std::env::current_exe().expect("the test binary's own path");
    let output = Command::new(exe)
        .current_dir(cwd)
        .args(["--exact", HELPER, "--ignored", "--nocapture"])
        .output()
        .expect("the test binary re-executes");
    /* parse prefixed lines out of output.stdout */
}
```

`#[ignore]` keeps the instrument out of ordinary runs; `--ignored` plus
`--exact` selects exactly it; `--nocapture` lets its `println!` reach the pipe.
Cross the boundary with **prefixed lines**, not a serialisation format — the
child's stdout also carries libtest's own chatter.

## The gotcha: `--exact` fails open

`--exact some::name::that::no::longer::exists` runs **zero** tests and exits
**0**. A rename therefore produces a child that prints nothing and succeeds. So:

- put the `--exact` name in a named constant, next to the helper; and
- make the parent **panic when the expected line is absent**, quoting the
  child's stdout and stderr.

Then a drift is a loud parse failure rather than a quietly green measurement of
nothing.

## When not to reach for this

If the process-wide state is mutated by *production* code you own, the answer is
a named guard that production takes and tests can take too — see
[[mem.pattern.tests.process-wide-state-needs-the-production-guard]]. This
pattern is for state with no owner, where every reader in the process is a
victim. It costs a process (~1.5 s here, including a sandbox fixture), so it is
for the handful of tests that genuinely need it.


## Correction — a child closes an aggressor, it only narrows a victim

As first written this pattern did not distinguish the two roles, and applied
to a **victim** it licenses the trade `SL-248` `F-19` explicitly refused —
converting a flake into a smaller flake.

Measured (`PHASE-10` `T4` residual): `exec 9< /etc/hostname && sh -c 'sh -c
"ls -1 /proc/self/fd"'` prints `0 1 2 3 9`. **An inheritable descriptor
survives two exec levels.** So a re-executed test-binary child inherits
whatever the parent had inheritable *at spawn* and hands it on to its own
shell.

The asymmetry that follows:

- **Aggressor** — a test that *opens* shared process state others must not
  see. A child is a genuine **closure**: nothing it opens can reach another
  test's fork, because it has its own table.
- **Victim** — a test whose claim is that the state is *absent*. A child is
  only a **narrowing**: it shrinks the exposure to the spawn instant instead
  of removing the agent. The victim can still inherit whatever was open then.

For a victim, close the class at the source instead: make the guarded
resource unobtainable without its guard. `decoys_under_the_window` returns
the `MutexGuard` *with* the descriptor set, so a caller cannot bind the
descriptors without the lifetime that protects them, and the victim holding
that same guard across its open, its sweep and its fork excludes every other
thread by construction rather than by timing.

Sites that genuinely cannot take the guard — re-entrancy, or a fork that
takes the guard itself and would deadlock — are the ones that belong in a
child, and each should say so in the source.

See [[mem.pattern.testing.timing-tests-need-a-load-tally]]: repetition never
reproduced this: 27 unaggravated runs were green. A two-arm causality
experiment reproduced it on the first run.
