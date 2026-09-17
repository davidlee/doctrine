# ISS-457: Spawn-convention scan is blind inside macro invocations

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

`tests/spawn_cwd_convention.rs::every_test_spawns_the_binary_through_the_seam`
enforces the IMP-352 convention: only `common::doctrine_cmd` may resolve the
binary, keyed on the path resolver because "a spawn cannot dodge it". It flags
any test file whose code references `doctrine_bin`.

It cannot see a reference inside a macro invocation. `bin_refs` walks a `syn`
AST, and `syn::visit` does not descend into a macro's token stream, so
`doctrine_bin` inside `assert!`, `format!`, `dbg!` and friends is invisible to
the scan.

Found by `RV-369` `F-5` (SL-245 implementation audit), leg 2. Leg 1 — a real
violation in `tests/e2e_render_guard.rs`, `assert!(common::doctrine_bin().is_file(), …)`
— was fixed under that audit. The gate had been green throughout, because the
reference sat inside `assert!`.

## Why the blind spot is untested

The file's own anti-vacuity test, `scan_separates_a_spawn_from_a_mention`,
covers a bare statement and a bare fn wrapper. Neither is wrapped in a macro, so
nothing in the suite would notice the hole either.

## Scope

Incumbent and repo-wide — it predates SL-245 and was deliberately not folded
into that slice, whose brief was the `-X` seam.

Wants two things:

1. `bin_refs` descends into macro token streams (or otherwise detects the
   identifier there), so a `doctrine_bin` inside `assert!`/`format!`/`dbg!` is
   flagged.
2. A row in `scan_separates_a_spawn_from_a_mention` covering the macro case, so
   the hole cannot silently reopen.

Worth reading `RV-369`'s synthesis alongside this: the same shape — a guard that
verifies its rule exhaustively and never exercises its own premise — is what
`F-1` and `F-2` are about.

## Related

`IMP-352` (the spawn-seam convention this guard enforces), `RV-369` `F-5`,
SL-245.
