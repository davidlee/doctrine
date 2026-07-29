# IMP-357: Test binaries duplicate the design-run fixture

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The duplication

SL-233 PHASE-13 created `tests/e2e_design_materialise.rs`. Its two tests need the
same design-run harness `tests/e2e_design_state.rs` already carries: start the
run, learn the uid and snapshot path, build an envelope carrying the live
revision, apply, materialise, read the document back. That harness was **copied**
— roughly 120 lines — in a repo whose standards say *no parallel implementation*.

## Why the obvious fixes do not apply

- Integration tests are **separate compilation units**, so a fixture cannot be
  imported from a sibling test binary.
- The crate is **binary-only** (no `[lib]` target), so there is no library seam
  to import from either.
- The one shared seam that exists, `tests/common/`, is compiled into **every**
  test binary. Hoisting the design fixture there would force all ~90 of them to
  compile the `design_run` leaf tree via its `#[path]` include — a large,
  unrelated build cost paid by every test target.

## Why it is worth filing now rather than later

PHASE-14 adds **six more tests** to the duplicated copy (its `EX-8` completes
`tests/e2e_design_materialise.rs` at exactly eight named tests). The duplication
therefore stops being incidental and becomes load-bearing in two files that must
be kept in agreement by hand — the failure mode being a fixture fixed in one copy
and not the other.

## Candidate directions

1. A `[lib]` target exposing the leaf tree, so integration tests import rather
   than `#[path]`-include it. Largest change, cleanest end state, and it would
   also dissolve IMP-357's sibling symptom recorded as SL-233 PHASE-13 finding
   F-10 (leaf-tree unit tests re-running inside every including binary, which
   inflates the `cargo test -- --list` count used as `VA-ES` evidence).
2. A `tests/common/design.rs` module included **only** by the binaries that need
   it, keeping the ~90 other targets untouched.
3. Accept the duplication and pin the two copies with a shape assertion.

Direction 1 is the structural answer but is out of any single phase's scope and
would touch the whole test tree; it wants its own slice.

## Provenance

SL-233 PHASE-13, worker hand-back finding F-9. Friction observation
`019fada4-f957-7bb3-aea3-7c932fed8e71` carries the worker's own account.
