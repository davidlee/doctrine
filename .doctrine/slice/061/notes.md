# Notes SL-061: Rewire /code-review and /inquisition onto the RV review ledger

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## 2026-06-14 - external design inquisition and plan lock

- Ran the current prose `/inquisition` flow against SL-061 design + scope under
  ADR-007; recorded the findings in gitignored `inquisition.md`.
- Integrated three findings before lock: preserve `/audit` anti-escape +
  phase-sheet harvest mechanics in the shared extraction; make the ledger/prose
  trigger operational so `/code-review` cannot route around RV for durable
  reviews; remove stale scope language that still treated the inquisition facet
  as open.
- Advanced lifecycle `design -> plan -> ready`, authored `plan.toml` / `plan.md`,
  and materialised PHASE-01..PHASE-04 runtime sheets.
- Verification run: `doctrine slice show SL-061`, `doctrine slice phases 61`,
  and `git diff --check` for the touched SL-061 authored docs. No `just gate`
  run yet; no production code has been modified in this unit.
- Uncommitted work remains in SL-061 authored docs/plan/status/notes. Unrelated
  dirty workspace entries existed during the pass and were left untouched.

## 2026-06-14 - dispatch execution + reconciliation audit (RV-018)

- Drove all 4 phases via `/dispatch` (sole-writer funnel): P01 keystone alone,
  P02+P03 file-disjoint concurrent batch, P04 inline (authoring/smoke). Commits:
  `bc5e76f` (P01), `1b73a65` (P02+P03), `fcdfa60` (P04). Each batch re-anchored
  onto a moved coordination HEAD on a disjointness proof — heavy concurrent
  `main` (SL-057 close, SL-060 dep/seq, SL-062 authoring) never commingled.
- Funnel note: `git diff` is rtk-stat-proxied even under `RTK_DISABLE=1` in the
  Bash-hook context — imported deltas via `git checkout <fork> -- <paths>` (+ `git
  rm` for P02's `plugins/review` deletion), staged only own paths, committed
  without `-a`.
- INV-3 confirmed by dogfood: this audit ran on the refactored `/audit` + shipped
  `review-ledger.md`. Verb surface backs all three consumers; facet enum /
  `src/review.rs` untouched (INV-2); marketplace integrity clean.
- RV-018 (reconciliation) — 3 findings, all terminal:
  - F-1 (blocker/fix-now): P04 `doctrine claude install` self-appended a too-broad
    `.doctrine/agents/*` gitignore; wrongly committed, RED-ing the worktree
    classifier test + swallowing authored `AGENTS.md`. Reverted (`1037154`),
    derived `dispatch-worker.md` removed, gate green. Close-gate teeth worked.
  - F-2 (minor/follow-up): upstream SL-056 install-gitignore gap -> **ISS-012**
    (narrow ignore + classify in `DERIVED_RUNTIME`). Memory:
    `mem.pattern.distribution.claude-install-agents-gitignore-too-broad`.
  - F-3 (nit/aligned): "zero production src except src/skills.rs" wording
    undercounts the P01 `src/install.rs` belt test; both test-tier, invariant holds.
- Follow-ups minted: IMP-059 (cross-corpus harvest DRY, D6), IMP-060 (`/handover`
  relocation), ISS-012 (install-gitignore fix). Ready for `/close`.
