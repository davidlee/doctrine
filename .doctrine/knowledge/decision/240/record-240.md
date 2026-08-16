## Decision

The durable statement this slice owes is a **new requirement under `SPEC-029`** — minted as `REQ-478` (`FR-009`, *Report every recorded mutation on the change log*). `STD-003` is **not** widened, and the alternative of recording nothing durable is rejected.

## Options considered

- **A — new `REQ` under `SPEC-029`.** Additive authoring beside `REQ-433` (*derive every read projection from one canonical turn envelope*); no revision required against spec or ADR.
- **B — widen `STD-003`** (*no silent skip — a degraded read is disclosed*) by dropping or broadening its scoping clause.
- **C — neither.** Let `DEC-238` plus the slice's tests carry it, with a memory recorded at close.

## Why A

The obligation is container-local. The change log is `SPEC-029`'s own vocabulary and its own derived projection, and the thing that must stay true — *a mutation the run records is reported by the change log* — is a statement about this container's read model. `DEC-239`'s readable/emittable roster split is what makes it precisely statable: the obligation binds the emittable half only, so retired vocabulary kept alive for snapshot parsing is not held to an emission it must never perform.

It also lands where it will be met. A spec requirement is checked by coverage against `SPEC-029`; a standard is enforced by review remembering. The next person editing `ChangeEvent` meets the former without having to know the latter exists.

The acceptance criteria were already drafted — the slice's Verification & Closure Intent lines are them, and `every_material_event_kind_persists_a_change_row` is the standing enforcement that makes an unwired member fail loudly rather than be quietly absent.

## Why not B

`STD-003`'s exclusion of write paths is deliberate, not a gap. The standard says it twice: *"tolerate-and-disclose is a rule for readers"*, and *"a degraded read on a **write** path refuses"*. `SL-256`'s risk `A1` observed that the standard's second prohibition — *no empty success* — misses `ISS-355` only by its scoping clause; that is a coincidence of wording, not of subject. `STD-003` governs a reader that could not see. This defect is a writer that did not say.

The cost settles it independently. `STD-003` is `required` and cross-cutting, so broadening its scope would retroactively bind every write path in the repository to a disclosure obligation nobody has audited — a large, unbudgeted commitment bought to settle one container's defect.

## Why not C

`ISS-367` is already the second instance, so the class is real rather than anticipated. `DEC-238`'s general form — a derived row can only report changes in the *key* of the set it differences — is worth more than a slice-local test can hold.

## Consequences

- `REQ-478` is authored `pending` under `SPEC-029` as `FR-009`, covered by `SL-256` and verified at its close.
- `SL-256`'s risk `A1` is discharged: it asked design to choose between a new `REQ` and a `STD-003` widening, and this is the choice.
- The requirement carries an explicit bound: the row reports that a mutation was recorded, and is **not** a claim about the payload keys it was built from. That keeps it clear of the silent-absorption class (`ISS-333` / `ISS-346` / `ISS-327` / `ISS-328`, gated on `QUE-219`), which the slice's Non-Goals put out of bounds.
- Should the class later prove genuinely cross-cutting — other derived-row surfaces, the review ledger, dispatch receipts — the route is a **new** standard with `REQ-478` as its first adoption, not a retrofit of `STD-003`.