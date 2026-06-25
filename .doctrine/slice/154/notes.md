# Notes SL-154: Reliable conformance-registry capture

Durable per-slice scratchpad — tracked in git. Lift anything from a disposable phase
sheet that must survive `rm -rf` before close-out.

## Status (2026-06-26) — HANDOFF POINT

Slice in `design`. `design.md` went through internal + TWO codex (GPT-5.5) adversarial
passes + a deep design conversation. **Codex pass 2 found a governance BLOCKER (P2-3):
the planned working-ledger read violates SPEC-022.** User DECIDED to **absorb ISS-039**
(fork option 1) to fix it cleanly.

**The design.md §2/§5 body still describes the RETRACTED working-file read.** §10
(pass-2 + the decision) SUPERSEDES the body. The next agent's first job is to **revise
the design body to the committed-ref (ISS-039-absorbed) model**, then re-pass / plan.

Read in order: `slice-154.md` (scope) → `design.md` §10 FIRST (latest truth) → §1–§9
body (older, pending revision) → this → `handover.md`.

## What this slice is (one paragraph)

Close two conformance-registry **population** leaks RFC-004 v0.1 (SL-147) left, and
make recording robust across landing-path transitions. The registry is
`.doctrine/state/slice/NNN/boundaries.toml` (runtime) — one `[[boundary]]` row per
landed phase — the **actual-side input** to `slice conformance`. ISS-051 = solo path
final-phase miss; ISS-052 = funnel never reliably populates. Registry-population only;
references **RFC-004** (not RFC-005). Stand-alone.

## The locked design (what `design.md` now says)

Four moving parts; all flow into one registry (`record_source_delta`, upsert by phase),
guarded by one gate.

1. **Solo binding (`state.rs::capture_phase_boundary`).** Keep the stamp; record
   `(stamp, HEAD)` at the `completed` flip. **Two changes only:** (a) the arm-guard
   predicate flips from branch-proxy (`current_branch == dispatch/NNN`) to **"a live
   coord worktree exists for `dispatch/NNN`"** (`git::worktree_for_ref`); (b) no
   chaining when the stamp is absent — record nothing, surfaced warning, gate/conformance
   nets it. Stamp-present path byte-identical.
2. **Derive-at-gate (`dispatch.rs::run_prepare_review`), authoritative + self-correcting.**
   Read the **live coord worktree WORKING ledger** (NOT the committed ref — empty under
   ISS-039) and `record_source_delta` each row (**upsert**). This both fills missing
   funnel rows and **overwrites** any garbage the binding mis-captured.
3. **Gate (both arms).** `registry_completeness(primary, primary, slice)` — **primary-
   rooted** — `bail!` on any gap. THE enforcement ("can't reach audit incomplete").
4. **Funnel inline double-write retained** (`run_record_boundary` unchanged) — no
   contract break; the derive is a redundant-but-authoritative reconciler over it.

`record-delta` stays the manual escape hatch. No new authored tier.

## Why it ended up here — the decision trail (don't relitigate without cause)

- **A′ over pure-chain (B):** the `in_progress` stamp is the *precision* mechanism (it
  excludes inter-phase knowledge/notes commits); pure-chain folds them in. We kept the
  stamp.
- **Then codex F2 killed chaining entirely:** `conformance_outcome` diffs full
  `start..end` with **no `.doctrine/` strip** (slice.rs:1919–1928), so ANY non-exact
  start manufactures false `undeclared` edits. A *wrong* row is worse than a *missing*
  one. So the lost-stamp fallback is **fail-loud**, not auto-heal. (This narrowed the
  ISS-051 deliverable honestly: prevent the miss + make it loud; do not reconstruct.)
- **User pushed for robustness + auto-heal + solo↔dispatch transitions.** That surfaced
  the real defect (below) and the sound auto-heal: **derive-at-gate-with-upsert** (sound
  retroactively because the ledger persists; also corrects mis-captures).
