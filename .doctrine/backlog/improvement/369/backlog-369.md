# IMP-369: Extract common's worker-fork and marker-free-base clusters into sibling modules

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`tests/common/mod.rs` is declared by ~30 integration-test crates, so everything
in it is compiled by all of them. Only four of its items are genuinely
universal:

- `repo_root()`, `doctrine_bin()` (via the `#[path]`-included `src/test_support.rs`)
- `SLICE_DIR` (via `src/kinds/dirs.rs`)
- `DOCTRINE_TOML`

The rest — roughly 85 of its 138 lines — are two concern-specific clusters that
leaked in:

1. **`marker_free_base()`** — wanted only by tests exercising `root::find`'s
   no-root path.
2. **the worker-fork block** — `git`, `init_repo`, `is_linked_worktree`,
   `assert_marked_linked_fork`, `marked_linked_fork`, explicitly tagged
   ISS-028 / SL-236 §9.

## Why

Each cluster is exactly as concern-specific as the design-run bootstrap that was
deliberately kept *out* of `common` in SL-233 PHASE-07 (see
`mem.pattern.tests.shared-helper-placement`). They arrived by the reasoning that
memory now forbids: "it's shared by more than one crate, so it goes in common."
The containment property that makes a sibling module better is that only crates
declaring `mod X;` compile it; `common` has no such opt-in.

Left alone, `common` keeps accreting — it is the default destination for any
helper with two callers, and nothing pushes back.

## How

`tests/worker_fork/mod.rs` and `tests/root_probe/mod.rs` (names negotiable),
each `pub(crate)` and free to use `crate::common::…` for the universal four —
which makes "also declare `mod common;`" a precondition enforced by a compile
error. Then remove the moved items from `common` and add `mod worker_fork;` /
`mod root_probe;` to the crates that actually use them; the compiler names them.

Non-goal: shrinking `common` to nothing. `repo_root`/`doctrine_bin`/`SLICE_DIR`/
`DOCTRINE_TOML` are correctly universal and should stay.

## Sequencing

Independent of IMP-368 but the same shape; doing 368 first gives a second worked
example to copy. Neither is urgent — this is cohesion debt, not a defect.
