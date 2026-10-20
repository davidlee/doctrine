# ISS-527: SL-119's pi APPEND_SYSTEM delivery has no production caller

SL-119 (*Wire boot snapshot to pi via APPEND_SYSTEM.md*) is `done`, and IMP-116
is `closed`, but `boot::install_append_system` (`src/boot.rs`) has **no
production caller** — only its own definition and three unit tests. Both
`install_refresh` arms set `append_system: AppendSystemOutcome::NotApplicable`,
so `.pi/APPEND_SYSTEM.md` is never created by `boot install` or `doctrine
install`.

Consequence: `AppendSystemOutcome`'s `CreatedDirAndLink` / `CreatedLink` /
`ReplacedLink` reporting arms in `apply_wire` are unreachable, and the
`refresh_touched` entries for that path can never fire. A done slice's claimed
behaviour is dead code.

SL-284 noticed this (design sec-4 scopes it out explicitly: "has no production
caller; it is noted as a follow-up, not revived here") and carried it as a notes
bullet. This item is that follow-up, captured so it outlives SL-284's close.

Decide: revive it on the pi arm (the shape SL-119 intended), or retire the leg
and the enum with a decision record — either is honest; leaving a `done` slice
whose delivery path is unreachable is not.
