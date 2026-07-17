# Review RV-277 — reconciliation of SL-220

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed:** candidate interaction branch `candidate/220/review-001`
(cand-220-review-001, tip 5f1833c8 = refs/heads/main 2efcb52e + impl bundle
review/220 25421a7e), per the dispatched-slice rule — `review/*` and
`phase/220-01..06` are immutable evidence refs. Corpus/migration evidence
(PHASE-07) reviewed on edge (aa0c764e, 613e0141, fee25c7c, b2e0cd05); all
corpus verbs via the preserved v3 binary (.dispatch/doctrine-v3-sl220).

Lines of attack:

1. **Conformance algebra** — `slice conformance 220`: 21 undeclared / 1
   undelivered / 18 conformant. Reconcile each cell against the phase-sheet
   adjudications (9-fold show re-source, class-b golden churn, layering
   ratchet, PHASE-02 mechanical fallout); registry promotion vs scope creep.
2. **Suites + gate** — full `cargo test` + `just gate` on the candidate
   surface; behaviour-preservation classes a/b/c (design §8.5).
3. **Migration census (VH-1)** — 185/185/0/0 reconciliation, strip
   verification, idempotency no-op, post-migration scoring neutrality
   (all-zero diff vs pre-migration snapshot).
4. **REV-024 / canon coherence (VH-2)** — status done·approved; ADR-015 T3
   ladder + SPEC-020 REQ-278/279/280/286 + PRD-014 tell the shipped
   resolver's story (design D12, §8.10 VA).
5. **Accepted deviations** — ReasonKind::ValueUnmigratedFacet vs design's
   ValueFacetUnmigrated (NF-001 tripwire); hoisted-anchor renders as
   projected; demotion disclosure satisfied as-is; each needs a design
   append-note or explicit toleration.
6. **verify-vt attribution** — all VTs mechanical-fail from the primary tree
   (code pre-integration); confirm keyword presence on the candidate surface.
7. **RFC-020 T2 invariant (§8.10 VA)** — nothing value-specific in row
   schema, tier machinery, supersession, or the resolution seam.

## Synthesis

SL-220 delivers what the design promised, with every divergence ledgered and
adjudicated rather than silent. The audit ran against the candidate
interaction branch `candidate/220/review-001` (admitted at f8e7ca38 = base
main 2efcb52e + impl bundle review/220 25421a7e + one audit doc repair);
corpus/migration evidence reviewed on edge.

**Closure story.**

- **Mechanics green.** Full `cargo test` on the candidate: every suite ok,
  zero failures; `just gate` clean (clippy zero warnings). All 14 VTs across
  PHASE-02..06 PASS with clean attribution when run from the candidate
  surface (F-8 resolved the handover's UNATTRIBUTABLE question — that was a
  primary-tree artifact of the pre-integration window).
- **The flip is evidence-backed.** Phase 0 baseline (244 rows) → post-flip
  pre-migration snapshot → post-migration snapshot (246 rows) forms a clean
  three-point chain: the flip's re-ranking is confined to the designed
  class-b behaviour (agent-guess facets demoted from constitutional anchors,
  e.g. IMP-257 rank 3→150), and migration itself is scoring-neutral by
  construction (all-zero rank/score diff, 246/246). Golden churn was
  reviewed against the enumerated class-b list at PHASE-05 (VA-2) and
  matches design §3 classes a/b/c.
- **The census holds (VH-1, pending ratification).** 185 facets found, 185
  imported, 0 already-imported, 0 re-imported; per-entity table reconciles
  (13+135+12+5+20=185); 185 strips per-file verified; re-run is a no-op;
  interrupted-state rung-5 shadow semantics spot-checked live (RSK-014).
- **Canon moved with the code (VH-2, pending ratification).** REV-024
  done·approved; approval preceded the flip landing (D12 held at PHASE-05
  phase-plan); ADR-015/SPEC-020/PRD-014 now tell the T3-ladder story.
- **Conformance algebra fully explained.** 18 conformant, 21 undeclared —
  every one traceable to a ledgered mid-phase adjudication (PHASE-02
  mechanical fallout, PHASE-06 9-fold show re-source, class-b goldens,
  layering ratchet) — and 1 undelivered (config.rs docs), repaired at audit
  (f8e7ca38). Root cause worth keeping: belt design-target declarations made
  in the coord tree do not project into the primary selector registry.
- **RFC-020 T2 invariant (§8.10 VA) holds.** Row schema carries payload as
  optional per-domain columns; tier machinery, supersession, and the
  resolution seam are domain-generic (claims.rs value references are the
  domain instance and test fixtures, not baked-in schema); estimate payload
  columns slot in additively within v3.

**Standing risks / accepted tradeoffs.**

- **v3-binary interregnum**: the corpus is v3-only while the installed edge
  binary is pre-v3 — all corpus verbs via `.dispatch/doctrine-v3-sl220`
  until /close integrates to trunk. Bounded, known, ends at close.
