# SL-042 implementation notes

Durable implementation notes (storage rule: prose, no queried data). Harvested
into the audit at close.

## PHASE-02 — coverage substrate (REQ-109) — landed `e9fcc36`

- **Key ownership relocated (D-P2-1).** The 4-tuple `(slice, requirement,
  contributing_change, mode)` now lives in `src/coverage.rs` as `CoverageKey`
  (the cited thing owns its key); `src/rec.rs` imports it via
  `use crate::coverage::CoverageKey as EvidenceRef;`. rec's existing code and
  tests compile byte-unchanged. Corrects the backward coupling left by P1's
  sequencing (rec shipped first, so the key was born there). `mode` stays
  `String` in the key — rec is a verbatim ledger; membership ∈ {VT,VA,VH} is a
  coverage-layer validator, not a typed key field.

- **EX-2 / R-b premise was wrong — finding for the plan.** Plan EX-2 says the
  `CoverageStatus` `not(test) expect(dead_code)` is removed because the enum
  becomes "genuinely used in the non-test build" in P2. It does **not**: P2 is
  the *store*, and the whole coverage leaf has no runtime consumer until the
  P3/P4 reconcile reader. So the entire `coverage.rs` leaf (CoverageStatus
  included) is dead in `cargo clippy` (bins/lib). Deleting requirement.rs's
  suppression alone breaks the gate. **Resolution:** requirement.rs's suppression
  was removed as planned, and a module-level
  `#![cfg_attr(not(test), expect(dead_code, reason=…))]` was added to
  `coverage.rs` (the dead-code-self-clearing-leaf precedent). The
  `CoverageEntry.status` field references `CoverageStatus`, so it is no longer
  dead in requirement.rs; the leaf suppression retires itself when P3/P4 wires a
  consumer. **EX-2's "genuinely used in the non-test build" is therefore
  satisfied at P3, not P2** — reconcile at close.

- **VT-4 residency** shipped as `tests/e2e_coverage_authored_residency.rs`
  (integration, black-box `git check-ignore`): a rendered `coverage.toml` under
  `.doctrine/slice/NNN/` is tracked by default — coverage rides the
  `!.doctrine/slice/` default-track with **no negation row of its own** (D-Q1
  confirmed; the STOP condition for a needed negation was not triggered).

- **Surface:** pure leaf only — `parse`/`render` (serde toml round-trip, auto
  escaping), `upsert` (within-file no-clobber fold), `mode_is_valid`. No CLI
  verb, no disk I/O in the leaf (A-2/A-4). The corpus-scan + fs shell is P3
  (`scan_coverage`).

## Dispatch / concurrency context (this run)

- PHASE-02 was built via the **dispatch funnel** (orchestrator sole-writer) with a
  single worktree worker (`sl-042-p2-fork`), because a **concurrent SL-043
  inquisition session was live on `main`** (committing + amending the tip, dirty
  working tree). To preserve the sole-writer premise, SL-042 work runs on an
  **isolated coordination branch `sl-042-coord`** forked from the clean PHASE-01
  commit `3283727`, not on `main`. **P2–P04 land on `sl-042-coord`; merge to
  `main` once `main` settles.**
- **Tooling gotcha:** `just check` cannot load in a fresh worktree — the
  `justfile`'s `mod doctrine '.doctrine/doctrine.just'` import is an installed
  (gitignored) file absent from checkouts. The gate was run as its four
  constituent steps directly (`cargo fmt --check`, `cargo clippy`, `cargo test`,
  `cargo build`). Verify worker-mode **off** (the `DOCTRINE_WORKER=1` guard makes
  the `adr status` e2e goldens refuse-and-fail; orchestrator verify must unset it).
- **Worktree linkage is fragile under the concurrent session.** The SL-043 session
  on `main` ran something (gc/prune/worktree-remove) that wiped
  `.git/worktrees/`, ORPHANING the linked coord/p2 worktrees mid-run (their `.git`
  pointer dangled; the env also carried empty `GIT_DIR`/`GIT_WORK_TREE`). NO data
  lost — branch `sl-042-coord` and all commits survived in the object store; the
  worktree was just `prune`d + re-`add`ed. **The branch is the durable artifact,
  not the worktree dir.** Run git as `env -u GIT_DIR -u GIT_WORK_TREE git -C
  /workspace/sl-042-coord …` to dodge the empty-GIT_DIR env.

## PHASE-03 forward (pre-planning findings — not yet implemented)

