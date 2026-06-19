# IMP-107: Wire ReviewError::LockContention and ReviewError::DanglingRef to their upstream call sites

## Context

SL-109 defined `ReviewError` with 6 variants (design D8, RV-092 F-1). The MCP
error mapper handles all 6 by variant identity via `downcast_ref`
(tools.rs:390-490). However, two variants are never constructed:

- **`LockContention`** — `lock_guard.rs` uses `anyhow::bail!` for lock
  contention instead of returning `ReviewError::LockContention`. The MCP client
  sees a generic -32603 `Internal` instead of the structured -32000
  `LOCK_CONTENTION` with `canonical` + `details`.

- **`DanglingRef`** — `run_new` calls `crate::integrity::ensure_ref_resolves()`
  which returns a generic anyhow error instead of `ReviewError::DanglingRef`.
  The MCP client sees `Internal` instead of `DANGLING_REF` with the `target`.

The `#[expect(dead_code)]` on `ReviewError` suppresses the compiler warning.

## What to do

1. Convert the lock-guard error path to construct `ReviewError::LockContention`
   and propagate it through `with_turn_hooked`.
2. Convert the dangling-target check in `run_new` to construct
   `ReviewError::DanglingRef` when `ensure_ref_resolves` fails.
3. Remove the `#[expect(dead_code)]` attribute from `ReviewError`.
4. Update the existing unit test for dangling ref rejection to assert on
   variant identity instead of string content.
5. Add a unit test that `LockContention` surfaces correctly when the lock is
   held.

## Links

- RV-097 F-1 (LockContention)
- RV-097 F-2 (DanglingRef)
- SL-109 notes.md (PHASE-04 follow-ups)
