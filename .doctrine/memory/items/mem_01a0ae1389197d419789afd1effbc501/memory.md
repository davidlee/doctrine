# Injected probes test the rule and skip the premise

The pure/impure split says: probe in a thin shell, inject plain values, keep the
decision pure and unit-testable. Good. But it creates one seam that **neither**
half of the resulting test suite covers:

    [ impure shell ] --measures--> value --injected--> [ pure rule ] --decides-->
         ^                                                  ^
         |                                                  |
    e2e tests reach here only on                    unit tests start here,
    the cheap/refusal paths                         from values YOU chose

The pure tests start *after* the measurement, so they cannot tell you the shell
measured the wrong quantity. And an e2e suite that only exercises refusal paths
never reaches the measurement either. **The rule is verified exhaustively and the
premise is never checked once.**

## The tell

Ask of any pure decision: *is what the shell measures the same kind of thing this
function compares?* If no test answers that in a real environment, the feature's
success path may never have executed anywhere.

Warning signs: every input to the pure tests is a literal you typed; the
integration tests all assert on errors; the feature "can't be tested without a
real X" and the design says a harness for X is unnecessary *because the logic is
pure and already tested*. That last sentence is the defect's charter — it is true
and irrelevant.

## The remedy is one test, not a harness

Not full environment coverage. **One** test through the real adapter, asserting
the premise rather than the rule. In SL-245 that was one `script -qec …` spawn
(util-linux, allocates a pty, no new dependency, no timing assertion) asserting
only that the topology check *resolved* — deliberately not what it resolved to,
since that depends on the environment.

Pair it with a positive control, since the assertion is usually a negative
("the refusal is NOT X"): a sibling test proving X is reachable at all.

## What it cost

SL-245's `-X` shipped gate-green with 17/17 verification criteria and did not
work on any terminal, because the shell compared two `st_rdev` values that can
never be equal ([[mem.fact.tty.dev-tty-fstat-is-the-devnode]]). Human acceptance
caught it in one command. A second blocker was hiding behind it.

Sibling failure mode, different seam: [[mem.pattern.review.invariant-test-must-drive-the-write-seam]]
— testing a pure helper in isolation when the invariant lives at the call site.

Corollary for verification planning: if a phase's acceptance rests on `VH`
criteria, a green `VT` run carries almost no information about whether the
feature works. Do not let phase completion outrun the `VH` rows —
`doctrine slice verify-vt` reports `VT` and is silent about `VH`.