Surfaced while detailing P3; resolutions are design-faithful, additive, low-risk.

- **Schema gap: `CoverageEntry` lacks `touched_paths`.** Design names
  `touched_paths` 3× (§5.2 staleness call, §8 R-e) as the coverage attribute the
  staleness seam consumes, but §5.3's field LIST omitted it, so P2's landed
  `CoverageEntry` has no such field. **Fix in P3:** add `touched_paths:
  Vec<String>` with `#[serde(default)]` (additive — P2 round-trip tests stay
  green). The staleness check needs it: `commits_touching(root, &touched_paths,
  since=git_anchor, target=head_sha)`.
- **`scan_coverage` placement:** `src/coverage.rs` is a PURE LEAF (its module doc
  asserts "no git/disk"). The impure `scan_coverage` (corpus walk + git seam) must
  live ABOVE the leaf (ADR-001) — a NEW `src/coverage_scan.rs` shell, NOT inside
  the leaf. P4's staleness wiring grows there too.
- **`IsStale` is a leaf type** (`coverage.rs`): `Fresh | Stale | Unknown`, mapping
  the seam's `Some(0) | Some(≥1) | None`. The shell PRODUCES it; the pure
  `composite(&[(CoverageEntry, IsStale)])` CONSUMES it (purity F1: staleness
  resolved in the shell, never in the fold).
- **Git seam:** `crate::git::commits_touching(root, paths: &[String], since: &str,
  target: &str) -> Option<u32>` at `src/git.rs:901`. **Refuses literal `HEAD`** —
  resolve `HEAD → frozen SHA` ONCE per query upstream (`git rev-parse HEAD`), feed
  as `target`. `Some(0)⇒fresh, Some(≥1)⇒stale, None⇒undecidable`.
- **dead_code suppression persists through P3.** The module-level
  `#![cfg_attr(not(test), expect(dead_code))]` on `coverage.rs` stays: P3 still
  has NO bins/lib consumer (no CLI verb; the closure gate is Slice B), so the leaf
  remains dead in the gate build. EX-2's "genuinely used in the non-test build"
  therefore lands with the consumer (Slice B / a later read verb), not P3.

## PHASE-03 — composite + drift + scan shell (REQ-110/111) — landed `0c6c802`

Built via the dispatch funnel (orchestrator sole-writer) with a single worktree
worker `sl-042-p3-fork` (S=`20d2848`) forked off P2's `6e71e24`; net diff `B..S`
imported, R-5 belt clean (src-only: coverage.rs, coverage_scan.rs, git.rs,
main.rs), combined-tree verify green (fmt/clippy/test/build).

- **Folds (pure, `coverage.rs`).** `composite(&[(CoverageEntry, IsStale)]) ->
  Composite` fans one requirement's cells in across contributing changes,
  **deterministic** (sorted by the stable `CoverageKey`; no clock/rng/map-order),
  surfaces all (no precedence — OQ-3). Exposes `is_empty / any_fresh_verified /
  any_failed_or_blocked / only_forward`. `drift(ReqStatus, &Composite) -> Verdict`
  encodes the §5.2 **total** tree; returns `Verdict { Coherent,
  Divergent(DivergentReason{ObservedContradiction|EvidenceOutrunsAuthored}),
  Indeterminate }` — **never a `ReqStatus`** (type-level NF-001). Staleness is
  resolved in the shell and passed in (purity F1).
- **`IsStale` leaf** (`Fresh|Stale|Unknown`) with `From<Option<u32>>`
  (`Some(0)⇒Fresh, Some(≥1)⇒Stale, None⇒Unknown`) — maps the `commits_touching`
  seam result. **`touched_paths: Vec<String>`** added to `CoverageEntry`
  (`#[serde(default)]`, additive — P2 round-trips unchanged; closes the §5.3
  field-list gap flagged in "PHASE-03 forward").
- **Shell (`coverage_scan.rs`, NEW).** `scan_coverage(root, req)` corpus-walks
  `.doctrine/slice/*/coverage.toml`, filters by requirement, resolves `HEAD→SHA`
  ONCE via new `git::head_sha(root)` (T0 — minimal helper reusing the existing
  `rev-parse --verify HEAD^{commit}` form; `commits_touching` contract
  untouched), then per-entry staleness. Missing tree / unreadable / malformed /
  unborn HEAD all degrade, never abort. The ONLY git/disk in the data flow.
