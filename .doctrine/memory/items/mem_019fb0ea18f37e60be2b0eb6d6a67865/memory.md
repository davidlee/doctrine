## The rule

`tests/common/` is for helpers that are **universal** across integration tests.
A helper specific to one concern goes in its own sibling module directory:

    tests/<concern>_fixture/mod.rs      # e.g. tests/design_fixture/mod.rs

Consumed by `mod design_fixture;` in each crate that wants it.

## Why the sibling works — and why `common` doesn't contain the cost

Only top-level `tests/*.rs` files are compiled as crates. A subdirectory is
inert until some crate declares `mod X;`. So:

- a sibling module is compiled **only** into crates that opt in — cost and
  coupling are both opt-in;
- `tests/common/` is the opposite: ~30 e2e crates already declare `mod common;`,
  so anything added there is compiled by all of them, wanted or not.

That asymmetry — not style — is the reason to prefer the sibling.

## Mechanics when you do it

- A sibling module may use `crate::common::…` (e.g. `doctrine_bin()`,
  `SLICE_DIR`). That makes "also declare `mod common;`" a precondition of the
  including crate — enforced by a compile error, so it cannot rot silently.
  Say so in the module's doc comment.
- **Name collision to avoid:** every `tests/e2e_design_*.rs` already declares
  `mod design_run;` for the `#[path]`-included leaf `src/design_run/mod.rs`.
  A fixture module cannot reuse that name — hence `design_fixture`.
- Keep model-reading accessors out of the shared module. A type defined in a
  module of the crate can take a second inherent `impl` block in that same
  crate, so crate-specific methods stay local to the crate that needs them.

## `common`'s existing drift (do not extend it)

At the time of writing `tests/common/mod.rs` is 138 lines, of which only
`repo_root` / `doctrine_bin` / `SLICE_DIR` / `DOCTRINE_TOML` are universal. The
rest are two concern clusters that leaked in by exactly the reasoning this
memory forbids:

- `marker_free_base()` — only root-resolution tests want it;
- a five-function worker-fork block (`git`, `init_repo`, `is_linked_worktree`,
  `assert_marked_linked_fork`, `marked_linked_fork`), tagged ISS-028 / SL-236 §9.

Both are extraction candidates. Adding a third cluster is the heresy.

See [[mem.pattern.doctrine.tdd-loop]] for the surrounding test discipline.
