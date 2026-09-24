# ISS-478: Retired-key never-live pin conflates struct-only with the never-live invariant

Raised as `RV-375` `F-2` (code review of `SL-261` `PHASE-01`, 2026-09-24).

## Context

`SL-261` `PHASE-01` added the retired wire-key roster (`RetiredKey` /
`RETIRED_KEYS` beside `payload_contract::PAYLOAD`, `DEC-278`) and three pins in
`src/design_run/tests.rs`. `RETIRED_KEYS` is empty today; `PHASE-05` adds the
single row for `adopt_authored` on the `ApplyRequest` **struct**.

## The defect

`live_retirements` (`src/design_run/tests.rs`) answers
`TypeForm::Enum { .. } => true` — every enum owner is reported as a violation —
and documents that as "whose owner has no key rows at all, so the retirement
names a surface that cannot have held it". The premise is false: an enum's key
surface lives on its variants. `contract_check::walk_payload` passes
`VariantPayload::Keys` to `walk_keys` with **the enum contract** as `owner`, so
the walk does support refusing `RetiredPayloadKey` for an enum-variant key.

The checked domain (struct owners only) is therefore narrower than the supported
domain, and the pin's name overstates what it proves: it substitutes a
struct-only restriction for the "a retired key is never still live" invariant.
The first legitimate enum-variant retirement trips the pin with a misleading
reason (and the injected fixture repeats the false claim verbatim).

`SL-261` `PHASE-01`'s phase sheet records the choice as a Decisions entry, so
this is a design question rather than a slip.

## Options when it matters

1. Make the pin walk variant key surfaces recursively — check the invariant it
   names; or
2. split the struct-only restriction into its own named pin whose failure
   message says it checks struct key rows only, so a future retirement is not
   misdiagnosed.

Not load-bearing for `SL-261`; no current or planned roster row uses an enum
owner.
