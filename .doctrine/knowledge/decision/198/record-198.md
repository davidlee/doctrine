# Row identity splits a closed floor from an open profile

`SL-253` `inq-8` asked what `DEC-189`'s Firecracker membership derivation
concretely constrains, given `OQ-1` resolved narrow — derive the membership,
build no backend — and warned that an answer showing nothing makes the
derivation decoration.

`DEC-196` claimed this node was largely answered in advance by keying the kernel
on `RowId` with mechanism-owned rows. That was wrong. Working it reaches the
kernel's central type.

## The three-way squeeze

`Property` is a **closed** enum of fourteen members, one per table A row, and its
doc comment prizes the closedness: *"a row the suite can construct that this enum
cannot name is a compile error … the one machine-checked projection of table A,
and it is the whole of it."*

Three accepted decisions do not fit that:

- **`DEC-189`** — a microVM earns rows bubblewrap cannot, and rows 10, 12, 13 and
  14 (`ClosedDescriptorSet`, `OwnedStandardStreams`, `MappedCapsuleIdentity`,
  `ConfinedCapabilities`) lose their host-facing delta under a hypervisor.
- **`DEC-156`**, through `REQ-459` criterion 3 — a second backend is admitted by
  passing these assertions, **never by editing them**. Adding a variant to a
  kernel enum is editing them.
- **`DEC-195`** — the authority floor is a closed set reduced by exhaustive match
  so an empty floor is unrepresentable, and its membership is exactly row 3,
  `DeniedCanonicalStateAndCredentials`.

Closedness is load-bearing for the floor and forbidden for the profile. One type
cannot carry both, and today one type does. That is what the derivation
constrains, and it is checkable — which is what the node asked for.

## The split

`RowId` distinguishes three things rather than two (names indicative, settled at
implementation):

- **floor** — a closed enum, exhaustively matched, whose membership is
  `DEC-195`'s floor and currently one member
- **assurance** — an **open** mechanism-minted id: a `&'static str` newtype,
  exactly `BackendId`'s construction and for exactly `BackendId`'s stated reason,
  *"the contract must bind mechanisms nobody has written yet"*
- **axis** — closed, `REQ-450` criterion 1's five freshness axes

A hypervisor then publishes no rows 10/12/13/14 and mints its own, editing
nothing in the kernel. No new idea enters the crate: it is `BackendId`'s
construction, one level down.

## The residual, stated rather than waved through

Thirteen of today's fourteen `Property` members stop being kernel enum variants
and become payload-minted constants. The compile-time projection the doc comment
prizes therefore **survives only for the floor**; for the assurance profile the
vocabulary becomes a runtime one.

This is a real loss and is recorded as one. It is accepted because an open
vocabulary cannot be compile-checked by definition — that is what open means —
and because `BackendId` already took the identical trade for the identical
reason. What is not lost is the property `DEC-195` actually needs: the floor
stays exhaustively matched, so `EVD-021`'s vacuous-admission path remains
unrepresentable rather than merely untested.

## What this does not settle

`DEC-195` carried an open question here: row 8,
`TrustedTerminationObservation`, may be a fifth row losing its host-facing delta
under a hypervisor, since its control removes a file-size resource bound the
guest kernel would enforce. The split **de-escalates** rather than answers it.
Row 8 sits in the profile either way, and with an open profile key whether a
hypervisor publishes it is that mechanism's membership question, arising only
when such a backend is built. It joins `SL-253`'s existing follow-up for the
Firecracker row set.
