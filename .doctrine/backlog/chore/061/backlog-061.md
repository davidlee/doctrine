# CHR-061: Move CAPSULE_UID and CAPSULE_GID to backend.rs

`CAPSULE_UID` and `CAPSULE_GID` live in `backend/bubblewrap.rs`. They became
`pub(crate)` there so row 13's payload asserts the profile's **own declaration**
rather than a literal that resembles it (`STD-001`).

The declared capsule identity is a property of a *placement*, not of one
backend's argv assembly. `backend.rs` is where the other placement constants
live, and is the more plausible durable home — a second backend declaring the
same identity should not import it from bubblewrap's module.

Out of `SL-248` `PHASE-10` `T7`'s carded scope and deliberately not taken there.
Pure relocation: no behaviour change, no visibility change beyond the move.

Sequencing note: `IMP-419` proposes changing the *value* of `CAPSULE_UID`. These
are independent — do them in either order — but doing this one first makes the
other a one-line edit in the right file.

Originates from `SL-248` `RV-352` reconcile, `notes.md` item 132
(phase sheet `PHASE-10` `F-44`, `T7`).