- **The unsound-capture finding (the heart of obj 3):** phase status flips are authored
  writes run **from the session root** (dispatch skill:20), where `HEAD` is `edge` — not
  a dispatched phase's tip on `dispatch/NNN`. The branch-proxy guard keys on
  `project_root`'s branch, so from the session root it sees `edge`, **doesn't fire**, and
  the binding would capture a dispatched phase against the wrong tree → garbage. Fix =
  sound coord-worktree guard (D3) + derive-upsert self-correction (D2).

## codex pass 1 — findings + dispositions (all integrated)

- **F1 BLOCKER** gate root-mismatch → primary-rooted gate (D4).
- **F2 BLOCKER** chain pollutes range → drop chaining, fail loud (D1).
- **F3 MAJOR** `read_source_deltas` order-unstable → moot under D1; gate is set-based.
- **F4 MAJOR** derive vs `plan_phases` source divergence → justified (derive stage-1-only
  working ledger; `plan_phases` needs committed ref for stage-2); root fix is ISS-039,
  out of scope. Documented.
- **F5 MAJOR** dropping the registry half is a contract break (pinned test
  `e2e_dispatch_sync.rs:1132` + skill docs) → keep the double-write (D5).

## CODEX PASS 2 — outcome (durable; full record in design.md §10)

Pass-1 (F1/F2/F3/F5) confirmed resolved. Three new findings:

- **P2-1 BLOCKER — reopen leaves a stale row the gate blesses.** Reopen clears
  `completed` but NOT `code_start_oid` (state.rs:386–400), stamp kept on re-entry
  (:503), gate checks presence not freshness. **Fix (in scope, solo-side):** on reopen
  (completed→non-completed), EVICT the phase's registry row + clear its stamp.
- **P2-2 MAJOR — `worktree_for_ref` ignores `prunable`/path-liveness** (git.rs:1163), so
  a lingering coord entry suppresses solo capture forever (POL-002 footgun). **Fix:**
  liveness-verified probe OR a doctrine-owned "dispatch-active" runtime marker.
- **P2-3 BLOCKER (governance) — working-ledger read violates SPEC-022** (ledgers are
  tree-read from the `dispatch/<N>` tip, NEVER the working FS; spec-022.md:180,
  spec-022.toml responsibility). **RETRACTED.** ISS-052's clean fix is blocked on a
  committed boundaries ledger = ISS-039.

## DECISION (User) — absorb ISS-039; the committed-ref model

Pull **ISS-039** into this slice: commit `boundaries.toml` to `dispatch/NNN` alongside
`journal.toml`. Then derive + `plan_phases` both read the **committed ref** (SPEC-022-
legal), F4 divergence gone, claude phase-cuts (now 0 from the bug) restored. Bounded to
the claude arm; does NOT give codex/pi a ledger (that stays IMP-171). This SUPERSEDES
the working-file-read approach in the design body.

## NEXT AGENT — the work to land (revise design, then re-pass/plan)

1. **Absorb ISS-039 into scope.** Update `slice-154.md` (Scope/Objectives: add "commit
   the boundaries ledger to dispatch/NNN"; move ISS-039 out of Non-Goals; relate the
   slice to ISS-039). Confirm with the storage rule — relations via `doctrine link`,
   not hand-edits.
2. **Revise design.md body (§2/§5/§7) to the committed-ref model:**
   - ISS-039 fix: where/how `boundaries.toml` is committed to `dispatch/NNN`. Mirror
     `journal.toml`'s path — `commit_journal` (dispatch.rs:2094) splices `journal.toml`
     into the tip tree at prepare-review; do the same for `boundaries.toml` (or commit
     at the funnel Record beat). Decide the seam: prepare-review splice (simplest, one
     place) vs per-phase commit during the drive. SPEC-022 says identical stage-1/stage-2
     object-db read — so the ledger must be on the ref BEFORE prepare-review reads it.
   - Derive: read the **committed ref** via `read_ledger` (dispatch.rs:1991) — the same
     call `plan_phases` uses (:1523) — NOT the working file. Retract `worktree_for_ref`
     as the derive source (it stays only for the guard, P2-2 permitting).
   - Check whether SPEC-022 needs a REV at all once boundaries is committed — likely NO
     (the read becomes spec-compliant by construction). Confirm; if any spec text names
     boundaries as uncommitted, route a REV.
