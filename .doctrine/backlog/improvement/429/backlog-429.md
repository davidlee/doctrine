# IMP-429: Give the confinement prefix a home, and macOS parity with it

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced during `SL-254`'s design inquiry (2026-08-13), against the
`sufficiency-accepted` judgement, and deliberately kept out of that slice.

## The two halves

**Placement is the real question.** The bwrap confinement prefix currently lives
at the script tier — `scripts/pi-spawn-confined.sh:113-131`, an eight-token
`PREFIX` array — with a macOS `sandbox-exec` sibling beside it. `SL-254`
generalises it past `pi` and stops there, on `DEC-209`'s grounds: no
`doctrine-control` dependency, no hardening ported. That leaves unanswered where
it should *live*:

- a script, as today;
- part of the library / install surface, seeded like the other shipped assets;
- a Rust wrapper in the binary;
- or something else.

The tension is `POL-002` on one side (harness specifics stay out of the engine
core) and single-sourcing on the other (`STD-001` — the same confinement facts
are currently expressed twice, once per host OS, in shell). `jail.rs` already
owns `bwrap_core_argv` / `bwrap_argv` / `validate_policy` / `select_jailer`, and
`crates/doctrine-control` owns a much fuller `confinement_argv`
(`backend/bubblewrap.rs:1110`) that `SL-254` declined to depend on — so the
placement question is really *which of three existing homes wins*, not where to
build a fourth.

**macOS parity is wanted, and is awkward to verify.** `SL-254` objective 1
asserts the `sandbox-exec` sibling is kept at parity, and `DEC-206` re-homes
`write_seatbelt_profile` among the four jail primitives it moves to `jail.rs`
before any deletion — so the seatbelt path is inside the surface. But no
decision covers whether the collapsed claude arm actually *reaches* it, and
testing it needs a different host, which is painful to arrange mid-slice.

## Why it is not `SL-254`'s

`SL-254`'s posture is to avoid building new things: it collapses two arms onto
the incumbent one and deletes the apparatus that existed only for the arm being
removed. Choosing a new home for the prefix is construction, and the macOS leg
cannot be verified on the slice's own host. Both are better done deliberately
than folded into a collapse.

## Related

- `SL-254` — the collapse that surfaces this; `DEC-209` declines the
  `doctrine-control` dependency and defers hardening to `IMP-428`.
- `IMP-428` — harden the worker confinement prefix. Adjacent but distinct: that
  one is about *what the prefix contains*, this one about *where it lives* and
  whether the macOS sibling holds.
- `POL-002`, `STD-001` — the two rules in tension over the answer.
