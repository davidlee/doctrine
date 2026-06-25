# Notes SL-154: Reliable conformance-registry capture

Durable per-slice scratchpad — tracked in git. Lift anything from a disposable phase
sheet that must survive `rm -rf` before close-out.

## Status (2026-06-26) — COMMITTED-REF REVISION LANDED

Slice in `design`. `design.md` body (§1–§9) is **now revised to the committed-ref
(ISS-039-absorbed) model** — the working-file-read draft is fully retracted; the body
is the single source of truth. §10 carries the full review ledger + Revision-3 note.
Scope (`slice-154.md`) absorbs ISS-039 (objective 5, surface, non-goals, follow-ups);
SL-154 linked `related` ISS-039/051/052; selectors promoted (ledger.rs, git.rs →
design-target).

**Remaining:** a **3rd codex pass** on this revision (ISS-039 commit seam + P2-1/P2-2 +
committed-ref derive), then `/inquisition` or `/plan`. No code yet.

Read in order: `slice-154.md` (scope) → `design.md` §1–§9 (now current) → §10 (review
ledger) → this.

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

## DONE this session — committed-ref revision (commit pending)

1. ✅ Absorbed ISS-039 into scope (`slice-154.md` obj 5 + surface + non-goals +
   follow-ups); `doctrine link SL-154 related ISS-039` (051/052 already linked).
2. ✅ Revised `design.md` §1–§9 to the committed-ref model:
   - **D7** ISS-039 fix: `prepare_review` splices the working `boundaries.toml` onto
     `dispatch/NNN` via a new `commit_boundaries` (mirrors `commit_journal`,
     dispatch.rs:2094) — one place, before any read. Chosen over per-phase commit.
   - **D2/INV-4** derive reads the **committed** ledger via `read_ledger` — same source
     `plan_phases` uses; F4 divergence eliminated. New `ledger::read_boundaries_file`
     (working-file reader for the splice; OQ-4).
   - **D10** no SPEC-022 REV — the spec already mandates the committed boundaries ledger
     (spec-022.md:180); ISS-039 is the impl in violation, so the commit is conformance.
   - **D8** P2-1 reopen eviction: new `state::forget_source_delta` + clear stamp on
     completed→non-completed.
   - **D9** P2-2 liveness probe: new `git::live_worktree_for_ref` (reject prunable, stat
     path); used by the guard + the commit-boundaries locator; shared callers untouched.
   - **D3-kept rationale (load-bearing):** the guard stays even with the authoritative
     derive — without it a dispatched phase flipped from session root writes an
     empty-range row the presence-only gate blesses, and if the funnel also missed it the
     derive has no row to overwrite → gate passes with garbage. Guard → halt loudly.
3. ✅ Selectors: `ledger.rs`, `git.rs` promoted to design-target (+ notes).

## codex PASS 3 — done; all 5 integrated (design §10)

All verified against source, all ACCEPTED; none broke the committed-ref approach.
- **F1 BLOCKER** re-run journal poison → `commit_boundaries` content-idempotent + derive/gate
  BEFORE projection (a halt creates no refs). D7b + ordering.
- **F2 MAJOR** prunable unreachable by wrapping → D9 **extends** `parse_worktree_for_ref`
  to surface `{path,branch,prunable}`, not a wrapper.
- **F3 MAJOR** raw bytes committed unvalidated → parse+validate before commit (D7a).
- **F4 MAJOR** liveness≠ownership (audit-window false stand-down) → accepted w/ mitigation;
  precise dispatch-run ownership signal = hardening follow-up (file at /plan or close).
- **F5 MINOR** R4 stale — existing e2e pre-commit the ledger → add a no-pre-commit VT.

## NEXT — inquisition or plan

Design is now 3-passes-clean on the committed-ref model. Next: `/inquisition` (formal
hostile pass) OR `slice status 154 plan` → `/plan`.

Still-open at design (carry to /plan, not blockers): OQ-6 (factor shared
`splice_ledger_file`?), OQ-7 (projection re-enable — a verify task), F4 ownership-signal
hardening follow-up (file as backlog).

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
  `read_ledger` (committed-ref reader — NOW the derive source too); `:2094`
  `commit_journal` (mirror for `commit_boundaries`, D7); `:2041` `plan_phases`.
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
- **ISS-039 ABSORBED** (D7): commit `boundaries.toml` to `dispatch/NNN` (spec-022.md:180
  already mandates it); derive + `plan_phases` read the committed ref via `read_ledger`.
- **SPEC-022 §run-ledger sourcing:** ledger tree-read from the dispatch tip, never the
  working FS, identical stage-1/stage-2. The constraint that forced the committed-ref
  model (codex P2-3).
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
