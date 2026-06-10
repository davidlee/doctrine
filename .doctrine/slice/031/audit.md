# Audit SL-031 — Dispatch orchestrator funnel

**Mode:** conformance (post-implementation, all 3 phases complete).
**Basis:** `design.md` §5 + §9 validation table; ADR-006 Verification bullets
(funnel / D7 / D2a / branch-point / provision-exclusion); the per-phase
`EN-/EX-/VT-/VA-/VH-` criteria in `plan.toml`.
**Gate:** `just check` green; targeted VT suites green
(`e2e_trunk_minting` 6, `e2e_worktree_branch_point` 1, `e2e_worker_guard` 3,
`e2e_integrity` 7). HEAD b874fc4.

## PHASE-01 (A) — trunk-aware minting + command-tier KindRef registry

| # | Expected (design / criterion) | Observed | Evidence | Disposition |
|---|---|---|---|---|
| 1 | EX-1 / VA-1: `integrity::KindRef { kind: &'static entity::Kind, stem, state_dir }`; `entity::Kind` unchanged `{dir,prefix,scaffold}` (X-4, no `KindIdentity`) | `KindRef` at `integrity.rs:44`; `entity::Kind` is `{dir,prefix,scaffold}` (`entity.rs:66`); `git log -- src/entity.rs` shows no SL-031 commit | `entity.rs:66`; git log | **aligned** |
| 2 | EX-2 / VT-3 (F-5): `reseat` reads `state_dir`; only SL is stateful | `kinds_table_covers_the_twelve_numbered_kinds` asserts `state_dir.is_some() ⇒ ["SL"]` | `integrity.rs:647-653` | **aligned** |
| 3 | EX-3 / VT-5: all 5 `run_new` sites mint via `git::trunk_entity_ids`; no `&[]` | 5 per-verb e2e: slice / adr(governance) / spec / backlog / requirement each mint above a seeded trunk id | `e2e_trunk_minting.rs` (`*_mints_above_trunk`) | **aligned** |
| 4 | EX-4 / VT-3: `KINDS` membership guard pinned to a hand-written prefix literal (R-b, C-IV) | `kinds_table_covers_the_twelve_numbered_kinds` pins the 12-prefix literal | `integrity.rs:639-646` | **aligned** |
| 5 | VT-2: `next_id(local,&[])` byte-identical (no-trunk gate, SL-032 INV-1) | `no_trunk_repo_degrades_to_local_only_mint` green | `e2e_trunk_minting.rs` | **aligned** |
| 6 | VT-4 / R-3: behaviour-preservation — engine suite zero diff, only field-access renames | `entity.rs` untouched by SL-031; validate/reseat/integrity suites green unchanged | `e2e_integrity.rs` 7/7; git log | **aligned** |

## PHASE-02 (B.1) — `branch-point-check` verb + `/worktree mode=worker`

| # | Expected | Observed | Evidence | Disposition |
|---|---|---|---|---|
| 7 | EX-1/EX-2 / VT-1: pure `matches(base,head)` in the leaf, impure HEAD read in shell; Read-classed | `matches` at `worktree.rs:265`; `matches_is_ref_equality` truth table (equal⇒true, differ⇒false, empty⇒false) | `worktree.rs:505-508` | **aligned** |
| 8 | VT-2: e2e exit 0 when `base==HEAD`, exit 1 after a stray commit | `e2e_worktree_branch_point` green (1 test over the built binary) | `e2e_worktree_branch_point.rs` | **aligned** |
| 9 | EX-3/EX-4 / VA-1: `/worktree mode=worker` — source-only, no-degrade hard abort, one non-merge `S` to fork, report + `{fork_branch, head_sha_after}`; D2a refuses authored writes (raw `git commit` permitted) | Worktree skill "Worker mode" §; D2a covered by `e2e_worker_guard` 3/3 | `plugins/.../worktree/SKILL.md:191-244`; `e2e_worker_guard.rs` | **aligned** |

## PHASE-03 (B.2) — `/dispatch` funnel + reconciliation tail

