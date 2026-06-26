# Notes SL-154: Reliable conformance-registry capture

Durable per-slice scratchpad — tracked in git. Bootstrap for the next agent (plan +
codex). `design.md` is the authority; this is the fast-ingest map.

## Status (2026-06-26) — Rev 6 DESIGN-CONVERGED (passes 5–8). Plan-ready pending User lock.

> "Fixed" below = **fixed in the design prose. NO CODE YET.** Pass-8 (codex) confirmed the
> Rev-6 prose internally consistent + the remediation doctrinally sound, and found no new
> design defect; its "blockers" were design-vs-code divergence (the expected design-stage
> state = the /plan worklist, not defects). Next reviewer prompt MUST say "review the
> design prose, not the source tree".

Slice in `design`. `design.md` = committed-ref (ISS-039-absorbed) model + **pass-5 reshape
(Rev 4)** + **pass-6 edges (Rev 5)** + **pass-7 fixes (Rev 6)**: D11 is a **per-phase
provenance set-check** (`provenance ∈ {Funnel, Unknown}` must be in the committed ledger),
`D12` adds `provenance` to `BoundaryRow` with a **sticky merge in `record_source_delta`**
(incoming `Solo`/`Funnel` overwrite; `Manual` preserves existing incl. `Unknown`; atomic,
race-free), `git::code_delta_paths` **deleted**, §5.6 = registry nav/value role.
committed-ref core (D2/D7/D8/D9/D10) solid 5 passes; passes 6–7 found 4 edge defects
(record-delta downgrade, Unknown-on-active, wrong-seam race, doc drift), all fixed. Scope
absorbs ISS-039. **No code yet.**

**Next:** re-pass codex on **Rev 6**; if clean, `/plan`. Pass-5 OPEN FINDINGS **CLOSED**;
pass-6/7 dispositions in design.md §10.

Read order: `slice-154.md` (scope) → `design.md` §1–§9 → §10 (full pass ledger) → this.

## ⚠ OPEN FINDINGS — CLOSED (integrated into design.md Rev 4)

Pass-5 (D11 unsound) + the efficiency lens are **integrated**; this block is retained as a
disposition record only.

- **P5-1 empty-only too weak** → D11 reshaped to a per-phase set-check (catches partial loss).
- **P5-2 false-halts mixed/empty-code** → provenance excludes `Solo`; guard never reads code paths.
- **P5-3 exclusion-set mismatch** → `code_delta_paths` deleted; pure phase-id set comparison.
- **Provenance** → **D12**: `provenance: Solo|Funnel|Manual` (absent→`Unknown`) on `BoundaryRow`.
  The "derive from the writer (cheaper)" option was **illusory** — no committed run-state
  records landing path (`journal.toml` is derived from boundaries; `candidates.toml` is
  refs), so persisting the writer's identity = a field. `#[serde(default)]` is the entire
  back-compat story; **no migration machinery** (closed slices never re-hit D11; active
  mid-flight slices hand-fixed). `boundary.rs` → design-target.
- **Nav-coherence** → §5.6 claims it as value; derived nav view (`slice show --phase-files`)
  **backlogged**, gated on SL-154.