- **dead_code expects persist** (as predicted): no bins/lib consumer in P3, so
  `coverage_scan.rs` carries its own module-level `not(test) expect(dead_code)`,
  `coverage.rs`'s covers its new items, and `git::head_sha` got an item-level one.
  EX-2 "genuinely used in non-test build" still lands at the Slice-B consumer.

### R2 perf spike (VT-4) — debug timings, two axes measured separately
- **(a) scan fan-in** (walk+parse+filter, IsStale precomputed, no subprocess):
  N=50→3.33ms, 500→20.3ms, 2000→77.7ms (~0.039 ms/file, **linear, no cliff at
  2000**). 2000 tier is `#[ignore]`d to keep the gate light.
- **(b) staleness resolution** (per-call `git::commits_touching` subprocess vs
  the real repo): **~4.09 ms/call**, linear (one `merge-base`+`rev-list` pair
  per call) — the **dominant** cost.
- **Conditioned backlog triggers (EX-5 — recorded, NOT yet fired; no consumer
  exists until Slice B):** (a) a future scan-axis cliff ⇒ add a reverse-index
  (requirement→entries) so the corpus walk isn't re-paid per query; (b) since
  axis (b) dominates, when a reconcile read resolves *many* entries ⇒ batch the
  staleness resolution (single `rev-list` over the combined pathset, or memoize
  per anchor) rather than one subprocess per cell. Revisit when Slice B wires the
  reader; capture as `backlog new` then if still warranted.

## PHASE-04 — VH/VA staleness decay, NF-002 lock (REQ-115) — landed `8fcc7fe`

Built via the dispatch funnel with a single worktree worker `sl-042-p4-fork`
(S=`eecbe1f`) off P3's `5193238`; net diff `B..S` imported, R-5 belt clean
(src-only: `coverage_scan.rs` test module), combined-tree verify green.

- **Verification-led — NO production change.** P3's `scan_coverage` already
  resolves `IsStale` generically per entry, and NF-002 holds **structurally**
  (`IsStale` is a value distinct from `CoverageStatus`; the shell never mutates
  status). P4's job was to LOCK that contract with tests + confirm the seam-fit,
  not add a mechanism. Four tests in `coverage_scan.rs`'s `#[cfg(test)]` module.
- **R-e / H1 → FACT (EX-1).** `git::commits_touching(root, paths, since, target)`
  consumes coverage's `(git_anchor: String, touched_paths: Vec<String>)`
  granularity **verbatim** — driven against a temp git repo with a `CoverageEntry`'s
  actual field shapes; both Stale and Fresh resolve. **No leaf widening, no fork.**
  H1 ("reuse git.rs unchanged") is now test-backed, not hypothesis.
- **NF-002 lock (VT-1, EX-2).** Temp git repo: a VH **and** a VA `Verified` entry
  anchored at an old commit over `touched_paths` a later commit moved past →
  `scan_coverage` flags **both** `IsStale::Stale` while each `status` stays
  `Verified` — surfaced, **never auto-demoted**. Fresh contrast (untouched path)
  → `Fresh`, status unchanged.
- **No parallel impl (VT-2, EX-3).** Structural guard: coverage staleness flows
  ONLY through `git::commits_touching`; `contentset.rs` (SL-040's content-hash
  leaf — wrong model) is not on the coverage path. (Guard composes the rival
  `contentset::` token at runtime so it doesn't self-match its own source.)
- **dead_code expect persists** — P4 added no bins/lib consumer; `coverage_scan`'s
  module-level `not(test) expect(dead_code)` still self-clears at the Slice-B reader.

### Carry-forward for /audit + /close
- **All 4 phases complete** on `sl-042-coord` (tip `8fcc7fe`). Coord branch is the
  deliverable; **merge to `main` once `main` settles** (SL-043 was still live on
  `main` during this run).
- **EX-2 reconciliation (P2 plan finding):** "`CoverageStatus` genuinely used in
  the non-test build" still does NOT hold — the whole coverage leaf + scan shell
  remain dead in bins/lib (no CLI verb; the reader is **Slice B**). The
  self-clearing `not(test) expect(dead_code)` on `coverage.rs` and
  `coverage_scan.rs`, and the item-level one on `git::head_sha`, are all carried
  intentionally and retire at the Slice-B consumer. Audit should record this as a
  known, designed deferral (EX-2 lands at the consumer), not drift.
- **Conditioned perf-spike backlog triggers** (P3 notes) remain unfired — revisit
  at Slice B.
