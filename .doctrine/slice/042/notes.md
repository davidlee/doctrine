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
