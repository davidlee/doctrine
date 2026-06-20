# Implementation Plan SL-113: Shared entity mutation seam over atomic write

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three sequential phases: harden the leaf seam, migrate all call sites onto it,
then lock it with a clippy guard. Each phase is a strict prerequisite of the next.

## Sequencing & Rationale

**PHASE-01 must come first** — it changes the `write_atomic` function that every
subsequent migration depends on. The behaviour-preservation gate applies: the
existing VT-1 test must stay green unchanged. The new VT-2 concurrent test proves
the `AtomicU64` counter prevents intra-process temp-file collision.

**PHASE-02 is the bulk mechanical work** — 23 authored call sites across 12 files
(design §5.3, oracle-reconciled to HEAD on 2026-06-20), all the same transform.
The two non-standard error-wrapper sites (relation.rs, map_server) need adaptation
but no semantic change. The shared `dep_seq.rs` cores (and `facet_write.rs`
`edit_in_place`) are the highest-leverage sites — they alone cover ~7 entity
kinds. The count is a guide, not the gate: PHASE-03's clippy guard is the oracle
(design §5.3/R3), so the phase exits on a guard dry-run, not a tally. Existing
test suites (VT-3 unit, VT-4 E2E) are the proof the migration is correct.

**PHASE-03 is the lock** — it installs the clippy guard that makes the migration
permanent. It depends on PHASE-02 being complete: if any bare `std::fs::write`
remains on an authored entity in production code, `just check` will fail. The
gate runs clippy bins/lib only (no `--all-targets`), so `#[cfg(test)]` modules are
not linted — the test-fixture `fs::write` calls need no annotation. The **8**
runtime/derived production exceptions (7 files — fsutil.rs:63 seam, state.rs:409,
ledger.rs:408, worktree.rs:1845, install.rs:586, skills.rs:637, corpus.rs:403/406)
carry documented `#[allow]` (design §5.4/D3). `#[allow]`, **not** `#[expect]`:
`#[allow]` is dormant-safe (silent when the lint is not configured, so it never
breaks PHASE-01/02 commits) and survives call-graph drift; `#[expect]` would emit
`unfulfilled_lint_expectations` before the rule lands and is brittle after.

The phases are serial — no file-disjoint parallelism is possible because each
phase changes files that subsequent phases verify.

## Notes

- The scope's estimate of "~22 call sites" was from the audit; the design
  reconciled it against the code (§5.3) and the `/plan`-review oracle probe
  (2026-06-20) confirmed **23 authored sites across 12 files** at HEAD. The probe
  also surfaced `facet_write.rs:153` (the `edit_in_place` shared core), missed by
  the at-design sweep — added to §5.3. `revision.rs:888` IS an authored target
  (its status path funnels through `dep_seq`, but the row-append write at :888 is
  a direct site); an earlier draft wrongly stated revision.rs had none.
- The runtime/derived exclusion set is **8 sites / 7 files** (design §5.4), not
  two — every production `fs::write` trips the global guard, so each non-authored
  site needs its own `#[allow]`.
- Error display format changes in relation.rs and map_server/routes.rs are
  cosmetic — no test assertions depend on error message strings. Documented in
  VT-6, not a risk.
- The transform is behaviour-preserving for doctrine's normal authored files
  (git-tracked `0644` TOML/MD), but rename-replace is not bit-for-bit equivalent
  to in-place `fs::write` under chmod/metadata edge cases (new inode; mode/ACL/
  xattrs/hardlinks not carried). Immaterial here per design E3 (no doctrine file
  carries special mode/ACL, and write_atomic already rename-replaces authored
  files today) — noted so PHASE-02 isn't read as bit-identical.
- The clippy guard manual verification (VH-1) is a one-shot human gate. It proves
  the guard works once; `just check` prevents backsliding permanently.