- **F4 NOT closed by provenance** → ownership-signal hardening stays a backlog follow-up
  (provenance marks ownership post-record; F4 is the binding's pre-record stand-down).
- **Halt ergonomics** → D11/gate name the missing **phase ids**.

## What this slice is

Close two conformance-registry **population** leaks RFC-004 v0.1 (SL-147) left, and make
recording robust across solo↔dispatch landing-path transitions. The **registry** is
`.doctrine/state/slice/NNN/boundaries.toml` (runtime) — one `[[boundary]]` row per landed
phase — the actual-side input to `slice conformance` at audit. ISS-051 = solo final-phase
miss; ISS-052 = funnel never reliably populates. **Now also absorbs ISS-039** (the
dispatch *ledger* `boundaries.toml` is never committed to `dispatch/NNN` → projection +
the spec-legal derive source read empty). Claude-arm-bounded; codex/pi symmetry stays
IMP-171. References RFC-004.

## The locked design (committed-ref model)

Two ledgers, do not conflate:
- **Registry** `.doctrine/state/slice/NNN/boundaries.toml` (runtime, primary-rooted) —
  the conformance input.
- **Dispatch ledger** `.doctrine/dispatch/NNN/boundaries.toml` (claude-only) — the
  projection source `plan_phases` reads, and now the derive's source. SPEC-022 mandates
  it be tree-read from the `dispatch/NNN` **committed tip**, never the working FS.

Five moving parts:

1. **Solo binding** (`state.rs::capture_phase_boundary`). Keep the stamp; record
   `(stamp, HEAD)` at the `completed` flip. Changes: (a) guard predicate branch-proxy →
   **live coord worktree exists for `dispatch/NNN`** via `git::live_worktree_for_ref`
   (D3/D9); (b) no chaining on absent stamp — record nothing + warn (D1). Stamp-present
   path byte-identical. **Reopen (completed→non-completed): evict the registry row +
   clear the stamp** (D8, P2-1).
2. **ISS-039 commit** at `prepare_review`: `commit_boundaries` splices the live coord
   worktree's working ledger onto `dispatch/NNN` (mirrors `commit_journal`), **before any
   read**. Validates (parse before commit, D7a/F3) and is **content-idempotent via
   tree-oid compare** (no ref advance on identical content, D7b/F1).
3. **Derive** from the **committed** ledger (`read_ledger`) — the SAME source
   `plan_phases` uses (INV-4) — `record_source_delta` each row (upsert) into the registry.
4. **Gates, BEFORE ref projection** (ordering load-bearing, F1):
   - **Projection-source guard (D11, Rev 4/6):** before projection, every registry row with
     `provenance ∈ {Funnel, Unknown}` must have a committed-ledger row ⇒ else halt naming
     the phases. Pure phase-id set comparison (no `code_delta_paths`). Catches total +
     partial ledger loss; excludes `Solo`/`Manual`; includes `Unknown` (loud on
     mid-upgrade active slices).
   - **Registry gate (D4):** `registry_completeness(primary, primary, slice)` ⇒ bail on
     gap. A halt here creates no refs (clean record-delta → re-run).
5. **Funnel inline double-write retained** (`run_record_boundary` unchanged: writes the
   working ledger + the registry) — contract (D5/F5); the derive is the authoritative
   reconciler over it.

`record-delta` stays the manual escape hatch. No new authored tier.

## Implementation surface (new/changed — for /plan)

- `git::live_worktree_for_ref` — **extend** `parse_worktree_for_ref` (git.rs:1163) to
  surface `{ path, branch, prunable }`; helper applies `!prunable && path.exists()`. Keep
  `worktree_for_ref` signature (existing callers). (D9/F2)
- `boundary.rs::Provenance` (NEW enum `Solo|Funnel|Manual|Unknown(default)`) + a
  `#[serde(default)] provenance` field on `BoundaryRow`. (D12) `boundary.rs` →
  **design-target**. Write-sites: solo binding→`Solo`; `run_record_boundary`→`Funnel`
  (both ledger + registry rows); derive→`Funnel`; `run_record_delta`→`Manual`.
- ~~`git::code_delta_paths`~~ — **DROPPED** (Rev 4). D11 is a phase-id set compare over
  `state::read_source_deltas` (state.rs:588, existing) + the committed `boundaries.rows`;
  no new git helper, no working-tree diff.
- `dispatch::commit_boundaries(root, parent, coord_ref, coord_worktree, slice)` — twin of
  `commit_journal` (dispatch.rs:2094); parse+validate, tree-oid idempotency, CAS. (D7)
- `ledger::read_boundaries_file(worktree_root, slice) -> Option<String>` — raw
  working-file reader (`dispatch_dir` is private, ledger.rs:375). (OQ-4)
- `state::forget_source_delta(cwd, slice, phase) -> bool` — inverse of
  `record_source_delta`; read-modify-write removal. (D8)
- `dispatch::prepare_review` (dispatch.rs:1497) — insert steps 1–4 above; projection
  (step 6) unchanged.
- `state::set_phase_status` (state.rs:386–401) — reopen eviction branch. (D8)
- Dispatch SKILL.md ×3 — document the prepare-review gate as the enforced beat; codex/pi
  keeps `record-delta`.

## Decision ledger (D1–D11; full rationale in design.md §7)

- **D1** drop chaining; absent stamp → record nothing, fail loud (conformance folds all
  paths, slice.rs:1919 — non-exact start = false `undeclared`).
- **D2** commit-then-derive-at-gate, authoritative upsert, from the committed ledger.
- **D3** sound guard = live coord worktree, not branch-proxy. **Kept** despite the
  authoritative derive: without it a session-root flip writes an empty-range row the
  presence-only gate blesses, and if the funnel also missed that phase the derive has no
  row to overwrite → gate passes with garbage. Guard → halt loudly.
- **D4** primary-rooted gate (completed-set + registry from one canonical tree).
- **D5** keep the funnel double-write (dropping = contract break, F5).
- **D6** defer codex/pi symmetric derive (IMP-171); ISS-039 absorption is claude-bounded.
- **D7** commit the ledger via prepare-review splice (mirror `commit_journal`); **(a)**
  validate before commit (F3), **(b)** content-idempotent tree-oid compare (F1).
- **D8** reopen evicts the row + clears the stamp (P2-1).
- **D9** liveness-verified probe — **extend** the parser (prunable), don't wrap (F2).
  **Limitation (F4):** liveness ≠ ownership — a coord worktree un-pruned through the
  pre-integrate audit window false-stands-down a post-drive solo phase; caught loudly by
  gate/conformance; precise dispatch-run ownership signal = hardening follow-up.
- **D10** no SPEC-022 REV — the spec already mandates the committed ledger
  (spec-022.md:180); ISS-039 is the impl in violation, so committing is conformance.
- **D11** projection-source guard (Rev 4 reshape, Rev 5 edges) — every registry row with
  `provenance ∈ {Funnel, Unknown}` must have a committed-ledger row, else halt naming the
  gaps (run BEFORE projection). Per-phase set compare; no `code_delta_paths`. Catches total
  + partial loss; excludes `Solo`/`Manual`; includes `Unknown` so mid-upgrade active slices
  halt loudly (pass-6 P6-2).
- **D12** `provenance` field on `BoundaryRow` (the D11 discriminator) — `Solo|Funnel|Manual`
  (absent→`Unknown`). The only sound option (no committed run-state records landing path —
  "derive from writer" is illusory). **Sticky merge in `record_source_delta` (pass-6/7):**
  keyed on incoming provenance — `Solo`/`Funnel` (landing writers) overwrite; `Manual`
  (`record-delta`) preserves existing incl. `Unknown`, atomic in the writer's RMW (NOT a
  caller pre-read — race-free, P7-2). Legacy `Unknown` halt clears by **reclassification**
  (re-record / hand-edit runtime row), never bare `record-delta` (P7-1). `#[serde(default)]`
  = whole back-compat story; no migration code. Does NOT close F4 (post-record marker ≠
  pre-record stand-down).

