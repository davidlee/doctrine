# Notes SL-154: Reliable conformance-registry capture

Durable per-slice scratchpad — tracked in git. Bootstrap for the next agent (plan +
codex). `design.md` is the authority; this is the fast-ingest map.

## Status (2026-06-26) — NOT plan-ready: pass-5 reopened D11 + efficiency findings open

Slice in `design`. `design.md` (§1–§9 body + §10 review ledger) is the committed-ref
(ISS-039-absorbed) model — committed-ref/ISS-039/P2-1/P2-2 are solid across 5 passes. BUT
**pass-5 proved D11 (the projection-source guard) unsound**, and the User requested an
efficiency/workflow review that surfaced a converging design refinement (provenance). Both
finding-sets are in **⚠ OPEN FINDINGS** below — NOT yet integrated into design.md. Scope
(`slice-154.md`) absorbs ISS-039. **No code yet.**

**Next:** a fresh agent integrates both finding-sets (see "Integration plan"), re-passes,
THEN `/plan`. Do NOT /plan with D11 as drafted.

Read order: `slice-154.md` (scope) → `design.md` §1–§9 → §10 (full pass ledger) → this.

## ⚠ OPEN FINDINGS — NEXT AGENT MUST INTEGRATE (not yet in design.md)

Two finding-sets landed AFTER the §10 pass-4 record and are **not yet integrated** into
the design body. D11 as written (§5.2 step 3 / §7 D11) is **unsound** — do not /plan until
reshaped. Integrate both, then re-pass.

### A. codex pass-5 — D11 is unsound (reshape required)

- **BLOCKER — empty-only predicate too weak.** D11 fires only on
  `boundaries.rows.is_empty()`. A **partial** committed ledger (some phases lost) slips
  past D11 AND the registry gate (pre-filled by the double-write, dispatch.rs:614) →
  `plan_phases` projects an incomplete chain silently. Need: guard the projected phase set
  against the *expected* (completed, funnel-owned) set, not mere non-emptiness.
- **BLOCKER — false-halts mixed solo→dispatch.** "prepare-review only runs on dispatched
  slices, so landed code without a ledger row is anomalous" is too broad. Solo phases land
  code on the dispatch branch with **no** committed-ledger row (by design). If dispatch
  phases are empty-code/doc-only, `boundaries` is legitimately empty while
  `code_delta_paths` is non-empty → D11 wrongly halts (walk the SL-153 shape). Need an
  **ownership-aware** condition; don't equate "code on the branch" with "must have a
  dispatch row".
- **MAJOR — exclusion-set mismatch.** `plan_phases` filters only `.doctrine`
  (dispatch.rs:2058); `plan_review` also strips verified-orthogonal (dispatch.rs:2015–2020).
  D11 copies `plan_review`'s set → guards the wrong projection. Define the protected
  invariant precisely (compare projected phase-source coverage to the code delta it must
  cover) rather than duplicating `plan_review`'s filter.
- ~~"D11 absent in checkout"~~ — **FALSE finding, discard.** Design stage; no code yet.

### B. Efficiency / workflow lens (User-requested: ledger as agent file-finder)

Review axis = value/ergonomics + token efficiency, not correctness.

- **★ Provenance convergence (headline).** Neither ledger records the per-phase **landing
  path** (solo vs funnel). That missing bit is *exactly* what **D11** and **F4** need to be
  sound (distinguish "solo code, no ledger row = fine" from "funnel phase, ledger row lost
  = broken"), AND a navigation win (route an agent: funnel phase → `dispatch/NNN` /
  `review/NNN-NN`; solo → `edge`). **One per-phase provenance field → D11 soundness + F4
  ownership signal + agent nav.** Strongly consider reshaping D11/F4 around provenance
  instead of two patches. (Likely the right fix for the pass-5 blockers above.)
