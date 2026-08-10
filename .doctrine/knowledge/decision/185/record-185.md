## The bar

**The fixture's readable set must be no wider than the property under test requires.** The capsule contains its payload toolset and nothing else.

Not *match production*, and not merely *be declared*. Just: do not bind what the test does not use.

## Why not parity (S1)

`SL-248` `PHASE-05` `EX-8` — no package store ever bound whole, no host-shaped default, no fallback — was written for a capsule that holds a worker with authority, where a wide readable set is an escape surface. It does not govern the fixture, and research thread 1's negative finding is that nothing else does either: the fixture's capsule construction is ungoverned at this granularity.

The fixture's capsules hold the suite's own payload strings. No worker, no credentials, no canonical state, nothing hostile. Importing a rule written against a threat model that is absent here would buy strictness by argument rather than by need, and would price the slice as though a macOS-grade port were required.

## Why not declared-but-wide (S3)

What a wide set actually costs the fixture is not safety but **the evidential worth of the verdict**, which is `ISS-341`'s own formulation: *the verdicts are not false — they are worth much less than they read.* Rows 2, 3 and 4 demonstrate bounded input sets and bounded filesystem visibility *inside* the capsule. A capsule holding 691 store paths, with `git`, `curl` and `gcc` all executable, still satisfies those rows and still says much less than a reader takes it to say.

Making that legible (objective 3) is necessary and is not sufficient. A transcript that accurately reports a capsule holding the host toolchain has told the truth about a weak measurement; the measurement is still weak.

## What the bar buys

1. It is a claim the transcript can actually assert, and one an auditor can check against the payload toolset.
2. It gives `inq-5` a clean answer without a separate principle: a host on which the toolset's closure cannot be derived cannot build a representative capsule, and should say so rather than widen.
3. It lands on roughly shape (C) in practice — the toolset's directories plus the toolset's closures — without needing `EX-8`'s applicability to something `EX-8` does not govern.

## The cost this bar knowingly accepts

On a store host there is no cheap route from a binary to its libc: Nix store paths bear no lexical relation to their dependencies, so `bash` and its `glibc` are unrelated directories. Reaching the loader without binding the store whole requires an ELF read, a store query, or nothing — there is no fourth option. So *narrow* and *no new machinery* genuinely trade against each other under this bar rather than only appearing to.

The trade may still be cheap. `ldd` is not Nix internals; it is present on every glibc host, it answers exactly this question, and the production `closure-resolver` seam already exists to receive its output. Which route is taken is `inq-4`, not this node.

## Standing note

This bar bounds how much machinery `inq-4` may buy. A resolver proposal that costs more than the evidential worth it restores fails this decision, not merely the owner's taste.


## The bar is negotiable, and that is recorded rather than assumed

The owner has stated they would relent on this bar and take a cheaper option if
the implementation cost of meeting it looks intractable.

So this decision is a *default*, not a floor. If the shape that satisfies
representativeness turns out to cost more than the evidential worth it restores,
the correct move is to reopen this record and fall back — most likely to `S3`
(declared but wide, with the posture legible in the transcript), which is the
option this decision refused on worth rather than on principle.

What must not happen is falling back *silently*, or discovering the cost at
implementation and quietly widening the set without amending this record. The
fallback is a decision, and it gets made here.