## Codex passes (full ledger in design.md §10)

- **Pass 1:** F1 root-mismatch→D4; F2 chaining→D1; F3 order→moot; F4 source-divergence (was
  "justified", now CLOSED by committed-ref, INV-4); F5 double-write→D5.
- **Pass 2:** P2-1 reopen stale row→D8; P2-2 liveness→D9; **P2-3 working-ledger read
  violates SPEC-022→committed-ref model** (User decision to absorb ISS-039).
- **Pass 3:** F1 re-run journal poison→content-idempotent + gate-before-projection; F2
  prunable→extend parser; F3 validate-before-commit; F4 liveness≠ownership (accepted);
  F5 R4 test-gap→no-pre-commit VT.
- **Pass 4 (confirmatory):** confirmed F1–F3/F5 sound. **Residual BLOCKER:** registry gate
  masks an absent committed ledger — `run_record_boundary` double-writes the registry
  (dispatch.rs:614) but only the working ledger file (:606), so coord-gone-before-prepare-
  review ⇒ committed ledger empty, projection 0, yet registry pre-filled ⇒ green gate,
  silent broken projection. **Fixed: D11.** MINOR: idempotency compare → tree-oid (D7b).

## Code map (verified seams)

- `src/state.rs`: `:386` `set_phase_status` reopen branch (D8); `:466`
  `capture_phase_boundary` (guard `:481` → D3/D9; absent-stamp `:524`; stamp-once `:503`);
  `:613` `record_source_delta` (upsert; F-6 guard `is_ancestor`+non-merge, `:618`); `:765`
  `registry_completeness` (two roots → D4); `:588` `read_source_deltas`; `:743`
  completed-set read.
