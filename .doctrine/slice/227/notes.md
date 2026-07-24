# Notes SL-227: Library read surface and minimal projection

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-25 · audit (RV-302) · review/227 @ d32f9100

### Produced
- Impl bundle `review/227` (d32f9100): PHASE-01 publication engine additions
  (additive), PHASE-02 library veneer + 73-entry full-complement manifest,
  PHASE-03 minimal-projection install flip. Evidence refs: `dispatch/227`,
  `phase/227-{01,02,03}`. Landed via claude-arm `/dispatch`.
- Audit ledger **RV-302** — 10 findings, 0 blockers, reconcile-ready.
  Independent verification: `cargo test --bin doctrine` green on review/227,
  `clippy --workspace` clean; external adversarial pass by codex/GPT-5.5.
- Follow-up backlog: **IMP-312** (test-fidelity hardening — F-6/F-7/F-8/F-10),
  **IMP-313** (latent `library tree` prefix-collision — F-9).
- **ISS-226** annotated with the universal-UNATTRIBUTABLE dispatch-conclude datum (F-4).

### Learned
- **Auditing a dispatch-concluded slice:** the coord's authored corrections
  (selectors, plan.toml VT `test_file`) live on `review/NNN` / `dispatch/NNN`,
  NOT the primary tree — they land at close via `dispatch sync --integrate`. So
  `slice conformance` run on the primary tree reads a STALE registry and
  mis-reports the corrected selectors. Assess conformance against the projected
  registry (`git show review/NNN:.doctrine/slice/NNN/slice-NNN.toml`).
- **Independent green-verification of an evidence ref** needs a worktree with the
  gitignored derived `web/map/dist/` sourced from the primary tree (else
  map_server fails to compile); `test_support::…existing_executable` also
  false-reds in a from-scratch worktree build. Both are known/recorded gotchas.
- **NF-002 no-write** currently rests on structural read-only-ness (codex-verified
  no write path), NOT on VT-6 (vacuous — F-8). The crux reachability gate is
  robust at close (fresh build) but not the incremental dev loop (F-7).

### Open
- Reconcile (→ /reconcile, see RV-302 Reconciliation Brief): design.md §5.1/§5.2
  (F-1), §9 flip list (F-2), §5.3/D8 seed-gate decision (F-3); selector registry
  `rm src/main.rs` + `add src/commands/{guard,mod}.rs` (F-5).
- Deferred by design (NOT regressions): SPEC-009 FR-009/FR-010; SPEC-026 REQ-375
  ×2 (need >1 adapter). Already reconciled in slice-227.md scope (X-F6).
- IMP-312, IMP-313 await scheduling.
