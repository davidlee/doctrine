# ISS-314: derived_status returns Done on an empty ledger, against ADR-007 D-C8

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`ADR-007` **D-C8** (Derived status function) fixes the empty-ledger case
explicitly, and states its own reason:

```text
empty ledger                  → status = active, await = raiser   (raiser goes first)
any open/answered/contested   → status = active, await = derived role
all verified/withdrawn        → status = done,   await = none
```

> The empty ledger is `active` (not vacuously `done`), so an implementation can
> never mistake "no findings yet" for completion.

`derived_status` (`src/review.rs:1009-1012`) returns `(Done, None)`:

```rust
pub(crate) fn derived_status(findings: &[FindingState]) -> (ReviewStatus, Await) {
    if findings.is_empty() {
        return (ReviewStatus::Done, Await::None);
    }
```

That is the reading the ADR exists to forbid, and the doc comment above the
function states it as intended behaviour rather than flagging it.

## It is codified in tests

`ADR-007` § Verification requires the derived-status function to be covered
"including the empty-ledger and all-terminal cases". Two tests cover it and
assert the violating value:

- `src/review.rs:3199` `show_renders_empty_ledger_done_and_the_edge`
- `src/review.rs:3225` `list_renders_empty_ledger_done_and_the_edge`

So the conformance test exists and pins the wrong answer. Repairing the function
without repairing these is a red suite; repairing them without reading `ADR-007`
looks like a green-chase.

## How it surfaced

`RV-344` **`F-8`**, the `SL-244` design-coherence pass.
`SL-244`'s design (`sec-3`, "The condition model") builds an admissibility rule
on the observation that an empty finding list "reads as `(Done, None)`". The
observation is **accurate about the code** and contradicts the ADR; the design
did not notice it was describing a violation rather than a specification.

`DEC-138` carries the same sentence in its consequences.

Two consumers to check when fixing:

- `RV` close-gate `D-C9b` — an unresolved `blocker` refuses the target's closure
  transitions. A freshly-minted, finding-free `RV` currently reports `done`; under
  D-C8 it would report `active`/`raiser`. Whether anything gates on
  *review-is-done* rather than on *blockers-outstanding* decides whether this fix
  has teeth beyond display.
- `SL-244` itself specifies `review-disposition-attested`'s `Conducted` arm
  against this behaviour, and its stated premise needs rewording once this
  lands. **But do not read that as this issue owning a fix over there** — see
  the next section. The two are independent.

## This fix does NOT fix what SL-244 cites it for

Worth stating plainly, because the citation makes it look like it does, and a
fixer who assumes so will either scope-creep into `SL-244` or expect a design
correction that is not theirs to make.

`SL-244`'s design offers the `(Done, None)` reading as its evidence that the
`Conducted` arm *"would be satisfied the moment the stage was entered"*. That is
the wrong mechanism for the hazard it names. `DEC-138`'s predicate does not read
`status` at all:

> `Conducted { review }` satisfies the row while the minted `RV` carries **no
> finding** that is both `blocker`-severity **and** in `open` or `contested`
> state

That is universally quantified over the finding set, so it is **vacuously true on
an empty ledger** — independently of what `derived_status` returns. `DEC-138`
rejects binding to the review-level `await` explicitly, on `ADR-007` D7's ground
that it is a display summary and never a gate; the design cites D7 correctly two
paragraphs earlier and then argues from `status` anyway.

So the two defects are disjoint:

| | what is wrong | what fixes it |
|---|---|---|
| this issue | `derived_status` contradicts `ADR-007` D-C8 | repair the function and its two tests |
| `SL-244` `sec-3` | argues vacuous satisfaction from `status` rather than from the quantifier | reword the premise; the conclusion (a concluded-pass marker is required) stands unchanged |

Fixing `derived_status` leaves `SL-244`'s hazard exactly where it was. `DEC-138`
carries the same red herring in its consequences and wants the same correction
when the design is reconciled.

## Not urgent

Nothing is known to be broken by it today: `await` is a display summary and never
an exclusive gate (`ADR-007` D7 — the turn gate is per-finding `can`). The cost is
that the invariant an accepted ADR states, and tests, is false — and a design has
already load-borne on the false version.

## This is a revert, and it needs IMP-392 first (added 2026-08-04)

The two consumers above are now traced, and a third — unnamed here, and the one
that matters — was found.

**The close-gate question resolves to "no teeth".** `D-C9b` routes through
`unresolved_blockers_for` (`slice.rs:1073`), which filters the finding list; an
empty ledger yields no blockers whichever way `derived_status` answers. Nothing
gates on *review-is-done*.

**The teeth are in priority classification.** `derived_status_string` feeds
`catalog/scan.rs:420` → `status_class`, where `active` is `Workable` and `done`
is `Terminal`. So the fix moves every finding-free `RV` out of terminal and into
the actionable surfaces (`next`, survey, blockers) — permanently, per the next
paragraph.

**The fix as written reverts a deliberate change.** `SL-040` `PHASE-01` shipped
`(Active, Raiser)`, with a comment naming the ADR's own rationale ("empty ≠ done
— the `SL-009`-divergence-proof case"). Commit `20d43229` (2026-06-19) reverted
it to `(Done, None)` to close **`IMP-098`** ("Zero-finding review derives active
instead of done — needs a token round to go terminal"). `IMP-098` reasoned from
`review-ledger.md` §6 — "done when **every** finding is terminal", vacuously true
on an empty ledger — and never consulted `ADR-007`. §6 carries `D-C9a` without
`D-C8`'s empty-ledger carve-out, and the omission read as licence.

**So the naive fix re-opens `IMP-098`, which has field evidence.** `derived_status`
reads only the finding list — never the baton's `rounds`. Under `D-C8` alone a
review that receives no findings can *never* reach `done`: `done` requires every
finding terminal and there are none to terminalise. `IMP-098` records four clean
zero-finding reconciliation audits (`RV-055`, `RV-056`, `RV-061`, `RV-066`) that
each needed a token-nit-then-`withdraw` hack to go terminal. Landing this issue
as originally written restores that hack as the only exit, and parks those `RV`s
in `next` forever.

**`IMP-392` supplies what closes the gap.** Its concluded-pass marker — added
there for a different reason (admissibility of `DEC-138`'s `Conducted` arm) — is
the structured state that distinguishes *not yet conducted* from *conducted,
found nothing*. The finding list cannot express that difference, which is why
both `ADR-007` `D-C8` and `IMP-098` are right about different things.

**Revised fix, once `IMP-392` has landed** (this issue now `needs` it):

```text
empty ledger, not concluded  → (Active, Raiser)   -- D-C8 honoured
empty ledger, concluded      → (Done,   None)     -- IMP-098 honoured
```

plus the doc comment at `review.rs:1002`, the `doc_unresolved_blockers` comment
at `review.rs:1486`, and four tests — the two named above, `derived_status_empty_is_done_none`
(`review.rs:2839`), and `run_new_creates_an_empty_ledger_rv_against_a_real_target`
(`review.rs:3313`), whose own doc comment already claims "Active/Raiser" while its
assertion says `Done/None`. `derived_status_total_over_enum` (`review.rs:2895`)
tests singletons and pairs only, so its "Done ⇔ await=None" invariant is untouched.

Two shipped surfaces were corrected ahead of the code, to stop the next agent
repeating `IMP-098`'s reasoning: `review-ledger.md` §6 now carries the `D-C8`
carve-out, and `.agents/skills/close/SKILL.md`'s zero-finding note no longer
cites `IMP-098` as a live known-issue.
