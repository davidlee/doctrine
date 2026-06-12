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

## AUDIT + CLOSE (RV-005, commit `03a0d7a`)

Reconciliation review RV-005 (`done · await=none`). Two-pass review: my own
seam-read of the design contract + an external adversarial pass (codex gpt-5.5).

- **F-1 (major → fix-now → verified).** `spec req list` skipped the
  `validate_statuses` known-set guard every sibling list surface performs, so
  `--status bogus` silently emptied the roster instead of erroring (F4/SL-025
  uniform-contract breach). Surfaced by codex (E#1), confirmed at the seam. Fixed
  in-slice (03a0d7a): `requirement::REQ_STATUSES` const + drift canary; the
  `validate_statuses` call in `req_list_rows` mirroring `list_rows:1170`; red/green
  test. Durable lesson → memory `mem.pattern.listing.validate-statuses-is-opt-in`.
- **Codex E#2 REJECTED (no finding).** Claimed the `coverage_view.rs:39`
  `expect(dead_code)` is now an unfulfilled lint expectation breaking the gate.
  Empirically false — `cargo clippy` recompiles clean; the expectation still holds
  (a residual not-yet-wired item keeps `dead_code` firing). Static reasoning
  without building.
- **All design invariants verified holding at the seam** (not inferred from green
  tests): INV-1 wall, E2 canonicalize, E1/INV-4 dangling (enum + forbidden-keys),
  INV-2 one walk, INV-3 authored-only, E6 partition, ADR-001 no cycle, D5 single
  source, F6 (977 baseline + integration unchanged; SL-044 residency proof green).
- **Relaxed dangling-table goldens** consciously accepted (tempdir-path float;
  healthy cells stay byte-exact, dangling JSON forbidden-keys asserted exactly).
- **Dogfood drift → CHR-003.** REQ-108..116 still authored `pending` while the
  read surface ships; reconciling that corpus drift is out of SL-045 scope
  (a surfacing slice) — captured as a backlog chore.

Lifecycle: `plan → ready → started → audit → reconcile → done` (D-C9b gate clear,
RV-005 has no unresolved blocker).
