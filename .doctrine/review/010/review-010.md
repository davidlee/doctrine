# Review RV-010 — reconciliation of SL-050

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation of SL-050 (priority surface efficiency + conceptual-precision
cleanup) against its locked `design.md` (D1–D5, §1 seam shape, §2 finding table),
`slice-050.md` scope (the F6 carve-out + the no-policy-bump-beyond-`explain`
non-goal satisfied by D4), and governance (ADR-001 layering, ADR-004
outbound-only, REQ-072/077/094, NF-001).

Source under audit — three SL-050 commits, reviewed in isolation (main is
interleaved with concurrent SL-051/053):
`176cc0e` (PHASE-01 F2+F6), `fe1185e` (PHASE-02+03 F1+F3), `7641f44` (PHASE-04
F4+F5+F7+D4).

Lines of attack:
1. **Conformance of the three plan deviations** the handover flagged — 3 (not
   "keep thin") zero-caller wrappers deleted; 4 (not 5) `dead_code` suppressions
   removed; survey tie-break basis; one transitive walk in `explain`. Each
   verified clean (see Synthesis).
2. **Behaviour-preservation** (VT-2/VT-4): real-id goldens byte-identical; only
   the missing-id path (F6) and `explain` order-line (F5) change. `just check`
   green: 1044 lib + 40 suites, 0 failed; `cargo clippy` zero warnings.
3. **Test-quality erosion** from the F7 dangling-record drop — are the rewritten
   behavioural assertions genuine witnesses or vacuous?
4. **Policy-version coarseness** (D4) — one shared constant bumps the whole
   stamped set, false staleness for byte-identical survey/next/blockers.

Prior `/code-review` (this session) returned **solid**; this ledger dispositions
its two substantive notes (weak drift test; policy coarseness) plus the
conformance confirmations.

## Synthesis

**Verdict: audit-ready, no blocker.** SL-050 reconciles cleanly against its locked
design and governance. The slice is a pure-refactor cleanup (net −229 source
lines) whose central proof is the byte-identical real-id golden set; that gate
holds. `just check` green on fully-landed `main` after a shared-target fingerprint
clear: 1044 lib tests + 40 suites, 0 failed, 1 ignored; `cargo clippy` zero
warnings. The five `dead_code` suppressions the review targeted are gone, not
relocated.

**Conformance (F-4, aligned).** The three handover-flagged plan deviations are all
correct moves, not drift:
- The design's "keep thin delegate" wording (EX-1) assumed callers the `_from`
  split removed; with zero non-test callers, the no-dead-code clippy gate *forces*
  deletion of the 3 root wrappers (`build_relation_graph`, `relation_graph::render`,
  `surface::actionability_block`). `inspect` is kept test-only under
  `cfg_attr(not(test), expect(dead_code))`; `priority::graph::build` is kept (4 live
  standalone callers).
- "5 suppressions" was a plan miscount — `OrderContrib`/`seq_rank` were
  live-rendered, never suppressed; the 4 real suppressions (`Fallback`, `Dangling`
  struct + field, `ref_overlays`) all retired.
- F4's second `blocked_by_transitive` walk dies structurally with the F5
  `order_contrib` drop; exactly one walk remains in `explain` (`surface.rs:253`).

**Standing tradeoffs consciously accepted.**
- *F-1 (tolerated)* — `free_text_outbound_target_produces_no_edge` is a weak
  witness (consequence==0 for two independent reasons). It is the strongest
  observable left after F7 deliberately dropped the ref-overlay handles from
  `PriorityGraph`; the dep/seq branch is strongly witnessed by the sibling
  vec-equality test, and the consequence pair (positive `non_backlog…==1` +
  negative free-text `==0`) brackets the ref branch differentially. Strengthening
  it would mean re-exposing handles F7 intentionally removed.
- *F-2 (aligned)* — D4's single shared `PRIORITY_POLICY_VERSION` bumps the whole
  stamped set to `v2`, so survey/next/blockers `--json` carry `v2` with
  byte-identical data. Design D4 names this blast radius explicitly and REQ-094/
  NF-001 treat any versioned-envelope field change as a legitimate stamp trigger.
  Per-surface granularity would be new capability, not a slice defect.
- *F-3 (aligned)* — the survey tie-break resolves on `EntityKey` derived `Ord`
  `(prefix:&str, id)`, not a `canonical()` string compare as the §2 prose implies.
  The basis predates SL-050 (pre-F3 code used the same comparator), diverges from
  canonical-string order only at id≥1000 (cosmetic, unreachable for the corpus),
  and REQ-077 determinism holds.

**Governance reconciliation.** ADR-001 honoured — the single scan lives at the
command layer (`run_inspect`); `relation_graph` and `priority` still never call
each other (`priority` imports `ScannedEntity`/`EntityKey` downward only). ADR-004
untouched (consequence stays derived). REQ-072 tightened (one fewer reason kind;
renderer still only formats). No `design.md` correction needed — the only loose
spots are prose (EX-1 "keep thin", F3 "canonical-id"), both dispositioned aligned,
neither worth a back-edit against a locked design.

Handoff to `/close`: ledger resolved, no blocker, story coherent.
