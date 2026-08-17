# Should the unresolvable-refs advisory soften its wording?

Raised at `SL-238` PHASE-04 planning as the sheet's `F-1`, and carried through
implementation unchanged. **Answerable at reconcile**; it costs one string if
accepted and nothing if declined.

## The string

`backlog list --by sequence` emits, on stderr, when any authored `needs`/`after`
ref resolves to nothing (`src/backlog.rs`, the `UNRESOLVED_ADVISORY` constant):

```
backlog list: {n} authored needs/after refs name nothing — run `doctrine doctor`
```

## Why it might overstate

The design (§2) claims *"the advisory's count and §5's check report the same
population"*. They do not, on two independent counts, and closing either would
need a second corpus walk that §4 forbids in terms:

1. **Terminal dependents.** The projection iterates only non-terminal items, so
   a broken ref authored on a terminal item never becomes an `AbsentDrop`, never
   reaches the probe, and cannot be counted. §2 names this class itself — five
   of the 31 edges sit in it — while also making the parity claim.
2. **Unreadable items.** The `doctor` check emits the tolerant reader's read
   failures as findings; the listing path is fail-fast and has no corresponding
   disclosure.

Parity *does* hold over the three broken-ref **classes** on the live projection,
which is what the criteria actually assert. So no criterion is wrong and **no
code changes** — the divergence is a priced tradeoff.

What is at issue is only whether *"N authored needs/after refs name nothing"*
reads as a complete count of what `doctor` will say. §2's own argument against a
partial signpost — *"worse than no signpost, because the reader who follows it
once and finds it complete will trust it when it is not"* — is the case for
softening, and it convicts the design in its own words. This slice exists to
remove a surface that stated something it had not checked.

## The options

- **Decline.** The advisory points at `doctor`, and `doctor` is complete; a
  reader who follows the signpost gets the full answer regardless of the count.
  Zero cost.
- **Soften.** Reword so the count does not imply completeness — e.g. naming it
  as *at least* N, or dropping the count for a bare pointer. One string, one
  constant, and the tests that pin the count would need re-reading (the count is
  load-bearing in three of them).

The narrowing of §2's parity sentence is a **separate, already-accepted**
reconcile action and does not depend on this answer.

## Settled: soften (SL-238 reconcile, 2026-08-17)

The disposition is on the record (`answered`); this is the prose half. The
advisory now declares its count a lower bound and its pointer the complete check:

```
backlog list: at least 3 authored needs/after refs name nothing — run `doctrine doctor` for the full check
```

Settled **together with `RV-363` `F-4`** — the same template hardcoded the plural
and rendered *"1 … refs name nothing"* — because both defects rode one string and
splitting them would have spent the same tests twice. The shape is one template
plus two agreement constants (`STD-001`), and four `contains` assertions in
`src/backlog.rs`, of which the *"at least"* prefix broke none and the singular
broke two. Note for the record: the count was pinned in **four** places, not the
three this item and `F-4` both estimated.

`design.md` §2 carries the same wording and the reasoning.
