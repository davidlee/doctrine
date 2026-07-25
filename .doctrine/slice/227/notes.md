# Notes SL-227: Library read surface and minimal projection

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-25 · reconcile DONE (RV-302), pre-integrate · edge @ 137143fb

### Produced
- Impl bundle `review/227` (d32f9100): PHASE-01 publication engine additions
  (additive), PHASE-02 library veneer + 73-entry full-complement manifest,
  PHASE-03 minimal-projection install flip. Evidence refs: `dispatch/227`,
  `phase/227-{01,02,03}`. Landed via claude-arm `/dispatch`.
- Audit ledger **RV-302** — 10 findings, 0 blockers. All 10 verified/terminal.
  `## Reconciliation Outcome` recorded; `## Post-verification amendment` records
  F-6/F-7/F-8 fix-now.
- **ISS-238** (edge `f53fc81a`): `git_stdin` pipe-deadlock fix (scoped-thread
  stdin drain) — candidate-create on a large tree deadlocked via `check-attr`.
  Still OPEN: needs edge→main promotion + formal `backlog` resolve.
- **Fix-now candidate `cand-227-fix-001`** (`refs/heads/candidate/227/fix-001`,
  tip `30e538be`): F-6/F-7/F-8 hardening (asset_source disk-source helpers;
  install crux gate + publication VT-3 delegate; VT-6 non-vacuous rewrite). Green,
  clippy-clean, full suite 3850. Admitted `review_surface` (RV-302).
- **Reconcile edits (edge `137143fb`):** design.md §5.1/§5.2 (F-1), §9 (F-2), D8
  (F-3); selector registry full-correct (see Learned).
- Follow-up backlog: **IMP-312** (now F-10 only — test-fidelity); **IMP-313** (F-9).
- **ISS-226** annotated with the universal-UNATTRIBUTABLE dispatch-conclude datum (F-4).

### Learned
- **Auditing a dispatch-concluded slice:** the coord's authored corrections
  (selectors, plan.toml VT `test_file`) live on `review/NNN` / `dispatch/NNN`,
  NOT the primary tree — they land at close via `dispatch sync --integrate`
  (which projects `.doctrine/**`, FF+CAS, clobber-guarded via
  `--allow-corpus-clobber` — NOT code-only). So `slice conformance` on the
  primary tree reads a STALE registry. Corollary: reconcile edits made on edge to
  `.doctrine/slice/NNN` DIVERGE from what integration carries from `review/NNN` —
  edge leads, main follows; reconcile via edge↔main after integrate.
- **Verified RV findings are TERMINAL (ADR-007)** — no verb transitions out of
  `verified` (`src/review.rs:703-728`). A post-verification decision change
  (deferral→fix-now) canNOT reopen the finding; record it as a prose amendment on
  the RV `.md` (disposition stays the audit-time record). [memory candidate]
- **`candidate admit` accepts a descendant-of-recorded-merge tip** — a fix-now
  commit on top of the recorded merge is a first-class flow (I3 ancestor-check,
  `dispatch.rs:2099`). The `candidate status` **DRIFT** flag advises "supersede",
  but admit handles it — do NOT supersede (that re-runs the merge and drops the
  on-top commit). [memory candidate]
- **Census auto-merge gotcha:** both merge sides bumped the same
  `assert_eq!(visible.len(), N)` counter to the same value → git auto-merged to a
  WRONG total; the merged tree needs base+Δ_ours+Δ_theirs (52→53, both `graph` +
  `library`). Verify such counters empirically post-merge. [memory candidate]
- **NF-002 no-write / crux reachability** — F-8/F-7 NOW FIXED on the candidate
  (VT-6 non-vacuous; gate reads disk-source). The RV Synthesis "Standing risks"
  bullets are superseded by the amendment.
- **Independent green-verification of an evidence ref** needs `web/map/dist/`
  sourced from primary tree; `test_support::…existing_executable` false-reds in a
  from-scratch worktree build. Known/recorded gotchas.

### Open
- **CLOSE integration (blocked on user Q1/Q2 — see `handover.md`):** create+admit
  a `close_target` candidate CHAINED from `candidate/227/fix-001` (carries the
  fix-now), then `dispatch sync --integrate --trunk main` (+`--edge`?). Then
  `slice status 227 done`; resolve backlog origin.
- **ISS-238**: edge→main promotion + `backlog edit ISS-238 --status resolved`.
- **RFC-011 case-notes** owed (appended this session — see `.doctrine/rfc/011/`).
- Deferred by design (NOT regressions): SPEC-009 FR-009/FR-010; SPEC-026 REQ-375
  ×2 (need >1 adapter). Already reconciled in slice-227.md scope (X-F6).
- IMP-312 (F-10), IMP-313 await scheduling.