| # | Expected | Observed | Evidence | Disposition |
|---|---|---|---|---|
| 10 | EX-1 / VA conformance: `/dispatch` filled — sole-writer remit, Agent-isolation spawn composing `/worktree mode=worker`, D6 pre-distilled prompt, `DOCTRINE_WORKER=1` self-arm with fail-open disclosure (C-I) | Skill authored; remit table; spawn section links worktree skill (not restated); prompt section + explicit fail-open paragraph | `plugins/.../dispatch/SKILL.md` | **aligned** |
| 11 | EX-2 / VA-1: strict per-batch D7 order — precond(X-1) → net-diff `B..S`(X-2) → R-5 reject → non-committing apply → verify(RED⇒isolate, X-3) → branch-point guard → ONE commit → record-trails-code | Numbered 8-step funnel section in exact order; record step states knowledge trails the commit | dispatch SKILL.md "The funnel" | **aligned** |
| 12 | VA-2: R-5 belt — orchestrator-side (worker-mode OFF, trusted) `git diff --name-only B..S` ∩ `.doctrine/` → report+halt; non-droppable (C-II) | R-5 step 3 + Red Flags "non-droppable", named as the real protection over the fail-open env var | dispatch SKILL.md | **aligned** |
| 13 | EX-3 / VA-3: file-disjoint batching + serial fallback (C-III); conflict / moved-HEAD / `.doctrine/`-touch = report+halt; crash recovery from coordination branch + `git worktree list` | Batching, recovery, and out-of-scope sections all present; report-and-halt stated for all three triggers | dispatch SKILL.md | **aligned** |
| 14 | EX-4: IMP-002 reconciled terminal + resolution | `backlog edit IMP-002 --status resolved --resolution done`; `backlog show` confirms `resolved · done` | `backlog-002.toml`; commit b874fc4 | **aligned** |
| 15 | EX-5 / VH-1: IMP-003 closure prose staged; OQ-1 resolved-defer; backlog→slice edge deferred (C-VII) | `notes.md` "Reconciliation" section stages the IMP-003 flip + OQ-1 + edge-deferral rationale for `/close` | `notes.md` | **aligned (flip lands at /close)** |

## Cross-cutting findings

- **F-1 — slice status divergence (`⚠`, expected).** `slice list` shows `SL-031
  proposed ⚠ 3/3` — the hand-edited `slice-031.toml` status lags the phase rollup.
  This is the **known lifecycle-transition gap** (CLAUDE.md; ADR-009), not drift.
  **Disposition: fix now at `/close`** — `/close` flips the slice to its terminal
  status, which clears the `⚠`.
- **F-2 — IMP-003 flip deferred to `/close`.** Per design §5.4 the IMP-003
  transition is `/close`'s job (status-flip-with-resolution + prose); PHASE-03 only
  staged the prose. **Disposition: aligned** — owned and routed, not loose.
- **F-3 — attribution dropped (conscious).** Unlike `/worktree`, the funnel prose
  is authored wholesale from this slice's design; no prior-art prose was reused, so
  no NOTICE.md / attribution comment. **Disposition: aligned** — recorded in
  `notes.md`; a fabricated derivation would be the defect.
- **F-4 — D2a fails OPEN (residual, by design).** `DOCTRINE_WORKER=1` is a
  self-armed prompt contract (no `Agent` env seam, C-I); the enforceable protection
  is the R-5 import belt. The EDGE-D2b (harness does not confine the worker to its
  fork) is an unchanged residual deferred to ADR-008 (design §5.5).
  **Disposition: tolerated drift** — consciously accepted, belt-mitigated, ADR-tracked.
- **F-5 — pre-existing `policy-001.toml` edit not in scope.** A `draft→required`
  change to `.doctrine/policy/001/policy-001.toml` predates this session and is
  unrelated to SL-031; deliberately **excluded** from the slice commit.
  **Disposition: out of scope** — surfaced for the owner, not touched.

## Closure readiness

Every EX/VT/VA criterion across the 3 phases is **aligned** with test or skill-prose
evidence. The only open items are the two conscious deferrals routed to `/close`
(F-1 slice-status flip, F-2 IMP-003 transition) and one ADR-tracked residual (F-4).
No "fix now" code findings remain. **Audit-ready → `/close`.**
