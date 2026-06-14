# Data-only phase that mutates corpus contents must re-run corpus-walk oracles / full gate

A data-only backfill phase must re-run the full gate; per-phase funnel verify before the data lands misses corpus-walk oracle drift.

**Symptom.** SL-060 PHASE-05 backfilled `[relationships]` into every slice TOML. A
corpus-walk oracle (`tests/e2e_relation_migration_storage.rs::slice_corpus_*`) read
that data and went RED — but only at audit, not during the phase. The dispatch
funnel had verified each *code* phase (P02–P04) green, and PHASE-05 was treated as
"data-only, no gate". The data was correct (`validate` clean); the test side was
stale, and nothing re-ran it after the corpus changed.

**Why.** Per-phase funnel verification runs *before* a phase's commit. For a code
phase that is sufficient. But a corpus-walk oracle's input is the corpus *data*, so
a pure-data phase changes what that oracle reads — and if the data phase skips the
gate, the drift is invisible until a fresh full-gate run. The handover's "gate green
at HEAD (data-only PHASE-05 since)" is exactly the trap: green was measured before
the data landed.

**How to apply.** Treat a data/backfill phase as gate-bearing whenever any oracle
walks the corpus it mutates: re-run `just gate` (not just the per-phase suites)
after the data lands, before declaring the slice gate-green or writing the audit
handover. At audit, never trust an upstream "gate green" claim for a slice whose
last phase was data-only — re-run it.

Related: [[mem.pattern.testing.migration-oracle-restates-not-derives-from-ssot]]
(the corpus oracle restates expected vocab independently — which is why it must be
updated by hand when the canonical shape changes, as in SL-060 §5.3/E9).
