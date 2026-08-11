# just gate runs the named-package test gate; just check is the fast root-only inner loop

The justfile splits two gates (the key slug predates the split — the alias is not
authoritative; read this body):

- **`just check`** = `fmt lint lint-js build validate test`, where `test:` runs
  **bare `cargo test`** (root package only, i.e. `default-members`). Fast
  inner-loop variant. `crates/cordage` (incl. `tests/denylist.rs`, the REQ-079
  product-neutrality boundary) does NOT run under `check`.
- **`just gate`** = the same plus `test-all` instead of `test`. This is the
  end-of-phase / pre-commit guard, and it is where cordage is gated.

## `test-all` names its packages — it is NOT `--workspace` (since 2026-08-11)

`test-all:` runs **`cargo test -p doctrine -p cordage`**. It was
`cargo test --workspace` until `RV-353` `F-4` / `ISS-342`.

`--workspace` also selected `crates/doctrine-control`, whose conformance rows are
proved by live `bwrap` — **host-capability-dependent** claims. Off-jail nine of
them fail, so `gate` and therefore `release` were permanently red on the owner's
host. That crate cannot be fixed with `#[ignore]`: it executes `EX-14`/`DEC-156`
on itself at `no_ignored_test_in_this_crate_stands_in_for_a_claim`, which fails
any `#[ignore]` whose reason is not `"instrument: …"`. The fix was **selection,
not skipping** — the default run no longer makes the claim, rather than making it
and abstaining.

**Consequence, and the reason this memory exists: a new workspace member is NOT
auto-gated any more. Add it to `test-all` by name.** The previous version of this
memory promised the opposite, and that promise is now false.

`crates/doctrine-control` is reached explicitly instead:

- **`just capsule-check`** — `cargo clippy -p doctrine-control` + `cargo test -p
  doctrine-control`. In-jail: 299 passed / 0 failed / 9 ignored (~95s).
- **`just capsule-verify`** — the shipped `backend verify` verb.

Both fail loudly rather than skipping on a host that cannot host a capsule. That
is intended.

## `default-members` is `["."]`

The root package alone, since the same change. `crates/doctrine-control` was named
there by `SL-248` `sec-8` so its suite could not ship green by never compiling;
that entry made every default selection platform-dependent, because the crate is
Linux-only with no `cfg(target_os)` gate — both Apple targets failed a release
build (`ISS-343`). The release workflow now also names `-p doctrine`.

So bare `cargo build` / `cargo clippy` / `cargo test` = the root package only.

## History

A cordage-only regression once landed green because `check` ran bare `cargo test`.
SL-052 widened `check` itself to `--workspace`; a later chore reverted `check` to
fast root-only and moved the full guarantee to the new `gate` recipe.

**Supersedes** `mem.pattern.build.just-check-tests-root-package-only` — its
described gate hole (bare `cargo test` skips members; ISS-007 sits red) is CLOSED
as of SL-052. Treat that memory as historical, not current state.

Workspace members = `.` + `crates/cordage` + `crates/doctrine-control`. The slow
legs are cordage's debug-build scale test (~28s, under `gate`) and the capsule
suite (~95s, only under `capsule-check`).

Still live footgun when probing cordage's denylist manually: it bakes
`CARGO_MANIFEST_DIR` at compile time — a stale test binary panics "root resolution
is wrong (0 files)" and MASKS a real vocabulary hit. Force a recompile
(`touch crates/cordage/src/lib.rs`) before trusting a pass/fail. See
`mem.pattern.testing.stale-cargo-bin-exe`.
