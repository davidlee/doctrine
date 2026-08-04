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
