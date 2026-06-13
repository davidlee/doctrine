# IMP-032: Governance superseded_by carve-out — validate cross-check vs derived reciprocal (ADR-010 D4)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## CORRECTION (ADR-010 D4 — 2026-06-13)

**This item's original premise was wrong and is superseded.** It read
`superseded_by` as an ADR-004 violation to be *removed* ("derive it, don't store
it"). **ADR-010 D4 reclassified it:** `superseded_by` is the ADR-004 **§5
sanctioned reverse carve-out** — *kept, not removed*. Supersession flips the
predecessor to terminal `superseded` (its file is rewritten regardless), so the
verb co-writing the reverse adds zero marginal coupling and is the only honest
place a reader of the dead record finds its successor.

The honest remaining work is therefore **not** schema removal but a guard:

1. **A `validate` cross-check** — stored `superseded_by` agrees with the reciprocal
   derived from `supersedes` `in_edges`; report drift, never rewrite. **SHIPPED in
   SL-048 PHASE-05** (the `validate` corpus-edge walk + supersession cross-check) —
   the integrity half of this item is closed; only the verb-written half (below)
   remains, owned by [[IMP-006]].
2. **The carve-out becomes verb-written** (not hand-authored) once the
   transactional supersede verb exists — that verb is **[[IMP-006]]** (uniform
   lifecycle-transition verbs), not SL-048 (a gov-only build there = parallel
   implementation).

The stale analysis below is retained for history; do **not** act on its
"stop storing it / remove the field" conclusion.

---

## What (HISTORICAL — premise corrected above)

Governance kinds (ADR/POL/STD) store `superseded_by` as an authored
`Vec<String>` field (`src/governance.rs:160`; written on the adr/policy/standard
paths — `src/adr.rs:231`, `src/policy.rs:227`, `src/standard.rs:232`). It is the
**reciprocal** of `supersedes`: `A supersedes B ⟺ B superseded_by A`. Storing
both directions is a stored inbound/reverse field — the exact pattern **ADR-004**
forbids (relations outbound-only; reciprocity is derived).

No code derives or cross-checks it today — it is hand-maintained and inert
(parsed only for `show`).

## Why it matters

Surfaced during SL-046 design (the cross-kind relation reader). SL-046's reader
rule projects only the canonical **outbound** direction (`supersedes`) and
**derives** the reciprocal via cordage `in_edges`, deliberately **not** projecting
`superseded_by` as edges (projecting both would double-count the same fact in the
derived inbound view). So the stored field is already redundant to the reader.

This leaves a latent drift: a hand-maintained `superseded_by` that disagrees with
the derived inbound. The clean fix is to **stop storing it** and render it as
derived inbound everywhere (matching how spec `descends_from` already derives its
children view, and slice `supersedes` already stores outbound-only).

## Scope of the fix (NOT SL-046 — capture-side)

- Remove `superseded_by` from the governance authored schema + migrate existing
  TOML (a closure/edit-preserving migration).
- Render the reciprocal as derived inbound in `show` / `inspect`.
- Belongs with **SL-048** (structural cross-corpus relation edges) under the
  relation-governance ADR that slice needs. Asymmetric reciprocal only — `related`
  is genuinely symmetric and legitimately appears on both sides.

Related: [[SL-048]] (implements the cross-check) · [[IMP-006]] (the supersede
verb) · ADR-010 D4 (the reclassification) · ADR-004 §5 (the sanctioned carve-out)
· IMP-016 (cross-corpus relations).
