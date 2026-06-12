# Review RV-007 — reconciliation of SL-047

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-047 (cross-kind actionable survey/next/explain/blockers
CLI), 3 phases landed via /dispatch (cb03be6, 1402dc3, ae569e4). Reconciled
against `design.md` (incl. §9 validation list, §10 inquisition record), `plan.toml`
EX/VT, and SPEC-001/PRD-011/ADR-001/004/009/010.

Lines of attack:
- **The gate.** `just check` green? Behaviour-preservation suites (backlog_order,
  cordage) green unchanged, `backlog order` byte-identical?
- **Charge-bound facts from §10** (settled, not relitigated — verify they HELD):
  RV `Active`→eligible via derived_status (I); dep-blocking backlog-scoped (II);
  consequence = work/lineage label subset, excl. reviews/owning_slice (V); promoted
  via `resolution==Promoted` (VI); slice canary binds ADR-009 set (VII).
- **The three pre-authored audit flags** (notes.md §PHASE-03): narrowed dead-code
  expects; explain v1 fidelity (seq_rank/dep_level); the worker-raised, UNCONFIRMED
  cordage denylist failure — investigate, don't fold into SL-047.
- **Behaviour live**: do the four verbs + inspect block actually run on the real
  corpus and stamp policy_version?

## Synthesis

**Verdict: audit-ready.** No SL-047 source defect found. The gate (`just check`)
is green — clippy zero-warning, full test suite passing including the 13 priority +
9 inspect goldens. The behaviour-preservation contract held: backlog_order and
cordage main-suite green, inspect/backlog-order output byte-identical (the goldens
encode this). All four verbs run live on the real corpus: `survey`/`next` rank
cross-kind by consequence desc / id asc, `next` is actionable-only, `explain SL-047`
reads `started → Workable` with order/consequence reasons, `blockers` direct-only,
and `--json` stamps `policy_version = "priority.v1"`.

**Charge-bound facts (§10) verified held** via the passing VT goldens that were
written to assert exactly them — RV `Active`→eligible (Charge I), dep-blocking
backlog-scoped (II), consequence over the work/lineage label subset only (V),
promoted via `resolution==Promoted` (VI), slice canary against the ADR-009 set
(VII). The settled inquisition facts were not relitigated.

**Findings (3, all terminal, no blocker):**

- **F-1 (minor → follow-up, ISS-007).** The one genuine RED: `cargo test -p cordage
  --test denylist` fails on a whole-word `task` in `crates/cordage/README.md`
  (REQ-079 boundary). Pre-existing (dc120a7), disjoint from SL-047 source, and
  outside the green gate — `cargo test` runs only workspace default scope, so the
  cordage suite (needs `-p`) is never gated. The handover's "does not reproduce"
  was a stale test binary baking a removed dispatch-worktree `CARGO_MANIFEST_DIR`
  (root-resolution panic masking the real hit); a clean recompile surfaces it.
  Correctly kept OUT of SL-047 per scope discipline; captured as ISS-007, which
  also flags the deeper gate-coverage hole.
- **F-2 (nit → tolerated).** `ReasonKind::Fallback` carries a self-clearing
  `#[expect(dead_code)]` — §5.4 vocabulary completeness, renders, errors the moment
  an emitter appears. The `dangling`/`ref_overlays` expects that notes.md/handover
  flag-1 also named are already GONE at HEAD (now read), so flag 1 reduced to this
  single variant.
- **F-3 (minor → tolerated).** `explain` v1: `seq_rank=None` (design-sanctioned
  Option, surfaced instead via `evicted_seq_edges`) and `dep_level` as an
  agent-legible transitive-prereq proxy for the internal cordage level. REQ-072
  (reasons present) and D11 (full chain via explain) are met.

**Standing risk:** the gate does not exercise the cordage crate's own test suite —
a product-neutrality regression in cordage can land green. Tracked in ISS-007.

**Closure:** ledger resolved (`done · await=none`, 3 findings terminal), no blocker
at the close seam. Ready for `/close`.
