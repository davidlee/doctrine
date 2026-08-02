# IMP-379: Pin the EX-16 runbook clause non-blocking half with a test

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Raised as `RV-342` F-2 (minor) during the SL-233 audit campaign.

`gate::advance` evaluates the runbook clearance clause on **this edge only**,
deliberately outside `cumulative_conditions` (`src/design_run/gate.rs`, the
`if boundary_runbook(from, to).is_some()` block). The code says so and names
itself the machine's **first non-cumulative condition** (`EX-16`).

That is a real departure from how every other condition in the machine works,
and it is load-bearing: it is the difference between a stale step *warning* and
a stale step *blocking an unrelated later transition* — a prose edit to an
explore step must not block a boundary two stages later.

## What is covered, and what is not

The **rendering** half is covered — the envelope surfaces `stale`, and
`tests/e2e_design_runbook.rs` exercises it in ten places.

The **evaluation** half is pinned by nothing. No test advances a run past
`Exploring`, edits an explore step definition to make its prior discharge stale,
and then asserts that the `Drafting → Reviewing` boundary is **not** blocked.
The guarantee survives only as the structure of the `if` block plus a comment.

Measured: `rg -n cumulative_conditions tests/` returns no hits. Positive control
— the same search over `src/design_run/gate.rs` returns three hits, and
`rg -c stale tests/e2e_design_runbook.rs` returns 10, so the search reaches the
right places and `stale` is otherwise exercised.

## The failure mode this leaves open

A refactor that folds the runbook clause into `cumulative_conditions` for
uniformity. Nothing would go red, and the machine would silently start blocking
later boundaries on stale earlier rituals. The comment explains why that is
wrong; no test enforces it.

## The fix

One regression test: advance past `Exploring`, mutate an explore step definition
so its discharge goes stale, assert the later boundary still clears. Cheap, and
it converts a comment into a gate.
