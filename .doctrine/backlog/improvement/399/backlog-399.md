# IMP-399: Extract the design command's four asset-reading section builders

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`src/commands/design.rs` hosts four sibling section builders that all read
shipped assets out of the embed and hand the bytes to a pure renderer in
`src/design_run/`:

- `fragment_section` — the process fragment for the run's stage, bound by digest
- `fragment_lines` — what the run says about the fragments a caller declared
- `runbook_section` (with `runbook_facts`) — the obligation runbook on the
  stage's outbound edge
- `contract_section` — the stage-entry receipt's contract block (SL-244 PHASE-06)

Consider extracting all four into one module whose subject is *the design-prompt
corpus*, leaving the command to resolve the run, project the turn, and emit.

## Why it was raised

SL-244 PHASE-06 needed a home for `VT-1` — set equality between the generated
`Condition` vocabulary, the `design-prompts/conditions/` corpus on disk, and the
publication manifest's rows for that prefix. Three candidate homes, and each had
one thing wrong with it:

- `src/design_run/tests.rs` — **infeasible**, not merely undesirable. Its imports
  are `super::`-only by construction, because the `tests/e2e_design_*.rs`
  binaries `#[path]`-include `src/design_run/mod.rs` *including* its test module.
  A `crate::asset_source` reference there fails to compile in five binaries.
- `src/asset_source.rs` / `src/publication.rs` — would teach generic asset
  plumbing the condition vocabulary. Disqualified by the owner.
- `src/commands/design.rs` — **chosen**, with the cost stated: the test's subject
  is the corpus, and its host is a command that reads assets one key at a time
  and never enumerates them. It is also the first repo-disk-reading test in that
  tier; every other test there is a tempdir fixture over a synthetic repo.

A fourth option — give the corpus its own module and put the gate beside it — was
declined *at that moment* on placement symmetry rather than on merit: three
sibling readers already lived in the command, so extracting only the fourth would
have bought corpus cohesion and spent it on inconsistent placement for the other
three. Extracting all four does not have that defect, and is the version worth
considering on its own terms.

## What to weigh

- The command is ~2.4k lines and the four readers are individually small; the win
  is cohesion and a subject-matched test home, not line count. Measure before
  assuming there is a win.
- Tier: the module reads the embed, so it is engine tier and may reach the
  `design_run` leaf — `crate::install` → corpus → `design_run` renderer, no cycle
  (ADR-001).
- If it lands, `SL-244`'s `VT-1` follows its subject into the new module. That is
  the concrete trigger; without it this is a speculative refactor.
- The digest/receipt grammar stays in `src/design_run/prompt.rs` either way —
  this is about *where the bytes are fetched*, not about who renders them.

Raised at SL-244 PHASE-06 planning, 2026-08-05.
