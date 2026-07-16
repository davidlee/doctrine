# Ledgered value claims

## Context

RFC-020 first cut — Phases 0 and 1 of its implementation path. Absolute
`[value]` facets are today constitutional anchors that outrank the comparison
evidence built to calibrate them (REV-022's anchors-win posture), while ~90%
of the corpus's authored values are unattributed agent guesses (IMP-290). This
slice moves absolute value assignment into the comparison ledger as
first-class evidence — dated, attributed, supersedable anchor claims — and
replaces binary anchors-win with the T3 epistemic authority ladder:

> pin > human claim > comparison projection > agent claim > migrated claim >
> default

Phase 0 (the baseline diagnostic) rides along: it is trivial, and its ranking
diff is the evidence base and regression baseline the resolver flip is judged
against.

Estimate claims are deliberately Phase 2 (a separate slice, after SL-219
lands); hierarchy admissibility is Phase 3. The design invariant that nothing
here may be value-specific (RFC-020 T2) is what keeps those phases cheap.

## Scope & Objectives

Strictly additive until the migration census passes; same purity posture as
SL-213/217 — pure over `(ledger, authored facets, statuses, config)`, disk at
the scan seam.

- **Phase 0 — baseline diagnostic** (IMP-290): a throwaway `scripts/` Python
  script (adjudicated at design) — copy the corpus to a temp root, zero the
  value coefficient, run `doctrine reports survey -p` against both roots,
  diff top-N. Captured as an evidence artifact before any resolver motion;
  re-run post-flip as the regression comparison. No product surface.
- **Anchor-claim rows.** `form = anchor` joins `order`/`ratio` in the wire
  vocabulary: single-subject absolute claim, payload a magnitude (f64, T5),
  mandatory `rater`, `asserted_at`/`observed_at` split, riding the existing
  supersession / tombstone / within-session-revision / findings machinery
  unchanged. Schema bump rides the version gate (v2 → v3), additive.
- **Claim resolution as `AnchorMap` builder.** A pure upstream pass: competing
  claims per item resolve by the T3 ladder to zero or one anchor plus
  findings. `AnchorMap` stays the sole anchor seam (SL-219 D-NF); the
  compile/project tiers are not reshaped.
- **The resolver flip** (IMP-290): agent-authored magnitudes become priors
  below projection; pins retain the constitutional override as deliberate,
  attributed, auditable rows. Same-tier conflict never resolves silently
  (RV-275 F-1) — deterministic, surfaced, no invented winner, no fall-through.
- **Pin admission as a contract** (RV-275 F-5): `value pin` mints pin-admitted
  claims only through an operator-gated path (the `worker_commit` gating
  precedent); authority is derived from provenance, never a row column.
- **Verb re-plumbing.** `value set` appends an anchor claim; `value clear`
  appends a tombstone; correction is supersession, never in-place edit.
- **Migration import** per the census contract, as a throwaway `scripts/`
  Python script — NOT product surface (adjudicated at design): existing
  `[value]` facets import as `rater = migrated` claims (bottom of the ladder,
  below attributed agent claims), observed-at = migration date, asserted-at
  honestly absent, git archaeology as optional `basis`. Idempotent,
  census-verified, dry-run mode, lossless rollback (each row cites its source
  facet). The same pass **physically strips** `[value]` tables from entity
  TOMLs (adjudicated: removal, not read-path retirement — dead
  authored-looking data is a standing lie). Phase 2 reruns the pattern for
  `[estimate]`.
- **Rendering.** `show`/`explain` render value as derived-with-provenance
  (tier, rater, date, bounds, judgement count); value-fit certainty is
  derived bounds, never authored (RV-275 F-4).
- **Config.** Demotion-knob policy (RFC-019 T7/D7 generalised) as
  claim-resolution config — e.g. agent claims excluded from determinacy.
- **Governance.** The REV against ADR-015 (dissolving REV-022's anchors-win
  into tiered claim resolution) rides this slice's design gate.

**Design-gate obligation** (RV-275 F-1/F-5, binding): the complete active-claim
algebra — row identity, closed derived-authority vocabulary, pin admission,
supersession, same-tier concurrency, lens participation, deterministic conflict
handling — designed and proven with permutation, duplicate-merge,
cross-session, conflicting-pin, and lens-isolation tests before implementation.

## Non-Goals

- **Estimate claims** — Phase 2, its own slice, sequenced after SL-219 lands
  (the estimate anchor payload rides the same interface by construction).
- **Hierarchy admissibility** (REQ/PRD/SPEC subjects, pedigree posture,
  cross-level capture gate) — Phase 3; subject generality is preserved, not
  exercised.
- **All cross-level arithmetic** — aggregation modes, cascade, container
  progress views (RV-275 F-2; gated on OQ-1 + ADR-018 REV).
- **Lens-tagged anchors feeding `value_dim`** — captured losslessly, inert
  until IDE-035; pooled fit consumes unlensed anchors only (RFC-019 T5).
- **Magnitude coarsening** — rejected for now (T5), possible later config gate.
- **Per-audience / as-of resolution surfaces; the REQ lifecycle (T7).**
- **System confidence for estimates** — Phase E entry criterion.

## Affected surface

- `src/comparison/**` — wire (`form = anchor`, v3 gate), resolve, new claim
  resolution / `AnchorMap` builder pass, store pipeline.
- `src/priority/graph.rs`, `src/priority/config.rs`, `src/priority/surface.rs`,
  `src/priority/render.rs`, `src/priority/view.rs` — resolver flip,
  `effective_raw_value` precedence, demotion knob, provenance rendering.
- `src/commands/facet.rs`, `src/main.rs` — `value set|pin|clear` re-plumbing.
- `scripts/` — throwaway migration + Phase 0 diagnostic Python scripts.
- `src/value.rs`, `src/facet.rs`, `src/facet_write.rs` — `[value]` facet
  read/write path (retires at census; estimate/risk facets untouched).
- `src/commands/compare.rs` — capture admissibility for anchor rows.
- `.doctrine/adr/015/**` + revision machinery — the REV.

## Risks, assumptions, open questions

- **R1 — anchor attachment is row-gated per compiled system**
  ([mem.fact.comparison.anchor-attachment-row-gated-per-system]): anchors
  attach only to entities present in ≥1 row of that compile. Post-flip, an
  item whose *only* evidence is a claim (no comparison rows) must still
  resolve to its claimed value — the claims pass must not inherit the silent
  drop. Design must state where claim-derived values enter for row-less items.
- **R2 — deliberate behaviour change.** The flip re-ranks corpora where agent
  facets currently anchor. Phase 0's diagnostic is the accepted-evidence
  baseline; shared-machinery suites stay green unchanged (engine gate), and
  ranking deltas are justified against the baseline, not waved through.
- **R3 — same-tier conflict semantics** *(adjudicated at design, 2026-07-16)*:
  the winning tier's active claims with distinct magnitudes resolve to their
  arithmetic **midpoint** as the point anchor (D8-safe); the disagreement
  interval renders as bounds; a loud finding + reprobe candidate fires;
  resolution is a superseding row. Uniform across tiers (a conflicted pin is
  a contested pin, named as such). Rationale: independent human assessments —
  the average likely beats either guess; don't break the graph pending
  adjudication. Deterministic and surfaced, never silent; no lower tier wins
  because a higher tier disagrees.
- **A1** — one ledger, one schema; anchor claims are rows in the existing
  session files, no parallel store.
- **A2** — SL-219 executes unchanged, before or in parallel; `AnchorMap` is
  the integration seam.
- **OQ-1** — ladder × lens composition (RFC-020 OQ-2): Phase 1 gate material.
- **OQ-2** — does abstention need an anchor-claim analogue ("cannot value
  now") as selector fodder (RFC-020 OQ-4)? Capture posture only.
- **OQ-3** *(resolved at design)* — no product verbs for migration or the
  Phase 0 diagnostic: both are throwaway Python scripts in `scripts/`;
  doctrine only parses the v3 anchor rows the migration emits.

## Verification / closure intent

- Design-gate algebra proven by the named test obligations (permutation,
  duplicate-merge, cross-session, conflicting-pin, lens-isolation) before
  implementation (RV-275 F-1/F-5).
- Unit: claim resolution ladder (every tier pair, same-tier conflict,
  supersession chains, tombstones, migrated-below-agent ordering,
  deterministic output under row permutation).
- Migration census as hard VT: every facet accounted for, zero silent
  provenance conversions, idempotent re-run, lossless rollback (row cites
  source facet).
- Behaviour preservation where promised: corpora with no anchor claims and no
  `[value]` facets score bitwise-identically; shared suites green unchanged.
- Phase 0 diagnostic artifact recorded as audit evidence.
- E2E: `value set` → claim row → resolution → visible in `explain` with
  provenance; pin overrides projection; human claim beats agent claim;
  migrated claim loses to projection.
- Governance: REV against ADR-015 approved before the resolver flip ships
  (design gate); VA — RFC-020 T2 invariant holds (nothing value-specific in
  row schema, ladder, supersession, or resolution seam).

## Summary

## Follow-Ups
