# IMP-368: Migrate the six remaining design e2e crates onto tests/design_fixture

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`tests/design_fixture/mod.rs` (SL-233 PHASE-07) holds the design-run bootstrap:
a four-field `DesignRun`, `start()`, and the `run`/`fail` spawners. Two crates
share it — `e2e_design_projection.rs` and `e2e_claude_install.rs`. **Six more
still carry their own byte-identical copy:**

    tests/e2e_design_state.rs
    tests/e2e_design_checkpoint.rs
    tests/e2e_design_delegation.rs
    tests/e2e_design_import.rs
    tests/e2e_design_materialise.rs
    tests/e2e_design_review.rs

Each has the same `struct Fixture { _tmp, root, snapshot, uid }` and the same
`start()` body, differing only in the spawner's name (`run` vs `ok(spawn(..))`)
and the doc comment. Seven copies existed before the lift; five remain
unmigrated after it (projection was converted as the proving second consumer).

## Why it wasn't done in PHASE-07

Out of scope: PHASE-07's surface is install assets and the prompt pack. Touching
six test crates that PHASE-11/12 authored would have widened the phase's blast
radius for no criterion. The lift was justified by the *new* consumer needing a
bootstrap it could not see.

## How

Per crate: delete the local `struct Fixture` + `start()` + spawners, add
`mod design_fixture;`, `use design_fixture::{DesignRun, SLICE, run};` and keep
any model-reading accessors as a **local second inherent `impl` block** — legal
because the type is defined in a module of the same crate. That is exactly the
split `e2e_design_projection.rs` now demonstrates; copy its shape.

Watch for divergence rather than assuming it away: `e2e_design_delegation.rs`
names its constructor `inquiring()` and seeds extra state, and
`e2e_design_checkpoint.rs` wraps its spawner as `ok(spawn(..))`. Those are
per-crate extensions on top of the same bootstrap, not reasons to keep the copy.

Do NOT move accessors that read the snapshot through the `#[path]`-included
`design_run` leaf into the shared module — that would drag the leaf into every
including crate and is the thing the sibling-module placement exists to avoid.

## See also

- `mem.pattern.tests.shared-helper-placement` — why this is a sibling module and
  not `tests/common/`.
- IMP-369 — the same extraction owed for `tests/common/`'s own leaked clusters.
