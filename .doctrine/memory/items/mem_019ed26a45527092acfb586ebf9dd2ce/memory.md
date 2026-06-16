# EN criteria must name the honest dependency

When a plan.md prose section says two phases "could theoretically run in
parallel" but their EN criteria hard-gate on the earlier phase, the two
statements cannot both be true. Either the EN criteria are wrong (they invent a
structural dependency where only a soft narrative one exists) or the plan.md
prose is wrong.

Discovered during RV-047 inquisition on SL-080 plan: PHASE-02 and PHASE-03 both
had EN-1 gating on PHASE-01, but plan.md admitted they were file-disjoint with
no structural dependency. The EN criteria encoded a hard gate where only a soft
narrative preference existed.

## Rule

EN criteria must name the honest dependency:

- If the dependency is **structural** (phase B genuinely cannot start before
  phase A's output exists), name it as a hard gate.
- If the dependency is **soft** (narrative coherence, cross-reference
  convenience, implementation convenience), SAY SO in the EN text — e.g.
  "PHASE-01 complete so audit prose can cross-reference the written reconcile
  skill concretely."
- Or **drop the EN gate entirely** and note the advisory sequencing in plan.md
  prose.

## Test

Could a phase be executed *before* its EN prerequisite and still produce a
correct result (even if the prose is slightly less polished or needs a later
alignment pass)? If yes, the EN gate is lying.

## Provenance

RV-047 (inquisition on SL-080 plan), F-1.