- `src/dispatch.rs`: `:1497` `prepare_review` (insert commit→derive→guard→gate before
  projection); `:1522` reads `orthogonal`; `:1523` reads `boundaries`; `:587`
  `run_record_boundary` (double-write retained; Rev 4: stamps `provenance=Funnel` on both
  rows — `:606` working ledger, `:614` registry);
  `:1991` `read_ledger` (committed-ref; now derive + plan_phases source); `:2015`
  `plan_review` exclusion set; `:2041` `plan_phases` (iterates `boundaries.rows`); `:2094`
  `commit_journal` (mirror for `commit_boundaries`); `:2182` `with_journaled_projection`
  (the journal double-commit; recovery commit persists Failed rows — F1).
- `src/git.rs`: `:1163` `parse_worktree_for_ref` (EXTEND for prunable — D9); `:1189`
  `worktree_for_ref` (keep signature); `:554` `primary_worktree`; `:994` `current_branch`;
  `:1003` `is_ancestor`. (Rev 4: `code_delta_paths` NOT added — D11 needs no git helper.)
- `src/slice.rs`: `:1970` `run_record_delta` — Rev 5: provenance-preserving (read existing
  row, keep `Solo`/`Funnel`, else `Manual`). design-target.
- `src/boundary.rs`: `:16` `BoundaryRow` — Rev 4: add `provenance` (D12). design-target.
- `src/ledger.rs`: `:541` `record_boundary`; `:375` `dispatch_dir` (private — expose
  `read_boundaries_file`, OQ-4).
- `src/slice.rs`: `:1894`/`:1919` `conformance_outcome` (folds all paths, no `.doctrine/`
  strip — why chaining is unsound); `:1970` `run_record_delta` (escape hatch, unchanged).
