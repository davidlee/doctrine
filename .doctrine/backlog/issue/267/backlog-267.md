# ISS-267: Worker-marker blind test strips only the env leg

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`tests/e2e_backlog_filter_alias.rs` fails in any `--worker` fork. Found while
running the full suite during RV-317 turn-2 remediation; confirmed **pre-existing**
by reproducing it unchanged at the base commit (`23c4309ed`), so it is not an
SL-231 regression.

## Symptom

    test positional_substr_is_a_deprecated_alias_of_filter_which_wins ... FAILED
    panicked at tests/e2e_backlog_filter_alias.rs:36:5

Line 36 is the `assert!(out.status.success())` inside the `new_issue` helper. The
underlying refusal, reproduced directly:

    $ env -u DOCTRINE_WORKER ./target/debug/doctrine backlog new issue "probe" -p .
    Error: worker fork (signal: marker): refusing authored write `backlog new`
    — workers return a source delta; doctrine-mediated writes funnel through the
    orchestrator.

## Cause

The helper's doc comment states the intent exactly, and does half of it:

    /// `DOCTRINE_WORKER` is explicitly UNSET — `backlog new` is an authored write
    /// the worker-mode guard refuses under a leaked env leg; strip it so the write
    /// proceeds regardless of the caller's shell

It calls `.env_remove("DOCTRINE_WORKER")`, which neutralises the **env** leg. The
guard has two legs, and the **marker** leg (`.doctrine/state/dispatch/worker`,
stamped by `worktree fork --worker`) still fires. So the test passes in the
primary tree and fails in every worker fork.

## This is ISS-260's class, recurring

ISS-260 was exactly this: the ADR golden's worker-marker skip read only the env
leg. That was fixed on edge (`9cd0e7706`). The fix was applied to the instance,
not swept for siblings — the same pattern RV-317 named as this slice's standing
risk (`mem.pattern.review.sweep-defect-class-not-instance`).

So the work here is **not** just this one test. Sweep every test that spawns the
binary for an authored write and neutralises only the env leg; a grep for
`env_remove("DOCTRINE_WORKER")` is the starting point, and each hit needs the
marker leg handled too.

## Shape

Whatever ISS-260 used for the ADR golden is the existing seam — ride it rather
than inventing a second mechanism. Options, in preference order:

1. A shared test helper that neutralises **both** legs, used by every such test
   (the marker is a file in the temp repo under test, so the helper can simply
   not stamp it — these tests build their own repo in a tempdir and do not need
   the fork's marker at all).
2. Failing that, point the write at the test's own tempdir root explicitly so the
   fork's marker is out of the resolved root's path.

## Why it matters

It is invisible in normal development (the primary tree passes) and appears only
inside dispatch forks — which is precisely where workers run `cargo test` and
where a red suite is most expensive to diagnose. It also erodes the signal: a
worker seeing one pre-existing failure learns to discount failures.
