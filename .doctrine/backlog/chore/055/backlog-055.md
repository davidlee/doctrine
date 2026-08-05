# CHR-055: Declare baseline host tool requirements in README

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

REV-047 added POL-002 facet (3): a baseline host-tool dependency — one doctrine
needs to function at all — must be named in the project's stated requirements,
including any version floor. Doctrine does not currently meet its own new rule.

`git` is invoked on default paths across the codebase (`status.rs`, `state.rs`,
`retrieve.rs`, `doctor_checks.rs`, `reserve.rs`, `backlog.rs`, `ledger.rs`,
`coverage_verify.rs`). README has an Installation section but no Requirements or
Prerequisites section, so the dependency is undeclared.

## Scope

- A Requirements section in README naming the baseline host tools and their
  version floors.
- Establish the actual floor rather than asserting one. The floor is whatever
  the oldest git invocation in the codebase needs — a survey question, not a
  guess. `worktree` usage likely sets it higher than the rest.
- Check whether anything else qualifies as baseline: doctrine also spawns
  `claude` (`install.rs`) and a configured program (`coverage_verify.rs`), but
  both look feature-scoped rather than baseline and would fall under facet (3)'s
  second limb instead.

## Why it is a chore, not an issue

Nothing is broken for a user — git is universally present in any repo doctrine
governs. This is a conformance debt against a rule doctrine just adopted, and
the cost of leaving it is that the policy's first worked example is a violation.

## Related

- REV-047 — the amendment that created the obligation.
- POL-002 facet (3) — the rule.
- SL-245 — the slice whose `graphviz` dependency surfaced the gap; its `dot`
  dependency is feature-scoped and is discharged by opt-in rather than here.
