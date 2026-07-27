# Notes SL-234: Review prime ignores non-file selector entries

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-27 · PHASE-01 · 40bb99611

### Produced

- PHASE-01 done — review glob expansion excludes non-file Git entries; RV-315
  primes successfully (commit f630cbe4)
- ISS-259 — repair delivered by SL-234

### Learned

- No new reusable memory: the strict content-set ownership rule is already
  carried by SPEC-004 and `mem.concept.doctrine.entity-engine`.

### Open

- None.
