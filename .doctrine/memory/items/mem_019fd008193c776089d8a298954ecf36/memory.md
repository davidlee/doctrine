# Deleting a variant's only constructor leaves it invisible to `dead_code`

This repo denies `dead_code` and leans on that denial to find what a refactor
orphaned. There is a class it cannot see.

Delete the only site that **constructs** an enum variant. If any exhaustive
`match` still has an arm for it — a `Display` impl is the usual one — the
variant reads as *used*. The build stays green, clippy stays silent, and the
variant is unconstructible: reachable in no execution, still in the type.

Measured on `SL-244` PHASE-05 `T11`: `Refusal::DerivedConditionClaimed` lost its
only `Err(...)` site when the admission guard retired. Full `cargo clippy
--workspace` was clean. It surfaced only from a by-name sweep of the retiring
surface, alongside three doc comments citing it as precedent for a rule it no
longer enforced.

**Why the lint cannot help.** `dead_code` asks *is this symbol named anywhere*,
not *can a value of this shape exist*. A match arm names it. The two questions
come apart exactly when a type outlives its producer, which is what a deletion
sweep does by construction.

## What to do

Close a deletion sweep with a **by-name pass over the retiring surface**, not
with a green build:

    for n in RetiredType retired_fn RetiredVariant; do rg -c "\b$n\b" src/ tests/; done

Read every surviving hit — most are prose, and prose that cites a deleted
mechanism is its own defect (see the three docs above). Pair it with a positive
control on a name that *should* still be present, or a clean sweep proves
nothing ([[mem_019fa18161f47651af7687d8dccbbc67]]).

Same shape one level up: a struct field, a trait method with a default impl, or
a serde-only type can all stay "used" by a reader that no writer feeds. The
general question is *what still produces this*, and the compiler answers a
different one.

Related: [[mem_019f518e027770319100fb92d0f87ef2]] is the verification method
once you suspect a symbol is dead; this is why you must go looking in the first
place. [[mem_019e95ec587b71a2866e5fd97ffab45c]] is the denial this trusts.
