## The trap

Doctrine's `VA-ES` escape-sweep criteria pin a test census with

```bash
cargo test -- --list | wc -l
```

and assert it "has not shrunk below" a standing floor. The floor is a
**shrinkage alarm**: it exists to catch a `#[path]` include that stopped
compiling and silently dropped a whole file's tests (SL-233 PHASE-16 R3).

Capture that pipeline into a file with `2>&1` and the count inflates by roughly
**one line per test binary** — cargo writes a `Running target/debug/deps/…` line
to **stderr** for each, plus `Finished`. On a repo with 100 test binaries that is
about +101 lines.

Observed on SL-233 PHASE-16 close-out: the contaminated read was **6089** against
a prior-session figure of **5988** at the *same commit*. It looked like healthy
growth. Measured as the criterion actually pipes it (stdout only) it was **5988**
— byte-identical, no drift in either direction.

## Why it matters more than an off-by-N

The error is **one-directional and always upward**. It cannot make a census look
smaller, so it can never produce a false alarm — it can only *mask* a real one. A
genuine drop of 80 tests hidden under +101 lines of stderr reads as a pass. The
contamination defeats precisely the failure the criterion was written against.

## How to measure

```bash
N=$(cargo test -- --list 2>/dev/null | wc -l)   # stdout only
```

Then **sanity-check by decomposition** rather than trusting the total:

```bash
grep -c ': test$'                     # actual tests        e.g. 5788
grep -cE '^[0-9]+ tests?, [0-9]+ benchmarks?$'   # one summary line per binary   100
grep -c '^$'                          # one blank line per binary                100
# 5788 + 100 + 100 == 5988 == wc -l
```

If the decomposition does not reconcile to the total, the capture is polluted.
This also tells you the *binary count* for free, which is the other thing worth
watching — binaries vanishing is how test files disappear.

## The family

Same class as the zsh trap already in the corpus: **a pipeline's `$?` is the last
command's**, so `cargo fmt --check | head; echo $?` reports `head`'s status. Both
are cases of a measurement apparatus quietly substituting its own artifact for the
evidence. When a number or an exit code *is* the evidence, control the streams
explicitly — redirect stderr away, and read `$?` directly rather than through a
pipe.

Related: [[mem.pattern.testing.census-counter-auto-merge-wrong-total]] — the other
way a census total goes wrong without anyone editing a test.
