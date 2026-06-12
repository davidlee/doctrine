# SL-045 — implementation notes

Durable cross-phase record. Runtime progress lives in the gitignored phase
sheets; this survives them.

## PHASE-01 — Batched corpus scanner (DONE, commit `9127660`)

- `scan_coverage_batch(root, &BTreeSet<String>) -> BTreeMap<String,
  Vec<(CoverageEntry, IsStale)>>` in `src/coverage_scan.rs` — one corpus walk,
  one `git::head_sha` resolve for the whole requirement fan (INV-2 / RSK-006),
  dense over `wanted` (every wanted req keyed; `[]` if uncovered).
- Walker generalized in place: `collect_matching_entries` → dense
  `collect_matching_entries_batch` (BTreeMap seeded over `wanted`; `get_mut` IS
  the membership filter — no separate `contains`). Per-cell staleness closure
  lifted to `stale_each(root, head: Option<&str>, entries)`, shared by the single
  + batch paths (F3, no parallel impl). ISS-006 slug-symlink + parse-skip carried
  verbatim.
- **Q1 → DELEGATE (settled).** `scan_coverage(root, req)` is now a thin wrapper:
  `scan_coverage_batch(&{req}).remove(req).unwrap_or_default()`. Byte-identical
  for the single-req case (same `read_dir` order, single-key bucket); the existing
  `coverage_scan` suite passed UNCHANGED = the proof. No keep-both fallback.
- Equivalence pinned at the `composite` seam (E4), never raw `Vec` order
  (`read_dir` is incidental). VT-1 asserts `composite(batch[req]) ==
  composite(scan_coverage(req))`.
- Gate: `just check` clean; full bin suite 955 passed / 1 ignored (heavy
  2000-file tier `vt4a_scan_fanin_heavy_tier`, run explicitly to confirm cliff).

### Surface the later phases ride
- PHASE-03 (`coverage_view`) calls `scan_coverage_batch` per spec fan — ONE walk
  for all members (INV-2). PHASE-05 wires `doctrine coverage <ref>` onto it.
