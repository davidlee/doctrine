# Write seam must canonicalize every id axis the read view does

A write seam that stores a composite key must canonicalize EVERY id axis the
read/lookup view canonicalizes — or a cell written with a non-canonical ref keys
a different string than the view looks up, and the two silently never meet.

SL-057 PHASE-05 (RV-017 F-1): `coverage record`/`forget` canonicalized the slice
and change axes (`SL-57` → `SL-057` via `canonical_slice_ref`) but stored
`--requirement` verbatim, while the read view `coverage_view::rows` normalizes its
ref via `requirement::canonicalize_fk` (`REQ-1` → `REQ-001`). So `record
--requirement REQ-1` stored key `"REQ-1"` and `show REQ-001` (lookup `"REQ-001"`)
never saw it. Fix: route the requirement axis through the SAME `canonicalize_fk`
the view uses; share one `slice_key(u32)` for the SL-NNN spelling.

Tell: a new normalization helper applied to *some* key fields but not all. When
you add canonicalization to a write path, enumerate every axis of the key and
confirm each matches how the read path resolves that axis. Best-effort
canonicalizers (parse-and-reformat, junk passes through) keep write/read
symmetric even on garbage input — use the read path's exact normalizer, don't
hand-roll a second one.