3. **Integrate P2-1 (reopen eviction) + P2-2 (liveness probe/marker)** into the design.
4. **Re-run a 3rd codex pass** on the committed-ref revision (the ISS-039 commit seam +
   P2-1/P2-2 fixes), then `/inquisition` or `/plan`.

Open question for the ISS-039 seam: does committing `boundaries.toml` at prepare-review
interact with the R-5 belt (which strips `.doctrine/` from PHASE commits)? The journal
commit is a SEPARATE doctrine-mediated commit, not a phase commit — follow that pattern.
Verify `plan_phases` reading the now-populated committed ledger doesn't break
`e2e_dispatch_lifecycle` (it expects `phase/064-01`; phase-cuts will now actually fire).

## Evidence / forensics (don't re-derive)

- Both registries on disk are **already hand-bootstrapped** (147 all 6 phases; 153 all 4)
  — the original failing state is GONE; root-cause is from code, not live disk.
- SL-153 phase→commit map (linear `c371b839`→P01→P02→P03→P04): P01 `d3947526`, P02
  `ab2c642f` (dispatch drive started here), P03 `71466d0d`, P04 `0cc4800c`. Ledger
  `.doctrine/dispatch/153/boundaries.toml` has only P03/P04 (funnel); not committed
  (ISS-039).

## Code map (seams)

- `src/state.rs:466` `capture_phase_boundary` (guard at :481 — the D3 change); `:613`
  `record_source_delta` (the one engine writer, upsert); `:654` `check_completeness`,
  `:765` `registry_completeness` (two roots — F1), `:743` `completed_phase_ids`.
- `src/dispatch.rs:1497` `run_prepare_review` (derive + gate go here, after phase
  planning ~:1536); `:587` `run_record_boundary` (UNCHANGED — double-write); `:1991`
  `read_ledger` (committed-ref reader — do NOT use for derive); `:2041` `plan_phases`.
- `src/ledger.rs:541` `record_boundary`, `:375` `dispatch_dir` (private — OQ-4: expose a
  worktree reader).
- `src/git.rs:1189` `worktree_for_ref` (locator for derive + guard), `:554`
  `primary_worktree`, `:994` `current_branch`, `:1003` `is_ancestor`.
- `src/slice.rs:1894` `conformance_outcome` (folds all paths, no `.doctrine/` strip —
  the reason chaining is unsound), `:1970` `run_record_delta` (escape hatch, unchanged).
- `plugins/doctrine/skills/dispatch{,-agent,-subprocess}/SKILL.md` — step-8 wording;
  document the prepare-review gate as the enforcement; codex/pi keeps `record-delta`.

## Constraints / canon

- **POL-002:** keys on doctrine-owned signals (live coord worktree, recorded SHAs, the
  `dispatch/NNN` ref) — never host commit conventions.
- **ADR-001:** git/disk in the shell; pure cross-checks (`check_completeness`) in the
  leaf.
- **ISS-039 out** (RFC-005 H3): derive reads the working file, never the committed ref;
  this slice neither depends on nor fixes it.
- **IMP-171:** codex/pi symmetric ledger+derive (couples to phase-ref projection) —
  deferred follow-up.
- `just check` green; clippy plain (no `--all-targets`).
- `record-delta` STAYS (removes the *need* on a normal slice, not the verb).

## design-target selectors (recorded)

`src/state.rs`, `src/dispatch.rs`, `plugins/doctrine/skills/dispatch/SKILL.md`,
`dispatch-agent/SKILL.md`, `dispatch-subprocess/SKILL.md`. (`ledger.rs`, `boundary.rs`
remain `scope-relevant`.)

## Relations

references→RFC-004 (concerns); related→ISS-051, ISS-052. Follow-ups: IMP-171
(codex/pi symmetry), ISS-039 (ledger commit, RFC-005 H3). Non-goals unchanged.
