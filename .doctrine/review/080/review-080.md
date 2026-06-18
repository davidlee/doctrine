# Review RV-080 — reconciliation of SL-082

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-082 (dispose `doc/` as legacy heretical practice). Five phases implemented per `plan.toml` in serial.

**Lines of attack:**
1. Design §3.1: all 10 source citations (S1–S10) repointed correctly — verify each edit matches its row
2. Design §3.2: all 11 skill citations (K1–K11) repointed; no `doc/*` mental model survives
3. Design §3.3: install template citations (I1–I3) removed/updated; `reconcile/SKILL.md:20` caught during sweep
4. Design §3.4: memory record paths + prose (M1–M6) updated to real tech spec dirs
5. `doc/` deleted; `rg 'doc/'` sweep clean except 3 intentional residuals
6. `just check` green modulo SL-095 pre-existing failure; `doctrine install` propagates skills

**Invariants:** Every former `doc/` reference resolves to an existing entity or is intentionally removed. No `doc/*` mental model in skill prose. Memory paths resolve to extant tech spec dirs. Build and install clean.

## Synthesis

SL-082 implementation is complete and conforms to the design. All five phases executed in serial:

- **PHASE-01**: 10 source code references (S1–S10) in 6 Rust files repointed per design §3.1. Test expectations updated; all 1681 unit tests pass.
- **PHASE-02**: 11 skill references (K1–K11) in 8 skill files repointed per design §3.2. Additional `reconcile/SKILL.md` reference caught during sweep (F-2). `doctrine install` propagated all 30 skills cleanly.
- **PHASE-03**: 3 install template references (I1–I3) removed or updated. `doc/*` glossary section removed entirely.
- **PHASE-04**: 6 memory records (M1–M6) updated — 3 `paths` fields repointed to existing tech spec directories, 3 prose references updated. All target directories verified extant.
- **PHASE-05**: `doc/` directory removed (9 files, 2722 lines deleted). Final `rg` sweep confirms zero legacy `doc/` references across all surfaces except 3 intentional residuals: `src/install.rs` historical note (S8), `src/spec.rs` legacy-with-supersession marker (S10), and `retrieve-memory/SKILL.md` `file/doc/ADR` false positive.

**Standing risks:**
- SL-095 pre-existing test failure (`template_source_is_post_cut_shape_kind_specific`) remains — unrelated ADR template migration work.
- SL-021 PHASE-05 deferred — tech spec edges and draft→active flips remain incomplete. Content is rehomed; corpus is in an intermediate state but fully navigable.

**Tradeoffs accepted:**
- No `.gitignore` guard for `doc/` — removal is the only gate (D2). If `doc/` is accidentally recreated, the next `rg` sweep or CI catches it.
- Memory `paths` updates are scope-only (D4) — no verification flag reset; semantics unchanged.

## Reconciliation Brief

### Per-slice (direct edit)

None. All design→implementation gaps were caught and corrected during the sweep (F-2).

### Governance/spec (REV)

None. No governance or spec changes are required for SL-082 closure.

### Deferred work

- F-3: SL-021 PHASE-05 — resume or capture as backlog item for interactions edges, parent retrofit, draft→active flips, and capability-coverage audit.
