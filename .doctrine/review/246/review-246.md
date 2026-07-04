# Review RV-246 — reconciliation of SL-202

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance self-audit. **Surface reviewed:** the fork branch
`SL-202-exec` (2 commits `a0f567cf`→`a380aace`), not yet landed — audited via
`git diff 4ddb5244..SL-202-exec` (the true fork delta; the merge-base, not the
diverged `edge..` tip which conflates edge's advance). Doctrine authored state
(ledger, slice status) resolves to the primary tree, so the parent-tree caveat
(§6) is satisfied without landing.

**What this audit probes:**

1. **Conformance** — does the git touch-set match the `design-target` selectors?
2. **VA-1 behaviour-preservation** — the load-bearing invariant. SL-202 extends
   the shared entity-engine hydration path (`Catalog::from_scanned`). Existing
   edge/backlink suites must stay green *unchanged*, and the TOML relation pass
   must be byte-equivalent in behaviour. The design's F-1 divergence (body-pass
   `UnvalidatedText` warns; TOML stays silent) must be airtight and scoped.
3. **Design fidelity** — INV-1 (one-directional dedup), INV-2 (UnvalidatedText →
   1 edge + 1 Warning), the `Raw("related")` label + `role:None` (ADR-016), the
   named constants (STD-001).
4. **Plan hygiene** — the phase sheets flagged EN-2's `seed_memory` extension as
   over-specified (VTs exercise pure `from_scanned` in-memory); confirm this is a
   benign plan-vs-implementation gap, not dropped coverage.

**Invariants held:** ADR-001 (leaf←engine←command, no cycles); STD-001 (no magic
strings); ADR-016 (Raw edge carries role:None); the behaviour-preservation gate
(AGENTS.md) — shared-machinery change proven by the existing suites staying green.

## Synthesis

**Verdict: clean. No blocker, no drift, no canon correction.** SL-202 delivers
ISS-214 — memory body `[[mem.…]]` wikilinks now render as first-class catalog
edges alongside TOML `[[relation]]` rows.

**Conformance is exact.** `slice conformance SL-202` reports 0 undeclared, 0
undelivered, 2 conformant (`src/catalog/hydrate.rs`, `src/memory.rs`) — the git
touch-set equals the `design-target` selectors. The coarse `scope-relevant`
fence also names `src/catalog/scan.rs`; it went untouched (the impure read landed
one seam up, in `read_catalog_record`), which is correct and does not register as
undelivered (scope-relevant selectors don't gate conformance).

**The behaviour-preservation invariant (the one that mattered) holds — F-1.**
SL-202 extends the shared `Catalog::from_scanned` hydration path, so the risk was
collateral change to TOML-relation edge behaviour. The diff-read (true fork delta
`4ddb5244..SL-202-exec`) shows the TOML loop's sole change is an additive,
populate-only `seen.insert` — no `continue`, no diagnostic mutation. The design's
F-1 divergence (a body-pass `UnvalidatedText` wikilink warns, where the TOML path
stays silent) lives entirely in the new body-pass match arm; `classify_target`
and the TOML `:341` diagnostic are byte-untouched. VT-4 pins the TOML-stays-silent
control as an executable assertion — the proof is a test, not prose. Full suite
3089/0 unchanged; gate exit 0; all 5 VTs (PHASE-01 VT-1, PHASE-02 VT-1..4) PASS.

**Plan hygiene — F-2, benign.** Plan EN-2 anticipated a `seed_memory` scan-level
test extension; the VTs correctly exercise the pure `from_scanned` in-memory,
making it unnecessary. Coverage is complete at the right altitude, not dropped.
Plan EN criteria are immutable historical artifacts — no rewrite.

**Standing notes (process, not SL-202 defects):**
- *`#[cfg_attr(not(test), expect(dead_code, reason=…))]` scaffolding.* PHASE-01's
  producer/consumer split left `MemoryCatalogRecord.body` transiently dead; the
  `not(test)` gate was required because VT-1 reads the field in the test build
  (a bare `#[expect]` there is unfulfilled → error). Self-cleaning: PHASE-02's
  production read unfulfilled it → clippy forced removal (done, T5). A reusable
  idiom for phase-split transiently-dead fields — harvested to memory below.
- *`verify-vt` UNATTRIBUTABLE while `in_progress`.* Attribution range is
  `code_start..code_end`; `code_end` stamps only at completion, so a mid-phase
  `verify-vt` reads UNATTRIBUTABLE by design. Momentarily confusing, but correct.
  Distinct from IMP-228 (keyword-pre-exists inert-PASS, closed/fixed). Logged to
  RFC-011 case-notes; not a defect, no backlog route.
- *Diverged-base diff trap.* `git diff edge..SL-202-exec` shows a spurious
  `check.rs` deletion — edge added `check plan` after the fork base. The true
  delta is `merge-base..tip`; conformance (boundary-OID based) was never fooled.

## Reconciliation Brief

**Nothing to reconcile.** Both findings dispositioned `aligned`/`verified`; no
finding touches design, spec, or governance.

### Per-slice (direct edit)
- None. Implementation matches `design.md` §5.4/§5.5 as written.

### Governance/spec (REV)
- None.

`/reconcile` confirms the clean audit and advances the slice; no write surface is opened.

## Reconciliation Outcome

**No-op.** Both findings (F-1 behaviour-preservation, F-2 plan EN-2 over-spec)
are `aligned`/`verified` with no write needed. The Reconciliation Brief is empty
— no per-slice edit, no governance/spec REV, no tolerated drift. Implementation
matches `design.md` §5.4/§5.5 as written; no authored-truth diverged.

Reconcile pass complete — handoff to /close.
