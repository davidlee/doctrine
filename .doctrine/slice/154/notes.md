# Notes SL-154: Reliable conformance-registry capture

Durable per-slice scratchpad — tracked in git. Lift anything from a disposable phase
sheet that must survive `rm -rf` before close-out.

## Status (2026-06-26)

Slice in `design`. `design.md` **written and revised** through one internal + one
external (codex GPT-5.5) adversarial pass + a deep design conversation that reshaped
the approach. **Next step: a SECOND codex pass on the revised design** (the guard
change D3 + the unsound-capture model especially), then `/inquisition` or `/plan`.
Design decisions are LOCKED with the User unless the second codex pass overturns one.

Read in order: `slice-154.md` (scope) → `design.md` (full, current) → this.

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

## FOR THE SECOND CODEX PASS — aim the hostility here

- **D3 guard soundness.** Is "a live coord worktree exists for `dispatch/NNN`" the right
  predicate? Failure modes: coord worktree exists but the phase is genuinely solo
  (→ binding stands down → gate must catch; §5.4 crack); probe cost/error per flip (R5);
  does `worktree_for_ref` reliably resolve from the session root AND a coord tree?
- **D2 derive ordering vs the gate.** Derive then gate, same `run_prepare_review`. Any
  path where the derive runs but the working ledger is already gone, or the gate reads a
  different tree than the derive wrote? (Both must be `primary`.)
- **Self-correction completeness.** Binding mis-captures a funnel phase **and** the ledger
  has no row for it (inline write also failed) → derive can't overwrite → wrong row
  survives the gate (completeness passes on a wrong row). Is that reachable? (Gate checks
  *presence*, not *range correctness*.)
- **Mixed-transition matrix (§5.4).** Dispatch→solo and interleaved especially — the gate
  runs at dispatch conclude; solo phases after it rely on conformance only.
- **Irreducible manual case** — agree it's physically unrecoverable, or is there a sound
  retroactive source we missed?

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
