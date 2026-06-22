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

## Related

- CHR-021: Audit and improve shipped memory corpus
- IMP-148: MCP memory tool inline help
