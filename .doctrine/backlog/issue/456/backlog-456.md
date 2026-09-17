# ISS-456: endpoint st_rdev typing is Linux-only

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

Surfaced during SL-245 PHASE-03 (commit `6c6c11a51`), flagged by the worker
rather than papered over.

SL-245's design pins the pure endpoint decision as
`endpoint(stdout_is_tty, stdout_device: u64, tty_device: Option<u64>)`, and
`src/tty.rs` implements it that way. `rustix::fs::Stat::st_rdev` is `u64` on
Linux, so the field flows straight in and compiles. On macOS `dev_t` is `i32`,
so it will not.

The check exists to decide whether stdout and `/dev/tty` are the same device.
Getting its typing wrong is not cosmetic — it changes the answer.

## Why it was not fixed in PHASE-03

Every in-scope workaround was worse than the bug:

- `as u64` trips `unnecessary_cast` on Linux, where the value is already `u64`.
- `u64::try_from(...).unwrap_or(0)` maps a negative `dev_t` to `0`, which would
  make two *distinct* terminals compare equal — a false `Same`, precisely the
  failure the endpoint check exists to prevent.

The honest fix is a platform-conditional conversion (or a `dev_t` newtype that
owns the widening) and it sits outside PHASE-03's declared file set.

## Why it matters

macOS is a platform the SL-245 design explicitly reasons about — its
`poll`-on-macOS rationale is part of why the query path avoids readiness
syscalls. A build that cannot compile there is a real gap, not a theoretical
one.

## Related

SL-245 `DEC-259` (the single verified terminal endpoint). Pick this up in
SL-245 PHASE-05 if it is cheap there, or standalone.
