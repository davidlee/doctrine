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

## Resolution — obsolete (SL-245 reconcile, 2026-09-17)

Dissolved by `RV-369` `F-1`, not fixed. The audit found that the `st_rdev`
comparison this issue is about is not merely mistyped on macOS — it is
**unsatisfiable on every platform**. An fd opened from `/dev/tty` `fstat`s as the
`/dev/tty` devnode itself, `(5,0)`, and never as the pts it redirects to,
`(136,N)`, so `Endpoint::Same` was unreachable and `-X` refused every terminal on
earth with a message explaining it was the wrong one.

The fix replaced device ids with POSIX session ids via `tcgetsid`, which is a
total identity: a controlling terminal belongs to exactly one session and a
session has at most one controlling terminal. There is no `dev_t` left in the
path, so there is nothing left to widen and this issue has no subject.

**This is not macOS clearance.** `F-1`'s fix removes the *compile* barrier named
here; it verifies nothing about `VMIN`/`VTIME` timed reads on macOS `/dev/tty`,
which is the actual platform question. That remains open as `CHR-072` — do not
read this closure as covering it.
