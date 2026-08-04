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
  against this behaviour. Its conclusion survives the fix — a clean pass and an
  unrun pass are indistinguishable *from the finding set* either way, which is why
  `DEC-138` requires a concluded-pass marker (`IMP-392`) — but the design's stated
  premise needs rewording once this lands.

## Not urgent

Nothing is known to be broken by it today: `await` is a display summary and never
an exclusive gate (`ADR-007` D7 — the turn gate is per-finding `can`). The cost is
that the invariant an accepted ADR states, and tests, is false — and a design has
already load-borne on the false version.
