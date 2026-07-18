# Notes SL-222: Ledgered estimate claims

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Drive record (dispatch, pi subprocess arm; all 9 phases)

Funnel per phase: prove at base → regression capture → confined worker →
path-limited fork commit → dispatch_import → reset --hard → prove →
full-suite regression diff → record-delta → completed → reap. Import commits:
p01 901027eff (evidence-only phase), p02 bc07a43ea, p03 8be538ab,
p04 d8899c509, p06 d8345ad50, p07 a7e47f45b, p08 797ff41f1, p09 7f94adff2.
REV-026 authored 4c55eb58 / approved d4c7177b (operator) / applied 9ab197428.

## Evidence chain (R1)

Four live 246-row snapshots in `.doctrine/slice/222/`: phase1-preflip,
phase6-postflip, phase8-postmigration, phase9-final. Every pairwise diff and
the whole-slice diff (phase1 → phase9) is rank-neutral: zero rank or score
deltas, zero membership changes. Live migration (commit 464bf687f): 185
facets, census 185 = 185 imported + 0 + 0, --check proven tree-hash-identical
pre-execution, strip verified per-file, idempotent re-scan finds 0.

## Orchestrator interventions (audit should re-check these seams)

- PHASE-06: e2e anchor-conflict fixture direction + assertion text (426d6898).
- PHASE-09 laundered stubs — worker sweeps stubbed claim-era re-sources to
  constants and deleted/re-bucketed the tests that would have caught them:
  1. `has_nonterminal_interval` → `false` (β-sweep family dead) — re-sourced
     over `estimate_claims.anchored` interval rows + terminal exclusion.
  2. `max_authored_upper` → `None` (anchor decompose always empty-corpus) —
     re-sourced from `Pipeline.bare_anchor − margin`.
  3. `item_costing` `bare_estimate` → hardcoded `true` (every elicit
     participant masked; sizing-debt overcount) — re-keyed to the Pin/Human
     `anchored` claim map (a2775657a).
- Facet-era e2e fixtures hidden from workers by the DOCTRINE_WORKER filter,
  converted to hand-authored anchor rows by the orchestrator (a2775657a):
  e2e_priority_golden (composition + capped tensions), e2e_compare_elicit
  (stall corpus + frontier pairs). New helpers: capture_value_anchor /
  capture_cost_anchor.
- PHASE-03 VT-2 (E4 linearity battery) was never authored by any worker;
  written at conclude (a0679b25). PHASE-08/09 record-delta ranges initially
  excluded their import commits (`--start` is exclusive); re-recorded.

## Worker-session ledger

p02 1, p03 1, p04 2 (skipped mandated battery), p06 4, p07 2 (deferred
show-line), p08 1 (clean), p09 3 (mechanical-churn stop; laundered stubs).
Pattern for RFC-011: wide-but-shallow sweeps and "deferred" items are where
worker self-reports diverge from ground truth; verify cold and cross-check
against the design.

## Deferred / known residue

- `render.rs` `CostUnmigratedFacet => (0.0, 0.0)` / `ValueUnmigratedFacet =>
  (0.0, MARKER)` arms are unreachable-by-construction (the parse-free
  tripwire mints no magnitude) — render-shape audit may want them collapsed.
- `estimate.rs`/`value.rs` retain `deserialize_lenient` for serde round-trip
  on doc structs; NF-001 allowlist entries refreshed but a minimality pass
  was not enforced.
- spec-014 PRD prose still mentions `[estimate]` tables (prose, not parse).

## Reconcile + close session (2026-07-18)

Dispatched-slice landing on `main`. Ritual: promote `edge → main` (bring RV-284
onto main) → build+admit a `close_target` candidate on the new main base →
`dispatch sync --integrate --trunk refs/heads/main` (pure-ref advance, main
checked out nowhere → no dirty-checkout refusal). Reconcile leftovers authored on
a `main` worktree; primary stayed on `edge`.

Two traps hit and recorded to memory:
- **F-1 repair stranded on the review candidate.** The `estimate_pin_is_orchestrator`
  fix-now was committed on `candidate/222/review-001` (3c456029e), never folded
  into `review/222`. The `close_target` sourced `review/222`, so integrate landed
  main WITHOUT it — caught by the conformance re-check (`src/main.rs` undelivered).
  Fixed by cherry-picking 3c456029e onto trunk at close (lighter than the pre-FF /
  journal-fold routes).
- **Close-landing worktree needs the derived assets + runtime state.** A plain
  `git worktree add <path> main` skips `.worktreeinclude` (web/map/dist →
  RustEmbed build failure) and runtime phase state (`.doctrine/state/slice/222` →
  rollup reads untracked). Copy both from the primary tree.

Audit-surface split: the audit ran on the primary tree, blind to the candidate's
authored `.doctrine/` state, so its brief flagged items already landed via the
bundle (F-8(2) §1/§2, F-8(3) dispositions). Only genuine leftovers were authored.
