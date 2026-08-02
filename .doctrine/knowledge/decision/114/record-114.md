# DEC-114: One capture-with-timeout argv runner in a leaf; coverage folds its result

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

The mechanism currently inside `coverage_verify::run_argv` — run an argv with
`cwd == root`, capture both streams with each pipe drained on its own thread,
bounded by a wall-clock deadline that kills and reaps the child — moves into a
pure-of-policy leaf module, returning an outcome that names what happened.
`coverage_verify` calls that leaf and folds its result down to the local
`RunResult`. `doctrine spec anchors` calls the same leaf and reads the
unflattened outcome.

Two changes ride the extraction because the mechanism never needed either:

- The leaf takes a `Duration`, not `&VerificationConfig`. The timeout is the
  only thing the config was consulted for, and a leaf that runs a subprocess has
  no business depending on the verification config's shape (ADR-001).
- The adapter's timeout is its own config key with its own default, not
  `[verification]`'s. The declared adapter command is `cargo run -p <adapter>`,
  whose first invocation on a fresh checkout compiles a crate; inheriting
  `DEFAULT_TIMEOUT_SECS = 300`, a value chosen for a test suite, would time the
  first run out and report the corpus as uninventoried — the failure DEC-113
  closes, arriving through a different door.

## Why one runner rather than two

The two existing runners each have half of what this slice needs.
`coverage_verify::run_argv` has the mechanism and the wrong resolution: its
return type is `Ran { exit_ok, stdout, stderr } | Unobtainable`, and
`Unobtainable` deliberately collapses spawn failure, wall-clock timeout and
empty argv into one value, because the coverage fold only needs `Blocked`.
`verify::run_suite` has the richer outcome — `Completed{code}` / `NotFound` /
`SpawnFailed` / `EmptyArgv` — but inherits stdio and has no timeout, by design:
a dev gate streams and may legitimately run long. It cannot capture the
inventory JSON at all.

So the consumers differ in what they need to **know**, not in what they need to
**do**. That is the shape that wants one mechanism with two folds rather than
two mechanisms. The alternative — a second runner for the adapter — would put
the two-thread drain and the deadline poll-and-reap in the tree twice, which is
the code least tolerant of a subtly divergent copy: the single-threaded read of
one pipe deadlocks when the child fills the other, and that failure is invisible
until it happens.

DEC-113 is what makes the resolution question decisive rather than a
preference. The report must state what each declared adapter returned; a
straight lift of `run_argv`'s type would hand the provenance block one
undifferentiated "did not work" and undo that decision.

## What this obliges

`coverage_verify`'s existing suites are the proof of the extraction and must
stay green **unchanged** (the behaviour-preservation gate for shared
machinery). This is the whole defence against an extraction that quietly alters
timeout or reap behaviour, and it is why touching a module outside the slice's
objectives is acceptable here.

## Naming — a recommendation, not a settled point

The alternative raised was `run_argv_explicit_exit`, contrasting with the
existing `run_argv`. Two reasons to prefer something else.

The contrast misdescribes the difference. An explicit exit code is one part of
it; the property that matters is that the outcome is *unflattened* — it
distinguishes not-found from timed-out from ran-and-failed. `run_suite` also
returns an explicit code, so the name does not separate the new function from
the other runner either.

More to the point, the contrast is temporary. `run_argv` has exactly one caller,
the closure inside `run_argv_cached`. Once the mechanism moves out, what remains
is a short fold with one call site, and it can collapse into that closure
rather than keeping a name. Then there is no pair to disambiguate, and the leaf
takes the plain name.

So: name the leaf for its posture — it captures both streams and is bounded by a
deadline — and delete rather than rename the remainder. Module-qualified,
`subprocess::run_captured` or the equivalent reads without stutter at both call
sites. POL-001 rules out most of the metaphorical alternatives.

## Provenance

Settled at the `inq-6` fork of design run `dr-019fc13a` (SL-243). The single
call site of `run_argv`, and `cfg`'s single use for the timeout, were verified
against `src/coverage_verify.rs` while settling the fork.

## Related

- [[DEC-113]] — the provenance requirement that makes the unflattened outcome
  necessary rather than merely nicer.
