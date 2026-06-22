# IMP-158: Survey held claims with holder and acquisition time

This fulfills REQ-022 (PRD-005).

The `Claim` trait in `src/entity.rs` currently only provides an atomic
claim-or-detect race result (`Won` / `AlreadyHeld`). REQ-022 requires a
read-only surface that lists held claims under a requested namespace, each
reporting its holder and acquisition time.

Work items:
- Design and implement a `survey` method or associated function on the `Claim`
  trait (or a separate read seam) that enumerates active claims.
- Surface via a CLI verb (e.g. `doctrine inspect` extension or a dedicated
  command).
- For the `LocalFs` backend, this means scanning claim directories and reading
  metadata (mtime as acquisition time, owner from filesystem metadata or a
  marker file).
