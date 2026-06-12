# just check gate tests root package only; cordage crate suite is ungated

`just check` runs bare `cargo test`. The repo root is itself a package AND the
workspace root, so bare `cargo test` exercises only the root package's tests —
NOT the workspace members. `crates/cordage` is a member, so its integration
suites (notably `tests/denylist.rs`, the REQ-079 product-neutrality boundary) are
never run by the gate. A cordage-only regression lands green.

To actually exercise cordage: `cargo test --workspace` or `cargo test -p cordage`.
Verified: bare `cargo test` shows no `Running tests/denylist.rs`; `cargo test
--workspace` does (and at 2026-06-12 it fails on a whole-word `task` in the cordage
README — see ISS-007).

Watch-out when probing this manually: cordage's denylist test resolves its root via
`env!("CARGO_MANIFEST_DIR")` (compile-time baked). A stale test binary compiled
inside a since-removed worktree panics with "root resolution is wrong (0 files)"
and MASKS the real vocabulary hit — force a recompile (`touch crates/cordage/src/
lib.rs`) before trusting a pass/fail. See `mem.pattern.testing.stale-cargo-bin-exe`.

Surfaced by the SL-047 audit (RV-007 F-1).
