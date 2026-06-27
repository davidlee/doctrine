# Implementation Plan SL-165: Close-projection path for audit fix-now repairs

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-165 is a single surgical conformance fix: make `check_provenance`
(`src/dispatch.rs:805`) accept the `--source refs/heads/candidate/<N>/<label>`
that SPEC-022 REQ-317 already mandates for a `close_target`, while preserving
REQ-316's invariant (no candidate from unverified evidence) transitively. The
design (§5, §7) locks the mechanism — provenance model A, bounded recursion to a
Verified journaled root, close_target-scoped + audited-source exception,
`Created`-only status gate, and an INV-6 lineage binding (the source-side analog
of admit's I3). RV-175 (codex/GPT) hardened it with two blockers — name-trust
(→ INV-6) and an over-broad source role (→ INV-2/D3) — both integrated.

The split is along a behaviour boundary, not a code-size boundary: the change is
small, but a mechanical signature/read reshape that must keep the existing suites
green (the behaviour-preservation gate) is a genuinely separate risk from the new
accept/refuse logic that must change behaviour. Isolating them lets each verify
against the right oracle.

## Sequencing & Rationale

- **PHASE-01 (prep, behaviour-preserving).** Widen the gate signature, factor the
  journaled-evidence classifier into one predicate, move the candidates read ahead
  of the provenance call and thread the ledger. Nothing accepts or refuses
  differently — the proof is the *unchanged* `e2e_dispatch_candidate` /
  `e2e_dispatch_lifecycle` suites. Landing this first means PHASE-02 builds on a
  gate that already has the role + ledger in hand, and any green-suite breakage in
  PHASE-01 is unambiguously a refactor regression, not new logic.

- **PHASE-02 (capability).** The conformance fix proper: `trace_candidate_provenance`
  (count-exact row match, status/role/kind gates, bounded recursion to the journaled
  base-case which runs the *full* existing gate — F3) plus the INV-6 lineage binding
  in `candidate_create` post-`source_oid`-resolve. TDD against the gate accept/refuse
  matrix. Sequenced after the trace (by-name, EX-1 discipline) so the by-name chain
  proof and the by-content lineage binding stay at their design-specified seams
  (§5.1 — name trace pre-resolve, lineage binding post-resolve `:~916`).

- **PHASE-03 (anchor).** The headline regression: the IMP-188 reproduction made
  first-class — repair → close → integrate → `status done` with no manual fold or
  pre-FF dance. The red/green proof of the gated step belongs to PHASE-02 (its
  accept case is red before the gate, green after); PHASE-03 does not manufacture a
  second red — it is the *forward integration lock* over the whole sequence, with a
  distinct test home (`e2e_dispatch_lifecycle.rs`) and a distinct oracle (the
  closure-intent checklist), so it stands as its own phase rather than riding
  PHASE-02's gate matrix.

## Notes

- **Spec reconciliation is a reconcile obligation, not a phase.** REQ-316 ⊥ REQ-317
  is reconciled by authoring a Revision (REV) at reconcile (design D4 / Q3-A) — a
  normative-gate widening routes through governance + external review, never a quiet
  code-commit edit. The implementation phases conform to REQ-317 (the controlling
  intent); the REV lands at `/reconcile`. RFC-005 placement (close-projection
  hazard, H2-adjacent) is also noted at reconcile, not rewritten here.
- **Carried OQs.** OQ-1 depth budget settled here as a named constant `= 16`
  (PHASE-02 EX-4). OQ-2 exact REQ-316 wording → REV authoring. OQ-4
  hand-resolved-`Conflicted` source → v1 refuses (INV-3 limitation), revisit only if
  it bites.
- **No CLI surface change** — the `candidate create --role close_target --source
  refs/heads/candidate/<N>/<label>` invocation is unchanged; it simply stops being
  refused.
