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

## Relations & selectors

- references→RFC-004 (concerns); related→ISS-039, ISS-051, ISS-052. Follow-ups: IMP-171
  (codex/pi symmetry), F4 ownership-signal hardening (to file).
- design-target: `src/state.rs`, `src/dispatch.rs`, `src/ledger.rs`, `src/git.rs`,
  `src/boundary.rs` (Rev 4: `provenance` field, D12), `src/slice.rs` (Rev 5: record-delta
  provenance-preserving, P6-1), `plugins/doctrine/skills/dispatch{,-agent,-subprocess}/SKILL.md`.
