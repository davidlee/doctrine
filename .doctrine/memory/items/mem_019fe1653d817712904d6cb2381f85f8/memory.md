## What happened

SL-248 PHASE-05's mutation battery ran `M10` — *sort the derived `PATH` instead
of preserving host order* — against
`inner_path_draws_from_every_bound_path_in_host_path_order` and **redded
nothing**.

The rule was implemented correctly and the test asserted the right thing. The
fixture was the problem: host `PATH` was `/opt/toolchain/bin:/usr/bin`, whose
host order and lexical order **coincide**. `derived.sort()` was a no-op on that
input, so the assertion held under either rule.

## The pattern

A mutation that reds nothing rarely means "the rule is unimplemented". Far more
often it means the fixture cannot tell the two rules apart. Before concluding a
criterion is vacuous, check whether the *input* discriminates.

When ordering, containment, precedence, or dedup is the rule under test, choose
inputs where the plausible wrong rule yields a **different** answer:

- order: host order and sorted order must disagree (`/usr/bin` then
  `/opt/toolchain/bin`, not the reverse);
- containment: a directional rule needs the reversed pair too (a bound *file*
  `/bin/sh` must not admit the *directory* `/bin`);
- close-on-exec: a `std::fs::File` is opened `O_CLOEXEC` and passes against a
  sweep that does nothing — `dup(2)` the handle first, it does not set the flag.

## The follow-through

Fixing the fixture is not enough on its own: the next reader can regress it
without noticing. **Assert the discriminating property itself** —

```rust
let mut sorted = derived.clone();
sorted.sort();
assert_ne!(derived, sorted, "the fixture must discriminate");
```

— or, for the descriptor case, a precondition assertion that the dup'd handle
starts *without* `CLOEXEC`. The assertion is cheap and it is what stops the test
quietly going vacuous again.

Sibling: [[mem.pattern.tests.guard-needs-a-discriminating-difference]].