- **Nav-coherence is a real SL-154 value (claim it).** Committed-ref derive-upsert makes
  the registry and dispatch ledger consistent (pre-slice they diverged) → one authoritative
  populated registry = one place an agent looks. Frame as ergonomic value, not just
  correctness.
- **Storage rule holds; defer a derived nav view.** Registry stores OIDs (source), so nav
  consumers re-diff. Fine for one consumer; a derived per-phase file-set view (gitignored
  cache or `slice show --phase-files`) is a **follow-up feature gated on SL-154's reliable
  population** — backlog it, don't scope-creep.
- **Halt-message ergonomics (minor).** Name the (bounded) changed paths in the D11/gate
  bail, not just a count — saves the agent a re-diff.

### Integration plan for the next agent
1. Reshape D11 around **per-phase landing-path provenance** (resolves both pass-5 blockers
   + the exclusion-set mismatch + F4 ownership). Decide: provenance field on the boundary
   row (schema touch — `boundary.rs`, scope-relevant → may become design-target) vs derive
   provenance from which writer recorded (cheaper, less durable).
2. Integrate the efficiency findings: add a design §  on the registry's nav/value role;
   claim nav-coherence; backlog the derived nav view + the F4 hardening.
3. Re-pass (codex) on the provenance reshape, then /plan.

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
   - **Projection-source guard (D11):** empty committed ledger + non-orthogonal code
     delta over `trunk_base` ⇒ halt (catches the coord-gone-early hole the registry gate
     can't, see pass-4 below).
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
- `git::code_delta_paths(root, trunk_base, tip, &orthogonal)` — filtered diff
  (non-`.doctrine/`, non-verified-orthogonal; same exclusion set as `plan_review`
  dispatch.rs:2015–2020). (D11)
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
- **D11** projection-source guard — empty committed ledger + landed code ⇒ halt (pass-4
  BLOCKER fix; the registry gate alone can't see broken projection).

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
  `run_record_boundary` (UNCHANGED double-write — `:606` working ledger, `:614` registry);
  `:1991` `read_ledger` (committed-ref; now derive + plan_phases source); `:2015`
  `plan_review` exclusion set; `:2041` `plan_phases` (iterates `boundaries.rows`); `:2094`
  `commit_journal` (mirror for `commit_boundaries`); `:2182` `with_journaled_projection`
  (the journal double-commit; recovery commit persists Failed rows — F1).
- `src/git.rs`: `:1163` `parse_worktree_for_ref` (EXTEND for prunable — D9); `:1189`
  `worktree_for_ref` (keep signature); `:554` `primary_worktree`; `:994` `current_branch`;
  `:1003` `is_ancestor`. `code_delta_paths` to be added (D11).
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
- **`code_delta_paths` shape (D11):** filtered diff vs comparing the `plan_review` review
  tree to the trunk-base tree — pick the leaner read.
- **F4 hardening:** a precise dispatch-run ownership signal (run-state, not worktree
  presence) — file as a backlog item.

## Evidence / forensics (don't re-derive)

- Both registries on disk are already hand-bootstrapped (147 all 6 phases; 153 all 4) —
  the original failing state is gone; root-cause is from code.
- SL-153 phase→commit map (linear `c371b839`→P01→P04): P01 `d3947526`, P02 `ab2c642f`
  (dispatch drive started), P03 `71466d0d`, P04 `0cc4800c`. Ledger
  `.doctrine/dispatch/153/boundaries.toml` has only P03/P04 (funnel); not committed (ISS-039).

## Relations & selectors

- references→RFC-004 (concerns); related→ISS-039, ISS-051, ISS-052. Follow-ups: IMP-171
  (codex/pi symmetry), F4 ownership-signal hardening (to file).
- design-target: `src/state.rs`, `src/dispatch.rs`, `src/ledger.rs`, `src/git.rs`,
  `plugins/doctrine/skills/dispatch{,-agent,-subprocess}/SKILL.md`. scope-relevant:
  `src/boundary.rs`.