- **Compared facet-bearing corpora lose facet anchoring permanently** (the
  flip's stated semantics, D6/RV-278 F-4) — disclosed by provenance
  rendering and the presence-based finding; re-assertion verbs exist.
- **TTY pin gate is a posture bar, not authentication** (D13, stated
  honestly in design) — the append-only attributed ledger is the backstop.
- Deviations tolerated with rationale: ValueUnmigratedFacet naming (NF-001
  tripwire), hoisted-anchor renders as projected, demotion disclosure
  satisfied by superset condition.

**Non-blocking environmental notes** (not slice findings): 43 GB stale
dispatch worktrees under .doctrine/state/dispatch await gc; IMP-293
(ratchet-red handoff) and the RFC-011 conclude-beat case-notes (IDE-028,
prepare-review ledger clobber) are captured follow-up work.

## Reconciliation Brief

### Per-slice (direct edit)

- **Selector registry (F-3, load-bearing)**: `slice selector add` with
  design-target intent for the adjudicated extension paths —
  src/backlog.rs, src/commands/cli.rs, src/comparison/compile.rs,
  src/comparison/project.rs, src/comparison/query.rs, src/concept_map.rs,
  src/governance.rs, src/lazyspec.rs, src/memory.rs, src/rec.rs,
  src/retrieve.rs, src/review.rs, src/revision.rs, src/slice.rs,
  src/spec.rs, src/value.rs, tests/e2e_compare_elicit.rs,
  tests/e2e_compare_inference.rs, tests/e2e_estimate_non_blocking.rs —
  then re-run `slice conformance 220` expecting those cells conformant.
  (.doctrine/adr/001/layering.toml and slice-220.toml need no selector:
  governance ratchet and registry self-churn, dispositioned F-9/F-3.)
- **design.md §2/§6 append-notes (F-3 mirror, F-5)**: (i) code-impact
  append-note recording the PHASE-02 mechanical-fallout absorption and the
  PHASE-06 show re-source seam set (9 kind modules + lazyspec/retrieve);
  (ii) §6 append-note: ReasonKind landed as `ValueUnmigratedFacet` (NF-001
  facet-symbol tripwire; semantics and D11 JSON token unchanged).

### Governance/spec (REV or owned verb)

- **RFC-020 (F-10)**: move Phase 0/1 rows to delivered-by-SL-220 (`rfc`
  surface / REV as the writer skill judges); RFC stays open for Phases 2–3.
- **SPEC-020 (F-10)**: additive documentary REQs — full claim-schema
  retention (wire v3 anchor-row schema: observed_at/basis/admission,
  per-domain payload columns) — additive only; must not contradict the
  REV-024-amended normative text.
- **PRD-011/SPEC-001 (F-10)**: descent prose pointing at the claims model —
  additive/documentary.

### Already repaired at audit (no reconcile action)

- config.rs demote-knob doc widening (F-4) — candidate commit f8e7ca38,
  admitted under RV-277; lands with /close's stage-2 integrate.

## Reconciliation Outcome

Reconcile pass complete (2026-07-17). Every brief item resolved; no
escalation to design.

### Direct edits applied (per-slice)

- **Selector registry** (F-3, load-bearing): 19 adjudicated-extension paths
  promoted to design-target (`slice selector add`, note citing RV-277 F-3).
  `slice conformance 220` now: **37 conformant**; residuals are exactly the
  dispositioned cells — `layering.toml` + `slice-220.toml` undeclared
  (governance ratchet / registry self-churn, F-9/F-3) and `config.rs`
  undelivered (audit-repaired on the candidate at f8e7ca38, clears at
  integrate).
- **design.md**: appended `## Post-audit append-notes (RV-277)` — the
  ValueUnmigratedFacet naming deviation (F-5), the PHASE-02 mechanical
  fallout and PHASE-06 show re-source code-impact extensions (F-3 mirror),
  and the F-4 config-docs repair pointer. Locked sections untouched.

### REVs completed (governance/spec)

- **REV-025** (`reconcile-sl-220`, originates from RFC-020): done —
  approved, applied, all three surfaced rows hand-landed:
  - **introduce FR-012 (REQ-336) → SPEC-020**: full claim-schema retention
    (observed_at/basis/admission + per-domain payload columns; additive
    within v3); statement/rationale/acceptance authored; status → active.
  - **modify PRD-011**: descent note — value input resolves through the
    ADR-015 T3 claim ladder; derived, never authored truth.
  - **modify SPEC-001**: descent note — value channel consumes the resolved
    (value, provenance) as a pure input; graph core stays policy-free.
  `spec validate`: corpus clean. Narrative in revision-025.md.
- **RFC-020** (not a REV-able target — direct edit recorded in REV-025's
  narrative): Phase 0 and Phase 1 implementation-path rows annotated
  *Delivered by SL-220*; RFC stays `open` for Phases 2–3.

### Already repaired at audit (no action here)

- config.rs demote-knob doc widening (F-4) — candidate commit f8e7ca38,
  admitted under RV-277; lands with /close's stage-2 integrate.

Handoff → /close.
