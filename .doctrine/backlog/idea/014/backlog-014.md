# IDE-014: Clarify requirement acceptance_criteria contract — optional, not per-requirement

## Context

`acceptance_criteria` on a requirement is optional elaboration, not an
expected-per-requirement field. ~52% sit empty today — by design, not debt.

Verification authority lives elsewhere: the coverage matrix plus slice
`VT`/`VA`/`VH` criteria, not per-requirement AC.

## Proposal

Have spec `validate` surface an **advisory** empty-AC count (informational,
non-blocking) so the empty-by-design state reads as intentional rather than
incomplete.
