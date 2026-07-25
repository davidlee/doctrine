# Notes SL-227: Library read surface and minimal projection

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-25 · **CLOSED (`done`)** · integrated to main @ 4072318f

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
- **Close integration:** `cand-227-close-001` (`4072318f`) chained from
  `cand-227-fix-001`, admitted `close_target` vs RV-302, projected
  `refs/heads/main` 3e083d4→4072318 (FF, pure-ref). Both close §3a verify gates
  pass. `check gate` green. `slice status 227 done` (edge `42087a4b`).
- **4 durable memories** recorded (edge `313f7af8`) — see Learned.

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
- **edge↔main reconciliation (NOT SL-227 work, but SL-227 caused it).** Integrate
  advanced main past edge, so `git fetch . edge:main` no longer FFs. main gained
  4 code commits; edge holds ~20 authored commits plus the ISS-238 fix. Resolve by
  merging main into edge, then promoting. Only real corpus delta on
  `slice-227.toml` is `status` + the stale `src/main.rs` selector (edge is
  correct); `design.md`/`notes.md`/`plan.toml` also differ, edge leads.
- **ISS-238**: still open; fix is edge-only (`f53fc81a`), NOT on main. Needs the
  promotion above, then `backlog edit ISS-238 --status resolved --resolution fixed`.
- **RV-302 outcome carries a wrong premise** (harmless): its selector scope note
  says integrate is code-only so review/227's corrections "do not propagate" —
  they did. Corrected in [[mem.fact.dispatch.integrate-projects-authored-corpus]];
  the RV is a closed audit record, left unedited pending operator call.
- Deferred by design (NOT regressions): SPEC-009 FR-009/FR-010; SPEC-026 REQ-375
  ×2 (need >1 adapter). Already reconciled in slice-227.md scope (X-F6).
- IMP-312 (F-10), IMP-313 await scheduling.