- Tests: `tests/e2e_dispatch_lifecycle.rs:174` + `tests/e2e_dispatch_sync.rs:111` both
  **manually commit** the ledger (so they don't cover the new splice — F5);
  `e2e_dispatch_sync.rs:435` `refused_row_persists_failed_status_in_committed_journal`
  (proves the recovery commit persists Failed rows — F1); both assert `phase/064-01`.

## Constraints / canon

- **SPEC-022 §run-ledger sourcing (spec-022.md:180):** ledger (incl. `boundaries.toml`)
  tree-read from the dispatch tip, never the working FS, identical stage-1/stage-2. The
  constraint that forced the committed-ref model (P2-3) and grounds D10 (no REV).
- **POL-002:** doctrine-owned signals only (live coord worktree, recorded SHAs, the
  `dispatch/NNN` ref) — never host commit conventions.
- **ADR-001:** git/disk in the shell; pure cross-checks (`check_completeness`) in the leaf.
- **R-5 belt:** PHASE commits strip `.doctrine/` — the boundaries commit is a SEPARATE
  doctrine-mediated commit (like the journal), never a phase commit.
- **Behaviour-preservation:** `set_phase_status` + dispatch suites green; stamp-present
  path byte-identical; `worktree_for_ref` signature unchanged.
- `just check` green; clippy plain (no `--all-targets`); per commit.

## Open items for /plan (none are blockers)

- **OQ-6:** factor a shared `splice_ledger_file` for `commit_journal` + `commit_boundaries`
  (DRY) — decide at impl if it reads cleanly.
- **OQ-7 / R4:** committing the ledger re-enables claude `plan_phases` projection (0 in
  production today) — verify `e2e_dispatch_lifecycle` (`phase/064-01`) + `e2e_dispatch_sync`
  hold, and add the no-pre-commit-ledger fixture (F5).
- ~~`code_delta_paths` shape~~ — RESOLVED (Rev 4): helper dropped; D11 is a phase-id set
  compare, no diff.
- **Provenance write-site coverage:** confirm all four writers stamp correctly + the
  serde-default `Unknown` round-trips (VTs drafted §9).
- **F4 hardening:** a precise dispatch-run ownership signal (run-state, not worktree
  presence) — file as a backlog item (NOT closed by D12 provenance).

## Evidence / forensics (don't re-derive)

- Both registries on disk are already hand-bootstrapped (147 all 6 phases; 153 all 4) —
  the original failing state is gone; root-cause is from code.
- SL-153 phase→commit map (linear `c371b839`→P01→P04): P01 `d3947526`, P02 `ab2c642f`
  (dispatch drive started), P03 `71466d0d`, P04 `0cc4800c`. Ledger
  `.doctrine/dispatch/153/boundaries.toml` has only P03/P04 (funnel); not committed (ISS-039).

## PHASE-01 implemented (2026-06-26) — solo `/worktree` fork `sl-154-phase-01`

Data-model keystone landed (TDD red/green/refactor). `just check` green, clippy plain
zero warnings, 2579 bin tests pass.

- **`boundary.rs`** — `Provenance` enum `Solo|Funnel|Manual|Unknown` (`#[default] Unknown`,
  `#[serde(rename_all="snake_case")]`, `Copy`) + `#[serde(default)] provenance` on
  `BoundaryRow`. `#[serde(default)]` is the whole back-compat story (legacy row → `Unknown`).
  VT-1 in-module.
- **`state.rs`** — sticky merge in `record_source_delta` keyed on the INCOMING row, inside
  the existing RMW (atomic, F-6 guard byte-identical): `Solo`/`Funnel` overwrite, `Manual`/
  `Unknown` preserve existing. `forget_source_delta` (D8 reopen-evict sibling; dead-code-
  phased via `cfg_attr(not(test))` until its PHASE-03 caller). **Refactor (T5):** extracted
  `read_registry`/`write_registry` (mirror `ledger::load`/`store`); the `fs::write`/
  `create_dir_all` lint-expect now lives once on `write_registry`. VT-2/VT-3 at the state seam.
- **Construction sites (13)** stamped to FINAL values now (pre-satisfies later phases'
  *stamp-value* criteria; those phases still own BEHAVIOUR+VERIFY): `state.rs:438`→`Solo`,
  `dispatch.rs`→`Funnel` (one row cloned to ledger+registry), `slice.rs` record-delta→`Manual`;
  test sites: `state.rs row()`→`Unknown` (generic), `ledger.rs`→`Funnel`, `slice.rs` conf
  tests→`Manual`. EX-4 churn: 4 solo-binding capture tests now expect `Solo` (struct-update
  over `row()`) — behaviour identical, new field only.

**FINDING — solo capture + `worktree land` vs F-6 (for PHASE-03 / audit, NOT PHASE-01 scope):**
`capture_phase_boundary` reads HEAD/branch from the cwd worktree (`root::find`), but the
registry is primary-rooted. A solo phase landed via `doctrine worktree land` (`merge --no-ff`)
makes the trunk HEAD a **merge commit**; a post-land `completed` flip then captures a
merge-commit `code_end_oid`, which the F-6 non-merge guard rejects → capture degrades to a
named warning (non-blocking by design) with `slice record-delta` as the sanctioned remedy.
For a clean non-merge boundary the `completed` flip must capture the fork tip (a non-merge
commit) — i.e. flip from the same worktree whose HEAD is the code tip, before the no-ff land,
or record-delta the range after. Also observed: concurrent trunk advance (an unrelated SL-138
commit) landed on `edge` after the fork, so the in_progress `code_start_oid` stamped at edge
HEAD (`47910ba2`) ≠ the fork base (`071e9578`) — handle at completion (record-delta).

## PHASE-02 — live coordination-worktree probe (landed `8095d13a` on edge)

git.rs-only, file-disjoint. TDD red/green/refactor.
- **EX-1** `parse_worktree_for_ref` now returns `Option<WorktreeEntry { path, branch,
  prunable }>` via a `WorktreeBlock` accumulator that settles a block on the next
  `worktree` line / blank / EOF. The prior shape early-returned on the `branch` match;
  git emits `prunable` AFTER `branch`, so liveness was being dropped (the D9 watch-item).
  RED test `parse_worktree_for_ref_surfaces_trailing_prunable` pinned it (prunable=false
  under the old logic) → GREEN under block-accumulate. The 4 existing parse tests stayed
  green (now `.map(|e| e.path)`) — behaviour preserved.
- **EX-2** `live_worktree_for_ref(root, ref) -> Result<Option<WorktreeEntry>>` =
  `parse(...).filter(|e| !e.prunable && e.path.exists())`. Unused until PHASE-03 wires it
  into `capture_phase_boundary` → `cfg_attr(not(test), expect(dead_code, …))` (same pattern
  as `forget_source_delta`; `branch` field likewise).
- **EX-3** `worktree_for_ref` keeps `-> Option<PathBuf>` (`.map(|e| e.path)`); dispatch
  callers untouched. VT-3: git module 91 green, dispatch 61 green.
- **Land:** rebase fork onto edge + `merge --ff-only` (NOT `worktree land` — F-6 finding
  above). Edge was still at the in_progress `code_start` (`ee5a41a9`), so the boundary
  `[ee5a41a9 → 8095d13a]` is exact non-merge, provenance `solo` — no record-delta needed.
- **Foreign churn observed (left untouched):** edge base carried uncommitted
  `adr-012.toml`/`slice-138/156` etc.; `e2e_relation_migration_storage` REDs at base on
  `adr-012.toml`'s `supersedes` relation (relation-migration in flight) — out of SL-154
  scope, gated PHASE-02 on the git suite. `cargo fmt` also reformatted foreign
  `guard.rs`/`revision.rs` (SL-155 landed unformatted) — restored, not committed.

## PHASE-03 — solo binding soundness + reopen eviction (landed `e4dfd146` on edge)

state.rs (impl) + git.rs (dead-fn removal). TDD red/green/refactor. Two behaviour changes.
- **EX-1 guard swap** (`capture_phase_boundary`): branch-proxy → `live_worktree_for_ref(
  project_root, "refs/heads/dispatch/{slice_id:03}")`. `Ok(Some)`→stand down, `Ok(None)`→
  record, `Err`→`warn_capture`+stand down. Sound from the session root (the branch-proxy
  could not see a `dispatch/NNN` coord worktree when the flip ran from the primary tree on
  `main`); liveness (`!prunable && path.exists()`) means a stale/pruned coord entry reads as
  absent → capture is never suppressed forever. Removed the now-dead `git::current_branch`
  (the old guard was its sole caller; dispatch.rs has its own private `current_branch`,
  untouched). RED: rewrote `binding_skips_capture_in_a_dispatch_context` →
  `binding_stands_down_under_a_live_coord_worktree` (real `git worktree add -b dispatch/147`,
  flip from the repo root); added `binding_records_when_coord_worktree_is_prunable` (VT-2).
- **EX-3 reopen eviction** (`set_phase_status`): capture `was_completed` BEFORE the status
  insert overwrites it; on Completed→non-Completed clear `code_start_oid` (the next
  in_progress re-stamps a FRESH start) + `forget_source_delta` the registry row. RED:
  `binding_records_boundary_then_evicts_and_refreshes_on_reopen` asserts the row is empty
  mid-reopen and re-completion records `[restart→end2]`, not the old preserved `[start→end2]`
  (P2-1 reversed).
- **forget degrades, does NOT propagate (deliberate departure from §5.2 pseudocode).** The
  design shows `forget_source_delta(...)?`, but `?` makes a reopen *fail* in a non-repo / bare
  cwd (`boundaries_path` itself errors) — breaking D5 (the binding must never block a status
  transition; pinned by `binding_degrades_without_blocking_when_git_unavailable` and
  `set_phase_status_clears_completed_on_reopen`, both on non-git roots). So forget warns like
  the record tail and never blocks. Safe because a lingering row is **self-healing** (the
  re-completion's upsert overwrites it) and a never-recompleted reopen surfaces loudly via the
  completeness gate (`Extra`). Recorded as [[mem.pattern.state.reopen-evict-degrades-self-heal]].
- **EX-2/EX-4 verification-only** (Solo stamp + absent-warn landed P01; stamp-present path
  byte-identical). **VT-5** pins the untouched `registry_completeness` consumer end-to-end:
  `binding_absent_start_records_nothing_and_completeness_flags_incomplete`.
- **Gate:** bin unit 2584 green (state+git), dispatch lifecycle/sync + list-conformance
  30+4 green (funnel double-record invariant intact); clippy plain zero-warning; fmt.
- **Land:** rebase fork onto current edge + `merge --ff-only` (NOT `worktree land` — F-6).
  Edge advanced under the fork (`5c884cdb`→`748db566`, a foreign SL-156 `research.md` doc
  commit, disjoint), so the in_progress `code_start` (`5c884cdb`) was stale. Rebased onto
  `748db566` (clean), ff-landed `e4dfd146`, then `record-delta --start 748db566 --end
  e4dfd146` corrected the boundary to my true single-commit range (Manual incoming preserved
  the existing `solo` provenance via the sticky merge). gc oracle-certified landed → reaped.

## PHASE-04 — commit the boundaries ledger at prepare-review (landed `d82ec4b7` on edge)

- **What landed.** `ledger::read_boundaries_file` (EX-1, verbatim-bytes / None working-file
  reader over the private `dispatch_dir`); `dispatch::commit_boundaries` (EX-2, validate-
  before-commit → malformed `Err` leaves the tip; content-idempotent via a TREE-oid compare
  → identical content does not advance the ref; advances `dispatch/NNN` under CAS, `Moved`
  bails); `prepare_review` splice (EX-3 — `tip0` → `live_worktree_for_ref` guard →
  `commit_boundaries` → `tip`, with `trunk_base` recomputed off the post-splice tip).
  Dropped the `expect(dead_code)` on `Boundaries::parse`/`to_toml` (commit_boundaries is
  their first live caller). `run_record_delta` already constructs `Provenance::Manual`
  (PHASE-06's EX-1 is effectively pre-landed in code; PHASE-06 remains for the skill docs).
- **VT-2 ruling (/consult, user-approved — durable).** The literal "second prepare-review
  does not advance dispatch/NNN (same tip oid) and does not rewrite verified journal rows as
  Failed" is **unsatisfiable at PHASE-04**: a full re-run collides on the already-created
  `review/phase` refs (zero-oid CAS → `Moved` → Failed) and `with_journaled_projection`
  churns the journal, advancing the tip — pre-existing EX-5/VT-4 behaviour (design §882).
  The clean re-run needs the **PHASE-05** gate (halt-before-projection, F1). Verified instead
  at the **commit_boundaries grain**: exactly one `ledger: boundaries` commit after two runs +
  a stable committed blob. The "verified rows not Failed" clause is PHASE-05 territory.
  Recorded as [[mem.pattern.dispatch.prepare-review-rerun-not-idempotent-until-gate]].
- **D-OQ6 → keep separate.** `commit_journal` commits unconditionally; `commit_boundaries`
  computes the candidate tree *first* to decide idempotency, so it can't share a bundled
  `tree_with_file+commit` helper. The only common tail (`commit_tree → CAS → Moved-bail`) is
  an idiom already inline at dispatch.rs:972/1746/2110 with stage-distinct bail messages
  (RV-030 F-4). No helper — extraction adds indirection without riding a seam.
- **New e2e fixture (EX-4/VT-1/2/3).** `build_fixture_uncommitted_ledger` registers a REAL
  linked worktree on `dispatch/064` with an UNCOMMITTED `boundaries.toml` (so
  `live_worktree_for_ref` → Some and the splice actually fires — the existing pre-committed
  fixtures no-op it and prove EX-4 unchanged). VT-3 passes a malformed body via the same fixture.
- **Gate:** e2e_dispatch_sync 33, lifecycle 1, ledger unit 21, dispatch unit 33, candidate 23,
  h1 1, arm_spawn 3, shrinkage 3 — all green. clippy plain zero-warn. My files fmt-clean
  (foreign `src/commands/guard.rs` is pre-existing-unformatted SL-155/156 dirt — left untouched).
- **Land:** rebase fork onto current edge + `merge --ff-only` (F-6, not `worktree land`). Edge
  advanced twice under the fork (`507a64fb`→`3ee00a6f`→`19a3cb80`, foreign SL-156 plan/design),
  so the in_progress `code_start` (`3ee00a6f`) was stale. Rebased onto `19a3cb80` (clean),
  ff-landed `d82ec4b7`, then `record-delta --start 19a3cb80 --end d82ec4b7` corrected the
  boundary to my true single-commit range (Manual incoming preserved `solo`). Source committed
  separately from this notes/doc commit (knowledge trails code, design §4.3).

## PHASE-05 — derive + projection-source guard + completeness gate (landed `d9892674` on edge)

ISS-052 closed. At `prepare_review`, after the committed boundaries read and BEFORE the ref
projection, three beats in load-bearing order (a halt creates no refs → the operator's
record-delta → re-run collides with nothing, F1):

- **Guard (D11, EX-2).** New pure helper `missing_committed_funnel_phases(registry, committed)`
  (dispatch.rs): registry rows with provenance ∈ {Funnel, Unknown} whose phase ∉ the committed
  ledger → `bail!` naming them. Solo/Manual excluded. Phase-id set compare, never a code-delta
  diff. Reads the PRIMARY registry **pre-derive** (the derive can't mask the loss).
- **Derive (EX-3).** `record_source_delta` each committed-ledger row (Funnel) into the primary
  registry — upsert, so it fills a lost row and overwrites a binding mis-capture.
- **Gate (EX-4).** `registry_completeness(&primary, &primary, slice)` → `bail!` on any gap.

**EX-1 was already in code** — `run_record_boundary:606` stamps `Funnel` on the one row cloned
to both the committed ledger and the registry. PHASE-05 only *pins* it (strengthened
`record_boundary_also_writes_the_arm_neutral_registry` with a `provenance = "funnel"` assert on
both files). Like PHASE-04's relation to PHASE-06 EX-1, the production write predates its phase.

**KEY EMERGENT FINDING — the gate couples prepare-review to phase-completion status.**
`registry_completeness` reads `completed_phase_ids(primary)` (phase-NN.toml `status==completed`).
After the derive the registry mirrors the committed ledger, so a committed phase that is **not**
marked `completed` reads as an `Extra` gap → halt. This is **design-intended** (§5.2 lines 119/159:
prepare-review is the *pre-audit conclude beat*, post-completion), but it broke **27 existing
projection fixtures** that seeded a boundaries ledger yet never marked phases completed. The
behaviour-preservation clause (design line 154) names specific shared seams (`set_phase_status`
solo path, `worktree_for_ref` callers), **not** these dispatch projection tests — so the fixture
upgrade is in-scope, not a violation. Fix was one shared helper `seed_completed_phases` +
**gitignoring the runtime tier** in the fixture repos (`.doctrine/state/`): the derive now writes
the registry under runtime state, which otherwise shows untracked and dirties the
`integrate`-tests' clean-status assertions. Recorded as a memory for future dispatch-test authors.

- **Tests.** Guard predicate unit (VT-1/2/4) in dispatch::tests; e2e in e2e_dispatch_sync.rs via
  new helpers (`build_guard_repo`, `commit_ledger_on_dispatch`, `seed_registry`, `boundary_row`,
  `record_delta`, `prepare_review_from`): VT-1 total-loss-no-refs, VT-3 no-false-halt
  (Solo + empty-code), VT-5 derive-authoritative (garbage overwrite), VT-6 primary-rooted from a
  coord cwd, VT-7 gate-before-projection + clean re-run after record-delta.
- **HashSet is clippy-disallowed** (determinism) — used `BTreeSet<&str>` for the committed set.
- **Gate:** e2e_dispatch_sync 38, dispatch unit 37, state 46, ledger 21, record_delta 4,
  list_conformance 4, shrinkage 3 — all green; clippy plain zero-warn; my files fmt-clean
  (`cargo fmt` reflow of this slice's own pre-existing-unformatted `ledger.rs` PHASE-04 test was
  restored, not committed — out of PHASE-05 scope).
- **Land:** rebase onto current edge + `merge --ff-only` (F-6, not `worktree land`). Edge advanced
  `176d46eb`→`0c53d483` under the fork (foreign SL-156). Rebased clean (SL-156 untouched these
  files), ff-landed `d9892674`, then `record-delta --start 0c53d483 --end d9892674` corrected the
  stale in_progress `code_start` (`176d46eb`) to my true single-commit range (Manual preserved `solo`).

## Relations & selectors

- references→RFC-004 (concerns); related→ISS-039, ISS-051, ISS-052. Follow-ups: IMP-171
  (codex/pi symmetry), F4 ownership-signal hardening (to file).
- design-target: `src/state.rs`, `src/dispatch.rs`, `src/ledger.rs`, `src/git.rs`,
  `src/boundary.rs` (Rev 4: `provenance` field, D12), `src/slice.rs` (Rev 5: record-delta
  provenance-preserving, P6-1), `plugins/doctrine/skills/dispatch{,-agent,-subprocess}/SKILL.md`.
