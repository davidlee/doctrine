# IDE-019: backlog list: verbose/explain flag to surface override/dangling warnings explicitly

## Context

`doctrine backlog list` currently emits override/dangling warnings unconditionally
for every override whose source or target is terminal (resolved/closed) or absent.
These grow noisy as the backlog ages — a terminal item's overrides are often
uninteresting, and the noise buries the exceptions that matter.

## Proposal

Add a `--verbose` (or `--explain`) flag to `doctrine backlog list` that gates
override/dangling warnings behind an explicit opt-in. The default output would
suppress warnings where all involved items are terminal or absent.

### Suppression heuristics

- Override warning suppressed when **both** source and target are terminal
  (resolved or closed) — no action needed.
- Override warning surfaced (even without `--verbose`) when at least one side
  is non-terminal (open/triaged/started) — dangling references on live work
  are actionable.
- Override warning surfaced when either side is absent (still dangling on
  live or terminal).

### Flag naming

Candidate: `--verbose` (consistent with unix conventions). Alternative:
`--explain` (more precise — it explains why items were suppressed).

## Scope

- CLI change only: `doctrine backlog list`
- No schema/model changes
- Backward-compatible: existing output unchanged under `--verbose`

## Open questions

1. Should a dangling dep on a terminal *non-resolved* (e.g. an override where
   the source is resolved/closed but target is dismissed/invalid) still show
   without `--verbose`? Arguably yes — a live target matters.
2. Should `--verbose` also control other warnings, or only overrides?

## Outcome: intent delivered, mechanism declined twice (SL-238, 2026-08-17)

`SL-238` fulfils this item. The problem it names is fixed — `backlog list` no
longer buries actionable dangling references in noise about terminal ones — but
**neither of the two mechanisms proposed above was adopted**, and the divergences
are recorded here rather than only in the slice's notes, because a reader of this
item is who they concern.

- **The flag was declined (`DEC-234`).** No `--verbose`/`--explain` was added to
  `backlog list`. The per-item detail this item wanted behind an opt-in went to
  `doctrine backlog inspect` / `show` instead, which annotate each `needs`/`after`
  ref with its target's state. A flag would have made the listing surface answer
  two questions at two verbosities; a second verb already existed for the second
  question.

- **The dangling-ref report was resited, not gated (`DEC-232`).** The proposal
  assumed the warnings belong in the footer and only need suppressing. `SL-238`
  ruled the other way: **authored data that is wrong is `doctor`'s business, not
  the listing surface's.** Unresolvable `needs`/`after` refs are now reported by
  `doctrine doctor`'s ref-integrity leg, over every item including terminal ones —
  a wider population than any footer flag could have reached. `backlog list` keeps
  only a count-only stderr signpost pointing at `doctor`.

What replaced the suppression heuristics is a stricter version of the same
instinct: the footer's `boundary:` block discloses exactly the edges whose
dependent **and** target are both non-terminal, having checked both — which is the
"surfaced when at least one side is non-terminal" rule above, tightened, and made
true rather than asserted. The founding defect `SL-238` fixed was the old footer
claiming `dropped (dangling: SL-182 absent)` about an entity that existed.

The two open questions above are answered by that siting: `doctor` reports every
broken authored ref regardless of either side's status (Q1), and the resiting
applies to dep/seq refs only — no other warning class moved (Q2).

## Related

- CHR-021: Audit and improve shipped memory corpus
- IMP-148: MCP memory tool inline help
- SL-238: fulfils this item — see the outcome above (`DEC-232`, `DEC-234`)
