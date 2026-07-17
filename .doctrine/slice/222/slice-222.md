# Ledgered estimate claims

## Context

RFC-020 Phase 2 — the estimate half of the ledgered-facet-claims transition.
SL-220 (Phase 1) moved absolute `[value]` facets into the comparison ledger as
`form = anchor` claims under the T3 epistemic authority ladder (pin > human
claim > comparison projection > agent claim > migrated claim > default), with
REV-024 dissolving REV-022's anchors-win posture. SL-219 gave the estimate
domain its comparison machinery (est-domain compile/project, cost projection
feeding `est_cost`). Both prerequisites are done.

The `[estimate]` facet is now the last constitutional float: a hand-typed
range that outranks the calibration evidence, with ~90% of the corpus's
estimates unattributed agent guesses (IMP-290's census applies equally here).
Estimates are the *more* dangerous half of the provenance gap — `est_cost` is
the divisor of `value_dim`, so a guessed estimate scales the whole ranking.
SL-219's resolution ladder (design D2: authored > projected > bare anchor)
deliberately kept authored-on-top pending this phase; this slice performs the
flip for the estimate domain.

By RFC-020 T2 (the one-judgement-interface invariant), nothing in the row
schema, ladder, supersession machinery, or resolution seam is domain-specific
— this phase should be cheap precisely because SL-220 preserved that
invariant. Domains differ only in payload type, admissibility predicate, and
constraint compiler. IDE-013's estimate half dissolves here.

## Scope & Objectives

Strictly additive until the migration census passes; same purity posture as
SL-219/220 — pure over `(ledger, authored facets, statuses, config)`, disk at
the scan seam.

- **Estimate anchor payload.** The estimate-domain `form = anchor` claim
  carries the existing facet shape as payload: range `[lower, upper]`, skew β,
  unit, rater-stated range confidence% (RV-275 F-4: claim *content*, not
  system confidence — its `est_cost`-collapse force is unchanged). Rides the
  SL-220 row machinery (supersession, tombstones, within-session revision,
  findings) unchanged.
- **Est-domain claim resolution.** Competing estimate claims per item resolve
  by the T3 ladder through the same claim-resolution pass SL-220 built,
  feeding the est-domain `AnchorMap` (the SL-219 D-NF seam). Agent-authored
  estimates become priors below comparison projection; SL-219's
  authored-on-top D2 ladder is superseded by tiered claim resolution.
- **`est_cost` resolution through claims.** The scoring feed consumes the
  claim-resolved cost; the collapse semantics (range + skew + confidence →
  scalar, percentile framing) are unchanged — only the *source* precedence
  moves. Coupling honesty holds: `prefer-first`'s weighted inequality
  compiles over current costs (SL-219), so cost-source motion must not
  silently invalidate value-domain compilations.
- **Verb re-plumbing.** `estimate set` appends an anchor claim;
  `estimate clear` appends a tombstone; correction is supersession, never
  in-place edit. Pin admission for the estimate domain rides the same
  operator-gated contract as `value pin` (RV-275 F-5) — verb surface is
  design material (OQ-1).
- **Migration import** per the census contract — rerun of the SL-220 pattern
  as a throwaway `scripts/` Python script, NOT product surface: existing
  `[estimate]` facets import as `rater = migrated` claims (bottom of the
  ladder), observed-at = migration date, asserted-at honestly absent, git
  archaeology as optional `basis`. Idempotent, census-verified, dry-run,
  lossless rollback (each row cites its source facet). The pass physically
  strips `[estimate]` tables from entity TOMLs (SL-220 adjudication: removal,
  not read-path retirement). `[risk]` facets are untouched.
- **Rendering.** `show`/`explain` render `est_cost` as
  derived-with-provenance (tier, rater, date, range, judgement count);
  estimate-fit certainty is derived, never authored — rater confidence%
  renders as claim content, distinct (RV-275 F-4).
- **Regression evidence.** A Phase-0-style ranking diff (pre/post flip) is
  the accepted-evidence baseline for the deliberate behaviour change — the
  divisor position makes this flip's re-ranking wider than Phase 1's.
- **Governance.** Whether REV-024 already covers the estimate domain (T2
  uniformity) or the SL-219 D2 ladder needs its own dissolving REV against
  ADR-015 is settled at the design gate (OQ-4).

## Non-Goals

- **Hierarchy admissibility** (REQ/PRD/SPEC subjects, pedigree posture,
  cross-level capture gate) — Phase 3.
- **All cross-level arithmetic** — aggregation modes, cascade, container
  progress views (RV-275 F-2; gated on RFC-020 OQ-1 + ADR-018 REV).
- **Estimate feasible-region model / system confidence** — Phase E entry
  criterion (RV-260 F-5); rater-stated range confidence is claim content,
  not a system certainty.
- **Cross-domain yield ranking** — Phase E (IMP-287); estimate questions
  stay curator-surfaced (SL-217 D17).
- **Ratio/band row vocabulary** — voids the D8 marginal-exactness lemma;
  the phase admitting it revisits `determined`.
- **Abstention anchor-analogue** ("cannot estimate now", RFC-020 OQ-4) —
  deferred at SL-220 design; nothing here forecloses it.
- **Magnitude/range coarsening** — rejected for now (RFC-020 T5).

## Affected surface

- `src/comparison/**` — estimate anchor payload on the wire, est-domain
  claim resolution / `AnchorMap` builder extension.
- `src/priority/**` — `est_cost` resolution flip, demotion-knob application
  to the estimate domain, provenance rendering.
- `src/commands/facet.rs`, `src/main.rs` — `estimate set|clear` (± pin)
  re-plumbing.
- `src/estimate.rs`, `src/facet.rs`, `src/facet_write.rs` — `[estimate]`
  facet read/write path (retires at census; risk facet untouched).
- `src/commands/compare.rs` — capture admissibility for estimate anchor rows.
- `scripts/` — throwaway migration + ranking-diff scripts (SL-220 pattern).
- `.doctrine/adr/015/**` + revision machinery — if OQ-4 lands a REV.

## Risks, assumptions, open questions

- **R1 — divisor-wide re-ranking.** `est_cost` divides `value_dim`; the flip
  re-ranks corpora where agent estimates currently anchor, more broadly than
  Phase 1's numerator flip. Mitigation: the pre/post ranking diff is the
  accepted-evidence baseline; shared suites stay green unchanged (engine
  gate); deltas justified against the baseline, not waved through.
- **R2 — payload fidelity across the collapse.** The claim payload is the
  full range/skew/confidence shape while the constraint layer consumes the
  collapsed scalar (SL-219 D1: the latent is the operative scalar cost).
  Resolution must round-trip the payload losslessly even where consumption
  collapses it — migration census and supersession chains operate on the
  payload, not the scalar.
- **A1** *(confirmed at design)* — one ledger, one schema; estimate anchor
  claims are rows in the existing session files, no parallel store.
- **A2** *(resolved at design, E3)* — one generic claims fold parameterised
  by payload; the value instantiation is a behaviour-preserving refactor
  proven by the existing battery green unchanged.
- **OQ-1** *(resolved at design, E8)* — `estimate pin` + `pin --retire`
  added, SL-220 D13 gate verbatim (interactive-TTY + worker-refused class).
- **OQ-2** *(resolved at design, E4; operator-adjudicated)* — per-field mean
  over the winning-tier multiset; conflict interval over per-row operative
  costs; linearity lemma makes the two aggregations agree by construction.
- **OQ-3** *(resolved at design, E2)* — additive within v3, no version bump
  (SL-220 D1/D2 as designed).
- **OQ-4** *(resolved at design, E11)* — this slice authors its own REV
  against ADR-015 dissolving REV-023; REV-024 explicitly left it standing.
- **R3** *(surfaced at design, Q2/E7)* — post-strip the bare anchor's input
  (authored uppers) vanishes; `max_upper` re-sources from any-tier resolved
  claim uppers or the ISS-057 inversion returns corpus-wide.
- **R4** *(surfaced at design, Q4/E9)* — never-migrated corpora cross a
  disclosed behaviour cliff at the facet-path deletion; a scan-seam presence
  tripwire keeps the finding loud with a real remedy.

## Verification / closure intent

- Unit: est-domain claim resolution ladder (every tier pair, same-tier
  conflict over ranges, supersession chains, tombstones,
  migrated-below-agent ordering, deterministic output under row permutation)
  — the SL-220 battery re-run with estimate payloads.
- Migration census as hard VT: every `[estimate]` facet accounted for, zero
  silent provenance conversions, idempotent re-run, lossless rollback,
  confidence%/skew/unit round-trip bit-exact.
- Behaviour preservation where promised: corpora with no estimate claims and
  no `[estimate]` facets score bitwise-identically; shared suites green
  unchanged.
- Ranking-diff artifact (pre/post flip) recorded as audit evidence.
- E2E: `estimate set` → claim row → resolution → `est_cost` → visible in
  `explain` with provenance; human claim beats agent claim; migrated claim
  loses to projection; confidence% still parameterises the collapse.
- VA: RFC-020 T2 invariant holds — nothing estimate-specific in row schema,
  ladder, supersession, or resolution seam beyond payload, admissibility,
  and compiler.

## Summary

## Follow-Ups
