# SL-087 notes

## Audit (RV-060) — 2026-06-17

Clean audit. Zero code defects. Implementation matches settled design exactly.
Two findings: F-1 (aligned — all RV-058 findings faithfully implemented);
F-2 (tolerated — pre-existing `sync_produces_all_shipped_dirs` gate failure,
unrelated to SL-087).

### Durable observations

- `boot_keys()` pattern (narrow, no-parameter projection from `collect_all()`
  with internal filtering + sort) is a reusable seam for future boot-section
  compaction.
- The `section_or_marker` routing pattern now covers every boot source kind
  consistently.
- Key-ascending string sort on `Vec<String>` places period-prefixed keys
  (`mem.signpost.*`) before underscore-prefixed keys (`mem_*`) due to ASCII
  ordering — this is correct but worth noting if new key naming conventions
  are introduced.
