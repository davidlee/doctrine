# IMP-421: Move HostDescriptor derivation onto HostFacts

`host_descriptor()` reaches disk directly — `std::env::consts::OS`/`ARCH` plus a
`/proc/sys/kernel/osrelease` read with a documented `"unknown"` fallback. That is
an engine unit opening a second door onto facts `host.rs` exists to own.

## Why it shipped this way

`SL-248` `PHASE-07` `EX-15` requires the admission verdict to carry the host,
`verify`'s three parameters are fixed by `EX-2`, and `HostFacts` carries no OS,
kernel or architecture. With no seam available and the criteria closed, the
derivation went in-place (`D7`).

It was written defensively: the **whole** derivation sits behind one named
function, deliberately, so the swap is a one-function replacement rather than a
hunt.

## The two candidate homes

1. **`descriptor()` on `HostFacts`** — *preferred*. `host.rs` exists to be the
   host-facts seam, and this is a host fact. Edits `host.rs`.
2. **`rustix::system::uname()`** — needs the `system` feature on
   `crates/doctrine-control/Cargo.toml`. Removes the hand-rolled `/proc` read but
   leaves the derivation outside the module that owns host facts.

`SL-248`'s notes lean toward (1), and so does this item: the argument is
**coupling, not correctness**. Nothing is wrong with what ships — it is in the
wrong place, and the wrong place is the kind that grows a second one.

Both are `S1`-scale edits.

Originates from `SL-248` `RV-352` reconcile, `notes.md` item 50
(phase sheet `PHASE-07` `F-3` / `F-3/N`).
