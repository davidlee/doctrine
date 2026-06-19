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

**PHASE-02 is the bulk mechanical work** — 18 call sites across 9 files, all the
same transform. The two non-standard error-wrapper sites (relation.rs, map_server)
need adaptation but no semantic change. The shared dep_seq.rs cores are the
highest-leverage sites — they alone cover ~7 entity kinds. Existing test suites
(VT-3 unit, VT-4 E2E) are the proof the migration is correct.

**PHASE-03 is the lock** — it installs the clippy guard that makes the migration
permanent. It depends on PHASE-02 being complete: if any bare `std::fs::write`
remains in production code, `just check` will fail. The guard gates to non-test
via a crate-root inner attribute so the ~50 test-fixture `fs::write` calls are
unaffected. Two production exceptions (fsutil.rs write_atomic itself, ledger.rs
journal writes) carry documented `#[expect]`.

The phases are serial — no file-disjoint parallelism is possible because each
phase changes files that subsequent phases verify.

## Notes

- The scope's estimate of "~22 call sites" was from the audit; the design
  reconciled this against the actual code at current HEAD to 18 production sites
  (the difference: revision.rs had none, the audit counted some test sites).
- Error display format changes in relation.rs and map_server/routes.rs are
  cosmetic — no test assertions depend on error message strings. Documented in
  VT-6, not a risk.
- The clippy guard manual verification (VH-1) is a one-shot human gate. It proves
  the guard works once; `just check` prevents backsliding permanently.
