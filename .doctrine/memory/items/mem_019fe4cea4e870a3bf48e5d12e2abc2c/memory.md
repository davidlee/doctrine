# `--die-with-parent` gates the wall bound

`doctrine-control`'s bubblewrap backend applies its wall bound from outside the
namespace: `timeout -k <grace> <secs> bwrap …`. That reads as a hard bound. It is
not — it is a bound *conditional on `--die-with-parent`*, which the `Teardown`
property supplies.

Measured (SL-248 PHASE-09 T9), payload `sh -c "echo LIVE; sleep 10"`, bound 3 s:

```
Under::Confining          -> TimedOut in  3.00 s
Under::Removing(Teardown) -> TimedOut in 10.02 s
```

Same bound, same verdict, three times the wall clock. `timeout` fires on schedule
and kills `bwrap`, but without `--die-with-parent` that no longer kills what
`bwrap` started: the payload survives, keeps the harness's captured stdout, and
`wait_with_output()` returns only when *it* finishes.

Two consequences worth remembering:

- **Budget a teardown-removed arm at its payload's natural runtime**, not at its
  wall bound. A conformance control that removes teardown pays full price.
- **`TimedOut` on such an arm is a weaker statement than it looks.** It says the
  wrapper fired, not that the arm was bounded.

The same mechanism is why a capsule's capture pipe outlives the capsule: anything
still holding an inherited descriptor — a survivor, or the pid namespace's init —
delays EOF, and the trusted side is blocked on EOF, not on the process.

Related jail gotcha: `setsid(1)` and `perl` are absent from the dev jail;
`python3` is present, so session-escaping payloads use `os.fork()` +
`os.setsid()`. A payload that shells out to `setsid` silently stops escaping and
still prints its liveness marker — a clean false negative.
