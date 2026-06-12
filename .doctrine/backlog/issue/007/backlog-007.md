# ISS-007: cordage denylist test red: 'task' whole-word in crates/cordage/README.md trips REQ-079 boundary

## Repro

`cargo test -p cordage --test denylist` → `crate_source_carries_no_forbidden_vocabulary`
FAILS: `crates/cordage/README.md: forbidden token <task>` (whole-word, REQ-079
product-neutrality boundary). Hits `crates/cordage/README.md:22` and `:223`.

## Origin & scope

- Introduced by `dc120a7 doc(cordage): README` — a cordage-internal doc commit,
  pre-dates and is disjoint from SL-047 (SL-047 never touched the README).
- The `just check` gate stays GREEN because `cargo test` runs only workspace
  default scope; `cordage` needs `-p cordage` to test, so the denylist suite is
  outside the gate. This is a gate-coverage hole as much as a vocabulary hit.

## Surfaced by

SL-047 audit (RV-007 F-1). Worker originally misreported it as non-reproducing —
that was a stale test binary baking a removed dispatch-worktree `CARGO_MANIFEST_DIR`
(root-resolution panic masking the real hit); a clean recompile surfaces it.

## Fix options

- Reword README lines 22/223 to drop the whole-word `task` (e.g. "work item"), OR
- bring the cordage denylist suite into the `just check` gate so the boundary is
  actually enforced (the deeper fix — the hole let this sit red unnoticed).
