## The problem

Two implementations converge on the same final state, so no post-hoc assertion
separates them. SL-248 PHASE-06's capsule export is the clean case: adopting an
already-published export (the fast path) and *building* one, losing a no-replace
rename, then adopting the winner (the slow path) end with the same directory,
the same contents, the same sentinel file, and no leftover temporary — because
the loser cleans up after itself. Only the work paid for differs, and work is
not observable after it is done.

A sentinel file is the reflex and it does not work here. The mutation that
deleted the fast path redded **nothing**.

## The move

Withhold an input that only the path you are excluding needs, and assert
success.

```rust
let second = publish_or_adopt_export(
    &capsules,
    Path::new("/nonexistent/there-is-no-repository-here"),  // a rebuild cannot succeed
    &base,
    &identity("second"),
).expect("adopts without reaching for a source");
```

Adoption never touches the source repository; a build fetches from it. Naming a
source that does not exist makes the build the one thing that cannot happen, and
the assertion becomes exact. The same shape works for caches (point at an
unreachable origin), for memoisation (make the expensive function panic), and for
any short-circuit whose skipped work has a required input.

## Why it matters

This is the failure mode a mutation battery exists to find and the one a green
test hides best: the test passes, the name says the right thing, and the rule it
claims to protect is not protected. A test that only asserts the *end state* of a
convergent pair asserts nothing about which route was taken.

Related: [[mem.pattern.tests.mutation-needs-a-discriminating-fixture]].
